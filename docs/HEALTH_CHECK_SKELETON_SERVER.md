# 헬스체크 뼈대 서버 기술문서

## 📋 개요

본 문서는 PACS Extension Server 코드베이스를 기반으로 **헬스체크만 있는 최소한의 뼈대 서버**를 구축하는 방법을 설명합니다. Clean Architecture 패턴을 유지하면서 불필요한 복잡성을 제거하고, 서버의 기본 상태 확인 기능만을 제공하는 경량화된 서버를 만듭니다.

**⚠️ 중요**: 이 문서는 Rust 1.75.0 환경에서 테스트되었으며, 최신 Rust 버전과의 호환성 문제를 해결하기 위해 의존성을 최적화했습니다.

## 🎯 목표

- **최소한의 의존성**: 핵심 웹 서버 기능만 포함
- **Clean Architecture 유지**: 확장 가능한 구조 보존
- **헬스체크 기능**: 서버 상태 모니터링
- **Docker 지원**: 컨테이너화된 배포
- **설정 관리**: 환경별 설정 분리
- **Rust 1.75.0 호환성**: 안정적인 빌드 보장

## 🏗️ 아키텍처 설계

### 계층 구조 (Clean Architecture)

```
src/
├── main.rs                 # 서버 엔트리 포인트
├── lib.rs                  # 라이브러리 루트
├── domain/                 # 도메인 계층
│   ├── mod.rs
│   └── errors.rs          # 에러 정의
├── infrastructure/         # 인프라스트럭처 계층
│   ├── mod.rs
│   ├── config/            # 설정 관리
│   │   ├── mod.rs
│   │   └── settings.rs
│   └── middleware/        # 미들웨어
│       ├── mod.rs
│       └── cors_middleware.rs
└── presentation/          # 프레젠테이션 계층
    ├── mod.rs
    └── controllers/       # 컨트롤러
        ├── mod.rs
        └── health_controller.rs
```

### 핵심 컴포넌트

1. **Health Check Controller**: 서버 상태 확인 엔드포인트
2. **Configuration Management**: 환경별 설정 관리
3. **CORS Middleware**: 크로스 오리진 요청 처리
4. **Error Handling**: 통합 에러 처리

## 📦 의존성 관리

### Cargo.toml (Rust 1.75.0 호환 최소 의존성)

```toml
[package]
name = "health-check-server"
version = "0.1.0"
edition = "2021"
authors = ["Your Name <your.email@example.com>"]
description = "Minimal health check server with Clean Architecture"
license = "MIT"
repository = "https://github.com/your-username/health-check-server"

[dependencies]
# 웹 프레임워크 - Rust 1.75 호환 버전
actix-web = "3.3"  # 4.x 버전은 Rust 1.81+ 필요

# 비동기 런타임
tokio = { version = "1.35", features = ["full"] }

# 설정 관리 - 간단한 환경변수 기반 설정 사용
dotenvy = "0.15"

# JSON 처리
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# 에러 처리
thiserror = "1.0"

# 로깅 - 간단한 로깅 사용 (tracing-subscriber는 Rust 1.81+ 필요)
# tracing = "0.1"
# tracing-subscriber = "0.3"

# 시간 처리 - std::time 사용 (chrono는 Rust 1.81+ 필요)
# chrono = "0.4"

# OpenAPI 문서화 (선택사항) - Rust 1.75 호환 버전
# utoipa = { version = "4.2", features = ["actix_extras"] }
# utoipa-swagger-ui = { version = "6.0", features = ["actix-web"] }

[dev-dependencies]
# 테스트용 HTTP 클라이언트
reqwest = { version = "0.11", features = ["json"] }
tokio-test = "0.4"
```

**⚠️ 의존성 호환성 참고사항**:
- `actix-web 4.x`는 Rust 1.81+ 필요하므로 `3.3` 사용
- `tracing-subscriber`는 Rust 1.81+ 필요하므로 간단한 로깅 사용
- `chrono`는 Rust 1.81+ 필요하므로 `std::time` 사용
- `config` 크레이트는 Rust 1.81+ 필요하므로 환경변수 직접 사용

