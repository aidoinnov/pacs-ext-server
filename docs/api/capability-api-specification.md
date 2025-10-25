# Capability API 스펙 문서

## 개요

Capability API는 사용자 친화적인 권한 관리 인터페이스를 제공합니다. 복잡한 Permission 대신 직관적인 Capability를 사용하여 UI에서 권한 매트릭스를 구현할 수 있습니다.

## 아키텍처

```
Role → Capability → Permission
      (UI 표시)    (실제 권한)
```

## 기본 정보

- **Base URL**: `http://localhost:8080/api`
- **Content-Type**: `application/json`
- **인코딩**: UTF-8

## API 엔드포인트

### 1. 전역 Role-Capability 매트릭스 조회

**엔드포인트**: `GET /api/roles/global/capabilities/matrix`

**설명**: 전역 역할들의 Capability 할당 상태를 매트릭스 형태로 조회합니다.

**응답 예시**:
```json
{
  "roles": [
    {
      "id": 1,
      "name": "SUPER_ADMIN",
      "description": "시스템 전체 관리자",
      "scope": "GLOBAL"
    },
    {
      "id": 2,
      "name": "ADMIN",
      "description": "관리자",
      "scope": "GLOBAL"
    },
    {
      "id": 3,
      "name": "USER",
      "description": "일반 사용자",
      "scope": "GLOBAL"
    },
    {
      "id": 4,
      "name": "VIEWER",
      "description": "조회 전용 사용자",
      "scope": "GLOBAL"
    }
  ],
  "capabilities_by_category": {
    "관리": [
      {
        "id": 35,
        "name": "SYSTEM_ADMIN",
        "display_name": "시스템 관리",
        "description": "시스템 전체 관리 권한",
        "category": "관리",
        "permission_count": 43
      },
      {
        "id": 36,
        "name": "USER_MANAGEMENT",
        "display_name": "사용자 관리",
        "description": "사용자 계정 생성, 조회, 수정, 삭제 권한",
        "category": "관리",
        "permission_count": 4
      },
      {
        "id": 37,
        "name": "ROLE_MANAGEMENT",
        "display_name": "역할 관리",
        "description": "역할 생성, 조회, 수정, 삭제 권한",
        "category": "관리",
        "permission_count": 4
      },
      {
        "id": 38,
        "name": "PROJECT_MANAGEMENT",
        "display_name": "프로젝트 관리",
        "description": "프로젝트 생성, 조회, 수정, 삭제 권한",
        "category": "관리",
        "permission_count": 5
      }
    ],
    "프로젝트": [
      {
        "id": 39,
        "name": "PROJECT_CREATE",
        "display_name": "프로젝트 생성",
        "description": "새 프로젝트 생성 권한",
        "category": "프로젝트",
        "permission_count": 1
      },
      {
        "id": 40,
        "name": "PROJECT_EDIT",
        "display_name": "프로젝트 편집",
        "description": "프로젝트 정보 수정 권한",
        "category": "프로젝트",
        "permission_count": 1
      },
      {
        "id": 41,
        "name": "PROJECT_ASSIGN",
        "display_name": "프로젝트 할당",
        "description": "프로젝트에 사용자 할당 권한",
        "category": "프로젝트",
        "permission_count": 1
      }
    ],
    "DICOM 데이터 관리": [
      {
        "id": 42,
        "name": "DICOM_READ_ACCESS",
        "display_name": "DICOM 읽기 접근",
        "description": "DICOM 스터디, 시리즈, 인스턴스 조회 및 다운로드 권한",
        "category": "DICOM 데이터 관리",
        "permission_count": 6
      },
      {
        "id": 43,
        "name": "DICOM_WRITE_ACCESS",
        "display_name": "DICOM 쓰기 접근",
        "description": "DICOM 데이터 업로드 및 수정 권한",
        "category": "DICOM 데이터 관리",
        "permission_count": 6
      },
      {
        "id": 44,
        "name": "DICOM_DELETE_ACCESS",
        "display_name": "DICOM 삭제 접근",
        "description": "DICOM 데이터 삭제 권한",
        "category": "DICOM 데이터 관리",
        "permission_count": 3
      },
      {
        "id": 45,
        "name": "DICOM_SHARE_ACCESS",
        "display_name": "DICOM 공유 접근",
        "description": "DICOM 데이터 공유 권한",
        "category": "DICOM 데이터 관리",
        "permission_count": 1
      }
    ],
    "어노테이션 관리": [
      {
        "id": 46,
        "name": "ANNOTATION_READ_OWN",
        "display_name": "본인 어노테이션 읽기",
        "description": "자신이 작성한 어노테이션 조회 권한",
        "category": "어노테이션 관리",
        "permission_count": 1
      },
      {
        "id": 47,
        "name": "ANNOTATION_READ_ALL",
        "display_name": "모든 어노테이션 읽기",
        "description": "모든 사용자의 어노테이션 조회 권한",
        "category": "어노테이션 관리",
        "permission_count": 1
      },
      {
        "id": 48,
        "name": "ANNOTATION_WRITE",
        "display_name": "어노테이션 작성",
        "description": "어노테이션 생성 및 수정 권한",
        "category": "어노테이션 관리",
        "permission_count": 2
      },
      {
        "id": 49,
        "name": "ANNOTATION_DELETE",
        "display_name": "어노테이션 삭제",
        "description": "어노테이션 삭제 권한",
        "category": "어노테이션 관리",
        "permission_count": 1
      },
      {
        "id": 50,
        "name": "ANNOTATION_SHARE",
        "display_name": "어노테이션 공유",
        "description": "어노테이션 공유 권한",
        "category": "어노테이션 관리",
        "permission_count": 1
      }
    ],
    "마스크 관리": [
      {
        "id": 51,
        "name": "MASK_READ",
        "display_name": "마스크 읽기",
        "description": "마스크 조회 및 다운로드 권한",
        "category": "마스크 관리",
        "permission_count": 2
      },
      {
        "id": 52,
        "name": "MASK_WRITE",
        "display_name": "마스크 작성",
        "description": "마스크 생성 및 수정 권한",
        "category": "마스크 관리",
        "permission_count": 2
      },
      {
        "id": 53,
        "name": "MASK_DELETE",
        "display_name": "마스크 삭제",
        "description": "마스크 삭제 권한",
        "category": "마스크 관리",
        "permission_count": 1
      }
    ],
    "행잉 프로토콜 관리": [
      {
        "id": 54,
        "name": "HANGING_PROTOCOL_MANAGEMENT",
        "display_name": "행잉 프로토콜 관리",
        "description": "행잉 프로토콜 생성, 조회, 수정, 삭제 권한",
        "category": "행잉 프로토콜 관리",
        "permission_count": 4
      }
    ]
  },
  "assignments": [
    {
      "role_id": 1,
      "capability_id": 35,
      "assigned": true
    },
    {
      "role_id": 1,
      "capability_id": 36,
      "assigned": true
    }
    // ... 총 80개의 할당 관계
  ]
}
```

