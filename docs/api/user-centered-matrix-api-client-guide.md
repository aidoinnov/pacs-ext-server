# User-Centered Matrix API 클라이언트 가이드

## 개요

User-Centered Matrix API는 사용자 중심으로 프로젝트-역할 관계를 매트릭스 형태로 조회하는 API입니다. 기존의 프로젝트 중심 매트릭스 API와 함께 사용할 수 있습니다.

## API 엔드포인트

```
GET /api/user-project-matrix
```

## 쿼리 파라미터

| 파라미터 | 타입 | 필수 | 기본값 | 설명 |
|---------|------|------|--------|------|
| `user_page` | integer | No | 1 | 사용자 페이지 번호 |
| `user_page_size` | integer | No | 10 | 사용자 페이지 크기 (최대: 50) |
| `project_page` | integer | No | 1 | 프로젝트 페이지 번호 |
| `project_page_size` | integer | No | 10 | 프로젝트 페이지 크기 (최대: 50) |
| `user_sort_by` | string | No | "username" | 사용자 정렬 기준 (`username`, `email`, `created_at`) |
| `user_sort_order` | string | No | "asc" | 정렬 순서 (`asc`, `desc`) |
| `user_search` | string | No | - | 사용자 검색어 (username 또는 email) |
| `role_id` | integer | No | - | 특정 역할 ID로 필터링 |
| `project_ids` | array[integer] | No | - | 특정 프로젝트 ID 목록 |
| `user_ids` | array[integer] | No | - | 특정 사용자 ID 목록 |

## 응답 형식

### 성공 응답 (200 OK)

```json
{
  "matrix": [
    {
      "user_id": 1,
      "username": "TestUser2",
      "email": "user2@example.com",
      "full_name": null,
      "project_roles": [
        {
          "project_id": 14,
          "project_name": "Test Project 1420f1f3",
          "role_id": null,
          "role_name": null
        },
        {
          "project_id": 6,
          "project_name": "Test Project 1585c69a",
          "role_id": 1632,
          "role_name": "ADMIN"
        }
      ]
    }
  ],
  "projects": [
    {
      "project_id": 14,
      "project_name": "Test Project 1420f1f3",
      "description": "Test Description",
      "status": "InProgress"
    },
    {
      "project_id": 6,
      "project_name": "Test Project 1585c69a",
      "description": "Test Description",
      "status": "InProgress"
    }
  ],
  "pagination": {
    "user_page": 1,
    "user_page_size": 5,
    "user_total_count": 58,
    "user_total_pages": 12,
    "project_page": 1,
    "project_page_size": 5,
    "project_total_count": 37,
    "project_total_pages": 8
  }
}
```

### 응답 필드 설명

#### `matrix` 배열
각 요소는 사용자 한 명의 정보와 해당 사용자의 프로젝트 역할들을 나타냅니다.

| 필드 | 타입 | 설명 |
|------|------|------|
| `user_id` | integer | 사용자 ID |
| `username` | string | 사용자명 |
| `email` | string | 이메일 주소 |
| `full_name` | string\|null | 전체 이름 (선택사항) |
| `project_roles` | array | 해당 사용자의 프로젝트 역할 목록 |

#### `project_roles` 배열
각 요소는 사용자가 특정 프로젝트에서 가지는 역할을 나타냅니다.

| 필드 | 타입 | 설명 |
|------|------|------|
| `project_id` | integer | 프로젝트 ID |
| `project_name` | string | 프로젝트 이름 |
| `role_id` | integer\|null | 역할 ID (역할이 없으면 null) |
| `role_name` | string\|null | 역할 이름 (역할이 없으면 null) |

#### `projects` 배열
매트릭스의 열 헤더로 사용될 프로젝트 목록입니다.

| 필드 | 타입 | 설명 |
|------|------|------|
| `project_id` | integer | 프로젝트 ID |
| `project_name` | string | 프로젝트 이름 |
| `description` | string\|null | 프로젝트 설명 |
| `status` | string | 프로젝트 상태 (`InProgress`, `Completed`, `Cancelled` 등) |

#### `pagination` 객체
이중 페이지네이션 정보를 제공합니다.

