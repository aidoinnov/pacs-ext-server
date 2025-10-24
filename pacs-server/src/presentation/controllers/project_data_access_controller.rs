use actix_web::{web, HttpResponse, Result};
use std::sync::Arc;
use serde_json::json;

use crate::application::use_cases::ProjectDataAccessUseCase;
use crate::application::dto::project_data_access_dto::*;
use crate::domain::ServiceError;

/// ServiceError를 HttpResponse로 변환하는 헬퍼 함수
fn handle_service_error(error: ServiceError) -> HttpResponse {
    match error {
        ServiceError::NotFound(msg) => HttpResponse::NotFound().json(json!({
            "error": "Not Found",
            "message": msg
        })),
        ServiceError::ValidationError(msg) => HttpResponse::BadRequest().json(json!({
            "error": "Validation Error",
            "message": msg
        })),
        ServiceError::Unauthorized(msg) => HttpResponse::Unauthorized().json(json!({
            "error": "Unauthorized",
            "message": msg
        })),
        ServiceError::AlreadyExists(msg) => HttpResponse::Conflict().json(json!({
            "error": "Already Exists",
            "message": msg
        })),
        ServiceError::DatabaseError(msg) => HttpResponse::InternalServerError().json(json!({
            "error": "Database Error",
            "message": msg
        })),
        _ => HttpResponse::InternalServerError().json(json!({
            "error": "Internal Server Error",
            "message": "An unexpected error occurred"
        })),
    }
}

