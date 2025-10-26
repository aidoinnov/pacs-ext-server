# User Registration API (사용자 회원가입 API)

## 개요

사용자 회원가입, 이메일 인증, 관리자 승인, 계정 삭제 등의 사용자 등록 프로세스를 관리하는 API입니다.

**Base URL**: `/api`

---

## API 엔드포인트

### 1. 회원가입

새로운 사용자를 등록합니다. Keycloak과 데이터베이스에 원자적으로 사용자를 생성합니다.

**Endpoint**: `POST /api/auth/signup`

**Authentication**: Not Required (Public)

**Content-Type**: `application/json`

#### Request Body

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

#### Request Schema

| Field | Type | Required | Description | Example |
|-------|------|----------|-------------|---------|
| `username` | string | Yes | 사용자명 (3자 이상) | `"john_doe"` |
| `email` | string | Yes | 이메일 주소 | `"john@example.com"` |
| `password` | string | Yes | 비밀번호 (8자 이상) | `"SecurePassword123!"` |
| `full_name` | string | No | 실명 | `"John Doe"` |
| `organization` | string | No | 소속 기관 | `"Seoul National University Hospital"` |
| `department` | string | No | 소속 부서 | `"Radiology Department"` |
| `phone` | string | No | 연락처 | `"010-1234-5678"` |

#### Response

**Success Response** (201 Created)

```json
{
  "user_id": 123,
  "username": "john_doe",
  "email": "john@example.com",
  "account_status": "PENDING_EMAIL",
  "message": "회원가입이 완료되었습니다. 이메일 인증을 완료해주세요."
}
```

**Response Schema**

| Field | Type | Description |
|-------|------|-------------|
| `user_id` | integer | 생성된 사용자 ID |
| `username` | string | 사용자명 |
| `email` | string | 이메일 주소 |
| `account_status` | string | 계정 상태 (PENDING_EMAIL) |
| `message` | string | 응답 메시지 |

**Error Responses**

| Status Code | Description | Response Body |
|-------------|-------------|---------------|
| 400 | Invalid request | `{"error": "..."}` |
| 409 | Already exists | `{"error": "Username or email already exists"}` |
| 500 | Internal server error | `{"error": "..."}` |

---

### 2. 이메일 인증 완료

사용자가 이메일 인증을 완료했을 때 호출됩니다. 계정 상태를 PENDING_EMAIL에서 PENDING_APPROVAL로 변경합니다.

**Endpoint**: `POST /api/auth/verify-email`

**Authentication**: Not Required (Public)

**Content-Type**: `application/json`

#### Request Body

```json
{
  "user_id": 123
}
```

#### Request Schema

| Field | Type | Required | Description | Example |
|-------|------|----------|-------------|---------|
| `user_id` | integer | Yes | 사용자 ID | `123` |

#### Response

**Success Response** (200 OK)

```json
{
  "message": "이메일 인증이 완료되었습니다. 관리자 승인을 기다려주세요."
}
```

**Error Responses**

| Status Code | Description | Response Body |
|-------------|-------------|---------------|
| 400 | Invalid request | `{"error": "..."}` |
| 404 | User not found | `{"error": "User not found"}` |
| 500 | Internal server error | `{"error": "..."}` |

---

### 3. 사용자 승인 (관리자 전용)

관리자가 사용자를 승인할 때 호출됩니다. Keycloak에서 사용자를 활성화하고 계정 상태를 ACTIVE로 변경합니다.

**Endpoint**: `POST /api/admin/users/approve`

**Authentication**: Required (Admin JWT Token)

**Content-Type**: `application/json`

#### Request Body

```json
{
  "user_id": 123
}
```

#### Request Schema

| Field | Type | Required | Description | Example |
|-------|------|----------|-------------|---------|
| `user_id` | integer | Yes | 승인할 사용자 ID | `123` |

#### Response

**Success Response** (200 OK)

```json
{
  "message": "사용자가 승인되었습니다."
}
```

**Error Responses**

| Status Code | Description | Response Body |
|-------------|-------------|---------------|
| 403 | Forbidden (권한 없음) | `{"error": "..."}` |
| 404 | User not found | `{"error": "User not found"}` |
| 500 | Internal server error | `{"error": "..."}` |

---

### 4. 사용자 계정 삭제

사용자 계정을 삭제합니다. Keycloak과 데이터베이스에서 원자적으로 삭제합니다. 감사 로그는 별도 보관됩니다.

**Endpoint**: `DELETE /api/users/{user_id}`

**Authentication**: Required (JWT Token)

#### Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `user_id` | integer | Yes | 삭제할 사용자 ID |

#### Response

**Success Response** (200 OK)

```json
{
  "message": "계정이 삭제되었습니다."
}
```

**Error Responses**