| 필드 | 타입 | 설명 |
|------|------|------|
| `user_page` | integer | 현재 사용자 페이지 번호 |
| `user_page_size` | integer | 사용자 페이지 크기 |
| `user_total_count` | integer | 전체 사용자 수 |
| `user_total_pages` | integer | 전체 사용자 페이지 수 |
| `project_page` | integer | 현재 프로젝트 페이지 번호 |
| `project_page_size` | integer | 프로젝트 페이지 크기 |
| `project_total_count` | integer | 전체 프로젝트 수 |
| `project_total_pages` | integer | 전체 프로젝트 페이지 수 |

## 사용 예시

### 1. 기본 조회
```bash
curl -X GET "http://localhost:8080/api/user-project-matrix"
```

### 2. 페이지네이션 적용
```bash
curl -X GET "http://localhost:8080/api/user-project-matrix?user_page=2&user_page_size=10&project_page=1&project_page_size=20"
```

### 3. 사용자 정렬 (이메일 기준 내림차순)
```bash
curl -X GET "http://localhost:8080/api/user-project-matrix?user_sort_by=email&user_sort_order=desc"
```

### 4. 사용자 검색
```bash
curl -X GET "http://localhost:8080/api/user-project-matrix?user_search=testuser"
```

### 5. 특정 역할로 필터링
```bash
curl -X GET "http://localhost:8080/api/user-project-matrix?role_id=1632"
```

### 6. 특정 프로젝트들만 조회
```bash
curl -X GET "http://localhost:8080/api/user-project-matrix?project_ids=14,6,20"
```

### 7. 복합 조건 조회
```bash
curl -X GET "http://localhost:8080/api/user-project-matrix?user_search=test&user_sort_by=created_at&user_sort_order=desc&user_page_size=5&project_page_size=10"
```

## JavaScript/TypeScript 사용 예시

### 기본 조회
```typescript
interface UserProjectMatrixResponse {
  matrix: Array<{
    user_id: number;
    username: string;
    email: string;
    full_name: string | null;
    project_roles: Array<{
      project_id: number;
      project_name: string;
      role_id: number | null;
      role_name: string | null;
    }>;
  }>;
  projects: Array<{
    project_id: number;
    project_name: string;
    description: string | null;
    status: string;
  }>;
  pagination: {
    user_page: number;
    user_page_size: number;
    user_total_count: number;
    user_total_pages: number;
    project_page: number;
    project_page_size: number;
    project_total_count: number;
    project_total_pages: number;
  };
}

async function getUserProjectMatrix(params: {
  user_page?: number;
  user_page_size?: number;
  project_page?: number;
  project_page_size?: number;
  user_sort_by?: 'username' | 'email' | 'created_at';
  user_sort_order?: 'asc' | 'desc';
  user_search?: string;
  role_id?: number;
  project_ids?: number[];
  user_ids?: number[];
}): Promise<UserProjectMatrixResponse> {
  const queryParams = new URLSearchParams();
  
  if (params.user_page) queryParams.append('user_page', params.user_page.toString());
  if (params.user_page_size) queryParams.append('user_page_size', params.user_page_size.toString());
  if (params.project_page) queryParams.append('project_page', params.project_page.toString());
  if (params.project_page_size) queryParams.append('project_page_size', params.project_page_size.toString());
  if (params.user_sort_by) queryParams.append('user_sort_by', params.user_sort_by);
  if (params.user_sort_order) queryParams.append('user_sort_order', params.user_sort_order);
  if (params.user_search) queryParams.append('user_search', params.user_search);
  if (params.role_id) queryParams.append('role_id', params.role_id.toString());
  if (params.project_ids) queryParams.append('project_ids', params.project_ids.join(','));
  if (params.user_ids) queryParams.append('user_ids', params.user_ids.join(','));
  
  const response = await fetch(`/api/user-project-matrix?${queryParams.toString()}`);
  
  if (!response.ok) {
    throw new Error(`HTTP error! status: ${response.status}`);
  }
  
  return response.json();
}

// 사용 예시
const matrixData = await getUserProjectMatrix({
  user_page: 1,
  user_page_size: 10,
  user_sort_by: 'username',
  user_sort_order: 'asc',
  user_search: 'test'
});
```

