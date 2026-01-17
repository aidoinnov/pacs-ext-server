# E2E 테스트 문서 인덱스

> **이 문서를 먼저 읽으세요!** 모든 E2E 테스트 관련 문서의 인덱스입니다.

## 🎯 목적별 문서 가이드

### 새 테스트를 작성하고 싶다면
1. **[CONNECTION_INFO.md](./CONNECTION_INFO.md)** - 연결 정보 확인
2. **[QUICK_START.md](./QUICK_START.md)** ⭐ 5분 안에 시작
3. **[E2E_TEST_RULES.md](./E2E_TEST_RULES.md)** - 필수 규칙
4. 기존 테스트 참고: `test_annotation_level_filtering_refactored.py`

### 기존 테스트를 리팩토링하고 싶다면
1. **[REFACTORING_GUIDE.md](./REFACTORING_GUIDE.md)** - 단계별 가이드
2. **[REFACTORING_SUMMARY.md](./REFACTORING_SUMMARY.md)** - 요약
3. **[REFACTORING_COMPLETE.md](./REFACTORING_COMPLETE.md)** - 완료 보고서

### 클린업이 잘 되는지 확인하고 싶다면
1. **[CLEANUP_VERIFICATION.md](./CLEANUP_VERIFICATION.md)** - 검증 보고서
2. **[E2E_TEST_RULES.md](./E2E_TEST_RULES.md)** - 클린업 규칙

### AI 어시스턴트라면
1. **[/.ai-rules.md](../../.ai-rules.md)** ⭐ 프로젝트 전체 규칙
2. **[E2E_TEST_RULES.md](./E2E_TEST_RULES.md)** - E2E 테스트 규칙
3. 세션 시작 시 반드시 읽기!

---

## 📚 전체 문서 목록

### 🚀 시작 가이드
| 문서 | 설명 | 대상 |
|------|------|------|
| **[CONNECTION_INFO.md](./CONNECTION_INFO.md)** | 연결 정보 및 설정 | 모든 개발자 |
| **[QUICK_START.md](./QUICK_START.md)** | 5분 빠른 시작 | 모든 개발자 |
| **[README.md](./README.md)** | E2E 테스트 개요 | 모든 개발자 |
| **[VIEW_SELECTION_GUIDE.md](./VIEW_SELECTION_GUIDE.md)** | View Selection 스크립트 | 개발자 |

### 📋 규칙 및 컨벤션
| 문서 | 설명 | 대상 |
|------|------|------|
| **[E2E_TEST_RULES.md](./E2E_TEST_RULES.md)** | 필수 작성 규칙 | 모든 개발자, AI |
| **[/.ai-rules.md](../../.ai-rules.md)** | AI 어시스턴트 규칙 | AI 어시스턴트 |

### 🔧 리팩토링 가이드
| 문서 | 설명 | 대상 |
|------|------|------|
| **[REFACTORING_GUIDE.md](./REFACTORING_GUIDE.md)** | 상세 리팩토링 가이드 | 리팩토링 작업자 |
| **[REFACTORING_SUMMARY.md](./REFACTORING_SUMMARY.md)** | 리팩토링 요약 | 모든 개발자 |
| **[REFACTORING_COMPLETE.md](./REFACTORING_COMPLETE.md)** | 완료 보고서 | 참고용 |

### ✅ 검증 및 보고서
| 문서 | 설명 | 대상 |
|------|------|------|
| **[CLEANUP_VERIFICATION.md](./CLEANUP_VERIFICATION.md)** | 클린업 검증 | 모든 개발자 |

### 🔍 아키텍처 및 로직
| 문서 | 설명 | 대상 |
|------|------|------|
| **[QIDO_RBAC_FLOW.md](./QIDO_RBAC_FLOW.md)** | QIDO RBAC 로직 순서 | 개발자, 아키텍트 |

---

## 🗂️ 소스 파일

### 공통 모듈
| 파일 | 설명 | 라인 수 |
|------|------|---------|
| **[test_base.py](./test_base.py)** | 베이스 클래스, 설정, 인증, 출력 | 158줄 |
| **[test_fixtures.py](./test_fixtures.py)** | 테스트 데이터 생성 헬퍼 | 263줄 |
| **[test_utils.py](./test_utils.py)** | 유틸리티 함수 | 기존 |

