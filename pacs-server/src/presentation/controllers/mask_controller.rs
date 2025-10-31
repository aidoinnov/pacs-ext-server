#![allow(dead_code, unused_imports, unused_variables)]
use crate::application::dto::mask_dto::{
    CreateMaskRequest, DownloadUrlRequest, DownloadUrlResponse, MaskListResponse, MaskResponse,
    MaskStatsResponse, UpdateMaskRequest,
};
use crate::application::use_cases::MaskUseCase;
use crate::domain::ServiceError;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use serde_json::json;
use std::sync::Arc;

pub struct MaskController<MS, MGS, SUS>
where
    MS: crate::domain::services::MaskService + Send + Sync,
    MGS: crate::domain::services::MaskGroupService + Send + Sync,
    SUS: crate::application::services::SignedUrlService + Send + Sync,
{
    use_case: Arc<MaskUseCase<MS, MGS, SUS>>,
}

impl<MS, MGS, SUS> MaskController<MS, MGS, SUS>
where
    MS: crate::domain::services::MaskService + Send + Sync,
    MGS: crate::domain::services::MaskGroupService + Send + Sync,
    SUS: crate::application::services::SignedUrlService + Send + Sync,
{
    pub fn new(use_case: Arc<MaskUseCase<MS, MGS, SUS>>) -> Self {
        Self { use_case }
    }
}

/// 마스크 생성
#[utoipa::path(
    post,
    path = "/api/annotations/{annotation_id}/mask-groups/{group_id}/masks",
    tag = "masks",
    params(
        ("annotation_id" = i32, Path, description = "Annotation ID"),
        ("group_id" = i32, Path, description = "Mask Group ID")
    ),
    request_body = CreateMaskRequest,
    responses(
        (status = 201, description = "Mask created successfully", body = MaskResponse),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Mask group not found"),
        (status = 403, description = "Forbidden - insufficient permissions"),
    )
)]
pub async fn create_mask<MS, MGS, SUS>(
    path: web::Path<(i32, i32)>,
    req: web::Json<CreateMaskRequest>,
    use_case: web::Data<Arc<MaskUseCase<MS, MGS, SUS>>>,
    http_req: HttpRequest,
) -> impl Responder
where
    MS: crate::domain::services::MaskService + Send + Sync,
    MGS: crate::domain::services::MaskGroupService + Send + Sync,
    SUS: crate::application::services::SignedUrlService + Send + Sync,
{
    let (annotation_id, group_id) = path.into_inner();
    let mut request = req.into_inner();
    request.mask_group_id = group_id;

    // 테스트에서 X-User-ID 헤더로 사용자 ID를 전달받음
    let user_id = http_req
        .headers()
        .get("X-User-ID")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(1); // 기본값은 1 (기존 코드와 호환)

    match use_case.create_mask(request, user_id).await {
        Ok(mask) => HttpResponse::Created().json(mask),
        Err(ServiceError::NotFound(msg)) => HttpResponse::NotFound().json(json!({
            "error": "Not Found",
            "message": msg
        })),
        Err(ServiceError::Unauthorized(msg)) => HttpResponse::Unauthorized().json(json!({
            "error": "Unauthorized",
            "message": msg
        })),
        Err(ServiceError::ValidationError(msg)) => HttpResponse::BadRequest().json(json!({
            "error": "Validation Error",
            "message": msg
        })),
        Err(ServiceError::AlreadyExists(msg)) => HttpResponse::Conflict().json(json!({
            "error": "Already Exists",
            "message": msg
        })),
        Err(ServiceError::DatabaseError(msg)) => HttpResponse::InternalServerError().json(json!({
            "error": "Database Error",
            "message": msg
        })),
        _ => HttpResponse::InternalServerError().json(json!({
            "error": "Internal Server Error",
            "message": "An unexpected error occurred"
        })),
    }
}

