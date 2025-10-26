//! # 유저-프로젝트 매트릭스 컨트롤러 모듈
//! 
//! 이 모듈은 유저 중심 매트릭스 API 엔드포인트를 제공합니다.
//! 매트릭스는 관리 UI에서 테이블 형태로 보여주기 위한 것으로,
//! 행은 유저 목록, 열은 프로젝트 목록이며, 각 셀에는 해당 유저의 프로젝트 역할이 표시됩니다.

use actix_web::{web, HttpResponse, Responder};
use serde_json::json;
use std::sync::Arc;

use crate::application::use_cases::user_project_matrix_use_case::UserProjectMatrixUseCase;
use crate::application::dto::user_project_matrix_dto::{UserProjectMatrixQueryParams, UserProjectMatrixResponse};
use crate::domain::services::{ProjectService, UserService};

/// 유저-프로젝트 역할 매트릭스 조회
/// 
/// 유저와 프로젝트의 역할 관계를 매트릭스 형태로 조회합니다.
/// 이중 페이지네이션(유저/프로젝트)과 다양한 필터링 옵션을 지원합니다.
/// 
/// # Parameters
/// - `user_page`: 유저 페이지 번호 (기본값: 1)
/// - `user_page_size`: 유저 페이지 크기 (기본값: 10, 최대: 50)
/// - `project_page`: 프로젝트 페이지 번호 (기본값: 1)
/// - `project_page_size`: 프로젝트 페이지 크기 (기본값: 10, 최대: 50)
/// - `user_sort_by`: 유저 정렬 기준 (username, email, created_at) (기본값: username)
/// - `user_sort_order`: 정렬 순서 (asc, desc) (기본값: asc)
/// - `user_search`: 유저 이름/이메일 검색
/// - `role_id`: 역할 ID 필터
/// - `project_ids`: 특정 프로젝트 ID 목록
/// - `user_ids`: 특정 유저 ID 목록
/// 
/// # Returns
/// - `200`: 매트릭스 데이터와 페이지네이션 정보
/// - `500`: 내부 서버 오류
#[utoipa::path(
    get,
    path = "/api/user-project-matrix",
    params(
        ("user_page" = Option<i32>, Query, description = "User page number (default: 1)"),
        ("user_page_size" = Option<i32>, Query, description = "User page size (default: 10, max: 50)"),
        ("project_page" = Option<i32>, Query, description = "Project page number (default: 1)"),
        ("project_page_size" = Option<i32>, Query, description = "Project page size (default: 10, max: 50)"),
        ("user_sort_by" = Option<String>, Query, description = "User sort field (username, email, created_at) (default: username)"),
        ("user_sort_order" = Option<String>, Query, description = "Sort order (asc, desc) (default: asc)"),
        ("user_search" = Option<String>, Query, description = "User name/email search"),
        ("role_id" = Option<i32>, Query, description = "Role ID filter"),
        ("project_ids" = Option<Vec<i32>>, Query, description = "Specific project IDs to include"),
        ("user_ids" = Option<Vec<i32>>, Query, description = "Specific user IDs to include")
    ),
    responses(
        (status = 200, description = "Matrix retrieved successfully", body = UserProjectMatrixResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "user-project-matrix"
)]
pub async fn get_matrix<U, P>(
    query: web::Query<UserProjectMatrixQueryParams>,
    use_case: web::Data<Arc<UserProjectMatrixUseCase<U, P>>>,
) -> impl Responder
where
    U: UserService,
    P: ProjectService,
{
    match use_case.get_matrix(query.into_inner()).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to get matrix: {}", e)
        })),
    }
}

/// 라우팅 설정
/// 
/// 매트릭스 API 엔드포인트를 설정합니다.
pub fn configure_routes<U, P>(
    cfg: &mut web::ServiceConfig,
    use_case: Arc<UserProjectMatrixUseCase<U, P>>,
) where
    U: UserService + 'static,
    P: ProjectService + 'static,
{
    cfg.app_data(web::Data::new(use_case))
        .route("/user-project-matrix", web::get().to(get_matrix::<U, P>));
}