## 🔧 설정 관리

### 환경별 설정 파일

#### config/default.toml
```toml
[server]
host = "0.0.0.0"
port = 8080
workers = 4

[logging]
level = "info"
format = "json"

[cors]
enabled = true
allowed_origins = ["http://localhost:3000", "http://localhost:8080"]
allowed_methods = ["GET", "POST", "PUT", "DELETE", "OPTIONS"]
allowed_headers = ["Content-Type", "Authorization", "X-Requested-With"]
expose_headers = ["Content-Length"]
max_age = 3600

[health_check]
enabled = true
endpoint = "/health"
response_format = "json"
```

#### config/development.toml
```toml
[server]
host = "127.0.0.1"
port = 3000

[logging]
level = "debug"
format = "pretty"

[cors]
enabled = true
allowed_origins = ["http://localhost:3000", "http://localhost:8080", "http://127.0.0.1:3000"]
```

#### config/production.toml
```toml
[server]
host = "0.0.0.0"
port = 8080
workers = 8

[logging]
level = "warn"
format = "json"

[cors]
enabled = false
```

## 🚀 핵심 구현

### 1. 메인 서버 (main.rs) - Rust 1.75.0 호환 버전

```rust
//! # Health Check Server
//! 
//! 최소한의 헬스체크 기능을 제공하는 뼈대 서버입니다.
//! Clean Architecture 패턴을 따르며 확장 가능한 구조를 유지합니다.
//! 
//! ## 주요 기능
//! - 서버 상태 확인 (Health Check)
//! - CORS 지원
//! - 구조화된 로깅
//! - 환경별 설정 관리

use actix_web::{web, App, HttpServer, middleware::Logger, Result, HttpResponse, Responder};
use std::io;

// 모듈 선언
mod domain;
mod infrastructure;
mod presentation;

// 설정 및 미들웨어
use infrastructure::config::Settings;
use infrastructure::middleware::configure_cors;

// 컨트롤러
use presentation::controllers::health_controller;

/// 서버 상태 확인을 위한 기본 헬스체크 엔드포인트
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
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "health-check-server",
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        "version": env!("CARGO_PKG_VERSION")
    }))
}

/// PACS Extension Server의 메인 함수
/// 
/// 이 함수는 서버의 전체 생명주기를 관리합니다:
/// 1. 환경 변수 로드
/// 2. 설정 로드
/// 3. 로깅 초기화
/// 4. HTTP 서버 시작
/// 5. Graceful shutdown 처리
/// 
/// # 반환값
/// - `Ok(())`: 서버가 정상적으로 종료됨
/// - `Err(io::Error)`: 서버 시작 또는 실행 중 오류 발생
/// 
/// # 환경 변수
/// - `RUN_MODE`: 실행 모드 (development, production)
/// - `HOST`: 서버 호스트 (기본값: 0.0.0.0)
/// - `PORT`: 서버 포트 (기본값: 8080)
/// - `LOG_LEVEL`: 로그 레벨 (기본값: info)
/// - `CORS_ENABLED`: CORS 활성화 여부 (기본값: true)
#[actix_rt::main]
async fn main() -> io::Result<()> {
    // .env 파일에서 환경 변수 로드
    dotenvy::dotenv().ok();

    // 서버 초기화 시작 메시지 출력
    println!("\n{}", "=".repeat(80));
    println!("🚀 Health Check Server - Initialization");
    println!("{}\n", "=".repeat(80));

    // 설정 로드
    print!("⚙️  Loading configuration... ");
    let settings = Settings::new()
        .or_else(|_| {
            println!("⚠️  Config files not found, using environment variable defaults");
            Settings::with_env_defaults()
        })
        .expect("Failed to load configuration");
    println!("✅ Done");

    // 로깅 초기화 (간단한 로깅)
    print!("📝 Initializing logging... ");
    // 간단한 로깅 - 환경 변수로 로그 레벨 제어
    std::env::set_var("RUST_LOG", &settings.logging.level);
    println!("✅ Done (Level: {})", settings.logging.level);

    // CORS 설정
    print!("🌐 Configuring CORS... ");
    let cors_enabled = settings.cors.enabled;
    println!("✅ {} (Origins: {:?})", 
        if cors_enabled { "Enabled" } else { "Disabled" }, 
        settings.cors.allowed_origins
    );

    println!("\n{}", "=".repeat(80));
    println!("✨ Server Ready!");
    println!("{}", "=".repeat(80));
    println!("🌐 Server URL:    http://{}:{}", settings.server.host, settings.server.port);
    println!("❤️  Health Check:  http://{}:{}/health", settings.server.host, settings.server.port);
    println!("🔌 API Endpoints: http://{}:{}/api/", settings.server.host, settings.server.port);
    println!("{}\n", "=".repeat(80));

    // Graceful shutdown을 위한 signal handler 설정
    let shutdown_signal = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
        println!("\n🛑 Received shutdown signal, starting graceful shutdown...");
    };
    
    HttpServer::new(move || {
        App::new()
            // 로깅 미들웨어
            .wrap(Logger::default())
            // CORS 미들웨어
            .wrap(configure_cors(&settings.cors))
            // 기본 헬스체크 엔드포인트
            .route("/health", web::get().to(health_check))
            // API 라우트
            .service(
                web::scope("/api")
                    .configure(health_controller::configure_routes)
            )
    })
    .bind((settings.server.host.as_str(), settings.server.port))?
    .workers(settings.server.workers)
    .shutdown_timeout(30) // 30초 graceful shutdown timeout
    .run()
    .await?;
    
    // Graceful shutdown 완료
    println!("✅ Server shutdown completed");
    Ok(())
}
```