/// 마스크 조회
#[utoipa::path(
    get,
    path = "/api/annotations/{annotation_id}/mask-groups/{group_id}/masks/{mask_id}",
    tag = "masks",
    params(
        ("annotation_id" = i32, Path, description = "Annotation ID"),
        ("group_id" = i32, Path, description = "Mask Group ID"),
        ("mask_id" = i32, Path, description = "Mask ID")
    ),
    responses(
        (status = 200, description = "Mask retrieved successfully", body = MaskResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Mask not found"),
        (status = 403, description = "Forbidden - insufficient permissions"),
    )
)]
pub async fn get_mask<MS, MGS, SUS>(
    path: web::Path<(i32, i32, i32)>,
    use_case: web::Data<Arc<MaskUseCase<MS, MGS, SUS>>>,
    http_req: HttpRequest,
) -> impl Responder
where
    MS: crate::domain::services::MaskService + Send + Sync,
    MGS: crate::domain::services::MaskGroupService + Send + Sync,
    SUS: crate::application::services::SignedUrlService + Send + Sync,
{
    let (annotation_id, group_id, mask_id) = path.into_inner();

    // 테스트에서 X-User-ID 헤더로 사용자 ID를 전달받음
    let user_id = http_req
        .headers()
        .get("X-User-ID")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(1); // 기본값은 1 (기존 코드와 호환)

    match use_case.get_mask(mask_id, user_id).await {
        Ok(mask) => HttpResponse::Ok().json(mask),
        Err(ServiceError::NotFound(msg)) => HttpResponse::NotFound().json(json!({
            "error": "Not Found",
            "message": msg
        })),
        Err(ServiceError::Unauthorized(msg)) => HttpResponse::Unauthorized().json(json!({
            "error": "Unauthorized",
            "message": msg
        })),
        Err(ServiceError::DatabaseError(msg)) => HttpResponse::InternalServerError().json(json!({
            "error": "Database Error",
            "message": msg
        })),
        _ => HttpResponse::InternalServerError().json(json!({
            "error": "Internal Server Error",
            "message": "An unexpected error occurred"
        })),
    }
}

/// 마스크 목록 조회
#[utoipa::path(
    get,
    path = "/api/annotations/{annotation_id}/mask-groups/{group_id}/masks",
    tag = "masks",
    params(
        ("annotation_id" = i32, Path, description = "Annotation ID"),
        ("group_id" = i32, Path, description = "Mask Group ID"),
        ("offset" = Option<i64>, Query, description = "Offset for pagination"),
        ("limit" = Option<i64>, Query, description = "Limit for pagination")
    ),
    responses(
        (status = 200, description = "Masks retrieved successfully", body = MaskListResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Mask group not found"),
    )
)]
pub async fn list_masks<MS, MGS, SUS>(
    path: web::Path<(i32, i32)>,
    query: web::Query<serde_json::Value>,
    use_case: web::Data<Arc<MaskUseCase<MS, MGS, SUS>>>,
    http_req: HttpRequest,
) -> impl Responder
where
    MS: crate::domain::services::MaskService + Send + Sync,
    MGS: crate::domain::services::MaskGroupService + Send + Sync,
    SUS: crate::application::services::SignedUrlService + Send + Sync,
{
    let (annotation_id, group_id) = path.into_inner();
    println!(
        "DEBUG: list_masks called with annotation_id={}, group_id={}",
        annotation_id, group_id
    );

    // 테스트에서 X-User-ID 헤더로 사용자 ID를 전달받음
    let user_id = http_req
        .headers()
        .get("X-User-ID")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(1); // 기본값은 1 (기존 코드와 호환)

    // Query parameters 추출
    let offset = query
        .get("offset")
        .and_then(|v| v.as_str().and_then(|s| s.parse::<i64>().ok()));
    let limit = query
        .get("limit")
        .and_then(|v| v.as_str().and_then(|s| s.parse::<i64>().ok()));

    match use_case
        .list_masks(Some(group_id), user_id, offset, limit)
        .await
    {
        Ok(masks) => HttpResponse::Ok().json(masks),
        Err(ServiceError::NotFound(msg)) => HttpResponse::NotFound().json(json!({
            "error": "Not Found",
            "message": msg
        })),
        Err(ServiceError::Unauthorized(msg)) => HttpResponse::Unauthorized().json(json!({
            "error": "Unauthorized",
            "message": msg
        })),
        Err(ServiceError::DatabaseError(msg)) => HttpResponse::InternalServerError().json(json!({
            "error": "Database Error",
            "message": msg
        })),
        _ => HttpResponse::InternalServerError().json(json!({
            "error": "Internal Server Error",
            "message": "An unexpected error occurred"
        })),
    }
}

