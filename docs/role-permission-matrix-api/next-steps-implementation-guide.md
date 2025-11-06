# Role-Permission Matrix API 다음 단계 구현 가이드

## 📋 개요

이 문서는 현재 구현된 Role-Permission Matrix API를 기반으로 다음 단계의 기능을 구현하는 가이드입니다. Clean Architecture 패턴을 유지하면서 단계적으로 기능을 확장하는 방법을 설명합니다.

## 🎯 현재 구현된 기능

### ✅ 완료된 기능
- **글로벌 롤-권한 매트릭스 조회**: `GET /api/roles/global/permissions/matrix`
- **권한 할당/제거**: `PUT /api/roles/{role_id}/permissions/{permission_id}`
- **데이터베이스 스키마**: `security_roles`, `security_permissions`, `security_role_permission` 테이블
- **Clean Architecture 구조**: Domain, Application, Infrastructure, Presentation 계층
- **OpenAPI 문서화**: Swagger UI 지원
- **JWT 인증**: Bearer Token 기반 인증

## 🚀 다음 단계 구현 가이드

### 1단계: 프로젝트별 롤-권한 관리

#### 1.1 데이터베이스 확장

```sql
-- 프로젝트별 롤 테이블 생성
CREATE TABLE project_roles (
    id SERIAL PRIMARY KEY,
    project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    role_id INTEGER NOT NULL REFERENCES security_roles(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(project_id, role_id)
);

-- 프로젝트별 권한 할당 테이블 생성
CREATE TABLE project_role_permissions (
    id SERIAL PRIMARY KEY,
    project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    role_id INTEGER NOT NULL REFERENCES security_roles(id) ON DELETE CASCADE,
    permission_id INTEGER NOT NULL REFERENCES security_permissions(id) ON DELETE CASCADE,
    assigned BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(project_id, role_id, permission_id)
);

-- 인덱스 생성
CREATE INDEX idx_project_roles_project_id ON project_roles(project_id);
CREATE INDEX idx_project_role_permissions_project_id ON project_role_permissions(project_id);
CREATE INDEX idx_project_role_permissions_role_id ON project_role_permissions(role_id);
```

#### 1.2 Domain 계층 확장

```rust
// src/domain/entities/project_role.rs
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ProjectRole {
    pub id: i32,
    pub project_id: i32,
    pub role_id: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewProjectRole {
    pub project_id: i32,
    pub role_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRolePermission {
    pub id: i32,
    pub project_id: i32,
    pub role_id: i32,
    pub permission_id: i32,
    pub assigned: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewProjectRolePermission {
    pub project_id: i32,
    pub role_id: i32,
    pub permission_id: i32,
    pub assigned: bool,
}
```

#### 1.3 Repository 인터페이스 추가

```rust
// src/domain/repositories/project_role_repository.rs
use crate::domain::entities::{ProjectRole, NewProjectRole, ProjectRolePermission, NewProjectRolePermission};
use sqlx::Error;

#[async_trait::async_trait]
pub trait ProjectRoleRepository: Send + Sync {
    // 프로젝트별 롤 관리
    async fn create_project_role(&self, new_role: &NewProjectRole) -> Result<ProjectRole, Error>;
    async fn delete_project_role(&self, project_id: i32, role_id: i32) -> Result<bool, Error>;
    async fn get_project_roles(&self, project_id: i32) -> Result<Vec<ProjectRole>, Error>;
    
    // 프로젝트별 권한 관리
    async fn assign_permission(&self, assignment: &NewProjectRolePermission) -> Result<ProjectRolePermission, Error>;
    async fn remove_permission(&self, project_id: i32, role_id: i32, permission_id: i32) -> Result<bool, Error>;
    async fn get_project_role_permissions(&self, project_id: i32) -> Result<Vec<ProjectRolePermission>, Error>;
    async fn get_project_role_permission_matrix(&self, project_id: i32) -> Result<ProjectRolePermissionMatrix, Error>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRolePermissionMatrix {
    pub project_id: i32,
    pub roles: Vec<RoleInfo>,
    pub permissions_by_category: std::collections::HashMap<String, Vec<PermissionInfo>>,
    pub assignments: Vec<AssignmentInfo>,
}
```

