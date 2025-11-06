//! # 프로젝트 엔티티 모듈
//!
//! 이 모듈은 시스템의 프로젝트 정보를 나타내는 엔티티들을 정의합니다.
//! 프로젝트는 사용자들이 협업할 수 있는 작업 단위를 의미하며,
//! 어노테이션과 마스크는 특정 프로젝트에 속하게 됩니다.

// 날짜/시간 처리를 위한 chrono 라이브러리
use chrono::{DateTime, NaiveDate, Utc};
// JSON 직렬화/역직렬화를 위한 serde 라이브러리
use serde::{Deserialize, Serialize};
// SQLx를 통한 데이터베이스 행 매핑을 위한 트레이트
use sqlx::{FromRow, Type};

/// 프로젝트 상태를 나타내는 열거형
///
/// 프로젝트의 생명주기 상태를 나타내며, 데이터베이스의 `project_status` ENUM과 매핑됩니다.
///
/// # Variants
/// - `Planning`: 기획중 - 프로젝트가 기획 단계
/// - `Active`: 진행중 - 프로젝트가 활발히 진행 중
/// - `Completed`: 완료 - 프로젝트가 성공적으로 완료됨
/// - `Suspended`: 보류 - 프로젝트가 일시적으로 중단됨
/// - `Cancelled`: 취소 - 프로젝트가 취소됨
/// - `PendingCompletion`: 완료 대기 - 프로젝트 종료 대기 중
/// - `OverPlanning`: 계획 초과 - 프로젝트 계획 초과 상태
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
#[sqlx(type_name = "project_status", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProjectStatus {
    #[sqlx(rename = "PLANNING")]
    Planning,
    #[sqlx(rename = "ACTIVE")]
    Active,
    Completed,
    #[sqlx(rename = "SUSPENDED")]
    Suspended,
    Cancelled,
    #[sqlx(rename = "PENDING_COMPLETION")]
    PendingCompletion,
    #[sqlx(rename = "OVER_PLANNING")]
    OverPlanning,
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
/// - `sponsor`: 프로젝트 스폰서명
/// - `start_date`: 프로젝트 시작일
/// - `end_date`: 프로젝트 종료일/목표일 (선택사항)
/// - `auto_complete`: 자동 완료 여부 (true: end_date 도달 시 자동 완료)
/// - `is_active`: 프로젝트 활성화 상태 (true: 활성, false: 비활성/아카이브)
/// - `status`: 프로젝트의 생명주기 상태
/// - `created_at`: 프로젝트가 생성된 시각
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Project {
    /// 데이터베이스에서 자동 생성되는 고유 식별자
    pub id: i32,
    /// 프로젝트의 고유한 이름
    pub name: String,
    /// 프로젝트에 대한 상세 설명 (선택사항)
    pub description: Option<String>,
    /// 프로젝트 스폰서명
    pub sponsor: String,
    /// 프로젝트 시작일
    pub start_date: NaiveDate,
    /// 프로젝트 종료일/목표일 (선택사항)
    pub end_date: Option<NaiveDate>,
    /// 자동 완료 여부
    pub auto_complete: bool,
    /// 프로젝트 활성화 상태 (true: 활성, false: 비활성/아카이브)
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
/// - `sponsor`: 프로젝트 스폰서명
/// - `start_date`: 프로젝트 시작일
/// - `end_date`: 프로젝트 종료일/목표일 (선택사항)
/// - `auto_complete`: 자동 완료 여부
///
/// # 예시
/// ```ignore
/// let new_project = NewProject {
///     name: "새로운 의료영상 프로젝트".to_string(),
///     description: Some("AI 기반 의료영상 진단 프로젝트".to_string()),
///     sponsor: "서울대학교병원".to_string(),
///     start_date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
///     end_date: Some(NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()),
///     auto_complete: false,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewProject {
    /// 생성할 프로젝트명 (중복되지 않아야 함)
    pub name: String,
    /// 프로젝트에 대한 상세 설명 (선택사항)
    pub description: Option<String>,
    /// 프로젝트 스폰서명
    pub sponsor: String,
    /// 프로젝트 시작일
    pub start_date: NaiveDate,
    /// 프로젝트 종료일/목표일 (선택사항)
    pub end_date: Option<NaiveDate>,
    /// 자동 완료 여부 (기본값: false)
    pub auto_complete: bool,
}

/// 프로젝트 업데이트를 위한 DTO(Data Transfer Object)
///
/// 이 구조체는 프로젝트 업데이트 요청 시 전달되는 데이터를 나타냅니다.
/// 모든 필드는 선택사항이며, 제공된 필드만 업데이트됩니다.
///
/// # 필드
/// - `name`: 프로젝트명
/// - `description`: 프로젝트에 대한 상세 설명
/// - `sponsor`: 프로젝트 스폰서명
/// - `start_date`: 프로젝트 시작일
/// - `end_date`: 프로젝트 종료일/목표일
/// - `status`: 프로젝트 상태
/// - `auto_complete`: 자동 완료 여부
/// - `is_active`: 프로젝트 활성화 상태
///
/// # 예시
/// ```ignore
/// let update_project = UpdateProject {
///     name: None,
///     description: Some("업데이트된 설명".to_string()),
///     sponsor: None,
///     start_date: None,
///     end_date: Some(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()),
///     status: Some(ProjectStatus::Active),
///     auto_complete: Some(true),
///     is_active: None,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProject {
    /// 업데이트할 프로젝트명
    pub name: Option<String>,
    /// 업데이트할 프로젝트 설명
    pub description: Option<String>,
    /// 업데이트할 프로젝트 스폰서명
    pub sponsor: Option<String>,
    /// 업데이트할 프로젝트 시작일
    pub start_date: Option<NaiveDate>,
    /// 업데이트할 프로젝트 종료일
    pub end_date: Option<NaiveDate>,
    /// 업데이트할 프로젝트 상태
    pub status: Option<ProjectStatus>,
    /// 업데이트할 자동 완료 여부
    pub auto_complete: Option<bool>,
    /// 업데이트할 프로젝트 활성화 상태
    pub is_active: Option<bool>,
}
