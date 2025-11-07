# 🔐 인증 API 참조

## 📋 개요

Annotation API를 사용하기 전에 먼저 인증을 받아야 합니다. 이 문서는 로그인, 토큰 검증, 토큰 갱신 API를 설명합니다.

---

## 🔑 1. 로그인 (Login)

### 요청

```http
POST /api/auth/login
Content-Type: application/json
```

### 요청 본문

```json
{
  "keycloak_id": "550e8400-e29b-41d4-a716-446655440000",
  "username": "john_doe",
  "email": "john@example.com"
}
```

### 파라미터

| 파라미터 | 타입 | 필수 | 설명 |
|---------|------|------|------|
| `keycloak_id` | UUID | ✅ | Keycloak 사용자 ID |
| `username` | string | ✅ | 사용자명 |
| `email` | string | ✅ | 이메일 주소 |

### 응답 (200 OK)

```json
{
  "user_id": 1,
  "keycloak_id": "550e8400-e29b-41d4-a716-446655440000",
  "username": "john_doe",
  "email": "john@example.com",
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 86400
}
```

### 응답 필드

| 필드 | 타입 | 설명 |
|------|------|------|
| `user_id` | number | 사용자 ID |
| `keycloak_id` | UUID | Keycloak 사용자 ID |
| `username` | string | 사용자명 |
| `email` | string | 이메일 주소 |
| `token` | string | JWT 액세스 토큰 |
| `token_type` | string | 토큰 타입 (항상 "Bearer") |
| `expires_in` | number | 토큰 유효 시간 (초, 기본값: 86400 = 24시간) |

### 에러 응답

#### 401 Unauthorized
```json
{
  "error": "인증 실패"
}
```

---

## ✅ 2. 토큰 검증 (Verify Token)

### 요청

```http
GET /api/auth/verify/{token}
```

### 파라미터

| 파라미터 | 타입 | 필수 | 설명 |
|---------|------|------|------|
| `token` | string | ✅ | JWT 토큰 (경로 파라미터) |

### 응답 (200 OK)

```json
{
  "user_id": 1,
  "keycloak_id": "550e8400-e29b-41d4-a716-446655440000",
  "username": "john_doe",
  "email": "john@example.com",
  "is_valid": true
}
```

### 에러 응답

#### 401 Unauthorized
```json
{
  "error": "유효하지 않은 토큰"
}
```

---

## 🔄 3. 토큰 갱신 (Refresh Token)

### 요청

```http
POST /api/auth/refresh
Content-Type: application/json
```

### 요청 본문

```json
{
  "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

### 파라미터

| 파라미터 | 타입 | 필수 | 설명 |
|---------|------|------|------|
| `refresh_token` | string | ✅ | Keycloak refresh token |

### 응답 (200 OK)

```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 86400,
  "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

### 응답 필드

| 필드 | 타입 | 설명 |
|------|------|------|
| `access_token` | string | 새로운 JWT 액세스 토큰 |
| `token_type` | string | 토큰 타입 (항상 "Bearer") |
| `expires_in` | number | 토큰 유효 시간 (초) |
| `refresh_token` | string | 새로운 refresh token |

### 에러 응답

#### 401 Unauthorized
```json
{
  "error": "유효하지 않은 refresh token"
}
```

---

## 🔐 API 요청 시 인증

모든 Annotation API 요청에는 Authorization 헤더에 JWT 토큰을 포함해야 합니다.

### 요청 예제

```http
GET /api/annotations?study_instance_uid=1.2.3.4.5&level=study,series
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

### 헤더 형식

```
Authorization: Bearer <access_token>
```

---

## 🔄 토큰 갱신 흐름

```
1. 로그인
   POST /api/auth/login
   → access_token, refresh_token 획득

2. API 요청 (access_token 사용)
   GET /api/annotations
   Authorization: Bearer <access_token>

3. access_token 만료 시
   POST /api/auth/refresh
   { "refresh_token": <refresh_token> }
   → 새로운 access_token 획득

4. 새로운 access_token으로 API 요청
   GET /api/annotations
   Authorization: Bearer <new_access_token>
```

---

## 💡 구현 팁

### TypeScript 예제

```typescript
// 1. 로그인
const loginResponse = await fetch('/api/auth/login', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    keycloak_id: 'uuid',
    username: 'john_doe',
    email: 'john@example.com'
  })
});

const { token, refresh_token } = await loginResponse.json();

// 2. API 요청
const apiResponse = await fetch('/api/annotations', {
  headers: {
    'Authorization': `Bearer ${token}`
  }
});

// 3. 토큰 갱신 (필요시)
const refreshResponse = await fetch('/api/auth/refresh', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ refresh_token })
});

const { access_token } = await refreshResponse.json();
```

---

## ⏱️ 토큰 유효 시간

- **Access Token**: 24시간 (86400초)
- **Refresh Token**: Keycloak 설정에 따름

---

## 🚀 다음 단계

1. 로그인하여 access_token 획득
2. Annotation API 요청 시 Authorization 헤더에 token 포함
3. Token 만료 시 refresh token으로 새로운 token 획득

