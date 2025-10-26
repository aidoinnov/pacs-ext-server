# User CRUD API (완전한 문서)

## 개요

사용자 생성, 조회, 수정, 목록 조회, 삭제를 위한 완전한 CRUD API입니다.

**Base URL**: `/api/users`

---

## API 엔드포인트

### 0. 사용자 목록 조회

모든 사용자의 목록을 페이지네이션, 정렬, 검색을 지원하여 조회합니다.

**Endpoint**: `GET /api/users`

**Authentication**: Required (JWT Token)

#### Query Parameters

| Parameter | Type | Required | Description | Example |
|-----------|------|----------|-------------|---------|
| `page` | integer | No | 페이지 번호 (기본값: 1) | `1` |
| `page_size` | integer | No | 페이지 크기 (기본값: 20, 최대: 100) | `20` |
| `sort_by` | string | No | 정렬 기준 (username, email, created_at) | `username` |
| `sort_order` | string | No | 정렬 순서 (asc, desc) | `asc` |
| `search` | string | No | 검색어 (username, email 검색) | `john` |

#### Response

**Success Response** (200 OK)

```json
{
  "users": [
    {
      "id": 1,
      "keycloak_id": "550e8400-e29b-41d4-a716-446655440000",
      "username": "TestUser2",
      "email": "user2@example.com",
      "full_name": "홍길동",
      "organization": "서울대학교병원",
      "department": "영상의학과",
      "phone": "010-1234-5678",
      "created_at": "2024-01-01T00:00:00Z",
      "updated_at": "2024-01-02T00:00:00Z"
    }
  ],
  "pagination": {
    "page": 1,
    "page_size": 20,
    "total": 58,
    "total_pages": 3
  }
}
```

**Response Schema**

| Field | Type | Description |
|-------|------|-------------|
| `users` | array | 사용자 목록 |
| `pagination` | object | 페이지네이션 정보 |
| `pagination.page` | integer | 현재 페이지 번호 |
| `pagination.page_size` | integer | 페이지 크기 |
| `pagination.total` | integer | 전체 항목 수 |
| `pagination.total_pages` | integer | 전체 페이지 수 |

#### cURL 예시

