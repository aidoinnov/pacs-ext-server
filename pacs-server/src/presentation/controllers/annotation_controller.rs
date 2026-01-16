#![allow(dead_code, unused_imports, unused_variables)]
use crate::application::dto::annotation_dto::{
    AnnotationListResponse, AnnotationPermissionsResponse, AnnotationResponse, CreateAnnotationRequest, UpdateAnnotationRequest,
};
use crate::application::dto::{
    SnapshotUploadUrlRequest, SnapshotUploadUrlResponse,
    CompleteSnapshotUploadRequest, SnapshotStatusResponse,
};
use crate::application::services::SignedUrlServiceImpl;
use crate::application::use_cases::AnnotationUseCase;
use crate::domain::services::annotation_service::AnnotationService;
use crate::domain::services::{AnnotationServiceImpl, AccessControlServiceImpl, AccessControlService};
use crate::domain::repositories::UserRepository;
use crate::domain::ServiceError;
use crate::infrastructure::repositories::{
    AnnotationRepositoryImpl, ProjectRepositoryImpl, UserRepositoryImpl,
    AccessLogRepositoryImpl, RoleRepositoryImpl, PermissionRepositoryImpl,
};
use crate::infrastructure::auth::{extract_user_id_from_request, JwtService};
use crate::application::services::SignedUrlService;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use serde_json::json;
use std::sync::Arc;

pub struct AnnotationController;

impl AnnotationController {
    pub fn new() -> Self {
        Self
    }

