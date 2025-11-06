#![allow(dead_code, unused_imports, unused_variables)]
use actix_web::{web, HttpResponse, Responder};
use serde_json::json;
use std::sync::Arc;

use crate::application::dto::access_control_dto::{CheckPermissionRequest, LogDicomAccessRequest};
use crate::application::use_cases::access_control_use_case::AccessControlUseCase;
use crate::domain::services::access_control_service::AccessControlService;

pub struct AccessControlController<A: AccessControlService> {
    access_control_use_case: Arc<AccessControlUseCase<A>>,
}

impl<A: AccessControlService> AccessControlController<A> {
    pub fn new(access_control_use_case: Arc<AccessControlUseCase<A>>) -> Self {
        Self {
            access_control_use_case,
        }
    }

    pub async fn log_dicom_access(
        access_control_use_case: web::Data<Arc<AccessControlUseCase<A>>>,
        req: web::Json<LogDicomAccessRequest>,
    ) -> impl Responder {
        match access_control_use_case
            .log_dicom_access(req.into_inner())
            .await
        {
            Ok(log) => HttpResponse::Created().json(log),
            Err(e) => HttpResponse::BadRequest().json(json!({
                "error": format!("Failed to log access: {}", e)
            })),
        }
    }

    pub async fn get_user_access_logs(
        access_control_use_case: web::Data<Arc<AccessControlUseCase<A>>>,
        path: web::Path<i32>,
        query: web::Query<std::collections::HashMap<String, String>>,
    ) -> impl Responder {
        let user_id = path.into_inner();
        let limit = query
            .get("limit")
            .and_then(|l| l.parse::<i64>().ok())
            .unwrap_or(100);

        match access_control_use_case
            .get_user_access_logs(user_id, limit)
            .await
        {
            Ok(logs) => HttpResponse::Ok().json(logs),
            Err(e) => HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to get access logs: {}", e)
            })),
        }
    }

    pub async fn get_project_access_logs(
        access_control_use_case: web::Data<Arc<AccessControlUseCase<A>>>,
        path: web::Path<i32>,
        query: web::Query<std::collections::HashMap<String, String>>,
    ) -> impl Responder {
        let project_id = path.into_inner();
        let limit = query
            .get("limit")
            .and_then(|l| l.parse::<i64>().ok())
            .unwrap_or(100);

        match access_control_use_case
            .get_project_access_logs(project_id, limit)
            .await
        {
            Ok(logs) => HttpResponse::Ok().json(logs),
            Err(e) => HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to get access logs: {}", e)
            })),
        }
    }

    pub async fn get_study_access_logs(
        access_control_use_case: web::Data<Arc<AccessControlUseCase<A>>>,
        path: web::Path<String>,
        query: web::Query<std::collections::HashMap<String, String>>,
    ) -> impl Responder {
        let study_uid = path.into_inner();
        let limit = query
            .get("limit")
            .and_then(|l| l.parse::<i64>().ok())
            .unwrap_or(100);

        match access_control_use_case
            .get_study_access_logs(&study_uid, limit)
            .await
        {
            Ok(logs) => HttpResponse::Ok().json(logs),
            Err(e) => HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to get access logs: {}", e)
            })),
        }
    }

    pub async fn check_permission(
        access_control_use_case: web::Data<Arc<AccessControlUseCase<A>>>,
        req: web::Json<CheckPermissionRequest>,
    ) -> impl Responder {
        match access_control_use_case
            .check_permission(req.into_inner())
            .await
        {
            Ok(response) => HttpResponse::Ok().json(response),
            Err(e) => HttpResponse::Forbidden().json(json!({
                "error": format!("Permission check failed: {}", e)
            })),
        }
    }

    pub async fn get_user_permissions(
        access_control_use_case: web::Data<Arc<AccessControlUseCase<A>>>,
        path: web::Path<(i32, i32)>,
    ) -> impl Responder {
        let (user_id, project_id) = path.into_inner();

        match access_control_use_case
            .get_user_permissions(user_id, project_id)
            .await
        {
            Ok(permissions) => HttpResponse::Ok().json(permissions),
            Err(e) => HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to get permissions: {}", e)
            })),
        }
    }

    pub async fn can_access_project(
        access_control_use_case: web::Data<Arc<AccessControlUseCase<A>>>,
        path: web::Path<(i32, i32)>,
    ) -> impl Responder {
        let (user_id, project_id) = path.into_inner();

        match access_control_use_case
            .can_access_project(user_id, project_id)
            .await
        {
            Ok(response) => HttpResponse::Ok().json(response),
            Err(e) => HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to check access: {}", e)
            })),
        }
    }
}

pub fn configure_routes<A: AccessControlService + 'static>(
    cfg: &mut web::ServiceConfig,
    access_control_use_case: Arc<AccessControlUseCase<A>>,
) {
    cfg.app_data(web::Data::new(access_control_use_case))
        .service(
            web::scope("/access-control")
                .route(
                    "/logs",
                    web::post().to(AccessControlController::<A>::log_dicom_access),
                )
                .route(
                    "/logs/user/{user_id}",
                    web::get().to(AccessControlController::<A>::get_user_access_logs),
                )
                .route(
                    "/logs/project/{project_id}",
                    web::get().to(AccessControlController::<A>::get_project_access_logs),
                )
                .route(
                    "/logs/study/{study_uid}",
                    web::get().to(AccessControlController::<A>::get_study_access_logs),
                )
                .route(
                    "/permissions/check",
                    web::post().to(AccessControlController::<A>::check_permission),
                )
                .route(
                    "/permissions/user/{user_id}/project/{project_id}",
                    web::get().to(AccessControlController::<A>::get_user_permissions),
                )
                .route(
                    "/access/user/{user_id}/project/{project_id}",
                    web::get().to(AccessControlController::<A>::can_access_project),
                ),
        );
}
