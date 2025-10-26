use actix_web::{web, HttpResponse, Responder, Result};
use serde_json::json;
use std::sync::Arc;
use utoipa::OpenApi;

use crate::application::use_cases::{project_user_use_case::ProjectUserUseCase, ProjectDataAccessUseCase};
use crate::application::dto::project_user_dto::{
    AssignRoleRequest, BatchAssignRolesRequest, RoleAssignmentResponse, BatchRoleAssignmentResponse,
    AddMemberRequest
};
use crate::application::dto::permission_dto::PaginationQuery;
use crate::application::dto::project_data_access_dto::*;
use crate::domain::services::{ProjectService, UserService, ProjectDataService};
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

/// 프로젝트 멤버 목록 조회 (역할 정보 포함, 페이지네이션)
#[utoipa::path(
    get,
    path = "/api/projects/{project_id}/users",
    params(
        ("project_id" = i32, Path, description = "Project ID"),
        ("page" = Option<i32>, Query, description = "Page number (default: 1)"),
        ("page_size" = Option<i32>, Query, description = "Page size (default: 20, max: 100)")
    ),
    responses(
        (status = 200, description = "Project members retrieved successfully"),
        (status = 404, description = "Project not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "project-users"
)]
pub async fn get_project_members<P, U, D>(
    path: web::Path<i32>,
    query: web::Query<PaginationQuery>,
    use_case: web::Data<Arc<ProjectUserUseCase<P, U, D>>>,
) -> impl Responder
where
    P: ProjectService,
    U: UserService,
    D: ProjectDataService,
{
    let project_id = path.into_inner();
    
    match use_case
        .get_project_members_with_roles(project_id, query.page, query.page_size)
        .await
    {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to get project members: {}", e)
        })),
    }
}

/// 사용자의 프로젝트 목록 조회 (역할 정보 포함, 페이지네이션)
#[utoipa::path(
    get,
    path = "/api/users/{user_id}/projects",
    params(
        ("user_id" = i32, Path, description = "User ID"),
        ("page" = Option<i32>, Query, description = "Page number (default: 1)"),
        ("page_size" = Option<i32>, Query, description = "Page size (default: 20, max: 100)")
    ),
    responses(
        (status = 200, description = "User projects retrieved successfully"),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "project-users"
)]
pub async fn get_user_projects<P, U, D>(
    path: web::Path<i32>,
    query: web::Query<PaginationQuery>,
    use_case: web::Data<Arc<ProjectUserUseCase<P, U, D>>>,
) -> impl Responder
where
    P: ProjectService,
    U: UserService,
    D: ProjectDataService,
{
    let user_id = path.into_inner();
    
    match use_case
        .get_user_projects_with_roles(user_id, query.page, query.page_size)
        .await
    {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to get user projects: {}", e)
        })),
    }
}

/// 프로젝트 내 사용자에게 역할 할당
#[utoipa::path(
    put,
    path = "/api/projects/{project_id}/users/{user_id}/role",
    params(
        ("project_id" = i32, Path, description = "Project ID"),
        ("user_id" = i32, Path, description = "User ID")
    ),
    request_body = AssignRoleRequest,
    responses(
        (status = 200, description = "Role assigned successfully", body = RoleAssignmentResponse),
        (status = 404, description = "Project, user, or role not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "project-users"
)]
pub async fn assign_user_role<P, U, D>(
    path: web::Path<(i32, i32)>,
    req: web::Json<AssignRoleRequest>,
    use_case: web::Data<Arc<ProjectUserUseCase<P, U, D>>>,
) -> impl Responder
where
    P: ProjectService,
    U: UserService,
    D: ProjectDataService,
{
    let (project_id, user_id) = path.into_inner();
    
    match use_case
        .assign_role_to_user(project_id, user_id, req.role_id)
        .await
    {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to assign role: {}", e)
        })),
    }
}

/// 프로젝트 내 여러 사용자에게 역할 일괄 할당
#[utoipa::path(
    post,
    path = "/api/projects/{project_id}/users/roles",
    params(
        ("project_id" = i32, Path, description = "Project ID")
    ),
    request_body = BatchAssignRolesRequest,
    responses(
        (status = 200, description = "Roles assigned successfully", body = BatchRoleAssignmentResponse),
        (status = 404, description = "Project not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "project-users"
)]
pub async fn batch_assign_roles<P, U, D>(
    path: web::Path<i32>,
    req: web::Json<BatchAssignRolesRequest>,
    use_case: web::Data<Arc<ProjectUserUseCase<P, U, D>>>,
) -> impl Responder
where
    P: ProjectService,
    U: UserService,
    D: ProjectDataService,
{
    let project_id = path.into_inner();
    
    // DTO를 (user_id, role_id) 튜플로 변환
    let assignments: Vec<(i32, i32)> = req.assignments
        .iter()
        .map(|assignment| (assignment.user_id, assignment.role_id))
        .collect();
    
    match use_case
        .batch_assign_roles(project_id, assignments)
        .await
    {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to batch assign roles: {}", e)
        })),
    }
}

/// 프로젝트 내 사용자의 역할 제거
#[utoipa::path(
    delete,
    path = "/api/projects/{project_id}/users/{user_id}/role",
    params(
        ("project_id" = i32, Path, description = "Project ID"),
        ("user_id" = i32, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User role removed successfully", body = RoleAssignmentResponse),
        (status = 404, description = "Project or user not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "project-users"
)]
pub async fn remove_user_role<P, U, D>(
    path: web::Path<(i32, i32)>,
    use_case: web::Data<Arc<ProjectUserUseCase<P, U, D>>>,
) -> impl Responder
where
    P: ProjectService,
    U: UserService,
    D: ProjectDataService,
{
    let (project_id, user_id) = path.into_inner();
    
    match use_case
        .remove_user_role(project_id, user_id)
        .await
    {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to remove user role: {}", e)
        })),
    }
}

