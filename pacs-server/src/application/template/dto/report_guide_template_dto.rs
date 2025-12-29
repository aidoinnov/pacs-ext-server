use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use chrono::{DateTime, Utc};

// ========== 원본 템플릿 DTO ==========

/// 원본 템플릿 생성 요청 DTO
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CreateReportGuideTemplateRequest {
    #[schema(example = "Chest CT Normal")]
    pub name: String,
    #[schema(example = "정상 흉부 CT 소견")]
    pub description: Option<String>,
    #[schema(example = "추가 검사 불필요")]
    pub conclusion: Option<String>,
    #[schema(example = "chest")]
    pub bodypart: Option<String>,
    #[schema(example = true)]
    pub is_shared: Option<bool>,
    /// 하나 이상의 모달리티
    #[schema(example = "CT")]
    pub modalities: Vec<String>,
}

/// 원본 템플릿 수정 요청 DTO
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct UpdateReportGuideTemplateRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub conclusion: Option<String>,
    pub bodypart: Option<String>,
    pub is_shared: Option<bool>,
    pub is_active: Option<bool>,
}

/// 원본 템플릿 응답 DTO
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct ReportGuideTemplateResponse {
    #[schema(example = 1)]
    pub id: i32,
    #[schema(example = "Chest CT Normal")]
    pub name: String,
    pub description: Option<String>,
    pub conclusion: Option<String>,
    pub bodypart: Option<String>,
    #[schema(example = true)]
    pub is_shared: bool,
    #[schema(example = true)]
    pub is_active: bool,
    #[schema(example = 456)]
    pub created_by: i32,
    pub modalities: Vec<String>,
    pub images: Vec<TemplateImageResponse>,
    #[schema(value_type = String, example = "2025-01-15T10:00:00Z")]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, example = "2025-01-15T10:00:00Z")]
    pub updated_at: DateTime<Utc>,
}

/// 템플릿 이미지 응답 DTO
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TemplateImageResponse {
    #[schema(example = 1)]
    pub id: i32,
    #[schema(example = "templates/1/images/img1.png")]
    pub image_path: String,
    #[schema(example = "https://s3.example.com/templates/1/images/img1.png")]
    pub image_url: String,
    #[schema(example = 102400)]
    pub file_size: Option<i64>,
    #[schema(example = "image/png")]
    pub mime_type: Option<String>,
    #[schema(example = 0)]
    pub display_order: i32,
    #[schema(example = true)]
    pub is_shared: bool,
    #[schema(example = 456)]
    pub uploaded_by: i32,
    #[schema(value_type = String, example = "2025-01-15T10:00:00Z")]
    pub created_at: DateTime<Utc>,
}

/// 템플릿 이미지 추가 요청 DTO
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct AddTemplateImageRequest {
    #[schema(example = "templates/1/images/img1.png")]
    pub image_path: String,
    #[schema(example = "https://s3.example.com/templates/1/images/img1.png")]
    pub image_url: String,
    #[schema(example = 102400)]
    pub file_size: Option<i64>,
    #[schema(example = "image/png")]
    pub mime_type: Option<String>,
    #[schema(example = 0)]
    pub display_order: Option<i32>,
    #[schema(example = true)]
    pub is_shared: Option<bool>,
}

/// 템플릿 이미지 공유 설정 변경 요청 DTO
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct UpdateImageShareStatusRequest {
    #[schema(example = true)]
    pub is_shared: bool,
}

/// 템플릿 이미지 업로드 URL 요청 DTO
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct TemplateImageUploadUrlRequest {
    #[schema(example = "guide_image.png")]
    pub file_name: String,
    #[schema(example = "image/png")]
    pub mime_type: Option<String>,
    #[schema(example = 1024000)]
    pub file_size: Option<i64>,
}

/// 템플릿 이미지 업로드 URL 응답 DTO
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TemplateImageUploadUrlResponse {
    #[schema(example = true)]
    pub success: bool,
    #[schema(example = "https://s3.example.com/upload-url")]
    pub upload_url: String,
    #[schema(example = "templates/1/images/guide_image.png")]
    pub file_path: String,
    #[schema(example = 600)]
    pub expires_in: u64,
}

/// 템플릿 이미지 업로드 완료 요청 DTO
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct TemplateImageUploadCompleteRequest {
    #[schema(example = "templates/1/images/guide_image.png")]
    pub file_path: String,
    #[schema(example = 1024000)]
    pub file_size: i64,
    #[schema(example = "image/png")]
    pub mime_type: Option<String>,
    #[schema(example = 0)]
    pub display_order: Option<i32>,
    #[schema(example = true)]
    pub is_shared: Option<bool>,
}

/// 템플릿 이미지 업로드 완료 응답 DTO
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TemplateImageUploadCompleteResponse {
    #[schema(example = true)]
    pub success: bool,
    #[schema(example = "Image uploaded and added to template successfully")]
    pub message: String,
    pub image: TemplateImageResponse,
}

// ========== 사용자 커스텀 템플릿 DTO ==========

/// 커스텀 템플릿 생성 요청 DTO (원본 복사)
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CreateCustomTemplateFromBaseRequest {
    /// 원본 템플릿 ID
    #[schema(example = 1)]
    pub base_template_id: i32,
    #[schema(example = "My Custom Chest CT Template")]
    pub name: String,
}