### 리팩토링 완료 테스트 (권장)
| 파일 | 설명 | 라인 수 | 감소율 |
|------|------|---------|--------|
| `test_annotation_level_filtering_refactored.py` | 레벨 필터링 | 120줄 | -49% |
| `test_annotation_version_conflict_refactored.py` | 버전 충돌 | 180줄 | -23% |
| `test_annotation_head_request_refactored.py` | HEAD 요청 | 200줄 | -24% |
| `test_annotation_snapshot_e2e_refactored.py` | 스냅샷 업로드 | 281줄 | -6% |
| `test_annotation_permission_filtering_refactored.py` | 권한 필터링 | 180줄 | -28% |
| `test_annotation_permission_management_refactored.py` | 권한 관리 | 180줄 | -20% |

### 기존 테스트 (레거시)
- `test_annotation_level_filtering.py` (234줄)
- `test_annotation_version_conflict.py` (234줄)
- `test_annotation_head_request.py` (264줄)
- `test_annotation_snapshot_e2e.py` (300줄)
- `test_annotation_permission_filtering.py` (250줄)
- `test_annotation_permission_management.py` (226줄)

---

## 🎓 학습 경로

### 초보자
1. [QUICK_START.md](./QUICK_START.md) - 빠른 시작
2. [E2E_TEST_RULES.md](./E2E_TEST_RULES.md) - 규칙 학습
3. `test_annotation_level_filtering_refactored.py` - 예제 코드 읽기
4. 직접 테스트 작성해보기

### 중급자
1. [REFACTORING_GUIDE.md](./REFACTORING_GUIDE.md) - 리팩토링 방법
2. 기존 테스트 리팩토링 해보기
3. 공통 모듈 확장하기

### 고급자
1. [test_base.py](./test_base.py) - 베이스 클래스 개선
2. [test_fixtures.py](./test_fixtures.py) - 픽스처 추가
3. 새로운 패턴 도입

---

## 🔍 빠른 참조

### 새 테스트 템플릿
```python
from test_base import BaseE2ETest, TestPrinter
from test_fixtures import AnnotationFixtures

class MyTest(BaseE2ETest):
    def get_test_name(self) -> str:
        return "내 테스트"
    
    def run_tests(self):
        ann_id = AnnotationFixtures.create_basic_annotation(self.token)
        self.created_annotation_ids.append(ann_id)

if __name__ == '__main__':
    test = MyTest()
    test.run()
```

### 자주 사용하는 임포트
```python
from test_base import BaseE2ETest, TestConfig, TestAuth, TestPrinter
from test_fixtures import AnnotationFixtures, UserFixtures, ImageFixtures
from test_utils import cleanup_annotations, delete_user
import requests
```

### 자주 사용하는 설정
```python
TestConfig.BASE_URL              # 서버 URL
TestConfig.DEFAULT_PROJECT_ID    # 프로젝트 ID
TestConfig.DEFAULT_STUDY_UID     # Study UID
TestConfig.DEFAULT_SERIES_UID    # Series UID
TestConfig.DEFAULT_INSTANCE_UID  # Instance UID
```

---

## 📞 도움이 필요하면

1. **문서 확인**: 위의 목적별 가이드 참고
2. **예제 코드**: 리팩토링 완료 테스트 참고
3. **규칙 확인**: [E2E_TEST_RULES.md](./E2E_TEST_RULES.md)
4. **AI 어시스턴트**: [/.ai-rules.md](../../.ai-rules.md) 참고

---

## 🎯 핵심 원칙 (요약)

1. ✅ **항상 `BaseE2ETest` 상속**
2. ✅ **리소스는 리스트에 추가** → 자동 삭제
3. ✅ **공통 모듈 활용** → 중복 제거
4. ✅ **설정은 `TestConfig`** → 중앙 관리
5. ✅ **출력은 `TestPrinter`** → 일관성

---

**이 인덱스를 북마크하세요!** 📌

