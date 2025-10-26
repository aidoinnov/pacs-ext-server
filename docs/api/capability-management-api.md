# Capability 관리 API 문서

## 📋 개요

Capability 생성, 조회, 수정, 삭제를 위한 API 엔드포인트입니다.

## 🔗 기본 URL

```
http://localhost:8080/api
```

## 📊 데이터 모델

### Capability 구조

```typescript
interface Capability {
  id: number;                    // Capability ID
  name: string;                  // 내부 이름 (예: "MANAGE_USERS")
  display_name: string;          // UI 표시 이름 (예: "사용자 관리")
  display_label: string;         // UI 표시용 짧은 레이블 (예: "Users")
  description?: string;          // 설명
  category: string;              // 카테고리 (예: "관리")
  category_label: string;        // UI 카테고리 짧은 레이블 (예: "MANAGE")
  is_active: boolean;            // 활성화 여부
  created_at: string;            // 생성일시 (ISO 8601)
  updated_at: string;            // 수정일시 (ISO 8601)
}
```

### CreateCapabilityRequest

```typescript
interface CreateCapabilityRequest {
  name: string;                  // 내부 이름 (필수, 최대 100자)
  display_name: string;          // UI 표시 이름 (필수, 최대 200자)
  description?: string;          // 설명 (선택, 최대 500자)
  category: string;              // 카테고리 (필수, 최대 50자)
}
```

### UpdateCapabilityRequest

```typescript
interface UpdateCapabilityRequest {
  display_name?: string;         // UI 표시 이름 (선택, 최대 200자)
  description?: string;          // 설명 (선택, 최대 500자)
  category?: string;             // 카테고리 (선택, 최대 50자)
  is_active?: boolean;           // 활성화 여부 (선택)
}
```

## 🚀 API 엔드포인트

### 1. Capability 생성

**POST** `/api/capabilities`

새로운 Capability를 생성합니다.

#### 요청

```http
POST /api/capabilities
Content-Type: application/json

{
  "name": "CUSTOM_MANAGE",
  "display_name": "커스텀 관리",
  "description": "사용자 정의 관리 기능",
  "category": "관리"
}
```

#### 응답

**성공 (201 Created)**
```json
{
  "id": 21,
  "name": "CUSTOM_MANAGE",
  "display_name": "커스텀 관리",
  "display_label": "Custom",
  "description": "사용자 정의 관리 기능",
  "category": "관리",
  "category_label": "MANAGE",
  "is_active": true,
  "created_at": "2025-01-27T10:30:00Z",
  "updated_at": "2025-01-27T10:30:00Z"
}
```

**실패 (400 Bad Request)**
```json
{
  "error": "Validation error",
  "message": "Capability name already exists"
}
```

#### 에러 코드

| 상태 코드 | 설명 |
|-----------|------|
| 400 | 잘못된 요청 (이름 중복, 유효성 검사 실패) |
| 500 | 서버 내부 오류 |

### 2. Capability 조회

**GET** `/api/capabilities/{capability_id}`

특정 Capability의 상세 정보를 조회합니다.

#### 요청

```http
GET /api/capabilities/1
```

#### 응답

**성공 (200 OK)**
```json
{
  "capability": {
    "id": 1,
    "name": "SYSTEM_ADMIN",
    "display_name": "시스템 관리자",
    "display_label": "Admin",
    "description": "시스템 전체 관리 권한",
    "category": "관리",
    "category_label": "MANAGE",
    "is_active": true,
    "created_at": "2025-01-27T10:30:00Z",
    "updated_at": "2025-01-27T10:30:00Z"
  },
  "permissions": [
    {
      "id": 1,
      "category": "사용자 및 권한 관리",
      "resource_type": "USER",
      "action": "CREATE"
    }
  ]
}
```

**실패 (404 Not Found)**
```json
{
  "error": "Not found",
  "message": "Capability with id 999 not found"
}
```

### 3. Capability 수정

**PUT** `/api/capabilities/{capability_id}`

Capability 정보를 수정합니다.

#### 요청

```http
PUT /api/capabilities/1
Content-Type: application/json

{
  "display_name": "업데이트된 시스템 관리자",
  "description": "수정된 시스템 전체 관리 권한",
  "is_active": true
}
```

#### 응답

**성공 (200 OK)**
```json
{
  "id": 1,
  "name": "SYSTEM_ADMIN",
  "display_name": "업데이트된 시스템 관리자",
  "display_label": "Admin",
  "description": "수정된 시스템 전체 관리 권한",
  "category": "관리",
  "category_label": "MANAGE",
  "is_active": true,
  "created_at": "2025-01-27T10:30:00Z",
  "updated_at": "2025-01-27T10:35:00Z"
}
```

