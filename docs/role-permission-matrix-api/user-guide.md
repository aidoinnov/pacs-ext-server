# 롤별 권한 관리 API 사용 가이드

> **참고**: 빠른 시작을 원한다면 [README.md](README.md)를 먼저 확인하세요.

## 📋 개요

이 문서는 롤별 권한을 ON/OFF하는 기능을 구현할 때 사용할 수 있는 API에 대한 상세한 사용 가이드입니다. 

## 🎯 주요 기능

- **롤 목록 조회**: 시스템에 등록된 모든 역할을 확인
- **권한 목록 조회**: 카테고리별로 분류된 권한 목록 확인
- **권한 매트릭스 조회**: 각 롤별로 어떤 권한이 할당되어 있는지 표 형태로 확인
- **권한 ON/OFF**: 개별 롤의 특정 권한을 켜거나 끄기

## 🔗 API 기본 정보

- **서버 주소**: `http://localhost:8080`
- **인증 방식**: JWT Bearer Token
- **데이터 형식**: JSON

## 📚 API 사용법

### 1. 롤-권한 매트릭스 조회 (표 데이터 가져오기)

#### 글로벌 롤-권한 매트릭스
```http
GET /api/roles/global/permissions/matrix
Authorization: Bearer <your-jwt-token>
```

**응답 예시**:
```json
{
  "roles": [
    {
      "id": 1,
      "name": "Admin",
      "description": "시스템 관리자",
      "scope": "GLOBAL"
    },
    {
      "id": 2,
      "name": "User",
      "description": "일반 사용자",
      "scope": "GLOBAL"
    }
  ],
  "permissions_by_category": {
    "USER": [
      {
        "id": 1,
        "resource_type": "USER",
        "action": "CREATE",
        "description": "사용자 생성"
      },
      {
        "id": 2,
        "resource_type": "USER", 
        "action": "READ",
        "description": "사용자 조회"
      }
    ],
    "PROJECT": [
      {
        "id": 3,
        "resource_type": "PROJECT",
        "action": "CREATE",
        "description": "프로젝트 생성"
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
      "role_id": 2,
      "permission_id": 1,
      "assigned": false
    }
  ]
}
```

#### 프로젝트별 롤-권한 매트릭스
```http
GET /api/projects/{project_id}/roles/permissions/matrix
Authorization: Bearer <your-jwt-token>
```

### 2. 권한 ON/OFF (토글)

#### 글로벌 롤의 권한 토글
```http
PUT /api/roles/{role_id}/permissions/{permission_id}
Authorization: Bearer <your-jwt-token>
Content-Type: application/json

{
  "assign": true
}
```

**파라미터**:
- `role_id`: 롤 ID (예: 1)
- `permission_id`: 권한 ID (예: 5)
- `assign`: `true` (권한 켜기) 또는 `false` (권한 끄기)

**성공 응답**:
```json
{
  "success": true,
  "message": "Permission assigned successfully"
}
```

#### 프로젝트별 롤의 권한 토글
```http
PUT /api/projects/{project_id}/roles/{role_id}/permissions/{permission_id}
Authorization: Bearer <your-jwt-token>
Content-Type: application/json

{
  "assign": true
}
```

## 🖥️ 프론트엔드 구현 예시

### 1. 매트릭스 데이터 로딩

```javascript
// 글로벌 롤-권한 매트릭스 조회
async function loadGlobalMatrix() {
  try {
    const response = await fetch('/api/roles/global/permissions/matrix', {
      headers: {
        'Authorization': `Bearer ${getJwtToken()}`,
        'Content-Type': 'application/json'
      }
    });
    
    if (!response.ok) {
      throw new Error('Failed to load matrix');
    }
    
    const data = await response.json();
    return data;
  } catch (error) {
    console.error('Error loading matrix:', error);
    throw error;
  }
}
```

### 2. 표 렌더링

```javascript
function renderMatrix(matrixData) {
  const { roles, permissions_by_category, assignments } = matrixData;
  
  // 표 헤더 생성 (권한 카테고리별)
  const headers = Object.keys(permissions_by_category).map(category => {
    return permissions_by_category[category].map(permission => ({
      id: permission.id,
      name: permission.action,
      category: category,
      description: permission.description
    }));
  }).flat();
  
  // 표 바디 생성 (롤별)
  const rows = roles.map(role => {
    const row = { role, permissions: {} };
    
    headers.forEach(permission => {
      const assignment = assignments.find(a => 
        a.role_id === role.id && a.permission_id === permission.id
      );
      row.permissions[permission.id] = assignment ? assignment.assigned : false;
    });
    
    return row;
  });
  
  return { headers, rows };
}
```

### 3. ON/OFF 토글 기능

