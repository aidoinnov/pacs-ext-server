# 사용자 정보 조회 API 클라이언트 가이드

## 개요

사용자 정보를 조회하는 API입니다. 프로젝트 ID를 함께 제공하면 해당 프로젝트에서의 사용자 역할 이름(`role_name`)도 함께 반환됩니다.

---

## API 엔드포인트

### 1. 내 프로필 조회 (`/api/users/me`)

현재 로그인한 사용자의 프로필 정보를 조회합니다. JWT 토큰에서 사용자 ID를 자동으로 추출합니다.

**Endpoint**: `GET /api/users/me`

**Authentication**: 선택적 (JWT 토큰 또는 쿼리 파라미터)

#### 쿼리 파라미터

| 파라미터 | 타입 | 필수 | 설명 |
|---------|------|------|------|
| `user_id` | number | ❌ | 사용자 ID (JWT 토큰이 없을 때만 사용) |
| `project_id` | number | ❌ | 프로젝트 ID (제공 시 해당 프로젝트에서의 역할 이름 반환) |

#### 사용자 ID 추출 우선순위

1. **JWT 토큰** (`Authorization: Bearer ...`) - 1순위
2. **쿼리 파라미터** (`?user_id=xxx`) - 2순위 (JWT 토큰이 없을 때만)

#### 요청 예제

**JWT 토큰 사용 (권장)**
```http
GET /api/users/me?project_id=2
Authorization: Bearer {jwt_token}
```

**쿼리 파라미터 사용 (JWT 토큰 없을 때)**
```http
GET /api/users/me?user_id=5&project_id=2
```

#### 응답 예제

**project_id 없음**
```json
{
  "id": 5,
  "keycloak_id": "f4e2e355-2102-4fb6-8c6f-88c27443f5d9",
  "username": "reader1_user",
  "email": "reader1@naver.com",
  "full_name": "heeya8876@naver.com",
  "organization": null,
  "department": null,
  "phone": null,
  "account_status": "Active",
  "email_verified": true,
  "role_name": null,
  "created_at": "2025-11-07T09:40:13.067387Z",
  "updated_at": "2025-11-19T09:19:34.481076Z"
}
```

**project_id 포함**
```json
{
  "id": 5,
  "keycloak_id": "f4e2e355-2102-4fb6-8c6f-88c27443f5d9",
  "username": "reader1_user",
  "email": "reader1@naver.com",
  "full_name": "heeya8876@naver.com",
  "organization": null,
  "department": null,
  "phone": null,
  "account_status": "Active",
  "email_verified": true,
  "role_name": "READER2",
  "created_at": "2025-11-07T09:40:13.067387Z",
  "updated_at": "2025-11-19T09:19:34.481076Z"
}
```

#### 에러 응답

**401 Unauthorized** - 사용자 ID를 확인할 수 없음
```json
{
  "error": "Unauthorized",
  "message": "User ID is required. Provide JWT token or user_id query parameter."
}
```

---

### 2. 사용자 조회 (쿼리 파라미터 방식)

쿼리 파라미터로 사용자 ID를 전달하여 사용자 정보를 조회합니다.

**Endpoint**: `GET /api/users/info`

**Authentication**: 선택적

#### 쿼리 파라미터

| 파라미터 | 타입 | 필수 | 설명 |
|---------|------|------|------|
| `user_id` | number | ✅ | 조회할 사용자 ID |
| `project_id` | number | ❌ | 프로젝트 ID (제공 시 해당 프로젝트에서의 역할 이름 반환) |

#### 요청 예제

```http
GET /api/users/info?user_id=5&project_id=2
```

#### 응답 예제

```json
{
  "id": 5,
  "keycloak_id": "f4e2e355-2102-4fb6-8c6f-88c27443f5d9",
  "username": "reader1_user",
  "email": "reader1@naver.com",
  "full_name": "heeya8876@naver.com",
  "organization": null,
  "department": null,
  "phone": null,
  "account_status": "Active",
  "email_verified": true,
  "role_name": "READER2",
  "created_at": "2025-11-07T09:40:13.067387Z",
  "updated_at": "2025-11-19T09:19:34.481076Z"
}
```

#### 에러 응답

**400 Bad Request** - user_id가 없거나 유효하지 않음
```json
{
  "error": "Bad Request",
  "message": "user_id is required and must be greater than 0"
}
```

