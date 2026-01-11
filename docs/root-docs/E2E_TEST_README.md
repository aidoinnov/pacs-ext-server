# E2E Test for Viewer APIs with Pagination

## 개요

세 가지 Viewer API의 페이지네이션 기능을 검증하는 Python E2E 테스트입니다.

## 테스트 대상 API

1. ✅ **POST /api/v1/viewer/studies/meta** - Study Meta Batch API
2. ✅ **POST /api/v1/viewer/series/meta** - Series Meta Batch API
3. ✅ **GET /api/v1/viewer/studies/{study_uid}/series/meta** - Study Series Meta API

## 테스트 시나리오

### Test 1: Study Meta API
- **1.1**: 기본 페이지네이션 (파라미터 없음)
- **1.2**: 커스텀 페이지네이션 (page=1, page_size=1)

### Test 2: Series Meta API
- **2.1**: 기본 페이지네이션 (파라미터 없음)
- **2.2**: 커스텀 페이지네이션 (page=1, page_size=1)

### Test 3: Study Series Meta API
- **3.1**: 기본 페이지네이션 (쿼리 파라미터 없음)
- **3.2**: 커스텀 페이지네이션 (page=1, page_size=5)
- **3.3**: 페이지 크기 제한 (page_size=500 → 200으로 클램핑)
- **3.4**: 네비게이션 플래그 (has_next, has_previous)

## 검증 항목

각 테스트는 다음을 검증합니다:

### 페이지네이션 구조 검증
- ✅ `pagination` 필드 존재
- ✅ `page` 필드 존재 및 값 검증
- ✅ `page_size` 필드 존재 및 값 검증
- ✅ `total_items` 필드 존재
- ✅ `total_pages` 필드 존재
- ✅ `has_next` 필드 존재 및 로직 검증
- ✅ `has_previous` 필드 존재 및 로직 검증

### 데이터 검증
- ✅ 응답 데이터 필드 존재 (`studies`, `series`)
- ✅ 페이지 크기에 맞는 데이터 개수 반환
- ✅ Study UID 일치 (Study Series Meta API)
- ✅ Study Description 포함 (Study Series Meta API)

### 제한 검증
- ✅ `page_size` 최대값 200으로 클램핑

## 사전 준비

### 1. Python 환경
```bash
# Python 3.7 이상 필요
python3 --version

# requests 라이브러리 설치
pip3 install requests
```

### 2. 서버 실행
```bash
cd pacs-server
cargo run --bin pacs_server
```

### 3. 테스트 데이터 설정

`test_viewer_apis_e2e.py` 파일에서 실제 PACS 데이터로 수정:

```python
# Test data - Replace with actual UIDs from your PACS
TEST_STUDY_UID_1 = "1.2.840.113619.2.55.3.604688433.1234"  # 실제 Study UID
TEST_STUDY_UID_2 = "1.2.840.113619.2.55.3.604688433.5678"  # 실제 Study UID
TEST_SERIES_UID_1 = "1.2.840.113619.2.55.3.604688433.1234.1"  # 실제 Series UID
TEST_SERIES_UID_2 = "1.2.840.113619.2.55.3.604688433.1234.2"  # 실제 Series UID
```

### 4. 인증 정보 설정 (필요시)

```python
USERNAME = "admin"  # 실제 사용자명
PASSWORD = "admin123"  # 실제 비밀번호
```

## 실행 방법

### 기본 실행
```bash
./test_viewer_apis_e2e.py
```

또는

```bash
python3 test_viewer_apis_e2e.py
```

## 출력 예시

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

ℹ️  Test 1.2: Custom pagination (page=1, page_size=1)
✅ Test 1.2: Pagination structure valid
ℹ️    Page: 1/2
ℹ️    Items: 1 (Total: 2)
ℹ️    Has next: true, Has previous: false
✅ Test 1.2: Pagination working correctly (returned 1 study)
✅ ✨ Study Meta API: ALL TESTS PASSED

================================================================================
Test 2: POST /api/v1/viewer/series/meta (Series Meta Batch API)
================================================================================

