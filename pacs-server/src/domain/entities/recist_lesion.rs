//! # RECIST Lesion 엔티티 모듈
//!
//! 이 모듈은 RECIST 1.1 기준에 따른 병변(Lesion) 정보를 나타내는 엔티티들을 정의합니다.
//! RECIST Lesion은 Target, Non-target, New Lesion으로 분류되며,
//! Baseline TimePoint를 기준으로 추적됩니다.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

/// RECIST Lesion 타입
///
/// RECIST 1.1 기준에 따른 병변 분류입니다.
///
/// # 타입
/// - `TARGET`: 측정 가능한 병변 (Baseline에서만 생성 가능)
/// - `NON_TARGET`: 측정 불가능한 병변 (Baseline에서만 생성 가능)
/// - `NEW`: Follow-up에서 새로 발견된 병변 (Baseline에서 생성 불가)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[sqlx(type_name = "recist_lesion_type_enum", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RecistLesionType {
    /// 측정 가능한 병변 (Baseline에서만 생성)
    Target,
    /// 측정 불가능한 병변 (Baseline에서만 생성)
    NonTarget,
    /// 새로 발견된 병변 (Follow-up에서만 생성)
    New,
}

/// RECIST Lesion 엔티티
///
/// RECIST 1.1 기준에 따른 병변 정보를 나타내는 엔티티입니다.
///
/// # 필드
/// - `id`: 데이터베이스에서 자동 생성되는 고유 식별자
/// - `project_id`: 소속 프로젝트 ID
/// - `subject_id`: 소속 Subject ID
/// - `lesion_type`: Lesion 타입 (TARGET/NON_TARGET/NEW)
/// - `lesion_number`: Subject 내 병변 순번 (1, 2, 3, ...)
/// - `baseline_timepoint_id`: Baseline TimePoint ID (TARGET/NON_TARGET 필수, NEW는 NULL)
/// - `organ_site`: 병변 위치 (Liver, Lung, Lymph Node 등)
/// - `description`: 병변 상세 설명
/// - `created_at`: Lesion 생성 시각
/// - `updated_at`: Lesion 수정 시각
///
/// # 제약 조건
/// - `lesion_number`는 Subject 내에서 유일해야 함
/// - TARGET/NON_TARGET은 `baseline_timepoint_id` 필수
/// - NEW는 `baseline_timepoint_id`가 NULL이어야 함
///
/// # 예시
/// ```rust
/// use pacs_server::domain::entities::{RecistLesion, RecistLesionType};
/// use chrono::Utc;
///
/// let lesion = RecistLesion {
///     id: 1,
///     project_id: 1,
///     subject_id: 1,
///     lesion_type: RecistLesionType::Target,
///     lesion_number: 1,
///     baseline_timepoint_id: Some(1),
///     organ_site: Some("Liver".to_string()),
///     description: Some("Right lobe lesion".to_string()),
///     created_at: Utc::now(),
///     updated_at: Utc::now(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct RecistLesion {
    /// 데이터베이스에서 자동 생성되는 고유 식별자
    pub id: i32,
    /// 소속 프로젝트 ID
    pub project_id: i32,
    /// 소속 Subject ID
    pub subject_id: i32,
    /// Lesion 타입 (TARGET/NON_TARGET/NEW)
    pub lesion_type: RecistLesionType,
    /// Subject 내 병변 순번 (1, 2, 3, ...)
    pub lesion_number: i32,
    /// Baseline TimePoint ID (TARGET/NON_TARGET 필수, NEW는 NULL)
    pub baseline_timepoint_id: Option<i32>,
    /// 병변 위치 (Liver, Lung, Lymph Node 등)
    pub organ_site: Option<String>,
    /// 병변 상세 설명
    pub description: Option<String>,
    /// Lesion 생성 시각
    pub created_at: DateTime<Utc>,
    /// Lesion 수정 시각
    pub updated_at: DateTime<Utc>,
}

/// RECIST Lesion 생성 요청 DTO (내부용)
///
/// 새로운 RECIST Lesion을 생성할 때 사용하는 데이터 전송 객체입니다.
///
/// # 필드
/// - `project_id`: 소속 프로젝트 ID
/// - `subject_id`: 소속 Subject ID
/// - `lesion_type`: Lesion 타입 (TARGET/NON_TARGET/NEW)
/// - `baseline_timepoint_id`: Baseline TimePoint ID (TARGET/NON_TARGET 필수)
/// - `organ_site`: 병변 위치 (선택사항)
/// - `description`: 병변 상세 설명 (선택사항)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRecistLesion {
    /// 소속 프로젝트 ID
    pub project_id: i32,
    /// 소속 Subject ID
    pub subject_id: i32,
    /// Lesion 타입 (TARGET/NON_TARGET/NEW)
    pub lesion_type: RecistLesionType,
    /// Baseline TimePoint ID (TARGET/NON_TARGET 필수)
    pub baseline_timepoint_id: Option<i32>,
    /// 병변 위치 (선택사항)
    pub organ_site: Option<String>,
    /// 병변 상세 설명 (선택사항)
    pub description: Option<String>,
}

/// RECIST Lesion 생성 요청 DTO (API용 - subject_id는 URL 경로에서 받음)
///
/// API 엔드포인트에서 사용하는 RECIST Lesion 생성 요청 DTO입니다.
/// subject_id는 URL 경로에서 받으므로 포함하지 않습니다.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateRecistLesionRequest {
    /// Lesion 타입 (TARGET/NON_TARGET/NEW)
    pub lesion_type: RecistLesionType,
    /// Baseline TimePoint ID (TARGET/NON_TARGET 필수)
    pub baseline_timepoint_id: Option<i32>,
    /// 병변 위치 (선택사항)
    pub organ_site: Option<String>,
    /// 병변 상세 설명 (선택사항)
    pub description: Option<String>,
}