    /// 어노테이션 응답에 snapshot_image_url을 채워넣는 헬퍼 함수
    async fn populate_snapshot_urls(
        mut response: AnnotationListResponse,
        signed_url_service: &Arc<SignedUrlServiceImpl>,
    ) -> AnnotationListResponse {
        for annotation in &mut response.annotations {
            if let Some(ref snapshot_key) = annotation.snapshot_image_key {
                if annotation.snapshot_status.as_deref() == Some("completed") {
                    // Signed URL 생성 (TTL: 1시간)
                    match signed_url_service.generate_download_url_simple(snapshot_key, 3600).await {
                        Ok(url) => {
                            annotation.snapshot_image_url = Some(url);
                        }
                        Err(e) => {
                            tracing::warn!("Failed to generate snapshot URL for {}: {:?}", snapshot_key, e);
                        }
                    }
                }
            }
        }
        response
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

    /// user_id를 추출하는 통합 헬퍼 함수 (개발 모드 + JWT 인증)
    ///
    /// 우선순위:
    /// 1. 개발 모드 (`APP_ENV=development`): 쿼리 파라미터 또는 헤더에서 user_id 추출
    /// 2. 프로덕션 모드: JWT 토큰 검증 및 user_id 추출
    ///
    /// # 반환값
    /// - `Ok(user_id)`: user_id 추출 성공
    /// - `Err(HttpResponse)`: 인증 실패 (401 Unauthorized)
    pub async fn extract_user_id_with_auth(
        req: &HttpRequest,
        jwt: &Arc<JwtService>,
        user_repo: &Arc<UserRepositoryImpl>,
    ) -> Result<i32, HttpResponse> {
        // 1. 개발 모드 확인 (쿼리 파라미터/헤더)
        if let Some(dev_user_id) = Self::extract_user_id_for_dev_mode_impl(req) {
            return Ok(dev_user_id);
        }

        // 2. 프로덕션 모드: JWT 인증
        match extract_user_id_from_request(req, jwt, user_repo).await {
            Some(id) if id > 0 => Ok(id),
            _ => Err(HttpResponse::Unauthorized().json(json!({
                "error": "Unauthorized",
                "message": "Authentication required. Please provide valid credentials."
            }))),
        }
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
    http_req: HttpRequest,
    jwt: web::Data<Arc<JwtService>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
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
                Arc<SignedUrlServiceImpl>
            >,
        >,
    >,
) -> impl Responder {
    // user_id 추출 (개발 모드 또는 JWT 인증)
    let user_id = match AnnotationController::extract_user_id_with_auth(&http_req, &jwt, &user_repo).await {
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
    jwt: web::Data<Arc<JwtService>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
    signed_url_service: web::Data<Arc<SignedUrlServiceImpl>>,
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
                Arc<SignedUrlServiceImpl>
            >,
        >,
    >,
) -> impl Responder {
    // user_id 추출 (개발 모드 또는 JWT 인증)
    let user_id = match AnnotationController::extract_user_id_with_auth(&req, &jwt, &user_repo).await {
        Ok(id) => id,
        Err(response) => return response,
    };

    match use_case.get_annotation_by_id(user_id, *annotation_id).await {
        Ok(mut annotation) => {
            // Snapshot URL 생성 (있는 경우)
            if let Some(ref snapshot_key) = annotation.snapshot_image_key {
                if annotation.snapshot_status.as_deref() == Some("completed") {
                    match signed_url_service.generate_download_url_simple(snapshot_key, 3600).await {
                        Ok(url) => {
                            annotation.snapshot_image_url = Some(url);
                        }
                        Err(e) => {
                            tracing::warn!("Failed to generate snapshot URL for {}: {:?}", snapshot_key, e);
                        }
                    }
                }
            }
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
                        // RFC 2822는 초 단위까지만 표현하므로, 비교를 위해 초 단위로 truncate
                        let annotation_time_secs = annotation.updated_at.timestamp();
                        let client_time_secs = client_time.timestamp();

                        if annotation_time_secs <= client_time_secs {
                            // 304 Not Modified 응답
                            return HttpResponse::NotModified()
                                .insert_header(("Last-Modified", last_modified))
                                .insert_header(("Cache-Control", "public, max-age=5"))
                                .finish();
                        }
                    }
                }
            }

            // 200 OK 응답
            HttpResponse::Ok()
                .insert_header(("Cache-Control", "public, max-age=5"))
                .insert_header(("ETag", etag))
                .insert_header(("Last-Modified", last_modified))
                .json(annotation)
        }
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
                Arc<SignedUrlServiceImpl>,
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
    http_req: HttpRequest,
    jwt: web::Data<Arc<JwtService>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
    signed_url_service: web::Data<Arc<SignedUrlServiceImpl>>,
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
                Arc<SignedUrlServiceImpl>
            >,

        >,
    >,
) -> impl Responder {
    // JWT에서 user_id 추출 (fallback: query parameter)
    let user_id = match extract_user_id_from_request(&http_req, &jwt, &user_repo).await {
        Some(id) if id > 0 => id,
        _ => {
            // Fallback: query parameter (개발용, 곧 제거 예정)
            query.get("user_id")
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or_else(|| {
                    tracing::warn!("No valid user_id found in JWT or query parameter");
                    336 // 기본값
                })
        }
    };

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
        if let Some(proj_id) = project_id {
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
            // 권한 기반 필터링 수행
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
            // 권한 기반 필터링 수행
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
        Ok(mut annotations) => {
            // include_snapshot_urls 파라미터 확인 (기본값: true)
            let include_snapshot_urls = query
                .get("include_snapshot_urls")
                .and_then(|s| s.parse::<bool>().ok())
                .unwrap_or(true);

            // Snapshot URLs 생성 (bulk)
            if include_snapshot_urls {
                // snapshot_image_key가 있는 annotation들의 key 수집
                let snapshot_keys: Vec<String> = annotations
                    .annotations
                    .iter()
                    .filter_map(|ann| ann.snapshot_image_key.clone())
                    .collect();

                if !snapshot_keys.is_empty() {
                    tracing::info!("Generating signed URLs for {} snapshots", snapshot_keys.len());
                    // Bulk로 signed URL 생성 (병렬 처리)
                    match signed_url_service.generate_download_urls_bulk(snapshot_keys.clone(), Some(3600)).await {
                        Ok(url_map) => {
                            tracing::info!("Successfully generated {} signed URLs", url_map.len());
                            // HashMap으로 변환하여 빠른 조회
                            let url_lookup: std::collections::HashMap<String, Option<String>> =
                                url_map.into_iter().collect();

                            // 각 annotation에 signed URL 추가
                            let mut url_added_count = 0;
                            for ann in &mut annotations.annotations {
                                if let Some(ref key) = ann.snapshot_image_key {
                                    if let Some(url_opt) = url_lookup.get(key) {
                                        ann.snapshot_image_url = url_opt.clone();
                                        if url_opt.is_some() {
                                            url_added_count += 1;
                                        }
                                    }
                                }
                            }
                            tracing::info!("Added {} snapshot URLs to annotations", url_added_count);
                        }
                        Err(e) => {
                            tracing::warn!("Failed to generate snapshot URLs: {:?}", e);
                            // URL 생성 실패해도 응답은 계속 진행
                        }
                    }
                } else {
                    tracing::debug!("No snapshots to generate URLs for");
                }
            }

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
    http_req: HttpRequest,
    jwt: web::Data<Arc<JwtService>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
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
                Arc<SignedUrlServiceImpl>,
            >,
        >,
    >,
) -> impl Responder {
    // user_id 추출 (개발 모드 또는 JWT 인증)
    let user_id = match AnnotationController::extract_user_id_with_auth(&http_req, &jwt, &user_repo).await {
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
    http_req: HttpRequest,
    jwt: web::Data<Arc<JwtService>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
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
                Arc<SignedUrlServiceImpl>,
            >,
        >,
    >,
) -> impl Responder {
    // user_id 추출 (개발 모드 또는 JWT 인증)
    let user_id = match AnnotationController::extract_user_id_with_auth(&http_req, &jwt, &user_repo).await {
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
                Arc<SignedUrlServiceImpl>
            >,
        >,
    >,
    http_req: HttpRequest,
    jwt: web::Data<Arc<JwtService>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
) -> impl Responder {
    // 요청한 사용자의 user_id 추출 (개발 모드 + JWT 인증 지원)
    let requesting_user_id = match AnnotationController::extract_user_id_with_auth(&http_req, &jwt, &user_repo).await {
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

#[utoipa::path(
    post,
    path = "/annotations/{annotation_id}/snapshot/upload-url",
    tag = "Annotations",
    request_body = SnapshotUploadUrlRequest,
    responses(
        (status = 200, description = "Upload URL generated successfully", body = SnapshotUploadUrlResponse),
        (status = 401, description = "Unauthorized - Not the annotation owner"),
        (status = 404, description = "Annotation not found"),
        (status = 500, description = "Internal server error"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn generate_snapshot_upload_url<AS, UR, ACS, SUS>(
    path: web::Path<i32>,
    request: web::Json<SnapshotUploadUrlRequest>,
    http_req: HttpRequest,
    jwt_service: web::Data<Arc<JwtService>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
    use_case: web::Data<Arc<AnnotationUseCase<AS, UR, ACS, SUS>>>,
) -> impl Responder
where
    AS: AnnotationService + 'static,
    UR: UserRepository + 'static,
    ACS: AccessControlService + 'static,
    SUS: SignedUrlService + 'static,
{
    let annotation_id = path.into_inner();

    // Extract user_id from JWT token
    let user_id = match extract_user_id_from_request(&http_req, &jwt_service, &user_repo).await {
        Some(id) if id > 0 => id,
        _ => return HttpResponse::Unauthorized().json(json!({
            "error": "Unauthorized",
            "message": "Invalid or missing authentication token"
        })),
    };

    match use_case.generate_snapshot_upload_url(annotation_id, request.into_inner(), user_id).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => AnnotationController::handle_service_error(e),
    }
}

#[utoipa::path(
    post,
    path = "/annotations/{annotation_id}/snapshot/complete-upload",
    tag = "Annotations",
    request_body = CompleteSnapshotUploadRequest,
    responses(
        (status = 200, description = "Snapshot upload completed", body = AnnotationResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Annotation not found"),
        (status = 500, description = "Internal server error"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn complete_snapshot_upload<AS, UR, ACS, SUS>(
    path: web::Path<i32>,
    request: web::Json<CompleteSnapshotUploadRequest>,
    http_req: HttpRequest,
    jwt_service: web::Data<Arc<JwtService>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
    use_case: web::Data<Arc<AnnotationUseCase<AS, UR, ACS, SUS>>>,
) -> impl Responder
where
    AS: AnnotationService + 'static,
    UR: UserRepository + 'static,
    ACS: AccessControlService + 'static,
    SUS: SignedUrlService + 'static,
{
    let annotation_id = path.into_inner();

    // Extract user_id from JWT token
    let user_id = match extract_user_id_from_request(&http_req, &jwt_service, &user_repo).await {
        Some(id) if id > 0 => id,
        _ => return HttpResponse::Unauthorized().json(json!({
            "error": "Unauthorized",
            "message": "Invalid or missing authentication token"
        })),
    };

    match use_case.complete_snapshot_upload(
        annotation_id,
        request.into_inner(),
        user_id
    ).await {
        Ok(annotation) => HttpResponse::Ok().json(annotation),
        Err(e) => AnnotationController::handle_service_error(e),
    }
}

/// 스냅샷 업로드 상태 조회
#[utoipa::path(
    get,
    path = "/annotations/{annotation_id}/snapshot/status",
    tag = "Annotations",
    responses(
        (status = 200, description = "Snapshot status retrieved", body = SnapshotStatusResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Annotation not found"),
        (status = 500, description = "Internal server error"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_snapshot_status<AS, UR, ACS, SUS>(
    path: web::Path<i32>,
    http_req: HttpRequest,
    jwt: web::Data<Arc<JwtService>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
    use_case: web::Data<Arc<AnnotationUseCase<AS, UR, ACS, SUS>>>,
) -> impl Responder
where
    AS: AnnotationService + 'static,
    UR: UserRepository + 'static,
    ACS: AccessControlService + 'static,
    SUS: SignedUrlService + 'static,
{
    let annotation_id = path.into_inner();

    // Extract user_id from JWT token
    let user_id = match extract_user_id_from_request(&http_req, &jwt, &user_repo).await {
        Some(id) if id > 0 => id,
        _ => return HttpResponse::Unauthorized().json(json!({
            "error": "Unauthorized",
            "message": "Invalid or missing authentication token"
        })),
    };

    match use_case.get_snapshot_status(annotation_id, user_id).await {
        Ok(response) => HttpResponse::Ok().json(response),
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
            Arc<SignedUrlServiceImpl>,
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
    jwt_service: Arc<crate::infrastructure::auth::JwtService>,
    user_repo: Arc<UserRepositoryImpl>,
) {
    // cfg 레벨에서 app_data 등록 (scope 밖)
    cfg.app_data(web::Data::new(use_case))
        .app_data(web::Data::new(mask_group_use_case))
        .app_data(web::Data::new(jwt_service))
        .app_data(web::Data::new(user_repo))
        .service(
            web::scope("/annotations")
                .route("/summary", web::get().to(get_annotation_summary))
                .route("/summary", web::head().to(get_annotation_summary))
                .route("", web::post().to(create_annotation))
                .route("", web::get().to(list_annotations))
                .route("", web::head().to(list_annotations))
                .route("/permissions", web::get().to(get_annotation_permissions))
                .route("/{annotation_id}", web::get().to(get_annotation))
                .route("/{annotation_id}", web::head().to(get_annotation))
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
                )
                .route(
                    "/{annotation_id}/snapshot/upload-url",
                    web::post().to(
                        generate_snapshot_upload_url::<
                            crate::domain::services::AnnotationServiceImpl<
                                crate::infrastructure::repositories::AnnotationRepositoryImpl,
                                crate::infrastructure::repositories::UserRepositoryImpl,
                                crate::infrastructure::repositories::ProjectRepositoryImpl,
                            >,
                            crate::infrastructure::repositories::UserRepositoryImpl,
                            crate::domain::services::AccessControlServiceImpl<
                                crate::infrastructure::repositories::AccessLogRepositoryImpl,
                                crate::infrastructure::repositories::UserRepositoryImpl,
                                crate::infrastructure::repositories::ProjectRepositoryImpl,
                                crate::infrastructure::repositories::RoleRepositoryImpl,
                                crate::infrastructure::repositories::PermissionRepositoryImpl,
                            >,
                            Arc<crate::application::services::SignedUrlServiceImpl>,
                        >,
                    ),
                )
                .route(
                    "/{annotation_id}/snapshot/complete-upload",
                    web::post().to(
                        complete_snapshot_upload::<
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
                            Arc<SignedUrlServiceImpl>,
                        >,
                    ),
                )
                .route(
                    "/{annotation_id}/snapshot/status",
                    web::get().to(
                        get_snapshot_status::<
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
                            Arc<SignedUrlServiceImpl>,
                        >,
                    ),
                ),
        );
}
