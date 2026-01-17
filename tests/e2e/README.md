# PACS Server E2E & Performance Tests

Python 기반의 End-to-End 테스트 및 성능 테스트 스위트입니다.

## 📋 목차

- [설치](#설치)
- [설정](#설정)
- [테스트 실행](#테스트-실행)
- [테스트 시나리오](#테스트-시나리오)
- [성능 테스트](#성능-테스트)
- [리포트](#리포트)

## 🚀 설치

### 1. Python 가상환경 생성

```bash
cd tests/e2e
python3 -m venv venv
source venv/bin/activate  # Windows: venv\Scripts\activate
```

### 2. 의존성 설치

```bash
pip install -r requirements.txt
```

## ⚙️ 설정

### 환경 변수 설정

`.env` 파일을 생성하고 다음 내용을 설정하세요:

```bash
# 서버 설정
TEST_BASE_URL=http://localhost:8080

# 테스트 계정
TEST_ADMIN_EMAIL=admin@example.com
TEST_ADMIN_PASSWORD=admin123
TEST_USER_EMAIL=test@example.com
TEST_USER_PASSWORD=test123

# 타임아웃 (초)
TEST_TIMEOUT=30

# 성능 테스트 설정
PERF_CONCURRENT_USERS=10
PERF_REQUESTS_PER_USER=100
PERF_RAMP_UP_TIME=5
PERF_TEST_DURATION=60
```

## 🧪 테스트 실행

### 전체 테스트 실행 (쉘 스크립트)

```bash
./run_all_tests.sh
```

### 개별 테스트 실행 (쉘 스크립트) ⭐ 추천

각 테스트를 쉽게 실행할 수 있는 스크립트를 제공합니다:

```bash
# 🎬 데모 테스트 (빠른 확인용)
./run_demo.sh

# 🔐 인증 테스트
./run_auth.sh

# 📸 스냅샷 URL 테스트
./run_snapshot.sh

# ⚡ 성능 테스트 (동시성)
./run_performance.sh
```

### pytest로 직접 실행

```bash
# 인증 테스트
pytest test_01_auth.py -v -s

# 프로젝트 테스트
pytest test_02_project.py -v -s

# 어노테이션 테스트
pytest test_03_annotation.py -v -s

# 스냅샷 URL 테스트
python test_snapshot_url.py

# 성능 테스트 - 동시 요청
pytest test_performance_01_concurrent.py -v -s

# 성능 테스트 - 대량 데이터
pytest test_performance_02_bulk_data.py -v -s
```

### 특정 테스트만 실행

```bash
# 특정 클래스의 테스트만 실행
pytest test_01_auth.py::TestAuthentication -v -s

# 특정 테스트 메서드만 실행
pytest test_01_auth.py::TestAuthentication::test_01_login_success -v -s
```

## 📝 테스트 시나리오

### 1. 인증 및 사용자 관리 (test_01_auth.py)

- ✅ 로그인 성공
- ✅ 잘못된 인증 정보로 로그인 실패
- ✅ 현재 사용자 정보 조회
- ✅ 인증 없이 접근 시 실패
- ✅ 잘못된 토큰으로 접근 시 실패

### 2. 프로젝트 관리 (test_02_project.py)

- ✅ 프로젝트 생성
- ✅ 프로젝트 목록 조회
- ✅ 프로젝트 상세 조회
- ✅ 프로젝트 수정

### 3. 어노테이션 CRUD (test_03_annotation.py)

- ✅ 어노테이션 생성
- ✅ Study UID로 어노테이션 조회
- ✅ 프로젝트로 어노테이션 조회
- ✅ 어노테이션 수정
- ✅ 어노테이션 삭제

### 4. 스냅샷 이미지 (test_04_snapshot.py)

- ✅ 업로드용 Signed URL 생성
- ✅ 스냅샷 이미지 업로드
- ✅ 다운로드용 Signed URL 생성
- ✅ 대량 다운로드 URL 생성

## 📊 성능 테스트

### 1. 동시 요청 처리 (test_performance_01_concurrent.py)

- 동시 로그인 요청 처리
- 동시 어노테이션 조회 요청
- 동시 프로젝트 조회 요청

### 2. 대량 데이터 조회 (test_performance_02_bulk_data.py)

- 프로젝트의 모든 어노테이션 조회
- Study UID로 어노테이션 조회
- 페이지네이션 성능

## 📈 리포트

테스트 실행 후 다음 리포트가 생성됩니다:

- `performance_report_YYYYMMDD_HHMMSS.txt` - 텍스트 리포트
- `performance_metrics_YYYYMMDD_HHMMSS.csv` - CSV 데이터
- `performance_charts_YYYYMMDD_HHMMSS.png` - 성능 그래프
- `performance_report_YYYYMMDD_HHMMSS.html` - HTML 리포트

### 리포트 수동 생성

```bash
python generate_report.py metrics.json
```

## 🔧 트러블슈팅

### 연결 오류

서버가 실행 중인지 확인하세요:

```bash
curl http://localhost:8080/health
```

### 인증 오류

테스트 계정이 데이터베이스에 존재하는지 확인하세요.

### 타임아웃

`TEST_TIMEOUT` 환경 변수를 늘려보세요.

## 📚 추가 정보

- 테스트는 독립적으로 실행되며, 각 테스트는 자체 데이터를 생성하고 정리합니다.
- 성능 테스트는 실제 서버 부하를 발생시키므로 프로덕션 환경에서는 주의하세요.
- 모든 테스트는 pytest 프레임워크를 사용합니다.

