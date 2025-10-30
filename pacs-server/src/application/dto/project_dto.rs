use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use chrono::{DateTime, Utc, NaiveDate};

/// 프로젝트 생성 요청 DTO
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: Option<String>,
    #[schema(example = "서울대학교병원")]
    pub sponsor: String,
    #[schema(value_type = String, example = "2025-01-01")]
    pub start_date: NaiveDate,
    #[schema(value_type = String, example = "2025-12-31")]
    pub end_date: Option<NaiveDate>,
    #[schema(example = false)]
    pub auto_complete: Option<bool>,
}

/// 빈 문자열을 None으로 변환하는 커스텀 deserializer
fn deserialize_optional_date<'de, D>(deserializer: D) -> Result<Option<NaiveDate>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum DateOrString {
        Date(NaiveDate),
        String(String),
    }

    match DateOrString::deserialize(deserializer)? {
        DateOrString::Date(d) => Ok(Some(d)),
        DateOrString::String(s) if s.is_empty() => Ok(None),
        DateOrString::String(s) => {
            NaiveDate::parse_from_str(&s, "%Y-%m-%d")
                .map(Some)
                .map_err(serde::de::Error::custom)
        }
    }
}

/// 프로젝트 업데이트 요청 DTO
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub sponsor: Option<String>,
    #[schema(value_type = String, example = "2025-01-01")]
    pub start_date: Option<NaiveDate>,
    #[schema(value_type = String, example = "2025-12-31")]
    #[serde(deserialize_with = "deserialize_optional_date")]
    pub end_date: Option<NaiveDate>,
    pub status: Option<String>,
    pub auto_complete: Option<bool>,
    pub is_active: Option<bool>,
}

/// 프로젝트 응답 DTO
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ProjectResponse {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub sponsor: String,
    #[schema(value_type = String)]
    pub start_date: NaiveDate,
    #[schema(value_type = String)]
    pub end_date: Option<NaiveDate>,
    pub auto_complete: bool,
    pub is_active: bool,
    pub status: String,
    #[schema(value_type = String, example = "2024-01-01T00:00:00")]
    pub created_at: DateTime<Utc>,
}

impl From<crate::domain::entities::project::Project> for ProjectResponse {
    fn from(project: crate::domain::entities::project::Project) -> Self {
        Self {
            id: project.id,
            name: project.name,
            description: project.description,
            sponsor: project.sponsor,
            start_date: project.start_date,
            end_date: project.end_date,
            auto_complete: project.auto_complete,
            is_active: project.is_active,
            status: format!("{:?}", project.status),
            created_at: project.created_at,
        }
    }
}

/// 프로젝트 목록 응답 DTO
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ProjectListResponse {
    pub projects: Vec<ProjectResponse>,
    pub pagination: PaginationInfo,
}

/// 페이지네이션 정보
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct PaginationInfo {
    pub page: i32,
    pub page_size: i32,
    pub total: i64,
    pub total_pages: i32,
}

/// 프로젝트 역할 할당 요청 DTO
#[derive(Debug, Deserialize, Serialize)]
pub struct ProjectAssignRoleRequest {
    pub role_id: i32,
}

/// 프로젝트 멤버 목록 응답 DTO
#[derive(Debug, Deserialize, Serialize)]
pub struct ProjectMembersResponse {
    pub project_id: i32,
    pub members: Vec<MemberInfo>,
    pub total: i64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MemberInfo {
    pub id: i32,
    pub username: String,
    pub email: String,
    // pub joined_at: NaiveDateTime,
    pub joined_at: DateTime<Utc>,
}

/// 프로젝트 역할 목록 응답 DTO
#[derive(Debug, Deserialize, Serialize)]
pub struct ProjectRolesResponse {
    pub project_id: i32,
    pub roles: Vec<RoleInfo>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RoleInfo {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub scope: String,
}

/// 프로젝트 목록 조회 쿼리 DTO
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ProjectListQuery {
    /// 상태별 필터링
    pub status: Option<String>,
    /// 스폰서별 필터링
    pub sponsor: Option<String>,
    /// 시작일 범위 검색 (시작)
    #[schema(value_type = String, example = "2025-01-01")]
    pub start_date_from: Option<NaiveDate>,
    /// 시작일 범위 검색 (종료)
    #[schema(value_type = String, example = "2025-12-31")]
    pub start_date_to: Option<NaiveDate>,
    /// 종료일 범위 검색 (시작)
    #[schema(value_type = String, example = "2025-01-01")]
    pub end_date_from: Option<NaiveDate>,
    /// 종료일 범위 검색 (종료)
    #[schema(value_type = String, example = "2025-12-31")]
    pub end_date_to: Option<NaiveDate>,
    /// 페이지 번호 (기본값: 1)
    pub page: Option<i32>,
    /// 페이지 크기 (기본값: 10)
    pub page_size: Option<i32>,
    /// 정렬 기준 (created_at, name, start_date)
    pub sort_by: Option<String>,
    /// 정렬 순서 (asc, desc)
    pub sort_order: Option<String>,
}
