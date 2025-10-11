# 부하 테스트 가이드

## 개요

이 문서는 PACS 서버의 부하 테스트를 위한 가이드입니다. 마스크 업로드 시스템의 성능을 측정하고 병목 지점을 식별하기 위한 테스트 시나리오와 도구를 제공합니다.

## 테스트 목표

### 주요 목표
1. **동시 사용자 처리 능력** 측정
2. **대용량 파일 업로드** 성능 검증
3. **데이터베이스 성능** 최적화 확인
4. **메모리 사용량** 모니터링
5. **API 응답 시간** 측정

### 성능 지표
- **처리량 (Throughput)**: 초당 요청 수 (RPS)
- **응답 시간**: 평균, 95%, 99% 응답 시간
- **에러율**: 실패한 요청의 비율
- **리소스 사용률**: CPU, 메모리, 디스크 사용률

## 테스트 환경 설정

### 하드웨어 요구사항

**최소 사양**
- CPU: 4 cores
- RAM: 8GB
- Storage: 100GB SSD
- Network: 1Gbps

**권장 사양**
- CPU: 8+ cores
- RAM: 16GB+
- Storage: 500GB+ NVMe SSD
- Network: 10Gbps

### 소프트웨어 설정

**데이터베이스 최적화**
```sql
-- PostgreSQL 설정 최적화
ALTER SYSTEM SET max_connections = 200;
ALTER SYSTEM SET shared_buffers = '4GB';
ALTER SYSTEM SET effective_cache_size = '12GB';
ALTER SYSTEM SET work_mem = '256MB';
ALTER SYSTEM SET maintenance_work_mem = '1GB';
ALTER SYSTEM SET checkpoint_completion_target = 0.9;
ALTER SYSTEM SET wal_buffers = '64MB';
ALTER SYSTEM SET default_statistics_target = 100;
```

**Rust 애플리케이션 설정**
```toml
# Cargo.toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
```

## 테스트 도구

### 1. Apache Bench (ab)

**설치**
```bash
# Ubuntu/Debian
sudo apt-get install apache2-utils

# macOS
brew install httpie
```

**기본 사용법**
```bash
# 단일 엔드포인트 테스트
ab -n 1000 -c 10 http://localhost:8080/api/health

# POST 요청 테스트
ab -n 1000 -c 10 -p data.json -T application/json http://localhost:8080/api/annotations/1/mask-groups/1/masks
```

### 2. wrk

**설치**
```bash
# Ubuntu/Debian
sudo apt-get install wrk

# macOS
brew install wrk
```

**사용법**
```bash
# 기본 테스트
wrk -t12 -c400 -d30s http://localhost:8080/api/health

# Lua 스크립트와 함께
wrk -t12 -c400 -d30s -s load_test.lua http://localhost:8080/
```

### 3. Artillery

**설치**
```bash
npm install -g artillery
```

**설정 파일 (artillery.yml)**
```yaml
config:
  target: 'http://localhost:8080'
  phases:
    - duration: 60
      arrivalRate: 10
    - duration: 120
      arrivalRate: 50
    - duration: 60
      arrivalRate: 10
scenarios:
  - name: "Health Check"
    weight: 30
    flow:
      - get:
          url: "/api/health"
  - name: "Mask Upload"
    weight: 70
    flow:
      - post:
          url: "/api/annotations/{{ $randomInt(1, 100) }}/mask-groups/{{ $randomInt(1, 50) }}/masks"
          headers:
            X-User-ID: "{{ $randomInt(1, 10) }}"
          json:
            mask_group_id: "{{ $randomInt(1, 50) }}"
            file_path: "masks/test_{{ $randomInt(1, 1000) }}.png"
            mime_type: "image/png"
            file_size: "{{ $randomInt(1024, 10485760) }}"
```

## 테스트 시나리오

### 시나리오 1: 기본 API 부하 테스트

**목적**: 기본 API 엔드포인트의 성능 측정

**테스트 케이스**:
1. 헬스체크 API
2. 마스크 목록 조회 API
3. 마스크 통계 조회 API

**실행 명령**:
```bash
# 헬스체크 테스트
ab -n 10000 -c 100 http://localhost:8080/api/health

# 마스크 목록 조회 테스트
ab -n 5000 -c 50 "http://localhost:8080/api/annotations/1/mask-groups/1/masks"

# 마스크 통계 조회 테스트
ab -n 5000 -c 50 "http://localhost:8080/api/annotations/1/mask-groups/1/masks/stats"
```

### 시나리오 2: 마스크 업로드 부하 테스트

**목적**: 마스크 업로드 API의 동시 처리 능력 측정

**테스트 케이스**:
1. 소용량 파일 (1KB-1MB) 동시 업로드
2. 중용량 파일 (1MB-10MB) 동시 업로드
3. 대용량 파일 (10MB-100MB) 동시 업로드

**실행 명령**:
```bash
# 소용량 파일 테스트
artillery run mask_upload_small.yml

# 중용량 파일 테스트
artillery run mask_upload_medium.yml

# 대용량 파일 테스트
artillery run mask_upload_large.yml
```

### 시나리오 3: 데이터베이스 부하 테스트

**목적**: 데이터베이스 성능과 동시성 측정

**테스트 케이스**:
1. 동시 읽기 작업
2. 동시 쓰기 작업
3. 읽기/쓰기 혼합 작업

**실행 명령**:
```bash
# 데이터베이스 부하 테스트
pgbench -i -s 100 pacs_test
pgbench -c 10 -j 2 -t 1000 pacs_test
```

