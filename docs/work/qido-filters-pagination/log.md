# 작업 기록: QIDO 필터/페이지네이션

## 완료 요약
- 컨트롤러: 사용자 별칭 파싱/검증, DICOMweb 키로 변환, 병합(사용자 우선)
- 페이지네이션: limit/offset 패스스루, page/page_size 변환(기본 50, 최대 200)
- 규칙 매핑 확장: Modality/PatientID/StudyDate/AccessionNumber/PatientName
- 문서: `docs/api/dicom-gateway-api.md` 갱신(파라미터/예시/주의)
- 테스트: 모킹 통합(Studies/Series/Instances) 3건 통과, 단위테스트 보강

## 커밋
- feature/dicomweb-rbac-gateway 브랜치에 다수 커밋(컨트롤러/문서/테스트)

## 다음 과제
- 태그/연산자 커버리지 추가(필요 시)
- 실연동 게이트 테스트 정리(Keycloak/Dcm4chee 환경 변수 기반)
