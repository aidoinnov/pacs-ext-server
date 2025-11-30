#![allow(dead_code, unused_imports, unused_variables)]
use crate::application::dto::annotation_dto::{
    AnnotationListResponse, AnnotationPermissionsResponse, AnnotationResponse, CreateAnnotationRequest, UpdateAnnotationRequest,
};
use crate::application::use_cases::AnnotationUseCase;
use crate::domain::services::annotation_service::AnnotationService;
use crate::domain::services::{AnnotationServiceImpl, AccessControlServiceImpl};
use crate::domain::ServiceError;
use crate::infrastructure::repositories::{
    AnnotationRepositoryImpl, ProjectRepositoryImpl, UserRepositoryImpl,
    AccessLogRepositoryImpl, RoleRepositoryImpl, PermissionRepositoryImpl,
};
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use serde_json::json;
use std::sync::Arc;

pub struct AnnotationController;

impl AnnotationController {
    pub fn new() -> Self {
        Self
    }

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
            ServiceError::VersionConflict { current_version, client_version } => {
                HttpResponse::Conflict().json(json!({
                    "error": "Version Conflict",
                    "message": "Annotation has been modified by another user",
                    "current_version": current_version,
                    "client_version": client_version
                }))
            }
            _ => HttpResponse::InternalServerError().json(json!({
                "error": "Internal Server Error",
                "message": "An unexpected error occurred"
            })),
        }
    }

    /// 개발 모드에서 user_id를 추출하고, 없으면 Unauthorized 응답을 반환
    fn extract_user_id_or_unauthorized(req: &HttpRequest) -> Result<i32, HttpResponse> {
        match Self::extract_user_id_for_dev_mode_impl(req) {
            Some(id) => Ok(id),
            None => Err(HttpResponse::Unauthorized().json(json!({
                "error": "Unauthorized",
                "message": "User ID is required"
            }))),
        }
    }

    /// 쿼리 파라미터에서 project_id를 추출하고 검증
    fn validate_project_id(
        query: &std::collections::HashMap<String, String>,
    ) -> Result<i32, HttpResponse> {
        match query
            .get("project_id")
            .and_then(|v| v.parse::<i32>().ok())
        {
            Some(id) if id > 0 => Ok(id),
            _ => Err(HttpResponse::BadRequest().json(json!({
                "error": "Bad Request",
                "message": "project_id is required and must be greater than 0"
            }))),
        }
    }

    /// 쿼리 파라미터에서 target_user_id를 추출하고 기본값 설정
    fn extract_target_user_id(
        query: &std::collections::HashMap<String, String>,
        default_user_id: i32,
    ) -> i32 {
        query
            .get("user_id")
            .and_then(|v| v.parse::<i32>().ok())
            .unwrap_or(default_user_id)
    }

    /// 개발 모드에서 user_id를 추출하는 헬퍼 함수
    ///
    /// 개발 모드(`APP_ENV=development` 또는 `RUN_ENV=development`)에서만 동작합니다.
    /// 쿼리 파라미터 `?user_id=xxx` 또는 헤더 `X-User-ID: xxx`에서 `user_id`를 추출합니다.
    /// 프로덕션 모드에서는 `None`을 반환합니다.
    ///
    /// # 우선순위
    /// 1. 쿼리 파라미터 `user_id`
    /// 2. 헤더 `X-User-ID`
    ///
    /// # 반환값
    /// - `Some(user_id)`: 개발 모드이고 user_id를 추출한 경우
    /// - `None`: 프로덕션 모드이거나 user_id를 추출할 수 없는 경우
    #[cfg(test)]
    pub fn extract_user_id_for_dev_mode(req: &HttpRequest) -> Option<i32> {
        Self::extract_user_id_for_dev_mode_impl(req)
    }

    /// 개발 모드에서 user_id를 추출하는 헬퍼 함수 (내부 구현)
    pub(crate) fn extract_user_id_for_dev_mode_impl(req: &HttpRequest) -> Option<i32> {
        // 개발 모드 확인
        let is_dev_mode = std::env::var("APP_ENV")
            .or_else(|_| std::env::var("RUN_ENV"))
            .map(|env| env == "development")
            .unwrap_or(false);

        if !is_dev_mode {
            return None;
        }

        // 1. 쿼리 파라미터에서 추출 시도
        if let Some(query) = req.uri().query() {
            for pair in query.split('&') {
                let mut parts = pair.splitn(2, '=');
                if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                    if key == "user_id" {
                        if let Ok(user_id) = value.parse::<i32>() {
                            return Some(user_id);
                        }
                    }
                }
            }
        }

        // 2. 헤더에서 추출 시도
        if let Some(header_value) = req.headers().get("X-User-ID") {
            if let Ok(header_str) = header_value.to_str() {
                if let Ok(user_id) = header_str.parse::<i32>() {
                    return Some(user_id);
                }
            }
        }

        None
    }
}

#[utoipa::path(
    post,
    path = "/api/annotations",
    tag = "annotations",
    request_body = CreateAnnotationRequest,
    responses(
        (status = 201, description = "Annotation created successfully", body = AnnotationResponse),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "User or Project not found"),
    )
)]
pub async fn create_annotation(
    req: web::Json<CreateAnnotationRequest>,
    use_case: web::Data<
        Arc<
            AnnotationUseCase<
                AnnotationServiceImpl<
                    AnnotationRepositoryImpl,
                    UserRepositoryImpl,
                    ProjectRepositoryImpl,
                >,
                UserRepositoryImpl,
                AccessControlServiceImpl<
                    AccessLogRepositoryImpl,
                    UserRepositoryImpl,
                    ProjectRepositoryImpl,
                    RoleRepositoryImpl,
                    PermissionRepositoryImpl,
                >,
            >,
        >,
    >,
    http_req: HttpRequest,
) -> impl Responder {
    // user_id 추출
    let user_id = match AnnotationController::extract_user_id_or_unauthorized(&http_req) {
        Ok(id) => id,
        Err(response) => return response,
    };

    let request = req.into_inner();
    let project_id = request.project_id.unwrap_or(299); // 또는 적절한 기본값

    match use_case
        .create_annotation(request, user_id, project_id)
        .await
    {
        Ok(annotation) => HttpResponse::Created().json(annotation),
        Err(e) => AnnotationController::handle_service_error(e),
    }
}

