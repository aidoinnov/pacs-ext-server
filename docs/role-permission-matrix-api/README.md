# 롤별 권한 관리 API 문서

## 📋 개요

롤별 권한 관리 API는 시스템의 역할(Role)과 권한(Permission) 간의 관계를 매트릭스 형태로 조회하고 관리할 수 있는 RESTful API입니다. 이 API를 통해 관리자는 각 역할에 할당된 권한을 시각적으로 확인하고, 개별 권한을 ON/OFF로 쉽게 관리할 수 있습니다.

## 🎯 주요 기능

- **롤-권한 매트릭스 조회**: 모든 롤과 권한의 관계를 표 형태로 조회
- **권한 할당/제거**: 개별 롤의 특정 권한을 켜거나 끄기
- **카테고리별 권한 관리**: 권한을 리소스 타입별로 그룹화하여 관리
- **실시간 업데이트**: 권한 변경 시 즉시 반영

## 🔗 기본 정보

- **Base URL**: `http://localhost:8080/api`
- **Content-Type**: `application/json`
- **인증**: JWT Bearer Token
- **문서**: Swagger UI (`http://localhost:8080/swagger-ui/`)

## 📚 API 엔드포인트

### 1. 글로벌 롤-권한 매트릭스 조회

**엔드포인트**: `GET /api/roles/global/permissions/matrix`

**설명**: 시스템의 모든 롤과 권한 간의 관계를 매트릭스 형태로 조회합니다.

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

**상태 코드**:
- `200 OK`: 성공
- `401 Unauthorized`: 인증 실패
- `500 Internal Server Error`: 서버 오류

### 2. 롤에 권한 할당/제거

**엔드포인트**: `PUT /api/roles/{role_id}/permissions/{permission_id}`

**설명**: 특정 롤에 권한을 할당하거나 제거합니다.

**경로 매개변수**:
- `role_id` (integer, required): 롤 ID
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
- `404 Not Found`: 롤 또는 권한을 찾을 수 없음
- `409 Conflict`: 이미 할당된 권한
- `500 Internal Server Error`: 서버 오류

## 🖥️ 프론트엔드 구현 예시

### React 컴포넌트