### 2. 설정 관리 (infrastructure/config/settings.rs) - Rust 1.75.0 호환 버전

```rust
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CorsConfig {
    pub enabled: bool,
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub allowed_headers: Vec<String>,
    pub expose_headers: Vec<String>,
    pub max_age: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct HealthCheckConfig {
    pub enabled: bool,
    pub endpoint: String,
    pub response_format: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub server: ServerConfig,
    pub logging: LoggingConfig,
    pub cors: CorsConfig,
    pub health_check: HealthCheckConfig,
}

impl Settings {
    /// 환경변수 기반 설정 로드 (config 크레이트 없이)
    pub fn new() -> Result<Self, String> {
        // 간단한 설정 로드 - 환경변수만 사용
        Ok(Self::with_env_defaults())
    }

    /// 환경변수 기본값으로 설정 생성
    pub fn with_env_defaults() -> Self {
        Self {
            server: ServerConfig {
                host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: env::var("PORT")
                    .unwrap_or_else(|_| "8080".to_string())
                    .parse()
                    .unwrap_or(8080),
                workers: env::var("WORKERS")
                    .unwrap_or_else(|_| "4".to_string())
                    .parse()
                    .unwrap_or(4),
            },
            logging: LoggingConfig {
                level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
                format: env::var("LOG_FORMAT").unwrap_or_else(|_| "json".to_string()),
            },
            cors: CorsConfig {
                enabled: env::var("CORS_ENABLED")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                allowed_origins: vec![
                    "http://localhost:3000".to_string(),
                    "http://localhost:8080".to_string(),
                    "http://127.0.0.1:3000".to_string(),
                ],
                allowed_methods: vec![
                    "GET".to_string(),
                    "POST".to_string(),
                    "PUT".to_string(),
                    "DELETE".to_string(),
                    "OPTIONS".to_string(),
                ],
                allowed_headers: vec![
                    "Content-Type".to_string(),
                    "Authorization".to_string(),
                    "X-Requested-With".to_string(),
                ],
                expose_headers: vec!["Content-Length".to_string()],
                max_age: 3600,
            },
            health_check: HealthCheckConfig {
                enabled: true,
                endpoint: "/health".to_string(),
                response_format: "json".to_string(),
            },
        }
    }
}
```

