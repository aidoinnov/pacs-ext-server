use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// 권한 생성 요청 DTO
#[derive(Debug, Deserialize, Serialize)]
pub struct CreatePermissionRequest {
    pub permission_code: String,
    pub permission_name: String,
    pub description: Option<String>,
}

/// 권한 업데이트 요청 DTO
#[derive(Debug, Deserialize, Serialize)]
pub struct UpdatePermissionRequest {
    pub permission_name: Option<String>,
    pub description: Option<String>,
}

/// 역할 생성 요청 DTO
#[derive(Debug, Deserialize, Serialize)]
pub struct CreateRoleRequest {
    pub name: String,
    pub scope: String, // "GLOBAL" or "PROJECT"
    pub description: Option<String>,
}

/// 역할 응답 DTO
#[derive(Debug, Deserialize, Serialize)]
pub struct RoleResponse {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub scope: String,
}

/// 권한 응답 DTO
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct PermissionResponse {
    pub id: i32,
    pub resource_type: String,
    pub action: String,
}

/// 권한 할당 요청 DTO
#[derive(Debug, Deserialize, Serialize)]
pub struct AssignPermissionRequest {
    pub permission_id: i32,
}

/// 역할 권한 목록 응답 DTO
#[derive(Debug, Deserialize, Serialize)]
pub struct RolePermissionsResponse {
    pub role_id: i32,
    pub role_name: String,
    pub permissions: Vec<PermissionResponse>,
}

/// 프로젝트 권한 목록 응답 DTO
#[derive(Debug, Deserialize, Serialize)]
pub struct ProjectPermissionsResponse {
    pub project_id: i32,
    pub permissions: Vec<PermissionResponse>,
}

/// 리소스별 권한 목록 응답 DTO
#[derive(Debug, Deserialize, Serialize)]
pub struct ResourcePermissionsResponse {
    pub resource_type: String,
    pub permissions: Vec<PermissionResponse>,
}

/// 역할과 권한 정보를 포함하는 응답 DTO
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct RoleWithPermissionsResponse {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub scope: String,
    pub permissions: Vec<PermissionResponse>,
}

/// 페이지네이션이 포함된 역할 목록 응답 DTO
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct RolesWithPermissionsListResponse {
    pub roles: Vec<RoleWithPermissionsResponse>,
    pub total_count: i64,
    pub page: i32,
    pub page_size: i32,
    pub total_pages: i32,
}

/// 페이지네이션 쿼리 파라미터 DTO
#[derive(Debug, Deserialize, ToSchema)]
pub struct PaginationQuery {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
}
