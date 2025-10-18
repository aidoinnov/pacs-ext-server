use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct HangingProtocol {
    pub id: i32,
    pub project_id: i32,
    pub owner_user_id: i32,
    pub name: String,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct HpCondition {
    pub id: i32,
    pub protocol_id: i32,
    pub dicom_tag: String,
    pub operator: String,
    pub value: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct HpLayout {
    pub id: i32,
    pub protocol_id: i32,
    pub rows: i32,
    pub cols: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct HpViewport {
    pub id: i32,
    pub layout_id: i32,
    pub position_row: i32,
    pub position_col: i32,
    pub selection_rule: Option<String>,
    pub sort_order: Option<String>,
    pub created_at: DateTime<Utc>,
}