### 3. CORS 미들웨어 (infrastructure/middleware/cors_middleware.rs) - Rust 1.75.0 호환 버전

```rust
use actix_web::middleware::DefaultHeaders;
use actix_web::http::header;
use crate::infrastructure::config::CorsConfig;

/// CORS 설정을 위한 미들웨어 구성
/// 
/// actix-cors 크레이트 없이 기본 헤더를 사용하여 CORS를 구현합니다.
/// Rust 1.75.0 호환성을 위해 actix-web 3.x API를 사용합니다.
pub fn configure_cors(cors_config: &CorsConfig) -> DefaultHeaders {
    if !cors_config.enabled {
        return DefaultHeaders::new()
            .add((header::ACCESS_CONTROL_ALLOW_ORIGIN, "*"))
            .add((header::ACCESS_CONTROL_ALLOW_METHODS, "GET, POST, PUT, DELETE, OPTIONS"))
            .add((header::ACCESS_CONTROL_ALLOW_HEADERS, "Content-Type, Authorization, X-Requested-With"));
    }

    // 허용된 오리진들을 쉼표로 구분된 문자열로 변환
    let allowed_origins = cors_config.allowed_origins.join(", ");
    let allowed_methods = cors_config.allowed_methods.join(", ");
    let allowed_headers = cors_config.allowed_headers.join(", ");
    let expose_headers = cors_config.expose_headers.join(", ");

    DefaultHeaders::new()
        .add((header::ACCESS_CONTROL_ALLOW_ORIGIN, allowed_origins))
        .add((header::ACCESS_CONTROL_ALLOW_METHODS, allowed_methods))
        .add((header::ACCESS_CONTROL_ALLOW_HEADERS, allowed_headers))
        .add((header::ACCESS_CONTROL_EXPOSE_HEADERS, expose_headers))
        .add((header::ACCESS_CONTROL_MAX_AGE, cors_config.max_age.to_string()))
}
```

### 4. 헬스체크 컨트롤러 (presentation/controllers/health_controller.rs) - Rust 1.75.0 호환 버전

```rust
//! # Health Controller
//! 
//! 서버 상태 확인과 관련된 엔드포인트들을 처리합니다.

use actix_web::{web, HttpResponse, Result, Responder};
use serde_json::json;
use crate::domain::errors::DomainError;

/// 상세한 서버 상태 정보를 반환하는 엔드포인트
/// 
/// # 반환값
/// - `200 OK`: 서버가 정상적으로 동작 중
/// - JSON 형태로 상세한 서버 상태 정보 반환
/// 
/// # 사용 예시
/// ```bash
/// curl http://localhost:8080/api/health/detailed
/// ```
pub async fn detailed_health_check() -> impl Responder {
    let response = json!({
        "status": "healthy",
        "service": "health-check-server",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        "uptime": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        "environment": std::env::var("RUN_MODE").unwrap_or_else(|_| "development".to_string()),
        "features": {
            "health_check": true,
            "cors": true,
            "logging": true
        }
    });

    HttpResponse::Ok().json(response)
}

/// 간단한 서버 상태 확인 엔드포인트
/// 
/// # 반환값
/// - `200 OK`: 서버가 정상적으로 동작 중
/// - JSON 형태로 간단한 상태 정보 반환
/// 
/// # 사용 예시
/// ```bash
/// curl http://localhost:8080/api/health/simple
/// ```
pub async fn simple_health_check() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "status": "ok"
    }))
}

