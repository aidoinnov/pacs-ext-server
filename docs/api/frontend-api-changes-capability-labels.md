# 🚀 API 변경사항 - 프론트엔드 개발팀 전달사항

## 📋 변경 개요

`security_capability` 테이블에 UI 레이블 필드가 추가되어, Capability 관련 API 응답에 새로운 필드가 포함됩니다.

**변경일**: 2025-10-25  
**영향 범위**: Capability 관련 모든 API 엔드포인트  
**호환성**: 하위 호환성 유지 (기존 필드 유지 + 새 필드 추가)

## 🔄 변경된 API 엔드포인트

### 1. Role-Capability Matrix API
```
GET /api/roles/global/capabilities/matrix
GET /api/roles/global/capabilities/matrix?page=1&size=10&search=admin
```

### 2. Capability 목록 API
```
GET /api/capabilities
GET /api/capabilities/{capability_id}
GET /api/capabilities/category/{category}
```

## ✨ 새로 추가된 필드

모든 Capability 관련 API 응답에 다음 2개 필드가 추가됩니다:

```typescript
interface CapabilityInfo {
  id: number;
  name: string;
  display_name: string;
  display_label: string;        // ✨ 새로 추가
  description?: string;
  category: string;
  category_label: string;       // ✨ 새로 추가
  permission_count: number;
}
```

### 필드 설명

| 필드명 | 타입 | 설명 | 예시 |
|--------|------|------|------|
| `display_label` | string | UI 표시용 짧은 레이블 | "Users", "CREATE", "READ" |
| `category_label` | string | UI 카테고리 짧은 레이블 | "MANAGE", "PROJECT", "DICOM" |

## 📊 API 응답 예시

### 이전 응답
```json
{
  "capabilities_by_category": {
    "관리": [
      {
        "id": 36,
        "name": "USER_MANAGEMENT",
        "display_name": "사용자 관리",
        "description": "사용자 계정 생성, 조회, 수정, 삭제 권한",
        "category": "관리",
        "permission_count": 4
      }
    ]
  }
}
```

### 변경된 응답
```json
{
  "capabilities_by_category": {
    "관리": [
      {
        "id": 36,
        "name": "USER_MANAGEMENT",
        "display_name": "사용자 관리",
        "display_label": "Users",        // ✨ 새로 추가
        "description": "사용자 계정 생성, 조회, 수정, 삭제 권한",
        "category": "관리",
        "category_label": "MANAGE",      // ✨ 새로 추가
        "permission_count": 4
      }
    ]
  }
}
```

## 🎨 UI 활용 가이드

### 1. 역할 이름 표시
```javascript
// 역할을 UI에 표시할 때
const getRoleDisplayName = (role) => {
  return role.name;  // name이 "역할 이름"
};

const getRoleDescription = (role) => {
  return role.description;  // description이 "역할 설명"
};

// 사용 예시
const roleName = getRoleDisplayName(role);
// 결과: "SUPER_ADMIN", "ADMIN", "PROJECT_ADMIN" 등

const roleDesc = getRoleDescription(role);
// 결과: "시스템 전체 관리자", "관리자", "프로젝트 관리자" 등
```

### 2. 표 헤더 구성
```javascript
// 카테고리별 그룹화된 표 헤더
const categoryHeaders = capabilities.reduce((acc, cap) => {
  if (!acc[cap.category_label]) {
    acc[cap.category_label] = [];
  }
  acc[cap.category_label].push(cap.display_label);
  return acc;
}, {});

// 결과 예시:
// {
//   "MANAGE": ["Admin", "Users", "Roles", "Projects"],
//   "PROJECT": ["CREATE", "ASSIGN", "EDIT"],
//   "DICOM": ["READ", "WRITE", "DELETE", "SHARE"],
//   "ANNOTATION": ["READ OWN", "READ ALL", "WRITE", "DELETE", "SHARE"],
//   "MASK": ["READ", "WRITE", "DELETE"],
//   "HANGING_PROTOCOL": ["MANAGE"]
// }
```

### 3. 표 셀 렌더링
```javascript
// 각 capability의 표시 레이블
const cellValue = capability.display_label;
const tooltip = `${capability.display_name}: ${capability.description}`;

// 예시:
// cellValue = "Users"
// tooltip = "사용자 관리: 사용자 계정 생성, 조회, 수정, 삭제 권한"
```

### 4. 필터링 및 검색
```javascript
// 카테고리별 필터링
const filteredByCategory = capabilities.filter(cap => 
  cap.category_label === selectedCategory
);

// 레이블로 검색
const searchResults = capabilities.filter(cap => 
  cap.display_label.toLowerCase().includes(searchTerm.toLowerCase())
);

// 카테고리 레이블로 그룹화
const groupedByCategory = capabilities.reduce((acc, cap) => {
  const category = cap.category_label;
  if (!acc[category]) {
    acc[category] = [];
  }
  acc[category].push(cap);
  return acc;
}, {});
```

## 🏷️ 레이블 매핑 테이블

### MANAGE 카테고리
| Capability | display_label | category_label |
|------------|---------------|----------------|
| SYSTEM_ADMIN | Admin | MANAGE |
| USER_MANAGEMENT | Users | MANAGE |
| ROLE_MANAGEMENT | Roles | MANAGE |
| PROJECT_MANAGEMENT | Projects | MANAGE |

### PROJECT 카테고리
| Capability | display_label | category_label |
|------------|---------------|----------------|
| PROJECT_CREATE | CREATE | PROJECT |
| PROJECT_ASSIGN | ASSIGN | PROJECT |
| PROJECT_EDIT | EDIT | PROJECT |

