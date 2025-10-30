# RBAC Evaluator 구현 계획

목표: 프로젝트 멤버십+명시 권한+기관+규칙 기반 조건 결합으로 Study/Series/Instance 접근 허용 평가.

작업:
- 명시 권한 및 상속 검사(Study→Series→Instance)
- 기관 기반 접근 검사
- 규칙 기반 조건 조회 및 평가(EQ/NE/CONTAINS/RANGE)
- UID 기반 평가 API 추가
- 단위 테스트: 조건 연산, 날짜 범위, 우선순위/상속 충돌

완료 기준: 시나리오 테스트 통과, 컨트롤러 사후 필터 연계 완료.
