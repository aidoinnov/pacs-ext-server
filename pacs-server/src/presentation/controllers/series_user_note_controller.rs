#![allow(dead_code, unused_imports, unused_variables)]
use actix_web::{web, HttpRequest, HttpResponse, Result};
use serde_json::json;
use std::sync::Arc;

use crate::application::dto::series_user_note_dto::*;
use crate::application::use_cases::SeriesUserNoteUseCase;
use crate::domain::ServiceError;
use crate::infrastructure::auth::{extract_user_id_from_request, JwtService};
use crate::infrastructure::repositories::UserRepositoryImpl;

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

// ========================================
// 프로젝트 종속 API
// ========================================

/// 프로젝트 종속 Series User Note 생성/수정
#[utoipa::path(
    put,
    path = "/api/project-data/{project_id}/series/{series_id}/note",
    request_body = CreateOrUpdateSeriesNoteRequest,
    responses(
        (status = 200, description = "Note 생성/수정 성공", body = SeriesNoteResponse),
        (status = 400, description = "잘못된 요청"),
        (status = 401, description = "인증 실패"),
        (status = 403, description = "권한 없음"),
        (status = 404, description = "Series 또는 프로젝트를 찾을 수 없음"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("project_id" = i32, Path, description = "프로젝트 ID"),
        ("series_id" = i32, Path, description = "Series ID")
    ),
    tag = "series-user-note"
)]
pub async fn create_or_update_project_note<S, U>(
    path: web::Path<(i32, i32)>,
    request: web::Json<CreateOrUpdateSeriesNoteRequest>,
    req: HttpRequest,
    use_case: web::Data<Arc<SeriesUserNoteUseCase<S, U>>>,
    jwt: web::Data<Arc<JwtService>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
) -> Result<HttpResponse, actix_web::Error>
where
    S: crate::domain::services::SeriesUserNoteService + 'static,
    U: crate::domain::repositories::UserRepository + 'static,
{
    let (project_id, series_id) = path.into_inner();

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

    match use_case
        .create_or_update_note(series_id, user_id, Some(project_id), request.into_inner())
        .await
    {
        Ok(note) => Ok(HttpResponse::Ok().json(SeriesNoteSingleResponse {
            success: true,
            note: Some(note),
        })),
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// 프로젝트 종속 Series User Note 조회
#[utoipa::path(
    get,
    path = "/api/project-data/{project_id}/series/{series_id}/note",
    responses(
        (status = 200, description = "Note 조회 성공", body = SeriesNoteSingleResponse),
        (status = 401, description = "인증 실패"),
        (status = 404, description = "Note를 찾을 수 없음"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("project_id" = i32, Path, description = "프로젝트 ID"),
        ("series_id" = i32, Path, description = "Series ID")
    ),
    tag = "series-user-note"
)]
pub async fn get_project_note<S, U>(
    path: web::Path<(i32, i32)>,
    req: HttpRequest,
    use_case: web::Data<Arc<SeriesUserNoteUseCase<S, U>>>,
    jwt: web::Data<Arc<JwtService>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
) -> Result<HttpResponse, actix_web::Error>
where
    S: crate::domain::services::SeriesUserNoteService + 'static,
    U: crate::domain::repositories::UserRepository + 'static,
{
    let (project_id, series_id) = path.into_inner();

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

    match use_case
        .get_note(series_id, user_id, Some(project_id))
        .await
    {
        Ok(note) => Ok(HttpResponse::Ok().json(SeriesNoteSingleResponse {
            success: true,
            note,
        })),
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// 프로젝트 종속 Series의 모든 User Note 조회 (관리자용)
#[utoipa::path(
    get,
    path = "/api/project-data/{project_id}/series/{series_id}/notes",
    responses(
        (status = 200, description = "Note 목록 조회 성공", body = SeriesNoteListResponse),
        (status = 401, description = "인증 실패"),
        (status = 403, description = "권한 없음"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("project_id" = i32, Path, description = "프로젝트 ID"),
        ("series_id" = i32, Path, description = "Series ID")
    ),
    tag = "series-user-note"
)]
pub async fn get_project_notes<S, U>(
    path: web::Path<(i32, i32)>,
    req: HttpRequest,
    use_case: web::Data<Arc<SeriesUserNoteUseCase<S, U>>>,
    jwt: web::Data<Arc<JwtService>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
) -> Result<HttpResponse, actix_web::Error>
where
    S: crate::domain::services::SeriesUserNoteService + 'static,
    U: crate::domain::repositories::UserRepository + 'static,
{
    let (project_id, series_id) = path.into_inner();

    // 사용자 ID 추출 (권한 확인용)
    let _user_id = match extract_user_id_from_request(&req, &jwt, &user_repo).await {
        Some(id) if id > 0 => id,
        _ => {
            return Ok(HttpResponse::Unauthorized().json(json!({
                "error": "Unauthorized",
                "message": "Invalid or missing authorization token"
            })));
        }
    };

    // TODO: 프로젝트 관리자 권한 확인 추가 필요

    match use_case
        .get_notes_by_series(series_id, Some(project_id))
        .await
    {
        Ok(notes) => Ok(HttpResponse::Ok().json(SeriesNoteListResponse {
            success: true,
            notes,
        })),
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// 프로젝트 종속 Series User Note 삭제
#[utoipa::path(
    delete,
    path = "/api/project-data/{project_id}/series/{series_id}/note",
    responses(
        (status = 200, description = "Note 삭제 성공"),
        (status = 401, description = "인증 실패"),
        (status = 403, description = "권한 없음"),
        (status = 404, description = "Note를 찾을 수 없음"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("project_id" = i32, Path, description = "프로젝트 ID"),
        ("series_id" = i32, Path, description = "Series ID")
    ),
    tag = "series-user-note"
)]
pub async fn delete_project_note<S, U>(
    path: web::Path<(i32, i32)>,
    req: HttpRequest,
    use_case: web::Data<Arc<SeriesUserNoteUseCase<S, U>>>,
    jwt: web::Data<Arc<JwtService>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
) -> Result<HttpResponse, actix_web::Error>
where
    S: crate::domain::services::SeriesUserNoteService + 'static,
    U: crate::domain::repositories::UserRepository + 'static,
{
    let (project_id, series_id) = path.into_inner();

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

    match use_case
        .delete_note(series_id, user_id, Some(project_id))
        .await
    {
        Ok(_) => Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "message": "Note deleted successfully"
        }))),
        Err(e) => Ok(handle_service_error(e)),
    }
}

