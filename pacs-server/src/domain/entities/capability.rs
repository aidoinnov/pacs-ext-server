use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Capability {
    pub id: i32,
    pub name: String,
    pub display_name: String,
    pub display_label: String,
    pub description: Option<String>,
    pub category: String,
    pub category_label: String,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewCapability {
    pub name: String,
    pub display_name: String,
    pub display_label: String,
    pub description: Option<String>,
    pub category: String,
    pub category_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCapability {
    pub display_name: Option<String>,
    pub display_label: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub category_label: Option<String>,
    pub is_active: Option<bool>,
}
