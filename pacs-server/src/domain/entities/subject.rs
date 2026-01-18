//! # Subject 엔티티 모듈
//!
//! 이 모듈은 임상시험 프로젝트의 Subject(환자) 정보를 나타내는 엔티티들을 정의합니다.
//! Subject는 프로젝트 내에서 환자를 식별하는 논리적 엔티티이며,
//! CTIMS(Clinical Trial Information Management System) 연동을 대비한 설계입니다.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

/// Subject 엔티티
///
/// 프로젝트 내 환자(Subject)를 나타내는 엔티티입니다.
/// CTIMS 연동 전에는 내부적으로 관리되며, 연동 후에는 external_subject_key를 통해 매핑됩니다.
///
/// # 필드
/// - `id`: 데이터베이스에서 자동 생성되는 고유 식별자
/// - `project_id`: 소속 프로젝트 ID
/// - `subject_code`: Subject 코드 (A001, B002 등) - 프로젝트 내 유일
/// - `external_subject_key`: CTIMS Subject PK (연동 시 사용, nullable)
/// - `patient_id`: PACS Patient ID
/// - `patient_name`: 환자 이름
/// - `patient_birth_date`: 환자 생년월일
/// - `created_at`: Subject 생성 시각
/// - `updated_at`: Subject 수정 시각
///
/// # 제약 조건
/// - `subject_code`는 프로젝트 내에서 유일해야 함
/// - `patient_id`는 프로젝트 내에서 유일해야 함 (동일 환자 중복 방지)
/// - `external_subject_key`는 전역적으로 유일해야 함 (CTIMS 연동 시)
///
/// # 예시
/// ```rust
/// use pacs_server::domain::entities::Subject;
/// use chrono::{DateTime, NaiveDate, Utc};
///
/// let subject = Subject {
///     id: 1,
///     project_id: 1,
///     subject_code: "A001".to_string(),
///     external_subject_key: None,
///     patient_id: Some("P12345".to_string()),
///     patient_name: Some("홍길동".to_string()),
///     patient_birth_date: Some(NaiveDate::from_ymd_opt(1980, 1, 1).unwrap()),
///     created_at: Utc::now(),
///     updated_at: Utc::now(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Subject {
    /// 데이터베이스에서 자동 생성되는 고유 식별자
    pub id: i32,
    /// 소속 프로젝트 ID
    pub project_id: i32,
    /// Subject 코드 (A001, B002 등)
    pub subject_code: String,
    /// CTIMS Subject PK (연동 시 사용, nullable)
    pub external_subject_key: Option<String>,
    /// PACS Patient ID
    pub patient_id: Option<String>,
    /// 환자 이름
    pub patient_name: Option<String>,
    /// 환자 생년월일
    pub patient_birth_date: Option<NaiveDate>,
    /// Subject 생성 시각
    pub created_at: DateTime<Utc>,
    /// Subject 수정 시각
    pub updated_at: DateTime<Utc>,
}

/// Subject 생성 요청 DTO (내부용)
///
/// 새로운 Subject를 생성할 때 사용하는 데이터 전송 객체입니다.
///
/// # 필드
/// - `project_id`: 소속 프로젝트 ID
/// - `subject_code`: Subject 코드 (A001, B002 등)
/// - `patient_id`: PACS Patient ID (선택사항)
/// - `patient_name`: 환자 이름 (선택사항)
/// - `patient_birth_date`: 환자 생년월일 (선택사항)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSubject {
    /// 소속 프로젝트 ID
    pub project_id: i32,
    /// Subject 코드 (A001, B002 등)
    pub subject_code: String,
    /// PACS Patient ID (선택사항)
    pub patient_id: Option<String>,
    /// 환자 이름 (선택사항)
    pub patient_name: Option<String>,
    /// 환자 생년월일 (선택사항)
    pub patient_birth_date: Option<NaiveDate>,
}

/// Subject 생성 요청 DTO (API용 - project_id는 URL 경로에서 받음)
///
/// API 엔드포인트에서 사용하는 Subject 생성 요청 DTO입니다.
/// project_id는 URL 경로에서 받으므로 포함하지 않습니다.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateSubjectRequest {
    /// Subject 코드 (A001, B002 등)
    pub subject_code: String,
    /// PACS Patient ID (선택사항)
    pub patient_id: Option<String>,
    /// 환자 이름 (선택사항)
    pub patient_name: Option<String>,
    /// 환자 생년월일 (선택사항)
    pub patient_birth_date: Option<NaiveDate>,
}

/// Subject 수정 요청 DTO
///
/// 기존 Subject 정보를 수정할 때 사용하는 데이터 전송 객체입니다.
///
/// # 필드
/// - `subject_code`: Subject 코드 (선택사항)
/// - `external_subject_key`: CTIMS Subject PK (선택사항)
/// - `patient_id`: PACS Patient ID (선택사항)
/// - `patient_name`: 환자 이름 (선택사항)
/// - `patient_birth_date`: 환자 생년월일 (선택사항)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateSubject {
    /// Subject 코드 (선택사항)
    pub subject_code: Option<String>,
    /// CTIMS Subject PK (선택사항)
    pub external_subject_key: Option<String>,
    /// PACS Patient ID (선택사항)
    pub patient_id: Option<String>,
    /// 환자 이름 (선택사항)
    pub patient_name: Option<String>,
    /// 환자 생년월일 (선택사항)
    pub patient_birth_date: Option<NaiveDate>,
}

/// Subject 상세 정보 (통계 포함)
///
/// Subject 조회 시 TimePoint 및 Study 개수 등의 통계 정보를 포함한 응답 DTO입니다.
///
/// # 필드
/// - `subject`: Subject 기본 정보
/// - `timepoint_count`: 소속 TimePoint 개수
/// - `study_count`: 할당된 Study 개수
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SubjectDetail {
    /// Subject 기본 정보
    #[serde(flatten)]
    pub subject: Subject,
    /// 소속 TimePoint 개수
    pub timepoint_count: i64,
    /// 할당된 Study 개수
    pub study_count: i64,
}

impl Subject {
    /// Subject 코드 유효성 검증
    ///
    /// Subject 코드는 다음 규칙을 따라야 합니다:
    /// - 1~50자 이내
    /// - 영문자, 숫자, 하이픈(-), 언더스코어(_)만 허용
    ///
    /// # 예시
    /// ```rust
    /// assert!(Subject::validate_subject_code("A001"));
    /// assert!(Subject::validate_subject_code("SUB-001"));
    /// assert!(!Subject::validate_subject_code(""));
    /// assert!(!Subject::validate_subject_code("A@001"));
    /// ```
    pub fn validate_subject_code(code: &str) -> bool {
        if code.is_empty() || code.len() > 50 {
            return false;
        }
        code.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    }
}

