#![allow(dead_code, unused_imports, unused_variables)]
use actix_web::{web, HttpRequest, HttpResponse, Result};
use serde_json::json;
use std::sync::Arc;

use crate::application::reporting::dto::series_user_report_dto::*;
use crate::application::template::dto::report_guide_template_dto::*;
use crate::application::reporting::use_cases::SeriesUserReportUseCase;
use crate::application::template::use_cases::ReportGuideTemplateUseCase;
use crate::application::services::{SignedUrlService, SignedUrlRequest};
use crate::domain::ServiceError;
use crate::domain::repositories::ProjectDataRepository;
use crate::infrastructure::auth::{extract_user_id_from_request, JwtService};
use crate::infrastructure::repositories::{UserRepositoryImpl, ProjectDataRepositoryImpl};
use sqlx;

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

/// 프로젝트 종속 Series User Report 생성/수정
#[utoipa::path(
    put,
    path = "/api/project-data/{project_id}/series/{series_id}/report",
    request_body = CreateOrUpdateSeriesReportRequest,
    responses(
        (status = 200, description = "Report 생성/수정 성공", body = SeriesReportResponse),
        (status = 400, description = "잘못된 요청"),
        (status = 401, description = "인증 실패"),
        (status = 404, description = "Series 또는 프로젝트를 찾을 수 없음"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("project_id" = i32, Path, description = "프로젝트 ID"),
        ("series_id" = i32, Path, description = "Series ID")
    ),
    tag = "series-user-report"
)]
pub async fn create_or_update_project_report<S, U>(
    path: web::Path<(i32, i32)>,
    request: web::Json<CreateOrUpdateSeriesReportRequest>,
    req: HttpRequest,
    use_case: web::Data<Arc<SeriesUserReportUseCase<S, U>>>,
    jwt: web::Data<Arc<JwtService>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
) -> Result<HttpResponse, actix_web::Error>
where
    S: crate::domain::reporting::services::SeriesUserReportService + 'static,
    U: crate::domain::repositories::UserRepository + 'static,
{
    let (project_id, series_id) = path.into_inner();

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
        .create_or_update_report(series_id, user_id, Some(project_id), request.into_inner())
        .await
    {
        Ok(report) => Ok(HttpResponse::Ok().json(SeriesReportSingleResponse {
            success: true,
            id: Some(report.id),
            description: report.description,
            conclusion: report.conclusion,
        })),
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// 프로젝트 종속 Series User Report 조회
#[utoipa::path(
    get,
    path = "/api/project-data/{project_id}/series/{series_id}/report",
    responses(
        (status = 200, description = "Report 조회 성공", body = SeriesReportSingleResponse),
        (status = 401, description = "인증 실패"),
        (status = 404, description = "Report를 찾을 수 없음"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("project_id" = i32, Path, description = "프로젝트 ID"),
        ("series_id" = i32, Path, description = "Series ID")
    ),
    tag = "series-user-report"
)]
pub async fn get_project_report<S, U>(
    path: web::Path<(i32, i32)>,
    req: HttpRequest,
    use_case: web::Data<Arc<SeriesUserReportUseCase<S, U>>>,
    jwt: web::Data<Arc<JwtService>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
) -> Result<HttpResponse, actix_web::Error>
where
    S: crate::domain::reporting::services::SeriesUserReportService + 'static,
    U: crate::domain::repositories::UserRepository + 'static,
{
    let (project_id, series_id) = path.into_inner();

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
        .get_report(series_id, user_id, Some(project_id))
        .await
    {
        Ok(report) => {
            let (id, description, conclusion) = report
                .map(|r| (Some(r.id), r.description, r.conclusion))
                .unwrap_or((None, String::new(), String::new()));
            Ok(HttpResponse::Ok().json(SeriesReportSingleResponse {
                success: true,
                id,
                description,
                conclusion,
            }))
        },
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// 프로젝트 종속 Series의 모든 Report 조회
#[utoipa::path(
    get,
    path = "/api/project-data/{project_id}/series/{series_id}/reports",
    responses(
        (status = 200, description = "Report 목록 조회 성공", body = SeriesReportListResponse),
        (status = 401, description = "인증 실패"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("project_id" = i32, Path, description = "프로젝트 ID"),
        ("series_id" = i32, Path, description = "Series ID")
    ),
    tag = "series-user-report"
)]
pub async fn get_project_reports<S, U>(
    path: web::Path<(i32, i32)>,
    req: HttpRequest,
    use_case: web::Data<Arc<SeriesUserReportUseCase<S, U>>>,
    jwt: web::Data<Arc<JwtService>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
) -> Result<HttpResponse, actix_web::Error>
where
    S: crate::domain::reporting::services::SeriesUserReportService + 'static,
    U: crate::domain::repositories::UserRepository + 'static,
{
    let (project_id, series_id) = path.into_inner();

    let _user_id = match extract_user_id_from_request(&req, &jwt, &user_repo).await {
        Some(id) if id > 0 => id,
        _ => {
            return Ok(HttpResponse::Unauthorized().json(json!({
                "error": "Unauthorized",
                "message": "Invalid or missing authorization token"
            })));
        }
    };

    match use_case.get_reports_by_series(series_id, Some(project_id)).await {
        Ok(reports) => Ok(HttpResponse::Ok().json(reports)),
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// 프로젝트 종속 Series User Report 삭제
#[utoipa::path(
    delete,
    path = "/api/project-data/{project_id}/series/{series_id}/report",
    responses(
        (status = 200, description = "Report 삭제 성공"),
        (status = 401, description = "인증 실패"),
        (status = 404, description = "Report를 찾을 수 없음"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("project_id" = i32, Path, description = "프로젝트 ID"),
        ("series_id" = i32, Path, description = "Series ID")
    ),
    tag = "series-user-report"
)]
pub async fn delete_project_report<S, U>(
    path: web::Path<(i32, i32)>,
    req: HttpRequest,
    use_case: web::Data<Arc<SeriesUserReportUseCase<S, U>>>,
    jwt: web::Data<Arc<JwtService>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
) -> Result<HttpResponse, actix_web::Error>
where
    S: crate::domain::reporting::services::SeriesUserReportService + 'static,
    U: crate::domain::repositories::UserRepository + 'static,
{
    let (project_id, series_id) = path.into_inner();

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
        .delete_report(series_id, user_id, Some(project_id))
        .await
    {
        Ok(_) => Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "message": "Report deleted successfully"
        }))),
        Err(e) => Ok(handle_service_error(e)),
    }
}

