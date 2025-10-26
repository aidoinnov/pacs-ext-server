# Role 관리 API 문서

## 📋 개요

역할(Role) 생성, 조회, 수정, 삭제를 위한 API 엔드포인트입니다.

## 🔗 기본 URL

```
http://localhost:8080/api
```

## 📊 데이터 모델

### Role 구조

```typescript
interface Role {
  id: number;                    // 역할 ID
  name: string;                  // 역할 이름 (예: "SUPER_ADMIN", "ADMIN")
  description?: string;          // 역할 설명 (예: "시스템 전체 관리자", "관리자")
  scope: string;                 // 역할 범위 ("GLOBAL" | "PROJECT")
  created_at: string;            // 생성일시 (ISO 8601)
}
```

### CreateRoleRequest

```typescript
interface CreateRoleRequest {
  name: string;                  // 역할 이름 (필수, 최대 100자)
  scope: string;                 // 역할 범위 ("GLOBAL" | "PROJECT")
  description?: string;          // 역할 설명 (선택)
}
```

### UpdateRoleRequest

```typescript
interface UpdateRoleRequest {
  name?: string;                 // 역할 이름 (선택, 최대 100자)
  description?: string;          // 역할 설명 (선택)
  scope?: string;                // 역할 범위 ("GLOBAL" | "PROJECT")
}
```

## 🚀 API 엔드포인트

### 1. 역할 생성

**POST** `/api/roles`

새로운 역할을 생성합니다.

#### 요청

```http
POST /api/roles
Content-Type: application/json

{
  "name": "CUSTOM_ADMIN",
  "scope": "GLOBAL",
  "description": "커스텀 관리자 역할"
}
```

#### 응답

**성공 (201 Created)**
```json
{
  "id": 6,
  "name": "CUSTOM_ADMIN",
  "description": "커스텀 관리자 역할",
  "scope": "GLOBAL",
  "created_at": "2025-01-27T10:30:00Z"
}
```

**실패 (400 Bad Request)**
```json
{
  "error": "Failed to create role: Role name already exists"
}
```

#### 에러 코드

| 상태 코드 | 설명 |
|-----------|------|
| 400 | 잘못된 요청 (이름 중복, 유효성 검사 실패) |
| 500 | 서버 내부 오류 |

### 2. 역할 조회

**GET** `/api/roles/{role_id}`

특정 역할의 상세 정보를 조회합니다.

#### 요청

```http
GET /api/roles/1
```

#### 응답

**성공 (200 OK)**
```json
{
  "id": 1,
  "name": "SUPER_ADMIN",
  "description": "시스템 전체 관리자",
  "scope": "GLOBAL",
  "created_at": "2025-01-27T10:30:00Z"
}
```

**실패 (404 Not Found)**
```json
{
  "error": "Role not found: Role with id 999 not found"
}
```

### 3. 전역 역할 목록 조회

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
    "created_at": "2025-01-27T10:30:00Z"
  },
  {
    "id": 2,
    "name": "ADMIN",
    "description": "관리자",
    "scope": "GLOBAL",
    "created_at": "2025-01-27T10:30:00Z"
  }
]
```

### 4. 프로젝트 역할 목록 조회

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
    "created_at": "2025-01-27T10:30:00Z"
  }
]
```

### 5. 전역 역할 목록 조회 (권한 정보 포함)

**GET** `/api/roles/global/with-permissions`

전역 역할 목록을 권한 정보와 함께 페이지네이션으로 조회합니다.

#### 요청

```http
GET /api/roles/global/with-permissions?page=1&page_size=10
```

#### 쿼리 파라미터

| 파라미터 | 타입 | 기본값 | 설명 |
|----------|------|--------|------|
| page | integer | 1 | 페이지 번호 (1부터 시작) |
| page_size | integer | 10 | 페이지 크기 |

#### 응답

**성공 (200 OK)**
```json
{
  "roles": [
    {
      "id": 1,
      "name": "SUPER_ADMIN",
      "description": "시스템 전체 관리자",
      "scope": "GLOBAL",
      "created_at": "2025-01-27T10:30:00Z"
    }
  ],
  "total": 5,
  "page": 1,
  "page_size": 10,
  "total_pages": 1
}
```