```bash
# 기본 조회
curl -X GET "http://localhost:8080/api/users" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"

# 페이지네이션, 정렬, 검색
curl -X GET "http://localhost:8080/api/users?page=1&page_size=10&sort_by=username&sort_order=asc&search=john" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

---

## API 엔드포인트

### 1. 사용자 생성

새로운 사용자를 생성합니다.

**Endpoint**: `POST /api/users`

**Authentication**: Required (JWT Token)

**Content-Type**: `application/json`

#### Request Body

```json
{
  "keycloak_id": "550e8400-e29b-41d4-a716-446655440000",
  "username": "honggildong",
  "email": "hong@example.com",
  "full_name": "홍길동",
  "organization": "서울대학교병원",
  "department": "영상의학과",
  "phone": "010-1234-5678"
}
```

#### Request Schema

| Field | Type | Required | Description | Example |
|-------|------|----------|-------------|---------|
| `keycloak_id` | UUID | Yes | Keycloak 사용자 ID | `"550e8400-e29b-41d4-a716-446655440000"` |
| `username` | string | Yes | 사용자명 | `"honggildong"` |
| `email` | string | Yes | 이메일 주소 | `"hong@example.com"` |
| `full_name` | string | No | 실명 | `"홍길동"` |
| `organization` | string | No | 소속 기관 | `"서울대학교병원"` |
| `department` | string | No | 소속 부서/그룹 | `"영상의학과"` |
| `phone` | string | No | 연락처 | `"010-1234-5678"` |

#### Response

**Success Response** (201 Created)

```json
{
  "id": 1,
  "keycloak_id": "550e8400-e29b-41d4-a716-446655440000",
  "username": "honggildong",
  "email": "hong@example.com",
  "full_name": "홍길동",
  "organization": "서울대학교병원",
  "department": "영상의학과",
  "phone": "010-1234-5678",
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": null
}
```

**Response Schema**

| Field | Type | Description |
|-------|------|-------------|
| `id` | integer | 사용자 ID |
| `keycloak_id` | UUID | Keycloak 사용자 ID |
| `username` | string | 사용자명 |
| `email` | string | 이메일 주소 |
| `full_name` | string | 실명 |
| `organization` | string | 소속 기관 |
| `department` | string | 소속 부서/그룹 |
| `phone` | string | 연락처 |
| `created_at` | string (ISO 8601) | 생성 시간 |
| `updated_at` | string (ISO 8601) | 마지막 업데이트 시간 |

**Error Responses**

| Status Code | Description | Response Body |
|-------------|-------------|---------------|
| 400 | Invalid request | `{"error": "Failed to create user: ..."}` |
| 401 | Unauthorized | Authentication error |
| 409 | Already exists | `{"error": "Failed to create user: Username already exists"}` |

---

### 2. 사용자 조회 (by ID)

특정 사용자의 정보를 조회합니다.

**Endpoint**: `GET /api/users/{user_id}`

**Authentication**: Required (JWT Token)

#### Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `user_id` | integer | Yes | 조회할 사용자 ID |

#### Response

**Success Response** (200 OK)

```json
{
  "id": 1,
  "keycloak_id": "550e8400-e29b-41d4-a716-446655440000",
  "username": "honggildong",
  "email": "hong@example.com",
  "full_name": "홍길동",
  "organization": "서울대학교병원",
  "department": "영상의학과",
  "phone": "010-1234-5678",
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": null
}
```

**Error Responses**

| Status Code | Description | Response Body |
|-------------|-------------|---------------|
| 404 | User not found | `{"error": "User not found: ..."}` |
| 401 | Unauthorized | Authentication error |

---

### 3. 사용자 조회 (by username)

특정 사용자명으로 사용자 정보를 조회합니다.

**Endpoint**: `GET /api/users/username/{username}`

**Authentication**: Required (JWT Token)

#### Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `username` | string | Yes | 조회할 사용자명 |

#### Response

**Success Response** (200 OK)

```json
{
  "id": 1,
  "keycloak_id": "550e8400-e29b-41d4-a716-446655440000",
  "username": "honggildong",
  "email": "hong@example.com",
  "full_name": "홍길동",
  "organization": "서울대학교병원",
  "department": "영상의학과",
  "phone": "010-1234-5678",
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": null
}
```

---

### 4. 사용자 수정

사용자 정보를 수정합니다.

**Endpoint**: `PUT /api/users/{user_id}`

**Authentication**: Required (JWT Token)

**Content-Type**: `application/json`

#### Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `user_id` | integer | Yes | 수정할 사용자 ID |

#### Request Body

```json
{
  "email": "hong.updated@example.com",
  "full_name": "홍길동 (업데이트)",
  "organization": "서울대학교병원",
  "department": "영상의학과",
  "phone": "010-9876-5432"
}
```

#### Request Schema

| Field | Type | Required | Description | Example |
|-------|------|----------|-------------|---------|
| `email` | string | No | 이메일 주소 | `"hong.updated@example.com"` |
| `full_name` | string | No | 실명 | `"홍길동 (업데이트)"` |
| `organization` | string | No | 소속 기관 | `"서울대학교병원"` |
| `department` | string | No | 소속 부서/그룹 | `"영상의학과"` |
| `phone` | string | No | 연락처 | `"010-9876-5432"` |

#### Response

**Success Response** (200 OK)

```json
{
  "id": 1,
  "keycloak_id": "550e8400-e29b-41d4-a716-446655440000",
  "username": "honggildong",
  "email": "hong.updated@example.com",
  "full_name": "홍길동 (업데이트)",
  "organization": "서울대학교병원",
  "department": "영상의학과",
  "phone": "010-9876-5432",
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-02T00:00:00Z"
}
```

**Error Responses**

| Status Code | Description | Response Body |
|-------------|-------------|---------------|
| 400 | Invalid request | `{"error": "Failed to update user: ..."}` |
| 404 | User not found | `{"error": "Failed to update user: User not found"}` |
| 409 | Email already taken | `{"error": "Failed to update user: Email already exists"}` |
| 401 | Unauthorized | Authentication error |

---

### 5. 사용자가 참여하는 프로젝트 조회

사용자가 참여하는 모든 프로젝트 목록을 조회합니다.

**Endpoint**: `GET /api/users/{user_id}/projects`

**Authentication**: Required (JWT Token)

#### Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `user_id` | integer | Yes | 조회할 사용자 ID |

#### Query Parameters

| Parameter | Type | Required | Description | Example |
|-----------|------|----------|-------------|---------|
| `page` | integer | No | 페이지 번호 (기본값: 1) | `1` |
| `page_size` | integer | No | 페이지 크기 (기본값: 10) | `10` |

#### Response

**Success Response** (200 OK)

```json
{
  "projects": [
    {
      "id": 1,
      "name": "심장 질환 연구 프로젝트",
      "description": "심장 질환 관련 DICOM 영상 분석",
      "sponsor": "서울대학교병원",
      "start_date": "2025-01-01",
      "end_date": "2025-12-31",
      "auto_complete": false,
      "is_active": true,
      "status": "Active",
      "created_at": "2025-01-01T00:00:00Z"
    }
  ],
  "pagination": {
    "page": 1,
    "page_size": 10,
    "total": 1,
    "total_pages": 1
  }
}
```

---

## 사용 예시

### cURL 요청 예시

#### 사용자 생성

```bash
curl -X POST "http://localhost:8080/api/users" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "keycloak_id": "550e8400-e29b-41d4-a716-446655440000",
    "username": "honggildong",
    "email": "hong@example.com",
    "full_name": "홍길동",
    "organization": "서울대학교병원",
    "department": "영상의학과",
    "phone": "010-1234-5678"
  }'
