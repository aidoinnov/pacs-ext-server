# 프로젝트별 사용자 Role 관리 API 문서

## 📋 개요

프로젝트별로 사용자의 역할(Role)을 관리하기 위한 API 엔드포인트입니다. 사용자 x 프로젝트 매트릭스 형태로 데이터를 조회하고, 각 프로젝트에서 사용자의 역할을 변경할 수 있습니다.

## 🔗 기본 URL

```
http://localhost:8080/api
```

## 📊 데이터 모델

### UserWithRoleResponse
```typescript
interface UserWithRoleResponse {
  user_id: number;              // 사용자 ID
  username: string;             // 사용자명
  email: string;                // 이메일
  full_name?: string;           // 전체 이름
  organization?: string;        // 조직
  department?: string;          // 부서
  phone?: string;               // 전화번호
  role_id?: number;             // 할당된 역할 ID
  role_name?: string;           // 할당된 역할 이름
  role_description?: string;    // 할당된 역할 설명
  role_scope?: string;          // 역할 범위 ("GLOBAL" | "PROJECT")
  assigned_at?: string;         // 역할 할당일시 (ISO 8601)
}
```

### ProjectWithRoleResponse
```typescript
interface ProjectWithRoleResponse {
  project_id: number;           // 프로젝트 ID
  name: string;                 // 프로젝트 이름
  description?: string;         // 프로젝트 설명
  status: string;               // 프로젝트 상태
  created_at: string;           // 생성일시 (ISO 8601)
  role_id?: number;             // 할당된 역할 ID
  role_name?: string;           // 할당된 역할 이름
  role_description?: string;    // 할당된 역할 설명
  role_scope?: string;          // 역할 범위 ("GLOBAL" | "PROJECT")
  assigned_at?: string;         // 역할 할당일시 (ISO 8601)
}
```

### AssignRoleRequest
```typescript
interface AssignRoleRequest {
  role_id: number;              // 할당할 역할 ID
}
```

### BatchAssignRolesRequest
```typescript
interface BatchAssignRolesRequest {
  assignments: Array<{
    user_id: number;            // 사용자 ID
    role_id: number;            // 할당할 역할 ID
  }>;
}
```

### RoleAssignmentResponse
```typescript
interface RoleAssignmentResponse {
  user_id: number;              // 사용자 ID
  project_id: number;           // 프로젝트 ID
  role_id: number;              // 할당된 역할 ID
  role_name: string;            // 할당된 역할 이름
  message: string;              // 응답 메시지
  assigned_at: string;          // 할당일시 (ISO 8601)
}
```

### BatchRoleAssignmentResponse
```typescript
interface BatchRoleAssignmentResponse {
  successful_assignments: Array<RoleAssignmentResponse>;
  failed_assignments: Array<{
    user_id: number;
    error: string;
  }>;
  total_successful: number;     // 성공한 할당 수
  total_failed: number;         // 실패한 할당 수
}
```

### PaginationInfo
```typescript
interface PaginationInfo {
  current_page: number;         // 현재 페이지
  page_size: number;            // 페이지 크기
  total_items: number;          // 전체 항목 수
  total_pages: number;          // 전체 페이지 수
  has_next: boolean;            // 다음 페이지 존재 여부
  has_prev: boolean;            // 이전 페이지 존재 여부
}
```

## 🚀 API 엔드포인트

### 1. 프로젝트 멤버 목록 조회 (역할 정보 포함)

**GET** `/api/projects/{project_id}/users`

특정 프로젝트의 멤버 목록을 역할 정보와 함께 페이지네이션으로 조회합니다.

#### 요청

```http
GET /api/projects/1/users?page=1&page_size=20
```

#### 쿼리 파라미터

| 파라미터 | 타입 | 기본값 | 설명 |
|----------|------|--------|------|
| page | integer | 1 | 페이지 번호 (1부터 시작) |
| page_size | integer | 20 | 페이지 크기 (최대 100) |

#### 응답

**성공 (200 OK)**
```json
{
  "members": [
    {
      "user_id": 1,
      "username": "john.doe",
      "email": "john.doe@example.com",
      "full_name": "John Doe",
      "organization": "Medical Center",
      "department": "Radiology",
      "phone": "+1-555-0123",
      "role_id": 2,
      "role_name": "PROJECT_ADMIN",
      "role_description": "프로젝트 관리자",
      "role_scope": "PROJECT",
      "assigned_at": "2025-01-27T10:30:00Z"
    },
    {
      "user_id": 2,
      "username": "jane.smith",
      "email": "jane.smith@example.com",
      "full_name": "Jane Smith",
      "organization": "Medical Center",
      "department": "Radiology",
      "phone": "+1-555-0124",
      "role_id": 3,
      "role_name": "PROJECT_MEMBER",
      "role_description": "프로젝트 멤버",
      "role_scope": "PROJECT",
      "assigned_at": "2025-01-27T11:00:00Z"
    }
  ],
  "pagination": {
    "current_page": 1,
    "page_size": 20,
    "total_items": 2,
    "total_pages": 1,
    "has_next": false,
    "has_prev": false
  }
}
```