## 🔧 사용 예시

### JavaScript/TypeScript

```javascript
// 1. 역할 생성
const createRole = async (roleData) => {
  const response = await fetch('/api/roles', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(roleData)
  });
  
  if (!response.ok) {
    const error = await response.json();
    throw new Error(error.error);
  }
  
  return await response.json();
};

// 2. 역할 조회
const getRole = async (roleId) => {
  const response = await fetch(`/api/roles/${roleId}`);
  
  if (!response.ok) {
    const error = await response.json();
    throw new Error(error.error);
  }
  
  return await response.json();
};

// 3. 전역 역할 목록 조회
const getGlobalRoles = async () => {
  const response = await fetch('/api/roles/global');
  
  if (!response.ok) {
    const error = await response.json();
    throw new Error(error.error);
  }
  
  return await response.json();
};

// 사용 예시
try {
  // 새 역할 생성
  const newRole = await createRole({
    name: 'CUSTOM_ADMIN',
    scope: 'GLOBAL',
    description: '커스텀 관리자 역할'
  });
  console.log('Created role:', newRole);
  
  // 역할 조회
  const role = await getRole(newRole.id);
  console.log('Retrieved role:', role);
  
  // 전역 역할 목록 조회
  const globalRoles = await getGlobalRoles();
  console.log('Global roles:', globalRoles);
  
} catch (error) {
  console.error('Error:', error.message);
}
```

### cURL

```bash
# 1. 역할 생성
curl -X POST http://localhost:8080/api/roles \
  -H "Content-Type: application/json" \
  -d '{
    "name": "CUSTOM_ADMIN",
    "scope": "GLOBAL",
    "description": "커스텀 관리자 역할"
  }'

# 2. 역할 조회
curl http://localhost:8080/api/roles/1

# 3. 전역 역할 목록 조회
curl http://localhost:8080/api/roles/global

# 4. 프로젝트 역할 목록 조회
curl http://localhost:8080/api/roles/project

# 5. 전역 역할 목록 조회 (권한 정보 포함)
curl "http://localhost:8080/api/roles/global/with-permissions?page=1&page_size=10"
```

## ⚠️ 주의사항

### 1. 역할 이름 규칙

- **필수**: 역할 이름은 비어있을 수 없습니다
- **길이 제한**: 최대 100자
- **유일성**: 같은 이름의 역할은 존재할 수 없습니다
- **대소문자 구분**: 역할 이름은 대소문자를 구분합니다

### 2. 역할 범위 (Scope)

- **GLOBAL**: 시스템 전체에 적용되는 역할
- **PROJECT**: 특정 프로젝트에만 적용되는 역할

### 3. 에러 처리

모든 API는 일관된 에러 응답 형식을 사용합니다:

```json
{
  "error": "에러 메시지"
}
```

### 4. 인증 및 권한

현재 구현에서는 인증이 필요하지 않지만, 실제 운영 환경에서는 적절한 인증 및 권한 검사가 필요합니다.

## 🔄 향후 계획

### Role 업데이트 API 추가 예정

현재 Role 업데이트 API가 구현되지 않았습니다. 향후 다음 기능이 추가될 예정입니다:

- **PUT** `/api/roles/{role_id}` - 역할 정보 수정
- **DELETE** `/api/roles/{role_id}` - 역할 삭제

### 예상 업데이트 API

```http
PUT /api/roles/1
Content-Type: application/json

{
  "name": "UPDATED_ADMIN",
  "description": "업데이트된 관리자 역할",
  "scope": "GLOBAL"
}
```

## 📚 관련 문서

- [Capability 관리 API](./capability-management-api.md)
- [Role-Capability Matrix API](./role-capability-matrix-api.md)
- [프론트엔드 API 변경사항](./frontend-api-changes-capability-labels.md)

---

**마지막 업데이트**: 2025-01-27  
**문서 버전**: 1.0  
**작성자**: AI Assistant