/// 마스크 수정
#[utoipa::path(
    put,
    path = "/api/annotations/{annotation_id}/mask-groups/{group_id}/masks/{mask_id}",
    tag = "masks",
    params(
        ("annotation_id" = i32, Path, description = "Annotation ID"),
        ("group_id" = i32, Path, description = "Mask Group ID"),
        ("mask_id" = i32, Path, description = "Mask ID")
    ),
    request_body = UpdateMaskRequest,
    responses(
        (status = 200, description = "Mask updated successfully", body = MaskResponse),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Mask not found"),
        (status = 403, description = "Forbidden - insufficient permissions"),
    )
)]
pub async fn update_mask<MS, MGS, SUS>(
    path: web::Path<(i32, i32, i32)>,
    req: web::Json<UpdateMaskRequest>,
    use_case: web::Data<Arc<MaskUseCase<MS, MGS, SUS>>>,
    http_req: HttpRequest,
) -> impl Responder
where
    MS: crate::domain::services::MaskService + Send + Sync,
    MGS: crate::domain::services::MaskGroupService + Send + Sync,
    SUS: crate::application::services::SignedUrlService + Send + Sync,
{
    let (annotation_id, group_id, mask_id) = path.into_inner();

    // 테스트에서 X-User-ID 헤더로 사용자 ID를 전달받음
    let user_id = http_req
        .headers()
        .get("X-User-ID")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(1); // 기본값은 1 (기존 코드와 호환)

    match use_case
        .update_mask(mask_id, req.into_inner(), user_id)
        .await
    {
        Ok(mask) => HttpResponse::Ok().json(mask),
        Err(ServiceError::NotFound(msg)) => HttpResponse::NotFound().json(json!({
            "error": "Not Found",
            "message": msg
        })),
        Err(ServiceError::Unauthorized(msg)) => HttpResponse::Unauthorized().json(json!({
            "error": "Unauthorized",
            "message": msg
        })),
        Err(ServiceError::ValidationError(msg)) => HttpResponse::BadRequest().json(json!({
            "error": "Validation Error",
            "message": msg
        })),
        Err(ServiceError::AlreadyExists(msg)) => HttpResponse::Conflict().json(json!({
            "error": "Already Exists",
            "message": msg
        })),
        Err(ServiceError::DatabaseError(msg)) => HttpResponse::InternalServerError().json(json!({
            "error": "Database Error",
            "message": msg
        })),
        _ => HttpResponse::InternalServerError().json(json!({
            "error": "Internal Server Error",
            "message": "An unexpected error occurred"
        })),
    }
}

/// 마스크 삭제
#[utoipa::path(
    delete,
    path = "/api/annotations/{annotation_id}/mask-groups/{group_id}/masks/{mask_id}",
    tag = "masks",
    params(
        ("annotation_id" = i32, Path, description = "Annotation ID"),
        ("group_id" = i32, Path, description = "Mask Group ID"),
        ("mask_id" = i32, Path, description = "Mask ID")
    ),
    responses(
        (status = 204, description = "Mask deleted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Mask not found"),
        (status = 403, description = "Forbidden - insufficient permissions"),
    )
)]
pub async fn delete_mask<MS, MGS, SUS>(
    path: web::Path<(i32, i32, i32)>,
    use_case: web::Data<Arc<MaskUseCase<MS, MGS, SUS>>>,
    http_req: HttpRequest,
) -> impl Responder
where
    MS: crate::domain::services::MaskService + Send + Sync,
    MGS: crate::domain::services::MaskGroupService + Send + Sync,
    SUS: crate::application::services::SignedUrlService + Send + Sync,
{
    let (annotation_id, group_id, mask_id) = path.into_inner();

    // 테스트에서 X-User-ID 헤더로 사용자 ID를 전달받음
    let user_id = http_req
        .headers()
        .get("X-User-ID")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(1); // 기본값은 1 (기존 코드와 호환)

    match use_case.delete_mask(mask_id, user_id).await {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(ServiceError::NotFound(msg)) => HttpResponse::NotFound().json(json!({
            "error": "Not Found",
            "message": msg
        })),
        Err(ServiceError::Unauthorized(msg)) => HttpResponse::Unauthorized().json(json!({
            "error": "Unauthorized",
            "message": msg
        })),
        Err(ServiceError::DatabaseError(msg)) => HttpResponse::InternalServerError().json(json!({
            "error": "Database Error",
            "message": msg
        })),
        _ => HttpResponse::InternalServerError().json(json!({
            "error": "Internal Server Error",
            "message": "An unexpected error occurred"
        })),
    }
}

