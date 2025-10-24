# 기초 백엔드 코드 가이드

## 📋 개요

이 문서는 PACS Extension Server의 현재 코드베이스를 이해하는데 도움이 되는 기초 백엔드 코드 가이드입니다. Clean Architecture 패턴을 따르는 Rust 백엔드 서버의 구조와 핵심 개념을 설명합니다.

## 🏗️ 프로젝트 구조

```
pacs-server/
├── src/
│   ├── main.rs                    # 메인 엔트리 포인트
│   ├── lib.rs                     # 라이브러리 루트
│   ├── domain/                    # 도메인 계층 (비즈니스 로직)
│   │   ├── entities/              # 엔티티 (데이터 모델)
│   │   ├── repositories/          # 리포지토리 인터페이스
│   │   ├── services/              # 서비스 인터페이스
│   │   └── errors.rs              # 도메인 에러 정의
│   ├── application/               # 애플리케이션 계층 (유스케이스)
│   │   ├── use_cases/             # 유스케이스 구현
│   │   ├── services/              # 애플리케이션 서비스
│   │   └── dto/                   # 데이터 전송 객체
│   ├── infrastructure/            # 인프라스트럭처 계층
│   │   ├── repositories/          # 리포지토리 구현체
│   │   ├── auth/                  # 인증 관련
│   │   ├── config/                # 설정 관리
│   │   ├── middleware/            # 미들웨어
│   │   └── external/              # 외부 서비스 연동
│   └── presentation/              # 프레젠테이션 계층 (API)
│       ├── controllers/           # 컨트롤러
│       ├── middleware/            # 프레젠테이션 미들웨어
│       └── openapi.rs             # OpenAPI 문서
├── config/                        # 설정 파일
├── migrations/                    # 데이터베이스 마이그레이션
└── docs/                          # 문서
```

## 🎯 Clean Architecture 핵심 개념

### 1. 계층별 책임

- **Domain Layer**: 비즈니스 로직과 규칙을 담당
- **Application Layer**: 유스케이스와 애플리케이션 서비스를 담당
- **Infrastructure Layer**: 데이터베이스, 외부 API 등 외부 의존성을 담당
- **Presentation Layer**: HTTP API, 컨트롤러를 담당

### 2. 의존성 방향

```
Presentation → Application → Domain
     ↓              ↓
Infrastructure → Application → Domain
```

- 의존성은 항상 내부 계층을 향함
- 외부 계층은 내부 계층의 인터페이스에만 의존

## 🚀 기초 백엔드 서버 (Health Check만 포함)

### 1. Cargo.toml

```toml
[package]
name = "basic-pacs-server"
version = "0.1.0"
edition = "2021"

[dependencies]
# 웹 프레임워크
actix-web = "4.4"
tokio = { version = "1.0", features = ["full"] }

# JSON 직렬화
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# 환경 변수
dotenvy = "0.15"

# 로깅
tracing = "0.1"
tracing-subscriber = "0.3"

# OpenAPI 문서화
utoipa = { version = "4.2", features = ["actix_extras"] }
utoipa-swagger-ui = { version = "6.0", features = ["actix-web"] }
```

### 2. main.rs (기초 버전)

```rust
//! # Basic PACS Server
//! 
//! Health check만 포함하는 기초 백엔드 서버입니다.
//! Clean Architecture 패턴의 기본 구조를 보여줍니다.

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde_json::json;
use std::collections::HashMap;

/// 서버 상태 확인을 위한 헬스체크 엔드포인트
/// 
/// # 반환값
/// - `200 OK`: 서버가 정상적으로 동작 중
/// - JSON 형태로 서버 상태 정보 반환
/// 
/// # 사용 예시
/// ```bash
/// curl http://localhost:8080/health
/// ```
async fn health_check() -> impl Responder {
    let mut status = HashMap::new();
    status.insert("status", "healthy");
    status.insert("service", "basic-pacs-server");
    status.insert("version", "0.1.0");
    status.insert("timestamp", &chrono::Utc::now().to_rfc3339());
    
    HttpResponse::Ok().json(status)
}

