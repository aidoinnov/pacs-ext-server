# 롤별 권한 관리 API 사용 예시

## 🚀 빠른 시작

### 1. 매트릭스 데이터 가져오기

```bash
# 글로벌 롤-권한 매트릭스 조회
curl -X GET "http://localhost:8080/api/roles/global/permissions/matrix" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json"
```

### 2. 권한 ON/OFF 하기

```bash
# Admin 롤(1번)에 사용자 생성 권한(1번) 켜기
curl -X PUT "http://localhost:8080/api/roles/1/permissions/1" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"assign": true}'

# Admin 롤(1번)에 사용자 생성 권한(1번) 끄기
curl -X PUT "http://localhost:8080/api/roles/1/permissions/1" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"assign": false}'
```

## 📱 JavaScript/React 예시

### 기본 매트릭스 컴포넌트

```jsx
import React, { useState, useEffect } from 'react';

const PermissionMatrix = () => {
  const [matrix, setMatrix] = useState(null);
  const [loading, setLoading] = useState(true);

  // 매트릭스 데이터 로드
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

  // 권한 토글
  const togglePermission = async (roleId, permissionId, currentState) => {
    try {
      const response = await fetch(`/api/roles/${roleId}/permissions/${permissionId}`, {
        method: 'PUT',
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('token')}`,
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({ assign: !currentState })
      });

      if (response.ok) {
        // 성공 시 매트릭스 다시 로드
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

  if (loading) return <div>로딩 중...</div>;
  if (!matrix) return <div>데이터를 불러올 수 없습니다.</div>;

  // 권한을 카테고리별로 그룹화
  const permissionsByCategory = matrix.permissions_by_category;
  const allPermissions = Object.values(permissionsByCategory).flat();

  return (
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
                const assignment = matrix.assignments.find(
                  a => a.role_id === role.id && a.permission_id === permission.id
                );
                const isAssigned = assignment ? assignment.assigned : false;
                
                return (
                  <td key={permission.id} style={{ padding: '10px', border: '1px solid #ddd', textAlign: 'center' }}>
                    <button
                      onClick={() => togglePermission(role.id, permission.id, isAssigned)}
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
  );
};

export default PermissionMatrix;
```

## 🎨 Vue.js 예시

```vue
<template>
  <div class="permission-matrix">
    <div v-if="loading">로딩 중...</div>
    <div v-else-if="error">오류: {{ error }}</div>
    <div v-else>
      <table>
        <thead>
          <tr>
            <th>역할</th>
            <th v-for="permission in allPermissions" :key="permission.id">
              {{ permission.action }}
              <br />
              <small>({{ permission.resource_type }})</small>
            </th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="role in matrix.roles" :key="role.id">
            <td>
              <strong>{{ role.name }}</strong>
              <br />
              <small>{{ role.description }}</small>
            </td>
            <td v-for="permission in allPermissions" :key="permission.id">
              <button
                @click="togglePermission(role.id, permission.id)"
                :class="getPermissionClass(role.id, permission.id)"
              >
                {{ getPermissionState(role.id, permission.id) ? 'ON' : 'OFF' }}
              </button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<script>
export default {
  name: 'PermissionMatrix',
  data() {
    return {
      matrix: null,
      loading: true,
      error: null
    }
  },
  computed: {
    allPermissions() {
      if (!this.matrix) return [];
      return Object.values(this.matrix.permissions_by_category).flat();
    }
  },
  async mounted() {
    await this.loadMatrix();
  },
  methods: {
    async loadMatrix() {
      try {
        this.loading = true;
        const response = await fetch('/api/roles/global/permissions/matrix', {
          headers: {
            'Authorization': `Bearer ${this.$store.state.token}`,
            'Content-Type': 'application/json'
          }
        });
        
        if (response.ok) {
          this.matrix = await response.json();
        } else {
          this.error = '데이터를 불러올 수 없습니다.';
        }
      } catch (err) {
        this.error = err.message;
      } finally {
        this.loading = false;
      }
    },
    
    async togglePermission(roleId, permissionId) {
      try {
        const currentState = this.getPermissionState(roleId, permissionId);
        
        const response = await fetch(`/api/roles/${roleId}/permissions/${permissionId}`, {
          method: 'PUT',
          headers: {
            'Authorization': `Bearer ${this.$store.state.token}`,
            'Content-Type': 'application/json'
          },
          body: JSON.stringify({ assign: !currentState })
        });

        if (response.ok) {
          await this.loadMatrix();
          this.$toast.success('권한이 변경되었습니다.');
        } else {
          this.$toast.error('권한 변경에 실패했습니다.');
        }
      } catch (err) {
        this.$toast.error('오류가 발생했습니다.');
      }
    },
    
    getPermissionState(roleId, permissionId) {
      const assignment = this.matrix.assignments.find(
        a => a.role_id === roleId && a.permission_id === permissionId
      );
      return assignment ? assignment.assigned : false;
    },
    
    getPermissionClass(roleId, permissionId) {
      const isAssigned = this.getPermissionState(roleId, permissionId);
      return {
        'btn': true,
        'btn-success': isAssigned,
        'btn-danger': !isAssigned
      };
    }
  }
}
</script>

<style scoped>
.permission-matrix {
  overflow-x: auto;
}

table {
  width: 100%;
  border-collapse: collapse;
  min-width: 600px;
}

th, td {
  border: 1px solid #ddd;
  padding: 8px;
  text-align: center;
}

th {
  background-color: #f5f5f5;
  font-weight: bold;
}

.btn {
  padding: 6px 12px;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-weight: bold;
}

.btn-success {
  background-color: #4CAF50;
  color: white;
}

.btn-danger {
  background-color: #f44336;
  color: white;
}

.btn:hover {
  opacity: 0.8;
}
</style>
```

## 🔧 Angular 예시

```typescript
// permission-matrix.component.ts
import { Component, OnInit } from '@angular/core';
import { HttpClient } from '@angular/common/http';

interface Role {
  id: number;
  name: string;
  description: string;
  scope: string;
}

interface Permission {
  id: number;
  resource_type: string;
  action: string;
  description: string;
}

interface Assignment {
  role_id: number;
  permission_id: number;
  assigned: boolean;
}

interface MatrixData {
  roles: Role[];
  permissions_by_category: { [key: string]: Permission[] };
  assignments: Assignment[];
}

@Component({
  selector: 'app-permission-matrix',
  templateUrl: './permission-matrix.component.html',
  styleUrls: ['./permission-matrix.component.css']
})
export class PermissionMatrixComponent implements OnInit {
  matrix: MatrixData | null = null;
  loading = true;
  error: string | null = null;

  constructor(private http: HttpClient) {}

  ngOnInit() {
    this.loadMatrix();
  }

  async loadMatrix() {
    try {
      this.loading = true;
      this.matrix = await this.http.get<MatrixData>('/api/roles/global/permissions/matrix').toPromise();
    } catch (err) {
      this.error = '데이터를 불러올 수 없습니다.';
    } finally {
      this.loading = false;
    }
  }

  async togglePermission(roleId: number, permissionId: number) {
    try {
      const currentState = this.getPermissionState(roleId, permissionId);
      
      await this.http.put(`/api/roles/${roleId}/permissions/${permissionId}`, {
        assign: !currentState
      }).toPromise();

      await this.loadMatrix();
      alert('권한이 변경되었습니다.');
    } catch (err) {
      alert('권한 변경에 실패했습니다.');
    }
  }

  getPermissionState(roleId: number, permissionId: number): boolean {
    if (!this.matrix) return false;
    const assignment = this.matrix.assignments.find(
      a => a.role_id === roleId && a.permission_id === permissionId
    );
    return assignment ? assignment.assigned : false;
  }

  get allPermissions(): Permission[] {
    if (!this.matrix) return [];
    return Object.values(this.matrix.permissions_by_category).flat();
  }
}
```

```html
<!-- permission-matrix.component.html -->
<div class="permission-matrix">
  <div *ngIf="loading">로딩 중...</div>
  <div *ngIf="error">오류: {{ error }}</div>
  <div *ngIf="matrix">
    <table>
      <thead>
        <tr>
          <th>역할</th>
          <th *ngFor="let permission of allPermissions">
            {{ permission.action }}
            <br />
            <small>({{ permission.resource_type }})</small>
          </th>
        </tr>
      </thead>
      <tbody>
        <tr *ngFor="let role of matrix.roles">
          <td>
            <strong>{{ role.name }}</strong>
            <br />
            <small>{{ role.description }}</small>
          </td>
          <td *ngFor="let permission of allPermissions">
            <button
              (click)="togglePermission(role.id, permission.id)"
              [class]="getPermissionState(role.id, permission.id) ? 'btn btn-success' : 'btn btn-danger'"
            >
              {{ getPermissionState(role.id, permission.id) ? 'ON' : 'OFF' }}
            </button>
          </td>
        </tr>
      </tbody>
    </table>
  </div>
</div>
```

## 📝 간단한 HTML/JavaScript 예시

```html
<!DOCTYPE html>
<html>
<head>
    <title>권한 관리 매트릭스</title>
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
                    document.getElementById('matrix-container').innerHTML = '데이터를 불러올 수 없습니다.';
                }
            } catch (error) {
                document.getElementById('matrix-container').innerHTML = '오류가 발생했습니다.';
            }
        }

        function renderMatrix() {
            const container = document.getElementById('matrix-container');
            const permissions = Object.values(matrixData.permissions_by_category).flat();
            
            let html = '<table><thead><tr><th>역할</th>';
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

이제 어떤 프레임워크를 사용하든 쉽게 롤별 권한 관리 UI를 구현할 수 있습니다! 🎉
