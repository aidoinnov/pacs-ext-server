# Project CRUD API

## 개요

프로젝트 생성, 조회, 목록 조회, 활성 프로젝트 조회를 위한 API입니다.

**Base URL**: `/api/projects`

---

## API 엔드포인트

### 1. 프로젝트 생성

새로운 프로젝트를 생성합니다.

**Endpoint**: `POST /api/projects`

**Authentication**: Required (JWT Token)

**Content-Type**: `application/json`

#### Request Body

**Request DTO**: `CreateProjectRequest`

```json
{
  "name": "심장 질환 연구 프로젝트",
  "description": "심장 질환 관련 DICOM 영상 분석 프로젝트",
  "sponsor": "서울대학교병원",
  "start_date": "2025-01-01",
  "end_date": "2025-12-31",
  "auto_complete": false
}
```

**Request Schema**:

| Field | Type | Required | Description | Example |
|-------|------|----------|-------------|---------|
| `name` | string | Yes | 프로젝트 이름 | `"심장 질환 연구 프로젝트"` |
| `description` | string | No | 프로젝트 설명 | `"심장 질환 관련 DICOM 영상 분석 프로젝트"` |
| `sponsor` | string | Yes | 스폰서명 | `"서울대학교병원"` |
| `start_date` | date | Yes | 시작일 | `"2025-01-01"` |
| `end_date` | date | No | 종료일/목표일 | `"2025-12-31"` |
| `auto_complete` | boolean | No | 자동 완료 여부 | `false` |

#### Response

**Success Response** (201 Created)

```json
{
  "id": 1,
  "name": "심장 질환 연구 프로젝트",
  "description": "심장 질환 관련 DICOM 영상 분석 프로젝트",
  "is_active": true,
  "created_at": "2024-01-01T00:00:00Z"
}
```

**Response Schema**:

| Field | Type | Description |
|-------|------|-------------|
| `id` | integer | 프로젝트 ID |
| `name` | string | 프로젝트 이름 |
| `description` | string | 프로젝트 설명 |
| `is_active` | boolean | 활성 상태 (`true` 기본값) |
| `created_at` | string (ISO 8601) | 생성 시간 |

**Error Responses**

| Status Code | Description | Response Body |
|-------------|-------------|---------------|
| 400 | Invalid request | `{"error": "Failed to create project: ..."}` |
| 401 | Unauthorized | Authentication error |

---

### 2. 프로젝트 조회

특정 프로젝트의 정보를 조회합니다.

**Endpoint**: `GET /api/projects/{project_id}`

**Authentication**: Required (JWT Token)

#### Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `project_id` | integer | Yes | 조회할 프로젝트 ID |

#### Response

**Success Response** (200 OK)

```json
{
  "id": 1,
  "name": "심장 질환 연구 프로젝트",
  "description": "심장 질환 관련 DICOM 영상 분석 프로젝트",
  "is_active": true,
  "created_at": "2024-01-01T00:00:00Z"
}
```

**Error Responses**

| Status Code | Description | Response Body |
|-------------|-------------|---------------|
| 404 | Project not found | `{"error": "Project not found: ..."}` |

---

### 3. 프로젝트 수정

프로젝트 정보를 수정합니다.

**Endpoint**: `PUT /api/projects/{project_id}`

**Authentication**: Required (JWT Token)

**Content-Type**: `application/json`

#### Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `project_id` | integer | Yes | 수정할 프로젝트 ID |

#### Request Body

**Request DTO**: `UpdateProjectRequest`

```json
{
  "name": "업데이트된 프로젝트명",
  "description": "업데이트된 설명",
  "sponsor": "업데이트된 스폰서",
  "start_date": "2025-02-01",
  "end_date": "2026-01-31",
  "status": "ACTIVE",
  "auto_complete": true,
  "is_active": true
}
```

**Request Schema**:

| Field | Type | Required | Description | Example |
|-------|------|----------|-------------|---------|
| `name` | string | No | 프로젝트 이름 | `"업데이트된 프로젝트명"` |
| `description` | string | No | 프로젝트 설명 | `"업데이트된 설명"` |
| `sponsor` | string | No | 스폰서명 | `"서울대학교병원"` |
| `start_date` | date | No | 시작일 | `"2025-02-01"` |
| `end_date` | date | No | 종료일/목표일 | `"2026-01-31"` |
| `status` | string | No | 프로젝트 상태 | `"ACTIVE"` |
| `auto_complete` | boolean | No | 자동 완료 여부 | `true` |
| `is_active` | boolean | No | 활성 상태 | `true` |

#### Response

**Success Response** (200 OK)

