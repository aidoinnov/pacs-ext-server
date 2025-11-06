# 기술 요약: QIDO 필터/페이지네이션

## 매핑
- 별칭 → DICOMweb
  - modality → Modality (00080060)
  - patient_id → PatientID (00100020)
  - study_date → StudyDate (00080020)
  - accession_number → AccessionNumber (00080050)
  - patient_name → PatientName (00100010)

## 연산자
- EQ/EQUALS/== → key=value
- CONTAINS → key=substring(업스트림 부분일치 가정)
- RANGE/BETWEEN(StudyDate) → YYYYMMDD-YYYYMMDD
- NE 등 비지원 연산자: QIDO 미전달, 사후 필터로 위임

## 병합/페이지네이션
- 병합 우선순위: 사용자 > 규칙
- 페이지네이션 우선순위: limit/offset > page/page_size
- 변환식: offset=(page-1)*page_size, limit=page_size(기본 50, 최대 200)

## 호환성
- DICOMweb 네이티브 파라미터 패스스루(내부 키 제외)
- 응답 JSON 구조 변경 없음(사후 필터만 적용)
