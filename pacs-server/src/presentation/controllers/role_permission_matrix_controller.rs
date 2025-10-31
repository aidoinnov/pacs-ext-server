use actix_web::{web, HttpResponse, Result};
use std::sync::Arc;

use crate::application::dto::role_permission_matrix_dto::*;
use crate::application::use_cases::RolePermissionMatrixUseCase;
use crate::domain::ServiceError;

/// ServiceError를 HttpResponse로 변환하는 헬퍼 함수
fn handle_service_error(error: ServiceError) -> HttpResponse {
    match error {
        ServiceError::NotFound(msg) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Not Found",
            "message": msg
        })),
        ServiceError::ValidationError(msg) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Validation Error",
            "message": msg
        })),
        ServiceError::Unauthorized(msg) => HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Unauthorized",
            "message": msg
        })),
        ServiceError::AlreadyExists(msg) => HttpResponse::Conflict().json(serde_json::json!({
            "error": "Already Exists",
            "message": msg
        })),
        ServiceError::DatabaseError(msg) => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Database Error",
                "message": msg
            }))
        }
        _ => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Internal Server Error",
            "message": "An unexpected error occurred"
        })),
    }
}

/// 글로벌 역할-권한 매트릭스 조회
#[utoipa::path(
    get,
    path = "/api/roles/global/permissions/matrix",
    responses(
        (status = 200, description = "글로벌 역할-권한 매트릭스 조회 성공", body = RolePermissionMatrixResponse),
        (status = 500, description = "서버 내부 오류")
    ),
    tag = "role-permission-matrix"
)]
pub async fn get_global_matrix(
    use_case: web::Data<Arc<RolePermissionMatrixUseCase>>,
) -> Result<HttpResponse, actix_web::Error> {
    match use_case.get_global_matrix().await {
        Ok(matrix) => Ok(HttpResponse::Ok().json(matrix)),
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// 프로젝트별 역할-권한 매트릭스 조회
#[utoipa::path(
    get,
    path = "/api/projects/{project_id}/roles/permissions/matrix",
    responses(
        (status = 200, description = "프로젝트별 역할-권한 매트릭스 조회 성공", body = RolePermissionMatrixResponse),
        (status = 404, description = "프로젝트를 찾을 수 없음"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("project_id" = i32, Path, description = "프로젝트 ID")
    ),
    tag = "role-permission-matrix"
)]
pub async fn get_project_matrix(
    path: web::Path<i32>,
    use_case: web::Data<Arc<RolePermissionMatrixUseCase>>,
) -> Result<HttpResponse, actix_web::Error> {
    let project_id = path.into_inner();
    match use_case.get_project_matrix(project_id).await {
        Ok(matrix) => Ok(HttpResponse::Ok().json(matrix)),
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// 글로벌 역할에 권한 할당/제거
#[utoipa::path(
    put,
    path = "/api/roles/{role_id}/permissions/{permission_id}",
    request_body = AssignPermissionRequest,
    responses(
        (status = 200, description = "권한 할당/제거 성공", body = AssignPermissionResponse),
        (status = 404, description = "역할 또는 권한을 찾을 수 없음"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("role_id" = i32, Path, description = "역할 ID"),
        ("permission_id" = i32, Path, description = "권한 ID")
    ),
    tag = "role-permission-matrix"
)]
pub async fn update_global_permission_assignment(
    path: web::Path<(i32, i32)>,
    request: web::Json<AssignPermissionRequest>,
    use_case: web::Data<Arc<RolePermissionMatrixUseCase>>,
) -> Result<HttpResponse, actix_web::Error> {
    let (role_id, permission_id) = path.into_inner();
    let assign = request.into_inner().assign;

    match use_case
        .update_permission_assignment(role_id, permission_id, assign)
        .await
    {
        Ok(_) => {
            let response = AssignPermissionResponse {
                success: true,
                message: if assign {
                    "Permission assigned successfully".to_string()
                } else {
                    "Permission removed successfully".to_string()
                },
            };
            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// 프로젝트별 역할에 권한 할당/제거
#[utoipa::path(
    put,
    path = "/api/projects/{project_id}/roles/{role_id}/permissions/{permission_id}",
    request_body = AssignPermissionRequest,
    responses(
        (status = 200, description = "권한 할당/제거 성공", body = AssignPermissionResponse),
        (status = 404, description = "프로젝트, 역할 또는 권한을 찾을 수 없음"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("project_id" = i32, Path, description = "프로젝트 ID"),
        ("role_id" = i32, Path, description = "역할 ID"),
        ("permission_id" = i32, Path, description = "권한 ID")
    ),
    tag = "role-permission-matrix"
)]
pub async fn update_project_permission_assignment(
    path: web::Path<(i32, i32, i32)>,
    request: web::Json<AssignPermissionRequest>,
    use_case: web::Data<Arc<RolePermissionMatrixUseCase>>,
) -> Result<HttpResponse, actix_web::Error> {
    let (_project_id, role_id, permission_id) = path.into_inner();
    let assign = request.into_inner().assign;

    // 프로젝트별 역할인지 확인 (추가 검증이 필요한 경우)
    // 현재는 단순히 권한 할당/제거만 수행
    match use_case
        .update_permission_assignment(role_id, permission_id, assign)
        .await
    {
        Ok(_) => {
            let response = AssignPermissionResponse {
                success: true,
                message: if assign {
                    "Permission assigned successfully".to_string()
                } else {
                    "Permission removed successfully".to_string()
                },
            };
            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// 라우트 설정
pub fn configure_routes(cfg: &mut web::ServiceConfig, use_case: Arc<RolePermissionMatrixUseCase>) {
    let use_case = web::Data::new(use_case);

    cfg.service(
        web::resource("/roles/global/permissions/matrix").route(web::get().to(get_global_matrix)),
    )
    .service(
        web::resource("/projects/{project_id}/roles/permissions/matrix")
            .route(web::get().to(get_project_matrix)),
    )
    .service(
        web::resource("/roles/{role_id}/permissions/{permission_id}")
            .route(web::put().to(update_global_permission_assignment)),
    )
    .service(
        web::resource("/projects/{project_id}/roles/{role_id}/permissions/{permission_id}")
            .route(web::put().to(update_project_permission_assignment)),
    )
    .app_data(use_case);
}
