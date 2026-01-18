# PACS Extension Server

PACS (Picture Archiving and Communication System) Extension Server는 의료 영상 관리 시스템을 위한 확장 서버입니다.

## 🚀 빠른 시작

### 1️⃣ 전체 시스템 한 번에 실행

```bash
./start-all.sh
```

이 명령어 하나로 다음이 모두 실행됩니다:
- 🔌 **DB 터널** (AWS RDS 연결)
- 📦 **백엔드 서버** (Rust/Actix-web)
- 🎨 **프론트엔드 대시보드** (React)

실행 후 자동으로 브라우저가 열립니다: http://localhost:3000

### 2️⃣ 시스템 종료

```bash
./stop-all.sh
```

### 3️⃣ 시스템 재시작

```bash
./restart-all.sh
```

### 4️⃣ 시스템 상태 확인

```bash
./status-all.sh
```

---

## 📋 시스템 구성

### 🔌 DB 터널
- **로컬 포트**: 5456 (extension), 5457 (postgres)
- **원격 DB**: AWS RDS (pacs-extension.ciyua2gsk8ke.ap-northeast-2.rds.amazonaws.com)
- **Bastion Host**: 13.125.228.206
- **SSH 키**: `ssh/bastion-keypair.pem`

### 📦 백엔드 서버
- **언어/프레임워크**: Rust + Actix-web
- **포트**: 8080
- **API 문서**: http://localhost:8080/swagger-ui/
- **Health Check**: http://localhost:8080/health
- **로그**: `backend.log`

### 🎨 프론트엔드 대시보드
- **프레임워크**: React + TypeScript
- **포트**: 3000
- **URL**: http://localhost:3000
- **로그**: `frontend.log`

---

## 🛠️ 개발 환경 설정

### 사전 요구사항

1. **Rust** (1.70+)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Node.js** (18+)
   ```bash
   # macOS
   brew install node
   
   # 또는 nvm 사용
   nvm install 18
   ```

3. **PostgreSQL Client** (선택사항, DB 직접 접근 시)
   ```bash
   brew install postgresql
   ```

### 환경 변수 설정

백엔드 `.env` 파일 (`pacs-server/.env`):
```bash
# 개발 모드
APP_ENV=development

# 데이터베이스
DATABASE_URL=postgres://pacs_extension_admin:PacsExtension2024@localhost:5456/pacs_extension

# 서버 설정
SERVER_HOST=0.0.0.0
SERVER_PORT=8080

# JWT 설정
JWT_SECRET=your-secret-key-here
JWT_EXPIRATION=86400

# Keycloak (선택사항)
KEYCLOAK_URL=https://your-keycloak-url
KEYCLOAK_REALM=your-realm
```

---

## 📚 상세 문서

### 스크립트 사용법
자세한 스크립트 사용법은 [SCRIPTS_README.md](./SCRIPTS_README.md)를 참고하세요.

### API 문서
- Swagger UI: http://localhost:8080/swagger-ui/
- API 문서: [docs/api-documentation.md](./docs/api-documentation.md)

### 아키텍처
- [아키텍처 개요](./docs/architecture-overview.md)
- [개발 가이드](./docs/DEVELOPMENT.md)

---

## 🔧 수동 실행 (개별 서비스)

### DB 터널만 실행

```bash
./scripts/db-tunnel.sh
```

### 백엔드만 실행

```bash
cd pacs-server
cargo run --bin pacs_server
```

### 프론트엔드만 실행

```bash
cd auth-dashboard
npm start
```

---

## 🧪 테스트

### 백엔드 테스트

```bash
cd pacs-server
cargo test
```

### 프론트엔드 테스트

```bash
cd auth-dashboard
npm test
```

---

## 🐛 문제 해결

### 포트가 이미 사용 중

```bash
# 포트 확인
lsof -ti:8080  # 백엔드
lsof -ti:3000  # 프론트엔드
lsof -ti:5456  # DB 터널

# 강제 종료
lsof -ti:8080 | xargs kill -9
```

### DB 연결 실패

```bash
# DB 터널 로그 확인
tail -f db-tunnel.log

# DB 터널 재시작
./scripts/db-tunnel.sh -s  # 종료
./scripts/db-tunnel.sh     # 시작
```

### 백엔드 빌드 실패

```bash
# 로그 확인
tail -f backend.log

# 캐시 정리 후 재빌드
cd pacs-server
cargo clean
cargo build
```

---

## 📊 주요 기능

- ✅ **RBAC (Role-Based Access Control)**: 역할 기반 권한 관리
- ✅ **Annotation 관리**: 의료 영상 주석 생성/수정/삭제
- ✅ **프로젝트 관리**: 프로젝트별 데이터 접근 제어
- ✅ **JWT 인증**: 개발 모드 + 프로덕션 JWT 인증
- ✅ **Keycloak 통합**: 외부 인증 시스템 연동
- ✅ **DICOM 지원**: 의료 영상 표준 프로토콜
- ✅ **RECIST 1.1 Lesion 관리**: 종양 평가 기준 준수 병변 관리 시스템

### 🎯 RECIST Lesion 관리 (NEW!)

**RECIST 1.1 기준 병변 관리 시스템**이 완전히 구현되었습니다!

**주요 기능:**
- Target Lesion: 최대 5개 제한
- Non-Target Lesion: 무제한
- NEW Lesion: Follow-up에서만 생성
- TimePoint별 Annotation 추적
- 자동 Lesion Number 생성

**API 엔드포인트:**
- `POST /api/subjects/{subject_id}/recist-lesions` - Lesion 생성
- `GET /api/subjects/{subject_id}/recist-lesions` - Lesion 목록 조회
- `GET /api/recist-lesions/{id}` - Lesion 상세 조회
- `PUT /api/recist-lesions/{id}` - Lesion 수정
- `DELETE /api/recist-lesions/{id}` - Lesion 삭제
- `POST /api/recist-lesions/{id}/annotations` - Annotation 연결

**문서:**
- 📘 [구현 계획](./docs/target-lesion/plan.md)
- 📊 [구현 요약](./docs/target-lesion/IMPLEMENTATION_SUMMARY.md)
- 🧪 [E2E 테스트](./tests/e2e/RECIST_LESION_TEST.md)

**테스트 실행:**
```bash
cd tests/e2e
python run_recist_lesion.py
```

---

## 📝 라이선스

MIT License

---

## 👥 기여

이슈 및 PR은 언제나 환영합니다!

