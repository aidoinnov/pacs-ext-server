# Role-Permission Matrix API 문서

## 📋 개요

Role-Permission Matrix API는 역할과 권한 간의 관계를 매트릭스 형태로 조회하고 관리할 수 있는 RESTful API입니다. 이 API를 통해 사용자는 역할별로 할당된 권한을 시각적으로 확인하고, 개별 권한을 ON/OFF할 수 있습니다.

## 🔗 기본 정보

- **Base URL**: `http://localhost:8080/api`
- **Content-Type**: `application/json`
- **인증**: JWT Bearer Token
- **문서**: Swagger UI (`http://localhost:8080/swagger-ui/`)

## 📚 API 엔드포인트

### 1. 글로벌 역할-권한 매트릭스 조회

**엔드포인트**: `GET /api/roles/global/permissions/matrix`

**설명**: 글로벌 범위의 모든 역할과 권한 간의 관계를 매트릭스 형태로 조회합니다.

**요청**:
```http
GET /api/roles/global/permissions/matrix
Authorization: Bearer <jwt-token>
```

**응답**:
```json
{
  "roles": [
    {
      "id": 1,
      "name": "Admin",
      "description": "Administrator role",
      "scope": "GLOBAL"
    },
    {
      "id": 2,
      "name": "User",
      "description": "Regular user role",
      "scope": "GLOBAL"
    }
  ],
  "permissions_by_category": {
    "USER": [
      {
        "id": 1,
        "resource_type": "USER",
        "action": "CREATE"
      },
      {
        "id": 2,
        "resource_type": "USER",
        "action": "READ"
      },
      {
        "id": 3,
        "resource_type": "USER",
        "action": "UPDATE"
      },
      {
        "id": 4,
        "resource_type": "USER",
        "action": "DELETE"
      }
    ],
    "PROJECT": [
      {
        "id": 5,
        "resource_type": "PROJECT",
        "action": "CREATE"
      },
      {
        "id": 6,
        "resource_type": "PROJECT",
        "action": "READ"
      },
      {
        "id": 7,
        "resource_type": "PROJECT",
        "action": "UPDATE"
      },
      {
        "id": 8,
        "resource_type": "PROJECT",
        "action": "DELETE"
      }
    ],
    "ANNOTATION": [
      {
        "id": 9,
        "resource_type": "ANNOTATION",
        "action": "CREATE"
      },
      {
        "id": 10,
        "resource_type": "ANNOTATION",
        "action": "READ"
      },
      {
        "id": 11,
        "resource_type": "ANNOTATION",
        "action": "UPDATE"
      },
      {
        "id": 12,
        "resource_type": "ANNOTATION",
        "action": "DELETE"
      }
    ]
  },
  "assignments": [
    {
      "role_id": 1,
      "permission_id": 1,
      "assigned": true
    },
    {
      "role_id": 1,
      "permission_id": 2,
      "assigned": true
    },
    {
      "role_id": 1,
      "permission_id": 3,
      "assigned": true
    },
    {
      "role_id": 1,
      "permission_id": 4,
      "assigned": true
    },
    {
      "role_id": 1,
      "permission_id": 5,
      "assigned": true
    },
    {
      "role_id": 1,
      "permission_id": 6,
      "assigned": true
    },
    {
      "role_id": 1,
      "permission_id": 7,
      "assigned": true
    },
    {
      "role_id": 1,
      "permission_id": 8,
      "assigned": true
    },
    {
      "role_id": 1,
      "permission_id": 9,
      "assigned": true
    },
    {
      "role_id": 1,
      "permission_id": 10,
      "assigned": true
    },
    {
      "role_id": 1,
      "permission_id": 11,
      "assigned": true
    },
    {
      "role_id": 1,
      "permission_id": 12,
      "assigned": true
    },
    {
      "role_id": 2,
      "permission_id": 1,
      "assigned": false
    },
    {
      "role_id": 2,
      "permission_id": 2,
      "assigned": true
    },
    {
      "role_id": 2,
      "permission_id": 3,
      "assigned": false
    },
    {
      "role_id": 2,
      "permission_id": 4,
      "assigned": false
    },
    {
      "role_id": 2,
      "permission_id": 5,
      "assigned": false
    },
    {
      "role_id": 2,
      "permission_id": 6,
      "assigned": true
    },
    {
      "role_id": 2,
      "permission_id": 7,
      "assigned": false
    },
    {
      "role_id": 2,
      "permission_id": 8,
      "assigned": false
    },
    {
      "role_id": 2,
      "permission_id": 9,
      "assigned": false
    },
    {
      "role_id": 2,
      "permission_id": 10,
      "assigned": true
    },
    {
      "role_id": 2,
      "permission_id": 11,
      "assigned": false
    },
    {
      "role_id": 2,
      "permission_id": 12,
      "assigned": false
    }
  ]
}
```