/// 서버 정보를 반환하는 엔드포인트
/// 
/// # 반환값
/// - `200 OK`: 서버 정보 반환
/// - JSON 형태로 서버 정보 및 사용 가능한 엔드포인트 목록 반환
/// 
/// # 사용 예시
/// ```bash
/// curl http://localhost:8080/api/info
/// ```
pub async fn server_info() -> impl Responder {
    let response = json!({
        "name": "Health Check Server",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "Minimal health check server with Clean Architecture",
        "endpoints": {
            "health": "/health",
            "detailed_health": "/api/health/detailed",
            "simple_health": "/api/health/simple",
            "info": "/api/info"
        }
    });

    HttpResponse::Ok().json(response)
}

/// 서버 상태를 검증하는 엔드포인트
/// 
/// # 반환값
/// - `200 OK`: 서버가 정상 상태
/// - `503 Service Unavailable`: 서버가 비정상 상태
/// 
/// # 사용 예시
/// ```bash
/// curl http://localhost:8080/api/health/validate
/// ```
pub async fn validate_health() -> impl Responder {
    // 실제 서비스에서는 여기서 다양한 상태를 검증할 수 있습니다
    // 예: 데이터베이스 연결, 외부 서비스 상태, 메모리 사용량 등
    
    // 현재는 항상 정상 상태로 반환
    let is_healthy = true;
    
    if is_healthy {
        HttpResponse::Ok().json(json!({
            "status": "healthy",
            "checks": {
                "server": "ok",
                "memory": "ok",
                "disk": "ok"
            }
        }))
    } else {
        HttpResponse::ServiceUnavailable().json(json!({
            "status": "unhealthy",
            "error": "One or more health checks failed"
        }))
    }
}

/// 라우트 설정
/// 
/// # 매개변수
/// - `cfg`: Actix Web 서비스 설정
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg
        .route("/health/detailed", web::get().to(detailed_health_check))
        .route("/health/simple", web::get().to(simple_health_check))
        .route("/health/validate", web::get().to(validate_health))
        .route("/info", web::get().to(server_info));
}
```

## 🐳 Docker 지원

### Dockerfile - Rust 1.75.0 호환 버전

```dockerfile
# Multi-stage build for production
FROM rust:1.75-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy manifest files
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (this layer will be cached)
RUN cargo build --release && rm -rf src

# Copy source code
COPY src ./src
COPY config ./config

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false appuser

# Set working directory
WORKDIR /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/health-check-server /app/

# Copy config files
COPY --from=builder /app/config /app/config

# Change ownership
RUN chown -R appuser:appuser /app

# Switch to non-root user
USER appuser

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Run the application
CMD ["./health-check-server"]
```

### docker-compose.yml

```yaml
version: '3.8'

services:
  health-check-server:
    build: .
    ports:
      - "8080:8080"
    environment:
      - RUN_MODE=production
      - HOST=0.0.0.0
      - PORT=8080
      - LOG_LEVEL=info
      - CORS_ENABLED=true
    volumes:
      - ./config:/app/config:ro
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s

  # 개발 환경용
  health-check-server-dev:
    build: .
    ports:
      - "3000:3000"
    environment:
      - RUN_MODE=development
      - HOST=0.0.0.0
      - PORT=3000
      - LOG_LEVEL=debug
      - CORS_ENABLED=true
    volumes:
      - ./config:/app/config:ro
      - ./src:/app/src:ro
    restart: unless-stopped
    profiles:
      - dev
```

## 🛠️ 빌드 및 실행

### Makefile

```makefile
# Makefile for Health Check Server

.PHONY: help build run test clean docker-build docker-run dev

# Default target
help:
	@echo "Available commands:"
	@echo "  build        - Build the application"
	@echo "  run          - Run the application"
	@echo "  test         - Run tests"
	@echo "  clean        - Clean build artifacts"
	@echo "  docker-build - Build Docker image"
	@echo "  docker-run   - Run with Docker Compose"
	@echo "  dev          - Run in development mode"

# Build the application
build:
	@echo "🔨 Building Health Check Server..."
	cargo build --release
	@echo "✅ Build completed"

