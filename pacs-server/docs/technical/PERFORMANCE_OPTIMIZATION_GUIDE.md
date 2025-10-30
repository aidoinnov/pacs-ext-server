# 성능 최적화 가이드

## 개요

이 문서는 PACS 서버의 성능을 최적화하기 위한 포괄적인 가이드입니다. 데이터베이스, 애플리케이션, 네트워크, 그리고 인프라스트럭처 레벨에서의 최적화 방법을 다룹니다.

## 성능 측정 지표

### 주요 지표

1. **처리량 (Throughput)**
   - RPS (Requests Per Second)
   - TPS (Transactions Per Second)
   - 데이터 처리량 (MB/s)

2. **응답 시간 (Latency)**
   - 평균 응답 시간
   - 95th percentile 응답 시간
   - 99th percentile 응답 시간

3. **리소스 사용률**
   - CPU 사용률
   - 메모리 사용률
   - 디스크 I/O
   - 네트워크 대역폭

4. **에러율**
   - HTTP 에러율
   - 데이터베이스 에러율
   - 타임아웃 에러율

## 데이터베이스 최적화

### PostgreSQL 최적화

#### 1. 설정 최적화

**postgresql.conf 설정**
```conf
# 메모리 설정
shared_buffers = 4GB                    # 시스템 RAM의 25%
effective_cache_size = 12GB             # 시스템 RAM의 75%
work_mem = 256MB                        # 복잡한 쿼리용
maintenance_work_mem = 1GB              # VACUUM, CREATE INDEX용

# 연결 설정
max_connections = 200
shared_preload_libraries = 'pg_stat_statements'

# 체크포인트 설정
checkpoint_completion_target = 0.9
wal_buffers = 64MB
checkpoint_timeout = 15min

# 쿼리 플래너
default_statistics_target = 100
random_page_cost = 1.1                  # SSD용
effective_io_concurrency = 200          # SSD용
```

#### 2. 인덱스 최적화

**기본 인덱스**
```sql
-- 마스크 테이블 인덱스
CREATE INDEX CONCURRENTLY idx_annotation_mask_group_id 
ON annotation_mask(mask_group_id);

CREATE INDEX CONCURRENTLY idx_annotation_mask_created_at 
ON annotation_mask(created_at);

CREATE INDEX CONCURRENTLY idx_annotation_mask_mime_type 
ON annotation_mask(mime_type);

-- 마스크 그룹 테이블 인덱스
CREATE INDEX CONCURRENTLY idx_annotation_mask_group_annotation_id 
ON annotation_mask_group(annotation_id);

CREATE INDEX CONCURRENTLY idx_annotation_mask_group_created_by 
ON annotation_mask_group(created_by);

-- 복합 인덱스
CREATE INDEX CONCURRENTLY idx_mask_group_user_created 
ON annotation_mask_group(created_by, created_at);

-- 부분 인덱스 (활성 레코드만)
CREATE INDEX CONCURRENTLY idx_mask_active 
ON annotation_mask(mask_group_id) 
WHERE deleted_at IS NULL;
```

**쿼리별 최적화 인덱스**
```sql
-- 마스크 통계 쿼리 최적화
CREATE INDEX CONCURRENTLY idx_mask_stats 
ON annotation_mask(mask_group_id, mime_type, label_name, file_size);

-- 마스크 목록 조회 최적화
CREATE INDEX CONCURRENTLY idx_mask_list 
ON annotation_mask(mask_group_id, created_at DESC) 
INCLUDE (id, file_path, file_size);
```

#### 3. 쿼리 최적화

**EXPLAIN ANALYZE 사용**
```sql
-- 쿼리 성능 분석
EXPLAIN (ANALYZE, BUFFERS, FORMAT JSON) 
SELECT * FROM annotation_mask 
WHERE mask_group_id = $1 
ORDER BY created_at DESC 
LIMIT 100;
```

**쿼리 최적화 예시**
```sql
-- 비효율적인 쿼리
SELECT * FROM annotation_mask 
WHERE mask_group_id IN (
    SELECT id FROM annotation_mask_group 
    WHERE annotation_id = $1
);

-- 최적화된 쿼리
SELECT am.* FROM annotation_mask am
JOIN annotation_mask_group amg ON am.mask_group_id = amg.id
WHERE amg.annotation_id = $1;
```

#### 4. 파티셔닝

**날짜별 파티셔닝**
```sql
-- 마스크 테이블 파티셔닝
CREATE TABLE annotation_mask (
    id SERIAL,
    mask_group_id INTEGER NOT NULL,
    file_path VARCHAR(500) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    -- 기타 컬럼들
) PARTITION BY RANGE (created_at);

-- 월별 파티션 생성
CREATE TABLE annotation_mask_2024_01 
PARTITION OF annotation_mask 
FOR VALUES FROM ('2024-01-01') TO ('2024-02-01');

CREATE TABLE annotation_mask_2024_02 
PARTITION OF annotation_mask 
FOR VALUES FROM ('2024-02-01') TO ('2024-03-01');
```

