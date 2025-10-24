//! # 프로젝트-사용자 매트릭스 DTO 모듈
//! 
//! 이 모듈은 프로젝트-사용자 역할 매트릭스 API를 위한 DTO들을 정의합니다.
//! 매트릭스는 관리 UI에서 테이블 형태로 보여주기 위한 것으로,
//! 열은 사용자 목록, 행은 프로젝트 목록이며, 각 셀에는 해당 프로젝트에서 사용자의 역할이 표시됩니다.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// 매트릭스에서 사용자-역할 정보
/// 
/// 각 셀은 특정 프로젝트에서 특정 사용자의 역할 정보를 나타냅니다.
/// 역할이 할당되지 않은 경우 role_id와 role_name은 None이 됩니다.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserRoleCell {
    /// 사용자 ID
    pub user_id: i32,
    /// 사용자명
    pub username: String,
    /// 이메일
    pub email: String,
    /// 역할 ID (역할이 할당되지 않은 경우 None)
    pub role_id: Option<i32>,
    /// 역할명 (역할이 할당되지 않은 경우 None)
    pub role_name: Option<String>,
}

/// 매트릭스의 한 행 (프로젝트 + 사용자 역할들)
/// 
/// 각 행은 하나의 프로젝트와 해당 프로젝트에서의 모든 사용자들의 역할 정보를 포함합니다.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ProjectUserMatrixRow {
    /// 프로젝트 ID
    pub project_id: i32,
    /// 프로젝트명
    pub project_name: String,
    /// 프로젝트 설명
    pub description: Option<String>,
    /// 프로젝트 상태 (문자열 형태)
    pub status: String,
    /// 해당 프로젝트에서의 사용자 역할 목록
    pub user_roles: Vec<UserRoleCell>,
}

/// 매트릭스 응답 (전체 데이터 + 메타데이터)
/// 
/// 매트릭스 데이터와 페이지네이션 정보를 포함하는 최종 응답 구조체입니다.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ProjectUserMatrixResponse {
    /// 매트릭스 행 목록 (프로젝트별)
    pub matrix: Vec<ProjectUserMatrixRow>,
    /// 사용자 정보 목록 (열 헤더용)
    pub users: Vec<UserInfo>,
    /// 페이지네이션 정보
    pub pagination: MatrixPagination,
}

/// 사용자 기본 정보 (열 헤더용)
/// 
/// 매트릭스의 열 헤더에 표시될 사용자 정보입니다.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserInfo {
    /// 사용자 ID
    pub user_id: i32,
    /// 사용자명
    pub username: String,
    /// 이메일
    pub email: String,
    /// 실명 (선택사항)
    pub full_name: Option<String>,
}

/// 매트릭스 페이지네이션 정보
/// 
/// 프로젝트와 사용자에 대한 이중 페이지네이션 정보를 포함합니다.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MatrixPagination {
    /// 프로젝트 페이지 번호
    pub project_page: i32,
    /// 프로젝트 페이지 크기
    pub project_page_size: i32,
    /// 프로젝트 총 개수
    pub project_total_count: i64,
    /// 프로젝트 총 페이지 수
    pub project_total_pages: i32,
    /// 사용자 페이지 번호
    pub user_page: i32,
    /// 사용자 페이지 크기
    pub user_page_size: i32,
    /// 사용자 총 개수
    pub user_total_count: i64,
    /// 사용자 총 페이지 수
    pub user_total_pages: i32,
}

/// 매트릭스 조회 쿼리 파라미터
/// 
/// 매트릭스 API 호출 시 사용되는 쿼리 파라미터들입니다.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MatrixQueryParams {
    /// 프로젝트 페이지 번호 (기본값: 1)
    pub project_page: Option<i32>,
    /// 프로젝트 페이지 크기 (기본값: 10, 최대: 50)
    pub project_page_size: Option<i32>,
    
    /// 사용자 페이지 번호 (기본값: 1)
    pub user_page: Option<i32>,
    /// 사용자 페이지 크기 (기본값: 10, 최대: 50)
    pub user_page_size: Option<i32>,
    
    /// 프로젝트 상태 필터 (예: ["IN_PROGRESS", "PREPARING"])
    pub project_status: Option<Vec<String>>,
    /// 특정 프로젝트 ID 목록 (예: [1, 2, 3])
    pub project_ids: Option<Vec<i32>>,
    /// 특정 사용자 ID 목록 (예: [1, 2, 3])
    pub user_ids: Option<Vec<i32>>,
}
