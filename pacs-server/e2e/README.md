# PACS Server E2E 테스트

이 디렉토리에는 PACS Server의 E2E (End-to-End) 테스트가 포함되어 있습니다.

## 🚀 빠른 시작

### 전체 테스트 실행
```bash
cd pacs-server/e2e
./run_all_tests.sh
```

### 개별 테스트 실행
```bash
python3 test_dicom_gateway_study_series_e2e.py
```

## 📂 테스트 구조

모든 E2E 테스트는 다음 3단계 구조를 따릅니다:

```
1. 사전준비 (Setup)
   - 테스트 계정 생성 (또는 기존 계정 사용)
   - 필요한 데이터 생성 (프로젝트, Study, Annotation 등)

2. 본 테스트 (Test)
   - 실제 테스트 시나리오 실행
   - API 호출 및 응답 검증

3. 클린업 (Cleanup)
   - 생성한 데이터 정리
   - 테스트 계정 삭제
```

### 공통 유틸리티 (`test_common.py`)

모든 테스트에서 사용하는 공통 함수들:

- `get_admin_token()` - 관리자 로그인
- `create_test_user()` - 테스트 사용자 생성
- `create_test_project()` - 테스트 프로젝트 생성
- `add_user_to_project()` - 사용자를 프로젝트에 추가
- `cleanup_project()` - 프로젝트 삭제
- `cleanup_user()` - 사용자 삭제
- `health_check()` - 서버 헬스 체크

## 🎯 새로운 테스트 작성 시 필독!

**⚠️ 중요**: 새로운 E2E 테스트를 작성하거나 기존 테스트를 수정할 때는 반드시 다음 문서를 먼저 읽으세요:

- **[CONNECTION_INFO.md](./CONNECTION_INFO.md)** - 연결 정보 및 설정
- **[E2E_TEST_RULES.md](./E2E_TEST_RULES.md)** ⭐ 가장 중요!
- [REFACTORING_GUIDE.md](./REFACTORING_GUIDE.md) - 상세 가이드
- [CLEANUP_VERIFICATION.md](./CLEANUP_VERIFICATION.md) - 클린업 검증

### 핵심 규칙 요약

1. **항상 `BaseE2ETest` 상속** - 독립 스크립트 금지
2. **자동 클린업 사용** - `self.created_annotation_ids`에 추가만 하면 자동 삭제
3. **공통 모듈 활용** - `TestConfig`, `TestAuth`, `TestPrinter`, `AnnotationFixtures`
4. **설정 하드코딩 금지** - `TestConfig` 사용
5. **3단계 구조 준수** - Setup → Test → Cleanup

자세한 내용은 [E2E_TEST_RULES.md](./E2E_TEST_RULES.md)를 참고하세요.

## 📂 공통 모듈

### 핵심 파일
- **`test_base.py`** - 테스트 베이스 클래스, 설정, 인증, 출력 포맷
- **`test_fixtures.py`** - 테스트 데이터 생성 헬퍼 (어노테이션, 사용자, 이미지)
- **`test_utils.py`** - 유틸리티 함수 (cleanup 등)

### 문서
- **`E2E_TEST_RULES.md`** - E2E 테스트 작성 규칙 (필독!)
- **`REFACTORING_GUIDE.md`** - 리팩토링 상세 가이드
- **`REFACTORING_SUMMARY.md`** - 리팩토링 요약
- **`CLEANUP_VERIFICATION.md`** - 클린업 검증 보고서

## 📋 테스트 목록

### 📦 Annotation 테스트
- ✅ `test_annotation_level_filtering.py` - Study/Series/Instance 레벨 필터링
- ✅ `test_annotation_version_conflict.py` - Optimistic Locking 버전 충돌
- ✅ `test_annotation_head_request.py` - HEAD 요청 및 캐시 검증
- ✅ `test_annotation_snapshot_e2e.py` - 스냅샷 이미지 업로드
- ✅ `test_annotation_permission_filtering.py` - 권한 기반 필터링
- ✅ `test_annotation_permission_management.py` - 권한 관리

### 🏥 DICOM Gateway 테스트
- ✅ `test_dicom_gateway_study_series_e2e.py` - DICOM Gateway Study/Series
- ✅ `test_dicom_gateway_report_status_filter_e2e.py` - Report Status Filter
- ✅ `test_qido_enhanced_e2e.py` - QIDO Enhanced

### 📊 Series 테스트
- ✅ `test_series_note_e2e.py` - Series Note
- ✅ `test_series_report_e2e.py` - Series Report
- ✅ `test_series_resource_level_e2e.py` - Series Resource Level
- ✅ `test_series_uid_api_e2e.py` - Series UID API
- ✅ `test_series_user_report_api_e2e.py` - Series User Report API

### 🖥️ Viewer 테스트
- ✅ `test_viewer_api_e2e.py` - Viewer API
- ✅ `test_view_selection_e2e.py` - View Selection
- ✅ `test_study_list_view_e2e.py` - Study List View

### 리팩토링 완료 (권장)
- ✅ `test_annotation_level_filtering_refactored.py` - Study/Series/Instance 레벨 필터링
- ✅ `test_annotation_version_conflict_refactored.py` - Optimistic Locking 버전 충돌
- ✅ `test_annotation_head_request_refactored.py` - HEAD 요청 및 캐시 검증
- ✅ `test_annotation_snapshot_e2e_refactored.py` - 스냅샷 이미지 업로드
- ✅ `test_annotation_permission_filtering_refactored.py` - 권한 기반 필터링
- ✅ `test_annotation_permission_management_refactored.py` - 권한 관리
- ⚠️ `test_annotation_permission_filtering.py` - (리팩토링 버전 사용 권장)
- ⚠️ `test_annotation_permission_management.py` - (리팩토링 버전 사용 권장)

