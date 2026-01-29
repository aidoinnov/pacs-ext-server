#![allow(dead_code, unused_imports, unused_variables)]
use actix_web::{
    http::header::{EntityTag, IF_NONE_MATCH},
    web, HttpRequest, HttpResponse, Responder, Result,
};
use actix_web::http::header::CacheControl;
use actix_web::http::header::CacheDirective;
use serde_json::json;
use std::sync::Arc;

use crate::application::dto::permission_dto::{
    CreateRoleRequest, PaginationQuery, UpdateRoleRequest,
};
use crate::application::dto::role_capability_matrix_dto::*;
use crate::application::use_cases::permission_use_case::PermissionUseCase;
use crate::application::use_cases::RoleCapabilityMatrixUseCase;
use crate::domain::services::permission_service::PermissionService;
use crate::domain::ServiceError;

pub struct RoleController<P: PermissionService> {
    permission_use_case: Arc<PermissionUseCase<P>>,
}

impl<P: PermissionService> RoleController<P> {
    pub fn new(permission_use_case: Arc<PermissionUseCase<P>>) -> Self {
        Self {
            permission_use_case,
        }
    }

    // ===== Role Management APIs =====

    pub async fn create_role(
        permission_use_case: web::Data<Arc<PermissionUseCase<P>>>,
        req: web::Json<CreateRoleRequest>,
    ) -> impl Responder {
        match permission_use_case.create_role(req.into_inner()).await {
            Ok(role) => HttpResponse::Created().json(role),
            Err(e) => HttpResponse::BadRequest().json(json!({
                "error": format!("Failed to create role: {}", e)
            })),
        }
    }

    pub async fn get_role(
        permission_use_case: web::Data<Arc<PermissionUseCase<P>>>,
        role_id: web::Path<i32>,
    ) -> impl Responder {
        match permission_use_case.get_role(*role_id).await {
            Ok(role) => HttpResponse::Ok().json(role),
            Err(e) => HttpResponse::NotFound().json(json!({
                "error": format!("Role not found: {}", e)
            })),
        }
    }

    pub async fn update_role(
        permission_use_case: web::Data<Arc<PermissionUseCase<P>>>,
        role_id: web::Path<i32>,
        req: web::Json<UpdateRoleRequest>,
    ) -> impl Responder {
        match permission_use_case
            .update_role(*role_id, req.into_inner())
            .await
        {
            Ok(role) => HttpResponse::Ok().json(role),
            Err(e) => HttpResponse::BadRequest().json(json!({
                "error": format!("Failed to update role: {}", e)
            })),
        }
    }

    pub async fn delete_role(
        permission_use_case: web::Data<Arc<PermissionUseCase<P>>>,
        role_id: web::Path<i32>,
    ) -> impl Responder {
        match permission_use_case.delete_role(*role_id).await {
            Ok(_) => HttpResponse::Ok().json(json!({
                "message": "Role deleted successfully"
            })),
            Err(e) => HttpResponse::NotFound().json(json!({
                "error": format!("Failed to delete role: {}", e)
            })),
        }
    }

