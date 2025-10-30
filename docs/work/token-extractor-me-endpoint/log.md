# 작업 로그: 토큰 추출 유틸 공용화 + /api/users/me

## 진행 내역
- token extractor 유틸 추가: 내부 JWT 우선, 실패 시 Keycloak `sub`→로컬 사용자 매핑
- DICOM Gateway 컨트롤러에서 공용 유틸 사용하도록 수정
- Unauthorized 시 원인 추적 로그 추가(필요 로그 후 기본 tracing 초기화는 제거)
- `/api/users/me` 추가: 토큰 기반 현재 사용자 프로필 반환
- 통합 테스트(ignored) 추가: 내부 JWT로 `/api/users/me` 검증, DB URL 폴백 지원
- QIDO API 문서(`docs/api/qido.md`) 작성 완료(선행 작업)

## 커밋/PR
- refactor(auth): extract user_id-from-token logic to shared token_extractor and reuse in QIDO controller
- feat(users): add GET /api/users/me to return current user profile from token
- test(users): add ignored integration test for /api/users/me; DB URL fallback
- docs(api): add DICOMweb QIDO API spec for frontend

## 검증
- 빌드 통과
- `users_me_test.rs` 수동 실행: DB URL 제공 시 통과 확인

## 남은 것
- 실 Keycloak 토큰 서명 검증 필요 시, 공개키 검증기 연동 고려
- `/api/users/me` 스웨거 스키마 필드 보강(예시 등)
