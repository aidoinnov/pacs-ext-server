//! # 어노테이션 엔티티 모듈
//! 
//! 이 모듈은 의료 영상에 대한 어노테이션 정보를 나타내는 엔티티들을 정의합니다.
//! 어노테이션은 의료진이 의료 영상에 추가한 표시, 측정, 분석 결과 등을 의미합니다.

// 날짜/시간 처리를 위한 chrono 라이브러리
use chrono::NaiveDateTime;
// JSON 직렬화/역직렬화를 위한 serde 라이브러리
use serde::{Deserialize, Serialize};
// SQLx를 통한 데이터베이스 행 매핑을 위한 트레이트
use sqlx::FromRow;

/// 의료 영상 어노테이션을 나타내는 엔티티
/// 
/// 이 구조체는 데이터베이스의 `annotation_annotation` 테이블과 매핑되며,
/// 의료진이 의료 영상에 추가한 어노테이션 정보를 저장합니다.
/// 
/// # 필드
/// - `id`: 데이터베이스에서 자동 생성되는 고유 식별자
/// - `project_id`: 어노테이션이 속한 프로젝트의 ID
/// - `user_id`: 어노테이션을 생성한 사용자의 ID
/// - `study_uid`: DICOM Study Instance UID (필수)
/// - `series_uid`: DICOM Series Instance UID (선택사항)
/// - `instance_uid`: DICOM SOP Instance UID (선택사항)
/// - `tool_name`: 어노테이션 생성에 사용된 도구명
/// - `tool_version`: 도구의 버전 정보 (선택사항)
/// - `data`: 어노테이션의 실제 데이터 (JSON 형태)
/// - `is_shared`: 다른 사용자와 공유 여부
/// - `created_at`: 어노테이션이 생성된 시각
/// - `updated_at`: 어노테이션이 마지막으로 수정된 시각
/// - `viewer_software`: 어노테이션 생성에 사용된 뷰어 소프트웨어 (선택사항)
/// - `description`: 어노테이션에 대한 설명 (선택사항)
/// 
/// # 예시
/// ```rust
/// let annotation = Annotation {
///     id: 1,
///     project_id: 1,
///     user_id: 1,
///     study_uid: "1.2.3.4.5.6.7.8.9.1".to_string(),
///     series_uid: Some("1.2.3.4.5.6.7.8.9.2".to_string()),
///     instance_uid: Some("1.2.3.4.5.6.7.8.9.3".to_string()),
///     tool_name: "Measurement Tool".to_string(),
///     tool_version: Some("v2.1.0".to_string()),
///     data: serde_json::json!({"type": "line", "points": [[0, 0], [100, 100]]}),
///     is_shared: true,
///     created_at: NaiveDateTime::from_timestamp_opt(1640995200, 0).unwrap(),
///     updated_at: NaiveDateTime::from_timestamp_opt(1640995200, 0).unwrap(),
///     viewer_software: Some("DICOM Viewer Pro".to_string()),
///     description: Some("폐 결절 크기 측정".to_string()),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Annotation {
    /// 데이터베이스에서 자동 생성되는 고유 식별자
    pub id: i32,
    /// 어노테이션이 속한 프로젝트의 ID
    pub project_id: i32,
    /// 어노테이션을 생성한 사용자의 ID
    pub user_id: i32,
    /// DICOM Study Instance UID (필수)
    pub study_uid: String,
    /// DICOM Series Instance UID (선택사항)
    pub series_uid: Option<String>,
    /// DICOM SOP Instance UID (선택사항)
    pub instance_uid: Option<String>,
    /// 어노테이션 생성에 사용된 도구명
    pub tool_name: String,
    /// 도구의 버전 정보 (선택사항)
    pub tool_version: Option<String>,
    /// 어노테이션의 실제 데이터 (JSON 형태)
    pub data: serde_json::Value,
    /// 다른 사용자와 공유 여부
    pub is_shared: bool,
    /// 어노테이션이 생성된 시각
    pub created_at: NaiveDateTime,
    /// 어노테이션이 마지막으로 수정된 시각
    pub updated_at: NaiveDateTime,
    /// 어노테이션 생성에 사용된 뷰어 소프트웨어 (선택사항)
    pub viewer_software: Option<String>,
    /// 어노테이션에 대한 설명 (선택사항)
    pub description: Option<String>,
}

