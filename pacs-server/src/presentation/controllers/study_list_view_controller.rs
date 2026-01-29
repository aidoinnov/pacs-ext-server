//! Study List View API 컨트롤러

use actix_web::{web, HttpRequest, HttpResponse, Result};
use actix_web::http::header::{CacheControl, CacheDirective, EntityTag, IF_NONE_MATCH};
use serde_json::json;
use std::sync::Arc;

use crate::application::dto::study_list_view_dto::*;
use crate::application::use_cases::StudyListViewUseCase;
use crate::domain::repositories::StudyListViewRepository;
use crate::domain::ServiceError;
use crate::infrastructure::auth::{extract_user_id_from_request, JwtService};
use crate::infrastructure::repositories::{StudyListViewRepositoryImpl, UserRepositoryImpl};

/// ServiceError를 HttpResponse로 변환
fn handle_service_error(error: ServiceError) -> HttpResponse {
    match error {
        ServiceError::NotFound(msg) => HttpResponse::NotFound().json(json!({
            "error": "Not Found",
            "message": msg
        })),
        ServiceError::ValidationError(msg) | ServiceError::ValidationFailed(msg) => {
            HttpResponse::BadRequest().json(json!({
                "error": "Validation Error",
                "message": msg
            }))
        }
        ServiceError::Unauthorized(msg) => HttpResponse::Unauthorized().json(json!({
            "error": "Unauthorized",
            "message": msg
        })),
        ServiceError::Forbidden(msg) => HttpResponse::Forbidden().json(json!({
            "error": "Forbidden",
            "message": msg
        })),
        ServiceError::AlreadyExists(msg) => HttpResponse::Conflict().json(json!({
            "error": "Already Exists",
            "message": msg
        })),
        _ => HttpResponse::InternalServerError().json(json!({
            "error": "Internal Server Error",
            "message": "An unexpected error occurred"
        })),
    }
}

// ========================================
// View CRUD API
// ========================================

