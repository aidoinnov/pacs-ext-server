# PACS Extension Server - Docker 컨테이너화 가이드

## 개요

PACS Extension Server를 Docker 컨테이너로 배포하기 위한 환경변수 설정 및 컨테이너화 가이드입니다.

## 현재 상태 분석

### ✅ 이미 환경변수로 설정된 항목들

| 카테고리 | 환경변수 | 설명 | 기본값 |
|---------|---------|------|--------|
| **데이터베이스** | `DATABASE_URL` | 전체 연결 문자열 (최우선) | - |
| | `APP_DATABASE_HOST` | 데이터베이스 호스트 | localhost |
| | `APP_DATABASE_PORT` | 데이터베이스 포트 | 5432 |
| | `APP_DATABASE_USERNAME` | 데이터베이스 사용자명 | admin |
| | `APP_DATABASE_PASSWORD` | 데이터베이스 비밀번호 | admin123 |
| | `APP_DATABASE_DATABASE` | 데이터베이스 이름 | pacs_db |
| | `APP_DATABASE_MAX_CONNECTIONS` | 최대 연결 수 | 10 |
| | `APP_DATABASE_MIN_CONNECTIONS` | 최소 연결 수 | 2 |
| **서버** | `APP_SERVER_HOST` | 서버 호스트 | 0.0.0.0 |
| | `APP_SERVER_PORT` | 서버 포트 | 8080 |
| | `APP_SERVER_WORKERS` | 워커 스레드 수 | 4 |
| **JWT** | `APP_JWT_SECRET` | JWT 서명 키 | your-secret-key-change-this-in-production |
| | `APP_JWT_EXPIRATION_HOURS` | JWT 만료 시간(시간) | 24 |
| **Keycloak** | `APP_KEYCLOAK_URL` | Keycloak 서버 URL | http://localhost:8080 |
| | `APP_KEYCLOAK_REALM` | Keycloak Realm | pacs |
| | `APP_KEYCLOAK_CLIENT_ID` | Keycloak 클라이언트 ID | pacs-server |
| | `APP_KEYCLOAK_CLIENT_SECRET` | Keycloak 클라이언트 시크릿 | "" |
| **CORS** | `APP_CORS_ENABLED` | CORS 활성화 여부 | false |
| | `APP_CORS_ALLOWED_ORIGINS` | 허용된 Origin 목록 | ["http://localhost:3000", "http://localhost:8080"] |
| | `APP_CORS_ALLOWED_METHODS` | 허용된 HTTP 메서드 | ["GET", "POST", "PUT", "DELETE", "OPTIONS"] |
| | `APP_CORS_ALLOWED_HEADERS` | 허용된 헤더 | ["Content-Type", "Authorization", "X-Requested-With"] |
| | `APP_CORS_EXPOSE_HEADERS` | 노출할 헤더 | ["Content-Length", "X-Total-Count"] |
| | `APP_CORS_MAX_AGE` | Preflight 캐시 시간(초) | 3600 |
| **Object Storage** | `APP_OBJECT_STORAGE_PROVIDER` | 저장소 제공자 | s3 |
| | `APP_OBJECT_STORAGE_BUCKET_NAME` | 버킷 이름 | pacs-masks |
| | `APP_OBJECT_STORAGE_REGION` | AWS 리전 | us-east-1 |
| | `APP_OBJECT_STORAGE_ENDPOINT` | MinIO 엔드포인트 | "" |
| | `APP_OBJECT_STORAGE_ACCESS_KEY_ID` | 액세스 키 ID | "" |
| | `APP_OBJECT_STORAGE_SECRET_ACCESS_KEY` | 시크릿 액세스 키 | "" |
| **Signed URL** | `APP_SIGNED_URL_DEFAULT_TTL` | 기본 TTL(초) | 600 |
| | `APP_SIGNED_URL_MAX_TTL` | 최대 TTL(초) | 3600 |
| **로깅** | `APP_LOGGING_LEVEL` | 로그 레벨 | info |
| | `APP_LOGGING_FORMAT` | 로그 포맷 | json |

### ⚠️ 환경변수로 빼야 할 하드코딩된 설정들

| 파일 | 라인 | 현재 하드코딩된 값 | 환경변수로 변경 필요 |
|------|------|------------------|-------------------|
| **main.rs** | 289 | `"http://0.0.0.0:8080"` | `settings.server.host:port` 사용 |
| | 290 | `"http://0.0.0.0:8080/swagger-ui/"` | `settings.server.host:port` 사용 |
| | 291 | `"http://0.0.0.0:8080/health"` | `settings.server.host:port` 사용 |
| | 292 | `"http://0.0.0.0:8080/api/"` | `settings.server.host:port` 사용 |
| **cors_middleware.rs** | 97 | `"http://localhost:3000"` | `settings.cors.allowed_origins` 사용 |
| **openapi.rs** | 88 | `"http://localhost:8080"` | `settings.server.host:port` 사용 |
| | 89 | `"http://0.0.0.0:8080"` | `settings.server.host:port` 사용 |
| **settings.rs** | 143 | `"0.0.0.0"` | 환경변수 기본값 |
| | 148 | `"localhost"` | 환경변수 기본값 |
| | 157 | `"http://localhost:8080"` | 환경변수 기본값 |
| | 172 | `["http://localhost:3000"]` | 환경변수 기본값 |