```json
{
  "id": 1,
  "name": "업데이트된 프로젝트명",
  "description": "업데이트된 설명",
  "sponsor": "서울대학교병원",
  "start_date": "2025-02-01",
  "end_date": "2026-01-31",
  "auto_complete": true,
  "is_active": true,
  "status": "ACTIVE",
  "created_at": "2024-01-01T00:00:00Z"
}
```

**Error Responses**

| Status Code | Description | Response Body |
|-------------|-------------|---------------|
| 400 | Invalid request | `{"error": "Failed to update project: ..."}` |
| 404 | Project not found | `{"error": "Failed to update project: ..."}` |

---

### 4. 프로젝트 목록 조회

모든 프로젝트의 목록을 조회합니다.

**Endpoint**: `GET /api/projects`

**Authentication**: Required (JWT Token)

#### Response

**Success Response** (200 OK)

```json
{
  "projects": [
    {
      "id": 1,
      "name": "심장 질환 연구 프로젝트",
      "description": "심장 질환 관련 DICOM 영상 분석 프로젝트",
      "is_active": true,
      "created_at": "2024-01-01T00:00:00Z"
    },
    {
      "id": 2,
      "name": "뇌졸중 조기 진단 프로젝트",
      "description": "뇌졸중 조기 진단을 위한 영상 분석",
      "is_active": true,
      "created_at": "2024-01-02T00:00:00Z"
    }
  ],
  "total": 2
}
```

**Response Schema**:

| Field | Type | Description |
|-------|------|-------------|
| `projects` | array | 프로젝트 배열 |
| `total` | integer | 전체 프로젝트 개수 |

---

### 4. 활성 프로젝트 목록 조회

활성 상태인 프로젝트만 조회합니다.

**Endpoint**: `GET /api/projects/active`

**Authentication**: Required (JWT Token)

#### Response

**Success Response** (200 OK)

```json
{
  "projects": [
    {
      "id": 1,
      "name": "심장 질환 연구 프로젝트",
      "description": "심장 질환 관련 DICOM 영상 분석 프로젝트",
      "is_active": true,
      "created_at": "2024-01-01T00:00:00Z"
    }
  ],
  "total": 1
}
```

**Response Schema**: Project List Response와 동일

---

## 사용 예시

### cURL 요청 예시

#### 프로젝트 생성

```bash
curl -X POST "http://localhost:8080/api/projects" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "심장 질환 연구 프로젝트",
    "description": "심장 질환 관련 DICOM 영상 분석 프로젝트"
  }'
```

#### 프로젝트 조회

```bash
curl -X GET "http://localhost:8080/api/projects/1" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

#### 프로젝트 목록 조회

```bash
curl -X GET "http://localhost:8080/api/projects" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

#### 활성 프로젝트 목록 조회