// ========================================
// 전역 API
// ========================================

/// Series UID로 Series ID를 찾는 헬퍼 함수
async fn find_series_id_by_uid(
    series_uid: &str,
    project_data_repo: &ProjectDataRepositoryImpl,
) -> Result<i32, ServiceError> {
    let series_id: Option<i32> = sqlx::query_scalar(
        "SELECT id FROM project_data_series WHERE series_uid = $1 LIMIT 1"
    )
    .bind(series_uid)
    .fetch_optional(project_data_repo.pool())
    .await
    .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

    series_id.ok_or_else(|| ServiceError::NotFound(
        format!("Series not found: {}", series_uid)
    ))
}

/// 전역 Series User Report 생성/수정
#[utoipa::path(
    put,
    path = "/api/series/{series_uid}/report",
    request_body = CreateOrUpdateSeriesReportRequest,
    responses(
        (status = 200, description = "Report 생성/수정 성공", body = SeriesReportResponse),
        (status = 400, description = "잘못된 요청"),
        (status = 401, description = "인증 실패"),
        (status = 404, description = "Series를 찾을 수 없음"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("series_uid" = String, Path, description = "Series UID (DICOM Series Instance UID)")
    ),
    tag = "series-user-report"
)]
pub async fn create_or_update_global_report<S, U>(
    path: web::Path<String>,
    request: web::Json<CreateOrUpdateSeriesReportRequest>,
    req: HttpRequest,
    use_case: web::Data<Arc<SeriesUserReportUseCase<S, U>>>,
    jwt: web::Data<Arc<JwtService>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
    project_data_repo: web::Data<Arc<ProjectDataRepositoryImpl>>,
) -> Result<HttpResponse, actix_web::Error>
where
    S: crate::domain::reporting::services::SeriesUserReportService + 'static,
    U: crate::domain::repositories::UserRepository + 'static,
{
    let series_uid = path.into_inner();
    
    // Series UID로 Series ID 찾기
    let series_id = match find_series_id_by_uid(&series_uid, &project_data_repo).await {
        Ok(id) => id,
        Err(e) => return Ok(handle_service_error(e)),
    };

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
        .create_or_update_report(series_id, user_id, None, request.into_inner())
        .await
    {
        Ok(report) => Ok(HttpResponse::Ok().json(SeriesReportSingleResponse {
            success: true,
            id: Some(report.id),
            description: report.description,
            conclusion: report.conclusion,
        })),
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// 전역 Series User Report 조회
#[utoipa::path(
    get,
    path = "/api/series/{series_uid}/report",
    responses(
        (status = 200, description = "Report 조회 성공", body = SeriesReportSingleResponse),
        (status = 401, description = "인증 실패"),
        (status = 404, description = "Report를 찾을 수 없음"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("series_uid" = String, Path, description = "Series UID (DICOM Series Instance UID)")
    ),
    tag = "series-user-report"
)]
pub async fn get_global_report<S, U>(
    path: web::Path<String>,
    req: HttpRequest,
    use_case: web::Data<Arc<SeriesUserReportUseCase<S, U>>>,
    jwt: web::Data<Arc<JwtService>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
    project_data_repo: web::Data<Arc<ProjectDataRepositoryImpl>>,
) -> Result<HttpResponse, actix_web::Error>
where
    S: crate::domain::reporting::services::SeriesUserReportService + 'static,
    U: crate::domain::repositories::UserRepository + 'static,
{
    let series_uid = path.into_inner();
    
    // Series UID로 Series ID 찾기
    let series_id = match find_series_id_by_uid(&series_uid, &project_data_repo).await {
        Ok(id) => id,
        Err(e) => return Ok(handle_service_error(e)),
    };

    let user_id = match extract_user_id_from_request(&req, &jwt, &user_repo).await {
        Some(id) if id > 0 => id,
        _ => {
            return Ok(HttpResponse::Unauthorized().json(json!({
                "error": "Unauthorized",
                "message": "Invalid or missing authorization token"
            })));
        }
    };

    match use_case.get_report(series_id, user_id, None).await {
        Ok(report) => {
            let (id, description, conclusion) = report
                .map(|r| (Some(r.id), r.description, r.conclusion))
                .unwrap_or((None, String::new(), String::new()));
            Ok(HttpResponse::Ok().json(SeriesReportSingleResponse {
                success: true,
                id,
                description,
                conclusion,
            }))
        },
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// 전역 Series의 모든 Report 조회
#[utoipa::path(
    get,
    path = "/api/series/{series_uid}/reports",
    responses(
        (status = 200, description = "Report 목록 조회 성공", body = SeriesReportListResponse),
        (status = 401, description = "인증 실패"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("series_uid" = String, Path, description = "Series UID (DICOM Series Instance UID)")
    ),
    tag = "series-user-report"
)]
pub async fn get_global_reports<S, U>(
    path: web::Path<String>,
    req: HttpRequest,
    use_case: web::Data<Arc<SeriesUserReportUseCase<S, U>>>,
    jwt: web::Data<Arc<JwtService>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
    project_data_repo: web::Data<Arc<ProjectDataRepositoryImpl>>,
) -> Result<HttpResponse, actix_web::Error>
where
    S: crate::domain::reporting::services::SeriesUserReportService + 'static,
    U: crate::domain::repositories::UserRepository + 'static,
{
    let series_uid = path.into_inner();
    
    // Series UID로 Series ID 찾기
    let series_id = match find_series_id_by_uid(&series_uid, &project_data_repo).await {
        Ok(id) => id,
        Err(e) => return Ok(handle_service_error(e)),
    };

    let _user_id = match extract_user_id_from_request(&req, &jwt, &user_repo).await {
        Some(id) if id > 0 => id,
        _ => {
            return Ok(HttpResponse::Unauthorized().json(json!({
                "error": "Unauthorized",
                "message": "Invalid or missing authorization token"
            })));
        }
    };

    match use_case.get_reports_by_series(series_id, None).await {
        Ok(reports) => Ok(HttpResponse::Ok().json(reports)),
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// 전역 Series User Report 삭제
#[utoipa::path(
    delete,
    path = "/api/series/{series_uid}/report",
    responses(
        (status = 200, description = "Report 삭제 성공"),
        (status = 401, description = "인증 실패"),
        (status = 404, description = "Report를 찾을 수 없음"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("series_uid" = String, Path, description = "Series UID (DICOM Series Instance UID)")
    ),
    tag = "series-user-report"
)]
pub async fn delete_global_report<S, U>(
    path: web::Path<String>,
    req: HttpRequest,
    use_case: web::Data<Arc<SeriesUserReportUseCase<S, U>>>,
    jwt: web::Data<Arc<JwtService>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
    project_data_repo: web::Data<Arc<ProjectDataRepositoryImpl>>,
) -> Result<HttpResponse, actix_web::Error>
where
    S: crate::domain::reporting::services::SeriesUserReportService + 'static,
    U: crate::domain::repositories::UserRepository + 'static,
{
    let series_uid = path.into_inner();
    
    // Series UID로 Series ID 찾기
    let series_id = match find_series_id_by_uid(&series_uid, &project_data_repo).await {
        Ok(id) => id,
        Err(e) => return Ok(handle_service_error(e)),
    };

    let user_id = match extract_user_id_from_request(&req, &jwt, &user_repo).await {
        Some(id) if id > 0 => id,
        _ => {
            return Ok(HttpResponse::Unauthorized().json(json!({
                "error": "Unauthorized",
                "message": "Invalid or missing authorization token"
            })));
        }
    };

    match use_case.delete_report(series_id, user_id, None).await {
        Ok(_) => Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "message": "Report deleted successfully"
        }))),
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// 라우팅 설정
pub fn configure_routes<S, U>(
    cfg: &mut web::ServiceConfig,
    use_case: Arc<SeriesUserReportUseCase<S, U>>,
    jwt: Arc<JwtService>,
    user_repo: Arc<UserRepositoryImpl>,
) where
    S: crate::domain::reporting::services::SeriesUserReportService + 'static,
    U: crate::domain::repositories::UserRepository + 'static,
{
    cfg.app_data(web::Data::new(use_case))
        .app_data(web::Data::new(jwt))
        .app_data(web::Data::new(user_repo))
        // 프로젝트 종속 API - project_data_access_controller와 스코프 충돌 방지를 위해
        // 구체적인 경로를 먼저 등록
        .route(
            "/project-data/{project_id}/series/{series_id}/report",
            web::put().to(create_or_update_project_report::<S, U>),
        )
        .route(
            "/project-data/{project_id}/series/{series_id}/report",
            web::get().to(get_project_report::<S, U>),
        )
        .route(
            "/project-data/{project_id}/series/{series_id}/reports",
            web::get().to(get_project_reports::<S, U>),
        )
        .route(
            "/project-data/{project_id}/series/{series_id}/report",
            web::delete().to(delete_project_report::<S, U>),
        );
}

// ========================================
// 오디오 파일 업로드 API
// ========================================

/// 오디오 파일 업로드 URL 생성
#[utoipa::path(
    post,
    path = "/api/reports/{report_id}/dictate/upload-url",
    request_body = DictateUploadUrlRequest,
    responses(
        (status = 200, description = "업로드 URL 생성 성공", body = DictateUploadUrlResponse),
        (status = 400, description = "잘못된 요청"),
        (status = 401, description = "인증 실패"),
        (status = 404, description = "Report를 찾을 수 없음"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("report_id" = i32, Path, description = "Report ID")
    ),
    tag = "series-user-report"
)]
pub async fn generate_dictate_upload_url<S, U, SUS>(
    path: web::Path<i32>,
    request: web::Json<DictateUploadUrlRequest>,
    req: HttpRequest,
    use_case: web::Data<Arc<SeriesUserReportUseCase<S, U>>>,
    signed_url_service: web::Data<Arc<SUS>>,
    jwt: web::Data<Arc<JwtService>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
) -> Result<HttpResponse, actix_web::Error>
where
    S: crate::domain::reporting::services::SeriesUserReportService + 'static,
    U: crate::domain::repositories::UserRepository + 'static,
    SUS: SignedUrlService + 'static,
{
    let report_id = path.into_inner();
    let _user_id = match extract_user_id_from_request(&req, &jwt, &user_repo).await {
        Some(id) if id > 0 => id,
        _ => {
            return Ok(HttpResponse::Unauthorized().json(json!({
                "error": "Unauthorized",
                "message": "Invalid or missing authorization token"
            })));
        }
    };

    let file_name = format!("dictate_{}.mp3", report_id);
    let file_path = format!("reports/{}/dictate/{}", report_id, file_name);
    let content_type = request.mime_type.as_deref().unwrap_or("audio/mpeg");

    let signed_url_request = SignedUrlRequest {
        file_path: file_path.clone(),
        content_type: Some(content_type.to_string()),
        ttl_seconds: Some(600),
        content_disposition: None,
        metadata: None,
        acl: None,
    };

    match signed_url_service.generate_upload_url(signed_url_request).await {
        Ok(response) => Ok(HttpResponse::Ok().json(DictateUploadUrlResponse {
            success: true,
            upload_url: response.url,
            file_path: response.file_path,
            expires_in: response.ttl_seconds,
        })),
        Err(e) => Ok(HttpResponse::InternalServerError().json(json!({
            "error": "Failed to generate upload URL",
            "message": format!("{:?}", e)
        }))),
    }
}

