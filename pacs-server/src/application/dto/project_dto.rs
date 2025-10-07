use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use chrono::NaiveDateTime;

/// 프로젝트 생성 요청 DTO
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: Option<String>,
}

/// 프로젝트 업데이트 요청 DTO
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

/// 프로젝트 응답 DTO
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ProjectResponse {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
    #[schema(value_type = String, example = "2024-01-01T00:00:00")]
    pub created_at: NaiveDateTime,
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

/// 프로젝트 목록 응답 DTO
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ProjectListResponse {
    pub projects: Vec<ProjectResponse>,
    pub total: usize,
}

/// 프로젝트 역할 할당 요청 DTO
#[derive(Debug, Deserialize, Serialize)]
pub struct ProjectAssignRoleRequest {
    pub role_id: i32,
}

/// 프로젝트 멤버 목록 응답 DTO
#[derive(Debug, Deserialize, Serialize)]
pub struct ProjectMembersResponse {
    pub project_id: i32,
    pub members: Vec<MemberInfo>,
    pub total: i64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MemberInfo {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub joined_at: NaiveDateTime,
}

/// 프로젝트 역할 목록 응답 DTO
#[derive(Debug, Deserialize, Serialize)]
pub struct ProjectRolesResponse {
    pub project_id: i32,
    pub roles: Vec<RoleInfo>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RoleInfo {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub scope: String,
}