**상태 코드**:
- `200 OK`: 성공
- `401 Unauthorized`: 인증 실패
- `500 Internal Server Error`: 서버 오류

### 2. 프로젝트별 역할-권한 매트릭스 조회

**엔드포인트**: `GET /api/projects/{project_id}/roles/permissions/matrix`

**설명**: 특정 프로젝트에 할당된 역할과 권한 간의 관계를 매트릭스 형태로 조회합니다.

**경로 매개변수**:
- `project_id` (integer, required): 프로젝트 ID

**요청**:
```http
GET /api/projects/1/roles/permissions/matrix
Authorization: Bearer <jwt-token>
```

**응답**: 글로벌 매트릭스와 동일한 구조

**상태 코드**:
- `200 OK`: 성공
- `401 Unauthorized`: 인증 실패
- `404 Not Found`: 프로젝트를 찾을 수 없음
- `500 Internal Server Error`: 서버 오류

### 3. 글로벌 역할에 권한 할당/제거

**엔드포인트**: `PUT /api/roles/{role_id}/permissions/{permission_id}`

**설명**: 글로벌 역할에 특정 권한을 할당하거나 제거합니다.

**경로 매개변수**:
- `role_id` (integer, required): 역할 ID
- `permission_id` (integer, required): 권한 ID

**요청 본문**:
```json
{
  "assign": true
}
```

**요청 예시**:
```http
PUT /api/roles/1/permissions/5
Authorization: Bearer <jwt-token>
Content-Type: application/json

{
  "assign": true
}
```

**응답**:
```json
{
  "success": true,
  "message": "Permission assigned successfully"
}
```

**상태 코드**:
- `200 OK`: 성공
- `400 Bad Request`: 잘못된 요청
- `401 Unauthorized`: 인증 실패
- `404 Not Found`: 역할 또는 권한을 찾을 수 없음
- `409 Conflict`: 이미 할당된 권한
- `500 Internal Server Error`: 서버 오류

### 4. 프로젝트별 역할에 권한 할당/제거

**엔드포인트**: `PUT /api/projects/{project_id}/roles/{role_id}/permissions/{permission_id}`

**설명**: 프로젝트별 역할에 특정 권한을 할당하거나 제거합니다.

**경로 매개변수**:
- `project_id` (integer, required): 프로젝트 ID
- `role_id` (integer, required): 역할 ID
- `permission_id` (integer, required): 권한 ID

**요청 본문**:
```json
{
  "assign": false
}
```

**요청 예시**:
```http
PUT /api/projects/1/roles/2/permissions/6
Authorization: Bearer <jwt-token>
Content-Type: application/json

{
  "assign": false
}
```

**응답**:
```json
{
  "success": true,
  "message": "Permission removed successfully"
}
```

**상태 코드**:
- `200 OK`: 성공
- `400 Bad Request`: 잘못된 요청
- `401 Unauthorized`: 인증 실패
- `404 Not Found`: 프로젝트, 역할 또는 권한을 찾을 수 없음
- `409 Conflict`: 이미 할당된 권한
- `500 Internal Server Error`: 서버 오류

## 📊 데이터 모델

### RoleInfo
```json
{
  "id": 1,
  "name": "Admin",
  "description": "Administrator role",
  "scope": "GLOBAL"
}
```

**필드 설명**:
- `id`: 역할 ID (integer)
- `name`: 역할 이름 (string)
- `description`: 역할 설명 (string, optional)
- `scope`: 역할 범위 (string, "GLOBAL" 또는 "PROJECT")

### PermissionInfo
```json
{
  "id": 1,
  "resource_type": "USER",
  "action": "CREATE"
}
```

**필드 설명**:
- `id`: 권한 ID (integer)
- `resource_type`: 리소스 타입/카테고리 (string)
- `action`: 액션 (string)

### RolePermissionAssignment
```json
{
  "role_id": 1,
  "permission_id": 2,
  "assigned": true
}
```

**필드 설명**:
- `role_id`: 역할 ID (integer)
- `permission_id`: 권한 ID (integer)
- `assigned`: 할당 여부 (boolean)

### AssignPermissionRequest
```json
{
  "assign": true
}
```

**필드 설명**:
- `assign`: 할당 여부 (boolean, true: 할당, false: 제거)