// ========================================
// 전역 API
// ========================================

/// 전역 Series User Note 생성/수정
#[utoipa::path(
    put,
    path = "/api/series/{series_id}/note",
    request_body = CreateOrUpdateSeriesNoteRequest,
    responses(
        (status = 200, description = "Note 생성/수정 성공", body = SeriesNoteResponse),
        (status = 400, description = "잘못된 요청"),
        (status = 401, description = "인증 실패"),
        (status = 404, description = "Series를 찾을 수 없음"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("series_id" = i32, Path, description = "Series ID")
    ),
    tag = "series-user-note"
)]
pub async fn create_or_update_global_note<S, U>(
    path: web::Path<i32>,
    request: web::Json<CreateOrUpdateSeriesNoteRequest>,
    req: HttpRequest,
    use_case: web::Data<Arc<SeriesUserNoteUseCase<S, U>>>,
    jwt: web::Data<Arc<JwtService>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
) -> Result<HttpResponse, actix_web::Error>
where
    S: crate::domain::services::SeriesUserNoteService + 'static,
    U: crate::domain::repositories::UserRepository + 'static,
{
    let series_id = path.into_inner();

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

    match use_case
        .create_or_update_note(series_id, user_id, None, request.into_inner())
        .await
    {
        Ok(note) => Ok(HttpResponse::Ok().json(SeriesNoteSingleResponse {
            success: true,
            note: Some(note),
        })),
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// 전역 Series User Note 조회
#[utoipa::path(
    get,
    path = "/api/series/{series_id}/note",
    responses(
        (status = 200, description = "Note 조회 성공", body = SeriesNoteSingleResponse),
        (status = 401, description = "인증 실패"),
        (status = 404, description = "Note를 찾을 수 없음"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("series_id" = i32, Path, description = "Series ID")
    ),
    tag = "series-user-note"
)]
pub async fn get_global_note<S, U>(
    path: web::Path<i32>,
    req: HttpRequest,
    use_case: web::Data<Arc<SeriesUserNoteUseCase<S, U>>>,
    jwt: web::Data<Arc<JwtService>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
) -> Result<HttpResponse, actix_web::Error>
where
    S: crate::domain::services::SeriesUserNoteService + 'static,
    U: crate::domain::repositories::UserRepository + 'static,
{
    let series_id = path.into_inner();

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

    match use_case.get_note(series_id, user_id, None).await {
        Ok(note) => Ok(HttpResponse::Ok().json(SeriesNoteSingleResponse {
            success: true,
            note,
        })),
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// 전역 Series의 모든 User Note 조회 (관리자용)
#[utoipa::path(
    get,
    path = "/api/series/{series_id}/notes",
    responses(
        (status = 200, description = "Note 목록 조회 성공", body = SeriesNoteListResponse),
        (status = 401, description = "인증 실패"),
        (status = 403, description = "권한 없음"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("series_id" = i32, Path, description = "Series ID")
    ),
    tag = "series-user-note"
)]
pub async fn get_global_notes<S, U>(
    path: web::Path<i32>,
    req: HttpRequest,
    use_case: web::Data<Arc<SeriesUserNoteUseCase<S, U>>>,
    jwt: web::Data<Arc<JwtService>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
) -> Result<HttpResponse, actix_web::Error>
where
    S: crate::domain::services::SeriesUserNoteService + 'static,
    U: crate::domain::repositories::UserRepository + 'static,
{
    let series_id = path.into_inner();

    // 사용자 ID 추출 (권한 확인용)
    let _user_id = match extract_user_id_from_request(&req, &jwt, &user_repo).await {
        Some(id) if id > 0 => id,
        _ => {
            return Ok(HttpResponse::Unauthorized().json(json!({
                "error": "Unauthorized",
                "message": "Invalid or missing authorization token"
            })));
        }
    };

    // TODO: 시스템 관리자 권한 확인 추가 필요

    match use_case.get_notes_by_series(series_id, None).await {
        Ok(notes) => Ok(HttpResponse::Ok().json(SeriesNoteListResponse {
            success: true,
            notes,
        })),
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// 전역 Series User Note 삭제
#[utoipa::path(
    delete,
    path = "/api/series/{series_id}/note",
    responses(
        (status = 200, description = "Note 삭제 성공"),
        (status = 401, description = "인증 실패"),
        (status = 403, description = "권한 없음"),
        (status = 404, description = "Note를 찾을 수 없음"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("series_id" = i32, Path, description = "Series ID")
    ),
    tag = "series-user-note"
)]
pub async fn delete_global_note<S, U>(
    path: web::Path<i32>,
    req: HttpRequest,
    use_case: web::Data<Arc<SeriesUserNoteUseCase<S, U>>>,
    jwt: web::Data<Arc<JwtService>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
) -> Result<HttpResponse, actix_web::Error>
where
    S: crate::domain::services::SeriesUserNoteService + 'static,
    U: crate::domain::repositories::UserRepository + 'static,
{
    let series_id = path.into_inner();

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

    match use_case.delete_note(series_id, user_id, None).await {
        Ok(_) => Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "message": "Note deleted successfully"
        }))),
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// 라우트 설정 함수
pub fn configure_routes<S, U>(
    cfg: &mut web::ServiceConfig,
    use_case: Arc<SeriesUserNoteUseCase<S, U>>,
    jwt: Arc<JwtService>,
    user_repo: Arc<UserRepositoryImpl>,
) where
    S: crate::domain::services::SeriesUserNoteService + 'static,
    U: crate::domain::repositories::UserRepository + 'static,
{
    cfg.app_data(web::Data::new(use_case))
        .app_data(web::Data::new(jwt))
        .app_data(web::Data::new(user_repo))
        // 프로젝트 종속 API - project_data_access_controller와 스코프 충돌 방지를 위해
        // 구체적인 경로를 먼저 등록
        .route(
            "/project-data/{project_id}/series/{series_id}/note",
            web::put().to(create_or_update_project_note::<S, U>),
        )
        .route(
            "/project-data/{project_id}/series/{series_id}/note",
            web::get().to(get_project_note::<S, U>),
        )
        .route(
            "/project-data/{project_id}/series/{series_id}/notes",
            web::get().to(get_project_notes::<S, U>),
        )
        .route(
            "/project-data/{project_id}/series/{series_id}/note",
            web::delete().to(delete_project_note::<S, U>),
        );
}

/// 전역 Series API 스코프 설정 (Note + Report 통합)
pub fn configure_global_series_routes<S, U, R, UR>(
    cfg: &mut web::ServiceConfig,
    note_use_case: Arc<SeriesUserNoteUseCase<S, U>>,
    report_use_case: Arc<crate::application::reporting::use_cases::SeriesUserReportUseCase<R, UR>>,
    jwt: Arc<JwtService>,
    user_repo: Arc<UserRepositoryImpl>,
) where
    S: crate::domain::services::SeriesUserNoteService + 'static,
    U: crate::domain::repositories::UserRepository + 'static,
    R: crate::domain::reporting::services::SeriesUserReportService + 'static,
    UR: crate::domain::repositories::UserRepository + 'static,
{
    cfg.app_data(web::Data::new(note_use_case))
        .app_data(web::Data::new(report_use_case))
        .app_data(web::Data::new(jwt))
        .app_data(web::Data::new(user_repo))
        .service(
            // 전역 API (Note + Report 통합)
            web::scope("/series")
                // Note API
                .route("/{series_id}/note", web::put().to(create_or_update_global_note::<S, U>))
                .route("/{series_id}/note", web::get().to(get_global_note::<S, U>))
                .route("/{series_id}/notes", web::get().to(get_global_notes::<S, U>))
                .route("/{series_id}/note", web::delete().to(delete_global_note::<S, U>))
                // Report API
                .route("/{series_id}/report", web::put().to(crate::presentation::reporting::controllers::series_user_report_controller::create_or_update_global_report::<R, UR>))
                .route("/{series_id}/report", web::get().to(crate::presentation::reporting::controllers::series_user_report_controller::get_global_report::<R, UR>))
                .route("/{series_id}/reports", web::get().to(crate::presentation::reporting::controllers::series_user_report_controller::get_global_reports::<R, UR>))
                .route("/{series_id}/report", web::delete().to(crate::presentation::reporting::controllers::series_user_report_controller::delete_global_report::<R, UR>)),
        );
}