/// 오디오 파일 업로드 완료 처리
#[utoipa::path(
    post,
    path = "/api/reports/{report_id}/dictate/complete",
    request_body = DictateUploadCompleteRequest,
    responses(
        (status = 200, description = "업로드 완료 처리 성공", body = DictateUploadCompleteResponse),
        (status = 400, description = "잘못된 요청"),
        (status = 401, description = "인증 실패"),
        (status = 404, description = "Report를 찾을 수 없음"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("report_id" = i32, Path, description = "Report ID")
    ),
    tag = "series-user-report"
)]
pub async fn complete_dictate_upload<S, U>(
    path: web::Path<i32>,
    request: web::Json<DictateUploadCompleteRequest>,
    req: HttpRequest,
    use_case: web::Data<Arc<SeriesUserReportUseCase<S, U>>>,
    jwt: web::Data<Arc<JwtService>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
) -> Result<HttpResponse, actix_web::Error>
where
    S: crate::domain::reporting::services::SeriesUserReportService + 'static,
    U: crate::domain::repositories::UserRepository + 'static,
{
    let _report_id = path.into_inner();
    let _user_id = match extract_user_id_from_request(&req, &jwt, &user_repo).await {
        Some(id) if id > 0 => id,
        _ => {
            return Ok(HttpResponse::Unauthorized().json(json!({
                "error": "Unauthorized",
                "message": "Invalid or missing authorization token"
            })));
        }
    };

    let _req = request.into_inner();
    // TODO: Report 조회 및 업데이트 로직 구현 필요

    Ok(HttpResponse::Ok().json(DictateUploadCompleteResponse {
        success: true,
        message: "Audio file upload completed successfully".to_string(),
    }))
}