    /// Global 역할 목록 조회 (ETag 캐싱 지원)
    pub async fn get_global_roles(
        permission_use_case: web::Data<Arc<PermissionUseCase<P>>>,
        http_req: HttpRequest,
    ) -> Result<HttpResponse, actix_web::Error> {
        // 1. 먼저 updated_at 조회 (빠른 쿼리)
        let roles_updated_at = match permission_use_case.get_global_roles_updated_at().await {
            Ok(updated_at) => updated_at,
            Err(e) => {
                tracing::error!("Error getting global_roles_updated_at: {:?}", e);
                return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Internal server error",
                    "message": e.to_string()
                })));
            }
        };

        // 2. ETag 생성
        let etag_value = format!("{}", roles_updated_at.timestamp_millis());
        let entity_tag = EntityTag::new_weak(etag_value.clone());

        // 3. If-None-Match 헤더 확인
        if let Some(if_none_match) = http_req.headers().get(IF_NONE_MATCH) {
            if let Ok(client_etag) = if_none_match.to_str() {
                let client_etag_value = client_etag.trim_start_matches("W/\"").trim_end_matches('"');
                if client_etag_value == etag_value {
                    tracing::debug!("Cache hit for get_global_roles");
                    return Ok(HttpResponse::NotModified()
                        .insert_header(actix_web::http::header::ETag(entity_tag))
                        .finish());
                }
            }
        }

        // 4. 캐시 미스 - 전체 데이터 조회
        match permission_use_case.get_global_roles().await {
            Ok(roles) => Ok(HttpResponse::Ok()
                .insert_header(CacheControl(vec![
                    CacheDirective::Private,
                    CacheDirective::MaxAge(300),
                ]))
                .insert_header(actix_web::http::header::ETag(entity_tag))
                .json(roles)),
            Err(ServiceError::DatabaseError(msg)) => {
                tracing::error!("Database error in get_global_roles: {}", msg);
                Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Database Error",
                    "message": msg
                })))
            }
            Err(e) => {
                tracing::error!("Error in get_global_roles: {:?}", e);
                Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Internal Server Error",
                    "message": e.to_string()
                })))
            }
        }
    }

    /// Project 역할 목록 조회 (ETag 캐싱 지원)
    pub async fn get_project_roles(
        permission_use_case: web::Data<Arc<PermissionUseCase<P>>>,
        http_req: HttpRequest,
    ) -> Result<HttpResponse, actix_web::Error> {
        // 1. 먼저 updated_at 조회 (빠른 쿼리)
        let roles_updated_at = match permission_use_case.get_project_roles_updated_at().await {
            Ok(updated_at) => updated_at,
            Err(e) => {
                tracing::error!("Error getting project_roles_updated_at: {:?}", e);
                return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Internal server error",
                    "message": e.to_string()
                })));
            }
        };

        // 2. ETag 생성
        let etag_value = format!("{}", roles_updated_at.timestamp_millis());
        let entity_tag = EntityTag::new_weak(etag_value.clone());

        // 3. If-None-Match 헤더 확인
        if let Some(if_none_match) = http_req.headers().get(IF_NONE_MATCH) {
            if let Ok(client_etag) = if_none_match.to_str() {
                let client_etag_value = client_etag.trim_start_matches("W/\"").trim_end_matches('"');
                if client_etag_value == etag_value {
                    tracing::debug!("Cache hit for get_project_roles");
                    return Ok(HttpResponse::NotModified()
                        .insert_header(actix_web::http::header::ETag(entity_tag))
                        .finish());
                }
            }
        }

        // 4. 캐시 미스 - 전체 데이터 조회
        match permission_use_case.get_project_roles().await {
            Ok(roles) => Ok(HttpResponse::Ok()
                .insert_header(CacheControl(vec![
                    CacheDirective::Private,
                    CacheDirective::MaxAge(300),
                ]))
                .insert_header(actix_web::http::header::ETag(entity_tag))
                .json(roles)),
            Err(ServiceError::DatabaseError(msg)) => {
                tracing::error!("Database error in get_project_roles: {}", msg);
                Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Database Error",
                    "message": msg
                })))
            }
            Err(e) => {
                tracing::error!("Error in get_project_roles: {:?}", e);
                Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Internal Server Error",
                    "message": e.to_string()
                })))
            }
        }
    }

    /// Global 역할 목록 조회 (권한 정보 포함, 페이지네이션)
    pub async fn get_global_roles_with_permissions(
        permission_use_case: web::Data<Arc<PermissionUseCase<P>>>,
        query: web::Query<PaginationQuery>,
    ) -> impl Responder {
        match permission_use_case
            .get_global_roles_with_permissions(query.page, query.page_size)
            .await
        {
            Ok(response) => HttpResponse::Ok().json(response),
            Err(e) => HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to get global roles with permissions: {}", e)
            })),
        }
    }

    // ===== Permission Management APIs =====

    /// 모든 권한 조회 (ETag 캐싱 지원)
    pub async fn get_all_permissions(
        permission_use_case: web::Data<Arc<PermissionUseCase<P>>>,
        http_req: HttpRequest,
    ) -> Result<HttpResponse, actix_web::Error> {
        // 1. 먼저 updated_at 조회 (빠른 쿼리)
        let permissions_updated_at = match permission_use_case.get_all_permissions_updated_at().await {
            Ok(updated_at) => updated_at,
            Err(e) => {
                tracing::error!("Error getting permissions_updated_at: {:?}", e);
                return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Internal server error",
                    "message": e.to_string()
                })));
            }
        };

        // 2. ETag 생성
        let etag_value = format!("{}", permissions_updated_at.timestamp_millis());
        let entity_tag = EntityTag::new_weak(etag_value.clone());

        // 3. If-None-Match 헤더 확인
        if let Some(if_none_match) = http_req.headers().get(IF_NONE_MATCH) {
            if let Ok(client_etag) = if_none_match.to_str() {
                // client_etag는 "W/"1234567890"" 형식이므로 비교 시 따옴표 제거
                let client_etag_value = client_etag.trim_start_matches("W/\"").trim_end_matches('"');
                if client_etag_value == etag_value {
                    tracing::debug!("Cache hit for get_all_permissions");
                    return Ok(HttpResponse::NotModified()
                        .insert_header(actix_web::http::header::ETag(entity_tag))
                        .finish());
                }
            }
        }

        // 4. 캐시 미스 - 전체 데이터 조회
        match permission_use_case.get_all_permissions().await {
            Ok(permissions) => Ok(HttpResponse::Ok()
                .insert_header(CacheControl(vec![
                    CacheDirective::Private,
                    CacheDirective::MaxAge(300),
                ]))
                .insert_header(actix_web::http::header::ETag(entity_tag))
                .json(permissions)),
            Err(ServiceError::DatabaseError(msg)) => {
                tracing::error!("Database error in get_all_permissions: {}", msg);
                Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Database Error",
                    "message": msg
                })))
            }
            Err(e) => {
                tracing::error!("Error in get_all_permissions: {:?}", e);
                Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Internal Server Error",
                    "message": e.to_string()
                })))
            }
        }
    }

    // ===== Role-Capability Matrix APIs =====

    /// 전역 Role-Capability 매트릭스 조회 (페이지네이션 및 검색 포함)
    pub async fn get_global_matrix_paginated(
        query: web::Query<RoleCapabilityMatrixQuery>,
        http_req: HttpRequest,
        use_case: web::Data<Arc<RoleCapabilityMatrixUseCase>>,
    ) -> Result<HttpResponse, actix_web::Error> {
        // 파라미터 검증 및 기본값 설정
        let page = query.page.unwrap_or(1).max(1);
        let size = query.size.unwrap_or(10).min(100).max(1);

        // 1. 먼저 matrix_updated_at 조회 (빠른 쿼리)
        let matrix_updated_at = match use_case.get_matrix_updated_at().await {
            Ok(updated_at) => updated_at,
            Err(e) => {
                tracing::error!("Error getting matrix_updated_at: {:?}", e);
                return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Internal server error",
                    "message": e.to_string()
                })));
            }
        };

        let etag = matrix_updated_at.timestamp().to_string();
        let entity_tag = EntityTag::new_weak(etag.clone());

        // 2. If-None-Match 체크
        if let Some(if_none_match) = http_req.headers().get(IF_NONE_MATCH) {
            if let Ok(client_etag_str) = if_none_match.to_str() {
                // EntityTag::parse로 클라이언트 ETag 파싱
                if let Ok(client_entity_tag) = client_etag_str.parse::<EntityTag>() {
                    if client_entity_tag.weak_eq(&entity_tag) {
                        return Ok(HttpResponse::NotModified()
                            .insert_header(CacheControl(vec![
                                CacheDirective::Private,
                                CacheDirective::MaxAge(5),
                            ]))
                            .insert_header(actix_web::http::header::ETag(entity_tag))
                            .finish());
                    }
                }
            }
        }

        // 3. ETag 불일치 시에만 실제 데이터 조회
        match use_case
            .get_global_matrix_paginated(page, size, query.search.clone(), query.scope.clone())
            .await
        {
            Ok(matrix) => Ok(HttpResponse::Ok()
                .insert_header(CacheControl(vec![
                    CacheDirective::Private,
                    CacheDirective::MaxAge(5),
                ]))
                .insert_header(actix_web::http::header::ETag(entity_tag))
                .json(matrix)),
            Err(ServiceError::DatabaseError(msg)) => {
                tracing::error!("Database error in get_global_matrix_paginated: {}", msg);
                Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Database error",
                    "message": msg
                })))
            }
            Err(e) => {
                tracing::error!("Error in get_global_matrix_paginated: {:?}", e);
                Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Internal server error",
                    "message": e.to_string()
                })))
            }
        }
    }

    /// 전역 Role-Capability 매트릭스 조회 (전체 데이터, 페이지네이션 없음, scope 필터링 지원)
    pub async fn get_global_matrix(
        query: web::Query<RoleCapabilityMatrixQuery>,
        http_req: HttpRequest,
        use_case: web::Data<Arc<RoleCapabilityMatrixUseCase>>,
    ) -> Result<HttpResponse, actix_web::Error> {
        // 1. 먼저 matrix_updated_at 조회 (빠른 쿼리)
        let matrix_updated_at = match use_case.get_matrix_updated_at().await {
            Ok(updated_at) => updated_at,
            Err(e) => {
                tracing::error!("Error getting matrix_updated_at: {:?}", e);
                return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Internal server error",
                    "message": e.to_string()
                })));
            }
        };

        let etag = matrix_updated_at.timestamp().to_string();
        let entity_tag = EntityTag::new_weak(etag.clone());

        // 2. If-None-Match 체크
        if let Some(if_none_match) = http_req.headers().get(IF_NONE_MATCH) {
            if let Ok(client_etag_str) = if_none_match.to_str() {
                if let Ok(client_entity_tag) = client_etag_str.parse::<EntityTag>() {
                    if client_entity_tag.weak_eq(&entity_tag) {
                        return Ok(HttpResponse::NotModified()
                            .insert_header(CacheControl(vec![
                                CacheDirective::Private,
                                CacheDirective::MaxAge(5),
                            ]))
                            .insert_header(actix_web::http::header::ETag(entity_tag))
                            .finish());
                    }
                }
            }
        }

        // 3. ETag 불일치 시에만 실제 데이터 조회
        match use_case.get_global_matrix(query.scope.clone()).await {
            Ok(matrix) => Ok(HttpResponse::Ok()
                .insert_header(CacheControl(vec![
                    CacheDirective::Private,
                    CacheDirective::MaxAge(5),
                ]))
                .insert_header(actix_web::http::header::ETag(entity_tag))
                .json(matrix)),
            Err(ServiceError::DatabaseError(msg)) => {
                tracing::error!("Database error in get_global_matrix: {}", msg);
                Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Database error",
                    "message": msg
                })))
            }
            Err(e) => {
                tracing::error!("Error in get_global_matrix: {:?}", e);
                Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Internal server error",
                    "message": e.to_string()
                })))
            }
        }
    }

    /// 프로젝트별 Role-Capability 매트릭스 조회
    pub async fn get_project_matrix(
        project_id: web::Path<i32>,
        http_req: HttpRequest,
        use_case: web::Data<Arc<RoleCapabilityMatrixUseCase>>,
    ) -> Result<HttpResponse, actix_web::Error> {
        // 1. 먼저 matrix_updated_at 조회 (빠른 쿼리)
        let matrix_updated_at = match use_case.get_matrix_updated_at().await {
            Ok(updated_at) => updated_at,
            Err(e) => {
                tracing::error!("Error getting matrix_updated_at: {:?}", e);
                return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Internal server error",
                    "message": e.to_string()
                })));
            }
        };

        let etag = matrix_updated_at.timestamp().to_string();
        let entity_tag = EntityTag::new_weak(etag.clone());

        // 2. If-None-Match 체크
        if let Some(if_none_match) = http_req.headers().get(IF_NONE_MATCH) {
            if let Ok(client_etag_str) = if_none_match.to_str() {
                if let Ok(client_entity_tag) = client_etag_str.parse::<EntityTag>() {
                    if client_entity_tag.weak_eq(&entity_tag) {
                        return Ok(HttpResponse::NotModified()
                            .insert_header(CacheControl(vec![
                                CacheDirective::Private,
                                CacheDirective::MaxAge(5),
                            ]))
                            .insert_header(actix_web::http::header::ETag(entity_tag))
                            .finish());
                    }
                }
            }
        }

        // 3. ETag 불일치 시에만 실제 데이터 조회
        match use_case.get_project_matrix(*project_id).await {
            Ok(matrix) => Ok(HttpResponse::Ok()
                .insert_header(CacheControl(vec![
                    CacheDirective::Private,
                    CacheDirective::MaxAge(5),
                ]))
                .insert_header(actix_web::http::header::ETag(entity_tag))
                .json(matrix)),
            Err(ServiceError::DatabaseError(msg)) => {
                tracing::error!("Database error in get_project_matrix: {}", msg);
                Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Database error",
                    "message": msg
                })))
            }
            Err(e) => {
                tracing::error!("Error in get_project_matrix: {:?}", e);
                Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Internal server error",
                    "message": e.to_string()
                })))
            }
        }
    }

    /// 역할에 Capability 할당/제거
    pub async fn update_capability_assignment(
        path: web::Path<(i32, i32)>,
        req: web::Json<CapabilityAssignmentRequest>,
        use_case: web::Data<Arc<RoleCapabilityMatrixUseCase>>,
    ) -> Result<HttpResponse, actix_web::Error> {
        let (role_id, capability_id) = path.into_inner();

        match use_case
            .update_capability_assignment(role_id, capability_id, req.assign)
            .await
        {
            Ok(_) => {
                let message = if req.assign {
                    "Capability assigned successfully"
                } else {
                    "Capability removed successfully"
                };
                Ok(HttpResponse::Ok().json(serde_json::json!({
                    "message": message
                })))
            }
            Err(ServiceError::NotFound(msg)) => {
                tracing::warn!("Not found in update_capability_assignment: {}", msg);
                Ok(HttpResponse::NotFound().json(serde_json::json!({
                    "error": "Not found",
                    "message": msg
                })))
            }
            Err(ServiceError::DatabaseError(msg)) => {
                tracing::error!("Database error in update_capability_assignment: {}", msg);
                Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Database error",
                    "message": msg
                })))
            }
            Err(e) => {
                tracing::error!("Error in update_capability_assignment: {:?}", e);
                Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Internal server error",
                    "message": e.to_string()
                })))
            }
        }
    }
}

