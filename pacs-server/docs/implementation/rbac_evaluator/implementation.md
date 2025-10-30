# RBAC Evaluator 구현 내용

- 결합 로직: 프로젝트 멤버십 → 명시 권한 → 기관 → 규칙 기반 조건 순으로 검사
- 규칙 평가: Modality(00080060), PatientID(00100020), StudyDate(00080020) 지원, EQ/NE/CONTAINS/RANGE 연산
- UID 평가 API: study/series/instance UID를 DB 매핑 후 동일 로직 적용
- 유닛 테스트: 조건/날짜경계/알 수 없는 태그/CONTAINS 미일치 등 커버리지 추가

향후: 규칙 우선순위/충돌 처리 고도화, 캐싱 적용