// ========================================
// 템플릿 적용 API
// ========================================

/// 템플릿을 Report에 적용
#[utoipa::path(
    post,
    path = "/api/reports/{report_id}/apply-template",
    request_body = ApplyTemplateToReportRequest,
    responses(
        (status = 200, description = "템플릿 적용 성공", body = ApplyTemplateToReportResponse),
        (status = 400, description = "잘못된 요청"),
        (status = 401, description = "인증 실패"),
        (status = 404, description = "Report 또는 템플릿을 찾을 수 없음"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("report_id" = i32, Path, description = "Report ID")
    ),
    tag = "series-user-report"
)]
pub async fn apply_template_to_report<T, R, SUS>(
    path: web::Path<i32>,
    request: web::Json<ApplyTemplateToReportRequest>,
    req: HttpRequest,
    template_use_case: web::Data<Arc<ReportGuideTemplateUseCase<T, R, SUS>>>,
    jwt: web::Data<Arc<JwtService>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
) -> Result<HttpResponse, actix_web::Error>
where
    T: crate::domain::template::services::ReportGuideTemplateService + 'static,
    R: crate::domain::reporting::repositories::SeriesUserReportRepository + 'static,
    SUS: crate::application::services::SignedUrlService + 'static,
{
    let report_id = path.into_inner();
    let user_id = match extract_user_id_from_request(&req, &jwt, &user_repo).await {
        Some(id) if id > 0 => id,
        _ => {
            return Ok(HttpResponse::Unauthorized().json(json!({
                "error": "Unauthorized",
                "message": "Invalid or missing authorization token"
            })));
        }
    };

    match template_use_case
        .apply_template_to_report(report_id, request.into_inner(), user_id)
        .await
    {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(e) => Ok(handle_service_error(e)),
    }
}