/// 다운로드 URL 생성
#[utoipa::path(
    post,
    path = "/api/annotations/{annotation_id}/mask-groups/{group_id}/masks/{mask_id}/download-url",
    tag = "masks",
    params(
        ("annotation_id" = i32, Path, description = "Annotation ID"),
        ("group_id" = i32, Path, description = "Mask Group ID"),
        ("mask_id" = i32, Path, description = "Mask ID")
    ),
    request_body = DownloadUrlRequest,
    responses(
        (status = 200, description = "Download URL generated successfully", body = DownloadUrlResponse),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Mask not found"),
        (status = 403, description = "Forbidden - insufficient permissions"),
    )
)]
pub async fn generate_download_url<MS, MGS, SUS>(
    path: web::Path<(i32, i32, i32)>,
    req: web::Json<DownloadUrlRequest>,
    use_case: web::Data<Arc<MaskUseCase<MS, MGS, SUS>>>,
    http_req: HttpRequest,
) -> impl Responder
where
    MS: crate::domain::services::MaskService + Send + Sync,
    MGS: crate::domain::services::MaskGroupService + Send + Sync,
    SUS: crate::application::services::SignedUrlService + Send + Sync,
{
    let (annotation_id, group_id, mask_id) = path.into_inner();
    let mut request = req.into_inner();
    request.mask_id = mask_id;

    // 테스트에서 X-User-ID 헤더로 사용자 ID를 전달받음
    let user_id = http_req
        .headers()
        .get("X-User-ID")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(1); // 기본값은 1 (기존 코드와 호환)

    match use_case.generate_download_url(request, user_id).await {
        Ok(download_url) => HttpResponse::Ok().json(download_url),
        Err(ServiceError::NotFound(msg)) => HttpResponse::NotFound().json(json!({
            "error": "Not Found",
            "message": msg
        })),
        Err(ServiceError::Unauthorized(msg)) => HttpResponse::Unauthorized().json(json!({
            "error": "Unauthorized",
            "message": msg
        })),
        Err(ServiceError::ValidationError(msg)) => HttpResponse::BadRequest().json(json!({
            "error": "Validation Error",
            "message": msg
        })),
        Err(ServiceError::AlreadyExists(msg)) => HttpResponse::Conflict().json(json!({
            "error": "Already Exists",
            "message": msg
        })),
        Err(ServiceError::DatabaseError(msg)) => HttpResponse::InternalServerError().json(json!({
            "error": "Database Error",
            "message": msg
        })),
        _ => HttpResponse::InternalServerError().json(json!({
            "error": "Internal Server Error",
            "message": "An unexpected error occurred"
        })),
    }
}

/// 마스크 통계 조회
#[utoipa::path(
    get,
    path = "/api/annotations/{annotation_id}/mask-groups/{group_id}/masks/stats",
    tag = "masks",
    params(
        ("annotation_id" = i32, Path, description = "Annotation ID"),
        ("group_id" = i32, Path, description = "Mask Group ID")
    ),
    responses(
        (status = 200, description = "Mask statistics retrieved successfully", body = MaskStatsResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Mask group not found"),
        (status = 403, description = "Forbidden - insufficient permissions"),
    )
)]
pub async fn get_mask_stats<MS, MGS, SUS>(
    path: web::Path<(i32, i32)>,
    use_case: web::Data<Arc<MaskUseCase<MS, MGS, SUS>>>,
    http_req: HttpRequest,
) -> impl Responder
where
    MS: crate::domain::services::MaskService + Send + Sync,
    MGS: crate::domain::services::MaskGroupService + Send + Sync,
    SUS: crate::application::services::SignedUrlService + Send + Sync,
{
    let (annotation_id, group_id) = path.into_inner();
    println!(
        "DEBUG: get_mask_stats called with annotation_id={}, group_id={}",
        annotation_id, group_id
    );

    // 테스트에서 X-User-ID 헤더로 사용자 ID를 전달받음
    let user_id = http_req
        .headers()
        .get("X-User-ID")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(1); // 기본값은 1 (기존 코드와 호환)

    println!("DEBUG: get_mask_stats - user_id from header = {}", user_id);

    match use_case.get_mask_stats(Some(group_id), user_id).await {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(ServiceError::NotFound(msg)) => HttpResponse::NotFound().json(json!({
            "error": "Not Found",
            "message": msg
        })),
        Err(ServiceError::Unauthorized(msg)) => HttpResponse::Unauthorized().json(json!({
            "error": "Unauthorized",
            "message": msg
        })),
        Err(ServiceError::DatabaseError(msg)) => HttpResponse::InternalServerError().json(json!({
            "error": "Database Error",
            "message": msg
        })),
        _ => HttpResponse::InternalServerError().json(json!({
            "error": "Internal Server Error",
            "message": "An unexpected error occurred"
        })),
    }
}

/// 라우트 설정
pub fn configure_routes<MS, MGS, SUS>(
    cfg: &mut web::ServiceConfig,
    use_case: Arc<MaskUseCase<MS, MGS, SUS>>,
) where
    MS: crate::domain::services::MaskService + Send + Sync + 'static,
    MGS: crate::domain::services::MaskGroupService + Send + Sync + 'static,
    SUS: crate::application::services::SignedUrlService + Send + Sync + 'static,
{
    cfg.app_data(web::Data::new(use_case)).service(
        web::scope("/annotations/{annotation_id}/mask-groups/{group_id}/masks")
            .route("", web::post().to(create_mask::<MS, MGS, SUS>))
            .route("", web::get().to(list_masks::<MS, MGS, SUS>))
            .route("/stats", web::get().to(get_mask_stats::<MS, MGS, SUS>))
            .route("/{mask_id}", web::get().to(get_mask::<MS, MGS, SUS>))
            .route("/{mask_id}", web::put().to(update_mask::<MS, MGS, SUS>))
            .route("/{mask_id}", web::delete().to(delete_mask::<MS, MGS, SUS>))
            .route(
                "/{mask_id}/download-url",
                web::post().to(generate_download_url::<MS, MGS, SUS>),
            ),
    );
}