### Redis 캐싱

#### 1. 캐시 전략

**캐시 계층 구조**
```rust
// L1: 애플리케이션 메모리 캐시
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

struct MemoryCache {
    data: RwLock<HashMap<String, (String, Instant)>>,
    ttl: Duration,
}

// L2: Redis 캐시
use redis::{Client, Commands};

struct RedisCache {
    client: Client,
    ttl: Duration,
}
```

**캐시 키 전략**
```rust
// 캐시 키 생성
fn cache_key(prefix: &str, params: &[&str]) -> String {
    format!("{}:{}", prefix, params.join(":"))
}

// 사용 예시
let key = cache_key("mask_stats", &[&mask_group_id.to_string()]);
let key = cache_key("mask_list", &[&mask_group_id.to_string(), &offset.to_string()]);
```

#### 2. 캐시 무효화

**Write-Through 캐시**
```rust
async fn update_mask(&self, id: i32, data: UpdateMaskRequest) -> Result<Mask, ServiceError> {
    // 1. 데이터베이스 업데이트
    let mask = self.mask_repository.update(id, data).await?;
    
    // 2. 캐시 무효화
    self.cache.invalidate(&format!("mask:{}", id)).await;
    self.cache.invalidate(&format!("mask_list:{}", mask.mask_group_id)).await;
    self.cache.invalidate(&format!("mask_stats:{}", mask.mask_group_id)).await;
    
    Ok(mask)
}
```

## 애플리케이션 최적화

### Rust 최적화

#### 1. 컴파일 최적화

**Cargo.toml 설정**
```toml
[profile.release]
opt-level = 3          # 최대 최적화
lto = true            # 링크 타임 최적화
codegen-units = 1     # 단일 코드 생성 유닛
panic = "abort"       # 패닉 시 즉시 종료
strip = true          # 디버그 심볼 제거

[profile.release.package."*"]
opt-level = 3
```

**RUSTFLAGS 설정**
```bash
export RUSTFLAGS="-C target-cpu=native -C opt-level=3"
cargo build --release
```

#### 2. 메모리 최적화

**String vs &str 사용**
```rust
// 비효율적
fn process_data(data: String) -> String {
    format!("processed: {}", data)
}

// 효율적
fn process_data(data: &str) -> String {
    format!("processed: {}", data)
}
```

**Vec vs VecDeque**
```rust
use std::collections::VecDeque;

// 앞쪽 삽입이 많은 경우
let mut queue = VecDeque::new();
queue.push_front(item);

// 뒤쪽 삽입이 많은 경우
let mut vec = Vec::new();
vec.push(item);
```

#### 3. 비동기 최적화

**Future 최적화**
```rust
use futures::future::join_all;

// 순차 실행 (비효율적)
async fn process_sequential(items: Vec<Item>) -> Vec<Result> {
    let mut results = Vec::new();
    for item in items {
        let result = process_item(item).await;
        results.push(result);
    }
    results
}

// 병렬 실행 (효율적)
async fn process_parallel(items: Vec<Item>) -> Vec<Result> {
    let futures = items.into_iter().map(process_item);
    join_all(futures).await
}
```

**커넥션 풀 최적화**
```rust
use sqlx::postgres::PgPoolOptions;

let pool = PgPoolOptions::new()
    .max_connections(100)           // 최대 연결 수
    .min_connections(10)            // 최소 연결 수
    .acquire_timeout(Duration::from_secs(30))
    .idle_timeout(Duration::from_secs(600))
    .max_lifetime(Duration::from_secs(1800))
    .test_before_acquire(true)      // 연결 테스트
    .connect(&database_url)
    .await?;
```

### Actix-web 최적화

#### 1. 워커 스레드 최적화

**런타임 설정**
```rust
use actix_web::{web, App, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // CPU 코어 수에 맞는 워커 설정
    let workers = num_cpus::get();
    
    HttpServer::new(|| {
        App::new()
            .configure(configure_routes)
    })
    .workers(workers)
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

#### 2. 미들웨어 최적화

**커스텀 미들웨어**
```rust
use actix_web::{
    dev::{ServiceRequest, ServiceResponse},
    Error, Result,
};
use actix_web::middleware::Logger;
use std::time::Instant;

pub struct TimingMiddleware;

impl<S, B> Transform<S, ServiceRequest> for TimingMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = TimingMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(TimingMiddlewareService { service }))
    }
}
```

#### 3. JSON 직렬화 최적화

**serde 최적화**
```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct OptimizedResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    optional_field: Option<String>,
    
    #[serde(rename = "id")]
    identifier: i32,
    
    // 불필요한 필드 제외
    #[serde(skip)]
    internal_data: String,
}
```

## 네트워크 최적화

### HTTP/2 최적화

**TLS 설정**
```rust
use actix_web::web::Data;
use rustls::{ServerConfig, NoClientAuth};
use rustls::internal::pemfile;