// ========================================
// Report Guide Image 관리 API
// ========================================

/// Report의 Guide Image 목록 조회
#[utoipa::path(
    get,
    path = "/api/reports/{report_id}/guides",
    responses(
        (status = 200, description = "Guide Image 목록 조회 성공", body = ReportGuideListResponse),
        (status = 401, description = "인증 실패"),
        (status = 404, description = "Report를 찾을 수 없음"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("report_id" = i32, Path, description = "Report ID")
    ),
    tag = "series-user-report"
)]
pub async fn get_report_guides<T, R, SUS>(
    path: web::Path<i32>,
    req: HttpRequest,
    template_use_case: web::Data<Arc<ReportGuideTemplateUseCase<T, R, SUS>>>,
    jwt: web::Data<Arc<JwtService>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
) -> Result<HttpResponse, actix_web::Error>
where
    T: crate::domain::template::services::ReportGuideTemplateService + 'static,
    R: crate::domain::reporting::repositories::SeriesUserReportRepository + 'static,
    SUS: crate::application::services::SignedUrlService + 'static,
{
    let report_id = path.into_inner();
    let user_id = match extract_user_id_from_request(&req, &jwt, &user_repo).await {
        Some(id) if id > 0 => id,
        _ => {
            return Ok(HttpResponse::Unauthorized().json(json!({
                "error": "Unauthorized",
                "message": "Invalid or missing authorization token"
            })));
        }
    };

    match template_use_case.get_report_guides(report_id, user_id).await {
        Ok(guides) => Ok(HttpResponse::Ok().json(ReportGuideListResponse {
            success: true,
            guides,
        })),
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// Report에 Guide Image 추가
#[utoipa::path(
    post,
    path = "/api/reports/{report_id}/guides",
    request_body = AddReportGuideRequest,
    responses(
        (status = 200, description = "Guide Image 추가 성공", body = ReportGuideResponse),
        (status = 400, description = "잘못된 요청"),
        (status = 401, description = "인증 실패"),
        (status = 404, description = "Report 또는 템플릿을 찾을 수 없음"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("report_id" = i32, Path, description = "Report ID")
    ),
    tag = "series-user-report"
)]
pub async fn add_report_guide<T, R, SUS>(
    path: web::Path<i32>,
    request: web::Json<AddReportGuideRequest>,
    req: HttpRequest,
    template_use_case: web::Data<Arc<ReportGuideTemplateUseCase<T, R, SUS>>>,
    jwt: web::Data<Arc<JwtService>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
) -> Result<HttpResponse, actix_web::Error>
where
    T: crate::domain::template::services::ReportGuideTemplateService + 'static,
    R: crate::domain::reporting::repositories::SeriesUserReportRepository + 'static,
    SUS: crate::application::services::SignedUrlService + 'static,
{
    let report_id = path.into_inner();
    let user_id = match extract_user_id_from_request(&req, &jwt, &user_repo).await {
        Some(id) if id > 0 => id,
        _ => {
            return Ok(HttpResponse::Unauthorized().json(json!({
                "error": "Unauthorized",
                "message": "Invalid or missing authorization token"
            })));
        }
    };

    match template_use_case
        .add_report_guide(report_id, user_id, request.into_inner())
        .await
    {
        Ok(guide) => Ok(HttpResponse::Ok().json(guide)),
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// Report에서 Guide Image 삭제
#[utoipa::path(
    delete,
    path = "/api/reports/{report_id}/guides/{guide_id}",
    responses(
        (status = 200, description = "Guide Image 삭제 성공"),
        (status = 401, description = "인증 실패"),
        (status = 404, description = "Guide Image를 찾을 수 없음"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("report_id" = i32, Path, description = "Report ID"),
        ("guide_id" = i32, Path, description = "Guide ID")
    ),
    tag = "series-user-report"
)]
pub async fn delete_report_guide<T, R, SUS>(
    path: web::Path<(i32, i32)>,
    req: HttpRequest,
    template_use_case: web::Data<Arc<ReportGuideTemplateUseCase<T, R, SUS>>>,
    jwt: web::Data<Arc<JwtService>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
) -> Result<HttpResponse, actix_web::Error>
where
    T: crate::domain::template::services::ReportGuideTemplateService + 'static,
    R: crate::domain::reporting::repositories::SeriesUserReportRepository + 'static,
    SUS: crate::application::services::SignedUrlService + 'static,
{
    let (report_id, _guide_id) = path.into_inner();
    let user_id = match extract_user_id_from_request(&req, &jwt, &user_repo).await {
        Some(id) if id > 0 => id,
        _ => {
            return Ok(HttpResponse::Unauthorized().json(json!({
                "error": "Unauthorized",
                "message": "Invalid or missing authorization token"
            })));
        }
    };

    match template_use_case.delete_report_guide(report_id, user_id).await {
        Ok(_) => Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "message": "Guide deleted successfully"
        }))),
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// 오디오 파일 업로드 및 템플릿 적용 라우팅 설정
pub fn configure_report_extension_routes<S, U, T, R, SUS>(
    cfg: &mut web::ServiceConfig,
    report_use_case: Arc<SeriesUserReportUseCase<S, U>>,
    template_use_case: Arc<ReportGuideTemplateUseCase<T, R, SUS>>,
    signed_url_service: Arc<SUS>,
    jwt: Arc<JwtService>,
    user_repo: Arc<UserRepositoryImpl>,
) where
    S: crate::domain::reporting::services::SeriesUserReportService + 'static,
    U: crate::domain::repositories::UserRepository + 'static,
    T: crate::domain::template::services::ReportGuideTemplateService + 'static,
    R: crate::domain::reporting::repositories::SeriesUserReportRepository + 'static,
    SUS: SignedUrlService + 'static,
{
    cfg.app_data(web::Data::new(report_use_case))
        .app_data(web::Data::new(template_use_case))
        .app_data(web::Data::new(signed_url_service))
        .app_data(web::Data::new(jwt))
        .app_data(web::Data::new(user_repo))
        .service(
            web::scope("/reports")
                .route("/{report_id}/dictate/upload-url", web::post().to(generate_dictate_upload_url::<S, U, SUS>))
                .route("/{report_id}/dictate/complete", web::post().to(complete_dictate_upload::<S, U>))
                .route("/{report_id}/apply-template", web::post().to(apply_template_to_report::<T, R, SUS>))
                .route("/{report_id}/guides", web::get().to(get_report_guides::<T, R, SUS>))
                .route("/{report_id}/guides", web::post().to(add_report_guide::<T, R, SUS>))
                .route("/{report_id}/guides/{guide_id}", web::delete().to(delete_report_guide::<T, R, SUS>)),
        );
}