#[utoipa::path(
    get,
    path = "/api/annotations/{annotation_id}",
    tag = "annotations",
    params(
        ("annotation_id" = i32, Path, description = "Annotation ID")
    ),
    responses(
        (status = 200, description = "Get annotation successfully", body = AnnotationResponse),
        (status = 401, description = "Unauthorized - User does not have permission to read this annotation"),
        (status = 404, description = "Annotation not found"),
    )
)]
pub async fn get_annotation(
    annotation_id: web::Path<i32>,
    req: HttpRequest,
    use_case: web::Data<
        Arc<
            AnnotationUseCase<
                AnnotationServiceImpl<
                    AnnotationRepositoryImpl,
                    UserRepositoryImpl,
                    ProjectRepositoryImpl,
                >,
                UserRepositoryImpl,
                AccessControlServiceImpl<
                    AccessLogRepositoryImpl,
                    UserRepositoryImpl,
                    ProjectRepositoryImpl,
                    RoleRepositoryImpl,
                    PermissionRepositoryImpl,
                >,
            >,
        >,
    >,
) -> impl Responder {
    // user_id 추출 (개발 모드)
    let user_id = match AnnotationController::extract_user_id_for_dev_mode_impl(&req) {
        Some(id) => id,
        None => {
            return HttpResponse::Unauthorized().json(json!({
                "error": "Unauthorized",
                "message": "User ID is required"
            }));
        }
    };

    match use_case.get_annotation_by_id(user_id, *annotation_id).await {
        Ok(annotation) => HttpResponse::Ok()
            .insert_header(("Cache-Control", "public, max-age=5"))
            .insert_header(("ETag", format!("\"{}\"", annotation.version)))
            .insert_header(("Last-Modified", annotation.updated_at.to_rfc2822()))
            .json(annotation),
        Err(ServiceError::NotFound(msg)) => HttpResponse::NotFound().json(json!({
            "error": "Not Found",
            "message": msg
        })),
        Err(ServiceError::Unauthorized(msg)) => HttpResponse::Unauthorized().json(json!({
            "error": "Unauthorized",
            "message": msg
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": "Internal Server Error",
            "message": e.to_string()
        })),
    }
}

/// HEAD 요청 핸들러 - 응답 헤더만 반환 (본문 없음)
///
/// 이 메서드는 GET 요청과 동일한 헤더를 반환하지만 응답 본문은 비어있습니다.
/// 클라이언트는 ETag와 Last-Modified 헤더를 사용하여 캐시 검증을 수행할 수 있습니다.
///
/// # 사용 사례
/// - 캐시 검증: If-None-Match, If-Modified-Since 헤더 사용
/// - 대역폭 절약: 응답 본문 없이 메타데이터만 전송
/// - 리소스 존재 확인: 404 응답으로 리소스 존재 여부 확인
pub async fn head_annotation(
    annotation_id: web::Path<i32>,
    req: HttpRequest,
    use_case: web::Data<
        Arc<
            AnnotationUseCase<
                AnnotationServiceImpl<
                    AnnotationRepositoryImpl,
                    UserRepositoryImpl,
                    ProjectRepositoryImpl,
                >,
                UserRepositoryImpl,
                AccessControlServiceImpl<
                    AccessLogRepositoryImpl,
                    UserRepositoryImpl,
                    ProjectRepositoryImpl,
                    RoleRepositoryImpl,
                    PermissionRepositoryImpl,
                >,
            >,
        >,
    >,
) -> impl Responder {
    // user_id 추출
    let user_id = match AnnotationController::extract_user_id_or_unauthorized(&req) {
        Ok(id) => id,
        Err(response) => return response,
    };

    match use_case.get_annotation_by_id(user_id, *annotation_id).await {
        Ok(annotation) => {
            let etag = format!("\"{}\"", annotation.version);
            let last_modified = annotation.updated_at.to_rfc2822();

            // If-None-Match 헤더 확인 (ETag 기반 캐시 검증)
            if let Some(if_none_match) = req.headers().get("If-None-Match") {
                if let Ok(if_none_match_str) = if_none_match.to_str() {
                    if if_none_match_str == etag || if_none_match_str == "*" {
                        // 304 Not Modified 응답
                        return HttpResponse::NotModified()
                            .insert_header(("ETag", etag))
                            .insert_header(("Cache-Control", "public, max-age=5"))
                            .finish();
                    }
                }
            }

            // If-Modified-Since 헤더 확인 (Last-Modified 기반 캐시 검증)
            if let Some(if_modified_since) = req.headers().get("If-Modified-Since") {
                if let Ok(if_modified_since_str) = if_modified_since.to_str() {
                    // RFC 2822 형식으로 파싱하여 비교
                    if let Ok(client_time) = chrono::DateTime::parse_from_rfc2822(if_modified_since_str) {
                        if annotation.updated_at <= client_time.with_timezone(&chrono::Utc) {
                            // 304 Not Modified 응답
                            return HttpResponse::NotModified()
                                .insert_header(("Last-Modified", last_modified))
                                .insert_header(("Cache-Control", "public, max-age=5"))
                                .finish();
                        }
                    }
                }
            }

            // 200 OK 응답 (본문 없음)
            HttpResponse::Ok()
                .insert_header(("ETag", etag))
                .insert_header(("Last-Modified", last_modified))
                .insert_header(("Cache-Control", "public, max-age=5"))
                .finish()
        }
        Err(e) => AnnotationController::handle_service_error(e),
    }
}

