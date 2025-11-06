## Users API - GET /api/users/me

- 설명: 현재 요청자의 프로필 정보를 반환합니다. Authorization 헤더의 Bearer 토큰에서 `user_id`를 해석하여 DB 사용자 정보를 조회합니다.
- 메서드: GET
- 경로: `/api/users/me`
- 인증: Required (Authorization: `Bearer <access_token>`)
- 콘텐츠 타입: `application/json`

### 요청 헤더
- `Authorization: Bearer <token>`
  - 내부 JWT 또는 Keycloak 액세스 토큰 지원

### 응답: 200 OK
```json
{
  "id": 123,
  "keycloak_id": "8b1f8b0a-7e0c-4a2a-9d5f-0b9b2a3f4c5d",
  "username": "alice",
  "email": "alice@example.com",
  "created_at": "2025-10-30T12:34:56Z"
}
```

필드 설명
- `id`: 로컬 사용자 ID (정수)
- `keycloak_id`: Keycloak 사용자 UUID
- `username`: 사용자명
- `email`: 이메일
- `created_at`: 생성 시각 (UTC, ISO8601)

### 에러 응답
- 401 Unauthorized
```json
{ "error": "Invalid or missing authorization token" }
```
- 404 Not Found
```json
{ "error": "User not found" }
```

### 동작 요약
1) Authorization 헤더에서 Bearer 토큰 추출
2) 내부 JWT 검증 시도 → 성공하면 `claims.sub`를 `user_id`로 사용
3) 실패 시 Keycloak 토큰으로 간주 → payload의 `sub`(UUID) 추출 → `security_user.keycloak_id`로 사용자 조회
4) 사용자 정보를 JSON으로 반환

### 예시
- 요청
```bash
curl -s -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/api/users/me
```
- 응답(200)
```json
{
  "id": 336,
  "keycloak_id": "4f2d1b3a-0f8a-4b7c-92b1-2a8d0c9e7f11",
  "username": "alice",
  "email": "alice@example.com",
  "created_at": "2025-10-30T12:34:56Z"
}
```

### 비고
- Keycloak 토큰의 서명 검증은 게이트웨이 내 필수는 아니며, 현재 구현은 payload 기반 매핑을 수행합니다. 상위 레벨에서 검증이 필요하면 공개키 검증을 추가할 수 있습니다.

