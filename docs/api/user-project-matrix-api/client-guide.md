# 클라이언트 구현 가이드

## 🖥️ User-Project Matrix API 클라이언트 가이드

이 문서는 프론트엔드에서 User-Project Matrix API를 사용하는 방법을 설명합니다.

---

## 📦 TypeScript 타입 정의

### API 요청/응답 타입

```typescript
// 쿼리 파라미터
interface UserProjectMatrixParams {
  user_page?: number;
  user_page_size?: number;
  project_page?: number;
  project_page_size?: number;
  user_sort_by?: 'username' | 'email' | 'created_at';
  user_sort_order?: 'asc' | 'desc';
  user_search?: string;
  user_ids?: number[];
  project_ids?: number[];
  role_id?: number;
}

// 응답 데이터
interface UserProjectMatrixResponse {
  matrix: UserProjectMatrixRow[];
  projects: ProjectInfo[];
  pagination: UserProjectMatrixPagination;
}

interface UserProjectMatrixRow {
  user_id: number;
  username: string;
  email: string;
  full_name: string | null;
  project_roles: ProjectRoleCell[];
}

interface ProjectRoleCell {
  project_id: number;
  project_name: string;
  role_id: number | null;
  role_name: string | null;
}

interface ProjectInfo {
  project_id: number;
  project_name: string;
  description: string | null;
  status: string;
}

interface UserProjectMatrixPagination {
  user_page: number;
  user_page_size: number;
  user_total_count: number;
  user_total_pages: number;
  project_page: number;
  project_page_size: number;
  project_total_count: number;
  project_total_pages: number;
}
```

---

## 🚀 API 호출 예시

### 1. Fetch API 사용

```typescript
async function getUserProjectMatrix(
  params: UserProjectMatrixParams
): Promise<UserProjectMatrixResponse> {
  const queryParams = new URLSearchParams();
  
  if (params.user_page) queryParams.append('user_page', params.user_page.toString());
  if (params.user_page_size) queryParams.append('user_page_size', params.user_page_size.toString());
  if (params.project_page) queryParams.append('project_page', params.project_page.toString());
  if (params.project_page_size) queryParams.append('project_page_size', params.project_page_size.toString());
  if (params.user_sort_by) queryParams.append('user_sort_by', params.user_sort_by);
  if (params.user_sort_order) queryParams.append('user_sort_order', params.user_sort_order);
  if (params.user_search) queryParams.append('user_search', params.user_search);
  if (params.user_ids) queryParams.append('user_ids', params.user_ids.join(','));
  if (params.project_ids) queryParams.append('project_ids', params.project_ids.join(','));
  if (params.role_id) queryParams.append('role_id', params.role_id.toString());
  
  const response = await fetch(
    `https://extension.pacs.ai-do.kr/api/user-project-matrix?${queryParams.toString()}`
  );
  
  if (!response.ok) {
    throw new Error(`HTTP error! status: ${response.status}`);
  }
  
  return response.json();
}

// 사용 예시
const matrixData = await getUserProjectMatrix({
  user_page: 1,
  user_page_size: 10,
  project_page: 1,
  project_page_size: 10,
  user_sort_by: 'username',
  user_sort_order: 'asc'
});
```

### 2. Axios 사용

```typescript
import axios from 'axios';

async function getUserProjectMatrix(
  params: UserProjectMatrixParams
): Promise<UserProjectMatrixResponse> {
  const response = await axios.get<UserProjectMatrixResponse>(
    'https://extension.pacs.ai-do.kr/api/user-project-matrix',
    { params }
  );
  
  return response.data;
}

// 사용 예시
const matrixData = await getUserProjectMatrix({
  user_page: 1,
  user_page_size: 10,
  user_search: 'admin'
});
```

---

## ⚛️ React 컴포넌트 예시

### 1. 기본 테이블 컴포넌트

```tsx
import React, { useState, useEffect } from 'react';

