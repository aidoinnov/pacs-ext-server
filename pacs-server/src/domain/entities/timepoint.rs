//! # TimePoint 엔티티 모듈
//!
//! 이 모듈은 임상시험에서 Subject의 평가 시점(TimePoint)을 나타내는 엔티티들을 정의합니다.
//! TimePoint는 Baseline, TP1, TP2 등의 평가 시점을 의미하며,
//! RECIST Report 작성의 기준이 됩니다.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

use super::StudyInfo;

/// Visit 타입 열거형
///
/// 임상시험 프로토콜 상의 방문 의미를 나타냅니다.
///
/// # Variants
/// - `Baseline`: 기준선 평가 (첫 평가 시점)
/// - `Visit`: 일반 방문
/// - `EOT`: End of Treatment (치료 종료)
/// - `USV`: Unscheduled Visit (계획되지 않은 방문)
///
/// # Database Mapping
/// PostgreSQL ENUM 타입 `timepoint_visit_type_enum`과 매핑됩니다.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, PartialEq, Eq, ToSchema)]
#[sqlx(type_name = "timepoint_visit_type_enum")]
pub enum VisitType {
    Baseline,
    Visit,
    #[sqlx(rename = "EOT")]
    EOT,
    #[sqlx(rename = "USV")]
    USV,
}

impl VisitType {
    /// VisitType을 문자열로 변환
    pub fn as_str(&self) -> &str {
        match self {
            VisitType::Baseline => "Baseline",
            VisitType::Visit => "Visit",
            VisitType::EOT => "EOT",
            VisitType::USV => "USV",
        }
    }
}

/// TimePoint 엔티티
///
/// Subject별 평가 시점을 나타내는 엔티티입니다.
/// 각 Subject는 정확히 하나의 Baseline TimePoint를 가지며,
/// 추가로 여러 개의 Visit TimePoint를 가질 수 있습니다.
///
/// # 필드
/// - `id`: 데이터베이스에서 자동 생성되는 고유 식별자
/// - `project_id`: 소속 프로젝트 ID
/// - `subject_id`: 소속 Subject ID
/// - `name`: TimePoint 이름 (BL, TP1, TP2 등)
/// - `visit_type`: Visit 타입 (Baseline, Visit, EOT, USV)
/// - `visit_no`: CTIMS Visit Number (연동 시 사용, nullable)
/// - `order_index`: TimePoint 정렬 순서 (0부터 시작)
/// - `external_key`: CTIMS TimePoint Key (연동 시 사용, nullable)
/// - `created_at`: TimePoint 생성 시각
/// - `updated_at`: TimePoint 수정 시각
///
/// # 제약 조건
/// - Subject당 Baseline TimePoint는 정확히 1개만 허용 (Partial Unique Index)
/// - `name`은 Subject 내에서 유일해야 함
/// - `external_key`는 전역적으로 유일해야 함 (CTIMS 연동 시)
///
/// # 예시
/// ```rust
/// use pacs_server::domain::entities::{TimePoint, VisitType};
/// use chrono::Utc;
///
/// let baseline = TimePoint {
///     id: 1,
///     project_id: 1,
///     subject_id: 1,
///     name: "Baseline".to_string(),
///     visit_type: VisitType::Baseline,
///     visit_no: None,
///     order_index: 0,
///     external_key: None,
///     created_at: Utc::now(),
///     updated_at: Utc::now(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct TimePoint {
    /// 데이터베이스에서 자동 생성되는 고유 식별자
    pub id: i32,
    /// 소속 프로젝트 ID
    pub project_id: i32,
    /// 소속 Subject ID
    pub subject_id: i32,
    /// TimePoint 이름 (BL, TP1, TP2 등)
    pub name: String,
    /// Visit 타입 (Baseline, Visit, EOT, USV)
    pub visit_type: VisitType,
    /// CTIMS Visit Number (연동 시 사용, nullable)
    pub visit_no: Option<i32>,
    /// TimePoint 정렬 순서 (0부터 시작)
    pub order_index: i32,
    /// CTIMS TimePoint Key (연동 시 사용, nullable)
    pub external_key: Option<String>,
    /// TimePoint 생성 시각
    pub created_at: DateTime<Utc>,
    /// TimePoint 수정 시각
    pub updated_at: DateTime<Utc>,
}

/// TimePoint 생성 요청 DTO
///
/// 새로운 TimePoint를 생성할 때 사용하는 데이터 전송 객체입니다.
///
/// # 필드
/// - `subject_id`: 소속 Subject ID (경로 파라미터로 전달되는 경우 생략 가능)
/// - `name`: TimePoint 이름 (BL, TP1, TP2 등)
/// - `visit_type`: Visit 타입 (Baseline, Visit, EOT, USV)
/// - `visit_no`: CTIMS Visit Number (선택사항)
/// - `order_index`: TimePoint 정렬 순서
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateTimePoint {
    /// 소속 Subject ID (경로 파라미터로 전달되는 경우 생략 가능)
    #[serde(default)]
    pub subject_id: i32,
    /// TimePoint 이름 (BL, TP1, TP2 등)
    pub name: String,
    /// Visit 타입 (Baseline, Visit, EOT, USV)
    pub visit_type: VisitType,
    /// CTIMS Visit Number (선택사항)
    pub visit_no: Option<i32>,
    /// TimePoint 정렬 순서
    pub order_index: i32,
}

/// TimePoint 수정 요청 DTO
///
/// 기존 TimePoint 정보를 수정할 때 사용하는 데이터 전송 객체입니다.
///
/// # 필드
/// - `name`: TimePoint 이름 (선택사항)
/// - `visit_type`: Visit 타입 (선택사항)
/// - `order_index`: TimePoint 정렬 순서 (선택사항)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateTimePoint {
    /// TimePoint 이름 (선택사항)
    pub name: Option<String>,
    /// Visit 타입 (선택사항)
    pub visit_type: Option<VisitType>,
    /// TimePoint 정렬 순서 (선택사항)
    pub order_index: Option<i32>,
}

/// TimePoint with Studies 응답 DTO
///
/// TimePoint와 해당 TimePoint에 할당된 Study 목록을 포함합니다.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TimePointWithStudies {
    /// TimePoint 정보
    #[serde(flatten)]
    pub timepoint: TimePoint,
    /// 할당된 Study 목록
    pub studies: Vec<StudyInfo>,
}

/// Subject의 TimePoints with Studies 응답 DTO
///
/// Subject의 모든 TimePoint와 각 TimePoint에 할당된 Study 목록,
/// 그리고 Unassigned Study 목록을 포함합니다.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TimePointsWithStudiesResponse {
    /// Subject ID
    pub subject_id: i32,
    /// Subject Code
    pub subject_code: String,
    /// TimePoint 목록 (Study 포함)
    pub timepoints: Vec<TimePointWithStudies>,
    /// Unassigned Study 목록
    pub unassigned_studies: Vec<StudyInfo>,
}