/// 프로젝트 데이터 접근 매트릭스 조회
#[utoipa::path(
    get,
    path = "/api/projects/{project_id}/data-access/matrix",
    responses(
        (status = 200, description = "프로젝트 데이터 접근 매트릭스 조회 성공", body = ProjectDataAccessMatrixResponse),
        (status = 404, description = "프로젝트를 찾을 수 없음"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("project_id" = i32, Path, description = "프로젝트 ID"),
        ("page" = Option<i32>, Query, description = "페이지 번호 (기본값: 1)"),
        ("page_size" = Option<i32>, Query, description = "페이지 크기 (기본값: 20)"),
        ("search" = Option<String>, Query, description = "검색어 (Study UID, Patient ID, Patient Name)"),
        ("status" = Option<String>, Query, description = "상태 필터 (APPROVED, DENIED, PENDING)"),
        ("user_id" = Option<i32>, Query, description = "사용자 ID 필터")
    ),
    tag = "project-data-access"
)]
pub async fn get_project_data_access_matrix(
    path: web::Path<i32>,
    query: web::Query<GetProjectDataListRequest>,
    use_case: web::Data<Arc<ProjectDataAccessUseCase>>,
) -> Result<HttpResponse, actix_web::Error> {
    let project_id = path.into_inner();
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);
    let search = query.search.clone();
    let status = query.status.clone();
    let user_id = query.user_id;

    match use_case.get_project_data_access_matrix(
        project_id,
        page,
        page_size,
        search,
        status,
        user_id,
    ).await {
        Ok(matrix) => Ok(HttpResponse::Ok().json(matrix)),
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// 프로젝트 데이터 생성
#[utoipa::path(
    post,
    path = "/api/projects/{project_id}/data",
    request_body = CreateProjectDataRequest,
    responses(
        (status = 201, description = "프로젝트 데이터 생성 성공", body = CreateProjectDataResponse),
        (status = 400, description = "잘못된 요청"),
        (status = 409, description = "이미 존재하는 Study"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("project_id" = i32, Path, description = "프로젝트 ID")
    ),
    tag = "project-data-access"
)]
pub async fn create_project_data(
    path: web::Path<i32>,
    request: web::Json<CreateProjectDataRequest>,
    use_case: web::Data<Arc<ProjectDataAccessUseCase>>,
) -> Result<HttpResponse, actix_web::Error> {
    let project_id = path.into_inner();

    match use_case.create_project_data(project_id, request.into_inner()).await {
        Ok(response) => Ok(HttpResponse::Created().json(response)),
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// 개별 접근 권한 수정
#[utoipa::path(
    put,
    path = "/api/projects/{project_id}/data/{data_id}/access/{user_id}",
    request_body = UpdateDataAccessRequest,
    responses(
        (status = 200, description = "접근 권한 수정 성공", body = UpdateDataAccessResponse),
        (status = 400, description = "잘못된 요청"),
        (status = 404, description = "데이터 또는 사용자를 찾을 수 없음"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("project_id" = i32, Path, description = "프로젝트 ID"),
        ("data_id" = i32, Path, description = "데이터 ID"),
        ("user_id" = i32, Path, description = "사용자 ID")
    ),
    tag = "project-data-access"
)]
pub async fn update_data_access(
    path: web::Path<(i32, i32, i32)>,
    request: web::Json<UpdateDataAccessRequest>,
    use_case: web::Data<Arc<ProjectDataAccessUseCase>>,
) -> Result<HttpResponse, actix_web::Error> {
    let (project_id, data_id, user_id) = path.into_inner();

    match use_case.update_data_access(data_id, user_id, request.into_inner()).await {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// 일괄 접근 권한 수정
#[utoipa::path(
    put,
    path = "/api/projects/{project_id}/data/{data_id}/access/batch",
    request_body = BatchUpdateDataAccessRequest,
    responses(
        (status = 200, description = "일괄 접근 권한 수정 성공", body = BatchUpdateDataAccessResponse),
        (status = 400, description = "잘못된 요청"),
        (status = 404, description = "데이터를 찾을 수 없음"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("project_id" = i32, Path, description = "프로젝트 ID"),
        ("data_id" = i32, Path, description = "데이터 ID")
    ),
    tag = "project-data-access"
)]
pub async fn batch_update_data_access(
    path: web::Path<(i32, i32)>,
    request: web::Json<BatchUpdateDataAccessRequest>,
    use_case: web::Data<Arc<ProjectDataAccessUseCase>>,
) -> Result<HttpResponse, actix_web::Error> {
    let (project_id, data_id) = path.into_inner();

    match use_case.batch_update_data_access(data_id, request.into_inner()).await {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// 접근 요청
#[utoipa::path(
    post,
    path = "/api/projects/{project_id}/data/{data_id}/access/request",
    responses(
        (status = 200, description = "접근 요청 성공", body = RequestDataAccessResponse),
        (status = 400, description = "잘못된 요청"),
        (status = 404, description = "데이터를 찾을 수 없음"),
        (status = 409, description = "이미 접근 요청이 존재함"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("project_id" = i32, Path, description = "프로젝트 ID"),
        ("data_id" = i32, Path, description = "데이터 ID")
    ),
    tag = "project-data-access"
)]
pub async fn request_data_access(
    path: web::Path<(i32, i32)>,
    use_case: web::Data<Arc<ProjectDataAccessUseCase>>,
) -> Result<HttpResponse, actix_web::Error> {
    let (project_id, data_id) = path.into_inner();
    // TODO: Get user_id from authentication context
    let user_id = 1; // Mock user ID

    match use_case.request_data_access(data_id, user_id).await {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// 상태별 접근 권한 조회
#[utoipa::path(
    get,
    path = "/api/data-access/status/{status}",
    responses(
        (status = 200, description = "상태별 접근 권한 조회 성공", body = Vec<DataAccessInfo>),
        (status = 400, description = "잘못된 상태 값"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("status" = String, Path, description = "접근 상태 (APPROVED, DENIED, PENDING)"),
        ("page" = Option<i32>, Query, description = "페이지 번호 (기본값: 1)"),
        ("page_size" = Option<i32>, Query, description = "페이지 크기 (기본값: 20)")
    ),
    tag = "project-data-access"
)]
pub async fn get_access_by_status(
    path: web::Path<String>,
    query: web::Query<GetProjectDataListRequest>,
    use_case: web::Data<Arc<ProjectDataAccessUseCase>>,
) -> Result<HttpResponse, actix_web::Error> {
    let status = path.into_inner();
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);

    match use_case.get_access_by_status(status, page, page_size).await {
        Ok(access_list) => Ok(HttpResponse::Ok().json(access_list)),
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// 사용자별 접근 권한 조회
#[utoipa::path(
    get,
    path = "/api/users/{user_id}/data-access",
    responses(
        (status = 200, description = "사용자별 접근 권한 조회 성공", body = Vec<DataAccessInfo>),
        (status = 404, description = "사용자를 찾을 수 없음"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("user_id" = i32, Path, description = "사용자 ID"),
        ("page" = Option<i32>, Query, description = "페이지 번호 (기본값: 1)"),
        ("page_size" = Option<i32>, Query, description = "페이지 크기 (기본값: 20)")
    ),
    tag = "project-data-access"
)]
pub async fn get_user_access_list(
    path: web::Path<i32>,
    query: web::Query<GetProjectDataListRequest>,
    use_case: web::Data<Arc<ProjectDataAccessUseCase>>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = path.into_inner();
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);

    match use_case.get_user_access_list(user_id, page, page_size).await {
        Ok(access_list) => Ok(HttpResponse::Ok().json(access_list)),
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// 라우트 설정
pub fn configure_routes(cfg: &mut web::ServiceConfig, use_case: Arc<ProjectDataAccessUseCase>) {
    let use_case = web::Data::new(use_case);
    cfg.service(
        web::scope("/projects/{project_id}")
            .app_data(use_case.clone())
            .route("/data-access/matrix", web::get().to(get_project_data_access_matrix))
            .route("/data", web::post().to(create_project_data))
            .route("/data/{data_id}/access/{user_id}", web::put().to(update_data_access))
            .route("/data/{data_id}/access/batch", web::put().to(batch_update_data_access))
            .route("/data/{data_id}/access/request", web::post().to(request_data_access))
    )
    .service(
        web::scope("/data-access")
            .app_data(use_case.clone())
            .route("/status/{status}", web::get().to(get_access_by_status))
    )
    .service(
        web::scope("/users/{user_id}")
            .app_data(use_case)
            .route("/data-access", web::get().to(get_user_access_list))
    );
}