let mut config = ServerConfig::new(NoClientAuth::new());
let cert_file = &mut BufReader::new(File::open("cert.pem")?);
let key_file = &mut BufReader::new(File::open("key.pem")?);
let cert_chain = pemfile::certs(cert_file).unwrap();
let mut keys = pemfile::rsa_private_keys(key_file).unwrap();
config.set_single_cert(cert_chain, keys[0].clone())?;
```

### 압축 최적화

**Gzip 압축**
```rust
use actix_web::middleware::Compress;

App::new()
    .wrap(Compress::default())
    .configure(configure_routes)
```

**압축 레벨 설정**
```rust
use actix_web::middleware::Compress;
use actix_web::http::ContentEncoding;

App::new()
    .wrap(Compress::new(ContentEncoding::Gzip, 6)) // 압축 레벨 6
    .configure(configure_routes)
```

## 모니터링 및 프로파일링

### 성능 모니터링

**메트릭 수집**
```rust
use prometheus::{Counter, Histogram, Registry};

lazy_static! {
    static ref HTTP_REQUESTS_TOTAL: Counter = Counter::new(
        "http_requests_total", 
        "Total number of HTTP requests"
    ).unwrap();
    
    static ref HTTP_REQUEST_DURATION: Histogram = Histogram::new(
        "http_request_duration_seconds",
        "HTTP request duration in seconds"
    ).unwrap();
}
```

**로깅 최적화**
```rust
use tracing::{info, warn, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// 구조화된 로깅
tracing_subscriber::registry()
    .with(
        tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "pacs_server=debug".into())
    )
    .with(tracing_subscriber::fmt::layer())
    .init();
```

### 프로파일링 도구

**Flamegraph**
```bash
# 설치
cargo install flamegraph

# 프로파일링 실행
cargo flamegraph --bin pacs-server
```

**perf 사용**
```bash
# CPU 프로파일링
perf record -g ./target/release/pacs-server
perf report

# 메모리 프로파일링
perf mem record ./target/release/pacs-server
perf mem report
```

## 인프라스트럭처 최적화

### 컨테이너 최적화

**Dockerfile 최적화**
```dockerfile
# 멀티스테이지 빌드
FROM rust:1.70 as builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

# 최종 이미지
FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/pacs-server /usr/local/bin/
EXPOSE 8080
CMD ["pacs-server"]
```

**Docker Compose 최적화**
```yaml
version: '3.8'
services:
  pacs-server:
    build: .
    ports:
      - "8080:8080"
    environment:
      - RUST_LOG=info
      - DATABASE_URL=postgresql://user:pass@postgres:5432/pacs
    depends_on:
      - postgres
      - redis
    deploy:
      resources:
        limits:
          cpus: '2.0'
          memory: 4G
        reservations:
          cpus: '1.0'
          memory: 2G
```

### 로드 밸런싱

**Nginx 설정**
```nginx
upstream pacs_backend {
    least_conn;
    server pacs-server-1:8080 weight=3;
    server pacs-server-2:8080 weight=3;
    server pacs-server-3:8080 weight=2;
}

server {
    listen 80;
    server_name api.pacs.example.com;
    
    location / {
        proxy_pass http://pacs_backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        
        # 성능 최적화
        proxy_buffering on;
        proxy_buffer_size 128k;
        proxy_buffers 4 256k;
        proxy_busy_buffers_size 256k;
        
        # 타임아웃 설정
        proxy_connect_timeout 30s;
        proxy_send_timeout 30s;
        proxy_read_timeout 30s;
    }
}
```

## 성능 테스트

### 벤치마크 테스트

**Rust 벤치마크**
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pacs_server::services::MaskService;

fn benchmark_mask_creation(c: &mut Criterion) {
    c.bench_function("create_mask", |b| {
        b.iter(|| {
            // 벤치마크 코드
            let service = MaskService::new();
            service.create_mask(black_box(test_data()))
        })
    });
}

criterion_group!(benches, benchmark_mask_creation);
criterion_main!(benches);
```

**부하 테스트**
```bash
# Apache Bench
ab -n 10000 -c 100 http://localhost:8080/api/health

# wrk
wrk -t12 -c400 -d30s http://localhost:8080/api/health

# Artillery
artillery run load_test.yml
```

## 성능 모니터링 대시보드

### Grafana 설정

**대시보드 패널**
```json
{
  "dashboard": {
    "title": "PACS Server Performance",
    "panels": [
      {
        "title": "Request Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(http_requests_total[5m])",
            "legendFormat": "{{method}} {{endpoint}}"
          }
        ]
      },
      {
        "title": "Response Time",
        "type": "graph",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))",
            "legendFormat": "95th percentile"
          }
        ]
      }
    ]
  }
}
```

## 결론

성능 최적화는 지속적인 과정입니다. 정기적인 모니터링과 프로파일링을 통해 병목 지점을 식별하고 개선해야 합니다. 이 가이드의 방법들을 단계적으로 적용하여 시스템의 성능을 향상시킬 수 있습니다.
