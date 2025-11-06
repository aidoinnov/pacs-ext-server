use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
// use chrono::NaiveDateTime;
use chrono::{DateTime, Utc};

/// 사용자 생성 요청 DTO
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CreateUserRequest {
    pub keycloak_id: Uuid,
    pub username: String,
    pub email: String,
    /// 사용자의 실명 (선택사항)
    #[schema(example = "홍길동")]
    pub full_name: Option<String>,
    /// 소속 기관 (선택사항)
    #[schema(example = "서울대학교병원")]
    pub organization: Option<String>,
    /// 소속 부서/그룹 (선택사항)
    #[schema(example = "영상의학과")]
    pub department: Option<String>,
    /// 연락처 (선택사항)
    #[schema(example = "010-1234-5678")]
    pub phone: Option<String>,
}

/// 사용자 업데이트 요청 DTO
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct UpdateUserRequest {
    /// 이메일 주소 (선택사항)
    #[schema(example = "hong@example.com")]
    pub email: Option<String>,
    /// 사용자의 실명 (선택사항)
    #[schema(example = "홍길동")]
    pub full_name: Option<String>,
    /// 소속 기관 (선택사항)
    #[schema(example = "서울대학교병원")]
    pub organization: Option<String>,
    /// 소속 부서/그룹 (선택사항)
    #[schema(example = "영상의학과")]
    pub department: Option<String>,
    /// 연락처 (선택사항)
    #[schema(example = "010-1234-5678")]
    pub phone: Option<String>,
}

/// 사용자 응답 DTO
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct UserResponse {
    pub id: i32,
    pub keycloak_id: Uuid,
    pub username: String,
    pub email: String,
    /// 사용자의 실명
    #[schema(example = "홍길동")]
    pub full_name: Option<String>,
    /// 소속 기관
    #[schema(example = "서울대학교병원")]
    pub organization: Option<String>,
    /// 소속 부서/그룹
    #[schema(example = "영상의학과")]
    pub department: Option<String>,
    /// 연락처
    #[schema(example = "010-1234-5678")]
    pub phone: Option<String>,
    /// 계정 상태
    #[schema(example = "Active")]
    pub account_status: String,
    /// 이메일 인증 여부
    #[schema(example = true)]
    pub email_verified: bool,
    #[schema(value_type = String, example = "2024-01-01T00:00:00Z")]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, example = "2024-01-02T00:00:00Z")]
    pub updated_at: Option<DateTime<Utc>>,
}

impl From<crate::domain::entities::user::User> for UserResponse {
    fn from(user: crate::domain::entities::user::User) -> Self {
        Self {
            id: user.id,
            keycloak_id: user.keycloak_id,
            username: user.username,
            email: user.email,
            full_name: user.full_name,
            organization: user.organization,
            department: user.department,
            phone: user.phone,
            account_status: format!("{:?}", user.account_status),
            email_verified: user.email_verified,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

/// 사용자 목록 쿼리 DTO
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct UserListQuery {
    /// 페이지 번호 (기본값: 1)
    #[serde(default = "default_page")]
    #[schema(example = 1)]
    pub page: Option<i32>,

    /// 페이지 크기 (기본값: 20, 최대: 100)
    #[serde(default = "default_page_size")]
    #[schema(example = 20)]
    pub page_size: Option<i32>,

    /// 정렬 기준 (username, email, created_at)
    #[serde(default)]
    #[schema(example = "username")]
    pub sort_by: Option<String>,

    /// 정렬 순서 (asc, desc)
    #[serde(default)]
    #[schema(example = "asc")]
    pub sort_order: Option<String>,

    /// 검색어 (username, email 검색)
    #[serde(default)]
    #[schema(example = "john")]
    pub search: Option<String>,
}

fn default_page() -> Option<i32> {
    Some(1)
}
fn default_page_size() -> Option<i32> {
    Some(20)
}

/// 사용자 목록 응답 DTO
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct UserListResponse {
    pub users: Vec<UserResponse>,
    pub pagination: PaginationInfo,
}

/// 페이지네이션 정보
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct PaginationInfo {
    /// 현재 페이지 번호
    pub page: i32,
    /// 페이지 크기
    pub page_size: i32,
    /// 전체 항목 수
    pub total: i32,
    /// 전체 페이지 수
    pub total_pages: i32,
}

/// 프로젝트 멤버 추가 요청 DTO
#[derive(Debug, Deserialize, Serialize)]
pub struct AddProjectMemberRequest {
    pub user_id: i32,
    pub project_id: i32,
}

/// 프로젝트 목록 응답 DTO (사용자별)
#[derive(Debug, Deserialize, Serialize)]
pub struct UserProjectsResponse {
    pub user_id: i32,
    pub projects: Vec<ProjectSummary>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProjectSummary {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
}
