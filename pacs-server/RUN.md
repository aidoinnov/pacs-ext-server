# 🚀 PACS Server 실행 가이드

## 📋 목차
- [사전 요구사항](#사전-요구사항)
- [환경 설정](#환경-설정)
- [서버 실행](#서버-실행)
- [서버 모드](#서버-모드)
- [DICOM 동기화](#dicom-동기화)
- [Subject 생성](#subject-생성)
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

## DICOM 동기화

PACS Extension Server는 Dcm4chee PACS 데이터베이스와 자동으로 동기화하는 기능을 제공합니다.

### 동기화 상태 확인

```bash
# 동기화 상태 확인
curl http://localhost:8080/api/sync/status | jq .
```

**응답 예시:**
```json
{
  "is_running": false,
  "last_run": "2026-01-20T09:50:00Z",
  "next_run": "2026-01-20T09:55:00Z",
  "interval_sec": 300
}
```

**필드 설명:**
- `is_running`: 현재 동기화 실행 중 여부
- `last_run`: 마지막 동기화 실행 시간
- `next_run`: 다음 자동 동기화 예정 시간
- `interval_sec`: 자동 동기화 주기 (초)

### 수동 동기화 실행

```bash
# 즉시 동기화 실행
curl -X POST http://localhost:8080/api/sync/run
```

**응답 예시:**
```json
{
  "success": true,
  "processed": 1089,
  "duration_ms": 32407,
  "error": null
}
```

**참고:** 동기화는 백그라운드에서 실행되며, 대량의 데이터가 있을 경우 60초 이상 걸릴 수 있습니다.

### 동기화 일시정지/재개

```bash
# 자동 동기화 일시정지
curl -X POST http://localhost:8080/api/sync/pause

# 자동 동기화 재개
curl -X POST http://localhost:8080/api/sync/resume
```

### 동기화 주기 변경

```bash
# 동기화 주기를 600초(10분)로 변경
curl -X PUT http://localhost:8080/api/sync/schedule \
  -H "Content-Type: application/json" \
  -d '{"interval_sec": 600}'
```

### 동기화 로그 확인

```bash
# 동기화 관련 로그만 확인
tail -f ../backend.log | grep -i sync

# 또는 pacs-server 디렉토리에서
tail -f server.log | grep -i sync
```

**로그 예시:**
```
🔄 Initializing Sync service... ✅ Done (Interval: 300s)
🔄 Sync scheduler started (Mode: Full)
🔄 [Sync] run_once() called
🔄 [Sync] Starting sync_studies...
🔄 [Sync] sync_studies completed: 24 studies
🔄 [Sync] Starting cleanup of missing data...
🔄 [Sync] Deleted 0 missing instances
🔄 [Sync] Cleanup completed: 0 items deleted
```

### 동기화 데이터 확인

```bash
# Study 개수 확인
curl http://localhost:8080/api/projects/1/studies | jq '.data | length'

# Series 개수 확인
curl http://localhost:8080/api/projects/1/series | jq '.data | length'
```

---

## Subject 생성

프로젝트에 할당된 Study들에 대해 Subject를 자동으로 생성할 수 있습니다.

### 스크립트 실행

```bash
# 프로젝트 루트에서 실행
cd /path/to/pacs-ext-server

# 특정 프로젝트에 Subject 생성
./scripts/create_subjects.sh --project-id 1

# 모든 프로젝트에 Subject 생성
./scripts/create_subjects.sh --all-projects

# Dry-run 모드 (실제 생성하지 않고 시뮬레이션만)
./scripts/create_subjects.sh --project-id 1 --dry-run
```

### 실행 결과 예시

```
✓ Activating virtual environment...
Starting Subject creation...

============================================================
Project: AI Image Analysis Project (ID: 2)
============================================================
Found 3 studies total
  - With patient_id: 3
  - Without patient_id: 0
  ✓ Created Subject: DEMO for WEBPACS_A001 (ID: 174, Patient: DEMO for WEBPACS_A001)
  ✓ Reuse Subject: DEMO for WEBPACS_A001 (Patient: DEMO for WEBPACS_A001)
  ✓ Reuse Subject: DEMO for WEBPACS_A001 (Patient: DEMO for WEBPACS_A001)

Summary:
  - Created: 1
  - Reused: 2
  - Total: 3

✓ Done!
```

### Subject 확인

```bash
# 프로젝트의 Subject 목록 확인
curl http://localhost:8080/api/projects/1/subjects | jq .
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

### 동기화 관련 문제

#### 동기화가 실행되지 않는 경우

1. **서버 모드 확인**
   ```bash
   # .env 파일에서 SERVER_MODE 확인
   cat .env | grep SERVER_MODE
   # full 또는 sync_only 여야 함
   ```

2. **Dcm4chee DB 연결 확인**
   ```bash
   # 로그에서 연결 에러 확인
   tail -f ../backend.log | grep -i "dcm4chee\|sync"
   ```

3. **동기화 상태 확인**
   ```bash
   curl http://localhost:8080/api/sync/status | jq .
   ```

4. **수동으로 동기화 실행**
   ```bash
   curl -X POST http://localhost:8080/api/sync/run
   ```

#### 동기화가 느린 경우

- 동기화는 대량의 데이터를 처리할 때 시간이 걸릴 수 있습니다
- 처리량:
  - Study: 최대 500개/실행
  - Series: 최대 1000개/실행
  - Instance: 최대 2000개/실행
- 백그라운드에서 실행되므로 API 타임아웃(60초)이 발생해도 계속 진행됩니다

---

## 관련 문서

- [테스트 가이드](./TESTING.md)
- [도구 스크립트 가이드](./TOOLS.md)
- [API 문서](../docs/api/)
- [개발 가이드](../docs/DEVELOPMENT.md)
- [DICOM 동기화 상세 문서](../docs/technical/db-sync/README.md)

