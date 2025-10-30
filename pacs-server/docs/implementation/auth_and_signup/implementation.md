# Keycloak/회원가입 구현 내용

- Admin 토큰: realm/{realm}/protocol/openid-connect/token, grant=client_credentials, client_id/secret 사용
- 사용자 생성: enabled=false, email_verified=true, required_actions 없음
- 이메일 인증 메일 발송 제거(관리자 승인 모델)
- signup: security_user.account_status='PENDING_APPROVAL', email_verified=true 저장
- 삭제: fetch_optional + NotFound 변환으로 404 케이스 우아 처리
- DTO: UserResponse에 account_status, email_verified 포함

문서: admin-user-approval, signup/activation, password-reset 추가
