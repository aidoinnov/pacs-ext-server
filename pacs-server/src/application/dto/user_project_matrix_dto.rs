//! # 유저-프로젝트 매트릭스 DTO 모듈
//!
//! 이 모듈은 유저 중심 매트릭스 API를 위한 DTO들을 정의합니다.
//! 매트릭스는 관리 UI에서 테이블 형태로 보여주기 위한 것으로,
//! 행은 유저 목록, 열은 프로젝트 목록이며, 각 셀에는 해당 유저의 프로젝트 역할이 표시됩니다.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// 멤버십 정보
///
/// 일괄 조회를 위한 멤버십 정보 구조체입니다.
#[derive(Debug, Clone)]
pub struct MembershipInfo {
    /// 역할 ID
    pub role_id: Option<i32>,
    /// 역할명
    pub role_name: Option<String>,
}

/// 매트릭스에서 프로젝트-역할 정보
///
/// 각 셀은 특정 유저가 특정 프로젝트에서 가진 역할 정보를 나타냅니다.
/// 역할이 할당되지 않은 경우 role_id와 role_name은 None이 됩니다.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ProjectRoleCell {
    /// 프로젝트 ID
    pub project_id: i32,
    /// 프로젝트명
    pub project_name: String,
    /// 역할 ID (역할이 할당되지 않은 경우 None)
    pub role_id: Option<i32>,
    /// 역할명 (역할이 할당되지 않은 경우 None)
    pub role_name: Option<String>,
}

/// 매트릭스의 한 행 (유저 + 프로젝트 역할들)
///
/// 각 행은 하나의 유저와 해당 유저가 가진 모든 프로젝트의 역할 정보를 포함합니다.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserProjectMatrixRow {
    /// 유저 ID
    pub user_id: i32,
    /// 유저명
    pub username: String,
    /// 이메일
    pub email: String,
    /// 실명 (선택사항)
    pub full_name: Option<String>,
    /// 해당 유저의 프로젝트 역할 목록
    pub project_roles: Vec<ProjectRoleCell>,
}

/// 매트릭스 응답 (전체 데이터 + 메타데이터)
///
/// 매트릭스 데이터와 페이지네이션 정보를 포함하는 최종 응답 구조체입니다.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserProjectMatrixResponse {
    /// 매트릭스 행 목록 (유저별)
    pub matrix: Vec<UserProjectMatrixRow>,
    /// 프로젝트 정보 목록 (열 헤더용)
    pub projects: Vec<ProjectInfo>,
    /// 페이지네이션 정보
    pub pagination: UserProjectMatrixPagination,
}

/// 프로젝트 기본 정보 (열 헤더용)
///
/// 매트릭스의 열 헤더에 표시될 프로젝트 정보입니다.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ProjectInfo {
    /// 프로젝트 ID
    pub project_id: i32,
    /// 프로젝트명
    pub project_name: String,
    /// 프로젝트 설명
    pub description: Option<String>,
    /// 프로젝트 상태 (문자열 형태)
    pub status: String,
}

/// 매트릭스 페이지네이션 정보
///
/// 유저와 프로젝트에 대한 이중 페이지네이션 정보를 포함합니다.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserProjectMatrixPagination {
    /// 유저 페이지 번호
    pub user_page: i32,
    /// 유저 페이지 크기
    pub user_page_size: i32,
    /// 유저 총 개수
    pub user_total_count: i64,
    /// 유저 총 페이지 수
    pub user_total_pages: i32,
    /// 프로젝트 페이지 번호
    pub project_page: i32,
    /// 프로젝트 페이지 크기
    pub project_page_size: i32,
    /// 프로젝트 총 개수
    pub project_total_count: i64,
    /// 프로젝트 총 페이지 수
    pub project_total_pages: i32,
}

/// 매트릭스 조회 쿼리 파라미터
///
/// 매트릭스 API 호출 시 사용되는 쿼리 파라미터들입니다.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserProjectMatrixQueryParams {
    /// 유저 페이지 번호 (기본값: 1)
    pub user_page: Option<i32>,
    /// 유저 페이지 크기 (기본값: 10, 최대: 50)
    pub user_page_size: Option<i32>,

    /// 프로젝트 페이지 번호 (기본값: 1)
    pub project_page: Option<i32>,
    /// 프로젝트 페이지 크기 (기본값: 10, 최대: 50)
    pub project_page_size: Option<i32>,

    /// 유저 정렬 기준 (username, email, created_at) (기본값: username)
    pub user_sort_by: Option<String>,
    /// 정렬 순서 (asc, desc) (기본값: asc)
    pub user_sort_order: Option<String>,
    /// 유저 이름/이메일 검색
    pub user_search: Option<String>,
    /// 역할 ID 필터
    pub role_id: Option<i32>,
    /// 특정 프로젝트 ID 목록 (예: [1, 2, 3])
    pub project_ids: Option<Vec<i32>>,
    /// 특정 유저 ID 목록 (예: [1, 2, 3])
    pub user_ids: Option<Vec<i32>>,
}
