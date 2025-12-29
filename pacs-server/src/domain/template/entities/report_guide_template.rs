//! # Report Guide Template 엔티티 모듈
//!
//! 이 모듈은 리포트 가이드 템플릿을 나타내는 엔티티들을 정의합니다.
//! 원본 템플릿과 사용자 커스텀 템플릿을 지원합니다.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 원본 리포트 가이드 템플릿
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ReportGuideTemplate {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub conclusion: Option<String>,
    pub bodypart: Option<String>,
    pub is_shared: bool,
    pub is_active: bool,
    pub created_by: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 새로운 원본 템플릿 생성용 구조체
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewReportGuideTemplate {
    pub name: String,
    pub description: Option<String>,
    pub conclusion: Option<String>,
    pub bodypart: Option<String>,
    pub is_shared: bool,
    pub created_by: i32,
}

/// 원본 템플릿 업데이트용 구조체
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateReportGuideTemplate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub conclusion: Option<String>,
    pub bodypart: Option<String>,
    pub is_shared: Option<bool>,
    pub is_active: Option<bool>,
}

/// 템플릿-모달리티 매핑
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ReportGuideTemplateModality {
    pub id: i32,
    pub template_id: i32,
    pub modality: String,
}

/// 템플릿 이미지
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ReportGuideTemplateImage {
    pub id: i32,
    pub template_id: i32,
    pub image_path: String,
    pub image_url: String,
    pub file_size: Option<i64>,
    pub mime_type: Option<String>,
    pub display_order: i32,
    pub is_shared: bool,
    pub uploaded_by: i32,
    pub created_at: DateTime<Utc>,
}

/// 새로운 템플릿 이미지 생성용 구조체
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewReportGuideTemplateImage {
    pub template_id: i32,
    pub image_path: String,
    pub image_url: String,
    pub file_size: Option<i64>,
    pub mime_type: Option<String>,
    pub display_order: i32,
    pub is_shared: bool,
    pub uploaded_by: i32,
}

/// 사용자 커스텀 리포트 템플릿
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserCustomReportTemplate {
    pub id: i32,
    pub user_id: i32,
    pub base_template_id: Option<i32>,
    pub name: String,
    pub description: Option<String>,
    pub conclusion: Option<String>,
    pub bodypart: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 새로운 커스텀 템플릿 생성용 구조체
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewUserCustomReportTemplate {
    pub user_id: i32,
    pub base_template_id: Option<i32>,
    pub name: String,
    pub description: Option<String>,
    pub conclusion: Option<String>,
    pub bodypart: Option<String>,
}

/// 커스텀 템플릿 업데이트용 구조체
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserCustomReportTemplate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub conclusion: Option<String>,
    pub bodypart: Option<String>,
    pub is_active: Option<bool>,
}

/// 커스텀 템플릿-모달리티 매핑
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserCustomTemplateModality {
    pub id: i32,
    pub custom_template_id: i32,
    pub modality: String,
}

/// 커스텀 템플릿 이미지
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserCustomTemplateImage {
    pub id: i32,
    pub custom_template_id: i32,
    pub image_path: String,
    pub image_url: String,
    pub file_size: Option<i64>,
    pub mime_type: Option<String>,
    pub display_order: i32,
    pub is_shared: bool,
    pub uploaded_by: i32,
    pub created_at: DateTime<Utc>,
}

/// 새로운 커스텀 템플릿 이미지 생성용 구조체
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewUserCustomTemplateImage {
    pub custom_template_id: i32,
    pub image_path: String,
    pub image_url: String,
    pub file_size: Option<i64>,
    pub mime_type: Option<String>,
    pub display_order: i32,
    pub is_shared: bool,
    pub uploaded_by: i32,
}

/// Report-가이드 매핑
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SeriesUserReportGuide {
    pub id: i32,
    pub report_id: i32,
    pub template_id: Option<i32>,
    pub custom_template_id: Option<i32>,
    pub display_order: i32,
    pub created_at: DateTime<Utc>,
}

/// 새로운 Report-가이드 매핑 생성용 구조체
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewSeriesUserReportGuide {
    pub report_id: i32,
    pub template_id: Option<i32>,
    pub custom_template_id: Option<i32>,
    pub display_order: i32,
}

