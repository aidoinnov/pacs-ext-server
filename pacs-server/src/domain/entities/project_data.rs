use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 리소스 레벨 (STUDY, SERIES, INSTANCE)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, PartialEq, Eq, Default)]
#[sqlx(type_name = "resource_level_enum", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ResourceLevel {
    #[default]
    Study,
    Series,
    Instance,
}

impl std::fmt::Display for ResourceLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResourceLevel::Study => write!(f, "STUDY"),
            ResourceLevel::Series => write!(f, "SERIES"),
            ResourceLevel::Instance => write!(f, "INSTANCE"),
        }
    }
}

/// 프로젝트 데이터 (DICOM Study)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProjectDataStudy {
    pub id: i32,
    pub project_id: i32,
    pub study_uid: String,
    pub study_description: Option<String>,
    pub patient_id: Option<String>,
    pub patient_name: Option<String>,
    pub patient_birth_date: Option<NaiveDate>,
    pub study_date: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 프로젝트 데이터 (DICOM Series)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProjectDataSeries {
    pub id: i32,
    pub study_id: i32,
    pub series_uid: String,
    pub series_description: Option<String>,
    pub modality: Option<String>,
    pub series_number: Option<i32>,
    pub created_at: DateTime<Utc>,
}

/// 프로젝트 데이터 접근 권한 (Study, Series, Instance 레벨)
/// NOTE: project_data_id는 하위 호환성을 위한 임시 필드 (기존 project_data 테이블 참조)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProjectDataAccess {
    pub id: i32,
    #[sqlx(default)]
    pub project_id: i32,
    #[sqlx(default)]
    pub resource_level: ResourceLevel,
    #[sqlx(default)]
    pub study_id: Option<i32>, // NULL 허용 (기존 데이터 호환성)
    #[sqlx(default)]
    pub series_id: Option<i32>,
    #[sqlx(default)]
    pub project_data_id: i32, // 임시 필드: 기존 테이블 호환성 유지
    pub user_id: i32,
    pub status: DataAccessStatus,
    pub requested_at: Option<DateTime<Utc>>,
    pub requested_by: Option<i32>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub reviewed_by: Option<i32>,
    pub review_note: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// 기존 ProjectData는 하위 호환성을 위해 유지
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProjectData {
    pub id: i32,
    pub project_id: i32,
    pub study_uid: String,
    pub study_description: Option<String>,
    pub patient_id: Option<String>,
    pub patient_name: Option<String>,
    pub study_date: Option<NaiveDate>,
    pub modality: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// 데이터 접근 상태
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(
    type_name = "data_access_status_enum",
    rename_all = "SCREAMING_SNAKE_CASE"
)]
pub enum DataAccessStatus {
    Approved,
    Denied,
    Pending,
}

impl std::fmt::Display for DataAccessStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataAccessStatus::Approved => write!(f, "APPROVED"),
            DataAccessStatus::Denied => write!(f, "DENIED"),
            DataAccessStatus::Pending => write!(f, "PENDING"),
        }
    }
}

impl std::str::FromStr for DataAccessStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "APPROVED" => Ok(DataAccessStatus::Approved),
            "DENIED" => Ok(DataAccessStatus::Denied),
            "PENDING" => Ok(DataAccessStatus::Pending),
            _ => Err(format!("Invalid status: {}", s)),
        }
    }
}

/// 새로운 프로젝트 데이터 생성
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewProjectData {
    pub project_id: i32,
    pub study_uid: String,
    pub study_description: Option<String>,
    pub patient_id: Option<String>,
    pub patient_name: Option<String>,
    pub study_date: Option<NaiveDate>,
    pub modality: Option<String>,
}

/// 프로젝트 데이터 업데이트
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProjectData {
    pub study_description: Option<String>,
    pub patient_id: Option<String>,
    pub patient_name: Option<String>,
    pub study_date: Option<NaiveDate>,
    pub modality: Option<String>,
}

/// 새로운 프로젝트 데이터 접근 권한 생성 (기존 - 하위 호환용)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewProjectDataAccess {
    pub project_data_id: i32,
    pub user_id: i32,
    pub status: DataAccessStatus,
    pub requested_at: Option<DateTime<Utc>>,
    pub requested_by: Option<i32>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub reviewed_by: Option<i32>,
    pub review_note: Option<String>,
}

/// 새로운 계층적 데이터 접근 권한 생성
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewHierarchicalDataAccess {
    pub project_id: i32,
    pub user_id: i32,
    pub resource_level: ResourceLevel,
    pub study_id: i32,
    pub series_id: Option<i32>,
    pub status: DataAccessStatus,
    pub requested_at: Option<DateTime<Utc>>,
    pub requested_by: Option<i32>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub reviewed_by: Option<i32>,
    pub review_note: Option<String>,
}

/// 프로젝트 데이터 접근 권한 업데이트
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProjectDataAccess {
    pub status: Option<DataAccessStatus>,
    pub requested_at: Option<DateTime<Utc>>,
    pub requested_by: Option<i32>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub reviewed_by: Option<i32>,
    pub review_note: Option<String>,
}

impl NewProjectData {
    pub fn new(project_id: i32, study_uid: String) -> Self {
        Self {
            project_id,
            study_uid,
            study_description: None,
            patient_id: None,
            patient_name: None,
            study_date: None,
            modality: None,
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.study_description = Some(description);
        self
    }

    pub fn with_patient_info(mut self, patient_id: String, patient_name: String) -> Self {
        self.patient_id = Some(patient_id);
        self.patient_name = Some(patient_name);
        self
    }

    pub fn with_study_date(mut self, study_date: NaiveDate) -> Self {
        self.study_date = Some(study_date);
        self
    }

    pub fn with_modality(mut self, modality: String) -> Self {
        self.modality = Some(modality);
        self
    }
}

impl NewProjectDataAccess {
    pub fn new(project_data_id: i32, user_id: i32, status: DataAccessStatus) -> Self {
        Self {
            project_data_id,
            user_id,
            status,
            requested_at: None,
            requested_by: None,
            reviewed_at: None,
            reviewed_by: None,
            review_note: None,
        }
    }

    pub fn with_request_info(mut self, requested_by: i32) -> Self {
        self.requested_at = Some(Utc::now());
        self.requested_by = Some(requested_by);
        self
    }

    pub fn with_review_info(mut self, reviewed_by: i32, review_note: Option<String>) -> Self {
        self.reviewed_at = Some(Utc::now());
        self.reviewed_by = Some(reviewed_by);
        self.review_note = review_note;
        self
    }
}

impl Default for UpdateProjectDataAccess {
    fn default() -> Self {
        Self {
            status: None,
            requested_at: None,
            requested_by: None,
            reviewed_at: None,
            reviewed_by: None,
            review_note: None,
        }
    }
}
