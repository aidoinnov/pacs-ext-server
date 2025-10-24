//! # 프로젝트 엔티티 모듈
//! 
//! 이 모듈은 시스템의 프로젝트 정보를 나타내는 엔티티들을 정의합니다.
//! 프로젝트는 사용자들이 협업할 수 있는 작업 단위를 의미하며,
//! 어노테이션과 마스크는 특정 프로젝트에 속하게 됩니다.

// 날짜/시간 처리를 위한 chrono 라이브러리
use chrono::{DateTime, Utc};
// JSON 직렬화/역직렬화를 위한 serde 라이브러리
use serde::{Deserialize, Serialize};
// SQLx를 통한 데이터베이스 행 매핑을 위한 트레이트
use sqlx::{FromRow, Type};

/// 프로젝트 상태를 나타내는 열거형
/// 
/// 프로젝트의 생명주기 상태를 나타내며, 데이터베이스의 `project_status` ENUM과 매핑됩니다.
/// 
/// # Variants
/// - `Preparing`: 준비중 - 프로젝트가 생성되었지만 아직 시작되지 않음
/// - `InProgress`: 진행중 - 프로젝트가 활발히 진행 중
/// - `Completed`: 완료 - 프로젝트가 성공적으로 완료됨
/// - `OnHold`: 보류 - 프로젝트가 일시적으로 중단됨
/// - `Cancelled`: 취소 - 프로젝트가 취소됨
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
#[sqlx(type_name = "project_status", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProjectStatus {
    Preparing,
    InProgress,
    Completed,
    OnHold,
    Cancelled,
}

/// 시스템 프로젝트를 나타내는 엔티티
/// 
/// 이 구조체는 데이터베이스의 `security_project` 테이블과 매핑되며,
/// 사용자들이 협업할 수 있는 작업 단위를 나타냅니다.
/// 
/// # 필드
/// - `id`: 데이터베이스에서 자동 생성되는 고유 식별자
/// - `name`: 프로젝트의 고유한 이름
/// - `description`: 프로젝트에 대한 상세 설명 (선택사항)
/// - `is_active`: 프로젝트 활성화 상태 (true: 활성, false: 비활성)
/// - `status`: 프로젝트의 생명주기 상태 (PREPARING, IN_PROGRESS, COMPLETED, ON_HOLD, CANCELLED)
/// - `created_at`: 프로젝트가 생성된 시각
/// 
/// # 예시
/// ```rust
/// let project = Project {
///     id: 1,
///     id: 1,
///     name: "의료영상 분석 프로젝트".to_string(),
///     description: Some("폐암 진단을 위한 CT 영상 분석".to_string()),
///     is_active: true,
///     created_at: NaiveDateTime::from_timestamp_opt(1640995200, 0).unwrap(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Project {
    /// 데이터베이스에서 자동 생성되는 고유 식별자
    pub id: i32,
    /// 프로젝트의 고유한 이름
    pub name: String,
    /// 프로젝트에 대한 상세 설명 (선택사항)
    pub description: Option<String>,
    /// 프로젝트 활성화 상태 (true: 활성, false: 비활성)
    pub is_active: bool,
    /// 프로젝트의 생명주기 상태
    pub status: ProjectStatus,
    /// 프로젝트가 생성된 시각
    pub created_at: DateTime<Utc>,
}

/// 새로운 프로젝트 생성을 위한 DTO(Data Transfer Object)
/// 
/// 이 구조체는 프로젝트 생성 요청 시 전달되는 데이터를 나타냅니다.
/// 데이터베이스에 저장되기 전의 프로젝트 정보를 담고 있습니다.
/// 
/// # 필드
/// - `name`: 생성할 프로젝트명 (중복되지 않아야 함)
/// - `description`: 프로젝트에 대한 상세 설명 (선택사항)
/// 
/// # 예시
/// ```rust
/// let new_project = NewProject {
///     name: "새로운 의료영상 프로젝트".to_string(),
///     description: Some("AI 기반 의료영상 진단 프로젝트".to_string()),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewProject {
    /// 생성할 프로젝트명 (중복되지 않아야 함)
    pub name: String,
    /// 프로젝트에 대한 상세 설명 (선택사항)
    pub description: Option<String>,
}
