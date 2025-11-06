# PACS 서버 Project CRUD 시스템 기술 문서

## 📋 개요

이 문서는 PACS 서버의 Project 관리 시스템에 대한 기술적 구현과 아키텍처를 설명합니다. Clean Architecture 패턴을 따라 구현된 프로젝트 생성, 조회, 업데이트, 삭제(CRUD) 기능을 중심으로 다룹니다.

## 🏗️ 아키텍처 개요

### Clean Architecture 계층 구조

```
┌─────────────────────────────────────────────────────────────┐
│                    Presentation Layer                       │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │              ProjectController                          │ │
│  │  - create_project()                                     │ │
│  │  - get_project()                                        │ │
│  │  - list_projects()                                      │ │
│  │  - get_active_projects()                                │ │
│  └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│                    Application Layer                        │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │              ProjectUseCase                             │ │
│  │  - create_project()                                     │ │
│  │  - get_project()                                        │ │
│  │  - get_all_projects()                                   │ │
│  │  - get_active_projects()                                │ │
│  │  - activate_project()                                   │ │
│  │  - deactivate_project()                                 │ │
│  │  - delete_project()                                     │ │
│  └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│                      Domain Layer                           │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │              ProjectService                             │ │
│  │  - create_project()                                     │ │
│  │  - get_project()                                        │ │
│  │  - get_all_projects()                                   │ │
│  │  - get_active_projects()                                │ │
│  │  - activate_project()                                   │ │
│  │  - deactivate_project()                                 │ │
│  │  - delete_project()                                     │ │
│  └─────────────────────────────────────────────────────────┘ │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │              ProjectRepository                          │ │
│  │  - find_by_id()                                         │ │
│  │  - find_by_name()                                       │ │
│  │  - find_all()                                           │ │
│  │  - find_active()                                        │ │
│  │  - create()                                             │ │
│  │  - update()                                             │ │
│  │  - set_active()                                         │ │
│  │  - delete()                                             │ │
│  └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│                  Infrastructure Layer                       │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │              ProjectRepositoryImpl                      │ │
│  │  - PostgreSQL 연결                                      │ │
│  │  - SQL 쿼리 실행                                        │ │
│  │  - 데이터 매핑                                          │ │
│  └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

## 📊 Project 엔티티 및 데이터 모델

### Project 엔티티 구조

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Project {
    /// 데이터베이스에서 자동 생성되는 고유 식별자
    pub id: i32,
    /// 프로젝트의 고유한 이름
    pub name: String,
    /// 프로젝트에 대한 상세 설명 (선택사항)
    pub description: Option<String>,
    /// 프로젝트 활성화 상태 (true: 활성, false: 비활성)
    pub is_active: bool,
    /// 프로젝트의 생명주기 상태
    pub status: ProjectStatus,
    /// 프로젝트가 생성된 시각
    pub created_at: DateTime<Utc>,
}
```

### ProjectStatus 열거형

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
#[sqlx(type_name = "project_status", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProjectStatus {
    /// 준비중 - 프로젝트가 생성되었지만 아직 시작되지 않음
    Preparing,
    /// 진행중 - 프로젝트가 활발히 진행 중
    InProgress,
    /// 완료 - 프로젝트가 성공적으로 완료됨
    Completed,
    /// 보류 - 프로젝트가 일시적으로 중단됨
    OnHold,
    /// 취소 - 프로젝트가 취소됨
    Cancelled,
}
```

### NewProject DTO

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewProject {
    /// 생성할 프로젝트명 (중복되지 않아야 함)
    pub name: String,
    /// 프로젝트에 대한 상세 설명 (선택사항)
    pub description: Option<String>,
}
```

## 🔄 Project CRUD 작업 상세

### 1. 프로젝트 생성 (Create)

#### API 엔드포인트
```
POST /api/projects
```

#### 요청 DTO
```rust
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: Option<String>,
}
```

#### 구현 흐름 (Domain Service)
```rust
impl<P, U, R> ProjectService for ProjectServiceImpl<P, U, R>
where
    P: ProjectRepository,
    U: UserRepository,
    R: RoleRepository,
{
    async fn create_project(
        &self,
        name: String,
        description: Option<String>,
    ) -> Result<Project, ServiceError> {
        // 프로젝트 이름 중복 체크
        if let Some(_) = self.project_repository.find_by_name(&name).await? {
            return Err(ServiceError::AlreadyExists(
                "Project name already exists".into(),
            ));
        }

        // 프로젝트 이름 검증
        if name.trim().is_empty() {
            return Err(ServiceError::ValidationError(
                "Project name cannot be empty".into(),
            ));
        }

        if name.len() > 255 {
            return Err(ServiceError::ValidationError(
                "Project name too long (max 255 characters)".into(),
            ));
        }

        let new_project = NewProject { name, description };

        Ok(self.project_repository.create(new_project).await?)
    }
}
```

