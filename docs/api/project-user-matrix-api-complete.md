# Project User Matrix API 문서

## 📋 개요

Project User Matrix API는 프로젝트와 사용자 간의 역할 관계를 매트릭스 형태로 조회할 수 있는 API입니다. 관리자 UI에서 테이블 형태로 표시하기 위해 설계되었으며, 각 셀은 특정 사용자가 특정 프로젝트에서 담당하는 역할을 나타냅니다.

## 🔗 기본 URL

```
http://localhost:8080/api
```

## 📊 데이터 모델

### MatrixQueryParams
```typescript
interface MatrixQueryParams {
  project_page?: number;           // 프로젝트 페이지 번호 (기본값: 1)
  project_page_size?: number;      // 프로젝트 페이지 크기 (기본값: 10, 최대: 50)
  user_page?: number;              // 사용자 페이지 번호 (기본값: 1)
  user_page_size?: number;         // 사용자 페이지 크기 (기본값: 10, 최대: 50)
  project_status?: string[];       // 프로젝트 상태 필터 (PREPARING, IN_PROGRESS, COMPLETED, ON_HOLD, CANCELLED)
  project_ids?: number[];          // 특정 프로젝트 ID 목록
  user_ids?: number[];             // 특정 사용자 ID 목록
}
```

### UserRoleCell
```typescript
interface UserRoleCell {
  user_id: number;                 // 사용자 ID
  username: string;                // 사용자명
  email: string;                   // 이메일
  role_id?: number;                // 역할 ID (역할이 할당되지 않은 경우 null)
  role_name?: string;              // 역할명 (역할이 할당되지 않은 경우 null)
}
```

### ProjectUserMatrixRow
```typescript
interface ProjectUserMatrixRow {
  project_id: number;              // 프로젝트 ID
  project_name: string;            // 프로젝트명
  description?: string;            // 프로젝트 설명
  status: string;                   // 프로젝트 상태
  user_roles: UserRoleCell[];      // 해당 프로젝트에서의 사용자 역할 목록
}
```

### UserInfo
```typescript
interface UserInfo {
  user_id: number;                 // 사용자 ID
  username: string;                // 사용자명
  email: string;                   // 이메일
  full_name?: string;              // 실명 (선택사항)
}
```

### MatrixPagination
```typescript
interface MatrixPagination {
  project_page: number;            // 프로젝트 페이지 번호
  project_page_size: number;       // 프로젝트 페이지 크기
  project_total_count: number;     // 프로젝트 총 개수
  project_total_pages: number;     // 프로젝트 총 페이지 수
  user_page: number;              // 사용자 페이지 번호
  user_page_size: number;          // 사용자 페이지 크기
  user_total_count: number;        // 사용자 총 개수
  user_total_pages: number;         // 사용자 총 페이지 수
}
```

### ProjectUserMatrixResponse
```typescript
interface ProjectUserMatrixResponse {
  matrix: ProjectUserMatrixRow[];  // 매트릭스 행 목록 (프로젝트별)
  users: UserInfo[];               // 사용자 정보 목록 (열 헤더용)
  pagination: MatrixPagination;    // 페이지네이션 정보
}
```

## 🚀 API 엔드포인트

### 프로젝트-사용자 역할 매트릭스 조회

**GET** `/api/project-user-matrix`

프로젝트와 사용자의 역할 관계를 매트릭스 형태로 조회합니다. 이중 페이지네이션(프로젝트/사용자)과 다양한 필터링 옵션을 지원합니다.

#### 요청

```http
GET /api/project-user-matrix?project_page=1&project_page_size=5&user_page=1&user_page_size=5&project_status[]=IN_PROGRESS&project_status[]=PREPARING
```

#### 쿼리 파라미터

| 파라미터 | 타입 | 기본값 | 설명 |
|----------|------|--------|------|
| project_page | integer | 1 | 프로젝트 페이지 번호 (1부터 시작) |
| project_page_size | integer | 10 | 프로젝트 페이지 크기 (최대 50) |
| user_page | integer | 1 | 사용자 페이지 번호 (1부터 시작) |
| user_page_size | integer | 10 | 사용자 페이지 크기 (최대 50) |
| project_status | array | - | 프로젝트 상태 필터 (PREPARING, IN_PROGRESS, COMPLETED, ON_HOLD, CANCELLED) |
| project_ids | array | - | 특정 프로젝트 ID 목록 |
| user_ids | array | - | 특정 사용자 ID 목록 |

#### 응답