**HTTP 상태 코드**:
- `200 OK`: 성공
- `500 Internal Server Error`: 서버 오류

### 2. 프로젝트별 Role-Capability 매트릭스 조회

**엔드포인트**: `GET /api/projects/{project_id}/roles/capabilities/matrix`

**설명**: 특정 프로젝트의 역할들의 Capability 할당 상태를 조회합니다.

**경로 매개변수**:
- `project_id` (integer, required): 프로젝트 ID

**응답**: 전역 매트릭스와 동일한 구조

### 3. 모든 Capability 목록 조회

**엔드포인트**: `GET /api/capabilities`

**설명**: 모든 Capability 목록을 조회합니다.

**응답 예시**:
```json
[
  {
    "id": 35,
    "name": "SYSTEM_ADMIN",
    "display_name": "시스템 관리",
    "description": "시스템 전체 관리 권한",
    "category": "관리",
    "permission_count": 43
  },
  {
    "id": 36,
    "name": "USER_MANAGEMENT",
    "display_name": "사용자 관리",
    "description": "사용자 계정 생성, 조회, 수정, 삭제 권한",
    "category": "관리",
    "permission_count": 4
  }
  // ... 총 20개 Capability
]
```

### 4. 카테고리별 Capability 조회

**엔드포인트**: `GET /api/capabilities/category/{category}`

**설명**: 특정 카테고리의 Capability 목록을 조회합니다.