**실패 (404 Not Found)**
```json
{
  "error": "Failed to get project members: Project not found"
}
```

### 2. 사용자의 프로젝트 목록 조회 (역할 정보 포함)

**GET** `/api/users/{user_id}/projects`

특정 사용자가 참여한 프로젝트 목록을 역할 정보와 함께 페이지네이션으로 조회합니다.

#### 요청

```http
GET /api/users/1/projects?page=1&page_size=20
```

#### 쿼리 파라미터

| 파라미터 | 타입 | 기본값 | 설명 |
|----------|------|--------|------|
| page | integer | 1 | 페이지 번호 (1부터 시작) |
| page_size | integer | 20 | 페이지 크기 (최대 100) |

#### 응답

**성공 (200 OK)**
```json
{
  "projects": [
    {
      "project_id": 1,
      "name": "Chest X-ray Analysis",
      "description": "흉부 X-ray 이미지 분석 프로젝트",
      "status": "ACTIVE",
      "created_at": "2025-01-27T10:00:00Z",
      "role_id": 2,
      "role_name": "PROJECT_ADMIN",
      "role_description": "프로젝트 관리자",
      "role_scope": "PROJECT",
      "assigned_at": "2025-01-27T10:30:00Z"
    },
    {
      "project_id": 2,
      "name": "MRI Brain Scan",
      "description": "MRI 뇌 스캔 이미지 분석 프로젝트",
      "status": "ACTIVE",
      "created_at": "2025-01-27T11:00:00Z",
      "role_id": 3,
      "role_name": "PROJECT_MEMBER",
      "role_description": "프로젝트 멤버",
      "role_scope": "PROJECT",
      "assigned_at": "2025-01-27T11:30:00Z"
    }
  ],
  "pagination": {
    "current_page": 1,
    "page_size": 20,
    "total_items": 2,
    "total_pages": 1,
    "has_next": false,
    "has_prev": false
  }
}
```

### 3. 프로젝트 내 사용자에게 역할 할당

**PUT** `/api/projects/{project_id}/users/{user_id}/role`

특정 프로젝트에서 사용자에게 역할을 할당합니다.

#### 요청

```http
PUT /api/projects/1/users/2/role
Content-Type: application/json

{
  "role_id": 2
}
```

#### 응답

**성공 (200 OK)**
```json
{
  "user_id": 2,
  "project_id": 1,
  "role_id": 2,
  "role_name": "PROJECT_ADMIN",
  "message": "Role assigned successfully",
  "assigned_at": "2025-01-27T12:00:00Z"
}
```

**실패 (404 Not Found)**
```json
{
  "error": "Failed to assign role: Project, user, or role not found"
}
```

### 4. 프로젝트 내 여러 사용자에게 역할 일괄 할당

**POST** `/api/projects/{project_id}/users/roles`

특정 프로젝트에서 여러 사용자에게 역할을 일괄 할당합니다.

#### 요청

```http
POST /api/projects/1/users/roles
Content-Type: application/json

{
  "assignments": [
    {
      "user_id": 2,
      "role_id": 2
    },
    {
      "user_id": 3,
      "role_id": 3
    }
  ]
}
```

#### 응답

**성공 (200 OK)**
```json
{
  "successful_assignments": [
    {
      "user_id": 2,
      "project_id": 1,
      "role_id": 2,
      "role_name": "PROJECT_ADMIN",
      "message": "Role assigned successfully",
      "assigned_at": "2025-01-27T12:00:00Z"
    },
    {
      "user_id": 3,
      "project_id": 1,
      "role_id": 3,
      "role_name": "PROJECT_MEMBER",
      "message": "Role assigned successfully",
      "assigned_at": "2025-01-27T12:00:00Z"
    }
  ],
  "failed_assignments": [],
  "total_successful": 2,
  "total_failed": 0
}
```

### 5. 프로젝트 내 사용자의 역할 제거

**DELETE** `/api/projects/{project_id}/users/{user_id}/role`

특정 프로젝트에서 사용자의 역할을 제거합니다.

#### 요청

```http
DELETE /api/projects/1/users/2/role
```

#### 응답

**성공 (200 OK)**
```json
{
  "user_id": 2,
  "project_id": 1,
  "role_id": null,
  "role_name": null,
  "message": "User role removed successfully",
  "assigned_at": null
}
```