```bash
curl -X GET "http://localhost:8080/api/projects/active" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

### JavaScript (fetch) 예시

```javascript
// 프로젝트 생성
async function createProject(projectData) {
  try {
    const response = await fetch('http://localhost:8080/api/projects', {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${yourJwtToken}`,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        name: projectData.name,
        description: projectData.description
      })
    });

    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }

    const project = await response.json();
    console.log('Project created:', project);
    return project;
  } catch (error) {
    console.error('Error creating project:', error);
    throw error;
  }
}

// 사용 예시
createProject({
  name: '심장 질환 연구 프로젝트',
  description: '심장 질환 관련 DICOM 영상 분석 프로젝트'
});

// 프로젝트 조회
async function getProject(projectId) {
  const response = await fetch(`http://localhost:8080/api/projects/${projectId}`, {
    headers: {
      'Authorization': `Bearer ${yourJwtToken}`
    }
  });
  return response.json();
}

// 프로젝트 목록 조회
async function listProjects() {
  const response = await fetch('http://localhost:8080/api/projects', {
    headers: {
      'Authorization': `Bearer ${yourJwtToken}`
    }
  });
  return response.json();
}

// 활성 프로젝트 목록 조회
async function getActiveProjects() {
  const response = await fetch('http://localhost:8080/api/projects/active', {
    headers: {
      'Authorization': `Bearer ${yourJwtToken}`
    }
  });
  return response.json();
}
```

### TypeScript 예시

```typescript
interface CreateProjectRequest {
  name: string;
  description?: string;
}

interface ProjectResponse {
  id: number;
  name: string;
  description: string | null;
  is_active: boolean;
  created_at: string;
}

interface ProjectListResponse {
  projects: ProjectResponse[];
  total: number;
}

// 프로젝트 생성
async function createProject(
  request: CreateProjectRequest
): Promise<ProjectResponse> {
  const response = await fetch('http://localhost:8080/api/projects', {
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${getJwtToken()}`,
      'Content-Type': 'application/json'
    },
    body: JSON.stringify(request)
  });

  if (!response.ok) {
    const error = await response.json();
    throw new Error(error.error || 'Failed to create project');
  }

  return response.json();
}

// 프로젝트 조회
async function getProject(projectId: number): Promise<ProjectResponse> {
  const response = await fetch(
    `http://localhost:8080/api/projects/${projectId}`,
    {
      headers: {
        'Authorization': `Bearer ${getJwtToken()}`
      }
    }
  );

  if (!response.ok) {
    throw new Error('Failed to get project');
  }

  return response.json();
}

// 프로젝트 목록 조회
async function listProjects(): Promise<ProjectListResponse> {
  const response = await fetch('http://localhost:8080/api/projects', {
    headers: {
      'Authorization': `Bearer ${getJwtToken()}`
    }
  });

  if (!response.ok) {
    throw new Error('Failed to list projects');
  }

  return response.json();
}

// 활성 프로젝트 목록 조회
async function getActiveProjects(): Promise<ProjectListResponse> {
  const response = await fetch('http://localhost:8080/api/projects/active', {
    headers: {
      'Authorization': `Bearer ${getJwtToken()}`
    }
  });

  if (!response.ok) {
    throw new Error('Failed to get active projects');
  }

  return response.json();
}
```

---

## 프로젝트 상태

### status 필드

프로젝트의 생명주기 상태를 나타냅니다.

- `PLANNING`: 기획중 - 프로젝트가 기획 단계
- `ACTIVE`: 진행중 - 프로젝트가 활발히 진행 중
- `COMPLETED`: 완료 - 프로젝트가 성공적으로 완료됨
- `SUSPENDED`: 보류 - 프로젝트가 일시적으로 중단됨
- `CANCELLED`: 취소 - 프로젝트가 취소됨
- `PENDING_COMPLETION`: 완료 대기 - 프로젝트 종료 대기 중
- `OVER_PLANNING`: 계획 초과 - 프로젝트 계획 초과 상태

### is_active 필드

프로젝트의 아카이브 상태를 나타냅니다. `status`와는 별개로 관리됩니다.

- `true`: 프로젝트가 활성 상태 (기본값)
- `false`: 프로젝트가 비활성/아카이브 상태

### auto_complete 필드

프로젝트의 자동 완료 여부를 나타냅니다.

- `true`: 종료일(end_date) 도달 시 자동으로 완료 상태로 전환
- `false`: 수동으로만 완료 처리 (기본값)

### 활성 프로젝트 필터링

활성 프로젝트만 조회하려면 `/api/projects/active` 엔드포인트를 사용하세요.

---

## 구현 세부사항

### 레이어별 구현

#### 1. Controller (`project_controller.rs`)

```rust
// 프로젝트 생성
pub async fn create_project<P: ProjectService>(
    project_use_case: web::Data<Arc<ProjectUseCase<P>>>,
    req: web::Json<CreateProjectRequest>,
) -> impl Responder {
    match project_use_case.create_project(req.into_inner()).await {
        Ok(project) => HttpResponse::Created().json(project),
        Err(e) => HttpResponse::BadRequest().json(json!({
            "error": format!("Failed to create project: {}", e)
        })),
    }
}

// 프로젝트 조회
pub async fn get_project<P: ProjectService>(
    project_use_case: web::Data<Arc<ProjectUseCase<P>>>,
    project_id: web::Path<i32>,
) -> impl Responder {
    match project_use_case.get_project(*project_id).await {
        Ok(project) => HttpResponse::Ok().json(project),
        Err(e) => HttpResponse::NotFound().json(json!({
            "error": format!("Project not found: {}", e)
        })),
    }
}

// 프로젝트 목록 조회
pub async fn list_projects<P: ProjectService>(
    project_use_case: web::Data<Arc<ProjectUseCase<P>>>,
) -> impl Responder {
    match project_use_case.get_all_projects().await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to list projects: {}", e)
        })),
    }
}

// 활성 프로젝트 목록 조회
pub async fn get_active_projects<P: ProjectService>(
    project_use_case: web::Data<Arc<ProjectUseCase<P>>>,
) -> impl Responder {
    match project_use_case.get_active_projects().await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to get active projects: {}", e)
        })),
    }
}
```

#### 2. Use Case (`project_use_case.rs`)

```rust
pub async fn create_project(&self, request: CreateProjectRequest) -> Result<ProjectResponse, ServiceError> {
    let new_project = NewProject::new(request.name)
        .with_description(request.description);

    let project = self.project_service.create_project(new_project).await?;
    Ok(project.into())
}

pub async fn get_project(&self, project_id: i32) -> Result<ProjectResponse, ServiceError> {
    let project = self.project_service.get_project(project_id).await?;
    Ok(project.into())
}

