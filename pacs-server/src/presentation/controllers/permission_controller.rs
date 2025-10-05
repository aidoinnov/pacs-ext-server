use actix_web::{web, HttpResponse, Responder};
use serde_json::json;
use std::sync::Arc;

use crate::application::dto::permission_dto::CreateRoleRequest;
use crate::application::use_cases::permission_use_case::PermissionUseCase;
use crate::domain::services::permission_service::PermissionService;

pub struct PermissionController<P: PermissionService> {
    permission_use_case: Arc<PermissionUseCase<P>>,
}

impl<P: PermissionService> PermissionController<P> {
    pub fn new(permission_use_case: Arc<PermissionUseCase<P>>) -> Self {
        Self {
            permission_use_case,
        }
    }

    pub async fn create_role(
        permission_use_case: web::Data<Arc<PermissionUseCase<P>>>,
        req: web::Json<CreateRoleRequest>,
    ) -> impl Responder {
        match permission_use_case.create_role(req.into_inner()).await {
            Ok(role) => HttpResponse::Created().json(role),
            Err(e) => HttpResponse::BadRequest().json(json!({
                "error": format!("Failed to create role: {}", e)
            })),
        }
    }

    pub async fn get_role(
        permission_use_case: web::Data<Arc<PermissionUseCase<P>>>,
        role_id: web::Path<i32>,
    ) -> impl Responder {
        match permission_use_case.get_role(*role_id).await {
            Ok(role) => HttpResponse::Ok().json(role),
            Err(e) => HttpResponse::NotFound().json(json!({
                "error": format!("Role not found: {}", e)
            })),
        }
    }

    pub async fn get_global_roles(
        permission_use_case: web::Data<Arc<PermissionUseCase<P>>>,
    ) -> impl Responder {
        match permission_use_case.get_global_roles().await {
            Ok(roles) => HttpResponse::Ok().json(roles),
            Err(e) => HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to get global roles: {}", e)
            })),
        }
    }

    pub async fn get_project_roles(
        permission_use_case: web::Data<Arc<PermissionUseCase<P>>>,
    ) -> impl Responder {
        match permission_use_case.get_project_roles().await {
            Ok(roles) => HttpResponse::Ok().json(roles),
            Err(e) => HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to get project roles: {}", e)
            })),
        }
    }
}

pub fn configure_routes<P: PermissionService + 'static>(
    cfg: &mut web::ServiceConfig,
    permission_use_case: Arc<PermissionUseCase<P>>,
) {
    cfg.app_data(web::Data::new(permission_use_case))
        .service(
            web::scope("/roles")
                .route("", web::post().to(PermissionController::<P>::create_role))
                .route("/global", web::get().to(PermissionController::<P>::get_global_roles))
                .route("/project", web::get().to(PermissionController::<P>::get_project_roles))
                .route("/{role_id}", web::get().to(PermissionController::<P>::get_role)),
        );
}
