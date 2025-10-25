use actix_web::{web, HttpResponse, Result};
use std::sync::Arc;
use crate::application::use_cases::RoleCapabilityMatrixUseCase;
use crate::application::dto::role_capability_matrix_dto::*;
use crate::domain::ServiceError;

/// 전역 Role-Capability 매트릭스 조회 (페이지네이션 및 검색 포함)
#[utoipa::path(
    get,
    path = "/api/roles/global/capabilities/matrix",
    params(
        ("page" = Option<i32>, Query, description = "페이지 번호 (기본값: 1)"),
        ("size" = Option<i32>, Query, description = "페이지 크기 (기본값: 10, 최대: 100)"),
        ("search" = Option<String>, Query, description = "역할 이름 또는 설명 검색"),
        ("scope" = Option<String>, Query, description = "역할 범위 필터 (GLOBAL, PROJECT)")
    ),
    responses(
        (status = 200, description = "Success", body = RoleCapabilityMatrixResponse),
        (status = 400, description = "Bad Request"),
        (status = 500, description = "Internal Server Error")
    ),
    tag = "Role-Capability Management"
)]
pub async fn get_global_matrix_paginated(
    query: web::Query<RoleCapabilityMatrixQuery>,
    use_case: web::Data<Arc<RoleCapabilityMatrixUseCase>>,
) -> Result<HttpResponse, actix_web::Error> {
    // 파라미터 검증 및 기본값 설정
    let page = query.page.unwrap_or(1).max(1);
    let size = query.size.unwrap_or(10).min(100).max(1);
    
    match use_case.get_global_matrix_paginated(
        page,
        size,
        query.search.clone(),
        query.scope.clone(),
    ).await {
        Ok(matrix) => Ok(HttpResponse::Ok().json(matrix)),
        Err(ServiceError::DatabaseError(msg)) => {
            tracing::error!("Database error in get_global_matrix_paginated: {}", msg);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Database error",
                "message": msg
            })))
        }
        Err(e) => {
            tracing::error!("Error in get_global_matrix_paginated: {:?}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Internal server error",
                "message": e.to_string()
            })))
        }
    }
}

/// 전역 Role-Capability 매트릭스 조회 (기존 - 하위 호환성)
#[utoipa::path(
    get,
    path = "/api/roles/global/capabilities/matrix/all",
    responses(
        (status = 200, description = "Success", body = RoleCapabilityMatrixResponse),
        (status = 500, description = "Internal Server Error")
    ),
    tag = "Role-Capability Management"
)]
pub async fn get_global_matrix(
    use_case: web::Data<Arc<RoleCapabilityMatrixUseCase>>,
) -> Result<HttpResponse, actix_web::Error> {
    match use_case.get_global_matrix().await {
        Ok(matrix) => Ok(HttpResponse::Ok().json(matrix)),
        Err(ServiceError::DatabaseError(msg)) => {
            tracing::error!("Database error in get_global_matrix: {}", msg);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Database error",
                "message": msg
            })))
        }
        Err(e) => {
            tracing::error!("Error in get_global_matrix: {:?}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Internal server error",
                "message": e.to_string()
            })))
        }
    }
}

/// 프로젝트별 Role-Capability 매트릭스 조회
#[utoipa::path(
    get,
    path = "/api/projects/{project_id}/roles/capabilities/matrix",
    responses(
        (status = 200, description = "Success", body = RoleCapabilityMatrixResponse),
        (status = 500, description = "Internal Server Error")
    ),
    tag = "Role-Capability Management"
)]
pub async fn get_project_matrix(
    path: web::Path<i32>,
    use_case: web::Data<Arc<RoleCapabilityMatrixUseCase>>,
) -> Result<HttpResponse, actix_web::Error> {
    let project_id = path.into_inner();
    
    match use_case.get_project_matrix(project_id).await {
        Ok(matrix) => Ok(HttpResponse::Ok().json(matrix)),
        Err(ServiceError::DatabaseError(msg)) => {
            tracing::error!("Database error in get_project_matrix: {}", msg);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Database error",
                "message": msg
            })))
        }
        Err(e) => {
            tracing::error!("Error in get_project_matrix: {:?}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Internal server error",
                "message": e.to_string()
            })))
        }
    }
}

/// Capability 상세 조회 (매핑된 Permission 포함)
#[utoipa::path(
    get,
    path = "/api/capabilities/{capability_id}",
    responses(
        (status = 200, description = "Success", body = CapabilityDetailResponse),
        (status = 404, description = "Capability not found"),
        (status = 500, description = "Internal Server Error")
    ),
    tag = "Role-Capability Management"
)]
pub async fn get_capability_detail(
    path: web::Path<i32>,
    use_case: web::Data<Arc<RoleCapabilityMatrixUseCase>>,
) -> Result<HttpResponse, actix_web::Error> {
    let capability_id = path.into_inner();
    
    match use_case.get_capability_detail(capability_id).await {
        Ok(detail) => Ok(HttpResponse::Ok().json(detail)),
        Err(ServiceError::NotFound(msg)) => {
            tracing::warn!("Capability not found: {}", msg);
            Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "Not found",
                "message": msg
            })))
        }
        Err(ServiceError::DatabaseError(msg)) => {
            tracing::error!("Database error in get_capability_detail: {}", msg);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Database error",
                "message": msg
            })))
        }
        Err(e) => {
            tracing::error!("Error in get_capability_detail: {:?}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Internal server error",
                "message": e.to_string()
            })))
        }
    }
}