#### 1.4 Application 계층 확장

```rust
// src/application/use_cases/project_role_permission_use_case.rs
use crate::domain::repositories::ProjectRoleRepository;
use crate::domain::services::ProjectRoleService;
use crate::application::dto::project_role_permission_dto::*;

pub struct ProjectRolePermissionUseCase<R: ProjectRoleRepository> {
    project_role_repository: Arc<R>,
}

impl<R: ProjectRoleRepository> ProjectRolePermissionUseCase<R> {
    pub fn new(project_role_repository: Arc<R>) -> Self {
        Self { project_role_repository }
    }

    /// 프로젝트별 롤-권한 매트릭스 조회
    pub async fn get_project_role_permission_matrix(
        &self,
        project_id: i32,
    ) -> Result<ProjectRolePermissionMatrixResponse, ServiceError> {
        // 구현 로직
    }

    /// 프로젝트별 롤에 권한 할당/제거
    pub async fn assign_permission_to_project_role(
        &self,
        project_id: i32,
        role_id: i32,
        permission_id: i32,
        assign: bool,
    ) -> Result<AssignPermissionResponse, ServiceError> {
        // 구현 로직
    }
}
```

#### 1.5 DTO 정의

```rust
// src/application/dto/project_role_permission_dto.rs
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ProjectRolePermissionMatrixResponse {
    pub project_id: i32,
    pub project_name: String,
    pub roles: Vec<RoleInfo>,
    pub permissions_by_category: std::collections::HashMap<String, Vec<PermissionInfo>>,
    pub assignments: Vec<AssignmentInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AssignProjectRolePermissionRequest {
    pub assign: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AssignPermissionResponse {
    pub success: bool,
    pub message: String,
}
```

#### 1.6 Controller 구현

```rust
// src/presentation/controllers/project_role_permission_controller.rs
use actix_web::{web, HttpResponse, Result};
use crate::application::use_cases::ProjectRolePermissionUseCase;

pub struct ProjectRolePermissionController<R: ProjectRoleRepository> {
    use_case: Arc<ProjectRolePermissionUseCase<R>>,
}

impl<R: ProjectRoleRepository> ProjectRolePermissionController<R> {
    /// 프로젝트별 롤-권한 매트릭스 조회
    #[utoipa::path(
        get,
        path = "/api/projects/{project_id}/roles/permissions/matrix",
        responses(
            (status = 200, description = "프로젝트별 롤-권한 매트릭스 조회 성공", body = ProjectRolePermissionMatrixResponse),
            (status = 401, description = "인증 실패"),
            (status = 403, description = "권한 부족"),
            (status = 404, description = "프로젝트를 찾을 수 없음"),
            (status = 500, description = "서버 오류")
        ),
        params(
            ("project_id" = i32, Path, description = "프로젝트 ID")
        ),
        security(
            ("bearer_auth" = [])
        )
    )]
    pub async fn get_project_role_permission_matrix(
        path: web::Path<i32>,
        use_case: web::Data<ProjectRolePermissionUseCase<R>>,
    ) -> Result<HttpResponse> {
        let project_id = path.into_inner();
        
        match use_case.get_project_role_permission_matrix(project_id).await {
            Ok(matrix) => Ok(HttpResponse::Ok().json(matrix)),
            Err(e) => {
                log::error!("Failed to get project role permission matrix: {:?}", e);
                Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Failed to get project role permission matrix",
                    "message": e.to_string()
                })))
            }
        }
    }

    /// 프로젝트별 롤에 권한 할당/제거
    #[utoipa::path(
        put,
        path = "/api/projects/{project_id}/roles/{role_id}/permissions/{permission_id}",
        request_body = AssignProjectRolePermissionRequest,
        responses(
            (status = 200, description = "권한 할당/제거 성공", body = AssignPermissionResponse),
            (status = 400, description = "잘못된 요청"),
            (status = 401, description = "인증 실패"),
            (status = 403, description = "권한 부족"),
            (status = 404, description = "프로젝트, 롤 또는 권한을 찾을 수 없음"),
            (status = 500, description = "서버 오류")
        ),
        params(
            ("project_id" = i32, Path, description = "프로젝트 ID"),
            ("role_id" = i32, Path, description = "롤 ID"),
            ("permission_id" = i32, Path, description = "권한 ID")
        ),
        security(
            ("bearer_auth" = [])
        )
    )]
    pub async fn assign_permission_to_project_role(
        path: web::Path<(i32, i32, i32)>,
        request: web::Json<AssignProjectRolePermissionRequest>,
        use_case: web::Data<ProjectRolePermissionUseCase<R>>,
    ) -> Result<HttpResponse> {
        let (project_id, role_id, permission_id) = path.into_inner();
        let assign = request.into_inner().assign;
        
        match use_case.assign_permission_to_project_role(project_id, role_id, permission_id, assign).await {
            Ok(response) => Ok(HttpResponse::Ok().json(response)),
            Err(e) => {
                log::error!("Failed to assign permission to project role: {:?}", e);
                Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Failed to assign permission to project role",
                    "message": e.to_string()
                })))
            }
        }
    }
}
```