### 시나리오 4: 메모리 누수 테스트

**목적**: 장시간 실행 시 메모리 사용량 모니터링

**테스트 케이스**:
1. 24시간 연속 실행
2. 메모리 사용량 모니터링
3. 가비지 컬렉션 분석

**실행 명령**:
```bash
# 장시간 부하 테스트
artillery run long_running_test.yml --duration 86400
```

## 모니터링

### 시스템 리소스 모니터링

**htop 사용**
```bash
htop
```

**iostat 사용**
```bash
iostat -x 1
```

**netstat 사용**
```bash
netstat -tuln | grep :8080
```

### 애플리케이션 모니터링

**로그 모니터링**
```bash
tail -f pacs-server.log | grep -E "(ERROR|WARN|INFO)"
```

**메트릭 수집**
```bash
# Prometheus 메트릭 (구현 예정)
curl http://localhost:8080/metrics
```

### 데이터베이스 모니터링

**PostgreSQL 모니터링**
```sql
-- 활성 연결 수
SELECT count(*) FROM pg_stat_activity;

-- 느린 쿼리 확인
SELECT query, mean_time, calls 
FROM pg_stat_statements 
ORDER BY mean_time DESC 
LIMIT 10;

-- 테이블 크기 확인
SELECT schemaname, tablename, 
       pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) as size
FROM pg_tables 
WHERE schemaname = 'public'
ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;
```

## 성능 최적화

### 애플리케이션 최적화

**커넥션 풀 최적화**
```rust
// SQLx 커넥션 풀 설정
let pool = PgPoolOptions::new()
    .max_connections(100)
    .min_connections(10)
    .acquire_timeout(Duration::from_secs(30))
    .idle_timeout(Duration::from_secs(600))
    .max_lifetime(Duration::from_secs(1800))
    .connect(&database_url)
    .await?;
```

**캐싱 전략**
```rust
// Redis 캐싱 구현
use redis::Client;

let client = Client::open("redis://127.0.0.1/")?;
let mut con = client.get_connection()?;

// 캐시 설정
redis::cmd("SET")
    .arg("mask_stats:123")
    .arg(serialized_data)
    .arg("EX")
    .arg(300) // 5분 TTL
    .execute(&mut con);
```

### 데이터베이스 최적화

**인덱스 최적화**
```sql
-- 복합 인덱스 생성
CREATE INDEX CONCURRENTLY idx_mask_group_user_created 
ON annotation_mask_group(created_by, created_at);

-- 부분 인덱스 생성
CREATE INDEX CONCURRENTLY idx_mask_active 
ON annotation_mask(mask_group_id) 
WHERE deleted_at IS NULL;
```

**쿼리 최적화**
```sql
-- EXPLAIN ANALYZE로 쿼리 분석
EXPLAIN ANALYZE 
SELECT * FROM annotation_mask 
WHERE mask_group_id = $1 
ORDER BY created_at DESC 
LIMIT 100;
```

## 테스트 결과 분석

### 성능 기준

**헬스체크 API**
- 목표 RPS: 1000+
- 목표 응답시간: < 10ms
- 목표 에러율: < 0.1%

**마스크 업로드 API**
- 목표 RPS: 100+
- 목표 응답시간: < 500ms
- 목표 에러율: < 1%

**마스크 조회 API**
- 목표 RPS: 500+
- 목표 응답시간: < 100ms
- 목표 에러율: < 0.5%

### 결과 해석

**좋은 결과**
- 응답시간이 목표치 이하
- 에러율이 1% 미만
- CPU 사용률이 80% 미만
- 메모리 사용률이 안정적

**개선이 필요한 경우**
- 응답시간이 목표치 초과
- 에러율이 5% 이상
- CPU 사용률이 90% 이상
- 메모리 사용량이 지속적으로 증가

## 문제 해결

### 일반적인 문제

1. **높은 응답시간**
   - 데이터베이스 쿼리 최적화
   - 인덱스 추가
   - 캐싱 도입

2. **높은 에러율**
   - 커넥션 풀 크기 조정
   - 타임아웃 설정 조정
   - 리소스 한계 확인

3. **메모리 누수**
   - 가비지 컬렉션 분석
   - 메모리 프로파일링
   - 코드 리뷰

### 디버깅 도구

**프로파일링**
```bash
# Rust 프로파일링
cargo install flamegraph
cargo flamegraph --bin pacs-server

# 메모리 프로파일링
cargo install cargo-profdata
cargo profdata --bin pacs-server
```

**로깅 설정**
```rust
// 상세 로깅 설정
env_logger::Builder::from_env(Env::default().default_filter_or("debug"))
    .init();
```

## 자동화

### CI/CD 통합

**GitHub Actions 예시**
```yaml
name: Load Test
on:
  schedule:
    - cron: '0 2 * * *'  # 매일 새벽 2시

jobs:
  load-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run load test
        run: |
          cargo test --test load_test
          artillery run load_test.yml
```

### 모니터링 대시보드

**Grafana 설정**
```yaml
# docker-compose.yml
version: '3'
services:
  grafana:
    image: grafana/grafana
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
    volumes:
      - grafana-storage:/var/lib/grafana

  prometheus:
    image: prom/prometheus
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
```

## 결론

부하 테스트는 시스템의 성능 한계를 파악하고 사용자 경험을 보장하는 중요한 과정입니다. 정기적인 부하 테스트를 통해 시스템의 안정성과 성능을 지속적으로 모니터링하고 개선해야 합니다.