```javascript
// 권한 토글 함수
async function togglePermission(roleId, permissionId, isGlobal = true, projectId = null) {
  const currentState = getCurrentPermissionState(roleId, permissionId);
  const newState = !currentState;
  
  const url = isGlobal 
    ? `/api/roles/${roleId}/permissions/${permissionId}`
    : `/api/projects/${projectId}/roles/${roleId}/permissions/${permissionId}`;
  
  try {
    const response = await fetch(url, {
      method: 'PUT',
      headers: {
        'Authorization': `Bearer ${getJwtToken()}`,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        assign: newState
      })
    });
    
    if (!response.ok) {
      throw new Error('Failed to toggle permission');
    }
    
    const result = await response.json();
    
    if (result.success) {
      // UI 업데이트
      updatePermissionUI(roleId, permissionId, newState);
      showSuccessMessage(`권한이 ${newState ? '활성화' : '비활성화'}되었습니다.`);
    }
    
  } catch (error) {
    console.error('Error toggling permission:', error);
    showErrorMessage('권한 변경에 실패했습니다.');
  }
}
```

### 4. React 컴포넌트 예시

```jsx
import React, { useState, useEffect } from 'react';

const RolePermissionMatrix = ({ isGlobal = true, projectId = null }) => {
  const [matrixData, setMatrixData] = useState(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    loadMatrix();
  }, [isGlobal, projectId]);

  const loadMatrix = async () => {
    try {
      setLoading(true);
      const data = await loadGlobalMatrix();
      setMatrixData(data);
    } catch (err) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  };

  const handleToggle = async (roleId, permissionId) => {
    try {
      await togglePermission(roleId, permissionId, isGlobal, projectId);
      // 매트릭스 다시 로드
      await loadMatrix();
    } catch (err) {
      console.error('Toggle failed:', err);
    }
  };

  if (loading) return <div>로딩 중...</div>;
  if (error) return <div>오류: {error}</div>;
  if (!matrixData) return <div>데이터가 없습니다.</div>;

  const { headers, rows } = renderMatrix(matrixData);

  return (
    <div className="role-permission-matrix">
      <table>
        <thead>
          <tr>
            <th>역할</th>
            {headers.map(permission => (
              <th key={permission.id} title={permission.description}>
                {permission.name}
                <br />
                <small>({permission.category})</small>
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {rows.map(row => (
            <tr key={row.role.id}>
              <td>
                <strong>{row.role.name}</strong>
                <br />
                <small>{row.role.description}</small>
              </td>
              {headers.map(permission => (
                <td key={permission.id}>
                  <button
                    className={`toggle-btn ${row.permissions[permission.id] ? 'active' : 'inactive'}`}
                    onClick={() => handleToggle(row.role.id, permission.id)}
                  >
                    {row.permissions[permission.id] ? 'ON' : 'OFF'}
                  </button>
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
};

export default RolePermissionMatrix;
```

## 🎨 CSS 스타일 예시

```css
.role-permission-matrix {
  overflow-x: auto;
  margin: 20px 0;
}

.role-permission-matrix table {
  width: 100%;
  border-collapse: collapse;
  min-width: 800px;
}

.role-permission-matrix th,
.role-permission-matrix td {
  border: 1px solid #ddd;
  padding: 8px;
  text-align: center;
}

.role-permission-matrix th {
  background-color: #f5f5f5;
  font-weight: bold;
}

.toggle-btn {
  padding: 6px 12px;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-weight: bold;
  transition: all 0.2s;
}

.toggle-btn.active {
  background-color: #4CAF50;
  color: white;
}

.toggle-btn.inactive {
  background-color: #f44336;
  color: white;
}

.toggle-btn:hover {
  opacity: 0.8;
  transform: scale(1.05);
}
```

## ⚠️ 주의사항

1. **인증 토큰**: 모든 API 호출 시 유효한 JWT 토큰이 필요합니다.
2. **권한 확인**: 사용자가 권한 관리 권한을 가지고 있는지 확인하세요.
3. **에러 처리**: 네트워크 오류나 서버 오류에 대한 적절한 처리가 필요합니다.
4. **UI 피드백**: 권한 변경 시 사용자에게 명확한 피드백을 제공하세요.
5. **데이터 동기화**: 권한 변경 후 매트릭스 데이터를 다시 로드하여 최신 상태를 유지하세요.

## 🔧 문제 해결

### 자주 발생하는 오류

1. **401 Unauthorized**: JWT 토큰이 유효하지 않거나 만료됨
   - 해결: 토큰을 갱신하거나 다시 로그인

2. **404 Not Found**: 역할이나 권한을 찾을 수 없음
   - 해결: 올바른 ID를 사용하고 있는지 확인

3. **409 Conflict**: 이미 할당된 권한을 다시 할당하려고 함
   - 해결: 현재 상태를 확인하고 적절히 처리

4. **500 Internal Server Error**: 서버 내부 오류
   - 해결: 서버 로그를 확인하고 관리자에게 문의

이 가이드를 참고하여 롤별 권한 관리 UI를 구현하시면 됩니다!