```

#### 사용자 조회 (by ID)

```bash
curl -X GET "http://localhost:8080/api/users/1" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

#### 사용자 조회 (by username)

```bash
curl -X GET "http://localhost:8080/api/users/username/honggildong" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

#### 사용자 수정

```bash
curl -X PUT "http://localhost:8080/api/users/1" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "hong.updated@example.com",
    "full_name": "홍길동 (업데이트)",
    "organization": "서울대학교병원",
    "department": "영상의학과",
    "phone": "010-9876-5432"
  }'
```

#### 사용자가 참여하는 프로젝트 조회

```bash
curl -X GET "http://localhost:8080/api/users/1/projects?page=1&page_size=10" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

---

## 인증 관련 API

### 6. 사용자 회원가입

**Endpoint**: `POST /api/auth/signup`

자세한 내용은 `docs/api/user-registration-api.md` 참조

### 7. 이메일 인증

**Endpoint**: `POST /api/auth/verify-email`

자세한 내용은 `docs/api/user-registration-api.md` 참조

### 8. 사용자 승인 (관리자)

**Endpoint**: `POST /api/admin/users/approve`

자세한 내용은 `docs/api/user-registration-api.md` 참조

### 9. 사용자 계정 삭제

**Endpoint**: `DELETE /api/users/{user_id}`

**Authentication**: Required (JWT Token)

#### Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `user_id` | integer | Yes | 삭제할 사용자 ID |

#### Response

**Success Response** (204 No Content)

사용자 계정이 성공적으로 삭제되면 내용 없이 204 상태 코드를 반환합니다.

**Error Responses**

| Status Code | Description | Response Body |
|-------------|-------------|---------------|
| 404 | User not found | `{"error": "Failed to delete user: ..."}` |
| 500 | Internal server error | `{"error": "..."}` |

#### cURL 예시

```bash
curl -X DELETE "http://localhost:8080/api/users/1" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

---

### 10. 사용자 상태 조회

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
  "user_id": 1,
  "username": "honggildong",
  "email": "hong@example.com",
  "account_status": "ACTIVE",
  "email_verified": true,
  "is_approved": true,
  "approved_by": 1,
  "approved_at": "2024-01-01T00:00:00Z"
}
```

---

## 프로젝트 관련 API

### 11. 프로젝트 멤버 조회

프로젝트에 참여하는 모든 멤버를 조회합니다.