// ===== Capability Management APIs =====

/// 모든 Capability 목록 조회
pub async fn get_all_capabilities(
    http_req: HttpRequest,
    use_case: web::Data<Arc<RoleCapabilityMatrixUseCase>>,
) -> Result<HttpResponse, actix_web::Error> {
    // 1. 먼저 updated_at 조회 (빠른 쿼리)
    let capabilities_updated_at = match use_case.get_all_capabilities_updated_at().await {
        Ok(updated_at) => updated_at,
        Err(e) => {
            tracing::error!("Error getting capabilities_updated_at: {:?}", e);
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Internal server error",
                "message": e.to_string()
            })));
        }
    };

    // 2. ETag 생성
    let etag_value = format!("{}", capabilities_updated_at.timestamp_millis());
    let entity_tag = EntityTag::new_weak(etag_value.clone());

    // 3. If-None-Match 헤더 확인
    if let Some(if_none_match) = http_req.headers().get(IF_NONE_MATCH) {
        if let Ok(client_etag) = if_none_match.to_str() {
            // client_etag는 "W/"1234567890"" 형식이므로 비교 시 따옴표 제거
            let client_etag_value = client_etag.trim_start_matches("W/\"").trim_end_matches('"');
            if client_etag_value == etag_value {
                tracing::debug!("Cache hit for get_all_capabilities");
                return Ok(HttpResponse::NotModified()
                    .insert_header(actix_web::http::header::ETag(entity_tag))
                    .finish());
            }
        }
    }

    // 4. 캐시 미스 - 전체 데이터 조회
    match use_case.get_all_capabilities().await {
        Ok(capabilities) => Ok(HttpResponse::Ok()
            .insert_header(CacheControl(vec![
                CacheDirective::Private,
                CacheDirective::MaxAge(60),
            ]))
            .insert_header(actix_web::http::header::ETag(entity_tag))
            .json(capabilities)),
        Err(ServiceError::DatabaseError(msg)) => {
            tracing::error!("Database error in get_all_capabilities: {}", msg);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Database error",
                "message": msg
            })))
        }
        Err(e) => {
            tracing::error!("Error in get_all_capabilities: {:?}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Internal server error",
                "message": e.to_string()
            })))
        }
    }
}

