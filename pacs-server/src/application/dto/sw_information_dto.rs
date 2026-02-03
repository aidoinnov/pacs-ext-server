//! # SW Information DTO
//!
//! SW Information API 요청/응답 DTO

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// SW Information 단일 항목 응답
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SwInformationResponse {
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
    #[schema(value_type = String, example = "2026-02-02T00:00:00Z")]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, example = "2026-02-02T00:00:00Z")]
    pub updated_at: DateTime<Utc>,
}

impl From<crate::domain::sw_information::SwInformation> for SwInformationResponse {
    fn from(e: crate::domain::sw_information::SwInformation) -> Self {
        Self {
            id: e.id,
            product_item: e.product_item,
            model_name: e.model_name,
            sw_version: e.sw_version,
            manufacturer: e.manufacturer,
            address: e.address,
            manufacturing_permit_number: e.manufacturing_permit_number,
            manufacturing_year_month: e.manufacturing_year_month,
            serial_number: e.serial_number,
            udi: e.udi,
            created_at: e.created_at,
            updated_at: e.updated_at,
        }
    }
}

/// SW Information 목록 응답
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SwInformationListResponse {
    pub success: bool,
    pub items: Vec<SwInformationResponse>,
    pub total_count: i64,
}