**성공 (200 OK)**
```json
{
  "matrix": [
    {
      "project_id": 1,
      "project_name": "Chest X-ray Analysis",
      "description": "흉부 X-ray 이미지 분석 프로젝트",
      "status": "IN_PROGRESS",
      "user_roles": [
        {
          "user_id": 1,
          "username": "john.doe",
          "email": "john.doe@example.com",
          "role_id": 2,
          "role_name": "PROJECT_ADMIN"
        },
        {
          "user_id": 2,
          "username": "jane.smith",
          "email": "jane.smith@example.com",
          "role_id": 3,
          "role_name": "PROJECT_MEMBER"
        },
        {
          "user_id": 3,
          "username": "bob.wilson",
          "email": "bob.wilson@example.com",
          "role_id": null,
          "role_name": null
        }
      ]
    },
    {
      "project_id": 2,
      "project_name": "MRI Brain Scan",
      "description": "MRI 뇌 스캔 이미지 분석 프로젝트",
      "status": "PREPARING",
      "user_roles": [
        {
          "user_id": 1,
          "username": "john.doe",
          "email": "john.doe@example.com",
          "role_id": null,
          "role_name": null
        },
        {
          "user_id": 2,
          "username": "jane.smith",
          "email": "jane.smith@example.com",
          "role_id": 2,
          "role_name": "PROJECT_ADMIN"
        },
        {
          "user_id": 3,
          "username": "bob.wilson",
          "email": "bob.wilson@example.com",
          "role_id": 3,
          "role_name": "PROJECT_MEMBER"
        }
      ]
    }
  ],
  "users": [
    {
      "user_id": 1,
      "username": "john.doe",
      "email": "john.doe@example.com",
      "full_name": "John Doe"
    },
    {
      "user_id": 2,
      "username": "jane.smith",
      "email": "jane.smith@example.com",
      "full_name": "Jane Smith"
    },
    {
      "user_id": 3,
      "username": "bob.wilson",
      "email": "bob.wilson@example.com",
      "full_name": "Bob Wilson"
    }
  ],
  "pagination": {
    "project_page": 1,
    "project_page_size": 5,
    "project_total_count": 15,
    "project_total_pages": 3,
    "user_page": 1,
    "user_page_size": 5,
    "user_total_count": 25,
    "user_total_pages": 5
  }
}
```

**실패 (500 Internal Server Error)**
```json
{
  "error": "Failed to get matrix: Database error: no column found for name: account_status"
}
```

## 📝 사용 예시

### 1. 기본 매트릭스 조회

```javascript
// 기본 매트릭스 조회 (기본 페이지네이션)
const getMatrix = async () => {
  const response = await fetch('/api/project-user-matrix');
  return await response.json();
};
```

### 2. 필터링된 매트릭스 조회

```javascript
// 진행중인 프로젝트만 조회
const getActiveProjectsMatrix = async () => {
  const params = new URLSearchParams({
    project_status: 'IN_PROGRESS',
    project_page_size: '20',
    user_page_size: '20'
  });
  
  const response = await fetch(`/api/project-user-matrix?${params}`);
  return await response.json();
};
```

### 3. 특정 프로젝트와 사용자만 조회

```javascript
// 특정 프로젝트와 사용자만 조회
const getSpecificMatrix = async () => {
  const params = new URLSearchParams();
  params.append('project_ids', '1');
  params.append('project_ids', '2');
  params.append('user_ids', '1');
  params.append('user_ids', '2');
  
  const response = await fetch(`/api/project-user-matrix?${params}`);
  return await response.json();
};
```

### 4. 페이지네이션 처리

```javascript
// 페이지네이션 처리
const getMatrixWithPagination = async (projectPage = 1, userPage = 1) => {
  const params = new URLSearchParams({
    project_page: projectPage.toString(),
    project_page_size: '10',
    user_page: userPage.toString(),
    user_page_size: '10'
  });
  
  const response = await fetch(`/api/project-user-matrix?${params}`);
  const data = await response.json();
  
  return {
    matrix: data.matrix,
    users: data.users,
    pagination: data.pagination,
    hasNextProjectPage: data.pagination.project_page < data.pagination.project_total_pages,
    hasNextUserPage: data.pagination.user_page < data.pagination.user_total_pages
  };
};
```

### 5. React 컴포넌트 예시