**실패 (404 Not Found)**
```json
{
  "error": "Not found",
  "message": "Capability with id 999 not found"
}
```

### 4. Capability 삭제

**DELETE** `/api/capabilities/{capability_id}`

Capability를 삭제합니다.

#### 요청

```http
DELETE /api/capabilities/1
```

#### 응답

**성공 (200 OK)**
```json
{
  "message": "Capability deleted successfully"
}
```

**실패 (404 Not Found)**
```json
{
  "error": "Not found",
  "message": "Capability with id 999 not found"
}
```

**실패 (400 Bad Request)**
```json
{
  "error": "Validation error",
  "message": "Cannot delete capability that is assigned to roles"
}
```

### 5. 모든 Capability 목록 조회

**GET** `/api/capabilities`

모든 Capability 목록을 조회합니다.

#### 요청

```http
GET /api/capabilities
```

#### 응답

**성공 (200 OK)**
```json
[
  {
    "id": 1,
    "name": "SYSTEM_ADMIN",
    "display_name": "시스템 관리자",
    "display_label": "Admin",
    "description": "시스템 전체 관리 권한",
    "category": "관리",
    "category_label": "MANAGE",
    "permission_count": 15,
    "is_active": true,
    "created_at": "2025-01-27T10:30:00Z",
    "updated_at": "2025-01-27T10:30:00Z"
  },
  {
    "id": 2,
    "name": "USER_MANAGEMENT",
    "display_name": "사용자 관리",
    "display_label": "Users",
    "description": "사용자 계정 관리 권한",
    "category": "관리",
    "category_label": "MANAGE",
    "permission_count": 4,
    "is_active": true,
    "created_at": "2025-01-27T10:30:00Z",
    "updated_at": "2025-01-27T10:30:00Z"
  }
]
```

### 6. 카테고리별 Capability 목록 조회

**GET** `/api/capabilities/category/{category}`

특정 카테고리의 Capability 목록을 조회합니다.

#### 요청

```http
GET /api/capabilities/category/관리
```

#### 응답

**성공 (200 OK)**
```json
[
  {
    "id": 1,
    "name": "SYSTEM_ADMIN",
    "display_name": "시스템 관리자",
    "display_label": "Admin",
    "description": "시스템 전체 관리 권한",
    "category": "관리",
    "category_label": "MANAGE",
    "permission_count": 15,
    "is_active": true,
    "created_at": "2025-01-27T10:30:00Z",
    "updated_at": "2025-01-27T10:30:00Z"
  }
]
```

## 🔧 사용 예시

### JavaScript/TypeScript

```javascript
// 1. Capability 생성
const createCapability = async (capabilityData) => {
  const response = await fetch('/api/capabilities', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(capabilityData)
  });
  
  if (!response.ok) {
    const error = await response.json();
    throw new Error(error.message || error.error);
  }
  
  return await response.json();
};

// 2. Capability 조회
const getCapability = async (capabilityId) => {
  const response = await fetch(`/api/capabilities/${capabilityId}`);
  
  if (!response.ok) {
    const error = await response.json();
    throw new Error(error.message || error.error);
  }
  
  return await response.json();
};

// 3. Capability 수정
const updateCapability = async (capabilityId, updateData) => {
  const response = await fetch(`/api/capabilities/${capabilityId}`, {
    method: 'PUT',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(updateData)
  });
  
  if (!response.ok) {
    const error = await response.json();
    throw new Error(error.message || error.error);
  }
  
  return await response.json();
};

// 4. Capability 삭제
const deleteCapability = async (capabilityId) => {
  const response = await fetch(`/api/capabilities/${capabilityId}`, {
    method: 'DELETE'
  });
  
  if (!response.ok) {
    const error = await response.json();
    throw new Error(error.message || error.error);
  }
  
  return await response.json();
};

// 5. 모든 Capability 목록 조회
const getAllCapabilities = async () => {
  const response = await fetch('/api/capabilities');
  
  if (!response.ok) {
    const error = await response.json();
    throw new Error(error.message || error.error);
  }
  
  return await response.json();
};

// 6. 카테고리별 Capability 목록 조회
const getCapabilitiesByCategory = async (category) => {
  const response = await fetch(`/api/capabilities/category/${encodeURIComponent(category)}`);
  
  if (!response.ok) {
    const error = await response.json();
    throw new Error(error.message || error.error);
  }
  
  return await response.json();
};

// 사용 예시
try {
  // 새 Capability 생성
  const newCapability = await createCapability({
    name: 'CUSTOM_MANAGE',
    display_name: '커스텀 관리',
    description: '사용자 정의 관리 기능',
    category: '관리'
  });
  console.log('Created capability:', newCapability);
  
  // Capability 조회
  const capability = await getCapability(newCapability.id);
  console.log('Retrieved capability:', capability);
  
  // Capability 수정
  const updatedCapability = await updateCapability(newCapability.id, {
    display_name: '업데이트된 커스텀 관리',
    description: '수정된 사용자 정의 관리 기능'
  });
  console.log('Updated capability:', updatedCapability);
  
  // 모든 Capability 목록 조회
  const allCapabilities = await getAllCapabilities();
  console.log('All capabilities:', allCapabilities);
  
  // 카테고리별 Capability 목록 조회
  const manageCapabilities = await getCapabilitiesByCategory('관리');
  console.log('Manage capabilities:', manageCapabilities);
  
  // Capability 삭제
  await deleteCapability(newCapability.id);
  console.log('Capability deleted successfully');
  
} catch (error) {
  console.error('Error:', error.message);
}
```

