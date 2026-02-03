# 사용자 관리 API (클라이언트 가이드)

클라이언트 개발자를 위한 회원가입, 회원수정, 회원승인 API 안내입니다.

**Base URL**: `http://localhost:8080/api`

---

## 1. 요약

| 구분 | API | Endpoint | 인증 | 비고 |
|------|-----|----------|------|------|
| 회원가입 | 회원가입 | `POST /auth/signup` | 불필요 | 공개 API |
| 회원수정 | 프로필 수정 | `PUT /users/{user_id}` | JWT 필요 | 본인 또는 관리자 |
| **회원승인** | **관리자 승인** | `POST /auth/admin/users/approve` | **관리자 JWT 필요** | **로그인 가능하게 하는 필수 단계** |
| 계정 삭제 | 계정 삭제 | `DELETE /users/{user_id}` | JWT 필요 | 본인 또는 관리자 |

### 회원승인이 필요한 이유

- 회원가입 시 Keycloak에 사용자가 **비활성화(enabled=false)** 상태로 생성됩니다.
- 관리자가 승인해야 Keycloak에서 **활성화(enabled=true)** 되어 로그인이 가능합니다.
- 따라서 **회원승인 API는 필수**이며, 승인 전에는 로그인이 불가능합니다.

### 제거된 API

| API | 비고 |
|-----|------|
| `POST /auth/verify-email` (이메일 인증) | 2026-02 제거 — 회원가입 직후 `PENDING_APPROVAL`로 진행 |

---

## 2. 회원가입

새 사용자를 등록합니다. 이메일 인증 없이 바로 `PENDING_APPROVAL` 상태가 됩니다.

### Endpoint

```
POST /api/auth/signup
```

### 인증

불필요 (공개 API)

### Request

```json
{
  "username": "john_doe",
  "email": "john@example.com",
  "password": "SecurePassword123!",
  "full_name": "John Doe",
  "organization": "Seoul National University Hospital",
  "department": "Radiology Department",
  "phone": "010-1234-5678"
}
```

| 필드 | 타입 | 필수 | 설명 |
|------|------|------|------|
| username | string | ✅ | 사용자명 (3자 이상) |
| email | string | ✅ | 이메일 주소 |
| password | string | ✅ | 비밀번호 (8자 이상, 대문자·소문자·숫자 각 1개 이상) |
| full_name | string | - | 실명 |
| organization | string | - | 소속 기관 |
| department | string | - | 소속 부서 |
| phone | string | - | 연락처 |

### Response (201 Created)

```json
{
  "user_id": 123,
  "username": "john_doe",
  "email": "john@example.com",
  "account_status": "PENDING_APPROVAL",
  "message": "회원가입이 완료되었습니다. 관리자 승인을 기다려주세요."
}
```

### 에러

| 상태 | 설명 |
|------|------|
| 400 | 유효성 검증 실패 (비밀번호 약함, 이메일 형식 오류 등) |
| 409 | username 또는 email 중복 |

---

## 3. 회원수정 (프로필 수정)

사용자의 프로필 정보(이메일, 실명, 소속, 연락처)를 수정합니다.

### Endpoint

```
PUT /api/users/{user_id}
```

### 인증

필수 (`Authorization: Bearer {JWT}`)

### Path Parameters

| 파라미터 | 타입 | 설명 |
|----------|------|------|
| user_id | integer | 수정할 사용자 ID |

### Request

부분 업데이트 지원. 수정하지 않을 필드는 생략하거나 `null` 전달.

```json
{
  "email": "newemail@example.com",
  "full_name": "홍길동",
  "organization": "서울대학교병원",
  "department": "영상의학과",
  "phone": "010-1234-5678"
}
```

| 필드 | 타입 | 필수 | 설명 |
|------|------|------|------|
| email | string | - | 이메일 |
| full_name | string | - | 실명 |
| organization | string | - | 소속 기관 |
| department | string | - | 소속 부서 |
| phone | string | - | 연락처 |

### Response (200 OK)

