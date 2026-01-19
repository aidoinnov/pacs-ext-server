# 🧪 PACS Server 테스트 가이드

## 📋 목차
- [테스트 개요](#테스트-개요)
- [E2E 테스트](#e2e-테스트)
- [전체 시스템 테스트](#전체-시스템-테스트)
- [개별 테스트 실행](#개별-테스트-실행)
- [테스트 작성 가이드](#테스트-작성-가이드)

---

## 테스트 개요

### 테스트 구조
```
pacs-ext-server/
├── pacs-server/e2e/          # PACS Server E2E 테스트 (12개)
│   └── run_all_tests.sh      # 일괄 실행 스크립트
└── tests/e2e/                # 전체 시스템 E2E 테스트 (21개)
    ├── run_all_tests.sh      # 일괄 실행 스크립트 (Bash)
    └── run_all_tests.py      # 일괄 실행 스크립트 (Python)
```

---

## E2E 테스트

### PACS Server E2E 테스트 (pacs-server/e2e/)

#### 전체 테스트 실행
```bash
cd pacs-server/e2e
./run_all_tests.sh
```

#### 테스트 카테고리

**📦 Annotation 테스트 (5개)**
- `test_annotation_head_request.py` - HEAD 요청 및 캐시 검증
- `test_annotation_level_filtering.py` - Study/Series/Instance 레벨 필터링
- `test_annotation_version_conflict.py` - Optimistic Locking 버전 충돌
- `test_annotation_permission_filtering.py` - 권한 기반 필터링
- `test_annotation_snapshot_e2e.py` - 스냅샷 업로드

**🏥 DICOM Gateway 테스트 (3개)**
- `test_dicom_gateway_study_series_e2e.py` - DICOM Gateway Study/Series
- `test_dicom_gateway_report_status_filter_e2e.py` - Report Status Filter
- `test_qido_enhanced_e2e.py` - QIDO Enhanced

**📊 Series 테스트 (5개)**
- `test_series_note_e2e.py` - Series Note
- `test_series_report_e2e.py` - Series Report
- `test_series_resource_level_e2e.py` - Series Resource Level
- `test_series_uid_api_e2e.py` - Series UID API
- `test_series_user_report_api_e2e.py` - Series User Report API

**🖥️ Viewer 테스트 (3개)**
- `test_viewer_api_e2e.py` - Viewer API
- `test_view_selection_e2e.py` - View Selection
- `test_study_list_view_e2e.py` - Study List View

#### 개별 테스트 실행
```bash
cd pacs-server/e2e
python3 test_annotation_head_request.py
```

---

## 전체 시스템 테스트

### 전체 시스템 E2E 테스트 (tests/e2e/)

#### 전체 테스트 실행 (Bash)
```bash
cd tests/e2e
./run_all_tests.sh
```

#### 전체 테스트 실행 (Python)
```bash
cd tests/e2e
source venv/bin/activate
python run_all_tests.py
```

#### 주요 테스트

**인증 및 기본 기능**
- `test_01_auth.py` - 인증 테스트 (5개 시나리오)
- `test_02_project.py` - 프로젝트 관리
- `test_03_annotation.py` - 어노테이션 CRUD
- `test_04_snapshot.py` - 스냅샷 업로드

**RECIST Lesion 관리**
- `test_11_lesion_assignment.py` - RECIST Lesion Assignment (6개 시나리오)
  - TARGET, NON_TARGET, TARGET_NEW, NON_TARGET_NEW
  - 생성, 수정, 삭제, 조회

**TimePoint 관리**
- `test_12_timepoint_with_studies.py` - TimePoint with Studies (4개 시나리오)
  - X축 API: TimePoints with Studies
  - Y축 API: Annotations by TimePoint

**성능 테스트**
- `test_performance_01_concurrent.py` - 동시성 테스트 (3개 시나리오)

#### 개별 테스트 실행
```bash
cd tests/e2e
source venv/bin/activate
pytest test_11_lesion_assignment.py -v -s
```

---

## 개별 테스트 실행

### pytest 사용
```bash
# 특정 테스트 파일 실행
pytest test_01_auth.py -v

# 특정 테스트 클래스 실행
pytest test_01_auth.py::TestAuthentication -v

# 특정 테스트 메서드 실행
pytest test_01_auth.py::TestAuthentication::test_01_login_success -v

# 상세 출력 (-s)
pytest test_01_auth.py -v -s

# 실패 시 즉시 중단 (-x)
pytest test_01_auth.py -v -x
```

### Python 직접 실행
```bash
python3 test_annotation_head_request.py
```

---

## 테스트 작성 가이드

### PACS Server E2E 테스트 작성

**필독 문서:**
- [E2E_TEST_RULES.md](./e2e/E2E_TEST_RULES.md) - 테스트 작성 규칙
- [REFACTORING_GUIDE.md](./e2e/REFACTORING_GUIDE.md) - 리팩토링 가이드

**핵심 규칙:**
1. 항상 `BaseE2ETest` 상속
2. 자동 클린업 사용 (`self.created_annotation_ids`)
3. 공통 모듈 활용 (`TestConfig`, `TestAuth`, `TestPrinter`)
4. 설정 하드코딩 금지

**예시:**
```python
from test_base import BaseE2ETest

class TestMyFeature(BaseE2ETest):
    def test_my_scenario(self):
        # 테스트 로직
        annotation_id = self.create_test_annotation()
        self.created_annotation_ids.append(annotation_id)
        # 자동으로 cleanup됨
```

### 전체 시스템 E2E 테스트 작성

**pytest 사용:**
```python
import pytest
from utils.api_client import APIClient

@pytest.fixture(scope="module")
def admin_client(config):
    client = APIClient(config.base_url, config.timeout)
    client.login("reader1_user", "Qlalfqjsgh1!")
    yield client
    client.close()

class TestMyFeature:
    def test_my_scenario(self, admin_client):
        response = admin_client.get("/api/endpoint")
        assert response.status_code == 200
```

---

## 테스트 결과 확인

### 성공 예시
```
========================================
📊 테스트 결과 요약
========================================
총 테스트:  6
통과:      6
실패:      0

🎉 모든 테스트 통과!
```

### 실패 시 디버깅
```bash
# 상세 로그 확인
pytest test_file.py -v -s --tb=long

# 특정 테스트만 재실행
pytest test_file.py::test_name -v -s
```

---

## 관련 문서

- [실행 가이드](./RUN.md)
- [도구 스크립트 가이드](./TOOLS.md)
- [E2E 테스트 규칙](./e2e/E2E_TEST_RULES.md)
- [API 문서](../docs/api/)