### cURL

```bash
# 1. Capability 생성
curl -X POST http://localhost:8080/api/capabilities \
  -H "Content-Type: application/json" \
  -d '{
    "name": "CUSTOM_MANAGE",
    "display_name": "커스텀 관리",
    "description": "사용자 정의 관리 기능",
    "category": "관리"
  }'

# 2. Capability 조회
curl http://localhost:8080/api/capabilities/1

# 3. Capability 수정
curl -X PUT http://localhost:8080/api/capabilities/1 \
  -H "Content-Type: application/json" \
  -d '{
    "display_name": "업데이트된 커스텀 관리",
    "description": "수정된 사용자 정의 관리 기능"
  }'

# 4. Capability 삭제
curl -X DELETE http://localhost:8080/api/capabilities/1

# 5. 모든 Capability 목록 조회
curl http://localhost:8080/api/capabilities

# 6. 카테고리별 Capability 목록 조회
curl "http://localhost:8080/api/capabilities/category/관리"
```

## ⚠️ 주의사항

### 1. Capability 이름 규칙

- **필수**: Capability 이름은 비어있을 수 없습니다
- **길이 제한**: 최대 100자
- **유일성**: 같은 이름의 Capability는 존재할 수 없습니다
- **대소문자 구분**: Capability 이름은 대소문자를 구분합니다

### 2. 카테고리 규칙

- **필수**: 카테고리는 비어있을 수 없습니다
- **길이 제한**: 최대 50자
- **기존 카테고리**: "관리", "프로젝트", "DICOM 데이터 관리", "어노테이션 관리", "마스크 관리", "행잉 프로토콜 관리"

### 3. 삭제 제한사항

- **역할 할당된 Capability**: 역할에 할당된 Capability는 삭제할 수 없습니다
- **시스템 기본 Capability**: 시스템에서 기본으로 제공하는 Capability는 삭제하지 않는 것을 권장합니다

### 4. 에러 처리

모든 API는 일관된 에러 응답 형식을 사용합니다:

```json
{
  "error": "에러 타입",
  "message": "상세 에러 메시지"
}
```

### 5. 인증 및 권한

현재 구현에서는 인증이 필요하지 않지만, 실제 운영 환경에서는 적절한 인증 및 권한 검사가 필요합니다.

## 🔄 향후 계획

### Capability-Permission 매핑 API 추가 예정

현재 Capability와 Permission 간의 매핑을 관리하는 API가 구현되지 않았습니다. 향후 다음 기능이 추가될 예정입니다:

- **POST** `/api/capabilities/{capability_id}/permissions` - Capability에 Permission 추가
- **DELETE** `/api/capabilities/{capability_id}/permissions/{permission_id}` - Capability에서 Permission 제거

### 예상 매핑 API

```http
POST /api/capabilities/1/permissions
Content-Type: application/json

{
  "permission_id": 5
}
```

## 📚 관련 문서

- [Role 관리 API](./role-management-api.md)
- [Role-Capability Matrix API](./role-capability-matrix-api.md)
- [프론트엔드 API 변경사항](./frontend-api-changes-capability-labels.md)

---

**마지막 업데이트**: 2025-01-27  
**문서 버전**: 1.0  
**작성자**: AI Assistant
