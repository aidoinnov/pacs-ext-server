#![allow(dead_code, unused_imports, unused_variables)]
use actix_web::{web, HttpResponse, Responder};
use serde_json::json;
use std::sync::Arc;

use crate::application::dto::project_dto::{
    CreateProjectRequest, ProjectListQuery, ProjectListResponse, ProjectMetaResponse,
    ProjectResponse, UpdateProjectRequest,
};
use crate::application::use_cases::project_use_case::ProjectUseCase;
use crate::domain::services::project_service::ProjectService;

pub struct ProjectController<P: ProjectService> {
    project_use_case: Arc<ProjectUseCase<P>>,
}

impl<P: ProjectService> ProjectController<P> {
    pub fn new(project_use_case: Arc<ProjectUseCase<P>>) -> Self {
        Self { project_use_case }
    }
}

#[utoipa::path(
    post,
    path = "/api/projects",
    tag = "projects",
    request_body = CreateProjectRequest,
    responses(
        (status = 201, description = "Project created successfully", body = ProjectResponse),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn create_project<P: ProjectService>(
    project_use_case: web::Data<Arc<ProjectUseCase<P>>>,
    req: web::Json<CreateProjectRequest>,
) -> impl Responder {
    match project_use_case.create_project(req.into_inner()).await {
        Ok(project) => HttpResponse::Created().json(project),
        Err(e) => HttpResponse::BadRequest().json(json!({
            "error": format!("Failed to create project: {}", e)
        })),
    }
}

#[utoipa::path(
    get,
    path = "/api/projects/{project_id}",
    tag = "projects",
    params(
        ("project_id" = i32, Path, description = "Project ID")
    ),
    responses(
        (status = 200, description = "Project retrieved successfully", body = ProjectResponse),
        (status = 404, description = "Project not found"),
    )
)]
pub async fn get_project<P: ProjectService>(
    project_use_case: web::Data<Arc<ProjectUseCase<P>>>,
    project_id: web::Path<i32>,
) -> impl Responder {
    match project_use_case.get_project(*project_id).await {
        Ok(project) => HttpResponse::Ok().json(project),
        Err(e) => HttpResponse::NotFound().json(json!({
            "error": format!("Project not found: {}", e)
        })),
    }
}

#[utoipa::path(
    get,
    path = "/api/projects",
    tag = "projects",
    params(
        ("page" = Option<i32>, Query, description = "Page number (default: 1)"),
        ("page_size" = Option<i32>, Query, description = "Page size (default: 20)"),
        ("sort_by" = Option<String>, Query, description = "Sort field (created_at, name, start_date)"),
        ("sort_order" = Option<String>, Query, description = "Sort order (asc, desc)"),
        ("status" = Option<String>, Query, description = "Filter by status"),
        ("sponsor" = Option<String>, Query, description = "Filter by sponsor"),
        ("start_date_from" = Option<String>, Query, description = "Start date from (YYYY-MM-DD)"),
        ("start_date_to" = Option<String>, Query, description = "Start date to (YYYY-MM-DD)"),
    ),
    responses(
        (status = 200, description = "Projects retrieved successfully", body = ProjectListResponse),
    )
)]
pub async fn list_projects<P: ProjectService>(
    project_use_case: web::Data<Arc<ProjectUseCase<P>>>,
    query: web::Query<ProjectListQuery>,
) -> impl Responder {
    match project_use_case.get_all_projects(query.into_inner()).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to list projects: {}", e)
        })),
    }
}

#[utoipa::path(
    get,
    path = "/api/projects/active",
    tag = "projects",
    params(
        ("page" = Option<i32>, Query, description = "Page number (default: 1)"),
        ("page_size" = Option<i32>, Query, description = "Page size (default: 20)"),
        ("sort_by" = Option<String>, Query, description = "Sort field (created_at, name, start_date)"),
        ("sort_order" = Option<String>, Query, description = "Sort order (asc, desc)"),
    ),
    responses(
        (status = 200, description = "Active projects retrieved successfully", body = ProjectListResponse),
    )
)]
pub async fn get_active_projects<P: ProjectService>(
    project_use_case: web::Data<Arc<ProjectUseCase<P>>>,
    query: web::Query<ProjectListQuery>,
) -> impl Responder {
    let q = query.into_inner();
    let page = q.page.unwrap_or(1);
    let page_size = q.page_size.unwrap_or(20);
    let sort_by = q.sort_by.as_deref().unwrap_or("created_at");
    let sort_order = q.sort_order.as_deref().unwrap_or("desc");

    match project_use_case
        .get_active_projects(page, page_size, sort_by, sort_order)
        .await
    {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to get active projects: {}", e)
        })),
    }
}