**경로 매개변수**:
- `category` (string, required): 카테고리명 (예: "관리", "프로젝트", "DICOM 데이터 관리")

**응답 예시**:
```json
[
  {
    "id": 35,
    "name": "SYSTEM_ADMIN",
    "display_name": "시스템 관리",
    "description": "시스템 전체 관리 권한",
    "category": "관리",
    "permission_count": 43
  },
  {
    "id": 36,
    "name": "USER_MANAGEMENT",
    "display_name": "사용자 관리",
    "description": "사용자 계정 생성, 조회, 수정, 삭제 권한",
    "category": "관리",
    "permission_count": 4
  }
]
```

### 5. Capability 상세 조회

**엔드포인트**: `GET /api/capabilities/{capability_id}`

**설명**: 특정 Capability의 상세 정보와 매핑된 Permission 목록을 조회합니다.

**경로 매개변수**:
- `capability_id` (integer, required): Capability ID

**응답 예시**:
```json
{
  "capability": {
    "id": 36,
    "name": "USER_MANAGEMENT",
    "display_name": "사용자 관리",
    "description": "사용자 계정 생성, 조회, 수정, 삭제 권한",
    "category": "관리",
    "permission_count": 4
  },
  "permissions": [
    {
      "id": 1,
      "category": "사용자 및 권한 관리",
      "resource_type": "USER",
      "action": "CREATE"
    },
    {
      "id": 2,
      "category": "사용자 및 권한 관리",
      "resource_type": "USER",
      "action": "READ"
    },
    {
      "id": 3,
      "category": "사용자 및 권한 관리",
      "resource_type": "USER",
      "action": "UPDATE"
    },
    {
      "id": 4,
      "category": "사용자 및 권한 관리",
      "resource_type": "USER",
      "action": "DELETE"
    }
  ]
}
```

### 6. Role에 Capability 할당/제거

**엔드포인트**: `PUT /api/roles/{role_id}/capabilities/{capability_id}`

**설명**: 특정 Role에 Capability를 할당하거나 제거합니다.

**경로 매개변수**:
- `role_id` (integer, required): Role ID
- `capability_id` (integer, required): Capability ID

**요청 본문**:
```json
{
  "assign": true
}
```

**응답 예시**:
```json
{
  "message": "Capability assigned successfully"
}
```

**HTTP 상태 코드**:
- `200 OK`: 성공
- `400 Bad Request`: 잘못된 요청
- `404 Not Found`: Role 또는 Capability를 찾을 수 없음
- `500 Internal Server Error`: 서버 오류

## 데이터 모델

### RoleInfo
```typescript
interface RoleInfo {
  id: number;
  name: string;
  description: string;
  scope: string;
}
```

### CapabilityInfo
```typescript
interface CapabilityInfo {
  id: number;
  name: string;
  display_name: string;
  description: string | null;
  category: string;
  permission_count: number;
}
```

### PermissionInfo
```typescript
interface PermissionInfo {
  id: number;
  category: string;
  resource_type: string;
  action: string;
}
```

### RoleCapabilityAssignment
```typescript
interface RoleCapabilityAssignment {
  role_id: number;
  capability_id: number;
  assigned: boolean;
}
```

### UpdateRoleCapabilityAssignmentRequest
```typescript
interface UpdateRoleCapabilityAssignmentRequest {
  assign: boolean;
}
```

## UI 구현 가이드

### 1. 매트릭스 테이블 구현