### React 컴포넌트 예시
```tsx
import React, { useState, useEffect } from 'react';

interface UserProjectMatrixProps {
  userPage?: number;
  userPageSize?: number;
  projectPage?: number;
  projectPageSize?: number;
}

const UserProjectMatrix: React.FC<UserProjectMatrixProps> = ({
  userPage = 1,
  userPageSize = 10,
  projectPage = 1,
  projectPageSize = 10
}) => {
  const [matrixData, setMatrixData] = useState<UserProjectMatrixResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchMatrix = async () => {
      try {
        setLoading(true);
        const data = await getUserProjectMatrix({
          user_page: userPage,
          user_page_size: userPageSize,
          project_page: projectPage,
          project_page_size: projectPageSize
        });
        setMatrixData(data);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Unknown error');
      } finally {
        setLoading(false);
      }
    };

    fetchMatrix();
  }, [userPage, userPageSize, projectPage, projectPageSize]);

  if (loading) return <div>Loading...</div>;
  if (error) return <div>Error: {error}</div>;
  if (!matrixData) return <div>No data</div>;

  return (
    <div>
      <h2>User-Project Matrix</h2>
      
      {/* 프로젝트 헤더 */}
      <div className="project-header">
        <div className="user-info-header">User</div>
        {matrixData.projects.map(project => (
          <div key={project.project_id} className="project-header-cell">
            {project.project_name}
          </div>
        ))}
      </div>
      
      {/* 매트릭스 행들 */}
      {matrixData.matrix.map(user => (
        <div key={user.user_id} className="matrix-row">
          <div className="user-info">
            <div className="username">{user.username}</div>
            <div className="email">{user.email}</div>
          </div>
          {matrixData.projects.map(project => {
            const userRole = user.project_roles.find(role => role.project_id === project.project_id);
            return (
              <div key={project.project_id} className="matrix-cell">
                {userRole?.role_name || '-'}
              </div>
            );
          })}
        </div>
      ))}
      
      {/* 페이지네이션 정보 */}
      <div className="pagination-info">
        <p>Users: {matrixData.pagination.user_page} / {matrixData.pagination.user_total_pages}</p>
        <p>Projects: {matrixData.pagination.project_page} / {matrixData.pagination.project_total_pages}</p>
      </div>
    </div>
  );
};

export default UserProjectMatrix;
```

## 에러 응답

### 500 Internal Server Error
```json
{
  "error": "Failed to get user-project matrix: Database error: ..."
}
```

## 기존 API와의 차이점

| 항목 | 프로젝트 중심 API | 사용자 중심 API (신규) |
|------|------------------|----------------------|
| 엔드포인트 | `/api/project-user-matrix` | `/api/user-project-matrix` |
| 행 | 프로젝트 | 사용자 |
| 열 | 사용자 | 프로젝트 |
| 정렬 | 프로젝트 정렬 지원 | 사용자 정렬 지원 |
| 검색 | 프로젝트 검색 | 사용자 검색 |
| 용도 | 프로젝트별 사용자 관리 | 사용자별 프로젝트 관리 |

## 주의사항

1. **페이지 크기 제한**: `user_page_size`와 `project_page_size`는 최대 50까지 설정 가능합니다.
2. **정렬 기준**: `user_sort_by`는 `username`, `email`, `created_at` 중에서만 선택 가능합니다.
3. **정렬 순서**: `user_sort_order`는 `asc` 또는 `desc`만 허용됩니다.
4. **검색**: `user_search`는 사용자명(username) 또는 이메일(email)에서 부분 일치 검색을 수행합니다.
5. **필터링**: `role_id`, `project_ids`, `user_ids`는 AND 조건으로 적용됩니다.

## 성능 고려사항

- 대용량 데이터의 경우 페이지네이션을 적절히 활용하세요.
- 필요한 필드만 요청하여 네트워크 트래픽을 최적화하세요.
- 클라이언트 측에서 캐싱을 고려하여 불필요한 API 호출을 줄이세요.
