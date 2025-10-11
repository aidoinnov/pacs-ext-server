use actix_web::{web, HttpResponse, Responder, HttpRequest};
use serde_json::json;
use std::sync::Arc;
use crate::application::dto::mask_group_dto::{
    CreateMaskGroupRequest, UpdateMaskGroupRequest, MaskGroupResponse,
    MaskGroupListResponse, MaskGroupDetailResponse, SignedUrlRequest,
    SignedUrlResponse, CompleteUploadRequest, CompleteUploadResponse
};
use crate::application::use_cases::MaskGroupUseCase;
use crate::domain::ServiceError;

pub struct MaskGroupController<MGS, SUS> 
where
    MGS: crate::domain::services::MaskGroupService + Send + Sync,
    SUS: crate::application::services::SignedUrlService + Send + Sync,
{
    use_case: Arc<MaskGroupUseCase<MGS, SUS>>,
}

impl<MGS, SUS> MaskGroupController<MGS, SUS>
where
    MGS: crate::domain::services::MaskGroupService + Send + Sync,
    SUS: crate::application::services::SignedUrlService + Send + Sync,
{
    pub fn new(use_case: Arc<MaskGroupUseCase<MGS, SUS>>) -> Self {
        Self { use_case }
    }
}

/// 마스크 그룹 생성
#[utoipa::path(
    post,
    path = "/api/annotations/{annotation_id}/mask-groups",
    tag = "mask-groups",
    params(
        ("annotation_id" = i32, Path, description = "Annotation ID")
    ),
    request_body = CreateMaskGroupRequest,
    responses(
        (status = 201, description = "Mask group created successfully", body = MaskGroupResponse),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Annotation not found"),
        (status = 403, description = "Forbidden - insufficient permissions"),
    )
)]
pub async fn create_mask_group<MGS, SUS>(
    path: web::Path<i32>,
    req: web::Json<CreateMaskGroupRequest>,
    use_case: web::Data<Arc<MaskGroupUseCase<MGS, SUS>>>,
    _http_req: HttpRequest,
) -> impl Responder
where
    MGS: crate::domain::services::MaskGroupService + Send + Sync,
    SUS: crate::application::services::SignedUrlService + Send + Sync,
{
    let annotation_id = path.into_inner();
    let mut request = req.into_inner();
    request.annotation_id = annotation_id;

    // X-User-ID 헤더에서 user_id 추출
    let user_id = _http_req
        .headers()
        .get("X-User-ID")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(1); // 기본값은 1 (기존 코드와 호환)

    match use_case.create_mask_group(request, user_id).await {
        Ok(mask_group) => HttpResponse::Created().json(mask_group),
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
    }
}

