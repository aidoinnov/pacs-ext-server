# 기술 문서: 사용자 프로젝트 목록 API에 기한 정보 추가

## 개요
사용자 프로젝트 목록 API의 응답에 프로젝트 기한 정보를 추가하기 위한 기술적 구현 내용을 문서화합니다.

## 아키텍처 개요

### 계층별 수정 사항
```
┌─────────────────────────────────────────────────┐
│ Presentation Layer                              │
│ (변경 없음 - 자동 반영)                         │
└─────────────────┬───────────────────────────────┘
                  │
┌─────────────────▼───────────────────────────────┐
│ Application Layer                               │
│ (변경 없음 - 자동 반영)                         │
│ - ProjectUserUseCase                            │
│ - UserProjectsResponse                          │
└─────────────────┬───────────────────────────────┘
                  │
┌─────────────────▼───────────────────────────────┐
│ Domain Layer                                    │
│ ✅ Service 수정 필요                            │
│ - UserService::get_user_projects_with_roles()  │
└─────────────────┬───────────────────────────────┘
                  │
┌─────────────────▼───────────────────────────────┐
│ Infrastructure Layer                            │
│ (변경 없음)                                     │
│ - Database (기존 컬럼 사용)                     │
└─────────────────────────────────────────────────┘
```

## 데이터 흐름

### 기존 데이터 흐름
```
1. Client Request
   ↓
2. Controller (GET /api/users/{user_id}/projects)
   ↓
3. Use Case (get_user_projects_with_roles)
   ↓
4. User Service (get_user_projects_with_roles)
   ↓
5. Database Query
   - SELECT: project_id, project_name, description, is_active, role_id, role_name, role_scope
   ↓
6. DTO 변환
   ↓
7. Response (ProjectWithRoleResponse)
```

### 수정된 데이터 흐름
```
1. Client Request
   ↓
2. Controller (GET /api/users/{user_id}/projects)
   ↓
3. Use Case (get_user_projects_with_roles)
   ↓
4. User Service (get_user_projects_with_roles)
   ↓
5. Database Query
   - SELECT: project_id, project_name, description, is_active, 
            start_date, end_date, role_id, role_name, role_scope
   ↓
6. DTO 변환 (start_date, end_date 포함)
   ↓
7. Response (ProjectWithRoleResponse with deadline info)
```

## 기술적 구현 상세

### 1. DTO 구조 변경

#### 변경 전
```rust
pub struct ProjectWithRoleResponse {
    pub project_id: i32,
    pub project_name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub role_id: Option<i32>,
    pub role_name: Option<String>,
    pub role_scope: Option<String>,
}
```

#### 변경 후
```rust
pub struct ProjectWithRoleResponse {
    pub project_id: i32,
    pub project_name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub start_date: Option<String>,      // 추가
    pub end_date: Option<String>,        // 추가
    pub role_id: Option<i32>,
    pub role_name: Option<String>,
    pub role_scope: Option<String>,
}
```

### 2. SQL 쿼리 변경

#### 변경 전
```sql
SELECT 
    p.id as project_id, 
    p.name as project_name, 
    p.description, 
    p.is_active,
    r.id as role_id, 
    r.name as role_name, 
    r.scope as role_scope
FROM security_project p
INNER JOIN security_user_project up ON p.id = up.project_id
LEFT JOIN security_role r ON up.role_id = r.id
WHERE up.user_id = $1
ORDER BY p.name
LIMIT $2 OFFSET $3
```

#### 변경 후
```sql
SELECT 
    p.id as project_id, 
    p.name as project_name, 
    p.description, 
    p.is_active,
    p.start_date,        -- 추가
    p.end_date,           -- 추가
    r.id as role_id, 
    r.name as role_name, 
    r.scope as role_scope
FROM security_project p
INNER JOIN security_user_project up ON p.id = up.project_id
LEFT JOIN security_role r ON up.role_id = r.id
WHERE up.user_id = $1
ORDER BY p.name
LIMIT $2 OFFSET $3
```

### 3. Rust 타입 변경

#### 변경 전
```rust
let projects_with_roles = sqlx::query_as::<_, (
    i32,                    // project_id
    String,                 // project_name
    Option<String>,         // description
    bool,                   // is_active
    Option<i32>,            // role_id
    Option<String>,         // role_name
    Option<String>,         // role_scope
)>(...)
```

#### 변경 후
```rust
let projects_with_roles = sqlx::query_as::<_, (
    i32,                    // project_id
    String,                 // project_name
    Option<String>,         // description
    bool,                   // is_active
    Option<String>,         // start_date (추가)
    Option<String>,         // end_date (추가)
    Option<i32>,            // role_id
    Option<String>,         // role_name
    Option<String>,         // role_scope
)>(...)
```