### 2단계: 사용자별 권한 관리

#### 2.1 사용자-롤 할당 테이블

```sql
-- 사용자-롤 할당 테이블
CREATE TABLE user_roles (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id INTEGER NOT NULL REFERENCES security_roles(id) ON DELETE CASCADE,
    project_id INTEGER REFERENCES projects(id) ON DELETE CASCADE, -- NULL이면 글로벌 롤
    assigned_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    assigned_by INTEGER REFERENCES users(id),
    UNIQUE(user_id, role_id, project_id)
);

-- 사용자별 권한 조회를 위한 뷰
CREATE VIEW user_permissions AS
SELECT 
    u.id as user_id,
    u.username,
    ur.role_id,
    r.name as role_name,
    r.scope as role_scope,
    ur.project_id,
    p.name as project_name,
    rp.permission_id,
    perm.resource_type,
    perm.action,
    perm.description as permission_description,
    rp.assigned
FROM users u
JOIN user_roles ur ON u.id = ur.user_id
JOIN security_roles r ON ur.role_id = r.id
LEFT JOIN projects p ON ur.project_id = p.id
LEFT JOIN security_role_permission rp ON r.id = rp.role_id AND r.scope = 'GLOBAL'
LEFT JOIN security_permissions perm ON rp.permission_id = perm.id
UNION ALL
SELECT 
    u.id as user_id,
    u.username,
    ur.role_id,
    r.name as role_name,
    r.scope as role_scope,
    ur.project_id,
    p.name as project_name,
    prp.permission_id,
    perm.resource_type,
    perm.action,
    perm.description as permission_description,
    prp.assigned
FROM users u
JOIN user_roles ur ON u.id = ur.user_id
JOIN security_roles r ON ur.role_id = r.id
JOIN projects p ON ur.project_id = p.id
JOIN project_role_permissions prp ON r.id = prp.role_id AND p.id = prp.project_id
JOIN security_permissions perm ON prp.permission_id = perm.id;
```

#### 2.2 사용자 권한 조회 API

