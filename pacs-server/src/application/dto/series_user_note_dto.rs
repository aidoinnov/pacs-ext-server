use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use chrono::{DateTime, Utc};

/// Series User Note 생성/수정 요청 DTO
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CreateOrUpdateSeriesNoteRequest {
    /// 메모 텍스트 (note 또는 content 필드 사용 가능)
    #[serde(alias = "content")]
    #[schema(example = "이 시리즈는 프로젝트 A에서 분석 중입니다")]
    pub note: String,

    /// 태그 (선택사항, 현재는 무시됨)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[schema(example = json!(["tag1", "tag2"]))]
    pub tags: Vec<String>,
}

/// Series User Note 응답 DTO
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SeriesNoteResponse {
    /// Note ID
    #[schema(example = 1)]
    pub id: i32,
    /// Series ID
    #[schema(example = 123)]
    pub series_id: i32,
    /// 사용자 ID
    #[schema(example = 456)]
    pub user_id: i32,
    /// 프로젝트 ID (None이면 전역 note, Some(id)이면 프로젝트별 note)
    #[schema(example = 1)]
    pub project_id: Option<i32>,
    /// 메모 텍스트
    #[schema(example = "이 시리즈는 프로젝트 A에서 분석 중입니다")]
    pub note: String,
    /// 생성 시각
    #[schema(example = "2025-01-15T10:00:00Z", value_type = String)]
    pub created_at: DateTime<Utc>,
    /// 수정 시각
    #[schema(example = "2025-01-15T10:00:00Z", value_type = String)]
    pub updated_at: DateTime<Utc>,
}

/// Series User Note 목록 응답 DTO (사용자 정보 포함)
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SeriesNoteWithUserResponse {
    /// Note ID
    #[schema(example = 1)]
    pub id: i32,
    /// Series ID
    #[schema(example = 123)]
    pub series_id: i32,
    /// 사용자 정보
    pub user: SeriesNoteUserInfo,
    /// 프로젝트 ID (None이면 전역 note, Some(id)이면 프로젝트별 note)
    #[schema(example = 1)]
    pub project_id: Option<i32>,
    /// 메모 텍스트
    #[schema(example = "이 시리즈는 프로젝트 A에서 분석 중입니다")]
    pub note: String,
    /// 생성 시각
    #[schema(example = "2025-01-15T10:00:00Z", value_type = String)]
    pub created_at: DateTime<Utc>,
    /// 수정 시각
    #[schema(example = "2025-01-15T10:00:00Z", value_type = String)]
    pub updated_at: DateTime<Utc>,
}

/// 사용자 정보 DTO (Series Note용)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SeriesNoteUserInfo {
    /// 사용자 ID
    #[schema(example = 456)]
    pub id: i32,
    /// 사용자명
    #[schema(example = "user1")]
    pub username: String,
    /// 이메일
    #[schema(example = "user1@example.com")]
    pub email: String,
    /// 전체 이름
    #[schema(example = "홍길동")]
    pub full_name: Option<String>,
}

/// Series User Note 목록 응답 DTO
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SeriesNoteListResponse {
    /// 성공 여부
    #[schema(example = true)]
    pub success: bool,
    /// Note 목록
    pub notes: Vec<SeriesNoteWithUserResponse>,
}

/// Series User Note 단일 응답 DTO
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SeriesNoteSingleResponse {
    /// 성공 여부
    #[schema(example = true)]
    pub success: bool,
    /// Note 내용 (없으면 빈 문자열)
    #[schema(example = "이 시리즈는 프로젝트 A에서 분석 중입니다")]
    #[serde(default)]
    pub note: String,
}

