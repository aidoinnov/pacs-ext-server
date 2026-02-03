//! # SW Information 엔티티
//!
//! 의료영상저장장치 소프트웨어 정보 (SW Information 화면 데이터)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// SW Information (의료영상저장장치 소프트웨어 정보)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SwInformation {
    pub id: i32,
    pub product_item: String,
    pub model_name: String,
    pub sw_version: Option<String>,
    pub manufacturer: String,
    pub address: String,
    pub manufacturing_permit_number: String,
    pub manufacturing_year_month: Option<String>,
    pub serial_number: Option<String>,
    pub udi: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
