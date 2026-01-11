# Build and E2E Test Instructions

## 1. 빌드 확인

터미널에서 다음 명령어를 실행하세요:

```bash
cd pacs-server
cargo build
```

빌드가 성공하면 다음과 같은 메시지가 표시됩니다:
```
   Compiling pacs_server v0.1.0 (/Users/aido/Code/pacs-ext-server/pacs-server)
    Finished dev [unoptimized + debuginfo] target(s) in X.XXs
```

## 2. 서버 실행

빌드가 성공하면 서버를 실행하세요:

```bash
cargo run --bin pacs_server
```

서버가 정상적으로 시작되면 다음과 같은 로그가 표시됩니다:
```
INFO  pacs_server > Server starting on 0.0.0.0:8080
```

## 3. E2E 테스트 준비

### 3.1 테스트 데이터 설정

`test_viewer_apis_e2e.py` 파일을 열고 실제 PACS 데이터로 수정하세요:

```python
# Test data - Replace with actual UIDs from your PACS
TEST_STUDY_UID_1 = "1.2.840.113619.2.55.3.604688433.1234"  # 실제 Study UID
TEST_STUDY_UID_2 = "1.2.840.113619.2.55.3.604688433.5678"  # 실제 Study UID
TEST_SERIES_UID_1 = "1.2.840.113619.2.55.3.604688433.1234.1"  # 실제 Series UID
TEST_SERIES_UID_2 = "1.2.840.113619.2.55.3.604688433.1234.2"  # 실제 Series UID
```

### 3.2 실제 Study UID 찾기

PACS에서 실제 Study UID를 찾으려면:

```bash
# 로그인
curl -X POST "http://localhost:8080/api/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"username": "admin", "password": "admin123"}' | jq -r '.access_token'

# 토큰을 변수에 저장
TOKEN="<위에서 받은 토큰>"

# QIDO로 Study 목록 조회
curl -X GET "http://localhost:8080/api/dicom/qido/studies" \
  -H "Authorization: Bearer $TOKEN" | jq '.[].["0020000D"].Value[0]'
```

출력 예시:
```
"1.2.840.113619.2.55.3.604688433.1234"
"1.2.840.113619.2.55.3.604688433.5678"
```

### 3.3 실제 Series UID 찾기

특정 Study의 Series UID를 찾으려면:

```bash
STUDY_UID="1.2.840.113619.2.55.3.604688433.1234"

curl -X GET "http://localhost:8080/api/dicom/qido/studies/${STUDY_UID}/series" \
  -H "Authorization: Bearer $TOKEN" | jq '.[].["0020000E"].Value[0]'
```

출력 예시:
```
"1.2.840.113619.2.55.3.604688433.1234.1"
"1.2.840.113619.2.55.3.604688433.1234.2"
```

## 4. E2E 테스트 실행

### 4.1 Python 환경 확인

```bash
python3 --version  # Python 3.7 이상 필요
pip3 install requests  # requests 라이브러리 설치
```

### 4.2 테스트 실행

새 터미널을 열고 (서버는 계속 실행 중):

```bash
cd /Users/aido/Code/pacs-ext-server
./test_viewer_apis_e2e.py
```

또는

```bash
python3 test_viewer_apis_e2e.py
```

## 5. 예상 출력

테스트가 성공하면 다음과 같은 출력이 표시됩니다:

```
================================================================================
🚀 E2E Test: Viewer APIs with Pagination
================================================================================

ℹ️  Base URL: http://localhost:8080
ℹ️  Test Study UID: 1.2.840.113619.2.55.3.604688433.1234

================================================================================
Step 1: Authentication
================================================================================

ℹ️  Logging in as admin...
✅ Login successful! Token length: 245

================================================================================
Test 1: POST /api/v1/viewer/studies/meta (Study Meta Batch API)
================================================================================

ℹ️  Test 1.1: Default pagination (no page params)
✅ Test 1.1: Pagination structure valid
ℹ️    Page: 1/1
ℹ️    Items: 50 (Total: 2)
ℹ️    Has next: false, Has previous: false
✅ Test 1.1: Found 2 studies
✅ Test 1.2: Pagination working correctly (returned 1 study)
✅ ✨ Study Meta API: ALL TESTS PASSED

================================================================================
Test 2: POST /api/v1/viewer/series/meta (Series Meta Batch API)
================================================================================

ℹ️  Test 2.1: Default pagination (no page params)
✅ Test 2.1: Pagination structure valid
✅ Test 2.1: Found 2 series
✅ ✨ Series Meta API: ALL TESTS PASSED

================================================================================
Test 3: GET /api/v1/viewer/studies/{study_uid}/series/meta (Study Series Meta API)
================================================================================

ℹ️  Test 3.1: Default pagination (no query params)
✅ Test 3.1: Pagination structure valid
✅ Test 3.1: Found 5 series for study 1.2.840.113619.2.55.3.604688433.1234
✅ Test 3.3: Page size correctly clamped to 200
✅ ✨ Study Series Meta API: ALL TESTS PASSED

================================================================================
🎉 ALL TESTS PASSED!
================================================================================

✅ POST /api/v1/viewer/studies/meta - Pagination working
✅ POST /api/v1/viewer/series/meta - Pagination working
✅ GET /api/v1/viewer/studies/{study_uid}/series/meta - Pagination working

ℹ️  All three Viewer APIs have been successfully tested with pagination!
```

## 6. 문제 해결

### 빌드 에러가 발생하는 경우

```bash
cd pacs-server
cargo clean
cargo build
```

### 서버 연결 실패

```bash
# 서버가 실행 중인지 확인
curl http://localhost:8080/health

# 포트가 사용 중인지 확인
lsof -i :8080
```

### 인증 실패

- `test_viewer_apis_e2e.py`에서 USERNAME과 PASSWORD가 올바른지 확인
- 데이터베이스에 사용자가 존재하는지 확인

### 404 Not Found

- 테스트 데이터 (Study UID, Series UID)가 실제 PACS에 존재하는지 확인
- 사용자가 해당 데이터에 접근 권한이 있는지 확인

## 7. 빠른 시작 (요약)

```bash
# 터미널 1: 서버 실행
cd pacs-server
cargo build
cargo run --bin pacs_server

# 터미널 2: 테스트 데이터 확인 및 수정
# test_viewer_apis_e2e.py 파일에서 실제 UID로 수정

# 터미널 2: E2E 테스트 실행
cd /Users/aido/Code/pacs-ext-server
python3 test_viewer_apis_e2e.py
```

## 8. 성공 기준

✅ 빌드 성공  
✅ 서버 정상 실행  
✅ 모든 E2E 테스트 통과 (종료 코드 0)  
✅ 세 가지 API 모두 페이지네이션 기능 검증 완료

