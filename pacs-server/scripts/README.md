# PACS Server Scripts

이 디렉토리는 개발 및 디버깅을 위한 유틸리티 스크립트들을 포함합니다.

## 📂 디렉토리 구조

### 🧪 `test/` (23개 파일)
개발/디버깅용 API 테스트 스크립트
- `test_*.py`: 다양한 API 엔드포인트 테스트 (개발용)
- `get_*.py`: 인증 토큰 획득 및 테스트 데이터 생성

**주요 파일:**
- `test_mask_*.py`: Mask API 테스트
- `test_performance_*.py`: 성능 테스트
- `test_me_*.py`: 개인 데이터 API 테스트
- `test_sync_only.py`: Sync 모드 테스트

**참고:** E2E 테스트는 `/pacs-server/e2e` 디렉토리에 있습니다.

### 🔍 `check/` (16개 파일)
데이터베이스 및 시스템 상태 확인 스크립트
- `check_*.py`: DB 데이터, API 응답, 시스템 상태 확인

**주요 파일:**
- `check_db_*.py`: 데이터베이스 데이터 확인
- `check_series_*.py`: Series 데이터 확인
- `check_project_data*.py`: Project 데이터 구조 확인
- `comprehensive_*.py`: 종합 시스템 검증

### 🐛 `debug/` (3개 파일)
디버깅 및 분석 스크립트
- `debug_*.py`: 특정 이슈 디버깅
- `analyze_*.py`: 데이터 분석

**주요 파일:**
- `debug_series_logic.py`: Series 로직 디버깅
- `debug_report_filter.py`: Report 필터 디버깅
- `analyze_series_issue.py`: Series 이슈 분석

### ⚡ `bench/` (2개 파일)
성능 벤치마크 스크립트
- `bench_*.py`: API 성능 측정

**주요 파일:**
- `bench_series_api.py`: Series API 성능 벤치마크
- `bench_series_performance.py`: Series 성능 측정

### 📋 `assign/` (3개 파일)
데이터 할당 및 마이그레이션 스크립트
- `assign_*.py`: Project에 데이터 할당

**주요 파일:**
- `assign_data_to_project.py`: 단일 데이터 할당
- `assign_all_data_to_project.py`: 전체 데이터 할당
- `assign_all_data_from_db.py`: DB에서 데이터 읽어서 할당

### ✅ `verify/` (3개 파일)
검증 스크립트
- `verify_*.py`: 데이터 및 API 검증

**주요 파일:**
- `verify_db_direct.py`: DB 직접 검증
- `verify_db_query.py`: DB 쿼리 검증
- `verify_via_api.py`: API를 통한 검증

## 🚀 사용법

### 테스트 실행
```bash
cd pacs-server/scripts/test
python test_dicom_gateway_study_series_e2e.py
```

### 데이터 확인
```bash
cd pacs-server/scripts/check
python check_project_data.py
```

### 성능 벤치마크
```bash
cd pacs-server/scripts/bench
python bench_series_api.py
```

## 📝 참고사항

- 대부분의 스크립트는 개발/디버깅 목적으로 작성되었습니다
- 프로덕션 환경에서는 사용하지 마세요
- **정식 E2E 테스트**:
  - **PACS Server**: `/pacs-server/e2e` 디렉토리 사용
  - **전체 시스템**: `/tests/e2e` 디렉토리 사용

## 🔗 관련 문서

- [PACS Server E2E 테스트](/pacs-server/e2e/README.md)
- [전체 시스템 E2E 테스트](/tests/e2e/README.md)
- [개발 가이드](/docs/DEVELOPMENT.md)