#### 데이터베이스 쿼리 (Repository)
```sql
INSERT INTO security_project (name, description)
VALUES ($1, $2)
RETURNING id, name, description, is_active, status, created_at
```

#### 컨트롤러 구현
```rust
#[utoipa::path(
    post,
    path = "/api/projects",
    tag = "projects",
    request_body = CreateProjectRequest,
    responses(
        (status = 201, description = "Project created successfully", body = ProjectResponse),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
    )
)]
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
```

### 2. 프로젝트 조회 (Read)

#### API 엔드포인트
```
GET /api/projects/{project_id}     # 특정 프로젝트 조회
GET /api/projects                   # 모든 프로젝트 조회
GET /api/projects/active            # 활성화된 프로젝트만 조회
```

#### 구현 흐름 (Domain Service)
```rust
async fn get_project(&self, id: i32) -> Result<Project, ServiceError> {
    self.project_repository
        .find_by_id(id)
        .await?
        .ok_or(ServiceError::NotFound("Project not found".into()))
}

async fn get_all_projects(&self) -> Result<Vec<Project>, ServiceError> {
    Ok(self.project_repository.find_all().await?)
}

async fn get_active_projects(&self) -> Result<Vec<Project>, ServiceError> {
    Ok(self.project_repository.find_active().await?)
}
```

#### 데이터베이스 쿼리 (Repository)
```sql
-- ID로 조회
SELECT id, name, description, is_active, status, created_at
FROM security_project 
WHERE id = $1

-- 모든 프로젝트 조회
SELECT id, name, description, is_active, status, created_at
FROM security_project
ORDER BY created_at DESC

-- 활성화된 프로젝트만 조회
SELECT id, name, description, is_active, status, created_at
FROM security_project
WHERE is_active = true
ORDER BY created_at DESC
```

#### 컨트롤러 구현
```rust
#[utoipa::path(
    get,
    path = "/api/projects/{project_id}",
    tag = "projects",
    params(
        ("project_id" = i32, Path, description = "Project ID")
    ),
    responses(
        (status = 200, description = "Project retrieved successfully", body = ProjectResponse),
        (status = 404, description = "Project not found"),
    )
)]
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
```

### 3. 프로젝트 업데이트 (Update)

#### API 엔드포인트
```
PUT /api/projects/{project_id}      # 프로젝트 정보 업데이트
POST /api/projects/{project_id}/activate   # 프로젝트 활성화
POST /api/projects/{project_id}/deactivate # 프로젝트 비활성화
```

#### 요청 DTO
```rust
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}
```

#### 구현 흐름 (Domain Service)
```rust
async fn activate_project(&self, id: i32) -> Result<Project, ServiceError> {
    // 프로젝트 존재 확인
    let project = self.get_project(id).await?;
    
    // 활성화 상태 변경
    self.project_repository.set_active(id, true).await?;
    
    // 업데이트된 프로젝트 반환
    self.get_project(id).await
}

async fn deactivate_project(&self, id: i32) -> Result<Project, ServiceError> {
    // 프로젝트 존재 확인
    let project = self.get_project(id).await?;
    
    // 비활성화 상태 변경
    self.project_repository.set_active(id, false).await?;
    
    // 업데이트된 프로젝트 반환
    self.get_project(id).await
}
```

#### 데이터베이스 쿼리 (Repository)
```sql
-- 프로젝트 정보 업데이트
UPDATE security_project
SET name = $2, description = $3
WHERE id = $1
RETURNING id, name, description, is_active, status, created_at

-- 활성화 상태 변경
UPDATE security_project 
SET is_active = $2 
WHERE id = $1
```

### 4. 프로젝트 삭제 (Delete)

#### API 엔드포인트
```
DELETE /api/projects/{project_id}
```

#### 구현 흐름 (Domain Service)
```rust
async fn delete_project(&self, id: i32) -> Result<(), ServiceError> {
    // 프로젝트 존재 확인
    let project = self.get_project(id).await?;
    
    // 프로젝트 삭제
    let deleted = self.project_repository.delete(id).await?;
    
    if !deleted {
        return Err(ServiceError::NotFound("Project not found".into()));
    }
    
    Ok(())
}
```

