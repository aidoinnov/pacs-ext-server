# DICOM Gateway E2E Test Guide

## 개요

DICOM Gateway의 RBAC 필터링 기능을 검증하는 E2E 테스트 가이드입니다.

## 테스트 전 준비사항

### 1. 서버 실행
```bash
cd pacs-server
RUST_LOG=info cargo run --bin pacs_server
```

### 2. 환경 변수 설정

Keycloak 인증 정보를 설정합니다:

```bash
export KEYCLOAK_BASE_URL=https://keycloak.pacs.ai-do.kr
export KEYCLOAK_REALM=dcm4che
export KEYCLOAK_CLIENT_ID=pacs-extension-server
export KEYCLOAK_CLIENT_SECRET=85TSWxK8ruF750z0Qzh0tQZ8xH5h3y99
export KEYCLOAK_USERNAME=<실제_사용자명>
export KEYCLOAK_PASSWORD=<실제_비밀번호>
export GATEWAY_BASE_URL=http://127.0.0.1:8080
export PROJECT_ID=1
```

## 테스트 실행

### 방법 1: Python 스크립트 (권장)

```bash
cd pacs-server
python3 scripts/e2e_gateway_test.py
```

**예상 출력:**
```
======================================================================
DICOM Gateway E2E Test - RBAC Filtering Verification
======================================================================

[1/4] Getting access token from Keycloak...
✅ Token obtained successfully (length: XXX)

[2/4] Testing GET /api/dicom/studies?project_id=1...
✅ Studies endpoint OK: X study(ies) returned

[3/4] Testing GET /api/dicom/studies/{study_uid}/series...
✅ Series endpoint OK: X series returned

[4/4] Testing GET /api/dicom/studies/{study_uid}/series/{series_uid}/instances...
✅ Instances endpoint OK: X instance(s) returned

✅ E2E Test Completed Successfully!
```

### 방법 2: Rust E2E 바이너리

```bash
cd pacs-server
cargo run --bin dicom_gw_e2e
```

### 방법 3: Bash 스크립트 (기본 체크)

```bash
cd pacs-server
./scripts/test_gateway_rbac.sh
```

## 테스트 검증 항목

### 1. 프로젝트별 필터링
- `project_id` 파라미터로 특정 프로젝트의 데이터만 반환되는지 확인
- 다른 프로젝트의 데이터는 필터링되는지 확인

### 2. RBAC 필터링
- 사용자가 접근 권한이 없는 Study/Series/Instance는 필터링되는지 확인
- Evaluator가 정상적으로 동작하여 권한이 있는 항목만 반환되는지 확인

### 3. 규칙 기반 필터링
- `AccessCondition`에 정의된 규칙(Modality, PatientID 등)이 QIDO 파라미터로 변환되어 적용되는지 확인

### 4. 계층적 필터링
- Study → Series → Instance 순서로 권한이 상속되는지 확인
- 상위 레벨 권한이 있으면 하위 레벨도 접근 가능한지 확인

## 문제 해결

### 서버가 응답하지 않음
```bash
# 서버 상태 확인
curl http://127.0.0.1:8080/health

# 게이트웨이 ping 확인
curl http://127.0.0.1:8080/api/dicom/ping
```

### 인증 오류 (401/403)
- Keycloak 인증 정보가 올바른지 확인
- 사용자가 해당 프로젝트에 속해 있는지 확인
- JWT 토큰이 유효한지 확인

### Dcm4chee 연결 오류 (502)
- `APP_DCM4CHEE__BASE_URL` 환경 변수가 올바른지 확인
- Dcm4chee 서버가 실행 중인지 확인
- Bearer 토큰이 Dcm4chee에 전달되고 있는지 확인

### 빈 응답 반환
- RBAC 필터링이 너무 강하게 적용되어 모든 데이터가 필터링되었을 수 있음
- 사용자에게 접근 권한이 있는 데이터가 실제로 존재하는지 확인

## 추가 테스트 시나리오

### 시나리오 1: 프로젝트별 필터링 테스트
```bash
# 프로젝트 1 조회
curl -H "Authorization: Bearer $TOKEN" \
  "http://127.0.0.1:8080/api/dicom/studies?project_id=1&limit=5"

# 프로젝트 2 조회
curl -H "Authorization: Bearer $TOKEN" \
  "http://127.0.0.1:8080/api/dicom/studies?project_id=2&limit=5"
```

### 시나리오 2: 규칙 기반 필터링 테스트
```bash
# Modality=CT 조건 적용
curl -H "Authorization: Bearer $TOKEN" \
  "http://127.0.0.1:8080/api/dicom/studies?project_id=1&Modality=CT"
```

### 시나리오 3: 계층적 접근 테스트
```bash
# Study 조회
STUDY_UID=$(curl -s -H "Authorization: Bearer $TOKEN" \
  "http://127.0.0.1:8080/api/dicom/studies?project_id=1&limit=1" \
  | jq -r '.[0]["0020000D"].Value[0]')

# 해당 Study의 Series 조회 (권한 상속 확인)
curl -H "Authorization: Bearer $TOKEN" \
  "http://127.0.0.1:8080/api/dicom/studies/${STUDY_UID}/series?project_id=1"
```

## 참고

- 게이트웨이 엔드포인트 문서: `docs/api/dicom-gateway-api.md`
- RBAC Evaluator 구현: `src/infrastructure/services/dicom_rbac_evaluator_impl.rs`
- 게이트웨이 컨트롤러: `src/presentation/controllers/dicom_gateway_controller.rs`

