# E2E 테스트 빠른 시작 가이드

## 🚀 5분 안에 시작하기

### 1. 새 테스트 작성하기

```python
#!/usr/bin/env python3
"""내 새로운 E2E 테스트"""

from test_base import BaseE2ETest, TestConfig, TestPrinter
from test_fixtures import AnnotationFixtures

class MyNewTest(BaseE2ETest):
    """내 테스트 클래스"""
    
    def get_test_name(self) -> str:
        return "내 새로운 테스트"
    
    def run_tests(self):
        """테스트 실행"""
        TestPrinter.print_header("테스트 1: 기능 테스트")
        
        # 어노테이션 생성
        ann_id = AnnotationFixtures.create_basic_annotation(
            self.token,
            description="테스트용 어노테이션"
        )
        self.created_annotation_ids.append(ann_id)  # ⭐ 자동 삭제됨!
        
        # 테스트 로직...
        TestPrinter.print_success("테스트 통과!", indent=1)

if __name__ == '__main__':
    test = MyNewTest()
    test.run()  # setup/teardown 자동 실행
```

### 2. 실행하기

```bash
cd pacs-server/e2e
python my_new_test.py
```

### 3. 결과 확인

```
🚀 내 새로운 테스트 시작...

🔐 로그인 중: iaid-pacs-admin
✅ 로그인 성공

======================================================================
테스트 1: 기능 테스트
======================================================================
   ✅ 테스트 통과!

======================================================================
🎉 모든 테스트 통과!
======================================================================

🧹 Cleanup: 1개 어노테이션 삭제 중...
   ✓ Deleted annotation ID: 1234
✅ Cleanup 완료: 1개 삭제 성공, 0개 실패
```

---

## 📚 자주 사용하는 패턴

### 패턴 1: 여러 어노테이션 생성

```python
def run_tests(self):
    # 방법 1: 반복문
    for i in range(3):
        ann_id = AnnotationFixtures.create_basic_annotation(
            self.token,
            description=f"Test annotation {i+1}"
        )
        self.created_annotation_ids.append(ann_id)
    
    # 방법 2: 헬퍼 사용
    ann_ids = AnnotationFixtures.create_multiple_annotations(
        self.token,
        count=5,
        description_prefix="Batch test"
    )
    self.created_annotation_ids.extend(ann_ids)
```

### 패턴 2: 레벨별 어노테이션

```python
def run_tests(self):
    # Study 레벨
    study_id = AnnotationFixtures.create_study_level_annotation(self.token)
    self.created_annotation_ids.append(study_id)
    
    # Series 레벨
    series_id = AnnotationFixtures.create_series_level_annotation(self.token)
    self.created_annotation_ids.append(series_id)
    
    # Instance 레벨
    instance_id = AnnotationFixtures.create_instance_level_annotation(self.token)
    self.created_annotation_ids.append(instance_id)
    
    # 또는 한 번에
    ann_ids = AnnotationFixtures.create_all_level_annotations(self.token)
    self.created_annotation_ids.extend(ann_ids)
```

### 패턴 3: 사용자 생성

```python
def run_tests(self):
    # 테스트 사용자 생성
    result = UserFixtures.create_test_user()
    if result:
        user_id, username = result
        self.created_user_ids.append(user_id)  # 자동 삭제됨!
        
        TestPrinter.print_info(f"Created user: {username}", indent=1)
```

### 패턴 4: API 호출

```python
import requests

def run_tests(self):
    # GET 요청
    response = requests.get(
        f"{TestConfig.BASE_URL}/api/annotations",
        headers={"Authorization": f"Bearer {self.token}"},
        timeout=30
    )
    
    if response.status_code == 200:
        TestPrinter.print_success("조회 성공!", indent=1)
    else:
        TestPrinter.print_error(f"조회 실패: {response.status_code}")
```

### 패턴 5: 여러 테스트 메서드

```python
class MyTest(BaseE2ETest):
    def get_test_name(self) -> str:
        return "복합 테스트"
    
    def run_tests(self):
        self.test_create()
        self.test_update()
        self.test_delete()
    
    def test_create(self):
        TestPrinter.print_header("테스트 1: 생성")
        # ...
    
    def test_update(self):
        TestPrinter.print_header("테스트 2: 수정")
        # ...
    
    def test_delete(self):
        TestPrinter.print_header("테스트 3: 삭제")
        # ...
```

---

## 🎨 출력 포맷

### 헤더

```python
TestPrinter.print_header("테스트 1: 기능 테스트")
# ======================================================================
# 테스트 1: 기능 테스트
# ======================================================================
```

### 성공

```python
TestPrinter.print_success("테스트 통과!", indent=1)
#    ✅ 테스트 통과!
```

### 정보

```python
TestPrinter.print_info("Status: 200", indent=1)
#    Status: 200
```

### 경고

```python
TestPrinter.print_warning("일부 기능 스킵", indent=1)
#    ⚠️  일부 기능 스킵
```

### 에러

```python
TestPrinter.print_error("테스트 실패")
# ❌ 테스트 실패
```

---

## 🔧 자주 사용하는 설정

```python
from test_base import TestConfig

# 서버 URL
TestConfig.BASE_URL  # "http://localhost:8080"

# 프로젝트 ID
TestConfig.DEFAULT_PROJECT_ID  # 1

# DICOM UIDs
TestConfig.DEFAULT_STUDY_UID
TestConfig.DEFAULT_SERIES_UID
TestConfig.DEFAULT_INSTANCE_UID
```

---

## ✅ 체크리스트

새 테스트 작성 시:

- [ ] `BaseE2ETest` 상속
- [ ] `get_test_name()` 구현
- [ ] `run_tests()` 구현
- [ ] 생성한 리소스를 `self.created_*_ids`에 추가
- [ ] `TestConfig` 사용 (하드코딩 금지)
- [ ] `TestPrinter` 사용
- [ ] `if __name__ == '__main__':` 블록 추가

---

## 📖 더 알아보기

- **[E2E_TEST_RULES.md](./E2E_TEST_RULES.md)** - 전체 규칙
- **[REFACTORING_GUIDE.md](./REFACTORING_GUIDE.md)** - 상세 가이드
- **[test_base.py](./test_base.py)** - 소스 코드
- **[test_fixtures.py](./test_fixtures.py)** - 픽스처 소스

---

## 💡 팁

1. **기존 테스트 참고하기**
   - `test_annotation_level_filtering_refactored.py` (가장 깔끔)
   - `test_annotation_head_request_refactored.py`

2. **에러 발생 시**
   - 클린업은 자동으로 실행됨 (걱정 안 해도 됨)
   - 에러 메시지 확인
   - 서버 로그 확인

3. **디버깅**
   - `TestPrinter.print_info()`로 중간 값 출력
   - `response.text`로 API 응답 확인

