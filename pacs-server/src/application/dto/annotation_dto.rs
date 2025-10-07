use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use chrono::NaiveDateTime;

/// Annotation 생성 요청 DTO
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CreateAnnotationRequest {
    /// Study Instance UID
    #[schema(example = "1.2.840.113619.2.55.3.604688119.868.1234567890.1")]
    pub study_instance_uid: String,

    /// Series Instance UID
    #[schema(example = "1.2.840.113619.2.55.3.604688119.868.1234567890.2")]
    pub series_instance_uid: String,

    /// SOP Instance UID
    #[schema(example = "1.2.840.113619.2.55.3.604688119.868.1234567890.3")]
    pub sop_instance_uid: String,

    /// Annotation 데이터 (JSON 형식)
    #[schema(example = json!({"type": "circle", "x": 100, "y": 200, "radius": 50}))]
    pub annotation_data: serde_json::Value,

    /// 설명
    #[schema(example = "의심되는 병변")]
    pub description: Option<String>,
}

/// Annotation 업데이트 요청 DTO
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct UpdateAnnotationRequest {
    /// Annotation 데이터 (JSON 형식)
    pub annotation_data: Option<serde_json::Value>,

    /// 설명
    pub description: Option<String>,
}

/// Annotation 응답 DTO
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct AnnotationResponse {
    /// Annotation ID
    pub id: i32,

    /// 사용자 ID
    pub user_id: i32,

    /// Study Instance UID
    pub study_instance_uid: String,

    /// Series Instance UID
    pub series_instance_uid: String,

    /// SOP Instance UID
    pub sop_instance_uid: String,

    /// Annotation 데이터
    pub annotation_data: serde_json::Value,

    /// 설명
    pub description: Option<String>,

    /// 생성 시간
    #[schema(value_type = String, example = "2024-01-01T00:00:00")]
    pub created_at: NaiveDateTime,

    /// 수정 시간
    #[schema(value_type = String, example = "2024-01-01T00:00:00")]
    pub updated_at: NaiveDateTime,
}

/// Annotation 목록 응답 DTO
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct AnnotationListResponse {
    /// Annotation 목록
    pub annotations: Vec<AnnotationResponse>,

    /// 전체 개수
    pub total: usize,
}
