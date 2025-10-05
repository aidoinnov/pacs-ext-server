use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;
use uuid::Uuid;

/// 역할 할당 요청 DTO
#[derive(Debug, Deserialize, Serialize)]
pub struct AssignRoleRequest {
    pub user_id: Uuid,
    pub role_id: Uuid,
    pub project_id: Option<Uuid>,
}

/// 권한 부여 요청 DTO
#[derive(Debug, Deserialize, Serialize)]
pub struct GrantPermissionRequest {
    pub user_id: Uuid,
    pub permission_id: Uuid,
    pub project_id: Option<Uuid>,
    pub resource_id: Option<String>,
}

/// DICOM 접근 로그 요청 DTO
#[derive(Debug, Deserialize, Serialize)]
pub struct LogDicomAccessRequest {
    pub user_id: i32,
    pub project_id: Option<i32>,
    pub resource_type: String,
    pub study_uid: Option<String>,
    pub series_uid: Option<String>,
    pub instance_uid: Option<String>,
    pub action: String,
    pub result: String,
    pub ip_address: Option<String>,
    pub ae_title: Option<String>,
}

/// 접근 로그 응답 DTO
#[derive(Debug, Deserialize, Serialize)]
pub struct AccessLogResponse {
    pub id: i64,
    pub user_id: i32,
    pub project_id: Option<i32>,
    pub resource_type: String,
    pub study_uid: Option<String>,
    pub series_uid: Option<String>,
    pub instance_uid: Option<String>,
    pub action: String,
    pub result: String,
    pub logged_at: NaiveDateTime,
}

/// 접근 로그 목록 응답 DTO
#[derive(Debug, Deserialize, Serialize)]
pub struct AccessLogListResponse {
    pub logs: Vec<AccessLogResponse>,
    pub total: usize,
}

/// 권한 검증 요청 DTO
#[derive(Debug, Deserialize, Serialize)]
pub struct CheckPermissionRequest {
    pub user_id: i32,
    pub project_id: i32,
    pub resource_type: String,
    pub action: String,
}

/// 권한 검증 응답 DTO
#[derive(Debug, Deserialize, Serialize)]
pub struct CheckPermissionResponse {
    pub user_id: i32,
    pub project_id: i32,
    pub resource_type: String,
    pub action: String,
    pub has_permission: bool,
}

/// 사용자 권한 목록 응답 DTO
#[derive(Debug, Deserialize, Serialize)]
pub struct UserPermissionsResponse {
    pub user_id: i32,
    pub project_id: i32,
    pub permissions: Vec<PermissionInfo>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PermissionInfo {
    pub id: i32,
    pub resource_type: String,
    pub action: String,
}

/// 프로젝트 접근 검증 응답 DTO
#[derive(Debug, Deserialize, Serialize)]
pub struct ProjectAccessResponse {
    pub user_id: i32,
    pub project_id: i32,
    pub can_access: bool,
    pub is_member: bool,
}
