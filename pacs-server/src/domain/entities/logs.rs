use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "grant_action_enum", rename_all = "UPPERCASE")]
pub enum GrantAction {
    Grant,
    Revoke,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct GrantLog {
    pub id: i64,
    pub granted_by: i32,
    pub granted_to: i32,
    pub role_id: Option<i32>,
    pub project_id: Option<i32>,
    pub action: GrantAction,
    pub via_group_id: Option<i32>,
    pub logged_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AccessLog {
    pub id: i64,
    pub user_id: i32,
    pub project_id: Option<i32>,
    pub resource_type: String,
    pub study_uid: Option<String>,
    pub series_uid: Option<String>,
    pub instance_uid: Option<String>,
    pub action: String,
    pub result: String,
    pub dicom_tag_check: Option<String>,
    pub ae_title: Option<String>,
    pub ip_address: Option<String>,
    pub session_id: Option<String>,
    pub via_group_id: Option<i32>,
    pub logged_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewAccessLog {
    pub user_id: i32,
    pub project_id: Option<i32>,
    pub resource_type: String,
    pub study_uid: Option<String>,
    pub series_uid: Option<String>,
    pub instance_uid: Option<String>,
    pub action: String,
    pub result: String,
    pub dicom_tag_check: Option<String>,
    pub ae_title: Option<String>,
    pub ip_address: Option<String>,
    pub session_id: Option<String>,
    pub via_group_id: Option<i32>,
}
