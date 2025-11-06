use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
// use chrono::NaiveDateTime;
use chrono::{DateTime, Utc};

/// Annotation 생성 요청 DTO
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CreateAnnotationRequest {
    /// User ID
    #[schema(example = 336)]
    pub user_id: Option<i32>,

    /// Project ID
    #[schema(example = 299)]
    pub project_id: Option<i32>,

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
    /// 어노테이션의 실제 데이터를 담는 JSON 객체
    /// 지원하는 타입: circle, rectangle, point, polygon 등
    #[schema(example = json!({"type": "circle", "x": 100, "y": 200, "radius": 50, "color": "#FF0000", "label": "Test Annotation"}))]
    pub annotation_data: serde_json::Value,

    /// 측정 도구 이름
    /// 어노테이션을 생성하는데 사용된 도구의 이름
    #[schema(example = "Circle Tool")]
    pub tool_name: Option<String>,

    /// 측정 도구 버전
    /// 사용된 도구의 버전 정보
    #[schema(example = "2.1.0")]
    pub tool_version: Option<String>,

    /// 뷰어 소프트웨어 정보
    /// 어노테이션을 생성한 뷰어 프로그램의 이름
    #[schema(example = "OHIF Viewer")]
    pub viewer_software: Option<String>,

    /// 어노테이션 설명
    /// 어노테이션에 대한 추가 설명이나 메모
    #[schema(example = "의심되는 병변 영역")]
    pub description: Option<String>,

    /// 측정값
    /// id, type, values, unit을 포함하는 측정 객체 배열
    #[schema(example = json!([
        {"id": "m1", "type": "raw", "values": [42.3, 18.7], "unit": "mm"},
        {"id": "m2", "type": "mean", "values": [30.5], "unit": "mm"}
    ]))]
    pub measurement_values: Option<serde_json::Value>,
}

/// Annotation 업데이트 요청 DTO
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct UpdateAnnotationRequest {
    /// Annotation 데이터 (JSON 형식)
    /// 어노테이션의 실제 데이터를 담는 JSON 객체
    #[schema(example = json!({"type": "rectangle", "x": 50, "y": 50, "width": 200, "height": 100, "color": "#0000FF"}))]
    pub annotation_data: Option<serde_json::Value>,

    /// 측정 도구 이름
    /// 어노테이션을 생성하는데 사용된 도구의 이름
    #[schema(example = "Rectangle Tool")]
    pub tool_name: Option<String>,

    /// 측정 도구 버전
    /// 사용된 도구의 버전 정보
    #[schema(example = "2.1.0")]
    pub tool_version: Option<String>,

    /// 뷰어 소프트웨어 정보
    /// 어노테이션을 생성한 뷰어 프로그램의 이름
    #[schema(example = "OHIF Viewer")]
    pub viewer_software: Option<String>,

    /// 어노테이션 설명
    /// 어노테이션에 대한 추가 설명이나 메모
    #[schema(example = "수정된 병변 영역")]
    pub description: Option<String>,

    /// 측정값
    /// id, type, values, unit을 포함하는 측정 객체 배열
    #[schema(example = json!([
        {"id": "m1", "type": "raw", "values": [42.3, 18.7], "unit": "mm"}
    ]))]
    pub measurement_values: Option<serde_json::Value>,
}

/// Annotation 응답 DTO
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct AnnotationResponse {
    /// Annotation ID
    /// 데이터베이스에서 생성된 고유 식별자
    pub id: i32,

    /// 사용자 ID
    /// 어노테이션을 생성한 사용자의 식별자
    pub user_id: i32,

    /// Study Instance UID
    /// DICOM Study의 고유 식별자
    pub study_instance_uid: String,

    /// Series Instance UID
    /// DICOM Series의 고유 식별자
    pub series_instance_uid: String,

    /// SOP Instance UID
    /// DICOM SOP Instance의 고유 식별자
    pub sop_instance_uid: String,

    /// Annotation 데이터
    /// 어노테이션의 실제 데이터를 담는 JSON 객체
    pub annotation_data: serde_json::Value,

    /// 측정 도구 이름
    /// 어노테이션을 생성하는데 사용된 도구의 이름
    #[schema(example = "Circle Tool")]
    pub tool_name: Option<String>,

    /// 측정 도구 버전
    /// 사용된 도구의 버전 정보
    #[schema(example = "2.1.0")]
    pub tool_version: Option<String>,

    /// 뷰어 소프트웨어 정보
    /// 어노테이션을 생성한 뷰어 프로그램의 이름
    #[schema(example = "OHIF Viewer")]
    pub viewer_software: Option<String>,

    /// 어노테이션 설명
    /// 어노테이션에 대한 추가 설명이나 메모
    #[schema(example = "의심되는 병변 영역")]
    pub description: Option<String>,

    /// 측정값
    pub measurement_values: Option<serde_json::Value>,

    /// 생성 시간
    /// 어노테이션이 생성된 시각
    #[schema(value_type = String, example = "2024-01-01T00:00:00")]
    // pub created_at: NaiveDateTime,
    pub created_at: DateTime<Utc>,

    /// 수정 시간
    /// 어노테이션이 마지막으로 수정된 시각
    #[schema(value_type = String, example = "2024-01-01T00:00:00")]
    // pub updated_at: NaiveDateTime,
    pub updated_at: DateTime<Utc>,
}

/// Annotation 목록 응답 DTO
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct AnnotationListResponse {
    /// Annotation 목록
    pub annotations: Vec<AnnotationResponse>,

    /// 전체 개수
    pub total: usize,
}