/// RECIST Lesion 수정 요청 DTO
///
/// 기존 RECIST Lesion 정보를 수정할 때 사용하는 데이터 전송 객체입니다.
/// lesion_type, lesion_number, baseline_timepoint_id는 수정 불가능합니다.
///
/// # 필드
/// - `organ_site`: 병변 위치 (선택사항)
/// - `description`: 병변 상세 설명 (선택사항)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateRecistLesion {
    /// 병변 위치 (선택사항)
    pub organ_site: Option<String>,
    /// 병변 상세 설명 (선택사항)
    pub description: Option<String>,
}

/// RECIST Lesion ↔ Annotation 매핑 엔티티
///
/// Lesion과 Annotation을 TimePoint별로 연결하는 매핑 테이블입니다.
/// 하나의 Lesion은 여러 TimePoint에서 여러 Annotation을 가질 수 있습니다.
///
/// # 필드
/// - `id`: 데이터베이스에서 자동 생성되는 고유 식별자
/// - `lesion_id`: RECIST Lesion ID
/// - `annotation_id`: Annotation ID
/// - `timepoint_id`: TimePoint ID
/// - `measured_length_mm`: 측정된 병변 길이 (mm)
/// - `measured_at`: 측정 시각
/// - `created_at`: 매핑 생성 시각
///
/// # 제약 조건
/// - (lesion_id, annotation_id) 유일 (하나의 Annotation은 하나의 Lesion에만 연결)
/// - (annotation_id, timepoint_id) 유일 (하나의 Annotation은 하나의 TimePoint에만 연결)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct RecistLesionAnnotationMap {
    /// 데이터베이스에서 자동 생성되는 고유 식별자
    pub id: i32,
    /// RECIST Lesion ID
    pub lesion_id: i32,
    /// Annotation ID
    pub annotation_id: i32,
    /// TimePoint ID
    pub timepoint_id: i32,
    /// 측정된 병변 길이 (mm)
    pub measured_length_mm: Option<f64>,
    /// 측정 시각
    pub measured_at: DateTime<Utc>,
    /// 매핑 생성 시각
    pub created_at: DateTime<Utc>,
}

/// RECIST Lesion ↔ Annotation 매핑 생성 요청 DTO
///
/// Annotation을 Lesion에 연결할 때 사용하는 데이터 전송 객체입니다.
///
/// # 필드
/// - `lesion_id`: RECIST Lesion ID
/// - `annotation_id`: Annotation ID
/// - `timepoint_id`: TimePoint ID
/// - `measured_length_mm`: 측정된 병변 길이 (mm, 선택사항)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRecistLesionAnnotationMap {
    /// RECIST Lesion ID
    pub lesion_id: i32,
    /// Annotation ID
    pub annotation_id: i32,
    /// TimePoint ID
    pub timepoint_id: i32,
    /// 측정된 병변 길이 (mm, 선택사항)
    pub measured_length_mm: Option<f64>,
}

/// RECIST Lesion 상세 정보 (Annotation 포함)
///
/// Lesion 조회 시 TimePoint별 Annotation 정보를 포함한 응답 DTO입니다.
///
/// # 필드
/// - `lesion`: Lesion 기본 정보
/// - `annotations`: TimePoint별 Annotation 목록
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RecistLesionDetail {
    /// Lesion 기본 정보
    #[serde(flatten)]
    pub lesion: RecistLesion,
    /// TimePoint별 Annotation 목록
    pub annotations: Vec<RecistLesionAnnotationInfo>,
}

/// RECIST Lesion Annotation 정보
///
/// Lesion 상세 조회 시 포함되는 Annotation 정보입니다.
///
/// # 필드
/// - `timepoint_id`: TimePoint ID
/// - `timepoint_name`: TimePoint 이름 (BL, TP1, TP2 등)
/// - `annotation_id`: Annotation ID
/// - `measured_length_mm`: 측정된 병변 길이 (mm)
/// - `measured_at`: 측정 시각
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RecistLesionAnnotationInfo {
    /// TimePoint ID
    pub timepoint_id: i32,
    /// TimePoint 이름 (BL, TP1, TP2 등)
    pub timepoint_name: String,
    /// Annotation ID
    pub annotation_id: i32,
    /// 측정된 병변 길이 (mm)
    pub measured_length_mm: Option<f64>,
    /// 측정 시각
    pub measured_at: DateTime<Utc>,
}

impl RecistLesion {
    /// Lesion 타입에 따른 Baseline TimePoint ID 유효성 검증
    ///
    /// # 규칙
    /// - TARGET/NON_TARGET: baseline_timepoint_id 필수
    /// - NEW: baseline_timepoint_id는 NULL이어야 함
    ///
    /// # 예시
    /// ```rust
    /// use pacs_server::domain::entities::RecistLesionType;
    ///
    /// assert!(RecistLesion::validate_baseline_requirement(RecistLesionType::Target, Some(1)));
    /// assert!(!RecistLesion::validate_baseline_requirement(RecistLesionType::Target, None));
    /// assert!(RecistLesion::validate_baseline_requirement(RecistLesionType::New, None));
    /// assert!(!RecistLesion::validate_baseline_requirement(RecistLesionType::New, Some(1)));
    /// ```
    pub fn validate_baseline_requirement(
        lesion_type: RecistLesionType,
        baseline_timepoint_id: Option<i32>,
    ) -> bool {
        match lesion_type {
            RecistLesionType::Target | RecistLesionType::NonTarget => {
                baseline_timepoint_id.is_some()
            }
            RecistLesionType::New => baseline_timepoint_id.is_none(),
        }
    }
}