**404 Not Found** - 사용자를 찾을 수 없음
```json
{
  "error": "User not found: ..."
}
```

---

### 3. 사용자 조회 (Path Parameter 방식)

경로 파라미터로 사용자 ID를 전달하여 사용자 정보를 조회합니다.

**Endpoint**: `GET /api/users/{user_id}`

**Authentication**: 선택적

#### 경로 파라미터

| 파라미터 | 타입 | 필수 | 설명 |
|---------|------|------|------|
| `user_id` | number | ✅ | 조회할 사용자 ID |

#### 쿼리 파라미터

| 파라미터 | 타입 | 필수 | 설명 |
|---------|------|------|------|
| `project_id` | number | ❌ | 프로젝트 ID (제공 시 해당 프로젝트에서의 역할 이름 반환) |

#### 요청 예제

```http
GET /api/users/5?project_id=2
```

#### 응답 예제

```json
{
  "id": 5,
  "keycloak_id": "f4e2e355-2102-4fb6-8c6f-88c27443f5d9",
  "username": "reader1_user",
  "email": "reader1@naver.com",
  "full_name": "heeya8876@naver.com",
  "organization": null,
  "department": null,
  "phone": null,
  "account_status": "Active",
  "email_verified": true,
  "role_name": "READER2",
  "created_at": "2025-11-07T09:40:13.067387Z",
  "updated_at": "2025-11-19T09:19:34.481076Z"
}
```

#### 에러 응답

**404 Not Found** - 사용자를 찾을 수 없음
```json
{
  "error": "User not found: ..."
}
```

---

## 응답 필드 설명

| 필드 | 타입 | 설명 |
|------|------|------|
| `id` | number | 사용자 ID |
| `keycloak_id` | string (UUID) | Keycloak 사용자 ID |
| `username` | string | 사용자명 |
| `email` | string | 이메일 주소 |
| `full_name` | string \| null | 실명 |
| `organization` | string \| null | 소속 기관 |
| `department` | string \| null | 소속 부서/그룹 |
| `phone` | string \| null | 연락처 |
| `account_status` | string | 계정 상태 (`Active`, `Inactive`, `Suspended` 등) |
| `email_verified` | boolean | 이메일 인증 여부 |
| `role_name` | string \| null | **프로젝트별 역할 이름** (project_id 제공 시) |
| `created_at` | string (ISO 8601) | 생성 시간 |
| `updated_at` | string (ISO 8601) \| null | 마지막 업데이트 시간 |

### `role_name` 필드 설명

- **제공 조건**: `project_id` 쿼리 파라미터를 제공한 경우에만 값이 반환됩니다.
- **값의 의미**: 해당 사용자가 지정된 프로젝트에서 가진 역할 이름입니다.
- **가능한 값**: `"ADJUDICATOR"`, `"READER2"`, `"PROJECT_ADMIN"` 등 프로젝트에 할당된 역할 이름
- **null인 경우**:
  - `project_id`를 제공하지 않은 경우
  - 사용자가 해당 프로젝트의 멤버가 아닌 경우
  - 역할이 할당되지 않은 경우

---

## 사용 시나리오

### 시나리오 1: 현재 로그인한 사용자의 프로필 조회

```javascript
// JWT 토큰을 사용하여 현재 사용자 정보 조회
const response = await fetch('/api/users/me', {
  headers: {
    'Authorization': `Bearer ${token}`
  }
});
const user = await response.json();
console.log(user.username); // 현재 사용자명
```

### 시나리오 2: 특정 프로젝트에서의 사용자 역할 확인

```javascript
// 프로젝트 2에서의 현재 사용자 역할 확인
const response = await fetch('/api/users/me?project_id=2', {
  headers: {
    'Authorization': `Bearer ${token}`
  }
});
const user = await response.json();
console.log(user.role_name); // "ADJUDICATOR" 또는 null
```

### 시나리오 3: 다른 사용자 정보 조회 (프로젝트별 역할 포함)

```javascript
// 사용자 5의 정보와 프로젝트 2에서의 역할 조회
const response = await fetch('/api/users/info?user_id=5&project_id=2');
const user = await response.json();
console.log(user.username); // "reader1_user"
console.log(user.role_name); // "READER2"
```

### 시나리오 4: JWT 토큰 없이 사용자 정보 조회

