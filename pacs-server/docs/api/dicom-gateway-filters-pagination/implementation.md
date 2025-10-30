# DICOM Gateway: QIDO 필터/페이지네이션 구현 기록

## 구현 요약
- 사용자 별칭 파라미터 파싱/검증 및 DICOMweb 키로 변환
  - modality→Modality, patient_id→PatientID, study_date→StudyDate
  - accession_number→AccessionNumber, patient_name→PatientName
  - study_date 포맷 검증(YYYYMMDD | YYYYMMDD-YYYYMMDD)
- 페이지네이션
  - limit/offset 명시 시 패스스루
  - 미지정 시 page/page_size를 limit/offset으로 변환(page_size 기본 50, 최대 200)
- 규칙(AccessCondition) → QIDO 파라미터 매핑 확장
  - Modality, PatientID, StudyDate(RANGE), AccessionNumber(EQ/CONTAINS), PatientName(CONTAINS)
  - 충돌 시 사용자 입력 우선
- DICOMweb 호환성
  - 요청 파라미터 패스스루(내부 키 제외)
  - 응답 JSON 구조 변경 없이 RBAC 사후 필터만 적용

## 테스트
- 단위 테스트
  - 날짜 포맷 검증, 페이지네이션 계산, 사용자-규칙 병합(사용자 우선)
  - 태그/연산자 매핑 확장 검증
- 모킹 통합 테스트(mockito)
  - Studies/Series/Instances에 Modality/StudyDate/AccessionNumber 및 limit/offset 쿼리 전파 확인(3/3 통과)

## 코드 포인터
- 컨트롤러: `src/presentation/controllers/dicom_gateway_controller.rs`
  - `build_qido_params_from_user_query`, `merge_qido_params`, 매핑/검증/페이지네이션
- 클라이언트: `src/infrastructure/external/dcm4chee_qido_client.rs`
  - 전달받은 params를 그대로 쿼리로 전파
- 문서: `docs/api/dicom-gateway-api.md`
  - 파라미터/예시/페이지네이션 설명 업데이트
- 테스트: `tests/qido_client_pagination_filters_test.rs`
  - 모킹 통합(3 케이스)