### DICOM 카테고리
| Capability | display_label | category_label |
|------------|---------------|----------------|
| DICOM_READ_ACCESS | READ | DICOM |
| DICOM_WRITE_ACCESS | WRITE | DICOM |
| DICOM_DELETE_ACCESS | DELETE | DICOM |
| DICOM_SHARE_ACCESS | SHARE | DICOM |

### ANNOTATION 카테고리
| Capability | display_label | category_label |
|------------|---------------|----------------|
| ANNOTATION_READ_OWN | READ OWN | ANNOTATION |
| ANNOTATION_READ_ALL | READ ALL | ANNOTATION |
| ANNOTATION_WRITE | WRITE | ANNOTATION |
| ANNOTATION_DELETE | DELETE | ANNOTATION |
| ANNOTATION_SHARE | SHARE | ANNOTATION |

### MASK 카테고리
| Capability | display_label | category_label |
|------------|---------------|----------------|
| MASK_READ | READ | MASK |
| MASK_WRITE | WRITE | MASK |
| MASK_DELETE | DELETE | MASK |

### HANGING_PROTOCOL 카테고리
| Capability | display_label | category_label |
|------------|---------------|----------------|
| HANGING_PROTOCOL_MANAGEMENT | MANAGE | HANGING_PROTOCOL |

## 🔧 구현 예시

### React 컴포넌트 예시
```jsx
import React from 'react';

const RoleCapabilityMatrix = ({ roles, capabilities }) => {
  // 카테고리별로 그룹화
  const groupedCapabilities = capabilities.reduce((acc, cap) => {
    const category = cap.category_label;
    if (!acc[category]) {
      acc[category] = [];
    }
    acc[category].push(cap);
    return acc;
  }, {});

  return (
    <div className="role-capability-matrix">
      <table>
        <thead>
          <tr>
            <th>역할</th>  {/* role.name이 여기에 표시됨 */}
            {Object.keys(groupedCapabilities).map(category => (
              <th key={category}>{category}</th>
            ))}
          </tr>
        </thead>
        <tbody>
          {roles.map(role => (
            <tr key={role.id}>
              <td className="role-name">
                {role.name}  {/* "SUPER_ADMIN", "ADMIN" 등 */}
                <br />
                <small className="role-description">{role.description}</small>
              </td>
              {Object.values(groupedCapabilities).map(caps => (
                <td key={caps[0]?.category_label}>
                  {caps.map(cap => (
                    <span key={cap.id} className="capability-label">
                      {cap.display_label}
                    </span>
                  ))}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
};
```

### Vue 컴포넌트 예시
```vue
<template>
  <div class="capability-matrix">
    <div v-for="(caps, category) in groupedCapabilities" :key="category" class="category-group">
      <h3 class="category-header">{{ category }}</h3>
      <div class="capability-list">
        <div v-for="cap in caps" :key="cap.id" class="capability-item">
          <span class="capability-label">{{ cap.display_label }}</span>
          <span class="capability-tooltip" :title="`${cap.display_name}: ${cap.description}`">
            ℹ️
          </span>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
export default {
  props: ['capabilities'],
  computed: {
    groupedCapabilities() {
      return this.capabilities.reduce((acc, cap) => {
        const category = cap.category_label;
        if (!acc[category]) {
          acc[category] = [];
        }
        acc[category].push(cap);
        return acc;
      }, {});
    }
  }
};
</script>
```

## ⚠️ 주의사항

### 1. 하위 호환성
- 기존 필드(`display_name`, `category`)는 그대로 유지됩니다
- 새 필드(`display_label`, `category_label`)는 추가로 제공됩니다
- 기존 코드는 수정 없이 계속 작동합니다

### 2. 필드 우선순위
- **표 헤더**: `category_label` 사용 (MANAGE, PROJECT, DICOM 등)
- **표 셀**: `display_label` 사용 (Admin, Users, CREATE, READ 등)
- **상세 설명**: `display_name`과 `description` 사용

### 3. 데이터 타입
- `display_label`: 최대 50자 문자열
- `category_label`: 최대 50자 문자열
- 둘 다 빈 문자열이 될 수 없습니다

## 🧪 테스트 방법

### 1. API 테스트
```bash
# Role-Capability Matrix API 테스트
curl "http://localhost:8080/api/roles/global/capabilities/matrix?page=1&size=10" | jq '.capabilities_by_category."관리"[0]'

# Capability 목록 API 테스트
curl "http://localhost:8080/api/capabilities" | jq '.[0]'
```

### 2. 응답 검증
```javascript
// 새 필드 존재 확인
const response = await fetch('/api/roles/global/capabilities/matrix');
const data = await response.json();
const capability = data.capabilities_by_category["관리"][0];

console.assert(capability.display_label !== undefined, 'display_label 필드가 없습니다');
console.assert(capability.category_label !== undefined, 'category_label 필드가 없습니다');
console.assert(capability.display_label.length > 0, 'display_label이 비어있습니다');
console.assert(capability.category_label.length > 0, 'category_label이 비어있습니다');
```

## 📞 문의사항

API 변경사항에 대한 문의사항이 있으시면 백엔드 개발팀에 연락해주세요.

- **변경사항 확인**: Swagger UI (http://localhost:8080/swagger-ui/)
- **API 테스트**: Postman Collection 또는 curl 명령어 사용
- **문서 참조**: `docs/api/` 폴더의 관련 문서들

---

**마지막 업데이트**: 2025-10-25  
**문서 버전**: 1.0  
**작성자**: AI Assistant