/// 특정 Capability 상세 조회
pub async fn get_capability_detail(
    capability_id: web::Path<i32>,
    http_req: HttpRequest,
    use_case: web::Data<Arc<RoleCapabilityMatrixUseCase>>,
) -> Result<HttpResponse, actix_web::Error> {
    let cap_id = *capability_id;

    // 1. 먼저 updated_at 조회 (빠른 쿼리)
    let detail_updated_at = match use_case.get_capability_detail_updated_at(cap_id).await {
        Ok(updated_at) => updated_at,
        Err(ServiceError::NotFound(msg)) => {
            tracing::warn!("Capability not found: {}", msg);
            return Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "Not found",
                "message": msg
            })));
        }
        Err(e) => {
            tracing::error!("Error getting capability_detail_updated_at: {:?}", e);
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Internal server error",
                "message": e.to_string()
            })));
        }
    };

    // 2. ETag 생성
    let etag_value = format!("{}", detail_updated_at.timestamp_millis());
    let entity_tag = EntityTag::new_weak(etag_value.clone());

    // 3. If-None-Match 헤더 확인
    if let Some(if_none_match) = http_req.headers().get(IF_NONE_MATCH) {
        if let Ok(client_etag) = if_none_match.to_str() {
            // client_etag는 "W/"1234567890"" 형식이므로 비교 시 따옴표 제거
            let client_etag_value = client_etag.trim_start_matches("W/\"").trim_end_matches('"');
            if client_etag_value == etag_value {
                tracing::debug!("Cache hit for get_capability_detail: {}", cap_id);
                return Ok(HttpResponse::NotModified()
                    .insert_header(actix_web::http::header::ETag(entity_tag))
                    .finish());
            }
        }
    }

    // 4. 캐시 미스 - 전체 데이터 조회
    match use_case.get_capability_detail(cap_id).await {
        Ok(capability) => Ok(HttpResponse::Ok()
            .insert_header(CacheControl(vec![
                CacheDirective::Private,
                CacheDirective::MaxAge(60),
            ]))
            .insert_header(actix_web::http::header::ETag(entity_tag))
            .json(capability)),
        Err(ServiceError::NotFound(msg)) => {
            tracing::warn!("Capability not found: {}", msg);
            Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "Not found",
                "message": msg
            })))
        }
        Err(ServiceError::DatabaseError(msg)) => {
            tracing::error!("Database error in get_capability_detail: {}", msg);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Database error",
                "message": msg
            })))
        }
        Err(e) => {
            tracing::error!("Error in get_capability_detail: {:?}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Internal server error",
                "message": e.to_string()
            })))
        }
    }
}

