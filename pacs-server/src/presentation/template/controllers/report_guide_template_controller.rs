#![allow(dead_code, unused_imports, unused_variables)]
use actix_web::{web, HttpRequest, HttpResponse, Result};
use serde_json::json;
use std::sync::Arc;

use crate::application::template::dto::report_guide_template_dto::*;
use crate::application::template::use_cases::ReportGuideTemplateUseCase;
use crate::application::services::{SignedUrlService, SignedUrlRequest};
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
// 원본 템플릿 API
// ========================================

/// 원본 템플릿 생성
#[utoipa::path(
    post,
    path = "/api/report-guide-templates",
    request_body = CreateReportGuideTemplateRequest,
    responses(
        (status = 200, description = "템플릿 생성 성공", body = ReportGuideTemplateResponse),
        (status = 400, description = "잘못된 요청"),
        (status = 401, description = "인증 실패"),
        (status = 500, description = "서버 내부 오류")
    ),
    tag = "report-guide-template"
)]
pub async fn create_template<T>(
    request: web::Json<CreateReportGuideTemplateRequest>,
    req: HttpRequest,
    use_case: web::Data<Arc<ReportGuideTemplateUseCase<T>>>,
    jwt: web::Data<Arc<JwtService>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
) -> Result<HttpResponse, actix_web::Error>
where
    T: crate::domain::template::services::ReportGuideTemplateService + 'static,
{
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
        .create_template(request.into_inner(), user_id)
        .await
    {
        Ok(template) => Ok(HttpResponse::Ok().json(template)),
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// 원본 템플릿 조회
#[utoipa::path(
    get,
    path = "/api/report-guide-templates/{template_id}",
    responses(
        (status = 200, description = "템플릿 조회 성공", body = ReportGuideTemplateResponse),
        (status = 401, description = "인증 실패"),
        (status = 404, description = "템플릿을 찾을 수 없음"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("template_id" = i32, Path, description = "템플릿 ID")
    ),
    tag = "report-guide-template"
)]
pub async fn get_template<T>(
    path: web::Path<i32>,
    req: HttpRequest,
    use_case: web::Data<Arc<ReportGuideTemplateUseCase<T>>>,
    jwt: web::Data<Arc<JwtService>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
) -> Result<HttpResponse, actix_web::Error>
where
    T: crate::domain::template::services::ReportGuideTemplateService + 'static,
{
    let template_id = path.into_inner();
    let user_id = extract_user_id_from_request(&req, &jwt, &user_repo).await;

    match use_case.get_template(template_id, user_id).await {
        Ok(Some(template)) => Ok(HttpResponse::Ok().json(template)),
        Ok(None) => Ok(HttpResponse::NotFound().json(json!({
            "error": "Not Found",
            "message": "Template not found"
        }))),
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// 원본 템플릿 목록 조회
#[utoipa::path(
    get,
    path = "/api/report-guide-templates",
    responses(
        (status = 200, description = "템플릿 목록 조회 성공", body = ReportGuideTemplateListResponse),
        (status = 401, description = "인증 실패"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("modality" = Option<String>, Query, description = "모달리티 필터"),
        ("bodypart" = Option<String>, Query, description = "신체 부위 필터"),
        ("is_active" = Option<bool>, Query, description = "활성 상태 필터")
    ),
    tag = "report-guide-template"
)]
pub async fn get_templates<T>(
    query: web::Query<std::collections::HashMap<String, String>>,
    req: HttpRequest,
    use_case: web::Data<Arc<ReportGuideTemplateUseCase<T>>>,
    jwt: web::Data<Arc<JwtService>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
) -> Result<HttpResponse, actix_web::Error>
where
    T: crate::domain::template::services::ReportGuideTemplateService + 'static,
{
    let user_id = extract_user_id_from_request(&req, &jwt, &user_repo).await;
    let modality = query.get("modality").map(|s| s.clone());
    let bodypart = query.get("bodypart").map(|s| s.clone());
    let is_active = query
        .get("is_active")
        .and_then(|s| s.parse::<bool>().ok());

    match use_case
        .get_templates(modality, bodypart, is_active, user_id)
        .await
    {
        Ok(templates) => Ok(HttpResponse::Ok().json(templates)),
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// 원본 템플릿 수정
#[utoipa::path(
    put,
    path = "/api/report-guide-templates/{template_id}",
    request_body = UpdateReportGuideTemplateRequest,
    responses(
        (status = 200, description = "템플릿 수정 성공", body = ReportGuideTemplateResponse),
        (status = 400, description = "잘못된 요청"),
        (status = 401, description = "인증 실패"),
        (status = 404, description = "템플릿을 찾을 수 없음"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("template_id" = i32, Path, description = "템플릿 ID")
    ),
    tag = "report-guide-template"
)]
pub async fn update_template<T>(
    path: web::Path<i32>,
    request: web::Json<UpdateReportGuideTemplateRequest>,
    req: HttpRequest,
    use_case: web::Data<Arc<ReportGuideTemplateUseCase<T>>>,
    jwt: web::Data<Arc<JwtService>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
) -> Result<HttpResponse, actix_web::Error>
where
    T: crate::domain::template::services::ReportGuideTemplateService + 'static,
{
    let template_id = path.into_inner();
    let _user_id = match extract_user_id_from_request(&req, &jwt, &user_repo).await {
        Some(id) if id > 0 => id,
        _ => {
            return Ok(HttpResponse::Unauthorized().json(json!({
                "error": "Unauthorized",
                "message": "Invalid or missing authorization token"
            })));
        }
    };

    match use_case.update_template(template_id, request.into_inner()).await {
        Ok(template) => Ok(HttpResponse::Ok().json(template)),
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// 원본 템플릿 삭제
#[utoipa::path(
    delete,
    path = "/api/report-guide-templates/{template_id}",
    responses(
        (status = 200, description = "템플릿 삭제 성공"),
        (status = 401, description = "인증 실패"),
        (status = 404, description = "템플릿을 찾을 수 없음"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("template_id" = i32, Path, description = "템플릿 ID")
    ),
    tag = "report-guide-template"
)]
pub async fn delete_template<T>(
    path: web::Path<i32>,
    req: HttpRequest,
    use_case: web::Data<Arc<ReportGuideTemplateUseCase<T>>>,
    jwt: web::Data<Arc<JwtService>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
) -> Result<HttpResponse, actix_web::Error>
where
    T: crate::domain::template::services::ReportGuideTemplateService + 'static,
{
    let template_id = path.into_inner();
    let _user_id = match extract_user_id_from_request(&req, &jwt, &user_repo).await {
        Some(id) if id > 0 => id,
        _ => {
            return Ok(HttpResponse::Unauthorized().json(json!({
                "error": "Unauthorized",
                "message": "Invalid or missing authorization token"
            })));
        }
    };

    match use_case.delete_template(template_id).await {
        Ok(_) => Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "message": "Template deleted successfully"
        }))),
        Err(e) => Ok(handle_service_error(e)),
    }
}

