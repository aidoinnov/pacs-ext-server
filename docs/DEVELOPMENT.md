# 개발 환경 설정

## 개발 환경

- **개발**: 맥 미니 (Mac Mini)
- **도커 빌드**: 개발 서버

## 프로젝트 구조

```
pacs-ext-server/
├── pacs-server/          # Rust 백엔드 서버
├── docs/                 # 문서
└── README.md
```

## 로컬 개발

### 필수 요구사항

- Rust (최신 stable 버전)
- PostgreSQL 14+
- Docker & Docker Compose

### 서버 실행

```bash
cd pacs-server
cargo run --bin pacs_server
```

### 빌드

```bash
cd pacs-server
cargo build --release
```

## 도커 빌드

도커 빌드는 개발 서버에서 수행합니다.

```bash
# 개발 서버에서 실행
docker build -t pacs-server .
docker-compose up -d
```

## API 엔드포인트

서버가 실행되면 다음 URL에서 접근 가능합니다:

- **서버**: http://localhost:8080
- **Swagger UI**: http://localhost:8080/swagger-ui/
- **Health Check**: http://localhost:8080/health
- **API**: http://localhost:8080/api/

## 주요 API

### 프로젝트 데이터 조회

- `GET /api/project-data/{project_id}/studies` - Study 목록 (직접 할당 + 규칙 기반)
- `GET /api/project-data/{project_id}/studies/{study_id}/series` - Series 목록
- `GET /api/project-data/{project_id}/series/{series_id}/instances` - Instance 목록

### 데이터 접근 권한

- `GET /api/projects/{project_id}/data-access/matrix` - 데이터 접근 매트릭스