### 1. 기본 API 테스트
- **파일**: `test_annotation_api_debug.py`
- **설명**: 어노테이션 생성, 조회, 삭제 등 기본 API 동작 확인
- **테스트 항목**:
  - 어노테이션 생성
  - 어노테이션 삭제
  - Series UID로 조회
  - SOP Instance UID로 조회

### 2. 권한 기반 필터링 테스트
- **파일**: `test_annotation_permission_filtering.py`
- **설명**: 사용자 권한에 따른 어노테이션 필터링 검증
- **테스트 항목**:
  - ADMIN 사용자: 모든 어노테이션 조회
  - 일반 사용자: 본인 어노테이션만 조회
  - Series UID 필터링 + 권한 필터링

### 3. 레벨 필터링 테스트
- **파일**: `test_annotation_level_filtering.py`
- **설명**: Study/Series/Instance 레벨 필터링 검증
- **테스트 항목**:
  - Study 레벨 필터링
  - Series 레벨 필터링
  - Instance 레벨 필터링

### 4. 버전 충돌 테스트
- **파일**: `test_annotation_version_conflict.py`
- **설명**: Optimistic Locking을 통한 동시 업데이트 충돌 처리 검증
- **테스트 항목**:
  - 버전 일치 시 업데이트 성공
  - 버전 불일치 시 409 Conflict
  - 동시 업데이트 시나리오

### 5. HEAD 요청 테스트
- **파일**: `test_annotation_head_request.py`
- **설명**: HEAD 요청을 통한 캐시 검증 및 리소스 존재 확인
- **테스트 항목**:
  - ETag 기반 캐시 검증
  - Last-Modified 기반 캐시 검증
  - 리소스 존재 확인
  - 어노테이션 목록 HEAD 요청

### 6. 스냅샷 업로드 테스트
- **파일**: `test_annotation_snapshot_e2e.py`
- **설명**: 어노테이션 스냅샷 이미지 업로드 전체 워크플로우 검증
- **테스트 항목**:
  - 어노테이션 생성
  - 업로드 URL 요청
  - S3 업로드
  - 업로드 완료 처리
  - 상태 조회

## 🚀 실행 방법

### 전체 테스트 실행
```bash
cd pacs-server/e2e
./run_all_tests.sh
```

### 주제별 테스트 실행

#### 캐시 테스트
```bash
./run_cache_tests.sh
```

#### 필터링 테스트
```bash
./run_filtering_tests.sh
```

#### 버전 관리 테스트
```bash
./run_version_tests.sh
```

#### 스냅샷 테스트
```bash
./run_snapshot_tests.sh
```

### 개별 테스트 실행
```bash
cd pacs-server/e2e

# 기본 API 테스트
python3 test_annotation_api_debug.py

# 권한 필터링 테스트
python3 test_annotation_permission_filtering.py

# 레벨 필터링 테스트
python3 test_annotation_level_filtering.py

# 버전 충돌 테스트
python3 test_annotation_version_conflict.py

# HEAD 요청 테스트
python3 test_annotation_head_request.py

# 스냅샷 업로드 테스트
python3 test_annotation_snapshot_e2e.py
```

## ⚙️ 사전 요구사항

1. **서버 실행**: 테스트 실행 전 PACS 서버가 실행 중이어야 합니다.
   ```bash
   cd pacs-server
   cargo run --bin pacs_server
   ```

2. **Python 패키지**: 필요한 Python 패키지 설치
   ```bash
   pip install requests pillow
   ```

3. **테스트 사용자**: 다음 사용자가 데이터베이스에 존재해야 합니다.
   - `iaid-pacs-admin` (SUPER_ADMIN 권한)
   - `iaid-pacs-user1` (일반 사용자)

4. **테스트 프로젝트**: 프로젝트 ID 2 (PerfProj)가 존재해야 합니다.

## 📊 테스트 결과 예시

```
🚀 어노테이션 E2E 테스트 실행 시작...

✅ 서버 실행 중

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📋 테스트: 기본 API 디버그 테스트
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✅ 기본 API 디버그 테스트 통과

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📊 테스트 결과 요약
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
총 테스트: 6
통과: 6
실패: 0

🎉 모든 테스트 통과!
```

## 🔧 트러블슈팅

### 서버 연결 실패
```
❌ 테스트 실패: Connection refused
```
→ 서버가 실행 중인지 확인하세요.

### 로그인 실패
```
❌ 로그인 실패: 401
```
→ 사용자 계정과 비밀번호를 확인하세요.

### 권한 에러
```
❌ 테스트 실패: 403 Forbidden
```
→ 사용자가 프로젝트에 할당되어 있는지 확인하세요.

## 🧹 Cleanup

모든 테스트는 자동으로 cleanup을 수행합니다:
- 테스트 중 생성된 어노테이션은 테스트 종료 시 자동으로 삭제됩니다
- `test_utils.py`의 `cleanup_annotations()` 함수를 사용합니다
- 테스트 실패 시에도 `finally` 블록에서 cleanup이 실행됩니다

### TestContext 사용 예시
```python
from test_utils import TestContext

with TestContext("username", "password") as ctx:
    # 어노테이션 생성 및 자동 추적
    ann_id = ctx.create_and_track(annotation_data)

    # 테스트 수행...

# 컨텍스트 종료 시 자동으로 cleanup 수행
```

## 📝 참고사항

- 모든 테스트는 독립적으로 실행 가능합니다.
- 테스트 실행 시 실제 데이터베이스에 데이터가 생성됩니다.
- **모든 테스트는 생성한 데이터를 자동으로 삭제합니다** (cleanup 로직 포함)