/// 서버 정보를 반환하는 엔드포인트
async fn server_info() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "name": "Basic PACS Server",
        "version": "0.1.0",
        "description": "Health check만 포함하는 기초 백엔드 서버",
        "architecture": "Clean Architecture",
        "framework": "Actix Web",
        "language": "Rust"
    }))
}

/// 메인 함수
/// 
/// 이 함수는 서버의 전체 생명주기를 관리합니다:
/// 1. 환경 변수 로드
/// 2. 로깅 설정
/// 3. HTTP 서버 시작
/// 4. Graceful shutdown 처리
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 환경 변수 로드
    dotenvy::dotenv().ok();
    
    // 로깅 초기화
    tracing_subscriber::fmt::init();
    
    println!("🚀 Basic PACS Server Starting...");
    println!("🌐 Server URL: http://localhost:8080");
    println!("❤️  Health Check: http://localhost:8080/health");
    println!("ℹ️  Server Info: http://localhost:8080/info");
    
    // Graceful shutdown을 위한 signal handler
    let shutdown_signal = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
        println!("\n🛑 Received shutdown signal, starting graceful shutdown...");
    };
    
    // HTTP 서버 시작
    HttpServer::new(|| {
        App::new()
            // Health check 엔드포인트
            .route("/health", web::get().to(health_check))
            // 서버 정보 엔드포인트
            .route("/info", web::get().to(server_info))
            // 기본 라우트
            .route("/", web::get().to(|| async {
                HttpResponse::Ok().json(json!({
                    "message": "Welcome to Basic PACS Server",
                    "endpoints": {
                        "health": "/health",
                        "info": "/info"
                    }
                }))
            }))
    })
    .bind("127.0.0.1:8080")?
    .workers(2)
    .shutdown_timeout(30)
    .run()
    .await?;
    
    println!("✅ Server shutdown completed");
    Ok(())
}
```

### 3. lib.rs (라이브러리 루트)

```rust
//! # Basic PACS Server Library
//! 
//! Clean Architecture 패턴을 따르는 기초 백엔드 서버 라이브러리입니다.

// 모듈 선언 (현재는 비어있지만 확장 가능)
// pub mod domain;
// pub mod application;
// pub mod infrastructure;
// pub mod presentation;

// 공개 API
pub use actix_web::{web, App, HttpResponse, HttpServer, Responder};
```

## 🔧 설정 관리

### 1. config/default.toml

```toml
[server]
host = "127.0.0.1"
port = 8080
workers = 2

[logging]
level = "info"
format = "json"

[health_check]
enabled = true
timeout_seconds = 5
```

### 2. 환경 변수 (.env)

```bash
# 서버 설정
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
SERVER_WORKERS=2

# 로깅 설정
RUST_LOG=info
LOG_LEVEL=info

# 개발 환경
RUST_ENV=development
```

## 🧪 테스트

### 1. 단위 테스트

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App};

    #[actix_web::test]
    async fn test_health_check() {
        let app = test::init_service(
            App::new().route("/health", web::get().to(health_check))
        ).await;
        
        let req = test::TestRequest::get().uri("/health").to_request();
        let resp = test::call_service(&app, req).await;
        
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_server_info() {
        let app = test::init_service(
            App::new().route("/info", web::get().to(server_info))
        ).await;
        
        let req = test::TestRequest::get().uri("/info").to_request();
        let resp = test::call_service(&app, req).await;
        
        assert!(resp.status().is_success());
    }
}
```