```jsx
import React, { useState, useEffect } from 'react';

const RolePermissionMatrix = () => {
  const [matrix, setMatrix] = useState(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadMatrix();
  }, []);

  const loadMatrix = async () => {
    try {
      const response = await fetch('/api/roles/global/permissions/matrix', {
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('token')}`,
          'Content-Type': 'application/json'
        }
      });
      
      if (response.ok) {
        const data = await response.json();
        setMatrix(data);
      }
    } catch (error) {
      console.error('Failed to load matrix:', error);
    } finally {
      setLoading(false);
    }
  };

  const togglePermission = async (roleId, permissionId) => {
    try {
      const currentState = getPermissionState(roleId, permissionId);
      
      const response = await fetch(`/api/roles/${roleId}/permissions/${permissionId}`, {
        method: 'PUT',
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('token')}`,
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({ assign: !currentState })
      });

      if (response.ok) {
        await loadMatrix();
        alert('권한이 변경되었습니다.');
      } else {
        alert('권한 변경에 실패했습니다.');
      }
    } catch (error) {
      console.error('Toggle failed:', error);
      alert('오류가 발생했습니다.');
    }
  };

  const getPermissionState = (roleId, permissionId) => {
    if (!matrix) return false;
    const assignment = matrix.assignments.find(
      a => a.role_id === roleId && a.permission_id === permissionId
    );
    return assignment ? assignment.assigned : false;
  };

  const getAllPermissions = () => {
    if (!matrix) return [];
    return Object.values(matrix.permissions_by_category).flat();
  };

  if (loading) return <div>로딩 중...</div>;
  if (!matrix) return <div>데이터를 불러올 수 없습니다.</div>;

  const allPermissions = getAllPermissions();

  return (
    <div className="role-permission-matrix">
      <h2>롤별 권한 관리</h2>
      <div style={{ overflowX: 'auto' }}>
        <table style={{ borderCollapse: 'collapse', width: '100%', minWidth: '600px' }}>
          <thead>
            <tr style={{ backgroundColor: '#f5f5f5' }}>
              <th style={{ padding: '10px', border: '1px solid #ddd' }}>역할</th>
              {allPermissions.map(permission => (
                <th key={permission.id} style={{ padding: '10px', border: '1px solid #ddd' }}>
                  {permission.action}
                  <br />
                  <small>({permission.resource_type})</small>
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {matrix.roles.map(role => (
              <tr key={role.id}>
                <td style={{ padding: '10px', border: '1px solid #ddd' }}>
                  <strong>{role.name}</strong>
                  <br />
                  <small>{role.description}</small>
                </td>
                {allPermissions.map(permission => {
                  const isAssigned = getPermissionState(role.id, permission.id);
                  return (
                    <td key={permission.id} style={{ padding: '10px', border: '1px solid #ddd', textAlign: 'center' }}>
                      <button
                        onClick={() => togglePermission(role.id, permission.id)}
                        style={{
                          padding: '5px 10px',
                          border: 'none',
                          borderRadius: '4px',
                          cursor: 'pointer',
                          backgroundColor: isAssigned ? '#4CAF50' : '#f44336',
                          color: 'white',
                          fontWeight: 'bold'
                        }}
                      >
                        {isAssigned ? 'ON' : 'OFF'}
                      </button>
                    </td>
                  );
                })}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
};

export default RolePermissionMatrix;
```

### 순수 JavaScript 예시

```html
<!DOCTYPE html>
<html>
<head>
    <title>롤별 권한 관리</title>
    <style>
        table { border-collapse: collapse; width: 100%; }
        th, td { border: 1px solid #ddd; padding: 8px; text-align: center; }
        th { background-color: #f5f5f5; }
        .btn { padding: 5px 10px; border: none; border-radius: 4px; cursor: pointer; }
        .btn-on { background-color: #4CAF50; color: white; }
        .btn-off { background-color: #f44336; color: white; }
    </style>
</head>
<body>
    <div id="matrix-container">
        <h2>롤별 권한 관리</h2>
        <div>로딩 중...</div>
    </div>

    <script>
        let matrixData = null;

        async function loadMatrix() {
            try {
                const response = await fetch('/api/roles/global/permissions/matrix', {
                    headers: {
                        'Authorization': `Bearer ${localStorage.getItem('token')}`,
                        'Content-Type': 'application/json'
                    }
                });
                
                if (response.ok) {
                    matrixData = await response.json();
                    renderMatrix();
                } else {
                    document.getElementById('matrix-container').innerHTML = 
                        '<h2>롤별 권한 관리</h2><div>데이터를 불러올 수 없습니다.</div>';
                }
            } catch (error) {
                document.getElementById('matrix-container').innerHTML = 
                    '<h2>롤별 권한 관리</h2><div>오류가 발생했습니다.</div>';
            }
        }

        function renderMatrix() {
            const container = document.getElementById('matrix-container');
            const permissions = Object.values(matrixData.permissions_by_category).flat();
            
            let html = '<h2>롤별 권한 관리</h2><table><thead><tr><th>역할</th>';
            permissions.forEach(permission => {
                html += `<th>${permission.action}<br><small>(${permission.resource_type})</small></th>`;
            });
            html += '</tr></thead><tbody>';

            matrixData.roles.forEach(role => {
                html += `<tr><td><strong>${role.name}</strong><br><small>${role.description}</small></td>`;
                permissions.forEach(permission => {
                    const assignment = matrixData.assignments.find(
                        a => a.role_id === role.id && a.permission_id === permission.id
                    );
                    const isAssigned = assignment ? assignment.assigned : false;
                    html += `<td><button class="btn ${isAssigned ? 'btn-on' : 'btn-off'}" 
                        onclick="togglePermission(${role.id}, ${permission.id})">
                        ${isAssigned ? 'ON' : 'OFF'}
                    </button></td>`;
                });
                html += '</tr>';
            });
            
            html += '</tbody></table>';
            container.innerHTML = html;
        }

        async function togglePermission(roleId, permissionId) {
            try {
                const currentState = getPermissionState(roleId, permissionId);
                
                const response = await fetch(`/api/roles/${roleId}/permissions/${permissionId}`, {
                    method: 'PUT',
                    headers: {
                        'Authorization': `Bearer ${localStorage.getItem('token')}`,
                        'Content-Type': 'application/json'
                    },
                    body: JSON.stringify({ assign: !currentState })
                });

                if (response.ok) {
                    await loadMatrix();
                    alert('권한이 변경되었습니다.');
                } else {
                    alert('권한 변경에 실패했습니다.');
                }
            } catch (error) {
                alert('오류가 발생했습니다.');
            }
        }

        function getPermissionState(roleId, permissionId) {
            const assignment = matrixData.assignments.find(
                a => a.role_id === roleId && a.permission_id === permissionId
            );
            return assignment ? assignment.assigned : false;
        }

        // 페이지 로드 시 매트릭스 로드
        loadMatrix();
    </script>
</body>
</html>
```

## 🔧 cURL 예시

```bash
# 1. 매트릭스 조회
curl -X GET "http://localhost:8080/api/roles/global/permissions/matrix" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json"

# 2. Admin 롤(1번)에 사용자 생성 권한(1번) 켜기
curl -X PUT "http://localhost:8080/api/roles/1/permissions/1" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"assign": true}'

# 3. Admin 롤(1번)에 사용자 생성 권한(1번) 끄기
curl -X PUT "http://localhost:8080/api/roles/1/permissions/1" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"assign": false}'
```

## 📊 데이터 모델

### Role (롤)
```json
{
  "id": 1,
  "name": "Admin",
  "description": "시스템 관리자",
  "scope": "GLOBAL"
}
```

### Permission (권한)
```json
{
  "id": 1,
  "resource_type": "USER",
  "action": "CREATE",
  "description": "사용자 생성"
}
```

### Assignment (할당)
```json
{
  "role_id": 1,
  "permission_id": 1,
  "assigned": true
}
```

## ⚠️ 주의사항

1. **인증 토큰**: 모든 API 호출 시 유효한 JWT 토큰이 필요합니다.
2. **권한 확인**: 사용자가 권한 관리 권한을 가지고 있는지 확인하세요.
3. **에러 처리**: 네트워크 오류나 서버 오류에 대한 적절한 처리가 필요합니다.
4. **UI 피드백**: 권한 변경 시 사용자에게 명확한 피드백을 제공하세요.
5. **데이터 동기화**: 권한 변경 후 매트릭스 데이터를 다시 로드하여 최신 상태를 유지하세요.

## 🔍 문제 해결

### 자주 발생하는 오류

1. **401 Unauthorized**: JWT 토큰이 유효하지 않거나 만료됨
   - 해결: 토큰을 갱신하거나 다시 로그인

2. **404 Not Found**: 롤이나 권한을 찾을 수 없음
   - 해결: 올바른 ID를 사용하고 있는지 확인

3. **409 Conflict**: 이미 할당된 권한을 다시 할당하려고 함
   - 해결: 현재 상태를 확인하고 적절히 처리

4. **500 Internal Server Error**: 서버 내부 오류
   - 해결: 서버 로그를 확인하고 관리자에게 문의

## 📁 관련 문서

- [기술 문서](technical-documentation.md) - API 구현 세부사항
- [API 참조](api-reference.md) - 상세 API 명세
- [사용 예시](api-examples.md) - 다양한 프레임워크 예시
- [사용자 가이드](user-guide.md) - 상세 사용법

이 문서를 참고하여 롤별 권한 관리 기능을 구현하시면 됩니다! 🚀