### AssignPermissionResponse
```json
{
  "success": true,
  "message": "Permission assigned successfully"
}
```

**필드 설명**:
- `success`: 성공 여부 (boolean)
- `message`: 메시지 (string)

## 🔒 인증 및 권한

### JWT 토큰
모든 API 요청에는 유효한 JWT 토큰이 필요합니다.

```http
Authorization: Bearer <jwt-token>
```

### 권한 요구사항
- **매트릭스 조회**: 인증된 사용자
- **권한 할당/제거**: 관리자 권한 필요

## 🚨 에러 처리

### 에러 응답 형식
```json
{
  "error": "Error Type",
  "message": "Error description"
}
```

### 일반적인 에러

#### 400 Bad Request
```json
{
  "error": "Validation Error",
  "message": "Invalid request data"
}
```

#### 401 Unauthorized
```json
{
  "error": "Unauthorized",
  "message": "Invalid or expired token"
}
```

#### 404 Not Found
```json
{
  "error": "Not Found",
  "message": "Resource not found"
}
```

#### 409 Conflict
```json
{
  "error": "Already Exists",
  "message": "Permission already assigned"
}
```

#### 500 Internal Server Error
```json
{
  "error": "Database Error",
  "message": "Database connection failed"
}
```

## 🧪 테스트 예시

### cURL 명령어

#### 글로벌 매트릭스 조회
```bash
curl -X GET "http://localhost:8080/api/roles/global/permissions/matrix" \
  -H "Authorization: Bearer <jwt-token>" \
  -H "Content-Type: application/json"
```

#### 프로젝트별 매트릭스 조회
```bash
curl -X GET "http://localhost:8080/api/projects/1/roles/permissions/matrix" \
  -H "Authorization: Bearer <jwt-token>" \
  -H "Content-Type: application/json"
```

#### 권한 할당
```bash
curl -X PUT "http://localhost:8080/api/roles/1/permissions/5" \
  -H "Authorization: Bearer <jwt-token>" \
  -H "Content-Type: application/json" \
  -d '{"assign": true}'
```

#### 권한 제거
```bash
curl -X PUT "http://localhost:8080/api/projects/1/roles/2/permissions/6" \
  -H "Authorization: Bearer <jwt-token>" \
  -H "Content-Type: application/json" \
  -d '{"assign": false}'
```

### JavaScript 예시

```javascript
// 글로벌 매트릭스 조회
const response = await fetch('/api/roles/global/permissions/matrix', {
  method: 'GET',
  headers: {
    'Authorization': 'Bearer <jwt-token>',
    'Content-Type': 'application/json'
  }
});
const matrix = await response.json();

// 권한 할당
const assignResponse = await fetch('/api/roles/1/permissions/5', {
  method: 'PUT',
  headers: {
    'Authorization': 'Bearer <jwt-token>',
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({ assign: true })
});
const result = await assignResponse.json();
```

## 📈 성능 고려사항

### 응답 시간
- **매트릭스 조회**: 평균 100ms 이하
- **권한 할당/제거**: 평균 50ms 이하

### 데이터 크기
- **작은 매트릭스** (10 역할, 20 권한): ~5KB
- **중간 매트릭스** (50 역할, 100 권한): ~50KB
- **큰 매트릭스** (200 역할, 500 권한): ~500KB

### 최적화 팁
- 클라이언트에서 캐싱 활용
- 필요한 경우에만 매트릭스 조회
- 배치 작업 시 개별 요청보다는 여러 권한을 한 번에 처리

## 🔧 문제 해결

### 일반적인 문제

#### 1. 401 Unauthorized
- JWT 토큰이 유효한지 확인
- 토큰이 만료되었는지 확인
- Authorization 헤더 형식 확인

#### 2. 404 Not Found
- 역할 ID가 존재하는지 확인
- 권한 ID가 존재하는지 확인
- 프로젝트 ID가 존재하는지 확인

#### 3. 409 Conflict
- 권한이 이미 할당되어 있는지 확인
- 중복 요청을 방지

#### 4. 500 Internal Server Error
- 서버 로그 확인
- 데이터베이스 연결 상태 확인
- 네트워크 상태 확인

### 디버깅 팁

#### 로그 확인
```bash
# 서버 로그 확인
tail -f logs/pacs-server.log

# 데이터베이스 쿼리 확인
psql -d pacs_db -c "SELECT * FROM security_role_permission;"
```

#### API 테스트
```bash
# 헬스 체크
curl http://localhost:8080/health

# Swagger UI 확인
open http://localhost:8080/swagger-ui/
```