### 2. 통합 테스트

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use actix_web::{test, web, App};

    #[actix_web::test]
    async fn test_server_startup() {
        let app = test::init_service(
            App::new()
                .route("/health", web::get().to(health_check))
                .route("/info", web::get().to(server_info))
        ).await;
        
        // Health check 테스트
        let req = test::TestRequest::get().uri("/health").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        
        // Server info 테스트
        let req = test::TestRequest::get().uri("/info").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }
}
```

## 🐳 Docker 설정

### 1. Dockerfile

```dockerfile
# 멀티스테이지 빌드
FROM rust:1.75 as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# 의존성 빌드 (캐시 최적화)
RUN cargo build --release

# 런타임 이미지
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/basic-pacs-server /app/

EXPOSE 8080

CMD ["./basic-pacs-server"]
```

### 2. docker-compose.yml

```yaml
version: '3.8'

services:
  basic-pacs-server:
    build: .
    ports:
      - "8080:8080"
    environment:
      - RUST_LOG=info
      - SERVER_HOST=0.0.0.0
      - SERVER_PORT=8080
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s
```

## 🚀 실행 방법

### 1. 로컬 실행

```bash
# 의존성 설치
cargo build

# 서버 실행
cargo run

# 또는 릴리즈 모드로 실행
cargo run --release
```

### 2. Docker 실행

```bash
# Docker 이미지 빌드
docker build -t basic-pacs-server .

# Docker 컨테이너 실행
docker run -p 8080:8080 basic-pacs-server

# 또는 docker-compose 사용
docker-compose up -d
```

### 3. 테스트 실행

```bash
# 단위 테스트
cargo test

# 통합 테스트
cargo test --test integration_tests

# 모든 테스트
cargo test --all
```

## 📊 API 엔드포인트

### 1. Health Check

```bash
GET /health
```

**응답:**
```json
{
  "status": "healthy",
  "service": "basic-pacs-server",
  "version": "0.1.0",
  "timestamp": "2024-01-01T00:00:00Z"
}
```

### 2. Server Info

```bash
GET /info
```

**응답:**
```json
{
  "name": "Basic PACS Server",
  "version": "0.1.0",
  "description": "Health check만 포함하는 기초 백엔드 서버",
  "architecture": "Clean Architecture",
  "framework": "Actix Web",
  "language": "Rust"
}
```

### 3. Root

```bash
GET /
```

**응답:**
```json
{
  "message": "Welcome to Basic PACS Server",
  "endpoints": {
    "health": "/health",
    "info": "/info"
  }
}
```

## 🔍 코드 이해 포인트

### 1. 비동기 프로그래밍

- `async/await` 문법 사용
- `tokio` 런타임 활용
- `actix_web::main` 매크로로 비동기 메인 함수

### 2. 에러 처리

- `Result<T, E>` 타입 사용
- `?` 연산자로 에러 전파
- 적절한 에러 변환

### 3. JSON 처리

- `serde` 크레이트로 직렬화/역직렬화
- `serde_json::json!` 매크로로 JSON 생성
- `HashMap`을 JSON으로 변환

### 4. HTTP 서버

- `actix_web` 프레임워크 사용
- 라우트 기반 엔드포인트 정의
- 미들웨어 지원

## 🎯 다음 단계

이 기초 서버를 바탕으로 다음과 같은 기능을 단계적으로 추가할 수 있습니다:

1. **데이터베이스 연동** (PostgreSQL + SQLx)
2. **인증 시스템** (JWT)
3. **API 컨트롤러** 추가
4. **미들웨어** 구현
5. **로깅 시스템** 구축
6. **테스트 커버리지** 확장
7. **Docker 최적화**
8. **모니터링** 추가

## 📚 참고 자료

- [Actix Web 공식 문서](https://actix.rs/)
- [Rust 공식 문서](https://doc.rust-lang.org/)
- [Clean Architecture](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html)
- [Serde 문서](https://serde.rs/)

---

이 가이드를 통해 현재 PACS Extension Server의 구조를 이해하고, 단계적으로 기능을 확장할 수 있습니다. 각 계층의 역할과 의존성 방향을 명확히 이해하는 것이 중요합니다.
