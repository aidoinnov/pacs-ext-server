use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Annotation {
    pub id: i32,
    pub project_id: i32,
    pub user_id: i32,
    pub study_uid: String,
    pub series_uid: Option<String>,
    pub instance_uid: Option<String>,
    pub tool_name: String,
    pub tool_version: Option<String>,
    pub data: serde_json::Value,
    pub is_shared: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub viewer_software: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AnnotationHistory {
    pub id: i32,
    pub annotation_id: i32,
    pub user_id: i32,
    pub action: String,
    pub data_before: Option<serde_json::Value>,
    pub data_after: Option<serde_json::Value>,
    pub action_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewAnnotation {
    pub project_id: i32,
    pub user_id: i32,
    pub study_uid: String,
    pub series_uid: Option<String>,
    pub instance_uid: Option<String>,
    pub tool_name: String,
    pub tool_version: Option<String>,
    pub viewer_software: Option<String>,
    pub description: Option<String>,
    pub data: serde_json::Value,
    pub is_shared: bool,
}