**실패 (404 Not Found)**
```json
{
  "error": "Failed to remove user role: Project or user not found"
}
```

## 🔧 역할 목록 조회 API

### 1. 전역 역할 목록 조회

**GET** `/api/roles/global`

모든 전역 역할 목록을 조회합니다.

#### 요청

```http
GET /api/roles/global
```

#### 응답

**성공 (200 OK)**
```json
[
  {
    "id": 1,
    "name": "SUPER_ADMIN",
    "description": "시스템 전체 관리자",
    "scope": "GLOBAL",
    "created_at": "2025-01-27T10:00:00Z"
  },
  {
    "id": 2,
    "name": "ADMIN",
    "description": "관리자",
    "scope": "GLOBAL",
    "created_at": "2025-01-27T10:00:00Z"
  }
]
```

### 2. 프로젝트 역할 목록 조회

**GET** `/api/roles/project`

모든 프로젝트 역할 목록을 조회합니다.

#### 요청

```http
GET /api/roles/project
```

#### 응답

**성공 (200 OK)**
```json
[
  {
    "id": 3,
    "name": "PROJECT_ADMIN",
    "description": "프로젝트 관리자",
    "scope": "PROJECT",
    "created_at": "2025-01-27T10:00:00Z"
  },
  {
    "id": 4,
    "name": "PROJECT_MEMBER",
    "description": "프로젝트 멤버",
    "scope": "PROJECT",
    "created_at": "2025-01-27T10:00:00Z"
  }
]
```

## 📝 사용 예시

### 1. 프로젝트 멤버 관리 화면

```javascript
// 1. 프로젝트 멤버 목록 조회
const getProjectMembers = async (projectId, page = 1, pageSize = 20) => {
  const response = await fetch(`/api/projects/${projectId}/users?page=${page}&page_size=${pageSize}`);
  return await response.json();
};

// 2. 사용자 역할 변경
const assignUserRole = async (projectId, userId, roleId) => {
  const response = await fetch(`/api/projects/${projectId}/users/${userId}/role`, {
    method: 'PUT',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({ role_id: roleId }),
  });
  return await response.json();
};

// 3. 역할 목록 조회
const getProjectRoles = async () => {
  const response = await fetch('/api/roles/project');
  return await response.json();
};
```

### 2. 사용자 프로젝트 관리 화면

```javascript
// 1. 사용자 프로젝트 목록 조회
const getUserProjects = async (userId, page = 1, pageSize = 20) => {
  const response = await fetch(`/api/users/${userId}/projects?page=${page}&page_size=${pageSize}`);
  return await response.json();
};

// 2. 일괄 역할 할당
const batchAssignRoles = async (projectId, assignments) => {
  const response = await fetch(`/api/projects/${projectId}/users/roles`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({ assignments }),
  });
  return await response.json();
};
```

## ⚠️ 주의사항

### 1. 역할 범위 (Scope)

- **GLOBAL**: 시스템 전체에 적용되는 역할
- **PROJECT**: 특정 프로젝트에만 적용되는 역할

### 2. 페이지네이션

- `page`는 1부터 시작합니다
- `page_size`의 최대값은 100입니다
- 페이지네이션 정보는 `pagination` 객체에 포함됩니다

### 3. 에러 처리

모든 API는 일관된 에러 응답 형식을 사용합니다:

```json
{
  "error": "에러 메시지"
}
```

### 4. 인증 및 권한

현재 구현에서는 인증이 필요하지 않지만, 실제 운영 환경에서는 적절한 인증 및 권한 검사가 필요합니다.

## 🔄 API 사용 흐름

### 1. 프로젝트 멤버 관리

1. **프로젝트 멤버 목록 조회**: `GET /api/projects/{project_id}/users`
2. **역할 목록 조회**: `GET /api/roles/project`
3. **사용자 역할 할당**: `PUT /api/projects/{project_id}/users/{user_id}/role`
4. **사용자 역할 제거**: `DELETE /api/projects/{project_id}/users/{user_id}/role`

### 2. 사용자 프로젝트 관리

1. **사용자 프로젝트 목록 조회**: `GET /api/users/{user_id}/projects`
2. **역할 목록 조회**: `GET /api/roles/project`
3. **일괄 역할 할당**: `POST /api/projects/{project_id}/users/roles`

## 📚 관련 문서

- [Role 관리 API](./role-management-api.md)
- [Capability 관리 API](./capability-management-api.md)
- [Role-Capability Matrix API](./role-capability-matrix-api-korean.md)

---

**마지막 업데이트**: 2025-01-27  
**문서 버전**: 1.0  
**작성자**: AI Assistant