| Status Code | Description | Response Body |
|-------------|-------------|---------------|
| 403 | Forbidden (권한 없음) | `{"error": "..."}` |
| 404 | User not found | `{"error": "User not found"}` |
| 500 | Internal server error | `{"error": "..."}` |

---

### 5. 사용자 상태 조회

사용자의 현재 계정 상태를 조회합니다.

**Endpoint**: `GET /api/users/{user_id}/status`

**Authentication**: Required (JWT Token)

#### Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `user_id` | integer | Yes | 조회할 사용자 ID |

#### Response

**Success Response** (200 OK)

```json
{
  "user_id": 123,
  "username": "john_doe",
  "email": "john@example.com",
  "account_status": "ACTIVE",
  "email_verified": true,
  "is_approved": true,
  "approved_by": 1,
  "approved_at": "2025-01-27T10:00:00Z"
}
```

**Response Schema**

| Field | Type | Description |
|-------|------|-------------|
| `user_id` | integer | 사용자 ID |
| `username` | string | 사용자명 |
| `email` | string | 이메일 주소 |
| `account_status` | string | 계정 상태 |
| `email_verified` | boolean | 이메일 인증 완료 여부 |
| `is_approved` | boolean | 승인 여부 |
| `approved_by` | integer | 승인자 ID (승인된 경우) |
| `approved_at` | string (ISO 8601) | 승인 시간 (승인된 경우) |

**계정 상태 (account_status)**

- `PENDING_EMAIL`: 이메일 인증 대기 중
- `PENDING_APPROVAL`: 관리자 승인 대기 중
- `ACTIVE`: 활성 계정
- `SUSPENDED`: 정지된 계정
- `DELETED`: 삭제된 계정

---

## 사용 예시

### cURL 요청 예시

#### 회원가입

```bash
curl -X POST "http://localhost:8080/api/auth/signup" \
  -H "Content-Type: application/json" \
  -d '{
    "username": "john_doe",
    "email": "john@example.com",
    "password": "SecurePassword123!",
    "full_name": "John Doe",
    "organization": "Seoul National University Hospital",
    "department": "Radiology Department",
    "phone": "010-1234-5678"
  }'
```

#### 이메일 인증 완료

```bash
curl -X POST "http://localhost:8080/api/auth/verify-email" \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": 123
  }'
```

#### 사용자 승인 (관리자)

```bash
curl -X POST "http://localhost:8080/api/admin/users/approve" \
  -H "Authorization: Bearer YOUR_ADMIN_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": 123
  }'
```

#### 계정 삭제

```bash
curl -X DELETE "http://localhost:8080/api/users/123" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

#### 사용자 상태 조회

```bash
curl -X GET "http://localhost:8080/api/users/123/status" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

---

## 회원가입 프로세스

### 1단계: 회원가입 (POST /api/auth/signup)
- 사용자가 회원가입 요청
- Keycloak에 사용자 생성
- 데이터베이스에 사용자 정보 저장
- 계정 상태: `PENDING_EMAIL`
- 이메일 인증 링크 전송

### 2단계: 이메일 인증 (POST /api/auth/verify-email)
- 사용자가 이메일 인증 링크 클릭
- 계정 상태: `PENDING_EMAIL` → `PENDING_APPROVAL`
- 관리자 승인 대기 상태

### 3단계: 관리자 승인 (POST /api/admin/users/approve)
- 관리자가 사용자 승인
- Keycloak에서 사용자 활성화
- 계정 상태: `PENDING_APPROVAL` → `ACTIVE`
- 사용자가 로그인 가능

---

## 에러 처리

### 공통 에러 응답 형식

```json
{
  "error": "[에러 메시지]"
}
```

### 에러 코드별 설명

| HTTP Status | 에러 내용 | 설명 |
|-------------|---------|------|
| 400 | Invalid request | 요청 데이터가 유효하지 않음 |
| 403 | Forbidden | 권한이 없음 (관리자 전용 API) |
| 404 | User not found | 사용자가 존재하지 않음 |
| 409 | Already exists | 이미 존재하는 사용자명 또는 이메일 |
| 500 | Internal server error | 서버 내부 오류 |

---

## 주요 기능

### 원자적 처리

모든 작업은 원자적으로 처리됩니다:
- 회원가입 실패 시 Keycloak과 데이터베이스 모두 롤백
- 이메일 인증 실패 시 상태 변경 없음
- 승인 실패 시 Keycloak 활성화 실패 시 롤백
- 계정 삭제 시 감사 로그는 영구 보관

### 감사 로그

모든 작업이 감사 로그에 기록됩니다:
- 회원가입 시간
- 이메일 인증 시간
- 승인 시간 및 승인자
- 계정 삭제 시간 및 삭제자

### Keycloak 통합

- 사용자 생성을 Keycloak에 먼저 요청
- Keycloak 사용자 ID (keycloak_id)를 데이터베이스에 저장
- Keycloak과 데이터베이스 동기화 유지

---

**최종 업데이트**: 2025-01-27

