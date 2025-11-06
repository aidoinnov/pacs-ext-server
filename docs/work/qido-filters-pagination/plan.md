# QIDO 필터/페이지네이션 작업 계획

## 목적
- DICOMweb(QIDO-RS) 호환 요청/응답 보장
- 사용자 필터 + 규칙 파라미터 병합(사용자 입력 우선)
- 페이지네이션(page/page_size ↔ limit/offset) 지원, 네이티브 limit/offset 우선

## 범위
- 컨트롤러: 쿼리 파싱/검증, 병합, 페이지네이션 변환
- QIDO 클라이언트: 파라미터 전달(변경 없음)
- 문서: API 파라미터/예시/주의사항 업데이트
- 테스트: 단위 + 모킹 통합(가벼운), DB/실연동은 게이트드

## 결정사항
- 별칭 → DICOMweb 매핑: modality/patient_id/study_date/accession_number/patient_name
- 페이지네이션: limit/offset 우선, 미지정 시 page/page_size 변환(기본 50, 최대 200)
- 병합 우선순위: 사용자 입력 > 규칙 파라미터

## 산출물
- 코드: 컨트롤러 업데이트, 매핑/검증/병합/페이지네이션
- 테스트: 모킹 통합(Studies/Series/Instances 전파 확인)
- 문서: docs/api/dicom-gateway-api.md, 기술 노트/구현 로그
