# 🚀 Quick Start Guide

PACS Server E2E 테스트를 빠르게 시작하는 가이드입니다.

## 1️⃣ 사전 준비

### 필수 요구사항

- Python 3.8 이상
- PACS Server가 실행 중이어야 함
- 테스트 계정 (관리자 권한)

### 서버 실행 확인

```bash
# 서버가 실행 중인지 확인
curl http://localhost:8080/health

# 또는
curl http://localhost:8080/api/health
```

## 2️⃣ 설치

```bash
# 1. 테스트 디렉토리로 이동
cd tests/e2e

# 2. 가상환경 생성
python3 -m venv venv

# 3. 가상환경 활성화
source venv/bin/activate  # macOS/Linux
# 또는
venv\Scripts\activate  # Windows

# 4. 의존성 설치
pip install -r requirements.txt
```

## 3️⃣ 설정

```bash
# .env 파일 생성
cp .env.example .env

# .env 파일 편집
nano .env  # 또는 원하는 에디터 사용
```

`.env` 파일 예시:

```bash
TEST_BASE_URL=http://localhost:8080
TEST_ADMIN_EMAIL=admin@example.com
TEST_ADMIN_PASSWORD=your_password_here
TEST_USER_EMAIL=test@example.com
TEST_USER_PASSWORD=test_password_here
```

## 4️⃣ 데모 테스트 실행

설정이 올바른지 확인하기 위해 데모 테스트를 실행합니다:

```bash
python demo_test.py
```

성공하면 다음과 같은 출력을 볼 수 있습니다:

```
================================================================================
PACS Server E2E Test - Demo
================================================================================

📋 Configuration:
  Base URL: http://localhost:8080
  Admin Email: admin@example.com
  Timeout: 30s

🏥 Testing health check...
  ✓ Server is healthy

🔐 Testing login...
  ✓ Login successful
  User ID: 1
  Email: admin@example.com

👤 Testing get current user...
  ✓ Got user info
  Name: Admin User
  Email: admin@example.com

📁 Testing list projects...
  ✓ Found 5 projects

📝 Testing query annotations...
  ✓ Found 3 annotations (limited to 5)

================================================================================
✅ Demo test completed successfully!
================================================================================
```

## 5️⃣ 전체 테스트 실행

### 방법 1: 전체 테스트 스위트 실행

```bash
python run_all_tests.py
```

### 방법 2: 개별 테스트 실행

```bash
# 인증 테스트
pytest test_01_auth.py -v -s

# 프로젝트 테스트
pytest test_02_project.py -v -s

# 어노테이션 테스트
pytest test_03_annotation.py -v -s

# 스냅샷 테스트
pytest test_04_snapshot.py -v -s
```

### 방법 3: 성능 테스트만 실행

```bash
# 동시 요청 처리 테스트
pytest test_performance_01_concurrent.py -v -s

# 대량 데이터 조회 테스트
pytest test_performance_02_bulk_data.py -v -s
```

## 6️⃣ 결과 확인

테스트 실행 후 `test_results_YYYYMMDD_HHMMSS/` 디렉토리에 결과가 저장됩니다:

```
test_results_20260115_143022/
├── performance_report_20260115_143022.txt   # 텍스트 리포트
├── performance_metrics_20260115_143022.csv  # CSV 데이터
├── performance_charts_20260115_143022.png   # 성능 그래프
├── performance_report_20260115_143022.html  # HTML 리포트
└── test_summary.json                        # 테스트 요약
```

HTML 리포트를 브라우저로 열어서 확인:

```bash
open test_results_*/performance_report_*.html  # macOS
# 또는
xdg-open test_results_*/performance_report_*.html  # Linux
# 또는
start test_results_*/performance_report_*.html  # Windows
```

## 🔧 문제 해결

### 연결 오류

```
Error: Connection refused
```

**해결방법**: 서버가 실행 중인지 확인하세요.

```bash
# 서버 상태 확인
curl http://localhost:8080/health
```

### 인증 오류

```
Error: 401 Unauthorized
```

**해결방법**: `.env` 파일의 이메일과 비밀번호를 확인하세요.

### 모듈 없음 오류

```
ModuleNotFoundError: No module named 'requests'
```

**해결방법**: 의존성을 다시 설치하세요.

```bash
pip install -r requirements.txt
```

## 📊 성능 테스트 커스터마이징

`.env` 파일에서 성능 테스트 파라미터를 조정할 수 있습니다:

```bash
# 동시 사용자 수
PERF_CONCURRENT_USERS=20

# 사용자당 요청 수
PERF_REQUESTS_PER_USER=200

# 램프업 시간 (초)
PERF_RAMP_UP_TIME=10

# 테스트 지속 시간 (초)
PERF_TEST_DURATION=120
```

## 🎯 다음 단계

1. **CI/CD 통합**: GitHub Actions, Jenkins 등에 통합
2. **커스텀 테스트 추가**: 프로젝트 요구사항에 맞는 테스트 추가
3. **성능 벤치마크**: 정기적인 성능 테스트로 회귀 방지
4. **모니터링**: 테스트 결과를 모니터링 시스템에 연동

## 📚 추가 문서

- [README.md](README.md) - 전체 문서
- [테스트 시나리오 상세](README.md#테스트-시나리오)
- [성능 테스트 가이드](README.md#성능-테스트)