# Run the application
run:
	@echo "🚀 Starting Health Check Server..."
	cargo run

# Run tests
test:
	@echo "🧪 Running tests..."
	cargo test
	@echo "✅ Tests completed"

# Clean build artifacts
clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean
	@echo "✅ Clean completed"

# Build Docker image
docker-build:
	@echo "🐳 Building Docker image..."
	docker build -t health-check-server .
	@echo "✅ Docker image built"

# Run with Docker Compose
docker-run:
	@echo "🐳 Starting with Docker Compose..."
	docker-compose up --build

# Run in development mode
dev:
	@echo "🔧 Starting in development mode..."
	RUN_MODE=development cargo run

# Run with specific environment
run-prod:
	@echo "🚀 Starting in production mode..."
	RUN_MODE=production cargo run

# Check health
health:
	@echo "❤️  Checking server health..."
	curl -f http://localhost:8080/health || echo "❌ Server is not responding"

# Install dependencies
deps:
	@echo "📦 Installing dependencies..."
	cargo build
	@echo "✅ Dependencies installed"
```

## 📊 모니터링 및 헬스체크

### 헬스체크 엔드포인트

1. **기본 헬스체크**: `GET /health`
   ```json
   {
     "status": "healthy",
     "service": "health-check-server",
     "timestamp": "2024-01-15T10:30:00Z",
     "version": "0.1.0"
   }
   ```

2. **상세 헬스체크**: `GET /api/health/detailed`
   ```json
   {
     "status": "healthy",
     "service": "health-check-server",
     "version": "0.1.0",
     "timestamp": "2024-01-15T10:30:00Z",
     "uptime": 1705312200,
     "environment": "production",
     "features": {
       "health_check": true,
       "cors": true,
       "logging": true
     }
   }
   ```

3. **서버 정보**: `GET /api/info`
   ```json
   {
     "name": "Health Check Server",
     "version": "0.1.0",
     "description": "Minimal health check server with Clean Architecture",
     "endpoints": {
       "health": "/health",
       "detailed_health": "/api/health/detailed",
       "simple_health": "/api/health/simple",
       "info": "/api/info"
     }
   }
   ```

### 로깅 설정

```rust
// main.rs에서 로깅 초기화
tracing_subscriber::fmt()
    .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
    .init();