/// 어노테이션 변경 이력을 나타내는 엔티티
/// 
/// 이 구조체는 데이터베이스의 `annotation_annotation_history` 테이블과 매핑되며,
/// 어노테이션의 모든 변경 사항을 추적합니다.
/// 
/// # 필드
/// - `id`: 데이터베이스에서 자동 생성되는 고유 식별자
/// - `annotation_id`: 변경된 어노테이션의 ID
/// - `user_id`: 변경을 수행한 사용자의 ID
/// - `action`: 수행된 작업 (create, update, delete 등)
/// - `data_before`: 변경 전 데이터 (JSON 형태)
/// - `data_after`: 변경 후 데이터 (JSON 형태)
/// - `action_at`: 변경이 수행된 시각
/// 
/// # 예시
/// ```rust
/// let history = AnnotationHistory {
///     id: 1,
///     annotation_id: 1,
///     user_id: 1,
///     action: "create".to_string(),
///     data_before: None,
///     data_after: Some(serde_json::json!({"type": "line", "points": [[0, 0], [100, 100]]})),
///     action_at: NaiveDateTime::from_timestamp_opt(1640995200, 0).unwrap(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AnnotationHistory {
    /// 데이터베이스에서 자동 생성되는 고유 식별자
    pub id: i32,
    /// 변경된 어노테이션의 ID
    pub annotation_id: i32,
    /// 변경을 수행한 사용자의 ID
    pub user_id: i32,
    /// 수행된 작업 (create, update, delete 등)
    pub action: String,
    /// 변경 전 데이터 (JSON 형태)
    pub data_before: Option<serde_json::Value>,
    /// 변경 후 데이터 (JSON 형태)
    pub data_after: Option<serde_json::Value>,
    /// 변경이 수행된 시각
    pub action_at: NaiveDateTime,
}

/// 새로운 어노테이션 생성을 위한 DTO(Data Transfer Object)
/// 
/// 이 구조체는 어노테이션 생성 요청 시 전달되는 데이터를 나타냅니다.
/// 데이터베이스에 저장되기 전의 어노테이션 정보를 담고 있습니다.
/// 
/// # 필드
/// - `project_id`: 어노테이션이 속할 프로젝트의 ID
/// - `user_id`: 어노테이션을 생성할 사용자의 ID
/// - `study_uid`: DICOM Study Instance UID (필수)
/// - `series_uid`: DICOM Series Instance UID (선택사항)
/// - `instance_uid`: DICOM SOP Instance UID (선택사항)
/// - `tool_name`: 어노테이션 생성에 사용된 도구명
/// - `tool_version`: 도구의 버전 정보 (선택사항)
/// - `viewer_software`: 어노테이션 생성에 사용된 뷰어 소프트웨어 (선택사항)
/// - `description`: 어노테이션에 대한 설명 (선택사항)
/// - `data`: 어노테이션의 실제 데이터 (JSON 형태)
/// - `is_shared`: 다른 사용자와 공유 여부
/// 
/// # 예시
/// ```rust
/// let new_annotation = NewAnnotation {
///     project_id: 1,
///     user_id: 1,
///     study_uid: "1.2.3.4.5.6.7.8.9.1".to_string(),
///     series_uid: Some("1.2.3.4.5.6.7.8.9.2".to_string()),
///     instance_uid: Some("1.2.3.4.5.6.7.8.9.3".to_string()),
///     tool_name: "Measurement Tool".to_string(),
///     tool_version: Some("v2.1.0".to_string()),
///     viewer_software: Some("DICOM Viewer Pro".to_string()),
///     description: Some("폐 결절 크기 측정".to_string()),
///     data: serde_json::json!({"type": "line", "points": [[0, 0], [100, 100]]}),
///     is_shared: true,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewAnnotation {
    /// 어노테이션이 속할 프로젝트의 ID
    pub project_id: i32,
    /// 어노테이션을 생성할 사용자의 ID
    pub user_id: i32,
    /// DICOM Study Instance UID (필수)
    pub study_uid: String,
    /// DICOM Series Instance UID (선택사항)
    pub series_uid: Option<String>,
    /// DICOM SOP Instance UID (선택사항)
    pub instance_uid: Option<String>,
    /// 어노테이션 생성에 사용된 도구명
    pub tool_name: String,
    /// 도구의 버전 정보 (선택사항)
    pub tool_version: Option<String>,
    /// 어노테이션 생성에 사용된 뷰어 소프트웨어 (선택사항)
    pub viewer_software: Option<String>,
    /// 어노테이션에 대한 설명 (선택사항)
    pub description: Option<String>,
    /// 어노테이션의 실제 데이터 (JSON 형태)
    pub data: serde_json::Value,
    /// 다른 사용자와 공유 여부
    pub is_shared: bool,
}
