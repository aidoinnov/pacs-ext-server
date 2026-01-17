# E2E 테스트 리팩토링 가이드

## 📋 개요

기존 E2E 테스트 코드의 중복을 제거하고 재사용성을 높이기 위한 리팩토링 가이드입니다.

## 🔍 문제점 분석

### 1. 중복 코드
- 모든 테스트 파일에 동일한 `login()` 함수
- 동일한 상수 (`BASE_URL`, `USER_ID`, `USER_PASSWORD`)
- 비슷한 어노테이션 생성 로직
- 동일한 cleanup 패턴
- 반복되는 출력 포맷

### 2. 유지보수 어려움
- 설정 변경 시 모든 파일 수정 필요
- 테스트 구조 변경 시 일관성 유지 어려움
- 새로운 테스트 작성 시 보일러플레이트 코드 반복

## 🎯 리팩토링 전략

### 새로운 파일 구조

```
pacs-server/e2e/
├── test_base.py              # 베이스 클래스 및 공통 설정
├── test_fixtures.py          # 테스트 데이터 생성 헬퍼
├── test_utils.py             # 기존 유틸리티 (유지)
├── test_annotation_level_filtering_refactored.py
├── test_annotation_version_conflict_refactored.py
└── ...
```

### 주요 컴포넌트

#### 1. `test_base.py`
- **TestConfig**: 공통 설정 (URL, 인증 정보, DICOM UIDs)
- **TestAuth**: 인증 관련 유틸리티
- **TestPrinter**: 일관된 출력 포맷
- **BaseE2ETest**: 테스트 베이스 클래스 (setup/teardown 자동화)

#### 2. `test_fixtures.py`
- **AnnotationFixtures**: 어노테이션 테스트 데이터 생성
  - `create_basic_annotation()`: 기본 어노테이션
  - `create_study_level_annotation()`: Study 레벨
  - `create_series_level_annotation()`: Series 레벨
  - `create_instance_level_annotation()`: Instance 레벨
  - `create_all_level_annotations()`: 모든 레벨 생성

## 📊 비교: 기존 vs 리팩토링

### 기존 코드 (234줄)
```python
#!/usr/bin/env python3
import requests
import json
from test_utils import cleanup_annotations

BASE_URL = "http://localhost:8080"
USER_ID = 'iaid-pacs-admin'
USER_PASSWORD = 'Qlalfqjsgh1!'

def login():
    """로그인하여 JWT 토큰 획득"""
    print("🔐 로그인 중...")
    response = requests.post(
        f"{BASE_URL}/api/auth/login",
        json={"username": USER_ID, "password": USER_PASSWORD},
        timeout=5
    )
    # ... 반복되는 코드 ...

def create_test_annotations(token: str):
    # ... 긴 어노테이션 생성 로직 ...

if __name__ == '__main__':
    created_ids = []
    token = None
    try:
        # ... setup ...
    finally:
        # ... cleanup ...
```

### 리팩토링 코드 (120줄)
```python
#!/usr/bin/env python3
from test_base import BaseE2ETest, TestConfig, TestPrinter
from test_fixtures import AnnotationFixtures

class AnnotationLevelFilteringTest(BaseE2ETest):
    def get_test_name(self) -> str:
        return "어노테이션 레벨 필터링 E2E 테스트"
    
    def run_tests(self):
        self.created_annotation_ids = AnnotationFixtures.create_all_level_annotations(self.token)
        self.test_level_filter_study()
        self.test_level_filter_series()
        self.test_level_filter_instance()
    
    def test_level_filter_study(self):
        # 테스트 로직만 집중

if __name__ == '__main__':
    test = AnnotationLevelFilteringTest()
    test.run()
```

## ✅ 개선 효과

### 1. 코드 감소
- **기존**: 234줄 → **리팩토링**: 120줄 (약 50% 감소)
- 보일러플레이트 코드 제거
- 테스트 로직에만 집중

### 2. 재사용성 향상
- 공통 설정 한 곳에서 관리
- 테스트 데이터 생성 로직 재사용
- 일관된 출력 포맷

### 3. 유지보수성 향상
- 설정 변경 시 `TestConfig`만 수정
- 새로운 테스트 작성 시 `BaseE2ETest` 상속
- 일관된 구조로 가독성 향상

### 4. 확장성
- 새로운 픽스처 추가 용이
- 테스트 베이스 클래스 확장 가능
- 공통 유틸리티 추가 간편

## 🚀 마이그레이션 가이드

### 기존 테스트를 리팩토링하는 방법

1. **BaseE2ETest 상속**
   ```python
   class MyTest(BaseE2ETest):
       def get_test_name(self) -> str:
           return "테스트 이름"
       
       def run_tests(self):
           # 테스트 로직
   ```

2. **공통 설정 사용**
   ```python
   # 기존
   BASE_URL = "http://localhost:8080"
   
   # 리팩토링
   from test_base import TestConfig
   TestConfig.BASE_URL
   ```

3. **픽스처 사용**
   ```python
   # 기존
   def create_test_annotation(token):
       # 긴 생성 로직...
   
   # 리팩토링
   from test_fixtures import AnnotationFixtures
   annotation_id = AnnotationFixtures.create_basic_annotation(self.token)
   ```

4. **출력 포맷 통일**
   ```python
   # 기존
   print("✅ 테스트 통과")
   
   # 리팩토링
   from test_base import TestPrinter
   TestPrinter.print_success("테스트 통과")
   ```

## 📝 다음 단계

1. ✅ `test_base.py` 생성
2. ✅ `test_fixtures.py` 생성
3. ✅ 예시 리팩토링 (level_filtering, version_conflict)
4. ⬜ 나머지 테스트 파일 리팩토링
5. ⬜ 기존 파일 제거 또는 deprecated 표시
6. ⬜ CI/CD 파이프라인 업데이트

## 🔧 추가 개선 사항

### 향후 고려사항
- **pytest 도입**: 더 강력한 테스트 프레임워크
- **테스트 병렬화**: 실행 시간 단축
- **Mock 서버**: 외부 의존성 제거
- **테스트 리포트**: HTML 리포트 생성

