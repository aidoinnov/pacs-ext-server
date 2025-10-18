use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "condition_type_enum", rename_all = "UPPERCASE")]
pub enum ConditionType {
    Allow,
    Deny,
    Limit,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "resource_level_enum", rename_all = "UPPERCASE")]
pub enum ResourceLevel {
    Study,
    Series,
    Instance,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AccessCondition {
    pub id: i32,
    pub resource_type: String,
    pub resource_level: ResourceLevel,
    pub dicom_tag: Option<String>,
    pub operator: String,
    pub value: Option<String>,
    pub condition_type: ConditionType,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewAccessCondition {
    pub resource_type: String,
    pub resource_level: ResourceLevel,
    pub dicom_tag: Option<String>,
    pub operator: String,
    pub value: Option<String>,
    pub condition_type: ConditionType,
}
