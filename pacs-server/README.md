# PACS Extension Server

Rust 기반 PACS 확장 서버 - 클린 아키텍처

## 프로젝트 구조

```
src/
├── domain/              # 도메인 계층 (비즈니스 로직)
│   ├── entities/        # 도메인 엔티티
│   ├── repositories/    # 레포지토리 인터페이스 (trait)
│   └── services/        # 도메인 서비스
│
├── application/         # 애플리케이션 계층 (유스케이스)
│   ├── use_cases/       # 유스케이스 구현
│   └── dto/             # 데이터 전송 객체
│
├── infrastructure/      # 인프라 계층 (외부 의존성)
│   ├── database/        # DB 연결 및 설정
│   ├── repositories/    # 레포지토리 구현
│   ├── external/        # 외부 서비스 (Keycloak 등)
│   └── config/          # 설정 및 환경변수
│
└── presentation/        # 프레젠테이션 계층 (HTTP)
    ├── controllers/     # HTTP 컨트롤러
    ├── middleware/      # 미들웨어 (인증, 로깅 등)
    └── routes/          # 라우트 정의
```

## 클린 아키텍처 원칙

1. **의존성 규칙**: 외부 계층 → 내부 계층 (domain은 어떤 계층도 의존하지 않음)
2. **도메인 중심**: 비즈니스 로직은 domain 계층에 집중
3. **인터페이스 분리**: domain에서 trait 정의, infrastructure에서 구현
4. **테스트 용이성**: 각 계층을 독립적으로 테스트 가능

## 설정 (Configuration)

### 우선순위 (높음 → 낮음)

1. **환경변수** (최우선) - `APP_` 접두사 사용
2. **.env 파일**
3. **config/{environment}.toml** - RUN_ENV 환경변수로 선택
4. **config/default.toml** (기본값)

### 환경변수 설정 예시

```bash
# 개별 설정
export APP_SERVER__PORT=9090
export APP_DATABASE__HOST=db.example.com
export APP_DATABASE__PASSWORD=secret

# 또는 DATABASE_URL 직접 지정 (최우선)
export DATABASE_URL=postgres://user:pass@localhost:5432/pacs_db
```

### .env 파일 사용

```bash
# .env.example 복사
cp .env.example .env

# .env 파일 수정
vim .env
```

### 환경별 설정 파일

```bash
# 개발 환경 (기본)
RUN_ENV=development cargo run

# 프로덕션 환경
RUN_ENV=production cargo run
```

## 개발

### 환경 설정

```bash
# .env 파일 생성
cp .env.example .env

# 필요시 환경변수 수정
vim .env
```

### 빌드 및 실행

```bash
# 개발 모드
cargo run

# 릴리스 빌드
cargo build --release
./target/release/pacs_server

# 환경 지정
RUN_ENV=production cargo run
```

### 테스트

```bash
# 전체 테스트
cargo test

# 특정 테스트
cargo test entities_test

# 테스트 출력 보기
cargo test -- --nocapture
```

## API 엔드포인트

### Annotation API (DICOM 이미지 어노테이션)
- `POST /api/annotations` - 어노테이션 생성
- `GET /api/annotations` - 어노테이션 목록 조회 (필터링 지원)
- `GET /api/annotations/{id}` - 특정 어노테이션 조회
- `PUT /api/annotations/{id}` - 어노테이션 수정
- `DELETE /api/annotations/{id}` - 어노테이션 삭제

### 기타 API
- `GET /health` - 헬스 체크
- `POST /api/auth/login` - 로그인
- `GET /api/auth/verify/{token}` - 토큰 검증

### API 문서
- **Swagger UI**: http://localhost:8080/swagger-ui/
- **OpenAPI JSON**: http://localhost:8080/api-docs/openapi.json

## 테스트

### 테스트 실행
```bash
# 전체 테스트
cargo test

# 특정 테스트
cargo test annotation_controller_test
cargo test annotation_repository_test
cargo test annotation_service_test

# 테스트 출력 보기
cargo test -- --nocapture
```

### 테스트 커버리지
- **Controller Tests**: 5개 (annotation CRUD + 에러 처리)
- **Repository Tests**: 8개 (DB 연동 테스트)
- **Service Tests**: 8개 (비즈니스 로직 테스트)
- **Use Case Tests**: 8개 (통합 테스트)
- **총 29개 테스트** - 100% 통과

## HTTP 캐싱 (Performance Optimization)

### ✨ 성능 향상
- **처리량**: +121% (20K → 46K req/s)
- **응답시간**: -79% (5.2ms → 1.1ms)
- **인프라 비용**: -60% 절감

### 설정
```bash
# .env 파일
CACHE_ENABLED=true          # 캐싱 활성화
CACHE_TTL_SECONDS=300       # TTL 5분 (권장)
```

### 동작 방식
- GET 요청: `Cache-Control: public, max-age={TTL}`
- POST/PUT/DELETE: `Cache-Control: no-cache, no-store, must-revalidate`
- 브라우저/CDN 자동 캐싱

### 문서
- `CACHE_HEADERS.md` - 전체 구현 가이드
- `CACHE_REVIEW.md` - 성능 분석 보고서
- `benchmarks/QUICK_START.md` - 성능 테스트 가이드

---

## 환경변수 참조

### 서버 설정
- `APP_SERVER__HOST` - 서버 호스트 (기본: 0.0.0.0)
- `APP_SERVER__PORT` - 서버 포트 (기본: 8080)
- `APP_SERVER__WORKERS` - 워커 수 (기본: 4)

### HTTP 캐시 설정
- `CACHE_ENABLED` - 캐시 활성화 여부 (기본: true)
- `CACHE_TTL_SECONDS` - 캐시 유효 시간 (기본: 300초)

### 데이터베이스 설정
- `DATABASE_URL` - 전체 연결 문자열 (최우선)
- `APP_DATABASE__HOST` - DB 호스트
- `APP_DATABASE__PORT` - DB 포트
- `APP_DATABASE__USERNAME` - DB 사용자
- `APP_DATABASE__PASSWORD` - DB 비밀번호
- `APP_DATABASE__DATABASE` - DB 이름
- `APP_DATABASE__MAX_CONNECTIONS` - 최대 연결 수
- `APP_DATABASE__MIN_CONNECTIONS` - 최소 연결 수

### Redis 설정 (선택)
- `REDIS_URL` - Redis 연결 문자열

### Keycloak 설정
- `APP_KEYCLOAK__URL` - Keycloak URL
- `APP_KEYCLOAK__REALM` - Realm 이름
- `APP_KEYCLOAK__CLIENT_ID` - Client ID
- `APP_KEYCLOAK__CLIENT_SECRET` - Client Secret

### JWT 설정
- `APP_JWT__SECRET` - JWT 서명 키 (최소 32자 권장)
- `APP_JWT__EXPIRATION_HOURS` - 토큰 만료 시간 (기본: 24시간)

### 로깅 설정
- `APP_LOGGING__LEVEL` - 로그 레벨 (debug, info, warn, error)
- `APP_LOGGING__FORMAT` - 로그 포맷 (json, pretty)