```jsx
import React, { useState, useEffect } from 'react';

const ProjectUserMatrix = () => {
  const [matrix, setMatrix] = useState([]);
  const [users, setUsers] = useState([]);
  const [pagination, setPagination] = useState({});
  const [loading, setLoading] = useState(false);

  const fetchMatrix = async (projectPage = 1, userPage = 1) => {
    setLoading(true);
    try {
      const params = new URLSearchParams({
        project_page: projectPage.toString(),
        project_page_size: '10',
        user_page: userPage.toString(),
        user_page_size: '10'
      });
      
      const response = await fetch(`/api/project-user-matrix?${params}`);
      const data = await response.json();
      
      setMatrix(data.matrix);
      setUsers(data.users);
      setPagination(data.pagination);
    } catch (error) {
      console.error('Failed to fetch matrix:', error);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchMatrix();
  }, []);

  return (
    <div className="matrix-container">
      <h2>프로젝트-사용자 역할 매트릭스</h2>
      
      {loading ? (
        <div>로딩 중...</div>
      ) : (
        <table className="matrix-table">
          <thead>
            <tr>
              <th>프로젝트</th>
              {users.map(user => (
                <th key={user.user_id}>
                  {user.full_name || user.username}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {matrix.map(row => (
              <tr key={row.project_id}>
                <td>
                  <div>
                    <strong>{row.project_name}</strong>
                    <br />
                    <small>{row.status}</small>
                  </div>
                </td>
                {users.map(user => {
                  const userRole = row.user_roles.find(ur => ur.user_id === user.user_id);
                  return (
                    <td key={user.user_id}>
                      {userRole?.role_name || '-'}
                    </td>
                  );
                })}
              </tr>
            ))}
          </tbody>
        </table>
      )}
      
      <div className="pagination">
        <button 
          onClick={() => fetchMatrix(pagination.project_page - 1, pagination.user_page)}
          disabled={pagination.project_page <= 1}
        >
          이전 프로젝트 페이지
        </button>
        <span>
          프로젝트: {pagination.project_page} / {pagination.project_total_pages}
        </span>
        <button 
          onClick={() => fetchMatrix(pagination.project_page + 1, pagination.user_page)}
          disabled={pagination.project_page >= pagination.project_total_pages}
        >
          다음 프로젝트 페이지
        </button>
      </div>
    </div>
  );
};

export default ProjectUserMatrix;
```

## ⚠️ 주의사항

### 1. 프로젝트 상태 (Project Status)

| 상태 | 설명 |
|------|------|
| PREPARING | 준비중 |
| IN_PROGRESS | 진행중 |
| COMPLETED | 완료 |
| ON_HOLD | 보류 |
| CANCELLED | 취소 |

### 2. 페이지네이션 제한

- `project_page_size`와 `user_page_size`의 최대값은 50입니다
- 페이지 번호는 1부터 시작합니다
- 프로젝트와 사용자 페이지네이션은 독립적으로 작동합니다

### 3. 필터링

- `project_status`는 배열로 전달하며, 여러 상태를 동시에 필터링할 수 있습니다
- `project_ids`와 `user_ids`는 배열로 전달하며, 특정 ID들만 조회할 수 있습니다
- 필터는 AND 조건으로 적용됩니다

### 4. 역할 정보

- 역할이 할당되지 않은 사용자의 경우 `role_id`와 `role_name`이 `null`입니다
- 매트릭스의 각 셀은 해당 프로젝트에서의 사용자 역할을 나타냅니다

## 🔄 API 사용 흐름

### 1. 기본 매트릭스 조회

1. **기본 조회**: `GET /api/project-user-matrix`
2. **페이지네이션**: 프로젝트와 사용자 각각 독립적으로 페이지 이동
3. **필터링**: 필요에 따라 상태나 특정 ID로 필터링

### 2. 관리자 UI 구현

1. **테이블 헤더**: `users` 배열을 사용하여 사용자 열 생성
2. **테이블 행**: `matrix` 배열을 사용하여 프로젝트 행 생성
3. **셀 데이터**: 각 행의 `user_roles` 배열에서 해당 사용자의 역할 정보 추출
4. **페이지네이션**: `pagination` 객체를 사용하여 페이지 네비게이션 구현

## 🚨 현재 알려진 문제

### 데이터베이스 스키마 문제

현재 API 호출 시 다음과 같은 에러가 발생합니다:

```json
{
  "error": "Failed to get matrix: Database error: no column found for name: account_status"
}
```

이는 데이터베이스 스키마에서 `account_status` 컬럼이 존재하지 않기 때문입니다. 이 문제를 해결하려면:

1. **데이터베이스 스키마 업데이트** 필요
2. **Repository 구현체 수정** 필요
3. **테스트 데이터 추가** 필요

## 📚 관련 문서

- [프로젝트별 사용자 Role 관리 API](./project-user-role-management-api.md)
- [Role 관리 API](./role-management-api.md)
- [Capability 관리 API](./capability-management-api.md)

## 🔧 문제 해결

### 일반적인 문제

1. **메모리 부족**: 페이지네이션 크기를 줄이세요
2. **느린 응답**: 필터링을 사용하여 필요한 데이터만 조회하세요
3. **데이터베이스 에러**: 스키마 문제인 경우 데이터베이스 마이그레이션을 실행하세요

### 디버깅

1. **로그 확인**: 서버 로그에서 상세한 에러 메시지 확인
2. **파라미터 검증**: 쿼리 파라미터가 올바른 형식인지 확인
3. **데이터베이스 연결**: 데이터베이스 연결 상태 확인

---

**마지막 업데이트**: 2025-01-27  
**문서 버전**: 1.0  
**작성자**: AI Assistant
