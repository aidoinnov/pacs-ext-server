# 🚀 PACS Server 실행 가이드

## 📋 목차
- [사전 요구사항](#사전-요구사항)
- [환경 설정](#환경-설정)
- [서버 실행](#서버-실행)
- [서버 모드](#서버-모드)
- [문제 해결](#문제-해결)

---

## 사전 요구사항

### 필수 소프트웨어
- **Rust** (1.70 이상)
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```
- **PostgreSQL** (14 이상)
- **Redis** (선택사항, 캐싱용)

### 데이터베이스 설정
```bash
# PostgreSQL 데이터베이스 생성
createdb pacs

# 마이그레이션 실행
cd pacs-server
sqlx migrate run
```

---

## 환경 설정

### 1. 환경 변수 파일 생성
`.env` 파일을 생성하고 다음 내용을 설정합니다:

```bash
# 데이터베이스
DATABASE_URL=postgresql://user:password@localhost/pacs

# 서버 설정
SERVER_HOST=0.0.0.0
SERVER_PORT=8080
SERVER_MODE=full  # full, sync_only, api_only

# JWT 인증
JWT_SECRET=your-secret-key-here
JWT_EXPIRATION=3600

# Object Storage (AWS S3 또는 MinIO)
AWS_ACCESS_KEY_ID=your-access-key
AWS_SECRET_ACCESS_KEY=your-secret-key
AWS_REGION=ap-northeast-2
S3_BUCKET_NAME=pacs-masks

# Redis (선택사항)
REDIS_URL=redis://localhost:6379

# Keycloak (선택사항)
KEYCLOAK_URL=http://localhost:8081
KEYCLOAK_REALM=pacs
KEYCLOAK_CLIENT_ID=pacs-server
```

### 2. 의존성 설치
```bash
cd pacs-server
cargo build --release
```

---

## 서버 실행

### 개발 모드
```bash
cd pacs-server
cargo run
```

### 릴리스 모드 (프로덕션)
```bash
cd pacs-server
cargo run --release
```

### 백그라운드 실행
```bash
cd pacs-server
nohup cargo run --release > server.log 2>&1 &
```

### 루트에서 전체 시스템 실행
```bash
cd /path/to/pacs-ext-server
./start-all.sh
```

---

## 서버 모드

### 1. Full Mode (기본)
API 서버 + DICOM Sync 모두 실행
```bash
SERVER_MODE=full cargo run --release
```

### 2. API Only Mode
API 서버만 실행 (DICOM Sync 비활성화)
```bash
SERVER_MODE=api_only cargo run --release
```

### 3. Sync Only Mode
DICOM Sync만 실행 (API 서버 비활성화)
```bash
SERVER_MODE=sync_only cargo run --release
```

---

## 서버 확인

### Health Check
```bash
curl http://localhost:8080/health
```

**응답 예시:**
```json
{
  "status": "healthy",
  "version": "0.1.0",
  "timestamp": "2026-01-19T12:00:00Z"
}
```

### Swagger UI
브라우저에서 API 문서 확인:
```
http://localhost:8080/swagger-ui/
```

---

## 문제 해결

### 포트 충돌
```bash
# 포트 사용 중인 프로세스 확인
lsof -i :8080

# 프로세스 종료
kill -9 <PID>
```

### 데이터베이스 연결 실패
```bash
# PostgreSQL 상태 확인
pg_isready

# 연결 테스트
psql -U user -d pacs -c "SELECT 1"
```

### 마이그레이션 오류
```bash
# 마이그레이션 상태 확인
sqlx migrate info

# 마이그레이션 재실행
sqlx migrate revert
sqlx migrate run
```

### 로그 확인
```bash
# 실시간 로그 확인
tail -f server.log

# 에러 로그만 확인
tail -f server.log | grep ERROR
```

---

## 관련 문서

- [테스트 가이드](./TESTING.md)
- [도구 스크립트 가이드](./TOOLS.md)
- [API 문서](../docs/api/)
- [개발 가이드](../docs/DEVELOPMENT.md)

