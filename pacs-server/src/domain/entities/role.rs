use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text")]
pub enum RoleScope {
    #[sqlx(rename = "GLOBAL")]
    Global,
    #[sqlx(rename = "PROJECT")]
    Project,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Role {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub scope: String, // Will be converted to/from RoleScope
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewRole {
    pub name: String,
    pub description: Option<String>,
    pub scope: RoleScope,
}
