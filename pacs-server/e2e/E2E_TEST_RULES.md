# E2E 테스트 작성 규칙 및 자동화

> **중요**: 이 문서는 AI 어시스턴트가 세션이 바뀔 때마다 참고해야 하는 **필수 규칙**입니다.

## 🎯 핵심 원칙

### 1. **반드시 `BaseE2ETest` 상속**
- ❌ 절대 독립적인 스크립트로 작성하지 말 것
- ✅ 항상 `BaseE2ETest`를 상속받아 작성

### 2. **자동 클린업 보장**
- `finally` 블록으로 100% 클린업 보장
- 에러 발생 시에도 반드시 클린업 실행
- 수동 cleanup 호출 금지 (자동화됨)

### 3. **공통 모듈 사용**
- `test_base.py`: 베이스 클래스, 설정, 인증, 출력
- `test_fixtures.py`: 테스트 데이터 생성 헬퍼
- `test_utils.py`: 기존 유틸리티 함수

---

## 📋 필수 구조

### 기본 템플릿

```python
#!/usr/bin/env python3
"""
테스트 설명
"""

from test_base import BaseE2ETest, TestConfig, TestPrinter
from test_fixtures import AnnotationFixtures, UserFixtures

class MyE2ETest(BaseE2ETest):
    """테스트 클래스"""
    
    def get_test_name(self) -> str:
        return "내 테스트 이름"
    
    def run_tests(self):
        """테스트 실행 (핵심 로직만 작성)"""
        # 테스트 로직...
        pass

if __name__ == '__main__':
    test = MyE2ETest()
    test.run()  # setup/teardown 자동 실행
```

---

## 🔒 클린업 규칙

### 자동 클린업 메커니즘

**리소스 생성 시 리스트에 추가만 하면 자동 삭제됨!**

```python
class MyTest(BaseE2ETest):
    def run_tests(self):
        # 1. 어노테이션 생성
        ann_id = AnnotationFixtures.create_basic_annotation(self.token)
        self.created_annotation_ids.append(ann_id)  # ⭐ 자동 삭제됨!
        
        # 2. 사용자 생성
        user_result = UserFixtures.create_test_user()
        if user_result:
            user_id, username = user_result
            self.created_user_ids.append(user_id)  # ⭐ 자동 삭제됨!
        
        # 3. 테스트 실행...
        # teardown()에서 자동으로 모두 삭제됨!
```

### 클린업 보장 원리

```python
# test_base.py의 run() 메서드
def run(self):
    try:
        self.setup()      # 로그인
        self.run_tests()  # 테스트 실행
    except Exception as e:
        print(f"❌ 테스트 실패: {e}")
        exit(1)
    finally:
        self.teardown()   # ⭐ 반드시 실행됨! (에러 발생해도)
```

### 클린업 대상

| 리소스 | 추가할 리스트 | 삭제 함수 |
|--------|--------------|----------|
| 어노테이션 | `self.created_annotation_ids` | `cleanup_annotations()` |
| 사용자 | `self.created_user_ids` | `delete_user()` |

---

## 🛠️ 공통 모듈 사용법

### 1. TestConfig (설정)

```python
from test_base import TestConfig

# 서버 URL
TestConfig.BASE_URL  # "http://localhost:8080"

# 인증 정보
TestConfig.DEFAULT_USER_ID      # "iaid-pacs-admin"
TestConfig.DEFAULT_USER_PASSWORD  # "Qlalfqjsgh1!"

# 프로젝트 ID
TestConfig.DEFAULT_PROJECT_ID  # 1

# DICOM UIDs (중앙화됨!)
TestConfig.DEFAULT_STUDY_UID    # "1.2.840.113619.2.55.3..."
TestConfig.DEFAULT_SERIES_UID   # "1.2.840.113619.2.55.3..."
TestConfig.DEFAULT_INSTANCE_UID # "1.2.840.113619.2.55.3..."

# 스냅샷 테스트용
TestConfig.SNAPSHOT_STUDY_UID
TestConfig.SNAPSHOT_SERIES_UID
TestConfig.SNAPSHOT_INSTANCE_UID
```

### 2. TestAuth (인증)

```python
from test_base import TestAuth

# 로그인 (자동으로 setup()에서 실행됨)
token = TestAuth.login()
token = TestAuth.login(user_id="custom-user", password="custom-pass")
```

### 3. TestPrinter (출력)

```python
from test_base import TestPrinter

# 헤더
TestPrinter.print_header("테스트 1: 기능 테스트")

# 성공
TestPrinter.print_success("테스트 통과!", indent=1)

# 정보
TestPrinter.print_info("Status: 200", indent=1)

# 경고
TestPrinter.print_warning("일부 기능 스킵", indent=1)

# 에러
TestPrinter.print_error("테스트 실패")
```