### 4. 매핑 로직 변경

#### 변경 전
```rust
.map(|(project_id, project_name, description, is_active, role_id, role_name, role_scope)| {
    ProjectWithRoleResponse {
        project_id,
        project_name,
        description,
        is_active,
        role_id,
        role_name,
        role_scope,
    }
})
```

#### 변경 후
```rust
.map(|(project_id, project_name, description, is_active, start_date, end_date, role_id, role_name, role_scope)| {
    ProjectWithRoleResponse {
        project_id,
        project_name,
        description,
        is_active,
        start_date,
        end_date,
        role_id,
        role_name,
        role_scope,
    }
})
```

## 데이터베이스 스키마

### security_project 테이블
```sql
CREATE TABLE security_project (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    sponsor VARCHAR(255),
    start_date DATE,           -- 사용됨
    end_date DATE,             -- 사용됨
    auto_complete BOOLEAN DEFAULT false,
    is_active BOOLEAN DEFAULT true,
    status VARCHAR(50) DEFAULT 'PLANNING',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

## 타입 안전성

### Option 타입 사용 이유
- `start_date`, `end_date`는 NULL 가능한 컬럼
- 프로젝트 생성 시 기한이 설정되지 않을 수 있음
- 클라이언트 측에서 유연한 처리 가능

### Rust의 Option 타입
```rust
pub start_date: Option<String>,  // Some("2025-01-01") 또는 None
pub end_date: Option<String>,    // Some("2025-12-31") 또는 None
```

## API 스펙

### 엔드포인트
- **Method**: `GET`
- **Path**: `/api/users/{user_id}/projects`
- **Authentication**: Required (JWT Token)

### 쿼리 파라미터
| 파라미터 | 타입 | 기본값 | 설명 |
|----------|------|--------|------|
| page | integer | 1 | 페이지 번호 |
| page_size | integer | 20 | 페이지 크기 |

### 응답 스펙
```json
{
  "projects": [
    {
      "project_id": 1,
      "project_name": "string",
      "description": "string",
      "is_active": true,
      "start_date": "2025-01-01",      // 추가
      "end_date": "2025-12-31",        // 추가
      "role_id": 2,
      "role_name": "PROJECT_ADMIN",
      "role_scope": "PROJECT"
    }
  ],
  "total_count": 1,
  "page": 1,
  "page_size": 20,
  "total_pages": 1
}
```

## 성능 고려사항

### 인덱스 활용
- `security_project.start_date`, `security_project.end_date` 컬럼에 별도 인덱스 불필요 (조회 조건 아님)
- 기존 JOIN 인덱스 유지 (`security_user_project` 테이블)

### 쿼리 복잡도
- **Before**: O(n) where n = number of projects for user
- **After**: O(n) - 동일 (추가 컬럼만 조회)
- 성능 영향: **최소** (추가 컬럼 2개만 증가)

## 호환성

### 하위 호환성
- ✅ 기존 클라이언트와 호환 (Optional 필드 추가)
- ✅ 기존 API 응답 구조 유지 (필드 추가만)

### 직렬화 호환성
```rust
// serde는 누락된 필드를 기본값(Option::None)으로 처리
// JSON 역직렬화 시 누락된 필드는 자동으로 None 처리
```

## 에러 처리

### NULL 값 처리
- 데이터베이스의 NULL 값은 Rust의 `None`으로 매핑
- JSON에서 `null` 또는 필드 누락 모두 `None`으로 처리

### 예시
```json
{
  "project_id": 1,
  "project_name": "프로젝트명",
  "start_date": null,    // 또는 필드 누락
  "end_date": "2025-12-31"
}
```

## 테스트 케이스

### 테스트 시나리오
1. 기한이 설정된 프로젝트
2. 기한이 설정되지 않은 프로젝트 (NULL)
3. 시작일만 있는 프로젝트
4. 종료일만 있는 프로젝트

### 예상 결과
```json
// Case 1: 기한 설정됨
{
  "start_date": "2025-01-01",
  "end_date": "2025-12-31"
}

// Case 2: 기한 없음
{
  "start_date": null,
  "end_date": null
}

// Case 3: 시작일만
{
  "start_date": "2025-01-01",
  "end_date": null
}
```

## 유지보수

### 변경 이력 추적
- DTO 필드 추가는 명확하게 추적 가능
- SQL 쿼리는 주석으로 변경 사유 명시

### 향후 확장 가능성
- 동일한 패턴으로 다른 프로젝트 정보 추가 가능
- 데이터베이스 스키마 변경 없이 쿼리만 수정

## 결론
- 최소한의 변경으로 요구사항 충족
- 하위 호환성 유지
- 성능 영향 최소화
- 유지보수 용이

