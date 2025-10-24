//! # 프로젝트-사용자 매트릭스 컨트롤러 모듈
//! 
//! 이 모듈은 프로젝트-사용자 역할 매트릭스 API 엔드포인트를 제공합니다.
//! 매트릭스는 관리 UI에서 테이블 형태로 보여주기 위한 것으로,
//! 열은 사용자 목록, 행은 프로젝트 목록이며, 각 셀에는 해당 프로젝트에서 사용자의 역할이 표시됩니다.

use actix_web::{web, HttpResponse, Responder};
use serde_json::json;
use std::sync::Arc;

use crate::application::use_cases::project_user_matrix_use_case::ProjectUserMatrixUseCase;
use crate::application::dto::project_user_matrix_dto::{MatrixQueryParams, ProjectUserMatrixResponse};
use crate::domain::services::{ProjectService, UserService};

/// 프로젝트-사용자 역할 매트릭스 조회
/// 
/// 프로젝트와 사용자의 역할 관계를 매트릭스 형태로 조회합니다.
/// 이중 페이지네이션(프로젝트/사용자)과 다양한 필터링 옵션을 지원합니다.
/// 
/// # Parameters
/// - `project_page`: 프로젝트 페이지 번호 (기본값: 1)
/// - `project_page_size`: 프로젝트 페이지 크기 (기본값: 10, 최대: 50)
/// - `user_page`: 사용자 페이지 번호 (기본값: 1)
/// - `user_page_size`: 사용자 페이지 크기 (기본값: 10, 최대: 50)
/// - `project_status`: 프로젝트 상태 필터 (PREPARING, IN_PROGRESS, COMPLETED, ON_HOLD, CANCELLED)
/// - `project_ids`: 특정 프로젝트 ID 목록
/// - `user_ids`: 특정 사용자 ID 목록
/// 
/// # Returns
/// - `200`: 매트릭스 데이터와 페이지네이션 정보
/// - `500`: 내부 서버 오류
#[utoipa::path(
    get,
    path = "/api/project-user-matrix",
    params(
        ("project_page" = Option<i32>, Query, description = "Project page number (default: 1)"),
        ("project_page_size" = Option<i32>, Query, description = "Project page size (default: 10, max: 50)"),
        ("user_page" = Option<i32>, Query, description = "User page number (default: 1)"),
        ("user_page_size" = Option<i32>, Query, description = "User page size (default: 10, max: 50)"),
        ("project_status" = Option<Vec<String>>, Query, description = "Project status filter (PREPARING, IN_PROGRESS, COMPLETED, ON_HOLD, CANCELLED)"),
        ("project_ids" = Option<Vec<i32>>, Query, description = "Specific project IDs to include"),
        ("user_ids" = Option<Vec<i32>>, Query, description = "Specific user IDs to include")
    ),
    responses(
        (status = 200, description = "Matrix retrieved successfully", body = ProjectUserMatrixResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "project-user-matrix"
)]
pub async fn get_matrix<P, U>(
    query: web::Query<MatrixQueryParams>,
    use_case: web::Data<Arc<ProjectUserMatrixUseCase<P, U>>>,
) -> impl Responder
where
    P: ProjectService,
    U: UserService,
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
pub fn configure_routes<P, U>(
    cfg: &mut web::ServiceConfig,
    use_case: Arc<ProjectUserMatrixUseCase<P, U>>,
) where
    P: ProjectService + 'static,
    U: UserService + 'static,
{
    cfg.app_data(web::Data::new(use_case))
        .route("/project-user-matrix", web::get().to(get_matrix::<P, U>));
}
