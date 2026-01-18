//! # TimePoint-Study 매핑 엔티티 모듈
//!
//! 이 모듈은 TimePoint와 Study 간의 매핑 관계를 나타내는 엔티티들을 정의합니다.
//! Study는 TimePoint에 할당되거나 Unassigned 상태로 존재할 수 있습니다.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// TimePoint-Study 매핑 엔티티
///
/// Study를 특정 TimePoint에 할당하는 관계를 나타냅니다.
/// Study는 한 번에 하나의 TimePoint에만 할당될 수 있습니다.
///
/// # 필드
/// - `id`: 데이터베이스에서 자동 생성되는 고유 식별자
/// - `timepoint_id`: TimePoint ID
/// - `study_id`: Study ID (project_data_study 테이블 참조)
/// - `assigned_by`: 할당한 사용자 ID
/// - `assigned_at`: 할당 시각
///
/// # 제약 조건
/// - `study_id`는 전역적으로 유일해야 함 (한 Study는 하나의 TimePoint에만 할당)
/// - TimePoint 삭제 시 매핑도 함께 삭제됨 (CASCADE DELETE)
///
/// # 예시
/// ```rust
/// use pacs_server::domain::entities::TimePointStudy;
/// use chrono::Utc;
///
/// let mapping = TimePointStudy {
///     id: 1,
///     timepoint_id: 1,
///     study_id: 100,
///     assigned_by: 5,
///     assigned_at: Utc::now(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TimePointStudy {
    /// 데이터베이스에서 자동 생성되는 고유 식별자
    pub id: i32,
    /// TimePoint ID
    pub timepoint_id: i32,
    /// Study ID (project_data_study 테이블 참조)
    pub study_id: i32,
    /// 할당한 사용자 ID
    pub assigned_by: i32,
    /// 할당 시각
    pub assigned_at: DateTime<Utc>,
}

/// Study 할당 요청 DTO
///
/// Study를 TimePoint에 할당할 때 사용하는 데이터 전송 객체입니다.
///
/// # 필드
/// - `study_ids`: 할당할 Study ID 목록
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignStudies {
    /// 할당할 Study ID 목록
    pub study_ids: Vec<i32>,
}

/// Study 할당 해제 요청 DTO
///
/// Study를 TimePoint에서 해제할 때 사용하는 데이터 전송 객체입니다.
///
/// # 필드
/// - `study_ids`: 해제할 Study ID 목록
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnassignStudies {
    /// 해제할 Study ID 목록
    pub study_ids: Vec<i32>,
}

/// Study 할당 결과 DTO
///
/// Study 할당/해제 작업의 결과를 나타내는 응답 DTO입니다.
///
/// # 필드
/// - `affected_count`: 영향받은 Study 개수
/// - `study_ids`: 영향받은 Study ID 목록
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignmentResult {
    /// 영향받은 Study 개수
    pub affected_count: i32,
    /// 영향받은 Study ID 목록
    pub study_ids: Vec<i32>,
}

/// TimePoint별 Study 목록 응답 DTO
///
/// TimePoint에 할당된 Study 목록을 조회할 때 사용하는 응답 DTO입니다.
///
/// # 필드
/// - `timepoint_id`: TimePoint ID
/// - `timepoint_name`: TimePoint 이름
/// - `studies`: Study 목록
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimePointStudies {
    /// TimePoint ID
    pub timepoint_id: i32,
    /// TimePoint 이름
    pub timepoint_name: String,
    /// Study 목록
    pub studies: Vec<StudyInfo>,
}

/// Study 정보 DTO
///
/// Study의 기본 정보를 나타내는 DTO입니다.
///
/// # 필드
/// - `study_id`: Study ID
/// - `study_uid`: DICOM Study Instance UID
/// - `study_description`: Study 설명
/// - `study_date`: Study 날짜
/// - `patient_id`: Patient ID
/// - `modality`: Modality
/// - `assigned_at`: 할당 시각 (할당된 경우)
/// - `assigned_by`: 할당한 사용자 ID (할당된 경우)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudyInfo {
    /// Study ID
    pub study_id: i32,
    /// DICOM Study Instance UID
    pub study_uid: String,
    /// Study 설명
    pub study_description: Option<String>,
    /// Study 날짜
    pub study_date: Option<String>,
    /// Patient ID
    pub patient_id: Option<String>,
    /// Modality
    pub modality: Option<String>,
    /// 할당 시각 (할당된 경우)
    pub assigned_at: Option<DateTime<Utc>>,
    /// 할당한 사용자 ID (할당된 경우)
    pub assigned_by: Option<i32>,
}

