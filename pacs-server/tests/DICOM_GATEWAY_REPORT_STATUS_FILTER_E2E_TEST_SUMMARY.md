# DICOM Gateway Report Status 필터링 E2E 테스트 요약

## ⚠️ 중요 사전 요구사항

**이 E2E 테스트는 실제 Dcm4chee에 DICOM 데이터가 있어야 합니다.**

- DICOM Gateway는 Dcm4chee의 QIDO-RS API를 통해 실제 DICOM 데이터를 조회합니다
- 테스트에서 생성한 Study/Series는 우리 DB에만 존재하며, Dcm4chee에는 실제 DICOM 데이터가 필요합니다
- Dcm4chee에 데이터가 없으면 테스트가 실패할 수 있습니다
- 테스트는 먼저 프로젝트에 할당된 Study를 조회하고, 없으면 테스트용 Study/Series를 생성합니다
- 하지만 Dcm4chee에 실제 DICOM 데이터가 없으면 DICOM Gateway에서 조회할 수 없습니다

## 테스트 개요

DICOM Gateway Series API의 `report_status` 필터링 기능에 대한 Python E2E 시나리오 테스트입니다.

**테스트 파일**: `test_dicom_gateway_report_status_filter_e2e.py`

## 테스트 시나리오

### 시나리오 1: 단일 status 필터링 (approved)
- **목적**: 단일 status 값으로 필터링이 올바르게 동작하는지 검증
- **절차**:
  1. 사용자, 프로젝트, Study, Series 생성 (5개)
  2. 첫 번째 Series: `approved` Report 생성
  3. 두 번째 Series: `unread` Report 생성
  4. 세 번째 Series: `unapproval` Report 생성
  5. 네 번째, 다섯 번째: Report 없음
  6. `report_status=approved`로 필터링
- **검증**: 첫 번째 Series만 반환되어야 함

### 시나리오 2: 다중 status 필터링 (approved,unread)
- **목적**: 여러 status 값을 동시에 필터링하는 기능 검증
- **절차**:
  1. 사용자, 프로젝트, Study, Series 생성 (5개)
  2. 첫 번째 Series: `approved` Report 생성
  3. 두 번째 Series: `unread` Report 생성
  4. 세 번째 Series: `unapproval` Report 생성
  5. `report_status=approved,unread`로 필터링
- **검증**: 첫 번째와 두 번째 Series만 반환되어야 함

### 시나리오 3: Report가 없는 Series는 필터링에서 제외
- **목적**: Report가 없는 Series는 필터링 결과에서 제외되는지 검증
- **절차**:
  1. 사용자, 프로젝트, Study, Series 생성 (3개)
  2. 첫 번째 Series만 `approved` Report 생성
  3. 두 번째, 세 번째: Report 없음
  4. `report_status=approved`로 필터링
- **검증**: 첫 번째 Series만 반환되어야 함 (Report 없는 Series는 제외)

### 시나리오 4: Global report vs Project-dependent report 우선순위
- **목적**: Project-dependent report가 global report보다 우선되는지 검증
- **절차**:
  1. 사용자, 프로젝트, Study, Series 생성 (2개)
  2. 첫 번째 Series에 Global report 생성 (`unread`)
  3. 같은 Series에 Project-dependent report 생성 (`approved`)
  4. `report_status=approved`로 필터링 (project_id 포함)
- **검증**: Project-dependent report(`approved`)가 우선되어 Series가 반환되어야 함

### 시나리오 5: 모든 status 값 필터링
- **목적**: 모든 유효한 status 값(approved, unread, unapproval)을 포함한 필터링 검증
- **절차**:
  1. 사용자, 프로젝트, Study, Series 생성 (4개)
  2. 첫 번째 Series: `approved` Report
  3. 두 번째 Series: `unread` Report
  4. 세 번째 Series: `unapproval` Report
  5. 네 번째: Report 없음
  6. `report_status=approved,unread,unapproval`로 필터링
- **검증**: 첫 번째, 두 번째, 세 번째 Series 모두 반환되어야 함

### 시나리오 6: 대소문자 무시 필터링
- **목적**: status 값의 대소문자 구분 없이 필터링되는지 검증
- **절차**:
  1. 사용자, 프로젝트, Study, Series 생성 (2개)
  2. 첫 번째 Series에 `approved` Report 생성
  3. `report_status=APPROVED` (대문자)로 필터링
