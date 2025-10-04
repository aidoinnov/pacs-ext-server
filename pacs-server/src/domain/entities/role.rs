use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "text")]
pub enum RoleScope {
    #[sqlx(rename = "GLOBAL")]
    Global,
    #[sqlx(rename = "PROJECT")]
    Project,
}

impl RoleScope {
    pub fn as_str(&self) -> &str {
        match self {
            RoleScope::Global => "GLOBAL",
            RoleScope::Project => "PROJECT",
        }
    }
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
