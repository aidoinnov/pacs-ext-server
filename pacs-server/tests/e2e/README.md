# GC Runner E2E Tests

GC Runner의 전체 워크플로우를 테스트하는 End-to-End 테스트입니다.

## 📋 테스트 시나리오

### Scenario 1: PENDING Timeout (Dry-run)
- **목적**: Dry-run 모드에서 PENDING 타임아웃 대상을 찾지만 실제 변경은 하지 않음
- **검증**: 어노테이션 상태가 변경되지 않음

### Scenario 2: PENDING Timeout (Actual)
- **목적**: Grace period를 초과한 PENDING 스냅샷을 FAILED로 변경
- **검증**: 
  - ID 90001 (4일) → FAILED로 변경됨
  - ID 90002 (2일) → PENDING 유지 (grace period 미만)

### Scenario 3: FAILED Cleanup (Dry-run)
- **목적**: Dry-run 모드에서 FAILED 스냅샷 정리 대상을 찾지만 실제 삭제는 하지 않음
- **검증**: snapshot_image_key가 변경되지 않음

### Scenario 4: FAILED Cleanup (Actual)
- **목적**: Grace period를 초과한 FAILED 스냅샷의 S3 파일 삭제 및 DB 업데이트
- **검증**: GC 로그가 기록됨 (S3 에러는 테스트 환경에서 예상됨)

### Scenario 5: Grace Period Validation
- **목적**: Grace period 미만의 데이터는 처리되지 않음을 검증
- **검증**:
  - ID 90002 (2일) → PENDING 유지
  - ID 90004 (5일) → snapshot_image_key 유지

### Scenario 6: Full Workflow
- **목적**: 전체 워크플로우 검증 및 최종 상태 확인
- **검증**: 모든 어노테이션의 최종 상태 출력

## 🚀 실행 방법

### 1. 의존성 설치

```bash
pip3 install -r tests/e2e/requirements.txt
```

### 2. 환경 변수 설정 (선택사항)

```bash
export DATABASE_URL="postgresql://user@localhost:5432/pacs_db"
export GC_RUNNER_PATH="./target/debug/gc_runner"
```

### 3. 테스트 실행

**방법 1: 스크립트 사용 (권장)**
```bash
./tests/e2e/run_e2e_tests.sh
```

**방법 2: Python 직접 실행**
```bash
# 먼저 빌드
cargo build --bin gc_runner

# 테스트 실행
python3 tests/e2e/test_gc_e2e.py
```

## 📊 테스트 데이터

테스트는 다음 데이터를 생성합니다:

| ID    | Status    | Days Old | Snapshot Key                              | 예상 동작                    |
|-------|-----------|----------|-------------------------------------------|------------------------------|
| 90001 | pending   | 4        | snapshots/99999/90001/test-90001.png      | FAILED로 변경 (grace > 3일)  |
| 90002 | pending   | 2        | snapshots/99999/90002/test-90002.png      | 변경 없음 (grace < 3일)      |
| 90003 | failed    | 8        | snapshots/99999/90003/test-90003.png      | S3 삭제 시도 (grace > 7일)   |
| 90004 | failed    | 5        | snapshots/99999/90004/test-90004.png      | 변경 없음 (grace < 7일)      |
| 90005 | completed | 10       | snapshots/99999/90005/test-90005.png      | 변경 없음 (처리 대상 아님)   |

## 🧹 정리

테스트는 자동으로 다음을 정리합니다:
- 테스트 어노테이션 (ID >= 90000)
- GC 로그 (annotation_id >= 90000)
- 테스트 프로젝트 (ID = 99999)
- 테스트 사용자 (ID = 99999)

## ⚠️ 주의사항

1. **S3 에러**: 테스트 환경에서는 실제 S3가 없으므로 S3 삭제 실패는 예상된 동작입니다.
2. **데이터베이스**: 테스트는 실제 데이터베이스를 사용하므로 프로덕션 환경에서 실행하지 마세요.
3. **ID 범위**: 테스트는 ID >= 90000 범위의 데이터만 사용하므로 기존 데이터와 충돌하지 않습니다.

## 📝 출력 예시

```
🧪 GC Runner E2E Test
============================================================

✅ Connected to database

============================================================
📋 Test Setup
============================================================

ℹ️  Cleaning up test data...
✅ Cleanup completed
ℹ️  Setting up test fixtures...
✅ Test fixtures created
ℹ️  Creating test data...
  - Created PENDING annotation (4 days old) - ID: 90001
  - Created PENDING annotation (2 days old) - ID: 90002
  - Created FAILED annotation (8 days old) - ID: 90003
  - Created FAILED annotation (5 days old) - ID: 90004
  - Created COMPLETED annotation (10 days old) - ID: 90005
✅ Test data created

📊 Current state before GC:
  ID 90001: pending (4 days old)
  ID 90002: pending (2 days old)
  ID 90003: failed (8 days old)
  ID 90004: failed (5 days old)
  ID 90005: completed (10 days old)

============================================================
📋 Test 1: Job A - PENDING Timeout (Dry-run)
============================================================

ℹ️  Running Job A: Timeout Pending Snapshots
   Grace Days: 3
   Batch Size: 100
   Dry-run: true
✅ Job A completed (dry-run)
✅ Test 1 PASSED: ID 90001 still PENDING (dry-run)

...

============================================================
📋 Test Results Summary
============================================================

✅ Scenario 1: PENDING Timeout (Dry-run)
✅ Scenario 2: PENDING Timeout (Actual)
✅ Scenario 3: FAILED Cleanup (Dry-run)
✅ Scenario 4: FAILED Cleanup (Actual)
✅ Scenario 5: Grace Period Validation
✅ Scenario 6: Full Workflow

Total: 6/6 tests passed
✅ All tests passed! 🎉
```

## 🔧 트러블슈팅

### psycopg2 설치 실패
```bash
# macOS
brew install postgresql
pip3 install psycopg2-binary

# Ubuntu/Debian
sudo apt-get install libpq-dev
pip3 install psycopg2-binary
```

### 데이터베이스 연결 실패
```bash
# DATABASE_URL 확인
echo $DATABASE_URL

# PostgreSQL 실행 확인
psql -d pacs_db -c "SELECT 1"
```

### GC Runner 바이너리 없음
```bash
# 빌드
cargo build --bin gc_runner

# 경로 확인
ls -la target/debug/gc_runner
```

