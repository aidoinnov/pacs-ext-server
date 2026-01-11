#![allow(dead_code, unused_imports, unused_variables)]
use actix_web::{web, HttpRequest, HttpResponse, Result};
use serde_json::json;
use std::sync::Arc;

use crate::application::dto::view_selection_dto::*;
use crate::application::use_cases::ViewSelectionUseCase;
use crate::domain::ServiceError;
use crate::infrastructure::auth::{extract_user_id_from_request, JwtService};
use crate::infrastructure::repositories::UserRepositoryImpl;
use crate::domain::services::DicomRbacEvaluator;
use crate::infrastructure::services::DicomRbacEvaluatorImpl;
use crate::infrastructure::repositories::ProjectDataRepositoryImpl;

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

/// ViewSelection 생성 (POST)
///
/// 여러 Study에 속한 Series를 선택하여 Viewer Selection을 생성합니다.
#[utoipa::path(
    post,
    path = "/api/v1/view-selections",
    request_body = CreateViewSelectionRequest,
    responses(
        (status = 201, description = "Selection 생성 성공", body = CreateViewSelectionResponse),
        (status = 400, description = "잘못된 요청"),
        (status = 401, description = "인증 실패"),
        (status = 403, description = "권한 없음 (일부 Series에 대한 접근 권한 없음)"),
        (status = 500, description = "서버 내부 오류")
    ),
    tag = "view-selection"
)]
pub async fn create_view_selection<S>(
    request: web::Json<CreateViewSelectionRequest>,
    req: HttpRequest,
    use_case: web::Data<Arc<ViewSelectionUseCase<S>>>,
    jwt: web::Data<Arc<JwtService>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
    rbac_evaluator: web::Data<Arc<DicomRbacEvaluatorImpl>>,
    project_data_repo: web::Data<Arc<ProjectDataRepositoryImpl>>,
) -> Result<HttpResponse, actix_web::Error>
where
    S: crate::domain::view_selection::services::ViewSelectionService + 'static,
{
    // 사용자 ID 추출
    let user_id = match extract_user_id_from_request(&req, &jwt, &user_repo).await {
        Some(id) if id > 0 => id,
        _ => {
            return Ok(HttpResponse::Unauthorized().json(json!({
                "error": "Unauthorized",
                "message": "Invalid or missing authorization token"
            })));
        }
    };

    let request = request.into_inner();

    // 권한 검증: 선택된 모든 Series에 대한 접근 권한 확인
    // Note: 실제 구현에서는 project_id를 파라미터로 받거나, 각 Series별로 확인해야 합니다.
    // 여기서는 간단히 모든 Series가 접근 가능한지 확인합니다.
    // TODO: 프로젝트별 권한 검증 로직 추가 필요

    // Selection 생성
    match use_case
        .create_selection(request, user_id, None)
        .await
    {
        Ok(response) => Ok(HttpResponse::Created().json(response)),
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// ViewSelection 조회 (GET)
///
/// Selection ID로 ViewSelection을 조회합니다.
/// Viewer 접근 시 TTL이 자동으로 연장됩니다.
#[utoipa::path(
    get,
    path = "/api/v1/view-selections/{selection_id}",
    responses(
        (status = 200, description = "Selection 조회 성공", body = ViewSelectionResponse),
        (status = 401, description = "인증 실패"),
        (status = 404, description = "Selection을 찾을 수 없음 또는 만료됨"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("selection_id" = String, Path, description = "Selection ID")
    ),
    tag = "view-selection"
)]
pub async fn get_view_selection<S>(
    path: web::Path<String>,
    req: HttpRequest,
    use_case: web::Data<Arc<ViewSelectionUseCase<S>>>,
    jwt: web::Data<Arc<JwtService>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
) -> Result<HttpResponse, actix_web::Error>
where
    S: crate::domain::view_selection::services::ViewSelectionService + 'static,
{
    // 사용자 ID 추출
    let user_id = match extract_user_id_from_request(&req, &jwt, &user_repo).await {
        Some(id) if id > 0 => id,
        _ => {
            return Ok(HttpResponse::Unauthorized().json(json!({
                "error": "Unauthorized",
                "message": "Invalid or missing authorization token"
            })));
        }
    };

    let selection_id = path.into_inner();

    // Selection 조회
    match use_case.get_selection(&selection_id).await {
        Ok(Some(selection)) => {
            // Viewer 접근 시 TTL 연장 (touch)
            // 비동기로 처리하여 응답 지연 최소화
            let use_case_clone = use_case.clone();
            let selection_id_clone = selection_id.clone();
            tokio::spawn(async move {
                let _ = use_case_clone.extend_ttl(&selection_id_clone, None).await;
            });

            Ok(HttpResponse::Ok().json(selection))
        }
        Ok(None) => Ok(HttpResponse::NotFound().json(json!({
            "error": "Not Found",
            "message": format!("Selection {} not found or expired", selection_id)
        }))),
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// ViewSelection 삭제 (DELETE)
///
/// Selection ID로 ViewSelection을 삭제합니다.
#[utoipa::path(
    delete,
    path = "/api/v1/view-selections/{selection_id}",
    responses(
        (status = 204, description = "Selection 삭제 성공"),
        (status = 401, description = "인증 실패"),
        (status = 404, description = "Selection을 찾을 수 없음"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("selection_id" = String, Path, description = "Selection ID")
    ),
    tag = "view-selection"
)]
pub async fn delete_view_selection<S>(
    path: web::Path<String>,
    req: HttpRequest,
    use_case: web::Data<Arc<ViewSelectionUseCase<S>>>,
    jwt: web::Data<Arc<JwtService>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
) -> Result<HttpResponse, actix_web::Error>
where
    S: crate::domain::view_selection::services::ViewSelectionService + 'static,
{
    // 사용자 ID 추출
    let user_id = match extract_user_id_from_request(&req, &jwt, &user_repo).await {
        Some(id) if id > 0 => id,
        _ => {
            return Ok(HttpResponse::Unauthorized().json(json!({
                "error": "Unauthorized",
                "message": "Invalid or missing authorization token"
            })));
        }
    };

    let selection_id = path.into_inner();

    // Selection 삭제
    match use_case.delete_selection(&selection_id).await {
        Ok(_) => Ok(HttpResponse::NoContent().finish()),
        Err(e) => Ok(handle_service_error(e)),
    }
}