```typescript
// React 예시
const RoleCapabilityMatrix = () => {
  const [matrix, setMatrix] = useState(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    fetchMatrix();
  }, []);

  const fetchMatrix = async () => {
    try {
      const response = await fetch('/api/roles/global/capabilities/matrix');
      const data = await response.json();
      setMatrix(data);
    } catch (error) {
      console.error('Failed to fetch matrix:', error);
    } finally {
      setLoading(false);
    }
  };

  const toggleAssignment = async (roleId: number, capabilityId: number, assign: boolean) => {
    try {
      await fetch(`/api/roles/${roleId}/capabilities/${capabilityId}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ assign })
      });
      // 매트릭스 새로고침
      fetchMatrix();
    } catch (error) {
      console.error('Failed to update assignment:', error);
    }
  };

  if (loading) return <div>Loading...</div>;

  return (
    <div className="role-capability-matrix">
      <table>
        <thead>
          <tr>
            <th>역할</th>
            {Object.entries(matrix.capabilities_by_category).map(([category, capabilities]) => (
              <th key={category} colSpan={capabilities.length}>
                {category}
              </th>
            ))}
          </tr>
          <tr>
            <th></th>
            {Object.values(matrix.capabilities_by_category).flat().map(capability => (
              <th key={capability.id} title={capability.description}>
                {capability.display_name}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {matrix.roles.map(role => (
            <tr key={role.id}>
              <td>{role.name}</td>
              {Object.values(matrix.capabilities_by_category).flat().map(capability => {
                const assignment = matrix.assignments.find(
                  a => a.role_id === role.id && a.capability_id === capability.id
                );
                return (
                  <td key={capability.id}>
                    <input
                      type="checkbox"
                      checked={assignment?.assigned || false}
                      onChange={(e) => toggleAssignment(role.id, capability.id, e.target.checked)}
                    />
                  </td>
                );
              })}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
};
```

### 2. 카테고리별 그룹화

```typescript
const CapabilityCategoryGroup = ({ category, capabilities, onToggle }) => {
  return (
    <div className="capability-category">
      <h3>{category}</h3>
      <div className="capability-list">
        {capabilities.map(capability => (
          <div key={capability.id} className="capability-item">
            <label>
              <input
                type="checkbox"
                onChange={(e) => onToggle(capability.id, e.target.checked)}
              />
              <span className="capability-name">{capability.display_name}</span>
              <span className="capability-description">{capability.description}</span>
              <span className="permission-count">({capability.permission_count}개 권한)</span>
            </label>
          </div>
        ))}
      </div>
    </div>
  );
};
```

### 3. Capability 상세 모달

```typescript
const CapabilityDetailModal = ({ capabilityId, onClose }) => {
  const [detail, setDetail] = useState(null);

  useEffect(() => {
    if (capabilityId) {
      fetchCapabilityDetail(capabilityId);
    }
  }, [capabilityId]);

  const fetchCapabilityDetail = async (id) => {
    try {
      const response = await fetch(`/api/capabilities/${id}`);
      const data = await response.json();
      setDetail(data);
    } catch (error) {
      console.error('Failed to fetch capability detail:', error);
    }
  };

  if (!detail) return null;

  return (
    <div className="modal">
      <div className="modal-content">
        <h2>{detail.capability.display_name}</h2>
        <p>{detail.capability.description}</p>
        <div className="permissions">
          <h3>포함된 권한 ({detail.permissions.length}개)</h3>
          <ul>
            {detail.permissions.map(permission => (
              <li key={permission.id}>
                {permission.resource_type}:{permission.action}
              </li>
            ))}
          </ul>
        </div>
        <button onClick={onClose}>닫기</button>
      </div>
    </div>
  );
};
```

## 에러 처리

### 일반적인 에러 응답

```json
{
  "error": "Error type",
  "message": "Detailed error message"
}
```

### 에러 타입

- `ValidationError`: 요청 데이터 검증 실패
- `NotFound`: 리소스를 찾을 수 없음
- `DatabaseError`: 데이터베이스 오류
- `InternalServerError`: 서버 내부 오류

## 성능 고려사항

1. **캐싱**: Capability 목록은 자주 변경되지 않으므로 클라이언트 측에서 캐싱 권장
2. **페이지네이션**: Capability가 많아질 경우 페이지네이션 구현 고려
3. **배치 업데이트**: 여러 Capability 할당을 한 번에 처리하는 API 추가 고려

## 보안 고려사항

1. **인증**: 모든 API는 적절한 인증이 필요
2. **권한**: Role-Capability 할당은 관리자만 가능
3. **감사 로그**: 권한 변경 시 감사 로그 기록

## 버전 관리

- **현재 버전**: v1.0
- **API 버전**: `/api/v1/` (향후 확장 시)

## 추가 리소스

- **Swagger UI**: `http://localhost:8080/swagger-ui/`
- **OpenAPI 스펙**: `http://localhost:8080/api-docs/openapi.json`
- **기술 문서**: `docs/technical/CAPABILITY_ABSTRACTION.md`
