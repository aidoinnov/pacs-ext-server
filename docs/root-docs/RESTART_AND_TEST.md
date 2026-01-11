# 서버 재시작 및 E2E 테스트 실행

## 문제 해결 완료

Viewer API의 의존성 주입 문제를 수정했습니다:
- `user_repo`와 `project_data_repo`의 이중 Arc 문제 해결
- `pacs-server/src/main.rs` 835, 837번 라인 수정

## 서버 재시작 방법

### 방법 1: 전체 재시작 (권장)
```bash
./restart-all.sh
```

### 방법 2: 백엔드만 재시작
```bash
# 1. 기존 서버 종료
pkill -f pacs_server

# 2. 빌드 및 실행
cd pacs-server
cargo build
cargo run --bin pacs_server &
cd ..
```

## E2E 테스트 실행

서버가 시작되면 (약 10초 대기):

```bash
python3 test_viewer_apis_e2e.py
```

## 예상 결과

성공 시:
```
✅ Login successful! Token length: 1385

================================================================================
Test 1: POST /api/v1/viewer/studies/meta (Study Meta Batch API)
================================================================================

ℹ️  Test 1.1: Default pagination (no page params)
✅ Test 1.1: Pagination structure valid
✅ Test 1.1: Found 2 studies
✅ Test 1.2: Pagination working correctly (returned 1 study)
✅ ✨ Study Meta API: ALL TESTS PASSED
```

## 문제 해결

### 여전히 500 에러가 발생하는 경우

1. **서버가 제대로 재시작되었는지 확인**:
```bash
ps aux | grep pacs_server
```

2. **빌드 에러 확인**:
```bash
cd pacs-server
cargo build
```

3. **백엔드 로그 확인**:
```bash
tail -f /Users/aido/Code/pacs-ext-server/backend.log
```

4. **의존성 주입 확인**:
```bash
curl -s http://localhost:8080/api/dicom/debug-deps | jq .
```

### "Requested application data is not configured correctly" 에러

이 에러는 actix-web의 `web::Data`로 주입된 의존성이 없을 때 발생합니다.

**원인**: 서버가 재시작되지 않아 코드 변경사항이 반영되지 않음

**해결**: 서버를 완전히 종료하고 다시 빌드/실행

```bash
# 모든 pacs_server 프로세스 종료
pkill -9 -f pacs_server

# 빌드 및 실행
cd pacs-server
cargo clean  # 필요시
cargo build
cargo run --bin pacs_server
```

## 수정된 코드 확인

`pacs-server/src/main.rs` 832-837번 라인:

```rust
// 의존성 주입
cfg.app_data(web::Data::new(qido_client.clone()));
cfg.app_data(web::Data::new(jwt_service.clone()));
cfg.app_data(web::Data::new(user_repo.clone()));          // ✅ 수정됨 (Arc::new 제거)
cfg.app_data(web::Data::new(dicom_evaluator.clone()));
cfg.app_data(web::Data::new(project_data_repo.clone()));  // ✅ 수정됨 (Arc::new 제거)
```

**변경 전** (이중 Arc):
```rust
cfg.app_data(web::Data::new(Arc::new(user_repo.clone())));
cfg.app_data(web::Data::new(Arc::new(project_data_repo.clone())));
```

**변경 후** (단일 Arc):
```rust
cfg.app_data(web::Data::new(user_repo.clone()));
cfg.app_data(web::Data::new(project_data_repo.clone()));
```

## 테스트 데이터 설정

E2E 테스트를 실행하기 전에 `test_viewer_apis_e2e.py`에서 실제 Study UID로 수정하세요:

```python
# Test data - Replace with actual UIDs from your PACS
TEST_STUDY_UID_1 = "1.2.840.113619.2.55.3.604688433.1234"  # 실제 UID로 변경
TEST_STUDY_UID_2 = "1.2.840.113619.2.55.3.604688433.5678"  # 실제 UID로 변경
TEST_SERIES_UID_1 = "1.2.840.113619.2.55.3.604688433.1234.1"  # 실제 UID로 변경
TEST_SERIES_UID_2 = "1.2.840.113619.2.55.3.604688433.1234.2"  # 실제 UID로 변경
```

실제 UID를 찾으려면:
```bash
# 로그인
TOKEN=$(curl -s -X POST "http://localhost:8080/api/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"username":"iaid-pacs-admin","password":"Qlalfqjsgh1!"}' | jq -r '.keycloak_access_token')

# Study 목록 조회
curl -s -X GET "http://localhost:8080/api/dicom/studies?limit=10&project_id=1" \
  -H "Authorization: Bearer $TOKEN" | jq '.[].["0020000D"].Value[0]'
```