/// 어노테이션 요약 목록 조회 (HEAD 요청)
///
/// Series UID로 어노테이션 목록의 메타데이터를 조회합니다.
/// 응답 헤더에 list_version을 포함하여 캐시 검증에 사용할 수 있습니다.
#[utoipa::path(
    head,
    path = "/api/annotations/summary",
    tag = "annotations",
    params(
        ("series_instance_uid" = String, Query, description = "Series Instance UID"),
        ("project_id" = i32, Query, description = "프로젝트 ID"),
        ("page" = Option<i32>, Query, description = "페이지 번호 (기본값: 1)"),
        ("limit" = Option<i32>, Query, description = "페이지 크기 (기본값: 20)"),
    ),
    responses(
        (status = 200, description = "Annotation summary metadata retrieved successfully"),
        (status = 304, description = "Not Modified (캐시 유효)"),
        (status = 400, description = "Invalid parameters"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn head_annotation_summary(
    query: web::Query<std::collections::HashMap<String, String>>,
    req: HttpRequest,
    use_case: web::Data<
        Arc<
            AnnotationUseCase<
                AnnotationServiceImpl<
                    AnnotationRepositoryImpl,
                    UserRepositoryImpl,
                    ProjectRepositoryImpl,
                >,
                UserRepositoryImpl,
                AccessControlServiceImpl<
                    AccessLogRepositoryImpl,
                    UserRepositoryImpl,
                    ProjectRepositoryImpl,
                    RoleRepositoryImpl,
                    PermissionRepositoryImpl,
                >,
            >,
        >,
    >,
) -> impl Responder {
    // 필수 파라미터 추출
    let series_instance_uid = match query.get("series_instance_uid") {
        Some(uid) => uid.clone(),
        None => {
            return HttpResponse::BadRequest().finish()
        }
    };

    let project_id = match query.get("project_id").and_then(|s| s.parse::<i32>().ok()) {
        Some(id) => id,
        None => {
            return HttpResponse::BadRequest().finish()
        }
    };

    // 선택적 파라미터 추출
    let page = query.get("page").and_then(|s| s.parse::<i32>().ok()).unwrap_or(1);
    let limit = query.get("limit").and_then(|s| s.parse::<i32>().ok()).unwrap_or(20);

    // 어노테이션 조회 (페이지네이션 포함)
    match use_case
        .get_annotations_by_project_and_series_paginated(project_id, &series_instance_uid, page, limit)
        .await
    {
        Ok(response) => {
            // If-Modified-Since 헤더 확인 (캐시 검증)
            if let Some(if_modified_since) = req.headers().get("If-Modified-Since") {
                if let Ok(if_modified_since_str) = if_modified_since.to_str() {
                    if let Ok(client_time) = chrono::DateTime::parse_from_rfc2822(if_modified_since_str) {
                        if let Some(list_version) = response.list_version {
                            if list_version <= client_time.with_timezone(&chrono::Utc) {
                                // 304 Not Modified 응답
                                return HttpResponse::NotModified()
                                    .insert_header(("Last-Modified", list_version.to_rfc2822()))
                                    .insert_header(("Cache-Control", "public, max-age=30"))
                                    .finish();
                            }
                        }
                    }
                }
            }

            // list_version을 Last-Modified 헤더로 설정
            if let Some(list_version) = response.list_version {
                HttpResponse::Ok()
                    .insert_header(("Cache-Control", "public, max-age=30"))
                    .insert_header(("Last-Modified", list_version.to_rfc2822()))
                    .insert_header(("X-List-Version", list_version.to_rfc3339()))
                    .finish()
            } else {
                HttpResponse::Ok()
                    .insert_header(("Cache-Control", "public, max-age=30"))
                    .finish()
            }
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

/// 어노테이션 목록 버전 조회 (HEAD 요청)
///
/// SOP Instance UID, Series UID, 또는 Study UID로 어노테이션 목록의 버전 정보를 조회합니다.
/// 응답 헤더에 list_version을 포함하여 캐시 검증에 사용할 수 있습니다.
#[utoipa::path(
    head,
    path = "/api/annotations",
    tag = "annotations",
    params(
        ("sop_instance_uid" = Option<String>, Query, description = "SOP Instance UID로 필터링"),
        ("series_instance_uid" = Option<String>, Query, description = "Series Instance UID로 필터링"),
        ("study_instance_uid" = Option<String>, Query, description = "Study Instance UID로 필터링"),
    ),
    responses(
        (status = 200, description = "Annotation list metadata", headers(
            ("Last-Modified" = String, description = "최신 어노테이션 수정 시간"),
            ("X-List-Version" = String, description = "목록 버전 (ISO 8601)"),
            ("X-Total-Count" = String, description = "총 어노테이션 개수"),
        )),
        (status = 304, description = "Not Modified"),
        (status = 400, description = "Bad Request"),
    )
)]
pub async fn head_annotations(
    query: web::Query<std::collections::HashMap<String, String>>,
    req: HttpRequest,
    use_case: web::Data<
        Arc<
            AnnotationUseCase<
                AnnotationServiceImpl<
                    AnnotationRepositoryImpl,
                    UserRepositoryImpl,
                    ProjectRepositoryImpl,
                >,
                UserRepositoryImpl,
                AccessControlServiceImpl<
                    AccessLogRepositoryImpl,
                    UserRepositoryImpl,
                    ProjectRepositoryImpl,
                    RoleRepositoryImpl,
                    PermissionRepositoryImpl,
                >,
            >,
        >,
    >,
) -> impl Responder {
    // 쿼리 파라미터 확인
    let sop_instance_uid = query.get("sop_instance_uid");
    let series_instance_uid = query.get("series_instance_uid");
    let study_instance_uid = query.get("study_instance_uid");

    // 최소한 하나의 UID가 필요
    if sop_instance_uid.is_none() && series_instance_uid.is_none() && study_instance_uid.is_none() {
        return HttpResponse::BadRequest().json(json!({
            "error": "Bad Request",
            "message": "At least one of sop_instance_uid, series_instance_uid, or study_instance_uid is required"
        }));
    }

    // 우선순위: sop_instance_uid > series_instance_uid > study_instance_uid
    let result = if let Some(sop_uid) = sop_instance_uid {
        use_case.get_annotations_by_instance(sop_uid).await
    } else if let Some(series_uid) = series_instance_uid {
        use_case.get_annotations_by_series(series_uid).await
    } else if let Some(study_uid) = study_instance_uid {
        use_case.get_annotations_by_study(study_uid).await
    } else {
        return HttpResponse::BadRequest().finish();
    };

    match result {
        Ok(response) => {
            // list_version 계산 (가장 최근 updated_at)
            let list_version = response
                .annotations
                .iter()
                .map(|ann| ann.updated_at)
                .max();

            // If-Modified-Since 헤더 확인 (캐시 검증)
            if let Some(if_modified_since) = req.headers().get("If-Modified-Since") {
                if let Ok(if_modified_since_str) = if_modified_since.to_str() {
                    if let Ok(client_time) = chrono::DateTime::parse_from_rfc2822(if_modified_since_str) {
                        if let Some(lv) = list_version {
                            if lv <= client_time.with_timezone(&chrono::Utc) {
                                // 304 Not Modified 응답
                                return HttpResponse::NotModified()
                                    .insert_header(("Last-Modified", lv.to_rfc2822()))
                                    .insert_header(("Cache-Control", "public, max-age=5"))
                                    .finish();
                            }
                        }
                    }
                }
            }

            // 200 OK 응답 (본문 없음)
            let mut resp = HttpResponse::Ok();
            resp.insert_header(("Cache-Control", "public, max-age=5"))
                .insert_header(("X-Total-Count", response.total.to_string()));

            if let Some(lv) = list_version {
                resp.insert_header(("Last-Modified", lv.to_rfc2822()))
                    .insert_header(("X-List-Version", lv.to_rfc3339()));
            }

            resp.finish()
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

/// 어노테이션 요약 목록 조회
///
/// Series UID로 어노테이션의 요약 정보를 조회합니다.
/// 사이드바에 표시할 목록 데이터를 빠르게 로드하기 위한 엔드포인트입니다.
#[utoipa::path(
    get,
    path = "/api/annotations/summary",
    tag = "annotations",
    params(
        ("series_instance_uid" = String, Query, description = "Series Instance UID"),
        ("project_id" = i32, Query, description = "프로젝트 ID"),
        ("user_id" = Option<i32>, Query, description = "사용자 ID (선택사항)"),
        ("page" = Option<i32>, Query, description = "페이지 번호 (기본값: 1)"),
        ("limit" = Option<i32>, Query, description = "페이지 크기 (기본값: 20, 최대: 100)"),
    ),
    responses(
        (status = 200, description = "Annotation summary list retrieved successfully", body = AnnotationListResponse),
        (status = 400, description = "Invalid parameters"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn get_annotation_summary(
    query: web::Query<std::collections::HashMap<String, String>>,
    use_case: web::Data<
        Arc<
            AnnotationUseCase<
                AnnotationServiceImpl<
                    AnnotationRepositoryImpl,
                    UserRepositoryImpl,
                    ProjectRepositoryImpl,
                >,
                UserRepositoryImpl,
                AccessControlServiceImpl<
                    AccessLogRepositoryImpl,
                    UserRepositoryImpl,
                    ProjectRepositoryImpl,
                    RoleRepositoryImpl,
                    PermissionRepositoryImpl,
                >,
            >,
        >,
    >,
) -> impl Responder {
    // 필수 파라미터 추출
    let series_instance_uid = match query.get("series_instance_uid") {
        Some(uid) => uid.clone(),
        None => {
            return HttpResponse::BadRequest().json(json!({
                "error": "Bad Request",
                "message": "series_instance_uid is required"
            }))
        }
    };

    let project_id = match query.get("project_id").and_then(|s| s.parse::<i32>().ok()) {
        Some(id) => id,
        None => {
            return HttpResponse::BadRequest().json(json!({
                "error": "Bad Request",
                "message": "project_id is required and must be a valid integer"
            }))
        }
    };

    // 선택적 파라미터 추출
    let user_id = query.get("user_id").and_then(|s| s.parse::<i32>().ok());
    let page = query.get("page").and_then(|s| s.parse::<i32>().ok()).unwrap_or(1);
    let limit = query.get("limit").and_then(|s| s.parse::<i32>().ok()).unwrap_or(20);

    // user_id가 제공되면 권한 기반 필터링, 없으면 모든 annotation 반환
    let result = if let Some(uid) = user_id {
        // 권한 기반 조회: READ_ALL 권한 확인 후 필터링
        use_case
            .get_annotations_by_series_and_project_with_user(uid, &series_instance_uid, project_id)
            .await
            .map(|mut response| {
                // 페이지네이션 적용
                let total = response.annotations.len();
                let start = ((page - 1) * limit) as usize;
                let end = (start + limit as usize).min(total);

                if start < total {
                    response.annotations = response.annotations[start..end].to_vec();
                } else {
                    response.annotations = vec![];
                }

                response.total = total;
                response.page = page;
                response.limit = limit;
                response.total_pages = ((total as f64) / (limit as f64)).ceil() as i32;
                response.has_next = end < total;

                response
            })
    } else {
        // user_id 없으면 권한 체크 없이 모든 annotation 반환 (기존 동작)
        use_case
            .get_annotations_by_project_and_series_paginated(project_id, &series_instance_uid, page, limit)
            .await
    };

    match result {
        Ok(response) => HttpResponse::Ok()
            .insert_header(("Cache-Control", "public, max-age=30"))
            .json(response),
        Err(ServiceError::Unauthorized(msg)) => HttpResponse::Unauthorized().json(json!({
            "error": "Unauthorized",
            "message": msg
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": "Internal Server Error",
            "message": e.to_string()
        })),
    }
}

#[utoipa::path(
    get,
    path = "/api/annotations",
    tag = "annotations",
    params(
        ("study_instance_uid" = Option<String>, Query, description = "Study Instance UID로 필터링"),
        ("series_instance_uid" = Option<String>, Query, description = "Series Instance UID로 필터링"),
        ("sop_instance_uid" = Option<String>, Query, description = "SOP Instance UID로 필터링"),
        ("user_id" = Option<i32>, Query, description = "사용자 ID로 필터링"),
        ("project_id" = Option<i32>, Query, description = "프로젝트 ID로 필터링"),
        ("viewer_software" = Option<String>, Query, description = "뷰어 소프트웨어로 필터링"),
        ("level" = Option<String>, Query, description = "어노테이션 레벨로 필터링 (study, series, instance)"),
    ),
    responses(
        (status = 200, description = "List annotations successfully", body = AnnotationListResponse),
    )
)]
pub async fn list_annotations(
    query: web::Query<std::collections::HashMap<String, String>>,
    use_case: web::Data<
        Arc<
            AnnotationUseCase<
                AnnotationServiceImpl<
                    AnnotationRepositoryImpl,
                    UserRepositoryImpl,
                    ProjectRepositoryImpl,
                >,
                UserRepositoryImpl,
                AccessControlServiceImpl<
                    AccessLogRepositoryImpl,
                    UserRepositoryImpl,
                    ProjectRepositoryImpl,
                    RoleRepositoryImpl,
                    PermissionRepositoryImpl,
                >,
            >,
        >,
    >,
) -> impl Responder {
    // TODO: 실제로는 인증에서 user_id를 가져와야 함
    // 기본값으로 1을 사용하지만, 쿼리 파라미터가 있으면 그것을 사용
    let mut user_id = 336;

    // 쿼리 파라미터에서 user_id 추출
    let user_id_param = query.get("user_id").and_then(|s| s.parse::<i32>().ok());
    if let Some(uid) = user_id_param {
        user_id = uid;
    }

    // viewer_software 파라미터 추출
    let viewer_software = query.get("viewer_software").map(|s| s.as_str());

    // project_id 파라미터 추출
    let project_id = query.get("project_id").and_then(|s| s.parse::<i32>().ok());

    // level 파라미터 추출
    let level = query.get("level").map(|s| s.as_str());

    // 쿼리 파라미터에 따라 다른 메서드 호출
    // 우선순위: sop_instance_uid > series_instance_uid > study_instance_uid > project_id > user_id
    // 주의: user_id 파라미터가 있으면 권한 기반 필터링을 수행합니다
    let result = if let Some(sop_instance_uid) = query.get("sop_instance_uid") {
        // SOP Instance UID (가장 구체적인 필터)
        // project_id와 user_id가 모두 있으면 권한 기반 필터링 수행
        if let (Some(proj_id), Some(_)) = (project_id, user_id_param) {
            // 권한 기반 조회 (UseCase에서 권한 체크 수행)
            use_case
                .get_annotations_by_project_and_instance_with_user(user_id, proj_id, sop_instance_uid)
                .await
                .map(|mut response| {
                    // level로 필터링
                    if let Some(lvl) = level {
                        match lvl {
                            "study" => {
                                response.annotations.retain(|ann| {
                                    ann.series_instance_uid.is_empty() && ann.sop_instance_uid.is_empty()
                                });
                            }
                            "series" => {
                                response.annotations.retain(|ann| {
                                    !ann.series_instance_uid.is_empty() && ann.sop_instance_uid.is_empty()
                                });
                            }
                            "instance" => {
                                response.annotations.retain(|ann| {
                                    !ann.sop_instance_uid.is_empty()
                                });
                            }
                            _ => {}
                        }
                        response.total = response.annotations.len();
                    }

                    // viewer_software로 추가 필터링
                    if let Some(viewer) = viewer_software {
                        response.annotations.retain(|ann| {
                            ann.viewer_software.as_ref()
                                .map(|v| v.as_str() == viewer)
                                .unwrap_or(false)
                        });
                        response.total = response.annotations.len();
                    }

                    response
                })
        } else {
            // project_id나 user_id가 없으면 권한 체크 없이 조회
            use_case
                .get_annotations_by_instance(sop_instance_uid)
                .await
                .map(|mut response| {
                    // level로 필터링
                    if let Some(lvl) = level {
                        match lvl {
                            "study" => {
                                response.annotations.retain(|ann| {
                                    ann.series_instance_uid.is_empty() && ann.sop_instance_uid.is_empty()
                                });
                            }
                            "series" => {
                                response.annotations.retain(|ann| {
                                    !ann.series_instance_uid.is_empty() && ann.sop_instance_uid.is_empty()
                                });
                            }
                            "instance" => {
                                response.annotations.retain(|ann| {
                                    !ann.sop_instance_uid.is_empty()
                                });
                            }
                            _ => {}
                        }
                        response.total = response.annotations.len();
                    }

                    // viewer_software로 추가 필터링
                    if let Some(viewer) = viewer_software {
                        response.annotations.retain(|ann| {
                            ann.viewer_software.as_ref()
                                .map(|v| v.as_str() == viewer)
                                .unwrap_or(false)
                        });
                        response.total = response.annotations.len();
                    }

                    // user_id로 추가 필터링 (쿼리 파라미터에 명시된 경우, 권한 체크 없음)
                    if query.get("user_id").is_some() {
                        response.annotations.retain(|ann| ann.user_id == user_id);
                        response.total = response.annotations.len();
                    }

                    response
                })
        }
    } else if let Some(series_instance_uid) = query.get("series_instance_uid") {
        // Series Instance UID 처리
        if let Some(proj_id) = project_id {
            // series_instance_uid + project_id 조합
            // user_id 파라미터가 있으면 권한 기반 필터링 수행
            if user_id_param.is_some() {
                use_case
                    .get_annotations_by_series_and_project_with_user(user_id, series_instance_uid, proj_id)
                    .await
                    .map(|mut response| {
                        // 권한 기반 필터링: READ_ALL 권한이 없으면 본인 어노테이션만
                        // (이미 UseCase에서 처리됨)

                        // level로 필터링
                        if let Some(lvl) = level {
                            match lvl {
                                "study" => {
                                    response.annotations.retain(|ann| {
                                        ann.series_instance_uid.is_empty() && ann.sop_instance_uid.is_empty()
                                    });
                                }
                                "series" => {
                                    response.annotations.retain(|ann| {
                                        !ann.series_instance_uid.is_empty() && ann.sop_instance_uid.is_empty()
                                    });
                                }
                                "instance" => {
                                    response.annotations.retain(|ann| {
                                        !ann.sop_instance_uid.is_empty()
                                    });
                                }
                                _ => {}
                            }
                            response.total = response.annotations.len();
                        }

                        // viewer_software로 추가 필터링
                        if let Some(viewer) = viewer_software {
                            response.annotations.retain(|ann| {
                                ann.viewer_software.as_ref()
                                    .map(|v| v.as_str() == viewer)
                                    .unwrap_or(false)
                            });
                            response.total = response.annotations.len();
                        }

                        response
                    })
            } else {
                // user_id 파라미터가 없으면 기존 방식 (권한 체크 없음)
                use_case
                    .get_annotations_by_project_and_series(proj_id, series_instance_uid)
                    .await
                    .map(|mut response| {
                        // level로 필터링
                        if let Some(lvl) = level {
                            match lvl {
                                "study" => {
                                    response.annotations.retain(|ann| {
                                        ann.series_instance_uid.is_empty() && ann.sop_instance_uid.is_empty()
                                    });
                                }
                                "series" => {
                                    response.annotations.retain(|ann| {
                                        !ann.series_instance_uid.is_empty() && ann.sop_instance_uid.is_empty()
                                    });
                                }
                                "instance" => {
                                    response.annotations.retain(|ann| {
                                        !ann.sop_instance_uid.is_empty()
                                    });
                                }
                                _ => {}
                            }
                            response.total = response.annotations.len();
                        }

                        // viewer_software로 추가 필터링
                        if let Some(viewer) = viewer_software {
                            response.annotations.retain(|ann| {
                                ann.viewer_software.as_ref()
                                    .map(|v| v.as_str() == viewer)
                                    .unwrap_or(false)
                            });
                            response.total = response.annotations.len();
                        }

                        response
                    })
            }
        } else {
            // series_instance_uid만 있으면 권한 체크 없이 조회
            use_case
                .get_annotations_by_series(series_instance_uid)
                .await
                .map(|mut response| {
                    // level로 필터링
                    if let Some(lvl) = level {
                        match lvl {
                            "study" => {
                                response.annotations.retain(|ann| {
                                    ann.series_instance_uid.is_empty() && ann.sop_instance_uid.is_empty()
                                });
                            }
                            "series" => {
                                response.annotations.retain(|ann| {
                                    !ann.series_instance_uid.is_empty() && ann.sop_instance_uid.is_empty()
                                });
                            }
                            "instance" => {
                                response.annotations.retain(|ann| {
                                    !ann.sop_instance_uid.is_empty()
                                });
                            }
                            _ => {}
                        }
                        response.total = response.annotations.len();
                    }

                    // viewer_software로 추가 필터링
                    if let Some(viewer) = viewer_software {
                        response.annotations.retain(|ann| {
                            ann.viewer_software.as_ref()
                                .map(|v| v.as_str() == viewer)
                                .unwrap_or(false)
                        });
                        response.total = response.annotations.len();
                    }

                    response
                })
        }
    } else if let Some(study_uid) = query.get("study_instance_uid") {
        if let Some(proj_id) = project_id {
            // study_instance_uid + project_id 조합
            // user_id 파라미터가 있으면 권한 기반 필터링 수행
            if user_id_param.is_some() {
                use_case
                    .get_annotations_by_project_and_study_with_user(user_id, proj_id, study_uid)
                    .await
                    .map(|mut response| {
                        // level로 필터링
                        if let Some(lvl) = level {
                            match lvl {
                                "study" => {
                                    response.annotations.retain(|ann| {
                                        ann.series_instance_uid.is_empty() && ann.sop_instance_uid.is_empty()
                                    });
                                }
                                "series" => {
                                    response.annotations.retain(|ann| {
                                        !ann.series_instance_uid.is_empty() && ann.sop_instance_uid.is_empty()
                                    });
                                }
                                "instance" => {
                                    response.annotations.retain(|ann| {
                                        !ann.sop_instance_uid.is_empty()
                                    });
                                }
                                _ => {}
                            }
                            response.total = response.annotations.len();
                        }

                        // viewer_software로 추가 필터링
                        if let Some(viewer) = viewer_software {
                            response.annotations.retain(|ann| {
                                ann.viewer_software.as_ref()
                                    .map(|v| v.as_str() == viewer)
                                    .unwrap_or(false)
                            });
                            response.total = response.annotations.len();
                        }

                        response
                    })
            } else {
                // user_id 파라미터가 없으면 기존 방식 (권한 체크 없음)
                match use_case
                    .get_annotations_by_project_and_study(proj_id, study_uid)
                    .await
                {
                    Ok(mut response) => {
                        // level로 필터링
                        if let Some(lvl) = level {
                            match lvl {
                                "study" => {
                                    response.annotations.retain(|ann| {
                                        ann.series_instance_uid.is_empty() && ann.sop_instance_uid.is_empty()
                                    });
                                }
                                "series" => {
                                    response.annotations.retain(|ann| {
                                        !ann.series_instance_uid.is_empty() && ann.sop_instance_uid.is_empty()
                                    });
                                }
                                "instance" => {
                                    response.annotations.retain(|ann| {
                                        !ann.sop_instance_uid.is_empty()
                                    });
                                }
                                _ => {}
                            }
                            response.total = response.annotations.len();
                        }

                        // viewer_software로 추가 필터링
                        if let Some(viewer) = viewer_software {
                            response.annotations.retain(|ann| {
                                ann.viewer_software.as_ref()
                                    .map(|v| v.as_str() == viewer)
                                    .unwrap_or(false)
                            });
                            response.total = response.annotations.len();
                        }

                        Ok(response)
                    }
                    Err(e) => Err(e),
                }
            }
        } else {
            // study_instance_uid만 있으면 study로 필터링
            use_case
                .get_annotations_by_study_with_viewer(study_uid, viewer_software)
                .await
                .map(|mut response| {
                    // level로 필터링
                    if let Some(lvl) = level {
                        match lvl {
                            "study" => {
                                response.annotations.retain(|ann| {
                                    ann.series_instance_uid.is_empty() && ann.sop_instance_uid.is_empty()
                                });
                            }
                            "series" => {
                                response.annotations.retain(|ann| {
                                    !ann.series_instance_uid.is_empty() && ann.sop_instance_uid.is_empty()
                                });
                            }
                            "instance" => {
                                response.annotations.retain(|ann| {
                                    !ann.sop_instance_uid.is_empty()
                                });
                            }
                            _ => {}
                        }
                        response.total = response.annotations.len();
                    }

                    response
                })
        }
    } else if let Some(proj_id) = project_id {
        // project_id만 있으면 권한 기반으로 필터링
        // READ_ALL 권한이 있으면 프로젝트의 모든 annotation, 없으면 본인 annotation만
        use_case
            .get_annotations_by_project_with_permission(user_id, proj_id, viewer_software)
            .await
            .map(|mut response| {
                // level로 필터링
                if let Some(lvl) = level {
                    match lvl {
                        "study" => {
                            response.annotations.retain(|ann| {
                                ann.series_instance_uid.is_empty() && ann.sop_instance_uid.is_empty()
                            });
                        }
                        "series" => {
                            response.annotations.retain(|ann| {
                                !ann.series_instance_uid.is_empty() && ann.sop_instance_uid.is_empty()
                            });
                        }
                        "instance" => {
                            response.annotations.retain(|ann| {
                                !ann.sop_instance_uid.is_empty()
                            });
                        }
                        _ => {}
                    }
                    response.total = response.annotations.len();
                }

                response
            })
    } else {
        // 기본적으로 사용자의 annotation 목록 반환
        use_case
            .get_annotations_by_user_with_viewer(user_id, viewer_software)
            .await
            .map(|mut response| {
                // level로 필터링
                if let Some(lvl) = level {
                    match lvl {
                        "study" => {
                            response.annotations.retain(|ann| {
                                ann.series_instance_uid.is_empty() && ann.sop_instance_uid.is_empty()
                            });
                        }
                        "series" => {
                            response.annotations.retain(|ann| {
                                !ann.series_instance_uid.is_empty() && ann.sop_instance_uid.is_empty()
                            });
                        }
                        "instance" => {
                            response.annotations.retain(|ann| {
                                !ann.sop_instance_uid.is_empty()
                            });
                        }
                        _ => {}
                    }
                    response.total = response.annotations.len();
                }

                response
            })
    };

    match result {
        Ok(annotations) => {
            // list_version 계산 (가장 최근 updated_at)
            let list_version = annotations
                .annotations
                .iter()
                .map(|ann| ann.updated_at)
                .max();

            let mut resp = HttpResponse::Ok();
            resp.insert_header(("Cache-Control", "public, max-age=5"))
                .insert_header(("X-Total-Count", annotations.total.to_string()));

            if let Some(lv) = list_version {
                resp.insert_header(("Last-Modified", lv.to_rfc2822()))
                    .insert_header(("X-List-Version", lv.to_rfc3339()));
            }

            resp.json(annotations)
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": "Internal Server Error",
            "message": e.to_string()
        })),
    }
}

#[utoipa::path(
    put,
    path = "/api/annotations/{annotation_id}",
    tag = "annotations",
    request_body = UpdateAnnotationRequest,
    params(
        ("annotation_id" = i32, Path, description = "Annotation ID")
    ),
    responses(
        (status = 200, description = "Annotation updated successfully", body = AnnotationResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Annotation not found"),
        (status = 400, description = "Invalid request"),
    )
)]
pub async fn update_annotation(
    annotation_id: web::Path<i32>,
    req: web::Json<UpdateAnnotationRequest>,
    use_case: web::Data<
        Arc<
            AnnotationUseCase<
                AnnotationServiceImpl<
                    AnnotationRepositoryImpl,
                    UserRepositoryImpl,
                    ProjectRepositoryImpl,
                >,
                UserRepositoryImpl,
                AccessControlServiceImpl<
                    AccessLogRepositoryImpl,
                    UserRepositoryImpl,
                    ProjectRepositoryImpl,
                    RoleRepositoryImpl,
                    PermissionRepositoryImpl,
                >,
            >,
        >,
    >,
    http_req: HttpRequest,
) -> impl Responder {
    // user_id 추출
    let user_id = match AnnotationController::extract_user_id_or_unauthorized(&http_req) {
        Ok(id) => id,
        Err(response) => return response,
    };

    match use_case
        .update_annotation(*annotation_id, req.into_inner(), user_id)
        .await
    {
        Ok(annotation) => HttpResponse::Ok().json(annotation),
        Err(e) => AnnotationController::handle_service_error(e),
    }
}

#[utoipa::path(
    delete,
    path = "/api/annotations/{annotation_id}",
    tag = "annotations",
    params(
        ("annotation_id" = i32, Path, description = "Annotation ID")
    ),
    responses(
        (status = 204, description = "Annotation deleted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Annotation not found"),
    )
)]
pub async fn delete_annotation(
    annotation_id: web::Path<i32>,
    use_case: web::Data<
        Arc<
            AnnotationUseCase<
                AnnotationServiceImpl<
                    AnnotationRepositoryImpl,
                    UserRepositoryImpl,
                    ProjectRepositoryImpl,
                >,
                UserRepositoryImpl,
                AccessControlServiceImpl<
                    AccessLogRepositoryImpl,
                    UserRepositoryImpl,
                    ProjectRepositoryImpl,
                    RoleRepositoryImpl,
                    PermissionRepositoryImpl,
                >,
            >,
        >,
    >,
    http_req: HttpRequest,
) -> impl Responder {
    // user_id 추출
    let user_id = match AnnotationController::extract_user_id_or_unauthorized(&http_req) {
        Ok(id) => id,
        Err(response) => return response,
    };

    match use_case.delete_annotation(*annotation_id, user_id).await {
        Ok(_) => HttpResponse::NoContent().json(json!({
            "message": "Annotation deleted successfully"
        })),
        Err(e) => AnnotationController::handle_service_error(e),
    }
}

#[utoipa::path(
    get,
    path = "/api/annotations/permissions",
    tag = "annotations",
    params(
        ("project_id" = i32, Query, description = "Project ID (required)"),
        ("user_id" = Option<i32>, Query, description = "Target User ID (optional, defaults to requesting user)")
    ),
    responses(
        (status = 200, description = "Get annotation permissions successfully", body = AnnotationPermissionsResponse),
        (status = 400, description = "Bad Request - project_id is required"),
        (status = 401, description = "Unauthorized - User ID is required"),
        (status = 403, description = "Forbidden - Insufficient permissions to view other user's permissions"),
    )
)]
pub async fn get_annotation_permissions(
    query: web::Query<std::collections::HashMap<String, String>>,
    use_case: web::Data<
        Arc<
            AnnotationUseCase<
                AnnotationServiceImpl<
                    AnnotationRepositoryImpl,
                    UserRepositoryImpl,
                    ProjectRepositoryImpl,
                >,
                UserRepositoryImpl,
                AccessControlServiceImpl<
                    AccessLogRepositoryImpl,
                    UserRepositoryImpl,
                    ProjectRepositoryImpl,
                    RoleRepositoryImpl,
                    PermissionRepositoryImpl,
                >,
            >,
        >,
    >,
    http_req: HttpRequest,
) -> impl Responder {
    // 요청한 사용자의 user_id 추출
    let requesting_user_id = match AnnotationController::extract_user_id_or_unauthorized(&http_req) {
        Ok(id) => id,
        Err(response) => return response,
    };

    // project_id 추출 및 검증
    let project_id = match AnnotationController::validate_project_id(&query) {
        Ok(id) => id,
        Err(response) => return response,
    };

    // target_user_id 추출 (기본값은 요청한 사용자)
    let target_user_id = AnnotationController::extract_target_user_id(&query, requesting_user_id);

    // 다른 사용자의 권한을 조회하려는 경우 프로젝트 멤버 확인
    if target_user_id != requesting_user_id {
        match use_case
            .is_project_member(requesting_user_id, project_id)
            .await
        {
            Ok(true) => {
                // 프로젝트 멤버이면 다른 사용자의 권한 조회 허용
            }
            Ok(false) => {
                return HttpResponse::Forbidden().json(json!({
                    "error": "Forbidden",
                    "message": "You must be a member of this project to view other user's permissions"
                }));
            }
            Err(e) => {
                return AnnotationController::handle_service_error(e);
            }
        }
    }

    // 권한 조회
    match use_case
        .get_user_annotation_permissions(target_user_id, project_id)
        .await
    {
        Ok(permissions) => HttpResponse::Ok().json(permissions),
        Err(e) => AnnotationController::handle_service_error(e),
    }
}