/// View 목록 조회
#[utoipa::path(
    get,
    path = "/api/study-list-views",
    responses(
        (status = 200, description = "View 목록 조회 성공", body = ViewListResponse),
        (status = 304, description = "Not Modified (ETag 일치)"),
        (status = 401, description = "인증 실패"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("scopeType" = Option<String>, Query, description = "범위 타입 필터"),
        ("scopeId" = Option<String>, Query, description = "범위 ID 필터")
    ),
    tag = "study-list-views"
)]
pub async fn list_views<R>(
    query: web::Query<ViewListQuery>,
    http_req: HttpRequest,
    use_case: web::Data<Arc<StudyListViewUseCase<R>>>,
    jwt: web::Data<Arc<JwtService>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
) -> Result<HttpResponse, actix_web::Error>
where
    R: StudyListViewRepository + 'static,
{
    let user_id = extract_user_id_from_request(&http_req, &jwt, &user_repo).await;
    let user_id_str = user_id.map(|id| id.to_string());

    // 1. 먼저 views_updated_at 조회 (빠른 쿼리)
    let views_updated_at = match use_case.get_views_updated_at(&query, user_id_str.as_deref()).await {
        Ok(updated_at) => updated_at,
        Err(e) => {
            tracing::error!("Failed to get views updated_at: {:?}", e);
            return Ok(handle_service_error(e));
        }
    };

    // 2. ETag 생성
    let etag = views_updated_at.timestamp().to_string();
    let entity_tag = EntityTag::new_weak(etag.clone());

    // 3. If-None-Match 헤더 확인
    if let Some(if_none_match) = http_req.headers().get(IF_NONE_MATCH) {
        if let Ok(client_etag_str) = if_none_match.to_str() {
            if let Ok(client_entity_tag) = client_etag_str.parse::<EntityTag>() {
                if client_entity_tag.weak_eq(&entity_tag) {
                    tracing::debug!("ETag match - returning 304 Not Modified");
                    return Ok(HttpResponse::NotModified()
                        .insert_header(CacheControl(vec![
                            CacheDirective::Private,
                            CacheDirective::MaxAge(60),
                        ]))
                        .insert_header(actix_web::http::header::ETag(entity_tag))
                        .finish());
                }
            }
        }
    }

    // 4. ETag 불일치 시에만 실제 데이터 조회
    match use_case.list_views(&query, user_id_str.as_deref()).await {
        Ok(response) => Ok(HttpResponse::Ok()
            .insert_header(CacheControl(vec![
                CacheDirective::Private,
                CacheDirective::MaxAge(60),
            ]))
            .insert_header(actix_web::http::header::ETag(entity_tag))
            .json(response)),
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// View 상세 조회
#[utoipa::path(
    get,
    path = "/api/study-list-views/{view_id}",
    responses(
        (status = 200, description = "View 상세 조회 성공", body = ViewDetailResponse),
        (status = 401, description = "인증 실패"),
        (status = 404, description = "View를 찾을 수 없음"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("view_id" = String, Path, description = "View ID")
    ),
    tag = "study-list-views"
)]
pub async fn get_view<R>(
    path: web::Path<String>,
    use_case: web::Data<Arc<StudyListViewUseCase<R>>>,
) -> Result<HttpResponse, actix_web::Error>
where
    R: StudyListViewRepository + 'static,
{
    let view_id = path.into_inner();

    match use_case.get_view(&view_id).await {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// View 생성
#[utoipa::path(
    post,
    path = "/api/study-list-views",
    request_body = CreateViewRequest,
    responses(
        (status = 201, description = "View 생성 성공", body = ViewResponse),
        (status = 400, description = "잘못된 요청"),
        (status = 401, description = "인증 실패"),
        (status = 409, description = "이미 존재하는 View"),
        (status = 500, description = "서버 내부 오류")
    ),
    tag = "study-list-views"
)]
pub async fn create_view<R>(
    request: web::Json<CreateViewRequest>,
    req: HttpRequest,
    use_case: web::Data<Arc<StudyListViewUseCase<R>>>,
    jwt: web::Data<Arc<JwtService>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
) -> Result<HttpResponse, actix_web::Error>
where
    R: StudyListViewRepository + 'static,
{
    let user_id = match extract_user_id_from_request(&req, &jwt, &user_repo).await {
        Some(id) => id.to_string(),
        None => {
            return Ok(HttpResponse::Unauthorized().json(json!({
                "error": "Unauthorized",
                "message": "User authentication required"
            })));
        }
    };

    match use_case.create_view(&request, &user_id).await {
        Ok(response) => Ok(HttpResponse::Created().json(response)),
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// View 수정
#[utoipa::path(
    put,
    path = "/api/study-list-views/{view_id}",
    request_body = UpdateViewRequest,
    responses(
        (status = 200, description = "View 수정 성공", body = ViewResponse),
        (status = 400, description = "잘못된 요청"),
        (status = 401, description = "인증 실패"),
        (status = 403, description = "시스템 View는 수정 불가"),
        (status = 404, description = "View를 찾을 수 없음"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("view_id" = String, Path, description = "View ID")
    ),
    tag = "study-list-views"
)]
pub async fn update_view<R>(
    path: web::Path<String>,
    request: web::Json<UpdateViewRequest>,
    use_case: web::Data<Arc<StudyListViewUseCase<R>>>,
) -> Result<HttpResponse, actix_web::Error>
where
    R: StudyListViewRepository + 'static,
{
    let view_id = path.into_inner();

    match use_case.update_view(&view_id, &request).await {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// View 삭제
#[utoipa::path(
    delete,
    path = "/api/study-list-views/{view_id}",
    responses(
        (status = 204, description = "View 삭제 성공"),
        (status = 401, description = "인증 실패"),
        (status = 403, description = "시스템 View는 삭제 불가"),
        (status = 404, description = "View를 찾을 수 없음"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("view_id" = String, Path, description = "View ID")
    ),
    tag = "study-list-views"
)]
pub async fn delete_view<R>(
    path: web::Path<String>,
    use_case: web::Data<Arc<StudyListViewUseCase<R>>>,
) -> Result<HttpResponse, actix_web::Error>
where
    R: StudyListViewRepository + 'static,
{
    let view_id = path.into_inner();

    match use_case.delete_view(&view_id).await {
        Ok(()) => Ok(HttpResponse::NoContent().finish()),
        Err(e) => Ok(handle_service_error(e)),
    }
}

// ========================================
// Field Definition API
// ========================================

/// 필드 정의 목록 조회
#[utoipa::path(
    get,
    path = "/api/study-list-views/field-defs",
    responses(
        (status = 200, description = "필드 정의 목록 조회 성공", body = FieldDefListResponse),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("source" = Option<String>, Query, description = "소스 필터 (dicom, extension)"),
        ("level" = Option<String>, Query, description = "레벨 필터 (study, series, instance)"),
        ("sortable" = Option<bool>, Query, description = "정렬 가능 여부 필터"),
        ("filterable" = Option<bool>, Query, description = "필터 가능 여부 필터")
    ),
    tag = "study-list-views"
)]
pub async fn list_field_defs<R>(
    query: web::Query<FieldDefListQuery>,
    use_case: web::Data<Arc<StudyListViewUseCase<R>>>,
) -> Result<HttpResponse, actix_web::Error>
where
    R: StudyListViewRepository + 'static,
{
    match use_case.list_field_defs(&query).await {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// 라우트 설정
pub fn configure_routes(
    cfg: &mut web::ServiceConfig,
    use_case: Arc<StudyListViewUseCase<StudyListViewRepositoryImpl>>,
    jwt_service: Arc<JwtService>,
    user_repo: Arc<UserRepositoryImpl>,
) {
    cfg.app_data(web::Data::new(use_case));
    cfg.app_data(web::Data::new(jwt_service));
    cfg.app_data(web::Data::new(user_repo));
    cfg.service(
        web::scope("/study-list-views")
            .route("/field-defs", web::get().to(list_field_defs::<StudyListViewRepositoryImpl>))
            .route("", web::get().to(list_views::<StudyListViewRepositoryImpl>))
            .route("", web::post().to(create_view::<StudyListViewRepositoryImpl>))
            .route("/{view_id}", web::get().to(get_view::<StudyListViewRepositoryImpl>))
            .route("/{view_id}", web::put().to(update_view::<StudyListViewRepositoryImpl>))
            .route("/{view_id}", web::delete().to(delete_view::<StudyListViewRepositoryImpl>)),
    );
}