/// 프로젝트에 멤버 추가
#[utoipa::path(
    post,
    path = "/api/projects/{project_id}/members",
    params(
        ("project_id" = i32, Path, description = "Project ID")
    ),
    request_body = AddMemberRequest,
    responses(
        (status = 200, description = "Member added successfully"),
        (status = 400, description = "Bad request"),
        (status = 404, description = "Project or user not found"),
        (status = 409, description = "User is already a member"),
        (status = 500, description = "Internal server error")
    ),
    tag = "project-members"
)]
pub async fn add_project_member<P, U, D>(
    path: web::Path<i32>,
    request: web::Json<AddMemberRequest>,
    use_case: web::Data<Arc<ProjectUserUseCase<P, U, D>>>,
) -> impl Responder
where
    P: ProjectService,
    U: UserService,
    D: ProjectDataService,
{
    let project_id = path.into_inner();
    
    match use_case
        .add_member_to_project(project_id, request.into_inner())
        .await
    {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            let status = match e.to_string().as_str() {
                s if s.contains("not found") => actix_web::http::StatusCode::NOT_FOUND,
                s if s.contains("already") => actix_web::http::StatusCode::CONFLICT,
                _ => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            };
            HttpResponse::build(status).json(json!({
                "error": format!("Failed to add member: {}", e)
            }))
        }
    }
}

/// 프로젝트에서 멤버 제거
#[utoipa::path(
    delete,
    path = "/api/projects/{project_id}/members/{user_id}",
    params(
        ("project_id" = i32, Path, description = "Project ID"),
        ("user_id" = i32, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "Member removed successfully"),
        (status = 404, description = "Project or user not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "project-members"
)]
pub async fn remove_project_member<P, U, D>(
    path: web::Path<(i32, i32)>,
    use_case: web::Data<Arc<ProjectUserUseCase<P, U, D>>>,
) -> impl Responder
where
    P: ProjectService,
    U: UserService,
    D: ProjectDataService,
{
    let (project_id, user_id) = path.into_inner();
    
    match use_case
        .remove_member_from_project(project_id, user_id)
        .await
    {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            let status = match e.to_string().as_str() {
                s if s.contains("not found") => actix_web::http::StatusCode::NOT_FOUND,
                _ => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            };
            HttpResponse::build(status).json(json!({
                "error": format!("Failed to remove member: {}", e)
            }))
        }
    }
}

/// 프로젝트 멤버십 확인
#[utoipa::path(
    get,
    path = "/api/projects/{project_id}/members/{user_id}/membership",
    params(
        ("project_id" = i32, Path, description = "Project ID"),
        ("user_id" = i32, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "Membership status retrieved successfully"),
        (status = 404, description = "Project not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "project-members"
)]
pub async fn check_project_membership<P, U, D>(
    path: web::Path<(i32, i32)>,
    use_case: web::Data<Arc<ProjectUserUseCase<P, U, D>>>,
) -> impl Responder
where
    P: ProjectService,
    U: UserService,
    D: ProjectDataService,
{
    let (project_id, user_id) = path.into_inner();
    
    match use_case
        .check_project_membership(project_id, user_id)
        .await
    {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            let status = match e.to_string().as_str() {
                s if s.contains("not found") => actix_web::http::StatusCode::NOT_FOUND,
                _ => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            };
            HttpResponse::build(status).json(json!({
                "error": format!("Failed to check membership: {}", e)
            }))
        }
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
    let user_id = 1; // TODO: Get user_id from authentication context

    match use_case.request_data_access(data_id, user_id).await {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// 라우팅 설정
pub fn configure_routes<P, U, D>(
    cfg: &mut web::ServiceConfig,
    project_user_use_case: Arc<ProjectUserUseCase<P, U, D>>,
    project_data_access_use_case: Arc<ProjectDataAccessUseCase>,
) where
    P: ProjectService + 'static,
    U: UserService + 'static,
    D: ProjectDataService + 'static,
{
    cfg.app_data(web::Data::new(project_user_use_case))
        .app_data(web::Data::new(project_data_access_use_case))
        .service(
            web::scope("/projects")
                .route("/{project_id}/users", web::get().to(get_project_members::<P, U, D>))
                .route("/{project_id}/users/{user_id}/role", web::put().to(assign_user_role::<P, U, D>))
                .route("/{project_id}/users/{user_id}/role", web::delete().to(remove_user_role::<P, U, D>))
                .route("/{project_id}/users/roles", web::post().to(batch_assign_roles::<P, U, D>))
                .route("/{project_id}/members", web::post().to(add_project_member::<P, U, D>))
                .route("/{project_id}/members/{user_id}", web::delete().to(remove_project_member::<P, U, D>))
                .route("/{project_id}/members/{user_id}/membership", web::get().to(check_project_membership::<P, U, D>))
                // Data access routes
                .route("/{project_id}/data-access/matrix", web::get().to(get_project_data_access_matrix))
                .route("/{project_id}/data", web::post().to(create_project_data))
                .route("/{project_id}/data/{data_id}/access/{user_id}", web::put().to(update_data_access))
                .route("/{project_id}/data/{data_id}/access/batch", web::put().to(batch_update_data_access))
                .route("/{project_id}/data/{data_id}/access/request", web::post().to(request_data_access))
        )
        .route("/users/{user_id}/projects", web::get().to(get_user_projects::<P, U, D>));
}