interface MatrixTableProps {
  userPage?: number;
  userPageSize?: number;
  projectPage?: number;
  projectPageSize?: number;
}

function MatrixTable({
  userPage = 1,
  userPageSize = 10,
  projectPage = 1,
  projectPageSize = 10
}: MatrixTableProps) {
  const [data, setData] = useState<UserProjectMatrixResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    async function fetchData() {
      try {
        setLoading(true);
        const result = await getUserProjectMatrix({
          user_page: userPage,
          user_page_size: userPageSize,
          project_page: projectPage,
          project_page_size: projectPageSize
        });
        setData(result);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Unknown error');
      } finally {
        setLoading(false);
      }
    }

    fetchData();
  }, [userPage, userPageSize, projectPage, projectPageSize]);

  if (loading) return <div>Loading...</div>;
  if (error) return <div>Error: {error}</div>;
  if (!data) return <div>No data</div>;

  return (
    <div>
      <table className="matrix-table">
        <thead>
          <tr>
            <th>User</th>
            {data.projects.map(project => (
              <th key={project.project_id}>{project.project_name}</th>
            ))}
          </tr>
        </thead>
        <tbody>
          {data.matrix.map(row => (
            <tr key={row.user_id}>
              <td>
                <div>{row.username}</div>
                <div className="text-sm text-gray-500">{row.email}</div>
              </td>
              {row.project_roles.map(cell => (
                <td key={cell.project_id}>
                  {cell.role_name ? (
                    <span className={`role-badge role-${cell.role_name.toLowerCase()}`}>
                      {cell.role_name}
                    </span>
                  ) : (
                    <span className="text-gray-400">-</span>
                  )}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>

      {/* 페이지네이션 */}
      <div className="pagination">
        <div>
          Users: Page {data.pagination.user_page} of {data.pagination.user_total_pages}
          ({data.pagination.user_total_count} total)
        </div>
        <div>
          Projects: Page {data.pagination.project_page} of {data.pagination.project_total_pages}
          ({data.pagination.project_total_count} total)
        </div>
      </div>
    </div>
  );
}
```

### 2. 검색 및 필터링 컴포넌트

```tsx
function MatrixWithFilters() {
  const [userPage, setUserPage] = useState(1);
  const [userSearch, setUserSearch] = useState('');
  const [sortBy, setSortBy] = useState<'username' | 'email'>('username');
  const [sortOrder, setSortOrder] = useState<'asc' | 'desc'>('asc');

  return (
    <div>
      {/* 검색 및 필터 */}
      <div className="filters">
        <input
          type="text"
          placeholder="Search users..."
          value={userSearch}
          onChange={(e) => setUserSearch(e.target.value)}
        />
        
        <select value={sortBy} onChange={(e) => setSortBy(e.target.value as any)}>
          <option value="username">Username</option>
          <option value="email">Email</option>
          <option value="created_at">Created At</option>
        </select>
        
        <select value={sortOrder} onChange={(e) => setSortOrder(e.target.value as any)}>
          <option value="asc">Ascending</option>
          <option value="desc">Descending</option>
        </select>
      </div>

      {/* 매트릭스 테이블 */}
      <MatrixTable
        userPage={userPage}
        userSearch={userSearch}
        sortBy={sortBy}
        sortOrder={sortOrder}
      />
    </div>
  );
}
```

### 3. 페이지네이션 컴포넌트

```tsx
interface PaginationProps {
  currentPage: number;
  totalPages: number;
  onPageChange: (page: number) => void;
}

function Pagination({ currentPage, totalPages, onPageChange }: PaginationProps) {
  return (
    <div className="pagination">
      <button
        disabled={currentPage === 1}
        onClick={() => onPageChange(currentPage - 1)}
      >
        Previous
      </button>
      
      <span>
        Page {currentPage} of {totalPages}
      </span>
      
      <button
        disabled={currentPage === totalPages}
        onClick={() => onPageChange(currentPage + 1)}
      >
        Next
      </button>
    </div>
  );
}
```

---

## 🎨 CSS 스타일 예시

```css
/* 매트릭스 테이블 */
.matrix-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 14px;
}

.matrix-table th,
.matrix-table td {
  padding: 12px;
  border: 1px solid #e0e0e0;
  text-align: left;
}

.matrix-table th {
  background-color: #f5f5f5;
  font-weight: 600;
}

.matrix-table tbody tr:hover {
  background-color: #f9f9f9;
}

/* 역할 배지 */
.role-badge {
  display: inline-block;
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 12px;
  font-weight: 500;
}

.role-project_admin {
  background-color: #ffebee;
  color: #c62828;
}

.role-member {
  background-color: #e3f2fd;
  color: #1565c0;
}

.role-viewer {
  background-color: #f5f5f5;
  color: #616161;
}

/* 페이지네이션 */
.pagination {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 16px;
  padding: 16px;
  background-color: #f5f5f5;
  border-radius: 4px;
}

.pagination button {
  padding: 8px 16px;
  border: 1px solid #ccc;
  background-color: white;
  cursor: pointer;
  border-radius: 4px;
}

.pagination button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.pagination button:hover:not(:disabled) {
  background-color: #f0f0f0;
}
```

---

## 🔄 상태 관리 (React Query)

```typescript
import { useQuery } from '@tanstack/react-query';

function useUserProjectMatrix(params: UserProjectMatrixParams) {
  return useQuery({
    queryKey: ['user-project-matrix', params],
    queryFn: () => getUserProjectMatrix(params),
    staleTime: 5 * 60 * 1000, // 5분
    cacheTime: 10 * 60 * 1000, // 10분
  });
}

// 사용 예시
function MatrixTableWithQuery() {
  const [userPage, setUserPage] = useState(1);
  
  const { data, isLoading, error } = useUserProjectMatrix({
    user_page: userPage,
    user_page_size: 10,
    project_page: 1,
    project_page_size: 10
  });

  if (isLoading) return <div>Loading...</div>;
  if (error) return <div>Error: {error.message}</div>;
  if (!data) return <div>No data</div>;

  return (
    <div>
      {/* 테이블 렌더링 */}
      <table>
        {/* ... */}
      </table>
      
      {/* 페이지네이션 */}
      <Pagination
        currentPage={userPage}
        totalPages={data.pagination.user_total_pages}
        onPageChange={setUserPage}
      />
    </div>
  );
}
```

---

## 💡 사용 팁

### 1. 역할 표시 헬퍼 함수

```typescript
function getRoleDisplay(cell: ProjectRoleCell): string {
  if (cell.role_id === null || cell.role_name === null) {
    return '-';
  }
  return cell.role_name;
}

function getRoleColor(roleName: string | null): string {
  switch (roleName) {
    case 'PROJECT_ADMIN': return '#c62828';
    case 'MEMBER': return '#1565c0';
    case 'VIEWER': return '#616161';
    default: return '#9e9e9e';
  }
}
```

### 2. 셀 클릭 핸들러

```typescript
function handleCellClick(userId: number, projectId: number, currentRole: string | null) {
  // 역할 변경 모달 열기
  openRoleChangeModal({
    userId,
    projectId,
    currentRole
  });
}
```

### 3. 에러 처리

```typescript
async function getUserProjectMatrixSafe(
  params: UserProjectMatrixParams
): Promise<UserProjectMatrixResponse | null> {
  try {
    return await getUserProjectMatrix(params);
  } catch (error) {
    console.error('Failed to fetch matrix:', error);
    // 에러 리포팅 (Sentry, etc.)
    return null;
  }
}
```

---

## 🔗 관련 문서

- [README](./README.md) - API 개요
- [데이터 구조 다이어그램](./data-structure-diagram.md) - 응답 데이터 구조
- [처리 흐름 다이어그램](./sequence-diagram.md) - API 처리 흐름

