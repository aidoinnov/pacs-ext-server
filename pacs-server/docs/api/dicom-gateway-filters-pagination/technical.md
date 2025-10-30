# DICOM Gateway: 기술 노트 (필터/페이지네이션)

## 키 매핑
- 별칭 → DICOMweb
  - modality → Modality (00080060)
  - patient_id → PatientID (00100020)
  - study_date → StudyDate (00080020)
  - accession_number → AccessionNumber (00080050)
  - patient_name → PatientName (00100010)

## 연산자 매핑(규칙)
- EQ/EQUALS/== → key=value
- CONTAINS → key=substring (업스트림 부분일치 동작 가정)
- RANGE/BETWEEN(StudyDate) → StudyDate=YYYYMMDD-YYYYMMDD
- NE 등 비지원 연산자는 QIDO 미전달, 사후 필터로 위임

## 병합 우선순위
- 사용자 입력 > 규칙 파라미터 > 기본값
- 동일 키 충돌 시 사용자 값으로 덮어씀

## 페이지네이션
- 입력 우선순위: `limit/offset` > `page/page_size`
- 변환: `offset=(page-1)*page_size`, `limit=page_size`
- page_size 기본 50, 최대 200 (상한)

## 호환성 원칙
- DICOMweb 네이티브 파라미터는 패스스루(내부 키 제외)
- 응답은 QIDO JSON을 구조 변경 없이 반환(사후 필터만 적용)