// ========================================
// 템플릿 이미지 API
// ========================================

/// 템플릿 이미지 추가
#[utoipa::path(
    post,
    path = "/api/report-guide-templates/{template_id}/images",
    request_body = AddTemplateImageRequest,
    responses(
        (status = 200, description = "이미지 추가 성공", body = TemplateImageResponse),
        (status = 400, description = "잘못된 요청"),
        (status = 401, description = "인증 실패"),
        (status = 404, description = "템플릿을 찾을 수 없음"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("template_id" = i32, Path, description = "템플릿 ID")
    ),
    tag = "report-guide-template"
)]
pub async fn add_template_image<T>(
    path: web::Path<i32>,
    request: web::Json<AddTemplateImageRequest>,
    req: HttpRequest,
    use_case: web::Data<Arc<ReportGuideTemplateUseCase<T>>>,
    jwt: web::Data<Arc<JwtService>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
) -> Result<HttpResponse, actix_web::Error>
where
    T: crate::domain::template::services::ReportGuideTemplateService + 'static,
{
    let template_id = path.into_inner();
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
        .add_template_image(template_id, request.into_inner(), user_id)
        .await
    {
        Ok(image) => Ok(HttpResponse::Ok().json(image)),
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// 템플릿 이미지 공유 설정 변경
#[utoipa::path(
    put,
    path = "/api/report-guide-templates/{template_id}/images/{image_id}/share",
    request_body = UpdateImageShareStatusRequest,
    responses(
        (status = 200, description = "공유 설정 변경 성공", body = TemplateImageResponse),
        (status = 401, description = "인증 실패"),
        (status = 403, description = "권한 없음"),
        (status = 404, description = "이미지를 찾을 수 없음"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("template_id" = i32, Path, description = "템플릿 ID"),
        ("image_id" = i32, Path, description = "이미지 ID")
    ),
    tag = "report-guide-template"
)]
pub async fn update_image_share_status<T>(
    path: web::Path<(i32, i32)>,
    request: web::Json<UpdateImageShareStatusRequest>,
    req: HttpRequest,
    use_case: web::Data<Arc<ReportGuideTemplateUseCase<T>>>,
    jwt: web::Data<Arc<JwtService>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
) -> Result<HttpResponse, actix_web::Error>
where
    T: crate::domain::template::services::ReportGuideTemplateService + 'static,
{
    let (template_id, image_id) = path.into_inner();
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
        .update_image_share_status(image_id, request.into_inner(), user_id)
        .await
    {
        Ok(image) => Ok(HttpResponse::Ok().json(image)),
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// 템플릿 이미지 업로드 URL 생성
#[utoipa::path(
    post,
    path = "/api/report-guide-templates/{template_id}/images/upload-url",
    request_body = TemplateImageUploadUrlRequest,
    responses(
        (status = 200, description = "업로드 URL 생성 성공", body = TemplateImageUploadUrlResponse),
        (status = 400, description = "잘못된 요청"),
        (status = 401, description = "인증 실패"),
        (status = 404, description = "템플릿을 찾을 수 없음"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("template_id" = i32, Path, description = "템플릿 ID")
    ),
    tag = "report-guide-template"
)]
pub async fn generate_template_image_upload_url<T, SUS>(
    path: web::Path<i32>,
    request: web::Json<TemplateImageUploadUrlRequest>,
    req: HttpRequest,
    use_case: web::Data<Arc<ReportGuideTemplateUseCase<T>>>,
    signed_url_service: web::Data<Arc<SUS>>,
    jwt: web::Data<Arc<JwtService>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
) -> Result<HttpResponse, actix_web::Error>
where
    T: crate::domain::template::services::ReportGuideTemplateService + 'static,
    SUS: SignedUrlService + 'static,
{
    let template_id = path.into_inner();
    let _user_id = match extract_user_id_from_request(&req, &jwt, &user_repo).await {
        Some(id) if id > 0 => id,
        _ => {
            return Ok(HttpResponse::Unauthorized().json(json!({
                "error": "Unauthorized",
                "message": "Invalid or missing authorization token"
            })));
        }
    };

    // 템플릿 존재 확인은 생략 (이미지 업로드 URL 생성만 수행)

    let file_name = request.file_name.clone();
    let file_path = format!("templates/{}/images/{}", template_id, file_name);
    let content_type = request.mime_type.as_deref().unwrap_or("image/png");

    let signed_url_request = SignedUrlRequest {
        file_path: file_path.clone(),
        content_type: Some(content_type.to_string()),
        ttl_seconds: Some(600),
        content_disposition: None,
        metadata: None,
        acl: None,
    };

    match signed_url_service.generate_upload_url(signed_url_request).await {
        Ok(response) => Ok(HttpResponse::Ok().json(TemplateImageUploadUrlResponse {
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

/// 템플릿 이미지 업로드 완료 처리
#[utoipa::path(
    post,
    path = "/api/report-guide-templates/{template_id}/images/complete",
    request_body = TemplateImageUploadCompleteRequest,
    responses(
        (status = 200, description = "업로드 완료 처리 성공", body = TemplateImageUploadCompleteResponse),
        (status = 400, description = "잘못된 요청"),
        (status = 401, description = "인증 실패"),
        (status = 404, description = "템플릿을 찾을 수 없음"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("template_id" = i32, Path, description = "템플릿 ID")
    ),
    tag = "report-guide-template"
)]
pub async fn complete_template_image_upload<T>(
    path: web::Path<i32>,
    request: web::Json<TemplateImageUploadCompleteRequest>,
    req: HttpRequest,
    use_case: web::Data<Arc<ReportGuideTemplateUseCase<T>>>,
    jwt: web::Data<Arc<JwtService>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
) -> Result<HttpResponse, actix_web::Error>
where
    T: crate::domain::template::services::ReportGuideTemplateService + 'static,
{
    let template_id = path.into_inner();
    let user_id = match extract_user_id_from_request(&req, &jwt, &user_repo).await {
        Some(id) if id > 0 => id,
        _ => {
            return Ok(HttpResponse::Unauthorized().json(json!({
                "error": "Unauthorized",
                "message": "Invalid or missing authorization token"
            })));
        }
    };

    // Object Storage에서 파일 URL 생성 (file_path 기반)
    // 실제 구현에서는 Object Storage 설정에 따라 URL을 생성해야 함
    let image_url = format!("https://s3.example.com/{}", request.file_path);

    let add_image_request = AddTemplateImageRequest {
        image_path: request.file_path.clone(),
        image_url,
        file_size: Some(request.file_size),
        mime_type: request.mime_type.clone(),
        display_order: request.display_order,
        is_shared: request.is_shared,
    };

    match use_case
        .add_template_image(template_id, add_image_request, user_id)
        .await
    {
        Ok(image) => Ok(HttpResponse::Ok().json(TemplateImageUploadCompleteResponse {
            success: true,
            message: "Image uploaded and added to template successfully".to_string(),
            image,
        })),
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// 템플릿 이미지 삭제
#[utoipa::path(
    delete,
    path = "/api/report-guide-templates/{template_id}/images/{image_id}",
    responses(
        (status = 200, description = "이미지 삭제 성공"),
        (status = 401, description = "인증 실패"),
        (status = 403, description = "권한 없음"),
        (status = 404, description = "이미지를 찾을 수 없음"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("template_id" = i32, Path, description = "템플릿 ID"),
        ("image_id" = i32, Path, description = "이미지 ID")
    ),
    tag = "report-guide-template"
)]
pub async fn delete_template_image<T>(
    path: web::Path<(i32, i32)>,
    req: HttpRequest,
    use_case: web::Data<Arc<ReportGuideTemplateUseCase<T>>>,
    jwt: web::Data<Arc<JwtService>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
) -> Result<HttpResponse, actix_web::Error>
where
    T: crate::domain::template::services::ReportGuideTemplateService + 'static,
{
    let (template_id, image_id) = path.into_inner();
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
        .delete_template_image(image_id, user_id)
        .await
    {
        Ok(_) => Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "message": "Image deleted successfully"
        }))),
        Err(e) => Ok(handle_service_error(e)),
    }
}

// ========================================
// 사용자 커스텀 템플릿 API
// ========================================

/// 커스텀 템플릿 생성 (원본 복사)
#[utoipa::path(
    post,
    path = "/api/user/custom-report-templates",
    request_body = CreateCustomTemplateFromBaseRequest,
    responses(
        (status = 200, description = "커스텀 템플릿 생성 성공", body = UserCustomReportTemplateResponse),
        (status = 400, description = "잘못된 요청"),
        (status = 401, description = "인증 실패"),
        (status = 404, description = "원본 템플릿을 찾을 수 없음"),
        (status = 500, description = "서버 내부 오류")
    ),
    tag = "user-custom-template"
)]
pub async fn create_custom_template_from_base<T>(
    request: web::Json<CreateCustomTemplateFromBaseRequest>,
    req: HttpRequest,
    use_case: web::Data<Arc<ReportGuideTemplateUseCase<T>>>,
    jwt: web::Data<Arc<JwtService>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
) -> Result<HttpResponse, actix_web::Error>
where
    T: crate::domain::template::services::ReportGuideTemplateService + 'static,
{
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
        .create_custom_template_from_base(user_id, request.into_inner())
        .await
    {
        Ok(template) => Ok(HttpResponse::Ok().json(template)),
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// 커스텀 템플릿 생성 (원본 없이)
#[utoipa::path(
    post,
    path = "/api/user/custom-report-templates/new",
    request_body = CreateCustomTemplateRequest,
    responses(
        (status = 200, description = "커스텀 템플릿 생성 성공", body = UserCustomReportTemplateResponse),
        (status = 400, description = "잘못된 요청"),
        (status = 401, description = "인증 실패"),
        (status = 500, description = "서버 내부 오류")
    ),
    tag = "user-custom-template"
)]
pub async fn create_custom_template<T>(
    request: web::Json<CreateCustomTemplateRequest>,
    req: HttpRequest,
    use_case: web::Data<Arc<ReportGuideTemplateUseCase<T>>>,
    jwt: web::Data<Arc<JwtService>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
) -> Result<HttpResponse, actix_web::Error>
where
    T: crate::domain::template::services::ReportGuideTemplateService + 'static,
{
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
        .create_custom_template(user_id, request.into_inner())
        .await
    {
        Ok(template) => Ok(HttpResponse::Ok().json(template)),
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// 커스텀 템플릿 목록 조회
#[utoipa::path(
    get,
    path = "/api/user/custom-report-templates",
    responses(
        (status = 200, description = "커스텀 템플릿 목록 조회 성공", body = UserCustomTemplateListResponse),
        (status = 401, description = "인증 실패"),
        (status = 500, description = "서버 내부 오류")
    ),
    tag = "user-custom-template"
)]
pub async fn get_custom_templates<T>(
    req: HttpRequest,
    use_case: web::Data<Arc<ReportGuideTemplateUseCase<T>>>,
    jwt: web::Data<Arc<JwtService>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
) -> Result<HttpResponse, actix_web::Error>
where
    T: crate::domain::template::services::ReportGuideTemplateService + 'static,
{
    let user_id = match extract_user_id_from_request(&req, &jwt, &user_repo).await {
        Some(id) if id > 0 => id,
        _ => {
            return Ok(HttpResponse::Unauthorized().json(json!({
                "error": "Unauthorized",
                "message": "Invalid or missing authorization token"
            })));
        }
    };

    match use_case.get_custom_templates_by_user(user_id).await {
        Ok(templates) => Ok(HttpResponse::Ok().json(templates)),
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// 커스텀 템플릿 조회
#[utoipa::path(
    get,
    path = "/api/user/custom-report-templates/{template_id}",
    responses(
        (status = 200, description = "커스텀 템플릿 조회 성공", body = UserCustomReportTemplateResponse),
        (status = 401, description = "인증 실패"),
        (status = 403, description = "권한 없음"),
        (status = 404, description = "템플릿을 찾을 수 없음"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("template_id" = i32, Path, description = "템플릿 ID")
    ),
    tag = "user-custom-template"
)]
pub async fn get_custom_template<T>(
    path: web::Path<i32>,
    req: HttpRequest,
    use_case: web::Data<Arc<ReportGuideTemplateUseCase<T>>>,
    jwt: web::Data<Arc<JwtService>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
) -> Result<HttpResponse, actix_web::Error>
where
    T: crate::domain::template::services::ReportGuideTemplateService + 'static,
{
    let template_id = path.into_inner();
    let user_id = match extract_user_id_from_request(&req, &jwt, &user_repo).await {
        Some(id) if id > 0 => id,
        _ => {
            return Ok(HttpResponse::Unauthorized().json(json!({
                "error": "Unauthorized",
                "message": "Invalid or missing authorization token"
            })));
        }
    };

    match use_case.get_custom_template(template_id, user_id).await {
        Ok(Some(template)) => Ok(HttpResponse::Ok().json(template)),
        Ok(None) => Ok(HttpResponse::NotFound().json(json!({
            "error": "Not Found",
            "message": "Custom template not found"
        }))),
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// 커스텀 템플릿 수정
#[utoipa::path(
    put,
    path = "/api/user/custom-report-templates/{template_id}",
    request_body = UpdateCustomTemplateRequest,
    responses(
        (status = 200, description = "커스텀 템플릿 수정 성공", body = UserCustomReportTemplateResponse),
        (status = 400, description = "잘못된 요청"),
        (status = 401, description = "인증 실패"),
        (status = 403, description = "권한 없음"),
        (status = 404, description = "템플릿을 찾을 수 없음"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("template_id" = i32, Path, description = "템플릿 ID")
    ),
    tag = "user-custom-template"
)]
pub async fn update_custom_template<T>(
    path: web::Path<i32>,
    request: web::Json<UpdateCustomTemplateRequest>,
    req: HttpRequest,
    use_case: web::Data<Arc<ReportGuideTemplateUseCase<T>>>,
    jwt: web::Data<Arc<JwtService>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
) -> Result<HttpResponse, actix_web::Error>
where
    T: crate::domain::template::services::ReportGuideTemplateService + 'static,
{
    let template_id = path.into_inner();
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
        .update_custom_template(template_id, user_id, request.into_inner())
        .await
    {
        Ok(template) => Ok(HttpResponse::Ok().json(template)),
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// 커스텀 템플릿 삭제
#[utoipa::path(
    delete,
    path = "/api/user/custom-report-templates/{template_id}",
    responses(
        (status = 200, description = "커스텀 템플릿 삭제 성공"),
        (status = 401, description = "인증 실패"),
        (status = 403, description = "권한 없음"),
        (status = 404, description = "템플릿을 찾을 수 없음"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("template_id" = i32, Path, description = "템플릿 ID")
    ),
    tag = "user-custom-template"
)]
pub async fn delete_custom_template<T>(
    path: web::Path<i32>,
    req: HttpRequest,
    use_case: web::Data<Arc<ReportGuideTemplateUseCase<T>>>,
    jwt: web::Data<Arc<JwtService>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
) -> Result<HttpResponse, actix_web::Error>
where
    T: crate::domain::template::services::ReportGuideTemplateService + 'static,
{
    let template_id = path.into_inner();
    let user_id = match extract_user_id_from_request(&req, &jwt, &user_repo).await {
        Some(id) if id > 0 => id,
        _ => {
            return Ok(HttpResponse::Unauthorized().json(json!({
                "error": "Unauthorized",
                "message": "Invalid or missing authorization token"
            })));
        }
    };

    match use_case.delete_custom_template(template_id, user_id).await {
        Ok(_) => Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "message": "Custom template deleted successfully"
        }))),
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// 커스텀 템플릿 이미지 추가
#[utoipa::path(
    post,
    path = "/api/user/custom-report-templates/{template_id}/images",
    request_body = AddCustomTemplateImageRequest,
    responses(
        (status = 200, description = "이미지 추가 성공", body = CustomTemplateImageResponse),
        (status = 400, description = "잘못된 요청"),
        (status = 401, description = "인증 실패"),
        (status = 403, description = "권한 없음"),
        (status = 404, description = "템플릿을 찾을 수 없음"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("template_id" = i32, Path, description = "템플릿 ID")
    ),
    tag = "user-custom-template"
)]
pub async fn add_custom_template_image<T>(
    path: web::Path<i32>,
    request: web::Json<AddCustomTemplateImageRequest>,
    req: HttpRequest,
    use_case: web::Data<Arc<ReportGuideTemplateUseCase<T>>>,
    jwt: web::Data<Arc<JwtService>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
) -> Result<HttpResponse, actix_web::Error>
where
    T: crate::domain::template::services::ReportGuideTemplateService + 'static,
{
    let template_id = path.into_inner();
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
        .add_custom_template_image(template_id, user_id, request.into_inner())
        .await
    {
        Ok(image) => Ok(HttpResponse::Ok().json(image)),
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// 커스텀 템플릿 이미지 삭제
#[utoipa::path(
    delete,
    path = "/api/user/custom-report-templates/{template_id}/images/{image_id}",
    responses(
        (status = 200, description = "이미지 삭제 성공"),
        (status = 401, description = "인증 실패"),
        (status = 403, description = "권한 없음"),
        (status = 404, description = "이미지를 찾을 수 없음"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("template_id" = i32, Path, description = "템플릿 ID"),
        ("image_id" = i32, Path, description = "이미지 ID")
    ),
    tag = "user-custom-template"
)]
pub async fn delete_custom_template_image<T>(
    path: web::Path<(i32, i32)>,
    req: HttpRequest,
    use_case: web::Data<Arc<ReportGuideTemplateUseCase<T>>>,
    jwt: web::Data<Arc<JwtService>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
) -> Result<HttpResponse, actix_web::Error>
where
    T: crate::domain::template::services::ReportGuideTemplateService + 'static,
{
    let (template_id, image_id) = path.into_inner();
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
        .delete_custom_template_image(image_id, user_id)
        .await
    {
        Ok(_) => Ok(HttpResponse::Ok().json(json!({
            "success": true,
            "message": "Image deleted successfully"
        }))),
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// 라우팅 설정
pub fn configure_routes<T, SUS>(
    cfg: &mut web::ServiceConfig,
    use_case: Arc<ReportGuideTemplateUseCase<T>>,
    signed_url_service: Arc<SUS>,
    jwt: Arc<JwtService>,
    user_repo: Arc<UserRepositoryImpl>,
) where
    T: crate::domain::template::services::ReportGuideTemplateService + 'static,
    SUS: SignedUrlService + 'static,
{
    cfg.app_data(web::Data::new(use_case))
        .app_data(web::Data::new(signed_url_service))
        .app_data(web::Data::new(jwt))
        .app_data(web::Data::new(user_repo))
        // 원본 템플릿 API
        .service(
            web::scope("/report-guide-templates")
                .route("", web::post().to(create_template::<T>))
                .route("", web::get().to(get_templates::<T>))
                .route("/{template_id}", web::get().to(get_template::<T>))
                .route("/{template_id}", web::put().to(update_template::<T>))
                .route("/{template_id}", web::delete().to(delete_template::<T>))
                .route("/{template_id}/images", web::post().to(add_template_image::<T>))
                .route("/{template_id}/images/upload-url", web::post().to(generate_template_image_upload_url::<T, SUS>))
                .route("/{template_id}/images/complete", web::post().to(complete_template_image_upload::<T>))
                .route(
                    "/{template_id}/images/{image_id}/share",
                    web::put().to(update_image_share_status::<T>),
                )
                .route(
                    "/{template_id}/images/{image_id}",
                    web::delete().to(delete_template_image::<T>),
                ),
        )
        // 사용자 커스텀 템플릿 API
        .service(
            web::scope("/user/custom-report-templates")
                .route("", web::post().to(create_custom_template_from_base::<T>))
                .route("/new", web::post().to(create_custom_template::<T>))
                .route("", web::get().to(get_custom_templates::<T>))
                .route("/{template_id}", web::get().to(get_custom_template::<T>))
                .route("/{template_id}", web::put().to(update_custom_template::<T>))
                .route("/{template_id}", web::delete().to(delete_custom_template::<T>))
                .route(
                    "/{template_id}/images",
                    web::post().to(add_custom_template_image::<T>),
                )
                .route(
                    "/{template_id}/images/{image_id}",
                    web::delete().to(delete_custom_template_image::<T>),
                ),
        );
}