### 🔧 추가로 필요한 환경변수들

| 환경변수 | 설명 | 기본값 | 용도 |
|---------|------|--------|------|
| `S3_ACCESS_KEY` | S3 액세스 키 (테스트용) | minioadmin | 테스트 환경 |
| `S3_SECRET_KEY` | S3 시크릿 키 (테스트용) | minioadmin | 테스트 환경 |
| `S3_ENDPOINT` | S3 엔드포인트 (테스트용) | http://localhost:9000 | 테스트 환경 |
| `S3_BUCKET` | S3 버킷 (테스트용) | pacs-test | 테스트 환경 |
| `S3_REGION` | S3 리전 (테스트용) | us-east-1 | 테스트 환경 |
| `REDIS_URL` | Redis 연결 URL | redis://localhost:6379 | 캐싱 (선택사항) |
| `APP_ENVIRONMENT` | 실행 환경 | development | 환경 구분 |
| `APP_LOG_LEVEL` | 로그 레벨 | info | 로깅 제어 |

## Docker 컨테이너화 작업 계획

### Phase 1: 하드코딩 제거
1. **main.rs 수정**
   - 로그 메시지에서 하드코딩된 URL 제거
   - 설정에서 동적으로 URL 생성

2. **cors_middleware.rs 수정**
   - 하드코딩된 Origin 제거
   - 설정에서 동적으로 가져오기

3. **openapi.rs 수정**
   - 하드코딩된 서버 URL 제거
   - 설정에서 동적으로 생성

4. **settings.rs 수정**
   - 기본값들을 환경변수로 대체
   - 더 나은 기본값 설정

### Phase 2: Docker 파일 생성
1. **Dockerfile 작성**
   - Multi-stage build 사용
   - 최적화된 이미지 크기
   - 보안 설정

2. **docker-compose.yml 작성**
   - 서비스 정의 (app, postgres, redis, minio)
   - 네트워크 설정
   - 볼륨 마운트

3. **환경별 설정 파일**
   - `.env.development`
   - `.env.production`
   - `.env.test`

### Phase 3: 배포 최적화
1. **Health Check 추가**
2. **Graceful Shutdown 개선**
3. **로깅 최적화**
4. **모니터링 설정**

## 환경변수 우선순위

1. **환경변수** (최우선)
2. **.env 파일**
3. **config/{environment}.toml**
4. **config/default.toml** (최하위)

## 보안 고려사항

### 민감한 정보
- `APP_JWT_SECRET`
- `APP_DATABASE_PASSWORD`
- `APP_KEYCLOAK_CLIENT_SECRET`
- `APP_OBJECT_STORAGE_SECRET_ACCESS_KEY`

### 권장사항
1. **Docker Secrets 사용** (프로덕션)
2. **환경변수 파일 분리** (.env.production)
3. **기본값 제거** (민감한 정보)
4. **설정 검증** (시작시 필수 값 확인)

## 테스트 환경 설정

### 개발 환경
```bash
# .env.development
APP_ENVIRONMENT=development
APP_SERVER_HOST=0.0.0.0
APP_SERVER_PORT=8080
DATABASE_URL=postgres://admin:admin123@localhost:5432/pacs_db
JWT_SECRET=dev-secret-key
S3_ENDPOINT=http://localhost:9000
S3_ACCESS_KEY=minioadmin
S3_SECRET_KEY=minioadmin
```

### 프로덕션 환경
```bash
# .env.production
APP_ENVIRONMENT=production
APP_SERVER_HOST=0.0.0.0
APP_SERVER_PORT=8080
DATABASE_URL=postgres://user:password@postgres:5432/pacs_production
JWT_SECRET=${JWT_SECRET}  # Docker Secret에서 가져오기
S3_ENDPOINT=https://s3.amazonaws.com
S3_ACCESS_KEY=${AWS_ACCESS_KEY_ID}
S3_SECRET_KEY=${AWS_SECRET_ACCESS_KEY}
```

## 구현 체크리스트

### 하드코딩 제거
- [ ] main.rs 로그 메시지 수정
- [ ] cors_middleware.rs Origin 수정
- [ ] openapi.rs 서버 URL 수정
- [ ] settings.rs 기본값 정리

### Docker 파일
- [ ] Dockerfile 작성
- [ ] .dockerignore 작성
- [ ] docker-compose.yml 작성
- [ ] 환경별 .env 파일 생성

### 테스트
- [ ] Docker 빌드 테스트
- [ ] 컨테이너 실행 테스트
- [ ] 환경변수 주입 테스트
- [ ] 통합 테스트 실행

### 문서화
- [ ] Docker 사용법 문서
- [ ] 배포 가이드 작성
- [ ] 환경변수 목록 문서화
- [ ] 트러블슈팅 가이드

## 참고사항

- 모든 환경변수는 `APP_` 접두사 사용
- 민감한 정보는 Docker Secrets 또는 외부 시크릿 관리 시스템 사용
- 개발/테스트/프로덕션 환경별로 다른 설정 파일 사용
- 컨테이너 시작시 필수 환경변수 검증 로직 추가 필요
