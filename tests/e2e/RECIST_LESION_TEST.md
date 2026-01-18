# RECIST Lesion E2E 테스트

RECIST 1.1 기준 병변(Lesion) 관리 API에 대한 통합 테스트입니다.

## 📋 테스트 범위

### 1. CRUD 테스트
- ✅ Target Lesion 생성
- ✅ Non-Target Lesion 생성
- ✅ Lesion 목록 조회 (전체 / 타입별)
- ✅ Lesion 상세 조회 (Annotation 포함)
- ✅ Lesion 수정
- ✅ Lesion 삭제

### 2. RECIST 1.1 비즈니스 규칙 검증
- ✅ Max 5 Target Lesions per Subject
- ✅ Baseline TimePoint 필수 (TARGET/NON_TARGET)
- ✅ NEW Lesion은 Baseline TimePoint 없이 생성 가능
- ✅ Non-Target Lesion은 개수 제한 없음

### 3. Annotation 연결
- ✅ Lesion에 Annotation 연결
- ✅ TimePoint별 측정값 저장
- ✅ 상세 조회 시 Annotation 목록 포함

### 4. 에러 케이스
- ✅ 존재하지 않는 Subject
- ✅ 존재하지 않는 TimePoint
- ✅ 존재하지 않는 Lesion 조회/수정/삭제

## 🚀 실행 방법

### 1. 환경 설정

```bash
cd tests/e2e

# 가상환경 생성 (처음 한 번만)
python3 -m venv venv

# 가상환경 활성화
source venv/bin/activate  # macOS/Linux
# or
venv\Scripts\activate  # Windows

# 의존성 설치
pip install -r requirements.txt
```

### 2. 서버 실행

```bash
# 별도 터미널에서 서버 실행
cd pacs-server
cargo run --release --bin pacs_server
```

### 3. 테스트 실행

#### Python 스크립트 사용 (권장)

```bash
# 전체 테스트 실행
python run_recist_lesion.py

# Verbose 모드
python run_recist_lesion.py --verbose

# 특정 테스트만 실행
python run_recist_lesion.py --test test_01_create_target_lesion

# 커스텀 서버 URL
python run_recist_lesion.py --base-url http://localhost:8080
```

#### Bash 스크립트 사용

```bash
# 전체 테스트 실행
./run_recist_lesion.sh
```

#### pytest 직접 사용

```bash
# 전체 테스트 실행
pytest test_07_recist_lesion.py -v

# Verbose 모드 (로그 출력)
pytest test_07_recist_lesion.py -v -s

# 특정 테스트만 실행
pytest test_07_recist_lesion.py::TestRecistLesionCRUD::test_01_create_target_lesion -v -s

# 특정 클래스의 모든 테스트 실행
pytest test_07_recist_lesion.py::TestRecistBusinessRules -v -s
```

## 📊 테스트 구조

```
test_07_recist_lesion.py
├── Fixtures
│   ├── config: 테스트 설정
│   ├── admin_client: 관리자 API 클라이언트
│   ├── test_project: 테스트용 프로젝트
│   ├── test_subject: 테스트용 Subject
│   ├── baseline_timepoint: Baseline TimePoint
│   └── followup_timepoint: Follow-up TimePoint
│
├── TestRecistLesionCRUD
│   ├── test_01_create_target_lesion
│   ├── test_02_create_non_target_lesion
│   ├── test_03_list_lesions
│   ├── test_04_get_lesion_detail
│   ├── test_05_update_lesion
│   └── test_06_delete_lesion
│
├── TestRecistBusinessRules
│   ├── test_01_max_5_target_lesions
│   ├── test_02_baseline_required_for_target
│   ├── test_03_new_lesion_no_baseline
│   └── test_04_non_target_unlimited
│
├── TestAnnotationLinking
│   └── test_01_link_annotation_to_lesion
│
└── TestErrorCases
    ├── test_01_create_lesion_invalid_subject
    ├── test_02_create_lesion_invalid_timepoint
    ├── test_03_get_nonexistent_lesion
    ├── test_04_update_nonexistent_lesion
    └── test_05_delete_nonexistent_lesion
```

