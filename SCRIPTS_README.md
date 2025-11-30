# PACS Extension Server - 관리 스크립트

DB 터널, 백엔드(Rust/Actix-web), 프론트엔드(React) 서버를 쉽게 관리할 수 있는 스크립트 모음입니다.

## 📋 목차

- [스크립트 목록](#스크립트-목록)
- [사용법](#사용법)
- [상세 설명](#상세-설명)
- [문제 해결](#문제-해결)

## 🚀 스크립트 목록

| 스크립트 | 설명 | 용도 |
|---------|------|------|
| `start-all.sh` | 전체 시스템 시작 | DB 터널 + 백엔드 + 프론트엔드 동시 실행 |
| `stop-all.sh` | 전체 시스템 종료 | 모든 서비스 안전하게 종료 |
| `restart-all.sh` | 전체 시스템 재시작 | 종료 후 다시 시작 |
| `status-all.sh` | 시스템 상태 확인 | 실행 상태, 메모리, CPU 확인 |
| `scripts/db-tunnel.sh` | DB 터널만 실행 | AWS RDS SSH 터널링 |

## 📖 사용법

### 1. 전체 시스템 시작

```bash
./start-all.sh
```

**동작:**
- 기존 프로세스 확인 및 종료
- 포트 5456, 5457, 8080, 3000 정리
- DB 터널 시작 (AWS RDS 연결)
- 백엔드 빌드 및 실행
- 프론트엔드 실행
- 브라우저 자동 열기 (http://localhost:3000)

**출력 예시:**
```
================================================================================
✨ 전체 시스템 시작 완료!
================================================================================

🔌 DB 터널:
   - PID: 12344
   - Local Port: 5456 (extension), 5457 (postgres)
   - Remote: pacs-extension.ciyua2gsk8ke.ap-northeast-2.rds.amazonaws.com
   - 로그: tail -f db-tunnel.log

📦 백엔드 서버:
   - PID: 12345
   - URL: http://localhost:8080
   - Swagger UI: http://localhost:8080/swagger-ui/
   - Health Check: http://localhost:8080/health
   - 로그: tail -f backend.log

🎨 프론트엔드 서버:
   - PID: 12346
   - URL: http://localhost:3000
   - 로그: tail -f frontend.log
```

### 2. 전체 시스템 종료

```bash
./stop-all.sh
```

**동작:**
- DB 터널 종료 (graceful shutdown)
- 백엔드 프로세스 종료 (graceful shutdown)
- 프론트엔드 프로세스 종료
- 필요시 강제 종료 (10초 대기 후)
- 포트 5456, 5457, 8080, 3000 정리
- PID 파일 삭제

### 3. 전체 시스템 재시작

```bash
./restart-all.sh
```

**동작:**
- `stop-all.sh` 실행
- 3초 대기
- `start-all.sh` 실행

**사용 시나리오:**
- 코드 변경 후 재시작
- 설정 변경 후 재시작
- 메모리 누수 의심 시

### 4. 시스템 상태 확인

```bash
./status-all.sh
```

**출력 정보:**
- 프로세스 실행 여부 (PID)
- 메모리 사용량
- CPU 사용률
- 실행 시간
- 포트 사용 상태
- Health Check 결과
- 로그 파일 크기

**출력 예시:**
```
================================================================================
📊 PACS Extension Server - 시스템 상태
================================================================================

🔧 백엔드 서버 (Rust - Actix-web)
--------------------------------------------------------------------------------
✅ 실행 중 (PID: 12345)
   메모리: 45.2 MB
   CPU: 0.5%
   실행 시간: 01:23:45
   포트 8080: 사용 중 (PID: 12345)
   Health Check: OK
   URL: http://localhost:8080
   Swagger UI: http://localhost:8080/swagger-ui/

🎨 프론트엔드 서버 (React)
--------------------------------------------------------------------------------
✅ 실행 중 (PID: 12346)
   메모리: 120.5 MB
   CPU: 1.2%
   실행 시간: 01:23:40
   포트 3000: 사용 중 (PID: 12346)
   HTTP Check: OK
   URL: http://localhost:3000

================================================================================
✅ 전체 시스템 정상 작동 중
================================================================================
```

## 🔧 상세 설명

### DB 터널 (scripts/db-tunnel.sh)

#### 개요
AWS RDS 데이터베이스에 SSH 터널을 통해 안전하게 연결합니다.

#### 연결 정보
- **Bastion Host**: 13.125.228.206
- **SSH 키**: `ssh/bastion-keypair.pem` (권한: 600)
- **원격 DB (Extension)**: pacs-extension.ciyua2gsk8ke.ap-northeast-2.rds.amazonaws.com:5432
- **원격 DB (Postgres)**: pacs-postgres.ciyua2gsk8ke.ap-northeast-2.rds.amazonaws.com:5432
- **로컬 포트**: 5456 (extension), 5457 (postgres)

#### 사용법

```bash
# 터널 시작
./scripts/db-tunnel.sh

# 터널 종료
./scripts/db-tunnel.sh -s

# 특정 DB만 연결
./scripts/db-tunnel.sh -t extension  # extension DB만
./scripts/db-tunnel.sh -t postgres   # postgres DB만
./scripts/db-tunnel.sh -t both       # 둘 다 (기본값)
```

#### DB 직접 접근

```bash
# Extension DB 접속
psql "postgres://pacs_extension_admin:PacsExtension2024@localhost:5456/pacs_extension"

# Postgres DB 접속
psql "postgres://postgres:your-password@localhost:5457/postgres"
```

#### 문제 해결

```bash
# SSH 키 권한 오류
chmod 600 ssh/bastion-keypair.pem

# 포트 충돌
lsof -ti:5456 | xargs kill -9
lsof -ti:5457 | xargs kill -9

# 터널 프로세스 확인
ps aux | grep ssh | grep 5456
```

---

### start-all.sh

#### 실행 순서
1. **기존 프로세스 확인**
   - `.db-tunnel.pid`, `.backend.pid`, `.frontend.pid` 파일 확인
   - 실행 중인 프로세스 종료

2. **포트 정리**
   - `lsof -ti:5456` - DB 터널 (extension)
   - `lsof -ti:5457` - DB 터널 (postgres)
   - `lsof -ti:8080` - 백엔드 포트
   - `lsof -ti:3000` - 프론트엔드 포트

3. **DB 터널 시작**
   - `scripts/db-tunnel.sh` 실행
   - 포트 5456 연결 대기 (최대 10초)

4. **백엔드 시작**
   - `.env` 파일 확인
   - `cargo build --bin pacs_server`
   - `cargo run --bin pacs_server` (백그라운드)
   - Health Check 대기 (최대 30초)

5. **프론트엔드 시작**
   - `node_modules` 확인 및 설치
   - `npm start` (백그라운드)
   - HTTP 응답 대기 (최대 60초)

6. **브라우저 열기**
   - macOS: `open http://localhost:3000`

#### 로그 파일
- `db-tunnel.log` - DB 터널 출력
- `backend.log` - 백엔드 출력
- `frontend.log` - 프론트엔드 출력

#### PID 파일
- `.db-tunnel.pid` - DB 터널 프로세스 ID
- `.backend.pid` - 백엔드 프로세스 ID
- `.frontend.pid` - 프론트엔드 프로세스 ID

### stop-all.sh

#### 종료 프로세스
1. **DB 터널 종료**
   - `kill <DB_TUNNEL_PID>` (SIGTERM)
   - 5초 대기
   - 필요시 강제 종료 (SIGKILL)

2. **백엔드 종료**
   - `kill <BACKEND_PID>` (SIGTERM)
   - 10초 대기
   - 필요시 강제 종료 (SIGKILL)

3. **프론트엔드 종료**
   - `kill <FRONTEND_PID>` (SIGTERM)
   - 10초 대기
   - 필요시 강제 종료 (SIGKILL)

4. **포트 정리**
   - `lsof -ti:5456 | xargs kill -9`
   - `lsof -ti:5457 | xargs kill -9`
   - `lsof -ti:8080 | xargs kill -9`
   - `lsof -ti:3000 | xargs kill -9`

5. **정리**
   - PID 파일 삭제

### status-all.sh

#### 확인 항목

**DB 터널:**
- PID 파일 존재 여부
- 프로세스 실행 여부
- 메모리 사용량 (`ps -o rss=`)
- CPU 사용률 (`ps -o %cpu=`)
- 실행 시간 (`ps -o etime=`)
- 포트 5456, 5457 사용 여부

**백엔드:**
- PID 파일 존재 여부
- 프로세스 실행 여부
- 메모리 사용량 (`ps -o rss=`)
- CPU 사용률 (`ps -o %cpu=`)
- 실행 시간 (`ps -o etime=`)
- 포트 8080 사용 여부
- Health Check (`curl http://localhost:8080/health`)

**프론트엔드:**
- PID 파일 존재 여부
- 프로세스 실행 여부
- 메모리 사용량
- CPU 사용률
- 실행 시간
- 포트 3000 사용 여부
- HTTP 응답 (`curl http://localhost:3000`)

## 🐛 문제 해결

### 1. 스크립트 실행 권한 오류

```bash
# 오류: Permission denied
chmod +x start-all.sh stop-all.sh restart-all.sh status-all.sh
chmod +x scripts/db-tunnel.sh
```

### 2. SSH 키 권한 오류

```bash
# 오류: Permissions 0755 for 'bastion-keypair.pem' are too open
chmod 600 ssh/bastion-keypair.pem
```

### 3. DB 터널 연결 실패

```bash
# 로그 확인
tail -f db-tunnel.log

# 터널 프로세스 확인
ps aux | grep ssh | grep 5456

# 포트 확인
lsof -ti:5456
lsof -ti:5457

# 터널 재시작
./scripts/db-tunnel.sh -s  # 종료
./scripts/db-tunnel.sh     # 시작
```

### 4. 포트가 이미 사용 중

```bash
# 포트 확인
lsof -ti:5456  # DB 터널 (extension)
lsof -ti:5457  # DB 터널 (postgres)
lsof -ti:8080  # 백엔드
lsof -ti:3000  # 프론트엔드

# 강제 종료
lsof -ti:5456 | xargs kill -9
lsof -ti:5457 | xargs kill -9
lsof -ti:8080 | xargs kill -9
lsof -ti:3000 | xargs kill -9
```

### 5. 백엔드 빌드 실패

```bash
# 로그 확인
tail -f backend.log

# 수동 빌드
cd pacs-server
cargo build --bin pacs_server
```

### 6. 프론트엔드 시작 실패

```bash
# 로그 확인
tail -f frontend.log

# node_modules 재설치
cd auth-dashboard
rm -rf node_modules package-lock.json
npm install
```

### 7. Health Check 실패

```bash
# 백엔드 로그 확인
tail -f backend.log

# DB 터널 확인
lsof -ti:5456

# 데이터베이스 연결 확인
psql "postgres://pacs_extension_admin:PacsExtension2024@localhost:5456/pacs_extension"

# .env 파일 확인
cat pacs-server/.env
```

### 8. PID 파일 불일치

```bash
# PID 파일 삭제
rm -f .db-tunnel.pid .backend.pid .frontend.pid

# 모든 프로세스 강제 종료
pkill -f "ssh.*5456"
pkill -f pacs_server
pkill -f "react-scripts start"

# 재시작
./start-all.sh
```

## 📝 로그 확인

### 실시간 로그 보기

```bash
# DB 터널 로그
tail -f db-tunnel.log

# 백엔드 로그
tail -f backend.log

# 프론트엔드 로그
tail -f frontend.log

# 모든 로그 동시에 보기
tail -f db-tunnel.log backend.log frontend.log
```

### 로그 검색

```bash
# 에러 검색
grep -i error db-tunnel.log
grep -i error backend.log
grep -i error frontend.log

# 특정 시간대 로그
grep "2024-01-15 10:" backend.log
```

## 🔄 일반적인 워크플로우

### 개발 시작

```bash
./start-all.sh
# 브라우저가 자동으로 열림
# http://localhost:3000
```

### 코드 변경 후

```bash
# 백엔드 코드 변경 시
./restart-all.sh

# 프론트엔드 코드 변경 시
# React는 자동 리로드되므로 재시작 불필요
```

### 상태 확인

```bash
./status-all.sh
```

### 작업 종료

```bash
./stop-all.sh
```

## 🎯 고급 사용법

### 백엔드만 재시작

```bash
# 백엔드 종료
if [ -f .backend.pid ]; then
    kill $(cat .backend.pid)
    rm .backend.pid
fi

# 백엔드 시작
cd pacs-server
nohup cargo run --bin pacs_server > ../backend.log 2>&1 &
echo $! > ../.backend.pid
```

### 프론트엔드만 재시작

```bash
# 프론트엔드 종료
if [ -f .frontend.pid ]; then
    kill $(cat .frontend.pid)
    rm .frontend.pid
fi

# 프론트엔드 시작
cd auth-dashboard
nohup npm start > ../frontend.log 2>&1 &
echo $! > ../.frontend.pid
```

### 로그 파일 정리

```bash
# 로그 백업
mv backend.log backend.log.$(date +%Y%m%d_%H%M%S)
mv frontend.log frontend.log.$(date +%Y%m%d_%H%M%S)

# 또는 삭제
rm -f backend.log frontend.log
```

## 📚 참고

- 백엔드 API 문서: http://localhost:8080/swagger-ui/
- 프론트엔드 대시보드: http://localhost:3000
- Health Check: http://localhost:8080/health