/// 카테고리별 Capability 목록 조회
pub async fn get_capabilities_by_category(
    category: web::Path<String>,
    http_req: HttpRequest,
    use_case: web::Data<Arc<RoleCapabilityMatrixUseCase>>,
) -> Result<HttpResponse, actix_web::Error> {
    // 1. 먼저 updated_at 조회 (빠른 쿼리)
    let category_updated_at = match use_case
        .get_capabilities_by_category_updated_at(&category)
        .await
    {
        Ok(updated_at) => updated_at,
        Err(e) => {
            tracing::error!("Error getting capabilities_by_category_updated_at: {:?}", e);
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Internal server error",
                "message": e.to_string()
            })));
        }
    };

    // 2. ETag 생성
    let etag_value = format!("{}", category_updated_at.timestamp_millis());
    let entity_tag = EntityTag::new_weak(etag_value.clone());

    // 3. If-None-Match 헤더 확인
    if let Some(if_none_match) = http_req.headers().get(IF_NONE_MATCH) {
        if let Ok(client_etag) = if_none_match.to_str() {
            // client_etag는 "W/"1234567890"" 형식이므로 비교 시 따옴표 제거
            let client_etag_value = client_etag.trim_start_matches("W/\"").trim_end_matches('"');
            if client_etag_value == etag_value {
                tracing::debug!("Cache hit for get_capabilities_by_category: {}", category.as_str());
                return Ok(HttpResponse::NotModified()
                    .insert_header(actix_web::http::header::ETag(entity_tag))
                    .finish());
            }
        }
    }

    // 4. 캐시 미스 - 전체 데이터 조회
    match use_case.get_capabilities_by_category(&category).await {
        Ok(capabilities) => Ok(HttpResponse::Ok()
            .insert_header(CacheControl(vec![
                CacheDirective::Private,
                CacheDirective::MaxAge(60),
            ]))
            .insert_header(actix_web::http::header::ETag(entity_tag))
            .json(capabilities)),
        Err(ServiceError::DatabaseError(msg)) => {
            tracing::error!("Database error in get_capabilities_by_category: {}", msg);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Database error",
                "message": msg
            })))
        }
        Err(e) => {
            tracing::error!("Error in get_capabilities_by_category: {:?}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Internal server error",
                "message": e.to_string()
            })))
        }
    }
}

