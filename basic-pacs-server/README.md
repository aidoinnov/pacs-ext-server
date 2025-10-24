# Basic PACS Server

Health check만 포함하는 기초 백엔드 서버입니다. Clean Architecture 패턴을 따르며 확장 가능한 구조를 유지합니다.

## 🏗️ 아키텍처

이 프로젝트는 Clean Architecture 패턴을 따릅니다:

```
src/
├── domain/           # 도메인 계층 (비즈니스 로직)
├── application/      # 애플리케이션 계층 (유스케이스)
├── infrastructure/   # 인프라스트럭처 계층 (외부 의존성)
└── presentation/     # 프레젠테이션 계층 (HTTP API)
```

## 🚀 주요 기능

- ✅ 서버 상태 확인 (Health Check)
- ✅ 서버 정보 조회
- ✅ CORS 지원
- ✅ 구조화된 로깅
- ✅ 환경별 설정 관리
- ✅ Docker 지원
- ✅ Clean Architecture 패턴

## 📋 요구사항

- Rust 1.75+
- Cargo
- Docker (선택사항)

## 🛠️ 설치 및 실행

### 1. 로컬 실행

```bash
# 저장소 클론
git clone <repository-url>
cd basic-pacs-server

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

### 3. 환경 변수 설정

```bash
# .env 파일 생성
cp env.example .env

# 필요한 값들 설정
vim .env
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
  "language": "Rust",
  "timestamp": "2024-01-01T00:00:00Z"
}
```

### 3. API Health Check

```bash
GET /api/health
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

### 4. API Server Info

```bash
GET /api/info
```

**응답:**
```json
{
  "name": "Basic PACS Server",
  "version": "0.1.0",
  "description": "Health check만 포함하는 기초 백엔드 서버",
  "architecture": "Clean Architecture",
  "framework": "Actix Web",
  "language": "Rust",
  "timestamp": "2024-01-01T00:00:00Z"
}
```

## 🧪 테스트

```bash
# 단위 테스트
cargo test

# 통합 테스트
cargo test --test integration_tests

# 모든 테스트
cargo test --all
```

## 🔧 설정

### 환경 변수

| 변수명 | 기본값 | 설명 |
|--------|--------|------|
| `RUN_MODE` | `development` | 실행 모드 |
| `HOST` | `127.0.0.1` | 서버 호스트 |
| `PORT` | `8080` | 서버 포트 |
| `WORKERS` | `2` | 워커 수 |
| `LOG_LEVEL` | `info` | 로그 레벨 |
| `CORS_ENABLED` | `true` | CORS 활성화 |
| `CORS_ORIGINS` | `*` | 허용된 오리진 |

### 설정 파일

- `config/default.toml`: 기본 설정
- `config/development.toml`: 개발 환경 설정
- `config/production.toml`: 프로덕션 환경 설정

## 🐳 Docker

### Dockerfile

멀티스테이지 빌드를 사용하여 최적화된 이미지를 생성합니다.

### docker-compose.yml

개발 환경을 위한 Docker Compose 설정을 제공합니다.

## 📚 코드 구조

### Domain Layer

- `entities/`: 도메인 엔티티
- `services/`: 도메인 서비스 인터페이스
- `repositories/`: 리포지토리 인터페이스
- `errors.rs`: 도메인 에러

### Application Layer

- `use_cases/`: 유스케이스 구현
- `dto/`: 데이터 전송 객체

### Infrastructure Layer

- `config/`: 설정 관리
- `middleware/`: HTTP 미들웨어

### Presentation Layer

- `controllers/`: HTTP 컨트롤러

## 🔍 개발 가이드

### 1. 새로운 기능 추가

1. Domain 계층에 엔티티/서비스 정의
2. Application 계층에 유스케이스 구현
3. Infrastructure 계층에 외부 의존성 구현
4. Presentation 계층에 컨트롤러 구현

### 2. 에러 처리

- Domain 계층에서 `DomainError` 사용
- 적절한 에러 변환 구현
- HTTP 상태 코드 매핑

### 3. 테스트 작성

- 단위 테스트: 각 계층별 테스트
- 통합 테스트: API 엔드포인트 테스트
- 모킹: 외부 의존성 모킹

## 🚀 배포

### 1. 로컬 배포

```bash
# 릴리즈 빌드
cargo build --release

# 바이너리 실행
./target/release/basic-pacs-server
```

### 2. Docker 배포

```bash
# 이미지 빌드
docker build -t basic-pacs-server .

# 컨테이너 실행
docker run -d -p 8080:8080 --name basic-pacs-server basic-pacs-server
```

### 3. Docker Compose 배포

```bash
# 백그라운드 실행
docker-compose up -d

# 로그 확인
docker-compose logs -f

# 중지
docker-compose down
```

## 📝 라이선스

MIT License

## 🤝 기여

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## 📞 지원

문제가 있거나 질문이 있으시면 이슈를 생성해주세요.

---

이 프로젝트는 Clean Architecture 패턴을 학습하고 이해하는데 도움이 됩니다. 각 계층의 역할과 의존성 방향을 명확히 이해하는 것이 중요합니다.