/// 마스크 그룹 상세 조회
#[utoipa::path(
    get,
    path = "/api/annotations/{annotation_id}/mask-groups/{group_id}",
    tag = "mask-groups",
    params(
        ("annotation_id" = i32, Path, description = "Annotation ID"),
        ("group_id" = i32, Path, description = "Mask Group ID")
    ),
    responses(
        (status = 200, description = "Mask group retrieved successfully", body = MaskGroupDetailResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Mask group not found"),
        (status = 403, description = "Forbidden - insufficient permissions"),
    )
)]
pub async fn get_mask_group<MGS, SUS>(
    path: web::Path<(i32, i32)>,
    use_case: web::Data<Arc<MaskGroupUseCase<MGS, SUS>>>,
    _http_req: HttpRequest,
) -> impl Responder
where
    MGS: crate::domain::services::MaskGroupService + Send + Sync,
    SUS: crate::application::services::SignedUrlService + Send + Sync,
{
    let (annotation_id, group_id) = path.into_inner();
    
    // X-User-ID 헤더에서 user_id 추출
    let user_id = _http_req
        .headers()
        .get("X-User-ID")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(1); // 기본값은 1 (기존 코드와 호환)

    match use_case.get_mask_group(group_id, user_id).await {
        Ok(mask_group) => HttpResponse::Ok().json(mask_group),
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

/// 마스크 그룹 목록 조회
#[utoipa::path(
    get,
    path = "/api/annotations/{annotation_id}/mask-groups",
    tag = "mask-groups",
    params(
        ("annotation_id" = i32, Path, description = "Annotation ID"),
        ("offset" = Option<i64>, Query, description = "Offset for pagination"),
        ("limit" = Option<i64>, Query, description = "Limit for pagination")
    ),
    responses(
        (status = 200, description = "Mask groups retrieved successfully", body = MaskGroupListResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Annotation not found"),
    )
)]
pub async fn list_mask_groups<MGS, SUS>(
    path: web::Path<i32>,
    query: web::Query<serde_json::Value>,
    use_case: web::Data<Arc<MaskGroupUseCase<MGS, SUS>>>,
    _http_req: HttpRequest,
) -> impl Responder
where
    MGS: crate::domain::services::MaskGroupService + Send + Sync,
    SUS: crate::application::services::SignedUrlService + Send + Sync,
{
    let annotation_id = path.into_inner();
    
    // X-User-ID 헤더에서 user_id 추출
    let user_id = _http_req
        .headers()
        .get("X-User-ID")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(1); // 기본값은 1 (기존 코드와 호환)

    // Query parameters 추출
    let offset = query.get("offset").and_then(|v| v.as_str().and_then(|s| s.parse::<i64>().ok()));
    let limit = query.get("limit").and_then(|v| v.as_str().and_then(|s| s.parse::<i64>().ok()));

    match use_case.list_mask_groups(Some(annotation_id), user_id, offset, limit).await {
        Ok(mask_groups) => HttpResponse::Ok().json(mask_groups),
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

/// 마스크 그룹 수정
#[utoipa::path(
    put,
    path = "/api/annotations/{annotation_id}/mask-groups/{group_id}",
    tag = "mask-groups",
    params(
        ("annotation_id" = i32, Path, description = "Annotation ID"),
        ("group_id" = i32, Path, description = "Mask Group ID")
    ),
    request_body = UpdateMaskGroupRequest,
    responses(
        (status = 200, description = "Mask group updated successfully", body = MaskGroupResponse),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Mask group not found"),
        (status = 403, description = "Forbidden - insufficient permissions"),
    )
)]
pub async fn update_mask_group<MGS, SUS>(
    path: web::Path<(i32, i32)>,
    req: web::Json<UpdateMaskGroupRequest>,
    use_case: web::Data<Arc<MaskGroupUseCase<MGS, SUS>>>,
    _http_req: HttpRequest,
) -> impl Responder
where
    MGS: crate::domain::services::MaskGroupService + Send + Sync,
    SUS: crate::application::services::SignedUrlService + Send + Sync,
{
    let (annotation_id, group_id) = path.into_inner();
    
    // X-User-ID 헤더에서 user_id 추출
    let user_id = _http_req
        .headers()
        .get("X-User-ID")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(1); // 기본값은 1 (기존 코드와 호환)

    match use_case.update_mask_group(group_id, req.into_inner(), user_id).await {
        Ok(mask_group) => HttpResponse::Ok().json(mask_group),
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

/// 마스크 그룹 삭제
#[utoipa::path(
    delete,
    path = "/api/annotations/{annotation_id}/mask-groups/{group_id}",
    tag = "mask-groups",
    params(
        ("annotation_id" = i32, Path, description = "Annotation ID"),
        ("group_id" = i32, Path, description = "Mask Group ID")
    ),
    responses(
        (status = 204, description = "Mask group deleted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Mask group not found"),
        (status = 403, description = "Forbidden - insufficient permissions"),
    )
)]
pub async fn delete_mask_group<MGS, SUS>(
    path: web::Path<(i32, i32)>,
    use_case: web::Data<Arc<MaskGroupUseCase<MGS, SUS>>>,
    _http_req: HttpRequest,
) -> impl Responder
where
    MGS: crate::domain::services::MaskGroupService + Send + Sync,
    SUS: crate::application::services::SignedUrlService + Send + Sync,
{
    let (annotation_id, group_id) = path.into_inner();
    
    // X-User-ID 헤더에서 user_id 추출
    let user_id = _http_req
        .headers()
        .get("X-User-ID")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(1); // 기본값은 1 (기존 코드와 호환)

    match use_case.delete_mask_group(group_id, user_id).await {
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

/// Signed URL 생성 (업로드용)
#[utoipa::path(
    post,
    path = "/api/annotations/{annotation_id}/mask-groups/{group_id}/signed-url",
    tag = "mask-groups",
    params(
        ("annotation_id" = i32, Path, description = "Annotation ID"),
        ("group_id" = i32, Path, description = "Mask Group ID")
    ),
    request_body = SignedUrlRequest,
    responses(
        (status = 200, description = "Signed URL generated successfully", body = SignedUrlResponse),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Mask group not found"),
        (status = 403, description = "Forbidden - insufficient permissions"),
    )
)]
pub async fn generate_upload_url<MGS, SUS>(
    path: web::Path<(i32, i32)>,
    req: web::Json<SignedUrlRequest>,
    use_case: web::Data<Arc<MaskGroupUseCase<MGS, SUS>>>,
    _http_req: HttpRequest,
) -> impl Responder
where
    MGS: crate::domain::services::MaskGroupService + Send + Sync,
    SUS: crate::application::services::SignedUrlService + Send + Sync,
{
    let (annotation_id, group_id) = path.into_inner();
    let mut request = req.into_inner();
    request.mask_group_id = group_id;
    
    // X-User-ID 헤더에서 user_id 추출
    let user_id = _http_req
        .headers()
        .get("X-User-ID")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(1); // 기본값은 1 (기존 코드와 호환)

    match use_case.generate_upload_url(request, user_id).await {
        Ok(signed_url) => HttpResponse::Ok().json(signed_url),
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

/// 업로드 완료 처리
#[utoipa::path(
    post,
    path = "/api/annotations/{annotation_id}/mask-groups/{group_id}/complete",
    tag = "mask-groups",
    params(
        ("annotation_id" = i32, Path, description = "Annotation ID"),
        ("group_id" = i32, Path, description = "Mask Group ID")
    ),
    request_body = CompleteUploadRequest,
    responses(
        (status = 200, description = "Upload completed successfully", body = CompleteUploadResponse),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Mask group not found"),
        (status = 403, description = "Forbidden - insufficient permissions"),
    )
)]
pub async fn complete_upload<MGS, SUS>(
    path: web::Path<(i32, i32)>,
    req: web::Json<CompleteUploadRequest>,
    use_case: web::Data<Arc<MaskGroupUseCase<MGS, SUS>>>,
    _http_req: HttpRequest,
) -> impl Responder
where
    MGS: crate::domain::services::MaskGroupService + Send + Sync,
    SUS: crate::application::services::SignedUrlService + Send + Sync,
{
    let (annotation_id, group_id) = path.into_inner();
    let mut request = req.into_inner();
    request.mask_group_id = group_id;
    
    // X-User-ID 헤더에서 user_id 추출
    let user_id = _http_req
        .headers()
        .get("X-User-ID")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(1); // 기본값은 1 (기존 코드와 호환)

    match use_case.complete_upload(request, user_id).await {
        Ok(response) => HttpResponse::Ok().json(response),
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

/// 라우트 설정
pub fn configure_routes<MGS, SUS>(
    cfg: &mut web::ServiceConfig,
    use_case: Arc<MaskGroupUseCase<MGS, SUS>>,
)
where
    MGS: crate::domain::services::MaskGroupService + Send + Sync + 'static,
    SUS: crate::application::services::SignedUrlService + Send + Sync + 'static,
{
    cfg.app_data(web::Data::new(use_case))
        .service(
            web::scope("/api/annotations/{annotation_id}/mask-groups")
                .route("", web::post().to(create_mask_group::<MGS, SUS>))
                .route("", web::get().to(list_mask_groups::<MGS, SUS>))
                .route("/{group_id}", web::get().to(get_mask_group::<MGS, SUS>))
                .route("/{group_id}", web::put().to(update_mask_group::<MGS, SUS>))
                .route("/{group_id}", web::delete().to(delete_mask_group::<MGS, SUS>))
                .route("/{group_id}/upload-url", web::post().to(generate_upload_url::<MGS, SUS>))
                .route("/{group_id}/complete-upload", web::post().to(complete_upload::<MGS, SUS>))
        );
}