// ===== Route Configuration =====

pub fn configure_routes<P: PermissionService + 'static>(
    cfg: &mut web::ServiceConfig,
    permission_use_case: Arc<PermissionUseCase<P>>,
    role_capability_use_case: Arc<RoleCapabilityMatrixUseCase>,
) {
    cfg.app_data(web::Data::new(permission_use_case))
        .app_data(web::Data::new(role_capability_use_case))
        .service(
            web::scope("/roles")
                // Role Management
                .route("", web::post().to(RoleController::<P>::create_role))
                .route(
                    "/global",
                    web::get().to(RoleController::<P>::get_global_roles),
                )
                .route(
                    "/global/with-permissions",
                    web::get().to(RoleController::<P>::get_global_roles_with_permissions),
                )
                .route(
                    "/project",
                    web::get().to(RoleController::<P>::get_project_roles),
                )
                // Role-Capability Matrix (specific routes first)
                .route(
                    "/global/capabilities/matrix",
                    web::get().to(RoleController::<P>::get_global_matrix_paginated),
                )
                .route(
                    "/global/capabilities/matrix/all",
                    web::get().to(RoleController::<P>::get_global_matrix),
                )
                .route(
                    "/projects/{project_id}/capabilities/matrix",
                    web::get().to(RoleController::<P>::get_project_matrix),
                )
                .route(
                    "/{role_id}/capabilities/{capability_id}",
                    web::put().to(RoleController::<P>::update_capability_assignment),
                )
                // Generic role routes (must be last)
                .route("/{role_id}", web::get().to(RoleController::<P>::get_role))
                .route(
                    "/{role_id}",
                    web::put().to(RoleController::<P>::update_role),
                )
                .route(
                    "/{role_id}",
                    web::delete().to(RoleController::<P>::delete_role),
                ),
        )
        .service(
            web::scope("/capabilities")
                .route("", web::get().to(get_all_capabilities))
                .route("/{capability_id}", web::get().to(get_capability_detail))
                .route(
                    "/category/{category}",
                    web::get().to(get_capabilities_by_category),
                ),
        )
        .service(
            web::scope("/permissions")
                .route("", web::get().to(RoleController::<P>::get_all_permissions)),
        );
}