### 4. AnnotationFixtures (어노테이션 생성)

```python
from test_fixtures import AnnotationFixtures

# 기본 어노테이션
ann_id = AnnotationFixtures.create_basic_annotation(
    token,
    description="테스트용 어노테이션"
)

# Study 레벨 어노테이션
ann_id = AnnotationFixtures.create_study_level_annotation(token)

# Series 레벨 어노테이션
ann_id = AnnotationFixtures.create_series_level_annotation(token)

# Instance 레벨 어노테이션
ann_id = AnnotationFixtures.create_instance_level_annotation(token)

# 모든 레벨 어노테이션 (3개)
ann_ids = AnnotationFixtures.create_all_level_annotations(token)
# 반환: [study_id, series_id, instance_id]

# 여러 개 생성
ann_ids = AnnotationFixtures.create_multiple_annotations(
    token,
    count=5,
    description_prefix="Batch test"
)
```

### 5. UserFixtures (사용자 생성)

```python
from test_fixtures import UserFixtures

# 테스트 사용자 생성
result = UserFixtures.create_test_user()
if result:
    user_id, username = result
    self.created_user_ids.append(user_id)  # 자동 삭제됨!

# 커스텀 사용자
result = UserFixtures.create_test_user(
    username_prefix="custom_user",
    email_prefix="custom"
)
```

### 6. ImageFixtures (이미지 생성)

```python
from test_fixtures import ImageFixtures

# 스냅샷 이미지 생성
image_data = ImageFixtures.create_test_snapshot_image()
# 반환: PNG 바이트 데이터 (800x600)
```

---

## ✅ 체크리스트

새로운 E2E 테스트 작성 시 확인:

- [ ] `BaseE2ETest` 상속했는가?
- [ ] `get_test_name()` 구현했는가?
- [ ] `run_tests()` 구현했는가?
- [ ] 생성한 어노테이션을 `self.created_annotation_ids`에 추가했는가?
- [ ] 생성한 사용자를 `self.created_user_ids`에 추가했는가?
- [ ] `TestConfig`의 설정을 사용하는가? (하드코딩 금지)
- [ ] `TestPrinter`로 일관된 출력을 하는가?
- [ ] `AnnotationFixtures`를 활용하는가?
- [ ] `if __name__ == '__main__':` 블록에서 `test.run()` 호출하는가?

---

## 🚫 금지 사항

### ❌ 하지 말아야 할 것

1. **독립 스크립트 작성**
   ```python
   # ❌ 이렇게 하지 마세요!
   if __name__ == '__main__':
       token = login()
       try:
           # 테스트...
       finally:
           cleanup_annotations(token, created_ids)
   ```

2. **설정 하드코딩**
   ```python
   # ❌ 이렇게 하지 마세요!
   BASE_URL = "http://localhost:8080"
   STUDY_UID = "1.2.840..."
   
   # ✅ 이렇게 하세요!
   from test_base import TestConfig
   url = TestConfig.BASE_URL
   study_uid = TestConfig.DEFAULT_STUDY_UID
   ```

3. **수동 cleanup 호출**
   ```python
   # ❌ 이렇게 하지 마세요!
   finally:
       cleanup_annotations(token, created_ids)
   
   # ✅ 이렇게 하세요!
   self.created_annotation_ids.append(ann_id)  # 자동!
   ```

4. **중복 로그인 코드**
   ```python
   # ❌ 이렇게 하지 마세요!
   def login():
       response = requests.post(...)
       # 30줄의 코드...
   
   # ✅ 이렇게 하세요!
   # BaseE2ETest가 자동으로 로그인함 (self.token 사용)
   ```

---

## 📚 참고 문서

- [REFACTORING_GUIDE.md](./REFACTORING_GUIDE.md) - 상세 가이드
- [REFACTORING_SUMMARY.md](./REFACTORING_SUMMARY.md) - 요약 및 사용법
- [CLEANUP_VERIFICATION.md](./CLEANUP_VERIFICATION.md) - 클린업 검증
- [test_base.py](./test_base.py) - 베이스 클래스 소스
- [test_fixtures.py](./test_fixtures.py) - 픽스처 소스

---

## 🎯 요약

1. **항상 `BaseE2ETest` 상속**
2. **리소스는 리스트에 추가만** → 자동 삭제
3. **공통 모듈 활용** → 중복 제거
4. **설정은 `TestConfig`** → 중앙 관리
5. **출력은 `TestPrinter`** → 일관성 유지

