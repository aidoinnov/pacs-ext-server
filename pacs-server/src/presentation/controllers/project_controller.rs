use actix_web::{web, HttpResponse, Responder};
use serde_json::json;
use std::sync::Arc;

use crate::application::dto::project_dto::{CreateProjectRequest, ProjectResponse, ProjectListResponse};
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
    responses(
        (status = 200, description = "Projects retrieved successfully", body = ProjectListResponse),
    )
)]
pub async fn list_projects<P: ProjectService>(
    project_use_case: web::Data<Arc<ProjectUseCase<P>>>,
) -> impl Responder {
    match project_use_case.get_all_projects().await {
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
    responses(
        (status = 200, description = "Active projects retrieved successfully", body = ProjectListResponse),
    )
)]
pub async fn get_active_projects<P: ProjectService>(
    project_use_case: web::Data<Arc<ProjectUseCase<P>>>,
) -> impl Responder {
    match project_use_case.get_active_projects().await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to get active projects: {}", e)
        })),
    }
}

pub fn configure_routes<P: ProjectService + 'static>(
    cfg: &mut web::ServiceConfig,
    project_use_case: Arc<ProjectUseCase<P>>,
) {
    cfg.app_data(web::Data::new(project_use_case))
        .service(
            web::scope("/projects")
                .route("", web::post().to(create_project::<P>))
                .route("", web::get().to(list_projects::<P>))
                .route("/active", web::get().to(get_active_projects::<P>))
                .route("/{project_id}", web::get().to(get_project::<P>)),
        );
}
