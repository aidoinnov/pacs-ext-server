use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use chrono::{DateTime, Utc};

/// Series User Report 생성/수정 요청 DTO
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CreateOrUpdateSeriesReportRequest {
    /// 리포트 상태 (unread, approval, unapproval)
    #[schema(example = "unread")]
    pub status: Option<String>,
    /// 설명
    #[schema(example = "이 시리즈는 정상 소견입니다")]
    pub description: String,
    /// 결론
    #[schema(example = "추가 검사 불필요")]
    pub conclusion: String,
    /// 신체 부위
    #[schema(example = "chest")]
    pub bodypart: Option<String>,
}

/// Series User Report 응답 DTO
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct SeriesReportResponse {
    #[schema(example = 1)]
    pub id: i32,
    #[schema(example = 123)]
    pub series_id: i32,
    #[schema(example = 456)]
    pub user_id: i32,
    #[schema(example = 1)]
    pub project_id: Option<i32>,
    #[schema(example = "unread")]
    pub status: String,
    pub dictate_file_path: Option<String>,
    pub dictate_file_size: Option<i64>,
    pub dictate_mime_type: Option<String>,
    #[schema(example = "이 시리즈는 정상 소견입니다")]
    pub description: String,
    #[schema(example = "추가 검사 불필요")]
    pub conclusion: String,
    #[schema(example = "chest")]
    pub bodypart: Option<String>,
    /// Report에 연결된 Guide Image 목록
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guides: Option<Vec<crate::application::template::dto::report_guide_template_dto::ReportGuideResponse>>,
    #[schema(value_type = String, example = "2025-01-15T10:00:00Z")]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, example = "2025-01-15T10:00:00Z")]
    pub updated_at: DateTime<Utc>,
}

/// Series User Report 목록 응답 DTO (사용자 정보 포함)
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct SeriesReportWithUserResponse {
    #[schema(example = 1)]
    pub id: i32,
    #[schema(example = 123)]
    pub series_id: i32,
    pub user: SeriesReportUserInfo,
    #[schema(example = 1)]
    pub project_id: Option<i32>,
    #[schema(example = "unread")]
    pub status: String,
    pub dictate_file_path: Option<String>,
    pub dictate_file_size: Option<i64>,
    pub dictate_mime_type: Option<String>,
    #[schema(example = "이 시리즈는 정상 소견입니다")]
    pub description: String,
    #[schema(example = "추가 검사 불필요")]
    pub conclusion: String,
    #[schema(example = "chest")]
    pub bodypart: Option<String>,
    #[schema(value_type = String, example = "2025-01-15T10:00:00Z")]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, example = "2025-01-15T10:00:00Z")]
    pub updated_at: DateTime<Utc>,
}

/// 사용자 정보 DTO (Series Report용)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SeriesReportUserInfo {
    #[schema(example = 456)]
    pub id: i32,
    #[schema(example = "user1")]
    pub username: String,
    #[schema(example = "user1@example.com")]
    pub email: String,
    #[schema(example = "홍길동")]
    pub full_name: Option<String>,
}

/// Report 목록 응답 DTO
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SeriesReportListResponse {
    #[schema(example = true)]
    pub success: bool,
    pub reports: Vec<SeriesReportWithUserResponse>,
}

/// Report 단일 응답 DTO
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SeriesReportSingleResponse {
    #[schema(example = true)]
    pub success: bool,
    /// Report 설명 (없으면 빈 문자열)
    #[schema(example = "이 시리즈는 정상 소견입니다")]
    #[serde(default)]
    pub description: String,
    /// Report 결론 (없으면 빈 문자열)
    #[schema(example = "추가 검사 불필요")]
    #[serde(default)]
    pub conclusion: String,
}

/// 오디오 파일 업로드 URL 요청 DTO
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct DictateUploadUrlRequest {
    #[schema(example = "audio/mpeg")]
    pub mime_type: Option<String>,
    #[schema(example = 1024000)]
    pub file_size: Option<i64>,
}

/// 오디오 파일 업로드 URL 응답 DTO
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DictateUploadUrlResponse {
    #[schema(example = true)]
    pub success: bool,
    #[schema(example = "https://s3.example.com/upload-url")]
    pub upload_url: String,
    #[schema(example = "reports/123/dictate/audio.mp3")]
    pub file_path: String,
    #[schema(example = 600)]
    pub expires_in: u64,
}

/// 오디오 파일 업로드 완료 요청 DTO
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct DictateUploadCompleteRequest {
    #[schema(example = "reports/123/dictate/audio.mp3")]
    pub file_path: String,
    #[schema(example = 1024000)]
    pub file_size: i64,
    #[schema(example = "audio/mpeg")]
    pub mime_type: String,
}

/// 오디오 파일 업로드 완료 응답 DTO
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DictateUploadCompleteResponse {
    #[schema(example = true)]
    pub success: bool,
    #[schema(example = "Audio file uploaded successfully")]
    pub message: String,
}

