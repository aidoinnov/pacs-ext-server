# 기술 문서: Token Extractor 및 /api/users/me

## Token Extractor 설계
- 모듈: `src/infrastructure/auth/token_extractor.rs`
- 흐름
  1) Authorization 헤더 추출 → Bearer 토큰 파싱
  2) 내부 JWT 경로: `JwtService::validate_token` → `claims.user_id()`
  3) 실패 시 Keycloak 경로: JWT payload base64url 디코드 → `sub` 추출(UUID)
     → `security_user.keycloak_id`로 사용자 조회 → `user.id`
- 보안 고려
  - Keycloak 경로는 서명 검증을 수행하지 않음(게이트웨이 내用途: 로컬 사용자 매핑)
  - 서명 검증 필요 시 Keycloak 공개키 기반 검증기 연동 가능

## /api/users/me
- 위치: `src/presentation/controllers/user_controller.rs`
- 동작: 토큰으로 `user_id` 해석 → 사용자 조회 → 프로필 JSON 반환
- 오류: 토큰 부재/무효(401), 사용자 없음(404)

## 테스트
- 파일: `tests/users_me_test.rs`
- 특징: DB URL 환경변수 폴백(`DATABASE_URL` → `APP_DATABASE_URL`)
- 시나리오: 테스트 사용자 insert → 내부 JWT 발급 → `/api/users/me` 호출 → 200/ID 일치 확인

## 기타
- DICOM Gateway 컨트롤러는 공용 extractor 사용으로 중복 제거
- Unauthorized 분기에 원인 로깅 추가(운영 시 문제 분석 용이)
