use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// User-Project relation
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserProject {
    pub id: i32,
    pub user_id: i32,
    pub project_id: i32,
    pub created_at: DateTime<Utc>,
}

// Project-Role relation
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProjectRole {
    pub id: i32,
    pub project_id: i32,
    pub role_id: i32,
    pub created_at: DateTime<Utc>,
}

// Role-Permission relation
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RolePermission {
    pub id: i32,
    pub role_id: i32,
    pub permission_id: i32,
    pub scope: Option<String>,
    pub created_at: DateTime<Utc>,
}

// Project-Permission relation
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProjectPermission {
    pub id: i32,
    pub project_id: i32,
    pub permission_id: i32,
    pub scope: Option<String>,
    pub inherits_from_role_permission: bool,
    pub created_at: DateTime<Utc>,
}

// Role-AccessCondition relation
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RoleAccessCondition {
    pub id: i32,
    pub role_id: i32,
    pub access_condition_id: i32,
    pub created_at: DateTime<Utc>,
}

// Project-AccessCondition relation
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProjectAccessCondition {
    pub id: i32,
    pub project_id: i32,
    pub access_condition_id: i32,
    pub created_at: DateTime<Utc>,
}

// User-Group relation
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserGroup {
    pub id: i32,
    pub user_id: i32,
    pub group_id: i32,
    pub created_at: DateTime<Utc>,
}

// Group-Role relation
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct GroupRole {
    pub id: i32,
    pub group_id: i32,
    pub role_id: i32,
    pub created_at: DateTime<Utc>,
}
