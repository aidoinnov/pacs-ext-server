# 작업 계획: 토큰 추출 유틸 공용화 + /api/users/me 엔드포인트

## 배경/목표
- 컨트롤러마다 중복되던 토큰 → 사용자 식별(`user_id`) 추출 로직을 공용 유틸로 분리
- 프론트엔드/테스트에서 자기 자신 프로필 확인용 `/api/users/me` 제공

## 범위
- `token_extractor` 모듈 추가: 내부 JWT → `user_id`, 실패 시 Keycloak `sub`(UUID) → 로컬 사용자 매핑
- DICOM Gateway 컨트롤러에서 공용 유틸 사용하도록 리팩터
- `GET /api/users/me` 추가: 토큰 기반 현재 사용자 프로필 반환
- 통합 테스트(ignored): 내부 JWT로 `/api/users/me` 검증, DB URL은 `DATABASE_URL`→`APP_DATABASE_URL` 폴백
- 간단 문서/CHANGELOG 업데이트

## 산출물
- 코드: `src/infrastructure/auth/token_extractor.rs`, `user_controller.rs` 라우트 추가
- 테스트: `tests/users_me_test.rs`
- 문서: 본 계획서, 작업로그, 기술문서
- 변경이력: CHANGELOG 갱신

## 완료 기준
- 빌드/테스트 통과 (`users_me_test`는 ignored, 필요 시 DB 세팅 후 수동 실행)
- 게이트웨이 컨트롤러가 공용 유틸을 사용
- Swagger/OpenAPI에 `/api/users/me` 노출(간단 주석 포함)

