use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// User with role information (for project members list)
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserWithRoleResponse {
    pub user_id: i32,
    pub username: String,
    pub email: String,
    pub full_name: Option<String>,
    pub role_id: Option<i32>,
    pub role_name: Option<String>,
    pub role_scope: Option<String>,
}

/// Project with role information (for user's projects list)
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ProjectWithRoleResponse {
    pub project_id: i32,
    pub project_name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub start_date: Option<String>, // 프로젝트 시작일
    pub end_date: Option<String>,   // 프로젝트 종료일
    pub role_id: Option<i32>,
    pub role_name: Option<String>,
    pub role_scope: Option<String>,
}

/// Paginated project members response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ProjectMembersResponse {
    pub members: Vec<UserWithRoleResponse>,
    pub total_count: i64,
    pub page: i32,
    pub page_size: i32,
    pub total_pages: i32,
}

/// Paginated user projects response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserProjectsResponse {
    pub projects: Vec<ProjectWithRoleResponse>,
    pub total_count: i64,
    pub page: i32,
    pub page_size: i32,
    pub total_pages: i32,
}

/// Request to assign role to a user
#[derive(Debug, Deserialize, ToSchema)]
pub struct AssignRoleRequest {
    pub role_id: i32,
}

/// Request for batch role assignment
#[derive(Debug, Deserialize, ToSchema)]
pub struct BatchAssignRolesRequest {
    pub assignments: Vec<UserRoleAssignment>,
}

/// Individual user-role assignment for batch operations
#[derive(Debug, Deserialize, ToSchema)]
pub struct UserRoleAssignment {
    pub user_id: i32,
    pub role_id: i32,
}

/// Response for successful role assignment
#[derive(Debug, Serialize, ToSchema)]
pub struct RoleAssignmentResponse {
    pub message: String,
    pub user_id: i32,
    pub project_id: i32,
    pub role_id: i32,
}

/// Response for successful batch role assignment
#[derive(Debug, Serialize, ToSchema)]
pub struct BatchRoleAssignmentResponse {
    pub message: String,
    pub project_id: i32,
    pub assigned_count: i32,
    pub failed_assignments: Vec<FailedAssignment>,
}

/// Information about failed role assignments
#[derive(Debug, Serialize, ToSchema)]
pub struct FailedAssignment {
    pub user_id: i32,
    pub role_id: i32,
    pub error: String,
}

/// Request to add a member to a project
#[derive(Debug, Deserialize, ToSchema)]
pub struct AddMemberRequest {
    pub user_id: i32,
    pub role_id: Option<i32>, // Optional, defaults to Viewer role if not provided
}

/// Response for membership check
#[derive(Debug, Serialize, ToSchema)]
pub struct MembershipResponse {
    pub is_member: bool,
    pub role_id: Option<i32>,
    pub role_name: Option<String>,
    pub joined_at: Option<String>,
}

/// Response for successful member addition
#[derive(Debug, Serialize, ToSchema)]
pub struct AddMemberResponse {
    pub message: String,
    pub user_id: i32,
    pub project_id: i32,
    pub role_id: i32,
    pub role_name: String,
}

/// Response for successful member removal
#[derive(Debug, Serialize, ToSchema)]
pub struct RemoveMemberResponse {
    pub message: String,
    pub user_id: i32,
    pub project_id: i32,
}