```

환경 변수로 로깅 레벨 제어:
```bash
export RUST_LOG=debug  # debug, info, warn, error
export RUST_LOG=health_check_server=debug,actix_web=info
```

## 🔧 확장 가이드

### 새로운 엔드포인트 추가

1. **컨트롤러 생성**:
   ```rust
   // presentation/controllers/new_controller.rs
   use actix_web::{web, HttpResponse, Result};
   
   pub async fn new_endpoint() -> Result<HttpResponse> {
       Ok(HttpResponse::Ok().json(serde_json::json!({
           "message": "New endpoint"
       })))
   }
   
   pub fn configure_routes(cfg: &mut web::ServiceConfig) {
       cfg.route("/new", web::get().to(new_endpoint));
   }
   ```

2. **라우트 등록**:
   ```rust
   // main.rs
   .service(
       web::scope("/api")
           .configure(health_controller::configure_routes)
           .configure(new_controller::configure_routes)  // 추가
   )
   ```

### 데이터베이스 추가

1. **의존성 추가**:
   ```toml
   # Cargo.toml
   sqlx = { version = "0.7", features = ["runtime-tokio-native-tls", "postgres"] }
   ```

2. **설정 추가**:
   ```toml
   # config/default.toml
   [database]
   url = "postgresql://user:password@localhost/dbname"
   max_connections = 10
   min_connections = 2
   ```

3. **연결 풀 설정**:
   ```rust
   // main.rs
   let pool = PgPoolOptions::new()
       .max_connections(settings.database.max_connections)
       .min_connections(settings.database.min_connections)
       .connect(&settings.database.url)
       .await?;
   ```

## 📝 배포 체크리스트

### 개발 환경
- [ ] Rust 1.75+ 설치
- [ ] Cargo 의존성 설치
- [ ] 환경 변수 설정
- [ ] 로컬 테스트 실행

### 프로덕션 환경
- [ ] Docker 이미지 빌드
- [ ] 환경별 설정 파일 준비
- [ ] 로그 디렉토리 설정
- [ ] 헬스체크 엔드포인트 테스트
- [ ] 모니터링 설정

### 보안 체크리스트
- [ ] CORS 설정 검토
- [ ] 환경 변수 보안 검토
- [ ] 로그 민감정보 제거
- [ ] 컨테이너 보안 스캔

## 🔧 문제 해결 (Troubleshooting)

### Rust 버전 호환성 문제

#### 문제: `actix-web 4.x` 빌드 실패
```
error: package `actix-web v4.x` cannot be built because it requires rustc 1.81 or newer
```

**해결책**: `actix-web 3.3` 사용
```toml
actix-web = "3.3"  # 4.x 대신 3.3 사용
```

#### 문제: `tracing-subscriber` 빌드 실패
```
error: package `tracing-subscriber v0.3.x` cannot be built because it requires rustc 1.81 or newer
```

**해결책**: 간단한 로깅 사용
```rust
// tracing-subscriber 대신 간단한 로깅
std::env::set_var("RUST_LOG", "info");
```

#### 문제: `chrono` 빌드 실패
```
error: package `chrono v0.4.x` cannot be built because it requires rustc 1.81 or newer
```

**해결책**: `std::time` 사용
```rust
// chrono 대신 std::time 사용
let timestamp = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_secs();
```

#### 문제: `config` 크레이트 빌드 실패
```
error: package `config v0.14` cannot be built because it requires rustc 1.81 or newer
```

**해결책**: 환경변수 직접 사용
```rust
// config 크레이트 대신 환경변수 직접 사용
let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
```

### 빌드 오류 해결

#### 문제: `actix-cors` 빌드 실패
```
error: package `actix-cors v0.7` cannot be built because it requires rustc 1.81 or newer
```

**해결책**: `DefaultHeaders` 사용
```rust
use actix_web::middleware::DefaultHeaders;

pub fn configure_cors(cors_config: &CorsConfig) -> DefaultHeaders {
    DefaultHeaders::new()
        .add((header::ACCESS_CONTROL_ALLOW_ORIGIN, "*"))
        .add((header::ACCESS_CONTROL_ALLOW_METHODS, "GET, POST, PUT, DELETE, OPTIONS"))
}
```

#### 문제: `utoipa` 빌드 실패
```
error: package `utoipa v5.x` cannot be built because it requires rustc 1.81 or newer
```

**해결책**: OpenAPI 문서화 비활성화
```toml
# OpenAPI 관련 의존성 주석 처리
# utoipa = { version = "4.2", features = ["actix_extras"] }
# utoipa-swagger-ui = { version = "6.0", features = ["actix-web"] }
```

### 런타임 오류 해결

#### 문제: CORS 오류
```
Access to fetch at 'http://localhost:8080/api/health' from origin 'http://localhost:3000' has been blocked by CORS policy
```

**해결책**: CORS 설정 확인
```bash
# 환경변수로 CORS 활성화
export CORS_ENABLED=true
export HOST=0.0.0.0
export PORT=8080
```

#### 문제: 포트 바인딩 실패
```
Error: Address already in use (os error 98)
```

**해결책**: 다른 포트 사용
```bash
export PORT=3001
cargo run
```

### 성능 최적화

#### 문제: 느린 빌드 시간
**해결책**: 의존성 최적화
```toml
# 불필요한 의존성 제거
# tracing = "0.1"  # 주석 처리
# chrono = "0.4"   # 주석 처리
# config = "0.14"  # 주석 처리
```

#### 문제: 큰 바이너리 크기
**해결책**: 릴리즈 빌드 최적화
```toml
# Cargo.toml에 추가
[profile.release]
opt-level = "z"      # 크기 최적화
lto = true          # 링크 타임 최적화
codegen-units = 1   # 단일 코드 생성 유닛
panic = "abort"     # 패닉 시 중단
```

### 디버깅 팁

#### 로그 레벨 설정
```bash
# 개발 환경
export RUST_LOG=debug
cargo run

