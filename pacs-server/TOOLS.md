# 🛠️ PACS Server 도구 스크립트 가이드

## 📋 목차
- [개요](#개요)
- [디렉토리 구조](#디렉토리-구조)
- [스크립트 카테고리](#스크립트-카테고리)
- [사용 예시](#사용-예시)

---

## 개요

`pacs-server/scripts/` 디렉토리에는 개발 및 디버깅을 위한 유틸리티 스크립트들이 포함되어 있습니다.

**⚠️ 주의사항:**
- 대부분의 스크립트는 개발/디버깅 목적으로 작성되었습니다
- 프로덕션 환경에서는 사용하지 마세요
- **정식 E2E 테스트**는 다음 디렉토리를 사용하세요:
  - PACS Server: `/pacs-server/e2e`
  - 전체 시스템: `/tests/e2e`

---

## 디렉토리 구조

```
pacs-server/scripts/
├── README.md              # 스크립트 사용 가이드
├── test/                  # 개발용 테스트 (23개)
├── check/                 # 상태 확인 (16개)
├── debug/                 # 디버깅 (3개)
├── bench/                 # 벤치마크 (2개)
├── assign/                # 데이터 할당 (3개)
└── verify/                # 검증 (3개)
```

---

## 스크립트 카테고리

### 🧪 test/ (23개 파일)
개발/디버깅용 API 테스트 스크립트

**주요 파일:**
- `test_mask_api.py` - Mask API 테스트
- `test_mask_group_api.py` - Mask Group API 테스트
- `test_mask_upload_workflow.py` - Mask 업로드 워크플로우 테스트
- `test_performance_quick.py` - 빠른 성능 테스트
- `test_me_series.py` - 개인 Series 데이터 테스트
- `get_token.py` - 인증 토큰 획득
- `get_test_token_and_assign.py` - 토큰 획득 및 데이터 할당

**사용법:**
```bash
cd pacs-server/scripts/test
python3 test_mask_api.py
```

---

### 🔍 check/ (16개 파일)
데이터베이스 및 시스템 상태 확인 스크립트

**주요 파일:**
- `check_db_with_env.py` - 환경 변수로 DB 연결 확인
- `check_project_data.py` - Project 데이터 확인
- `check_project_data_structure.py` - Project 데이터 구조 확인
- `check_series_count.py` - Series 개수 확인
- `check_series_api.py` - Series API 응답 확인
- `check_redis_connection.py` - Redis 연결 확인
- `comprehensive_check.py` - 종합 시스템 검증
- `comprehensive_review.py` - 종합 리뷰

**사용법:**
```bash
cd pacs-server/scripts/check
python3 check_project_data.py
```

---

### 🐛 debug/ (3개 파일)
디버깅 및 분석 스크립트

**주요 파일:**
- `debug_series_logic.py` - Series 로직 디버깅
- `debug_report_filter.py` - Report 필터 디버깅
- `analyze_series_issue.py` - Series 이슈 분석

**사용법:**
```bash
cd pacs-server/scripts/debug
python3 debug_series_logic.py
```

---

### ⚡ bench/ (2개 파일)
성능 벤치마크 스크립트

**주요 파일:**
- `bench_series_api.py` - Series API 성능 벤치마크
- `bench_series_performance.py` - Series 성능 측정

**사용법:**
```bash
cd pacs-server/scripts/bench
python3 bench_series_api.py
```

**출력 예시:**
```
========================================
Series API Performance Benchmark
========================================
Total Requests: 100
Success: 100
Errors: 0
Response Times (ms):
  Min: 45.2
  Avg: 67.8
  Median: 62.3
  P95: 89.1
  Max: 102.5
```

---

### 📋 assign/ (3개 파일)
데이터 할당 및 마이그레이션 스크립트

**주요 파일:**
- `assign_data_to_project.py` - 단일 데이터 할당
- `assign_all_data_to_project.py` - 전체 데이터 할당
- `assign_all_data_from_db.py` - DB에서 데이터 읽어서 할당

**사용법:**
```bash
cd pacs-server/scripts/assign
python3 assign_data_to_project.py --project-id 1 --study-uid "1.2.840..."
```

---

### ✅ verify/ (3개 파일)
검증 스크립트

**주요 파일:**
- `verify_db_direct.py` - DB 직접 검증
- `verify_db_query.py` - DB 쿼리 검증
- `verify_via_api.py` - API를 통한 검증

**사용법:**
```bash
cd pacs-server/scripts/verify
python3 verify_db_direct.py
```

---

## 사용 예시

### 1. 토큰 획득 및 API 테스트
```bash
# 1. 토큰 획득
cd pacs-server/scripts/test
python3 get_token.py

# 2. Mask API 테스트
python3 test_mask_api.py
```

### 2. 데이터베이스 상태 확인
```bash
cd pacs-server/scripts/check
python3 check_project_data.py
```

### 3. 성능 벤치마크
```bash
cd pacs-server/scripts/bench
python3 bench_series_api.py
```

### 4. 데이터 할당
```bash
cd pacs-server/scripts/assign
python3 assign_data_to_project.py --project-id 1 --study-uid "1.2.840.113619..."
```

### 5. 시스템 종합 검증
```bash
cd pacs-server/scripts/check
python3 comprehensive_check.py
```

---

## 환경 설정

대부분의 스크립트는 환경 변수를 사용합니다:

```bash
# .env 파일 또는 환경 변수 설정
export DATABASE_URL="postgresql://user:password@localhost/pacs"
export BASE_URL="http://localhost:8080"
export TEST_USER="reader1_user"
export TEST_PASSWORD="Qlalfqjsgh1!"
```

---

## 관련 문서

- [실행 가이드](./RUN.md)
- [테스트 가이드](./TESTING.md)
- [스크립트 README](./scripts/README.md)
- [E2E 테스트](./e2e/README.md)