pub fn configure_routes(
    cfg: &mut web::ServiceConfig,
    use_case: Arc<
        AnnotationUseCase<
            AnnotationServiceImpl<
                AnnotationRepositoryImpl,
                UserRepositoryImpl,
                ProjectRepositoryImpl,
            >,
            UserRepositoryImpl,
            AccessControlServiceImpl<
                AccessLogRepositoryImpl,
                UserRepositoryImpl,
                ProjectRepositoryImpl,
                RoleRepositoryImpl,
                PermissionRepositoryImpl,
            >,
        >,
    >,
    mask_group_use_case: Arc<
        crate::application::use_cases::MaskGroupUseCase<
            crate::domain::services::MaskGroupServiceImpl<
                crate::infrastructure::repositories::MaskGroupRepositoryImpl,
                crate::infrastructure::repositories::AnnotationRepositoryImpl,
                crate::infrastructure::repositories::UserRepositoryImpl,
            >,
            crate::application::services::SignedUrlServiceImpl,
        >,
    >,
) {
    cfg.app_data(web::Data::new(use_case))
        .app_data(web::Data::new(mask_group_use_case))
        .service(
            web::scope("/annotations")
                .route("/summary", web::get().to(get_annotation_summary))
                .route("/summary", web::head().to(head_annotation_summary))
                .route("", web::post().to(create_annotation))
                .route("", web::get().to(list_annotations))
                .route("", web::head().to(head_annotations))
                .route("/permissions", web::get().to(get_annotation_permissions))
                .route("/{annotation_id}", web::get().to(get_annotation))
                .route("/{annotation_id}", web::head().to(head_annotation))
                .route("/{annotation_id}", web::put().to(update_annotation))
                .route("/{annotation_id}", web::delete().to(delete_annotation))
                // Mask Groups routes
                .route(
                    "/{annotation_id}/mask-groups",
                    web::post().to(crate::presentation::controllers::mask_group_controller::create_mask_group::<
                        crate::domain::services::MaskGroupServiceImpl<
                            crate::infrastructure::repositories::MaskGroupRepositoryImpl,
                            crate::infrastructure::repositories::AnnotationRepositoryImpl,
                            crate::infrastructure::repositories::UserRepositoryImpl,
                        >,
                        crate::application::services::SignedUrlServiceImpl,
                    >),
                )
                .route(
                    "/{annotation_id}/mask-groups",
                    web::get().to(crate::presentation::controllers::mask_group_controller::list_mask_groups::<
                        crate::domain::services::MaskGroupServiceImpl<
                            crate::infrastructure::repositories::MaskGroupRepositoryImpl,
                            crate::infrastructure::repositories::AnnotationRepositoryImpl,
                            crate::infrastructure::repositories::UserRepositoryImpl,
                        >,
                        crate::application::services::SignedUrlServiceImpl,
                    >),
                )
                .route(
                    "/{annotation_id}/mask-groups/{group_id}",
                    web::get().to(crate::presentation::controllers::mask_group_controller::get_mask_group::<
                        crate::domain::services::MaskGroupServiceImpl<
                            crate::infrastructure::repositories::MaskGroupRepositoryImpl,
                            crate::infrastructure::repositories::AnnotationRepositoryImpl,
                            crate::infrastructure::repositories::UserRepositoryImpl,
                        >,
                        crate::application::services::SignedUrlServiceImpl,
                    >),
                )
                .route(
                    "/{annotation_id}/mask-groups/{group_id}",
                    web::put().to(crate::presentation::controllers::mask_group_controller::update_mask_group::<
                        crate::domain::services::MaskGroupServiceImpl<
                            crate::infrastructure::repositories::MaskGroupRepositoryImpl,
                            crate::infrastructure::repositories::AnnotationRepositoryImpl,
                            crate::infrastructure::repositories::UserRepositoryImpl,
                        >,
                        crate::application::services::SignedUrlServiceImpl,
                    >),
                )
                .route(
                    "/{annotation_id}/mask-groups/{group_id}",
                    web::delete().to(crate::presentation::controllers::mask_group_controller::delete_mask_group::<
                        crate::domain::services::MaskGroupServiceImpl<
                            crate::infrastructure::repositories::MaskGroupRepositoryImpl,
                            crate::infrastructure::repositories::AnnotationRepositoryImpl,
                            crate::infrastructure::repositories::UserRepositoryImpl,
                        >,
                        crate::application::services::SignedUrlServiceImpl,
                    >),
                )
                .route(
                    "/{annotation_id}/mask-groups/{group_id}/upload-url",
                    web::post().to(crate::presentation::controllers::mask_group_controller::generate_upload_url::<
                        crate::domain::services::MaskGroupServiceImpl<
                            crate::infrastructure::repositories::MaskGroupRepositoryImpl,
                            crate::infrastructure::repositories::AnnotationRepositoryImpl,
                            crate::infrastructure::repositories::UserRepositoryImpl,
                        >,
                        crate::application::services::SignedUrlServiceImpl,
                    >),
                )
                .route(
                    "/{annotation_id}/mask-groups/{group_id}/complete-upload",
                    web::post().to(crate::presentation::controllers::mask_group_controller::complete_upload::<
                        crate::domain::services::MaskGroupServiceImpl<
                            crate::infrastructure::repositories::MaskGroupRepositoryImpl,
                            crate::infrastructure::repositories::AnnotationRepositoryImpl,
                            crate::infrastructure::repositories::UserRepositoryImpl,
                        >,
                        crate::application::services::SignedUrlServiceImpl,
                    >),
                )
                // Mask routes
                .route(
                    "/{annotation_id}/mask-groups/{group_id}/masks",
                    web::post().to(crate::presentation::controllers::mask_controller::create_mask::<
                        crate::domain::services::MaskServiceImpl<
                            crate::infrastructure::repositories::MaskRepositoryImpl,
                            crate::infrastructure::repositories::MaskGroupRepositoryImpl,
                            crate::infrastructure::repositories::UserRepositoryImpl,
                        >,
                        crate::domain::services::MaskGroupServiceImpl<
                            crate::infrastructure::repositories::MaskGroupRepositoryImpl,
                            crate::infrastructure::repositories::AnnotationRepositoryImpl,
                            crate::infrastructure::repositories::UserRepositoryImpl,
                        >,
                        crate::application::services::SignedUrlServiceImpl,
                    >),
                )
                .route(
                    "/{annotation_id}/mask-groups/{group_id}/masks",
                    web::get().to(crate::presentation::controllers::mask_controller::list_masks::<
                        crate::domain::services::MaskServiceImpl<
                            crate::infrastructure::repositories::MaskRepositoryImpl,
                            crate::infrastructure::repositories::MaskGroupRepositoryImpl,
                            crate::infrastructure::repositories::UserRepositoryImpl,
                        >,
                        crate::domain::services::MaskGroupServiceImpl<
                            crate::infrastructure::repositories::MaskGroupRepositoryImpl,
                            crate::infrastructure::repositories::AnnotationRepositoryImpl,
                            crate::infrastructure::repositories::UserRepositoryImpl,
                        >,
                        crate::application::services::SignedUrlServiceImpl,
                    >),
                )
                .route(
                    "/{annotation_id}/mask-groups/{group_id}/masks/{mask_id}",
                    web::get().to(crate::presentation::controllers::mask_controller::get_mask::<
                        crate::domain::services::MaskServiceImpl<
                            crate::infrastructure::repositories::MaskRepositoryImpl,
                            crate::infrastructure::repositories::MaskGroupRepositoryImpl,
                            crate::infrastructure::repositories::UserRepositoryImpl,
                        >,
                        crate::domain::services::MaskGroupServiceImpl<
                            crate::infrastructure::repositories::MaskGroupRepositoryImpl,
                            crate::infrastructure::repositories::AnnotationRepositoryImpl,
                            crate::infrastructure::repositories::UserRepositoryImpl,
                        >,
                        crate::application::services::SignedUrlServiceImpl,
                    >),
                )
                .route(
                    "/{annotation_id}/mask-groups/{group_id}/masks/{mask_id}",
                    web::put().to(crate::presentation::controllers::mask_controller::update_mask::<
                        crate::domain::services::MaskServiceImpl<
                            crate::infrastructure::repositories::MaskRepositoryImpl,
                            crate::infrastructure::repositories::MaskGroupRepositoryImpl,
                            crate::infrastructure::repositories::UserRepositoryImpl,
                        >,
                        crate::domain::services::MaskGroupServiceImpl<
                            crate::infrastructure::repositories::MaskGroupRepositoryImpl,
                            crate::infrastructure::repositories::AnnotationRepositoryImpl,
                            crate::infrastructure::repositories::UserRepositoryImpl,
                        >,
                        crate::application::services::SignedUrlServiceImpl,
                    >),
                )
                .route(
                    "/{annotation_id}/mask-groups/{group_id}/masks/{mask_id}",
                    web::delete().to(crate::presentation::controllers::mask_controller::delete_mask::<
                        crate::domain::services::MaskServiceImpl<
                            crate::infrastructure::repositories::MaskRepositoryImpl,
                            crate::infrastructure::repositories::MaskGroupRepositoryImpl,
                            crate::infrastructure::repositories::UserRepositoryImpl,
                        >,
                        crate::domain::services::MaskGroupServiceImpl<
                            crate::infrastructure::repositories::MaskGroupRepositoryImpl,
                            crate::infrastructure::repositories::AnnotationRepositoryImpl,
                            crate::infrastructure::repositories::UserRepositoryImpl,
                        >,
                        crate::application::services::SignedUrlServiceImpl,
                    >),
                )
                .route(
                    "/{annotation_id}/mask-groups/{group_id}/masks/{mask_id}/download-url",
                    web::post().to(crate::presentation::controllers::mask_controller::generate_download_url::<
                        crate::domain::services::MaskServiceImpl<
                            crate::infrastructure::repositories::MaskRepositoryImpl,
                            crate::infrastructure::repositories::MaskGroupRepositoryImpl,
                            crate::infrastructure::repositories::UserRepositoryImpl,
                        >,
                        crate::domain::services::MaskGroupServiceImpl<
                            crate::infrastructure::repositories::MaskGroupRepositoryImpl,
                            crate::infrastructure::repositories::AnnotationRepositoryImpl,
                            crate::infrastructure::repositories::UserRepositoryImpl,
                        >,
                        crate::application::services::SignedUrlServiceImpl,
                    >),
                )
                .route(
                    "/{annotation_id}/mask-groups/{group_id}/masks/stats",
                    web::get().to(crate::presentation::controllers::mask_controller::get_mask_stats::<
                        crate::domain::services::MaskServiceImpl<
                            crate::infrastructure::repositories::MaskRepositoryImpl,
                            crate::infrastructure::repositories::MaskGroupRepositoryImpl,
                            crate::infrastructure::repositories::UserRepositoryImpl,
                        >,
                        crate::domain::services::MaskGroupServiceImpl<
                            crate::infrastructure::repositories::MaskGroupRepositoryImpl,
                            crate::infrastructure::repositories::AnnotationRepositoryImpl,
                            crate::infrastructure::repositories::UserRepositoryImpl,
                        >,
                        crate::application::services::SignedUrlServiceImpl,
                    >),
                ),
        );
}
