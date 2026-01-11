# 🔔 PACS Extension Server - 중요 사항 리마인더

## 📌 DB 터널 실행 위치

### ⚠️ 중요: 실행 경로 주의!

**문제:**
- `pacs-server/db-tunnel.sh`는 `pacs-server` 디렉토리 내에서 실행해야 함
- SSH 키 경로가 `./ssh/bastion-keypair.pem`로 상대 경로로 설정되어 있음
- 루트에서 실행하면 키를 찾지 못해 실패함

**해결:**
- 루트에서 실행 가능한 통합 스크립트 생성: `scripts/start-db-tunnels.sh`

---

## 🚀 DB 터널 시작 방법

### 1. **통합 스크립트 사용 (권장)** ✅

루트 디렉토리(`pacs-ext-server`)에서 실행:

```bash
./scripts/start-db-tunnels.sh
```

**실행 내용:**
- Extension DB 터널 시작 (포트 5456)
- Dcm4chee DB 터널 시작 (포트 5457)
- 두 터널 모두 자동으로 시작 및 상태 확인

### 2. **개별 터널 시작**

루트 디렉토리에서:

```bash
# Extension DB만 (포트 5456)
./scripts/db-tunnel.sh -t extension

# Dcm4chee DB만 (포트 5457)
./scripts/db-tunnel.sh -t postgres

# 둘 다
./scripts/db-tunnel.sh -t both
```

### 3. **터널 중지**

```bash
# 모든 터널 중지
./scripts/db-tunnel.sh -k -t both

# Extension DB만 중지
./scripts/db-tunnel.sh -k -t extension

# Dcm4chee DB만 중지
./scripts/db-tunnel.sh -k -t postgres
```

---

## 🔧 DB 터널 설정

### Extension DB (RBAC용)
- **포트:** 5456
- **RDS:** pacs-extension.ciyua2gsk8ke.ap-northeast-2.rds.amazonaws.com
- **Database:** pacs_db
- **용도:** RBAC, 사용자 관리, 프로젝트 관리

### Dcm4chee DB (Sync용)
- **포트:** 5457
- **RDS:** pacs-postgres.ciyua2gsk8ke.ap-northeast-2.rds.amazonaws.com
- **Database:** postgres
- **용도:** DICOM 메타데이터 동기화 (Sync Engine)

---

## 🔄 Sync 기능 사용 시 주의사항

### ⚠️ Sync 기능을 사용하려면:

1. **Dcm4chee DB 터널 필수** (포트 5457)
   ```bash
   ./scripts/start-db-tunnels.sh
   ```

2. **서버 재시작**
   - 터널 시작 후 서버를 재시작해야 Sync 기능 활성화
   ```bash
   ./scripts/start-all.sh
   ```

3. **Sync 상태 확인**
   ```bash
   curl http://localhost:8080/api/sync/status
   ```

4. **수동 동기화 실행**
   ```bash
   curl -X POST http://localhost:8080/api/sync/run
   ```

### 에러 메시지:
```
⚠️  Warning: Failed to initialize sync service: Failed to connect to DCM4CHEE DB: pool timed out
⚠️  Sync features will be disabled
```

**원인:** Dcm4chee DB 터널 (포트 5457)이 실행되지 않음  
**해결:** `./scripts/start-db-tunnels.sh` 실행 후 서버 재시작

---

## 📝 파일 구조

```
pacs-ext-server/
├── scripts/
│   ├── db-tunnel.sh              # 원본 DB 터널 스크립트 (루트에서 실행)
│   └── start-db-tunnels.sh       # 통합 DB 터널 스크립트 (NEW!)
├── pacs-server/
│   ├── db-tunnel.sh              # 레거시 (pacs-server 내에서만 실행 가능)
│   └── ssh/
│       └── bastion-keypair.pem   # SSH 키 (상대 경로 참조)
└── REMINDER.md                   # 이 파일
```

---

## 🎯 빠른 시작 체크리스트

서버를 처음 시작할 때:

- [ ] 1. DB 터널 시작: `./scripts/start-db-tunnels.sh`
- [ ] 2. 터널 확인: `lsof -i :5456` 및 `lsof -i :5457`
- [ ] 3. 서버 시작: `./scripts/start-all.sh`
- [ ] 4. Sync 상태 확인: `curl http://localhost:8080/api/sync/status`

---

## 📅 작성일: 2025-12-18

**변경 사항:**
- `scripts/start-db-tunnels.sh` 생성 (루트에서 실행 가능한 통합 스크립트)
- Extension DB (5456) + Dcm4chee DB (5457) 동시 시작
- Sync 기능 사용 시 필수 요구사항 문서화

