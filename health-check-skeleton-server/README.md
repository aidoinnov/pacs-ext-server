# Health Check Server

최소한의 헬스체크 기능을 제공하는 뼈대 서버입니다. Clean Architecture 패턴을 따르며 확장 가능한 구조를 유지합니다.

## 🚀 주요 기능

- ✅ **서버 상태 확인** (Health Check)
- ✅ **CORS 지원**
- ✅ **구조화된 로깅**
- ✅ **환경별 설정 관리**
- ✅ **Docker 지원**
- ✅ **Clean Architecture 패턴**

## 📋 요구사항

- Rust 1.75+
- Cargo
- Docker (선택사항)

## 🛠️ 설치 및 실행

### 로컬 개발 환경

1. **저장소 클론**
   ```bash
   git clone <repository-url>
   cd health-check-skeleton-server
   ```

2. **의존성 설치**
   ```bash
   make deps
   # 또는
   cargo build
   ```

3. **개발 모드 실행**
   ```bash
   make dev
   # 또는
   RUN_MODE=development cargo run
   ```

4. **서버 상태 확인**
   ```bash
   make health
   # 또는
   curl http://localhost:3000/health
   ```

### Docker를 사용한 실행

1. **Docker 이미지 빌드**
   ```bash
   make docker-build
   # 또는
   docker build -t health-check-server .
   ```

2. **Docker Compose로 실행**
   ```bash
   make docker-run
   # 또는
   docker-compose up --build
   ```

3. **개발 환경으로 실행**
   ```bash
   make docker-dev
   # 또는
   docker-compose --profile dev up --build
   ```

## 📁 프로젝트 구조

```
health-check-skeleton-server/
├── src/
│   ├── main.rs                 # 서버 엔트리 포인트
│   ├── lib.rs                  # 라이브러리 루트
│   ├── domain/                 # 도메인 계층
│   │   ├── mod.rs
│   │   └── errors.rs          # 에러 정의
│   ├── infrastructure/         # 인프라스트럭처 계층
│   │   ├── mod.rs
│   │   ├── config/            # 설정 관리
│   │   │   ├── mod.rs
│   │   │   └── settings.rs
│   │   └── middleware/        # 미들웨어
│   │       ├── mod.rs
│   │       └── cors_middleware.rs
│   └── presentation/          # 프레젠테이션 계층
│       ├── mod.rs
│       └── controllers/       # 컨트롤러
│           ├── mod.rs
│           └── health_controller.rs
├── config/                    # 설정 파일
│   ├── default.toml
│   ├── development.toml
│   └── production.toml
├── Cargo.toml                 # Rust 의존성
├── Dockerfile                 # Docker 이미지 정의
├── docker-compose.yml         # Docker Compose 설정
├── Makefile                   # 빌드 자동화
└── README.md                  # 프로젝트 문서
```

## 🔧 설정

### 환경 변수

```bash
# 서버 설정
HOST=0.0.0.0
PORT=8080
WORKERS=4

# 로깅 설정
LOG_LEVEL=info
LOG_FORMAT=json

# CORS 설정
CORS_ENABLED=true

# 애플리케이션 모드
RUN_MODE=development
```

### 설정 파일

- `config/default.toml`: 기본 설정
- `config/development.toml`: 개발 환경 설정
- `config/production.toml`: 프로덕션 환경 설정

## 🌐 API 엔드포인트

### 기본 헬스체크
- **GET** `/health` - 기본 서버 상태 확인

### 상세 API
- **GET** `/api/health/detailed` - 상세한 서버 상태 정보
- **GET** `/api/health/simple` - 간단한 상태 확인
- **GET** `/api/health/validate` - 서버 상태 검증
- **GET** `/api/info` - 서버 정보 및 엔드포인트 목록

### 응답 예시

#### 기본 헬스체크
```json
{
  "status": "healthy",
  "service": "health-check-server",
  "timestamp": "2024-01-15T10:30:00Z",
  "version": "0.1.0"
}
```

#### 상세 헬스체크
```json
{
  "status": "healthy",
  "service": "health-check-server",
  "version": "0.1.0",
  "timestamp": "2024-01-15T10:30:00Z",
  "uptime": 1705312200,
  "environment": "development",
  "features": {
    "health_check": true,
    "cors": true,
    "logging": true
  }
}
```

## 🛠️ 개발 명령어

```bash
# 도움말 보기
make help

# 빌드
make build

# 실행
make run

# 개발 모드 실행
make dev

# 테스트 실행
make test

# 코드 포맷팅
make fmt

# 린터 실행
make clippy

# 정리
make clean

# 헬스체크
make health
```

## 🐳 Docker 명령어

```bash
# Docker 이미지 빌드
make docker-build

# Docker Compose로 실행
make docker-run

# 개발 환경으로 실행
make docker-dev

# 테스트 환경으로 실행
make docker-test
```

## 🔍 모니터링

### 헬스체크

서버의 상태를 확인하려면 다음 엔드포인트를 사용하세요:

```bash
# 기본 헬스체크
curl http://localhost:8080/health

# 상세 헬스체크
curl http://localhost:8080/api/health/detailed

# 서버 정보
curl http://localhost:8080/api/info
```

### 로깅

로그 레벨을 환경 변수로 제어할 수 있습니다:

```bash
# 디버그 로그
RUST_LOG=debug cargo run

# 특정 모듈만 로그
RUST_LOG=health_check_server=debug,actix_web=info cargo run
```

## 🚀 확장 가이드

### 새로운 엔드포인트 추가

1. **컨트롤러 생성**:
   ```rust
   // src/presentation/controllers/new_controller.rs
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
   // src/main.rs
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

## 📝 라이선스

MIT License

## 🤝 기여하기

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## 📞 지원

문제가 발생하거나 질문이 있으시면 이슈를 생성해 주세요.

---

**Health Check Server** - Clean Architecture로 구축된 최소한의 헬스체크 서버