- **검증**: 대소문자와 관계없이 필터링되어야 함

## 테스트 실행 방법

### 사전 요구사항
1. 서버가 실행 중이어야 함 (`http://localhost:8080`)
2. 데이터베이스가 연결되어 있어야 함
3. **Dcm4chee에 실제 DICOM 데이터가 있어야 함** (중요!)
4. Python 3.x 및 `requests` 라이브러리 필요

### 실행 명령
```bash
# 직접 실행
python3 test_dicom_gateway_report_status_filter_e2e.py

# 또는 실행 권한 부여 후
chmod +x test_dicom_gateway_report_status_filter_e2e.py
./test_dicom_gateway_report_status_filter_e2e.py
```

### 의존성 설치
```bash
pip install requests
```

## 테스트 결과

테스트 실행 시 다음 정보가 출력됩니다:
- 각 시나리오별 테스트 진행 상황
- 성공/실패 메시지
- 최종 테스트 결과 요약

### 예상 출력
```
🚀 DICOM Gateway Report Status 필터링 E2E 테스트 시작
============================================================
🧪 Health Check
============================================================
✅ Server is healthy: {...}

🧪 시나리오 1: 단일 status 필터링 (approved)
============================================================
...
✅ Filtered series contains expected UID: ...
✅ Only one series returned (correct filtering)

...

📊 테스트 결과 요약
============================================================
총 테스트: 6
✅ 통과: 6
❌ 실패: 0
============================================================

🎉 모든 테스트가 통과했습니다!
```

## 테스트 커버리지

### 기능 커버리지
- ✅ 단일 status 필터링
- ✅ 다중 status 필터링
- ✅ Report 없는 Series 제외
- ✅ Project-dependent vs Global report 우선순위
- ✅ 모든 status 값 필터링
- ✅ 대소문자 무시

### 엔드포인트 커버리지
- ✅ `GET /api/dicom/studies/{study_uid}/series?project_id={id}&report_status={status}`
- ✅ `PUT /api/project-data/{project_id}/series/{series_id}/report` (Project-dependent)
- ✅ `PUT /api/series/{series_id}/report` (Global)

## 주의사항

1. **테스트 데이터**: 각 테스트는 독립적으로 실행되며, 테스트 데이터를 자동으로 생성합니다.
2. **타이밍**: Report 생성 후 DB 동기화를 위해 `time.sleep(0.5)`를 사용합니다.
3. **서버 상태**: 테스트 시작 전 서버 헬스 체크를 수행합니다.
4. **토큰 관리**: 각 사용자는 자동으로 생성되고 승인되며, 로그인하여 토큰을 획득합니다.

## 문제 해결

### 서버 연결 실패
- 서버가 실행 중인지 확인: `curl http://localhost:8080/health`
- 포트가 올바른지 확인 (기본값: 8080)

### 인증 실패
- Keycloak 서비스가 실행 중인지 확인
- 사용자 승인 API가 정상 동작하는지 확인

### 데이터베이스 오류
- 데이터베이스 연결 상태 확인
- 마이그레이션이 적용되었는지 확인 (`series_user_report` 테이블 존재)

### Dcm4chee 데이터 없음 (가장 흔한 문제)
- **증상**: 테스트에서 Study/Series를 생성했지만, DICOM Gateway에서 조회 시 빈 결과 반환
- **원인**: DICOM Gateway는 Dcm4chee의 QIDO-RS API를 호출하므로, 실제 DICOM 데이터가 Dcm4chee에 있어야 합니다
- **해결 방법**:
  1. Dcm4chee에 실제 DICOM 데이터를 업로드
  2. 또는 테스트를 수정하여 실제 Dcm4chee에 존재하는 Study/Series를 사용
  3. 테스트는 먼저 프로젝트에 할당된 Study를 조회하려고 시도하지만, 없으면 테스트용 데이터를 생성합니다

## 향후 개선 사항

1. **성능 테스트**: 대량 Series(100개 이상)에 대한 필터링 성능 검증
2. **에러 케이스**: 잘못된 status 값, 잘못된 project_id 등 에러 처리 검증
3. **동시성 테스트**: 여러 사용자가 동시에 Report를 생성하고 필터링하는 시나리오
4. **캐싱 테스트**: 필터링 결과 캐싱 동작 검증