pub async fn get_all_projects(&self) -> Result<ProjectListResponse, ServiceError> {
    let projects = self.project_service.get_all_projects().await?;
    Ok(ProjectListResponse {
        projects: projects.into_iter().map(|p| p.into()).collect(),
        total: projects.len(),
    })
}

pub async fn get_active_projects(&self) -> Result<ProjectListResponse, ServiceError> {
    let projects = self.project_service.get_active_projects().await?;
    Ok(ProjectListResponse {
        projects: projects.into_iter().map(|p| p.into()).collect(),
        total: projects.len(),
    })
}
```

#### 3. Repository (`project_repository_impl.rs`)

```rust
async fn create(&self, new_project: &NewProject) -> Result<Project, sqlx::Error> {
    sqlx::query_as::<_, Project>(
        "INSERT INTO security_project (name, description, is_active)
         VALUES ($1, $2, true)
         RETURNING id, name, description, is_active, created_at"
    )
    .bind(&new_project.name)
    .bind(&new_project.description)
    .fetch_one(&self.pool)
    .await
}

async fn find_by_id(&self, id: i32) -> Result<Option<Project>, sqlx::Error> {
    sqlx::query_as::<_, Project>(
        "SELECT id, name, description, status, created_at
         FROM security_project
         WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&self.pool)
    .await
}

async fn find_all(&self) -> Result<Vec<Project>, sqlx::Error> {
    sqlx::query_as::<_, Project>(
        "SELECT id, name, description, status, created_at
         FROM security_project
         ORDER BY id ASC"
    )
    .fetch_all(&self.pool)
    .await
}

async fn find_active(&self) -> Result<Vec<Project>, sqlx::Error> {
    sqlx::query_as::<_, Project>(
        "SELECT id, name, description, status, created_at
         FROM security_project
         WHERE status = 'ACTIVE'
         ORDER BY id ASC"
    )
    .fetch_all(&self.pool)
    .await
}
```

---

## 에러 처리

### 공통 에러 응답 형식

```json
{
  "error": "[에러 메시지]"
}
```

### 에러 코드별 설명

| HTTP Status | 에러 내용 | 설명 |
|-------------|---------|------|
| 400 | Invalid request | 요청 데이터가 유효하지 않음 |
| 401 | Unauthorized | 인증되지 않은 요청 |
| 404 | Project not found | 프로젝트가 존재하지 않음 |
| 500 | Internal server error | 서버 내부 오류 |

---

## 테스트 예시

### Swagger UI에서 테스트

1. `http://localhost:8080/swagger-ui/` 접속
2. `/api/projects` 관련 엔드포인트 선택
3. "Authorize" 버튼 클릭하여 JWT 토큰 입력
4. Request Body에 데이터 입력 (POST 요청인 경우)
5. "Execute" 버튼 클릭

### 로컬에서 테스트

```bash
# 프로젝트 생성
curl -X POST "http://localhost:8080/api/projects" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "심장 질환 연구 프로젝트",
    "description": "심장 질환 관련 DICOM 영상 분석 프로젝트"
  }'

# 프로젝트 조회
curl -X GET "http://localhost:8080/api/projects/1" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"

# 프로젝트 목록 조회
curl -X GET "http://localhost:8080/api/projects" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"

# 활성 프로젝트 목록 조회
curl -X GET "http://localhost:8080/api/projects/active" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

---

## 제한사항 및 참고사항

### 현재 미구현 기능

1. **프로젝트 수정 (Update)**: 향후 구현 예정
2. **프로젝트 삭제 (Delete)**: 향후 구현 예정
3. **프로젝트 상태 변경**: 비활성화 기능 향후 구현 예정
4. **페이지네이션**: 현재는 전체 목록 반환 (향후 구현 예정)
5. **검색 기능**: 프로젝트 이름으로 검색 기능 향후 구현 예정

### 보안 고려사항

1. **인증**: 모든 요청은 유효한 JWT 토큰이 필요합니다.
2. **권한 부여**: 현재는 모든 인증된 사용자가 프로젝트를 생성할 수 있습니다. (향후 역할 기반 권한 제어 구현 예정)
3. **SQL Injection 방지**: Prepared statement를 사용하여 SQL Injection을 방지합니다.

### 성능 고려사항

1. **인덱스 활용**: `id`, `status` 컬럼에 인덱스가 있어 빠른 조회가 가능합니다.
2. **정렬**: 기본적으로 `id ASC` 순으로 정렬됩니다.

---

## 관련 API

- [프로젝트 멤버 관리 API](./project-member-management-api-client-guide.md)
- [프로젝트 데이터 접근 매트릭스 API](./project-data-access-matrix-api.md)
- [프로젝트-사용자 매트릭스 API](./user-centered-matrix-api-client-guide.md)

---

**최종 업데이트**: 2025-01-27