#[utoipa::path(
    put,
    path = "/api/projects/{project_id}",
    tag = "projects",
    params(
        ("project_id" = i32, Path, description = "Project ID")
    ),
    request_body = UpdateProjectRequest,
    responses(
        (status = 200, description = "Project updated successfully", body = ProjectResponse),
        (status = 400, description = "Invalid request"),
        (status = 404, description = "Project not found"),
    )
)]
pub async fn update_project<P: ProjectService>(
    project_use_case: web::Data<Arc<ProjectUseCase<P>>>,
    project_id: web::Path<i32>,
    req: web::Json<UpdateProjectRequest>,
) -> impl Responder {
    match project_use_case
        .update_project(*project_id, req.into_inner())
        .await
    {
        Ok(project) => HttpResponse::Ok().json(project),
        Err(e) => HttpResponse::BadRequest().json(json!({
            "error": format!("Failed to update project: {}", e)
        })),
    }
}

#[utoipa::path(
    delete,
    path = "/api/projects/{project_id}",
    tag = "projects",
    params(
        ("project_id" = i32, Path, description = "Project ID to delete")
    ),
    responses(
        (status = 204, description = "Project deleted successfully"),
        (status = 404, description = "Project not found"),
        (status = 500, description = "Internal server error"),
    )
)]
pub async fn delete_project<P: ProjectService>(
    project_use_case: web::Data<Arc<ProjectUseCase<P>>>,
    project_id: web::Path<i32>,
) -> impl Responder {
    match project_use_case.delete_project(*project_id).await {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(e) => HttpResponse::NotFound().json(json!({
            "error": format!("Failed to delete project: {}", e)
        })),
    }
}

#[utoipa::path(
    get,
    path = "/api/projects/meta",
    tag = "projects",
    responses(
        (status = 200, description = "Project metadata retrieved successfully", body = ProjectMetaResponse),
    )
)]
pub async fn get_project_metadata<P: ProjectService>(
    project_use_case: web::Data<Arc<ProjectUseCase<P>>>,
) -> impl Responder {
    let metadata = project_use_case.get_project_metadata();
    HttpResponse::Ok().json(metadata)
}

pub fn configure_routes<P: ProjectService + 'static>(
    cfg: &mut web::ServiceConfig,
    project_use_case: Arc<ProjectUseCase<P>>,
    project_data_access_use_case: Arc<crate::application::use_cases::ProjectDataAccessUseCase>,
) {
    use crate::presentation::controllers::project_data_access_controller::{
        assign_series_to_project, assign_study_to_project, unassign_series_from_project,
        unassign_study_from_project,
    };

    cfg.app_data(web::Data::new(project_use_case))
        .app_data(web::Data::new(project_data_access_use_case))
        .route("/projects", web::post().to(create_project::<P>))
        .route("/projects", web::get().to(list_projects::<P>))
        // 중요: 구체적인 경로를 먼저 등록해야 {project_id}와 충돌하지 않음
        .route("/projects/meta", web::get().to(get_project_metadata::<P>))
        .route("/projects/active", web::get().to(get_active_projects::<P>))
        // 데이터 할당 API (구체적인 경로)
        .route(
            "/projects/{project_id}/series/assign",
            web::post().to(assign_series_to_project),
        )
        .route(
            "/projects/{project_id}/studies/assign",
            web::post().to(assign_study_to_project),
        )
        // 데이터 할당 해제 API (구체적인 경로)
        .route(
            "/projects/{project_id}/series/{series_id}/unassign",
            web::delete().to(unassign_series_from_project),
        )
        .route(
            "/projects/{project_id}/studies/{study_id}/unassign",
            web::delete().to(unassign_study_from_project),
        )
        // 동적 경로({project_id})는 마지막에 등록
        .route("/projects/{project_id}", web::get().to(get_project::<P>))
        .route("/projects/{project_id}", web::put().to(update_project::<P>))
        .route(
            "/projects/{project_id}",
            web::delete().to(delete_project::<P>),
        );
}
