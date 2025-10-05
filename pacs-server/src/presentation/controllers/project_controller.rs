use actix_web::{web, HttpResponse, Responder};
use serde_json::json;
use std::sync::Arc;

use crate::application::dto::project_dto::CreateProjectRequest;
use crate::application::use_cases::project_use_case::ProjectUseCase;
use crate::domain::services::project_service::ProjectService;

pub struct ProjectController<P: ProjectService> {
    project_use_case: Arc<ProjectUseCase<P>>,
}

impl<P: ProjectService> ProjectController<P> {
    pub fn new(project_use_case: Arc<ProjectUseCase<P>>) -> Self {
        Self { project_use_case }
    }

    pub async fn create_project(
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

    pub async fn get_project(
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

    pub async fn list_projects(
        project_use_case: web::Data<Arc<ProjectUseCase<P>>>,
    ) -> impl Responder {
        match project_use_case.get_all_projects().await {
            Ok(response) => HttpResponse::Ok().json(response),
            Err(e) => HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to list projects: {}", e)
            })),
        }
    }

    pub async fn get_active_projects(
        project_use_case: web::Data<Arc<ProjectUseCase<P>>>,
    ) -> impl Responder {
        match project_use_case.get_active_projects().await {
            Ok(response) => HttpResponse::Ok().json(response),
            Err(e) => HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to get active projects: {}", e)
            })),
        }
    }
}

pub fn configure_routes<P: ProjectService + 'static>(
    cfg: &mut web::ServiceConfig,
    project_use_case: Arc<ProjectUseCase<P>>,
) {
    cfg.app_data(web::Data::new(project_use_case))
        .service(
            web::scope("/projects")
                .route("", web::post().to(ProjectController::<P>::create_project))
                .route("", web::get().to(ProjectController::<P>::list_projects))
                .route("/active", web::get().to(ProjectController::<P>::get_active_projects))
                .route("/{project_id}", web::get().to(ProjectController::<P>::get_project)),
        );
}