```rust
// src/application/use_cases/user_permission_use_case.rs
use crate::domain::repositories::UserPermissionRepository;

pub struct UserPermissionUseCase<R: UserPermissionRepository> {
    user_permission_repository: Arc<R>,
}

impl<R: UserPermissionRepository> UserPermissionUseCase<R> {
    /// 사용자의 모든 권한 조회
    pub async fn get_user_permissions(
        &self,
        user_id: i32,
    ) -> Result<UserPermissionResponse, ServiceError> {
        // 구현 로직
    }

    /// 사용자의 특정 프로젝트 권한 조회
    pub async fn get_user_project_permissions(
        &self,
        user_id: i32,
        project_id: i32,
    ) -> Result<UserProjectPermissionResponse, ServiceError> {
        // 구현 로직
    }

    /// 사용자에게 롤 할당
    pub async fn assign_role_to_user(
        &self,
        user_id: i32,
        role_id: i32,
        project_id: Option<i32>,
        assigned_by: i32,
    ) -> Result<AssignRoleResponse, ServiceError> {
        // 구현 로직
    }
}
```

### 3단계: 권한 검증 시스템

#### 3.1 권한 검증 미들웨어

```rust
// src/infrastructure/middleware/permission_middleware.rs
use actix_web::{dev, Error, FromRequest, HttpRequest};
use futures_util::future::{ready, Ready};
use std::collections::HashSet;

pub struct PermissionMiddleware {
    required_permissions: HashSet<String>,
}

impl PermissionMiddleware {
    pub fn new(permissions: Vec<&str>) -> Self {
        Self {
            required_permissions: permissions.into_iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl<S> Transform<S, dev::ServiceRequest> for PermissionMiddleware
where
    S: dev::Service<dev::ServiceRequest, Response = dev::ServiceResponse, Error = Error> + 'static,
{
    type Response = dev::ServiceResponse;
    type Error = Error;
    type Transform = PermissionMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(PermissionMiddlewareService {
            service,
            required_permissions: self.required_permissions.clone(),
        }))
    }
}

pub struct PermissionMiddlewareService<S> {
    service: S,
    required_permissions: HashSet<String>,
}

impl<S> dev::Service<dev::ServiceRequest> for PermissionMiddlewareService<S>
where
    S: dev::Service<dev::ServiceRequest, Response = dev::ServiceResponse, Error = Error> + 'static,
{
    type Response = dev::ServiceResponse;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: dev::ServiceRequest) -> Self::Future {
        // 권한 검증 로직 구현
        let future = self.service.call(req);
        Box::pin(async move {
            // TODO: JWT 토큰에서 사용자 정보 추출
            // TODO: 사용자 권한 조회
            // TODO: 필요한 권한과 비교
            // TODO: 권한이 없으면 403 Forbidden 반환
            future.await
        })
    }
}
```

#### 3.2 권한 검증 매크로

```rust
// src/macros/permission_check.rs
#[macro_export]
macro_rules! require_permission {
    ($permission:expr) => {
        // 권한 검증 로직을 자동으로 생성하는 매크로
    };
}

// 사용 예시
#[require_permission("USER_CREATE")]
pub async fn create_user() -> impl Responder {
    // 사용자 생성 로직
}
```

### 4단계: 권한 관리 UI 지원

#### 4.1 권한 관리 API 확장

```rust
// src/application/use_cases/permission_management_use_case.rs
pub struct PermissionManagementUseCase {
    // 여러 리포지토리 의존성
}

impl PermissionManagementUseCase {
    /// 권한 카테고리별 그룹화
    pub async fn get_permissions_by_category(
        &self,
    ) -> Result<PermissionsByCategoryResponse, ServiceError> {
        // 구현 로직
    }

    /// 롤별 권한 요약
    pub async fn get_role_permission_summary(
        &self,
        role_id: i32,
    ) -> Result<RolePermissionSummaryResponse, ServiceError> {
        // 구현 로직
    }

    /// 사용자별 권한 요약
    pub async fn get_user_permission_summary(
        &self,
        user_id: i32,
    ) -> Result<UserPermissionSummaryResponse, ServiceError> {
        // 구현 로직
    }
}
```