```javascript
// 개발/테스트 환경에서 JWT 토큰 없이 조회
const response = await fetch('/api/users/me?user_id=5&project_id=2');
const user = await response.json();
console.log(user.role_name); // "READER2"
```

---

## TypeScript 타입 정의

```typescript
interface UserResponse {
  id: number;
  keycloak_id: string;
  username: string;
  email: string;
  full_name: string | null;
  organization: string | null;
  department: string | null;
  phone: string | null;
  account_status: string;
  email_verified: boolean;
  role_name: string | null;  // 프로젝트별 역할 이름 (project_id 제공 시)
  created_at: string;
  updated_at: string | null;
}

// API 호출 예시
async function getCurrentUser(projectId?: number): Promise<UserResponse> {
  const url = projectId 
    ? `/api/users/me?project_id=${projectId}`
    : '/api/users/me';
  
  const response = await fetch(url, {
    headers: {
      'Authorization': `Bearer ${getToken()}`
    }
  });
  
  if (!response.ok) {
    throw new Error('Failed to fetch user');
  }
  
  return response.json();
}

// 사용 예시
const user = await getCurrentUser(2);
console.log(user.role_name); // 프로젝트 2에서의 역할 이름
```

---

## 에러 처리

### 일반적인 에러 코드

| 상태 코드 | 설명 | 해결 방법 |
|----------|------|----------|
| 200 | 성공 | - |
| 400 | 잘못된 요청 (user_id가 없거나 유효하지 않음) | user_id 파라미터 확인 |
| 401 | 인증 실패 (JWT 토큰 또는 user_id가 없음) | JWT 토큰 제공 또는 user_id 쿼리 파라미터 추가 |
| 404 | 사용자를 찾을 수 없음 | user_id 확인 |

### 에러 응답 형식

```json
{
  "error": "Error type",
  "message": "Detailed error message"
}
```

---

## 주의사항

1. **`role_name` 필드**:
   - `project_id`를 제공하지 않으면 항상 `null`입니다.
   - 사용자가 해당 프로젝트의 멤버가 아니면 `null`입니다.
   - 역할이 할당되지 않은 경우에도 `null`입니다.

2. **JWT 토큰 우선순위**:
   - `/me` API는 JWT 토큰이 있으면 항상 토큰에서 user_id를 추출합니다.
   - 쿼리 파라미터의 `user_id`는 JWT 토큰이 없을 때만 사용됩니다.

3. **프로젝트 멤버십**:
   - `role_name`은 현재 시점의 프로젝트 멤버십을 기준으로 조회됩니다.
   - 사용자가 프로젝트에서 제거되면 `null`을 반환합니다.

---

## cURL 예제

### 내 프로필 조회 (JWT 토큰 사용)

```bash
curl -X GET "http://localhost:8080/api/users/me?project_id=2" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json"
```

### 내 프로필 조회 (쿼리 파라미터 사용)

```bash
curl -X GET "http://localhost:8080/api/users/me?user_id=5&project_id=2" \
  -H "Content-Type: application/json"
```

### 쿼리 파라미터 방식

```bash
curl -X GET "http://localhost:8080/api/users/info?user_id=5&project_id=2" \
  -H "Content-Type: application/json"
```

### Path Parameter 방식

```bash
curl -X GET "http://localhost:8080/api/users/5?project_id=2" \
  -H "Content-Type: application/json"
```

---

## 요약

| 엔드포인트 | user_id 제공 방식 | project_id | role_name 반환 |
|-----------|------------------|------------|----------------|
| `GET /api/users/me` | JWT 토큰 (1순위) 또는 쿼리 파라미터 (2순위) | 쿼리 파라미터 | ✅ (project_id 제공 시) |
| `GET /api/users/info` | 쿼리 파라미터 (필수) | 쿼리 파라미터 | ✅ (project_id 제공 시) |
| `GET /api/users/{user_id}` | Path parameter (필수) | 쿼리 파라미터 | ✅ (project_id 제공 시) |

---

## 관련 API

- [Annotation 권한 조회 API](./annotation-api/ANNOTATION-PERMISSIONS-API.md) - 사용자의 어노테이션 권한 조회
- [사용자 프로젝트 목록 조회](./user-crud-api-complete.md#사용자가-속한-프로젝트-목록-조회) - 사용자가 속한 프로젝트 목록 조회


