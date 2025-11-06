# DICOM Gateway 구현 내용

- QIDO 클라이언트: bearer 토큰 릴레이, URL 빌더 개선, instances 지원
- 컨트롤러: 토큰 디코드→사용자/프로젝트 검증, 규칙→QIDO 파라미터 병합, dcm4chee 호출, evaluator 사후 필터
- 유닛 테스트: 파라미터 빌드, 토큰 디코드, UID 추출
- 주의: audience 401/404 대응, 외부 서비스 에러 래핑, 캐싱은 미적용

향후: 규칙 캐싱/인덱스, E2E 시나리오 확장, 시드 데이터 다양화