/// Role에 Capability 할당/제거
#[utoipa::path(
    put,
    path = "/api/roles/{role_id}/capabilities/{capability_id}",
    request_body = UpdateRoleCapabilityRequest,
    responses(
        (status = 200, description = "Success"),
        (status = 400, description = "Bad Request"),
        (status = 500, description = "Internal Server Error")
    ),
    tag = "Role-Capability Management"
)]
pub async fn update_capability_assignment(
    path: web::Path<(i32, i32)>,
    request: web::Json<UpdateRoleCapabilityRequest>,
    use_case: web::Data<Arc<RoleCapabilityMatrixUseCase>>,
) -> Result<HttpResponse, actix_web::Error> {
    let (role_id, capability_id) = path.into_inner();
    let assign = request.into_inner().assign;
    
    match use_case.update_capability_assignment(role_id, capability_id, assign).await {
        Ok(()) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "message": if assign { "Capability assigned successfully" } else { "Capability removed successfully" }
        }))),
        Err(ServiceError::ValidationError(msg)) => {
            tracing::warn!("Validation error in update_capability_assignment: {}", msg);
            Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Validation error",
                "message": msg
            })))
        }
        Err(ServiceError::DatabaseError(msg)) => {
            tracing::error!("Database error in update_capability_assignment: {}", msg);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Database error",
                "message": msg
            })))
        }
        Err(e) => {
            tracing::error!("Error in update_capability_assignment: {:?}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Internal server error",
                "message": e.to_string()
            })))
        }
    }
}

/// 모든 Capability 목록 조회
#[utoipa::path(
    get,
    path = "/api/capabilities",
    responses(
        (status = 200, description = "Success", body = Vec<CapabilityInfo>),
        (status = 500, description = "Internal Server Error")
    ),
    tag = "Role-Capability Management"
)]
pub async fn get_all_capabilities(
    use_case: web::Data<Arc<RoleCapabilityMatrixUseCase>>,
) -> Result<HttpResponse, actix_web::Error> {
    match use_case.get_all_capabilities().await {
        Ok(capabilities) => Ok(HttpResponse::Ok().json(capabilities)),
        Err(ServiceError::DatabaseError(msg)) => {
            tracing::error!("Database error in get_all_capabilities: {}", msg);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Database error",
                "message": msg
            })))
        }
        Err(e) => {
            tracing::error!("Error in get_all_capabilities: {:?}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Internal server error",
                "message": e.to_string()
            })))
        }
    }
}

/// 카테고리별 Capability 목록 조회
#[utoipa::path(
    get,
    path = "/api/capabilities/category/{category}",
    responses(
        (status = 200, description = "Success", body = Vec<CapabilityInfo>),
        (status = 500, description = "Internal Server Error")
    ),
    tag = "Role-Capability Management"
)]
pub async fn get_capabilities_by_category(
    path: web::Path<String>,
    use_case: web::Data<Arc<RoleCapabilityMatrixUseCase>>,
) -> Result<HttpResponse, actix_web::Error> {
    let category = path.into_inner();
    
    match use_case.get_capabilities_by_category(&category).await {
        Ok(capabilities) => Ok(HttpResponse::Ok().json(capabilities)),
        Err(ServiceError::DatabaseError(msg)) => {
            tracing::error!("Database error in get_capabilities_by_category: {}", msg);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Database error",
                "message": msg
            })))
        }
        Err(e) => {
            tracing::error!("Error in get_capabilities_by_category: {:?}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Internal server error",
                "message": e.to_string()
            })))
        }
    }
}

/// 라우팅 설정
pub fn configure_routes(cfg: &mut web::ServiceConfig, use_case: Arc<RoleCapabilityMatrixUseCase>) {
    cfg.app_data(web::Data::new(use_case))
        .service(
            web::scope("/roles")
                .route("/global/capabilities/matrix", web::get().to(get_global_matrix_paginated))
                .route("/global/capabilities/matrix/all", web::get().to(get_global_matrix))
                .route("/{role_id}/capabilities/{capability_id}", web::put().to(update_capability_assignment))
        )
        .service(
            web::scope("/projects/{project_id}/roles")
                .route("/capabilities/matrix", web::get().to(get_project_matrix))
        )
        .service(
            web::scope("/capabilities")
                .route("", web::get().to(get_all_capabilities))
                .route("/{capability_id}", web::get().to(get_capability_detail))
                .route("/category/{category}", web::get().to(get_capabilities_by_category))
        );
}