#### 4.2 권한 검색 및 필터링

```rust
// src/application/use_cases/permission_search_use_case.rs
pub struct PermissionSearchUseCase {
    // 리포지토리 의존성
}

impl PermissionSearchUseCase {
    /// 권한 검색
    pub async fn search_permissions(
        &self,
        query: PermissionSearchQuery,
    ) -> Result<PermissionSearchResponse, ServiceError> {
        // 구현 로직
    }

    /// 롤 검색
    pub async fn search_roles(
        &self,
        query: RoleSearchQuery,
    ) -> Result<RoleSearchResponse, ServiceError> {
        // 구현 로직
    }
}
```

## 🧪 테스트 전략

### 1. 단위 테스트

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;

    mock! {
        ProjectRoleRepository {}
        
        #[async_trait]
        impl ProjectRoleRepository for ProjectRoleRepository {
            async fn create_project_role(&self, new_role: &NewProjectRole) -> Result<ProjectRole, Error>;
            async fn get_project_roles(&self, project_id: i32) -> Result<Vec<ProjectRole>, Error>;
            // ... 기타 메서드들
        }
    }

    #[tokio::test]
    async fn test_get_project_role_permission_matrix() {
        // 테스트 구현
    }
}
```

### 2. 통합 테스트

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use actix_web::{test, web, App};

    #[actix_web::test]
    async fn test_project_role_permission_api() {
        // 통합 테스트 구현
    }
}
```

## 📚 구현 순서

### Phase 1: 프로젝트별 롤-권한 관리
1. 데이터베이스 스키마 확장
2. Domain 엔티티 및 리포지토리 인터페이스 정의
3. Infrastructure 리포지토리 구현
4. Application Use Case 구현
5. Presentation Controller 구현
6. 테스트 작성

### Phase 2: 사용자별 권한 관리
1. 사용자-롤 할당 테이블 생성
2. 사용자 권한 조회 API 구현
3. 권한 검증 시스템 구현
4. 테스트 작성

### Phase 3: 고급 기능
1. 권한 검색 및 필터링
2. 권한 관리 UI 지원 API
3. 권한 감사 로그
4. 성능 최적화

## 🔧 설정 및 환경

### 환경 변수 추가

```bash
# .env 파일에 추가
# 권한 관리 설정
PERMISSION_CACHE_TTL=300
PERMISSION_CACHE_SIZE=1000
PERMISSION_AUDIT_ENABLED=true

# 프로젝트별 권한 설정
PROJECT_ROLE_ENABLED=true
PROJECT_ROLE_DEFAULT_ROLES=viewer,editor,admin
```

### 설정 파일 확장

```toml
# config/default.toml
[permission_management]
cache_ttl = 300
cache_size = 1000
audit_enabled = true
project_role_enabled = true
default_roles = ["viewer", "editor", "admin"]

[permission_validation]
strict_mode = true
allow_inheritance = true
log_violations = true
```

## 🎯 성능 고려사항

### 1. 캐싱 전략
- Redis를 사용한 권한 정보 캐싱
- 사용자별 권한 캐시
- 롤별 권한 캐시

### 2. 데이터베이스 최적화
- 적절한 인덱스 생성
- 쿼리 최적화
- 연결 풀 설정

### 3. API 응답 최적화
- 페이지네이션
- 필드 선택적 로딩
- 압축 응답

## 📖 문서화

### 1. API 문서
- OpenAPI 스펙 업데이트
- 사용 예시 추가
- 에러 코드 문서화

### 2. 개발자 가이드
- 권한 시스템 아키텍처 설명
- 구현 가이드
- 트러블슈팅 가이드

---

이 가이드를 따라 단계적으로 기능을 확장하면, 강력하고 확장 가능한 권한 관리 시스템을 구축할 수 있습니다. 각 단계마다 충분한 테스트를 작성하고, Clean Architecture 원칙을 유지하는 것이 중요합니다.