ℹ️  Test 2.1: Default pagination (no page params)
✅ Test 2.1: Pagination structure valid
ℹ️    Page: 1/1
ℹ️    Items: 50 (Total: 2)
ℹ️    Has next: false, Has previous: false
✅ Test 2.1: Found 2 series

ℹ️  Test 2.2: Custom pagination (page=1, page_size=1)
✅ Test 2.2: Pagination structure valid
ℹ️    Page: 1/2
ℹ️    Items: 1 (Total: 2)
ℹ️    Has next: true, Has previous: false
✅ Test 2.2: Pagination working correctly (returned 1 series)
✅ ✨ Series Meta API: ALL TESTS PASSED

================================================================================
Test 3: GET /api/v1/viewer/studies/{study_uid}/series/meta (Study Series Meta API)
================================================================================

ℹ️  Test 3.1: Default pagination (no query params)
✅ Test 3.1: Pagination structure valid
ℹ️    Page: 1/1
ℹ️    Items: 50 (Total: 5)
ℹ️    Has next: false, Has previous: false
✅ Test 3.1: Found 5 series for study 1.2.840.113619.2.55.3.604688433.1234
ℹ️    Study Description: Chest CT

ℹ️  Test 3.2: Custom pagination (page=1, page_size=5)
✅ Test 3.2: Pagination structure valid
ℹ️    Page: 1/1
ℹ️    Items: 5 (Total: 5)
ℹ️    Has next: false, Has previous: false
✅ Test 3.2: Pagination working correctly (returned 5 series)

ℹ️  Test 3.3: Page size limit (page_size=500 should be clamped to 200)
✅ Test 3.3: Pagination structure valid
ℹ️    Page: 1/1
ℹ️    Items: 200 (Total: 5)
ℹ️    Has next: false, Has previous: false
✅ Test 3.3: Page size correctly clamped to 200

ℹ️  Test 3.4: Navigation flags (has_next, has_previous)
✅ Test 3.4: Pagination structure valid
ℹ️    Page: 1/3
ℹ️    Items: 2 (Total: 5)
ℹ️    Has next: true, Has previous: false
✅ Test 3.4a: First page navigation flags correct
✅ Test 3.4: Pagination structure valid
ℹ️    Page: 2/3
ℹ️    Items: 2 (Total: 5)
ℹ️    Has next: true, Has previous: true
✅ Test 3.4b: Second page navigation flags correct
✅ ✨ Study Series Meta API: ALL TESTS PASSED

================================================================================
🎉 ALL TESTS PASSED!
================================================================================

✅ POST /api/v1/viewer/studies/meta - Pagination working
✅ POST /api/v1/viewer/series/meta - Pagination working
✅ GET /api/v1/viewer/studies/{study_uid}/series/meta - Pagination working

ℹ️  All three Viewer APIs have been successfully tested with pagination!
ℹ️  Pagination features verified:
ℹ️    - Default pagination (page=1, page_size=50)
ℹ️    - Custom pagination (page, page_size)
ℹ️    - Page size clamping (max 200)
ℹ️    - Navigation flags (has_next, has_previous)
ℹ️    - Pagination info structure (page, page_size, total_items, total_pages)
```

## 실패 시 확인 사항

### 1. 서버가 실행 중인지 확인
```bash
curl http://localhost:8080/health
```

### 2. 테스트 데이터 확인
- PACS에 실제 존재하는 Study UID와 Series UID를 사용하고 있는지 확인
- 사용자가 해당 데이터에 접근 권한이 있는지 확인

### 3. 인증 정보 확인
- 올바른 사용자명과 비밀번호를 사용하고 있는지 확인

### 4. 로그 확인
```bash
# 서버 로그에서 에러 확인
tail -f pacs-server/logs/app.log
```

## 종료 코드

- **0**: 모든 테스트 성공
- **1**: 테스트 실패

## 파일 구조

```
test_viewer_apis_e2e.py    # E2E 테스트 스크립트
E2E_TEST_README.md         # 이 문서
```

## 추가 정보

- 테스트는 실제 PACS 서버와 통신합니다
- 각 API는 독립적으로 테스트됩니다
- 모든 테스트는 페이지네이션 기능을 검증합니다
- 컬러 출력으로 결과를 쉽게 확인할 수 있습니다