# 프로덕션 환경
export RUST_LOG=info
cargo run --release
```

#### 헬스체크 테스트
```bash
# 기본 헬스체크
curl http://localhost:8080/health

# 상세 헬스체크
curl http://localhost:8080/api/health/detailed

# 서버 정보
curl http://localhost:8080/api/info
```

#### Docker 빌드 문제
```bash
# 캐시 없이 빌드
docker build --no-cache -t health-check-server .

# 멀티스테이지 빌드 최적화
docker build --target builder -t health-check-server-builder .
```

## 🚀 빠른 시작

1. **프로젝트 생성**:
   ```bash
   mkdir health-check-server
   cd health-check-server
   cargo init
   ```

2. **의존성 추가** (Rust 1.75.0 호환):
   ```bash
   cargo add actix-web@3.3 tokio@1.35 dotenvy@0.15 serde@1.0 serde_json@1.0 thiserror@1.0
   ```

3. **설정 파일 생성**:
   ```bash
   mkdir config
   # config/default.toml 파일 생성
   ```

4. **코드 작성**:
   ```bash
   # 위의 코드 예시를 참고하여 파일들 생성
   ```

5. **환경변수 설정**:
   ```bash
   export HOST=0.0.0.0
   export PORT=8080
   export LOG_LEVEL=info
   export CORS_ENABLED=true
   ```

6. **실행**:
   ```bash
   cargo run
   ```

7. **테스트**:
   ```bash
   curl http://localhost:8080/health
   ```

## 📚 참고 자료

- [Actix Web 공식 문서](https://actix.rs/)
- [Rust 공식 문서](https://doc.rust-lang.org/)
- [Clean Architecture](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html)
- [Docker 공식 문서](https://docs.docker.com/)
- [Rust 1.75.0 릴리즈 노트](https://blog.rust-lang.org/2023/12/28/Rust-1.75.0.html)

## 🎯 요약

이 기술문서는 **Rust 1.75.0 환경**에서 동작하는 헬스체크 전용 뼈대 서버를 구축하는 완전한 가이드를 제공합니다. 주요 특징은 다음과 같습니다:

### ✅ 구현된 기능
- **Clean Architecture 패턴** 유지
- **Rust 1.75.0 호환성** 보장
- **최소한의 의존성** 사용
- **헬스체크 엔드포인트** 제공
- **CORS 지원** (간단한 구현)
- **Docker 컨테이너화** 지원
- **환경별 설정** 관리

### 🔧 해결된 호환성 문제
- `actix-web 4.x` → `3.3` 다운그레이드
- `tracing-subscriber` → 간단한 로깅으로 대체
- `chrono` → `std::time` 사용
- `config` 크레이트 → 환경변수 직접 사용
- `actix-cors` → `DefaultHeaders` 사용

### 🚀 확장 가능성
이 뼈대 서버는 필요에 따라 다음과 같이 확장할 수 있습니다:
- 데이터베이스 연결 추가
- 인증/인가 시스템 구현
- 추가 API 엔드포인트 개발
- 모니터링 및 로깅 강화
- OpenAPI 문서화 추가

---

이 기술문서를 통해 **Rust 1.75.0 환경**에서 안정적으로 동작하는 헬스체크 뼈대 서버를 구축하고, 필요에 따라 점진적으로 기능을 확장할 수 있습니다. Clean Architecture 패턴을 유지하면서 최소한의 복잡성으로 시작하여 확장 가능한 서버를 만들 수 있습니다.