**Endpoint**: `GET /api/projects/{project_id}/users`

**Authentication**: Required (JWT Token)

#### Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `project_id` | integer | Yes | 프로젝트 ID |

#### Response

**Success Response** (200 OK)

```json
{
  "project_id": 1,
  "members": [
    {
      "id": 1,
      "username": "honggildong",
      "email": "hong@example.com",
      "joined_at": "2024-01-01T00:00:00Z"
    }
  ],
  "total": 1
}
```

---

### 12. 프로젝트 멤버 추가

프로젝트에 멤버를 추가합니다.

**Endpoint**: `POST /api/projects/{project_id}/members`

**Authentication**: Required (JWT Token)

**Content-Type**: `application/json`

#### Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `project_id` | integer | Yes | 프로젝트 ID |

#### Request Body

```json
{
  "user_id": 1
}
```

#### Response

**Success Response** (200 OK)

```json
{
  "message": "Member added successfully",
  "user_id": 1,
  "project_id": 1
}
```

---

### 13. 프로젝트 멤버 제거

프로젝트에서 멤버를 제거합니다.

**Endpoint**: `DELETE /api/projects/{project_id}/members/{user_id}`

**Authentication**: Required (JWT Token)

#### Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `project_id` | integer | Yes | 프로젝트 ID |
| `user_id` | integer | Yes | 사용자 ID |

#### Response

**Success Response** (204 No Content)

---

### 14. 프로젝트 멤버십 확인

사용자가 프로젝트의 멤버인지 확인합니다.

**Endpoint**: `GET /api/projects/{project_id}/members/{user_id}/membership`

**Authentication**: Required (JWT Token)

#### Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `project_id` | integer | Yes | 프로젝트 ID |
| `user_id` | integer | Yes | 사용자 ID |

#### Response

**Success Response** (200 OK)

```json
{
  "is_member": true,
  "user_id": 1,
  "project_id": 1
}
```

---

## 사용자 역할 관련 API

### 15. 프로젝트에서 사용자 역할 할당

프로젝트 내 사용자에게 역할을 할당합니다.

**Endpoint**: `PUT /api/projects/{project_id}/users/{user_id}/role`

**Authentication**: Required (JWT Token)

**Content-Type**: `application/json`

#### Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `project_id` | integer | Yes | 프로젝트 ID |
| `user_id` | integer | Yes | 사용자 ID |

#### Request Body

```json
{
  "role_id": 2
}
```

#### Response

**Success Response** (200 OK)

```json
{
  "message": "Role assigned successfully"
}
```

---

### 16. 프로젝트에서 사용자 역할 제거

프로젝트 내 사용자에게 할당된 역할을 제거합니다.

**Endpoint**: `DELETE /api/projects/{project_id}/users/{user_id}/role`

**Authentication**: Required (JWT Token)

#### Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `project_id` | integer | Yes | 프로젝트 ID |
| `user_id` | integer | Yes | 사용자 ID |

#### Response

**Success Response** (200 OK)

```json
{
  "message": "Role removed successfully"
}
```

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
| 401 | Unauthorized | 인증되지 않은 요청 |
| 404 | User not found | 사용자가 존재하지 않음 |
| 409 | Already exists | 이미 존재하는 사용자 (username, email 중복) |
| 500 | Internal server error | 서버 내부 오류 |

---

## 중요한 필드 설명

### keycloak_id

Keycloak 인증 시스템에서 사용하는 사용자 식별자입니다. UUID 형식입니다.

### username

로그인에 사용되는 고유한 사용자명입니다. 중복될 수 없습니다.

### email

사용자 이메일 주소입니다. 로그인 및 알림에 사용됩니다. 중복될 수 없습니다.

### full_name

사용자의 실명입니다. 한글명 또는 영문명이 될 수 있습니다.

### organization

사용자가 소속된 기관입니다. 예: "서울대학교병원", "삼성서울병원"

### department

사용자가 소속된 부서 또는 그룹입니다. 예: "영상의학과", "방사선과"

### phone

사용자 연락처입니다. 예: "010-1234-5678"

---

**최종 업데이트**: 2025-01-27

