# Keycloak/회원가입 변경 계획

목표: 서비스계정 client_credentials로 admin 토큰 획득, 사용자 생성 시 enabled=false, email_verified=true, 로컬 DB 계정상태=PENDING_APPROVAL.

작업:
- KeycloakClient: client_secret 추가, token 엔드포인트/그랜트 수정
- create_user: enabled=false, email_verified=true, 검증메일 발송 제거
- signup: 로컬 DB account_status=PENDING_APPROVAL 저장
- 사용자 삭제: 존재하지 않는 사용자 graceful 처리(fetch_optional)
- 사용자 리스트 DTO: account_status, email_verified 추가

완료 기준: 관리자 승인 플로우 문서/테스트와 일치