```json
{
  "id": 123,
  "keycloak_id": "550e8400-e29b-41d4-a716-446655440000",
  "username": "john_doe",
  "email": "newemail@example.com",
  "full_name": "홍길동",
  "organization": "서울대학교병원",
  "department": "영상의학과",
  "phone": "010-1234-5678",
  "account_status": "Active",
  "email_verified": true,
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-02T00:00:00Z"
}
```

### 에러

| 상태 | 설명 |
|------|------|
| 400 | 유효성 검증 실패 |
| 404 | 사용자 없음 |
| 409 | 이메일 중복 |

---

## 4. 회원승인 (관리자 전용)

관리자가 사용자를 승인하여 **로그인 가능**하게 합니다.

- 회원가입 직후 사용자는 Keycloak에서 비활성화 상태입니다.
- 이 API를 호출해야 Keycloak에서 활성화되어 로그인이 가능합니다.
- **승인 전에는 해당 사용자로 로그인할 수 없습니다.**

### Endpoint

```
POST /api/auth/admin/users/approve
```

### 인증

필수 — **관리자 권한** JWT (`Authorization: Bearer {JWT}`)

### Request

```json
{
  "user_id": 123
}
```

| 필드 | 타입 | 필수 | 설명 |
|------|------|------|------|
| user_id | integer | ✅ | 승인할 사용자 ID |

### Response (200 OK)

```json
{
  "message": "사용자가 승인되었습니다."
}
```

### 에러

| 상태 | 설명 |
|------|------|
| 403 | 권한 없음 (관리자 아님) |
| 404 | 사용자 없음 (DB에 없거나 Keycloak에 없음 — 이전 삭제/미생성 가능) |
| 400 | 그 외 승인 실패 |

**404 상세**: Keycloak에 사용자가 없을 경우 `"User not found in identity provider. The user may have been deleted or never created via signup."` 반환

---

## 5. 계정 삭제

사용자 계정을 삭제합니다. Keycloak 및 DB에서 삭제됩니다.

### Endpoint

```
DELETE /api/users/{user_id}
```

### 인증

필수 (`Authorization: Bearer {JWT}`)

### Path Parameters

| 파라미터 | 타입 | 설명 |
|----------|------|------|
| user_id | integer | 삭제할 사용자 ID |

### Response (200 OK)

```json
{
  "message": "계정이 삭제되었습니다."
}
```

---

## 6. 전체 플로우 (시퀀스)

```
[클라이언트]                [서버]                    [관리자]
     │                         │                         │
     │  1. POST /auth/signup   │                         │
     │────────────────────────>│                         │
     │  201 {user_id, ...}     │                         │
     │<────────────────────────│                         │
     │                         │                         │
     │  (관리자 승인 대기)      │  2. POST /auth/admin/users/approve
     │                         │<────────────────────────│
     │                         │  200 {message}          │
     │                         │────────────────────────>│
     │                         │                         │
     │  3. POST /auth/login    │                         │
     │────────────────────────>│                         │
     │  200 {token, ...}       │                         │
     │<────────────────────────│                         │
```

---

## 7. cURL 예시

### 회원가입

```bash
curl -X POST http://localhost:8080/api/auth/signup \
  -H "Content-Type: application/json" \
  -d '{
    "username": "john_doe",
    "email": "john@example.com",
    "password": "SecurePassword123!",
    "full_name": "John Doe"
  }'
```

### 회원수정

```bash
curl -X PUT http://localhost:8080/api/users/123 \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -d '{
    "full_name": "John Doe Updated",
    "phone": "010-9876-5432"
  }'
```

### 회원승인 (관리자)

```bash
curl -X POST http://localhost:8080/api/auth/admin/users/approve \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer ADMIN_JWT_TOKEN" \
  -d '{"user_id": 123}'
```

### 계정 삭제

```bash
curl -X DELETE http://localhost:8080/api/users/123 \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

---

## 8. 관련 문서

- [인증 API (로그인, 토큰)](./AUTH_API.md)
- [User Registration API (상세)](./user-registration-api.md)
- [User Profile Update API (상세)](./user-profile-update-api.md)