#### 데이터베이스 쿼리 (Repository)
```sql
DELETE FROM security_project WHERE id = $1
```

## 📡 API 라우팅 구조

### Project 관련 API 라우팅
```rust
pub fn configure_routes<P: ProjectService + 'static>(
    cfg: &mut web::ServiceConfig,
    project_use_case: Arc<ProjectUseCase<P>>,
) {
    cfg.app_data(web::Data::new(project_use_case))
        .service(
            web::scope("/projects")
                .route("", web::post().to(create_project::<P>))           // POST /api/projects
                .route("", web::get().to(list_projects::<P>))             // GET /api/projects
                .route("/active", web::get().to(get_active_projects::<P>)) // GET /api/projects/active
                .route("/{project_id}", web::get().to(get_project::<P>)), // GET /api/projects/{id}
        );
}
```

### 전체 API 라우팅 순서
```rust
web::scope("/api")
    // 🔐 인증 관련 API (가장 먼저 등록)
    .configure(|cfg| auth_controller::configure_routes(...))
    
    // 👥 사용자 관리 API
    .configure(|cfg| user_controller::configure_routes(...))
    
    // 🏗️ 프로젝트 관리 API
    .configure(|cfg| project_controller::configure_routes(...))
    
    // 🔑 권한 관리 API
    .configure(|cfg| role_permission_matrix_controller::configure_routes(...))
```

## 🗄️ 데이터베이스 스키마

### security_project 테이블
```sql
CREATE TABLE security_project (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) UNIQUE NOT NULL,
    description TEXT,
    is_active BOOLEAN DEFAULT true,
    status project_status DEFAULT 'PREPARING',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- 인덱스 생성
CREATE INDEX idx_security_project_name ON security_project(name);
CREATE INDEX idx_security_project_active ON security_project(is_active);
CREATE INDEX idx_security_project_status ON security_project(status);
```

### project_status ENUM 타입
```sql
CREATE TYPE project_status AS ENUM (
    'PREPARING',
    'IN_PROGRESS', 
    'COMPLETED',
    'ON_HOLD',
    'CANCELLED'
);
```

## 🔄 Repository 패턴 구현

### ProjectRepository 트레이트
```rust
#[async_trait]
pub trait ProjectRepository: Send + Sync {
    async fn find_by_id(&self, id: i32) -> Result<Option<Project>, sqlx::Error>;
    async fn find_by_name(&self, name: &str) -> Result<Option<Project>, sqlx::Error>;
    async fn find_all(&self) -> Result<Vec<Project>, sqlx::Error>;
    async fn find_active(&self) -> Result<Vec<Project>, sqlx::Error>;
    async fn create(&self, new_project: NewProject) -> Result<Project, sqlx::Error>;
    async fn update(&self, id: i32, new_project: NewProject) -> Result<Option<Project>, sqlx::Error>;
    async fn set_active(&self, id: i32, is_active: bool) -> Result<bool, sqlx::Error>;
    async fn delete(&self, id: i32) -> Result<bool, sqlx::Error>;
    fn pool(&self) -> &PgPool;
}
```

### ProjectRepositoryImpl 구현
```rust
pub struct ProjectRepositoryImpl {
    pool: PgPool,
}

impl ProjectRepositoryImpl {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProjectRepository for ProjectRepositoryImpl {
    async fn create(&self, new_project: NewProject) -> Result<Project, sqlx::Error> {
        sqlx::query_as::<_, Project>(
            "INSERT INTO security_project (name, description)
             VALUES ($1, $2)
             RETURNING id, name, description, is_active, status, created_at"
        )
        .bind(new_project.name)
        .bind(new_project.description)
        .fetch_one(&self.pool)
        .await
    }

    async fn find_by_id(&self, id: i32) -> Result<Option<Project>, sqlx::Error> {
        sqlx::query_as::<_, Project>(
            "SELECT id, name, description, is_active, status, created_at
             FROM security_project 
             WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    async fn find_all(&self) -> Result<Vec<Project>, sqlx::Error> {
        sqlx::query_as::<_, Project>(
            "SELECT id, name, description, is_active, status, created_at
             FROM security_project
             ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await
    }

    async fn find_active(&self) -> Result<Vec<Project>, sqlx::Error> {
        sqlx::query_as::<_, Project>(
            "SELECT id, name, description, is_active, status, created_at
             FROM security_project
             WHERE is_active = true
             ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await
    }

    async fn delete(&self, id: i32) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM security_project WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    fn pool(&self) -> &PgPool {
        &self.pool
    }
}
```