## 🔧 환경 변수

```bash
# API 서버 URL
export BASE_URL="http://localhost:8080"

# 관리자 계정
export ADMIN_EMAIL="admin@example.com"
export ADMIN_PASSWORD="admin123"

# 타임아웃 (초)
export TIMEOUT="30"
```

## 📝 테스트 시나리오 예시

### 시나리오 1: Target Lesion 생성 및 관리

1. Project 생성
2. Subject 생성
3. Baseline TimePoint 생성
4. Target Lesion 생성 (최대 5개)
5. Lesion 목록 조회
6. Lesion 상세 조회
7. Lesion 수정
8. Lesion 삭제

### 시나리오 2: RECIST 1.1 규칙 검증

1. 6번째 Target Lesion 생성 시도 → 400 Bad Request
2. Baseline TimePoint 없이 Target Lesion 생성 시도 → 400 Bad Request
3. NEW Lesion 생성 (Baseline TimePoint 없이) → 201 Created
4. 10개 Non-Target Lesion 생성 → 모두 성공

### 시나리오 3: Annotation 연결

1. Target Lesion 생성
2. Annotation 생성 (DICOM 이미지에 대한)
3. Lesion에 Annotation 연결 (측정값 포함)
4. Lesion 상세 조회로 Annotation 확인

## 🐛 트러블슈팅

### 1. 서버 연결 실패

```bash
# 서버가 실행 중인지 확인
curl http://localhost:8080/health

# 서버 로그 확인
cd pacs-server
cargo run --release --bin pacs_server
```

### 2. 인증 실패

```bash
# 관리자 계정 확인
export ADMIN_EMAIL="your-admin@example.com"
export ADMIN_PASSWORD="your-password"
```

### 3. 테스트 실패

```bash
# Verbose 모드로 실행하여 상세 로그 확인
pytest test_07_recist_lesion.py -v -s --tb=long

# 특정 테스트만 실행하여 디버깅
pytest test_07_recist_lesion.py::TestRecistLesionCRUD::test_01_create_target_lesion -v -s
```

## 📈 예상 결과

```
test_07_recist_lesion.py::TestRecistLesionCRUD::test_01_create_target_lesion PASSED
test_07_recist_lesion.py::TestRecistLesionCRUD::test_02_create_non_target_lesion PASSED
test_07_recist_lesion.py::TestRecistLesionCRUD::test_03_list_lesions PASSED
test_07_recist_lesion.py::TestRecistLesionCRUD::test_04_get_lesion_detail PASSED
test_07_recist_lesion.py::TestRecistLesionCRUD::test_05_update_lesion PASSED
test_07_recist_lesion.py::TestRecistLesionCRUD::test_06_delete_lesion PASSED
test_07_recist_lesion.py::TestRecistBusinessRules::test_01_max_5_target_lesions PASSED
test_07_recist_lesion.py::TestRecistBusinessRules::test_02_baseline_required_for_target PASSED
test_07_recist_lesion.py::TestRecistBusinessRules::test_03_new_lesion_no_baseline PASSED
test_07_recist_lesion.py::TestRecistBusinessRules::test_04_non_target_unlimited PASSED
test_07_recist_lesion.py::TestAnnotationLinking::test_01_link_annotation_to_lesion PASSED
test_07_recist_lesion.py::TestErrorCases::test_01_create_lesion_invalid_subject PASSED
test_07_recist_lesion.py::TestErrorCases::test_02_create_lesion_invalid_timepoint PASSED
test_07_recist_lesion.py::TestErrorCases::test_03_get_nonexistent_lesion PASSED
test_07_recist_lesion.py::TestErrorCases::test_04_update_nonexistent_lesion PASSED
test_07_recist_lesion.py::TestErrorCases::test_05_delete_nonexistent_lesion PASSED

==================== 16 passed in 5.23s ====================
```

