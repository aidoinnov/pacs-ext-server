use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Permission {
    pub id: i32,
    pub category: String,
    pub resource_type: String,
    pub action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewPermission {
    pub category: String,
    pub resource_type: String,
    pub action: String,
}