/// 커스텀 템플릿 생성 요청 DTO (원본 없이)
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CreateCustomTemplateRequest {
    #[schema(example = "My Custom Template")]
    pub name: String,
    pub description: Option<String>,
    pub conclusion: Option<String>,
    pub bodypart: Option<String>,
    /// 하나 이상의 모달리티
    #[schema(example = "CT")]
    pub modalities: Vec<String>,
}

/// 커스텀 템플릿 수정 요청 DTO
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct UpdateCustomTemplateRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub conclusion: Option<String>,
    pub bodypart: Option<String>,
    pub is_active: Option<bool>,
}

/// 커스텀 템플릿 응답 DTO
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct UserCustomReportTemplateResponse {
    #[schema(example = 1)]
    pub id: i32,
    #[schema(example = 456)]
    pub user_id: i32,
    #[schema(example = 1)]
    pub base_template_id: Option<i32>,
    #[schema(example = "My Custom Template")]
    pub name: String,
    pub description: Option<String>,
    pub conclusion: Option<String>,
    pub bodypart: Option<String>,
    #[schema(example = true)]
    pub is_active: bool,
    pub modalities: Vec<String>,
    pub images: Vec<CustomTemplateImageResponse>,
    #[schema(value_type = String, example = "2025-01-15T10:00:00Z")]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, example = "2025-01-15T10:00:00Z")]
    pub updated_at: DateTime<Utc>,
}

/// 커스텀 템플릿 이미지 응답 DTO
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CustomTemplateImageResponse {
    #[schema(example = 1)]
    pub id: i32,
    #[schema(example = "custom-templates/1/images/img1.png")]
    pub image_path: String,
    #[schema(example = "https://s3.example.com/custom-templates/1/images/img1.png")]
    pub image_url: String,
    #[schema(example = 102400)]
    pub file_size: Option<i64>,
    #[schema(example = "image/png")]
    pub mime_type: Option<String>,
    #[schema(example = 0)]
    pub display_order: i32,
    #[schema(example = false)]
    pub is_shared: bool,
    #[schema(example = 456)]
    pub uploaded_by: i32,
    #[schema(value_type = String, example = "2025-01-15T10:00:00Z")]
    pub created_at: DateTime<Utc>,
}

/// 커스텀 템플릿 이미지 추가 요청 DTO
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct AddCustomTemplateImageRequest {
    #[schema(example = "custom-templates/1/images/img1.png")]
    pub image_path: String,
    #[schema(example = "https://s3.example.com/custom-templates/1/images/img1.png")]
    pub image_url: String,
    #[schema(example = 102400)]
    pub file_size: Option<i64>,
    #[schema(example = "image/png")]
    pub mime_type: Option<String>,
    #[schema(example = 0)]
    pub display_order: Option<i32>,
}

// ========== Report-템플릿 적용 DTO ==========

/// 템플릿 적용 요청 DTO
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ApplyTemplateToReportRequest {
    /// 원본 템플릿 ID (template_id 또는 custom_template_id 중 하나만)
    #[schema(example = 1)]
    pub template_id: Option<i32>,
    /// 커스텀 템플릿 ID (template_id 또는 custom_template_id 중 하나만)
    #[schema(example = 2)]
    pub custom_template_id: Option<i32>,
}

/// 템플릿 적용 응답 DTO
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ApplyTemplateToReportResponse {
    #[schema(example = true)]
    pub success: bool,
    #[schema(example = "Template applied successfully")]
    pub message: String,
}

// ========== 템플릿 목록 응답 DTO ==========

/// 템플릿 목록 응답 DTO
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ReportGuideTemplateListResponse {
    #[schema(example = true)]
    pub success: bool,
    pub templates: Vec<ReportGuideTemplateResponse>,
}

/// 커스텀 템플릿 목록 응답 DTO
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserCustomTemplateListResponse {
    #[schema(example = true)]
    pub success: bool,
    pub templates: Vec<UserCustomReportTemplateResponse>,
}

// ========== Report Guide Image 관리 DTO ==========

/// Report Guide Image 추가 요청 DTO
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct AddReportGuideRequest {
    /// 원본 템플릿 ID (template_id 또는 custom_template_id 중 하나만)
    #[schema(example = 1)]
    pub template_id: Option<i32>,
    /// 커스텀 템플릿 ID (template_id 또는 custom_template_id 중 하나만)
    #[schema(example = 2)]
    pub custom_template_id: Option<i32>,
    /// 표시 순서
    #[schema(example = 0)]
    pub display_order: Option<i32>,
}

/// Report Guide Image 응답 DTO
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct ReportGuideResponse {
    #[schema(example = 1)]
    pub id: i32,
    #[schema(example = 123)]
    pub report_id: i32,
    #[schema(example = 1)]
    pub template_id: Option<i32>,
    #[schema(example = 2)]
    pub custom_template_id: Option<i32>,
    #[schema(example = 0)]
    pub display_order: i32,
    #[schema(value_type = String, example = "2025-01-15T10:00:00Z")]
    pub created_at: DateTime<Utc>,
}

/// Report Guide Image 목록 응답 DTO
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ReportGuideListResponse {
    #[schema(example = true)]
    pub success: bool,
    pub guides: Vec<ReportGuideResponse>,
}

