# DICOM Gateway 구현 계획

## 목표
- Keycloak 토큰 릴레이 기반으로 dcm4chee QIDO 호출
- evaluator 기반 RBAC 사후 필터링 적용 (Study/Series/Instance)
- 규칙 기반 조건 → QIDO 파라미터 병합(EQ/NE/RANGE/CONTAINS)
- `/studies`, `/studies/{uid}/series`, `/studies/{uid}/series/{uid}/instances` 제공

## 작업 항목
- QIDO 클라이언트: bearer 전송, URL 빌더, 인스턴스 엔드포인트 추가
- 컨트롤러: 토큰 디코딩, 프로젝트/사용자 검증, 파라미터 병합, 사후 필터
- 단위/통합 테스트: 파라미터 빌드, UID 추출, 토큰 디코드, 엔드포인트 스모크
- 문서: API 명세 갱신 및 예시 추가

## 완료 기준
- 3개 엔드포인트 정상 동작 및 테스트 통과
- 변경사항 CHANGELOG 반영 및 Swagger 문서 생성