## 🛡️ 보안 및 검증

### 1. 입력 검증
```rust
// 프로젝트 이름 검증
if name.trim().is_empty() {
    return Err(ServiceError::ValidationError(
        "Project name cannot be empty".into(),
    ));
}

if name.len() > 255 {
    return Err(ServiceError::ValidationError(
        "Project name too long (max 255 characters)".into(),
    ));
}
```

### 2. 중복 체크
```rust
// 프로젝트 이름 중복 체크
if let Some(_) = self.project_repository.find_by_name(&name).await? {
    return Err(ServiceError::AlreadyExists(
        "Project name already exists".into(),
    ));
}
```

### 3. 존재 여부 확인
```rust
// 프로젝트 존재 확인
let project = self.project_repository
    .find_by_id(id)
    .await?
    .ok_or(ServiceError::NotFound("Project not found".into()))?;
```

## 📊 DTO 변환

### ProjectResponse DTO
```rust
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ProjectResponse {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
    #[schema(value_type = String, example = "2024-01-01T00:00:00")]
    pub created_at: DateTime<Utc>,
}

impl From<crate::domain::entities::project::Project> for ProjectResponse {
    fn from(project: crate::domain::entities::project::Project) -> Self {
        Self {
            id: project.id,
            name: project.name,
            description: project.description,
            is_active: project.is_active,
            created_at: project.created_at,
        }
    }
}
```

### ProjectListResponse DTO
```rust
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ProjectListResponse {
    pub projects: Vec<ProjectResponse>,
    pub total: usize,
}
```

## 🧪 테스트 전략

### 단위 테스트 예시
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::project::{Project, ProjectStatus};

    #[tokio::test]
    async fn test_create_project() {
        let pool = create_test_pool().await;
        let repo = ProjectRepositoryImpl::new(pool);
        
        let new_project = NewProject {
            name: "Test Project".to_string(),
            description: Some("Test Description".to_string()),
        };
        
        let result = repo.create(new_project).await;
        assert!(result.is_ok());
        
        let project = result.unwrap();
        assert_eq!(project.name, "Test Project");
        assert_eq!(project.is_active, true);
        assert_eq!(project.status, ProjectStatus::Preparing);
    }

    #[tokio::test]
    async fn test_find_project_by_id() {
        let pool = create_test_pool().await;
        let repo = ProjectRepositoryImpl::new(pool);
        
        // 프로젝트 생성
        let new_project = NewProject {
            name: "Test Project".to_string(),
            description: None,
        };
        let created = repo.create(new_project).await.unwrap();
        
        // ID로 조회
        let found = repo.find_by_id(created.id).await.unwrap();
        assert!(found.is_some());
        
        let project = found.unwrap();
        assert_eq!(project.id, created.id);
        assert_eq!(project.name, "Test Project");
    }
}
```

## 🚀 성능 최적화

### 1. 데이터베이스 최적화
- **인덱스 설정**: `name`, `is_active`, `status` 컬럼에 인덱스
- **쿼리 최적화**: 필요한 컬럼만 SELECT
- **연결 풀**: PostgreSQL 연결 풀 사용

### 2. 메모리 최적화
- **Arc<T>** 사용으로 참조 카운팅
- **Clone** 최소화
- **String vs &str** 적절한 사용

### 3. 비동기 처리
- **async/await** 패턴 사용
- **tokio** 런타임 활용
- **병렬 처리** 가능한 작업 분리

## 📝 결론

PACS 서버의 Project CRUD 시스템은 다음과 같은 특징을 가집니다:

1. **Clean Architecture** 패턴을 따른 계층화된 구조
2. **Repository 패턴**을 통한 데이터 접근 추상화
3. **PostgreSQL** 기반의 안정적인 데이터 저장
4. **Rust**의 타입 안전성과 성능 최적화
5. **입력 검증** 및 **중복 체크**를 통한 데이터 무결성 보장
6. **OpenAPI** 문서화를 통한 API 명세

이 시스템은 의료 영상 관리 환경에서 요구되는 프로젝트 관리 기능을 제공하며, 확장 가능한 아키텍처를 통해 향후 기능 추가가 용이합니다.

