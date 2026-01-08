use actix_web::{web, HttpRequest, HttpResponse};
use base64::Engine;
use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;
use sqlx::PgPool;

use crate::domain::entities::access_condition::AccessCondition;
use crate::domain::repositories::{AccessConditionRepository, ProjectDataRepository, UserRepository, StudyListViewRepository, AnnotationRepository};
use crate::domain::services::DicomRbacEvaluator;
use crate::infrastructure::auth::{JwtService, extract_user_id_from_request, decode_keycloak_token_sub};
use crate::presentation::controllers::annotation_controller::AnnotationController;
use crate::infrastructure::external::Dcm4cheeQidoClient;
use crate::infrastructure::repositories::{AccessConditionRepositoryImpl, ProjectDataRepositoryImpl, UserRepositoryImpl, StudyListViewRepositoryImpl, AnnotationRepositoryImpl};
use crate::infrastructure::services::DicomRbacEvaluatorImpl;
use std::collections::HashMap;
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct GatewayQuery {
    #[serde(default)]
    pub project_id: Option<i32>,
    #[serde(default)]
    pub report_status: Option<String>, // "approved,unread" 형식
    #[serde(default)]
    pub view: Option<String>, // Study List View ID (e.g., "default")
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

pub fn configure_routes(
    cfg: &mut web::ServiceConfig,
    qido_client: Dcm4cheeQidoClient,
    evaluator: Arc<DicomRbacEvaluatorImpl>,
    jwt_service: Arc<JwtService>,
    access_condition_repo: Arc<AccessConditionRepositoryImpl>,
    user_repo: Arc<UserRepositoryImpl>,
    project_data_repo: Arc<ProjectDataRepositoryImpl>,
    study_list_view_repo: Arc<StudyListViewRepositoryImpl>,
    annotation_repo: Arc<AnnotationRepositoryImpl>,
    pool: PgPool,
) {
    // 공통 app_data 설정
    let shared_data = (
        web::Data::new(qido_client),
        web::Data::new(evaluator),
        web::Data::new(jwt_service),
        web::Data::new(access_condition_repo),
        web::Data::new(user_repo),
        web::Data::new(project_data_repo),
        web::Data::new(study_list_view_repo),
        web::Data::new(annotation_repo),
        web::Data::new(pool),
    );

    cfg
        // ========================================
        // 👤 사용자 관점 API (/me/dicom/*)
        // ========================================
        .service(
            web::scope("/me/dicom")
                .app_data(shared_data.0.clone())
                .app_data(shared_data.1.clone())
                .app_data(shared_data.2.clone())
                .app_data(shared_data.3.clone())
                .app_data(shared_data.4.clone())
                .app_data(shared_data.5.clone())
                .app_data(shared_data.6.clone())
                .app_data(shared_data.7.clone())
                .app_data(shared_data.8.clone())
                .route("/studies", web::get().to(get_all_user_studies))
                .route("/series", web::get().to(get_all_user_series))
                .route("/studies/{study_uid}/series", web::get().to(get_user_study_series))
        )
        // ========================================
        // 🔐 관리자 관점 API (/admin/dicom/*)
        // ========================================
        .service(
            web::scope("/admin/dicom")
                .app_data(shared_data.0.clone())
                .app_data(shared_data.1.clone())
                .app_data(shared_data.2.clone())
                .app_data(shared_data.3.clone())
                .app_data(shared_data.4.clone())
                .app_data(shared_data.5.clone())
                .app_data(shared_data.6.clone())
                .app_data(shared_data.7.clone())
                .app_data(shared_data.8.clone())
                .route("/studies", web::get().to(get_admin_studies))
                .route("/series", web::get().to(get_admin_series))
                .route("/studies/{study_uid}/series", web::get().to(get_admin_study_series))
        )
        // ========================================
        // 📋 프로젝트별 DICOM API (/dicom/*)
        // ========================================
        .service(
            web::scope("/dicom")
                .app_data(shared_data.0)
                .app_data(shared_data.1)
                .app_data(shared_data.2)
                .app_data(shared_data.3)
                .app_data(shared_data.4)
                .app_data(shared_data.5)
                .app_data(shared_data.6)
                .app_data(shared_data.7)
                .app_data(shared_data.8)
                .route(
                    "/ping",
                    web::get().to(|| async { HttpResponse::Ok().finish() }),
                )
                .route("/studies_raw", web::get().to(get_studies_raw))
                .route("/deps", web::get().to(debug_deps))
                .route("/patients", web::get().to(get_patients))
                .route("/studies", web::get().to(get_studies)) // project_id 필수
                .route("/series", web::get().to(get_series_all))
                .route("/studies/{study_uid}/series", web::get().to(get_series))
                .route(
                    "/studies/{study_uid}/series/{series_uid}/instances",
                    web::get().to(get_instances),
                ),
        );
}
pub async fn get_studies_raw(
    qido: web::Data<Dcm4cheeQidoClient>,
    req: HttpRequest,
) -> HttpResponse {
    let bearer_opt = extract_bearer_token(&req);
    match qido
        .qido_studies_with_bearer(
            bearer_opt.as_deref(),
            vec![("limit".to_string(), "1".to_string())],
        )
        .await
    {
        Ok(json) => HttpResponse::Ok().json(json),
        Err(e) => HttpResponse::BadGateway().json(serde_json::json!({"error": e.to_string()})),
    }
}

async fn debug_deps(req: HttpRequest) -> HttpResponse {
    let has_qido = req.app_data::<web::Data<Dcm4cheeQidoClient>>().is_some();
    let has_eval = req
        .app_data::<web::Data<Arc<DicomRbacEvaluatorImpl>>>()
        .is_some();
    let has_eval_plain = req
        .app_data::<web::Data<DicomRbacEvaluatorImpl>>()
        .is_some();
    let has_jwt = req.app_data::<web::Data<Arc<JwtService>>>().is_some();
    let has_jwt_plain = req.app_data::<web::Data<JwtService>>().is_some();
    let has_ac = req
        .app_data::<web::Data<Arc<AccessConditionRepositoryImpl>>>()
        .is_some();
    let has_ac_plain = req
        .app_data::<web::Data<AccessConditionRepositoryImpl>>()
        .is_some();
    HttpResponse::Ok().json(serde_json::json!({
        "qido": has_qido,
        "evaluator": has_eval,
        "evaluator_plain": has_eval_plain,
        "jwt": has_jwt,
        "jwt_plain": has_jwt_plain,
        "access_condition_repo": has_ac,
        "access_condition_repo_plain": has_ac_plain,
    }))
}

/// 사용자가 DICOM 전체 접근 권한을 가지고 있는지 확인
async fn has_global_dicom_access(user_id: i32, pool: &sqlx::PgPool) -> bool {
    sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(
            SELECT 1 FROM security_user_project sup
            INNER JOIN security_role r ON sup.role_id = r.id
            INNER JOIN security_role_capability src ON r.id = src.role_id
            INNER JOIN security_capability c ON src.capability_id = c.id
            WHERE sup.user_id = $1
              AND c.name = 'DICOM_GLOBAL_ACCESS'
        )"
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
    .unwrap_or(false)
}

/// 사용자가 특정 Study에 접근 가능한지 확인 (project_data_access 테이블 기반)
///
/// 로직:
/// 1. project_data_access 테이블에 레코드가 없으면 → 전체 접근 가능 (기본)
/// 2. 레코드가 있으면 → 해당 레코드의 status와 expires_at 확인
///    - status = 'APPROVED' AND (expires_at IS NULL OR expires_at > NOW()) → 접근 가능
///    - 그 외 → 접근 불가
pub async fn can_access_study(
    user_id: i32,
    project_id: i32,
    study_uid: &str,
    pool: &sqlx::PgPool,
) -> bool {
    // 1. project_data_access 테이블에 레코드가 있는지 확인
    let has_access_record: bool = sqlx::query_scalar(
        "SELECT EXISTS(
            SELECT 1 FROM project_data_access
            WHERE user_id = $1 AND project_id = $2
        )",
    )
    .bind(user_id)
    .bind(project_id)
    .fetch_one(pool)
    .await
    .unwrap_or(false);

    // 2. 레코드가 없으면 → 전체 접근 가능 (기본)
    if !has_access_record {
        tracing::debug!(
            "No access restrictions for user {} in project {} → Full access granted",
            user_id,
            project_id
        );
        return true;
    }

    // 3. 레코드가 있으면 → 해당 Study에 대한 접근 권한 확인
    let is_approved: bool = sqlx::query_scalar(
        "SELECT EXISTS(
            SELECT 1 FROM project_data_access pda
            INNER JOIN project_data_study pds ON pda.study_id = pds.id
            WHERE pda.user_id = $1
              AND pda.project_id = $2
              AND pds.study_uid = $3
              AND pda.status = 'APPROVED'
              AND (pda.expires_at IS NULL OR pda.expires_at > NOW())
        )",
    )
    .bind(user_id)
    .bind(project_id)
    .bind(study_uid)
    .fetch_one(pool)
    .await
    .unwrap_or(false);

    if is_approved {
        tracing::debug!(
            "User {} has approved access to study {} in project {}",
            user_id,
            study_uid,
            project_id
        );
    } else {
        tracing::debug!(
            "User {} does NOT have access to study {} in project {} (restricted)",
            user_id,
            study_uid,
            project_id
        );
    }

    is_approved
}

/// Study가 특정 프로젝트에 할당되어 있는지 확인
async fn check_study_assignment(study_uid: &str, project_id: i32, pool: &sqlx::PgPool) -> bool {
    sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(
            SELECT 1
            FROM project_data pd
            INNER JOIN project_data_study pds ON pd.study_id = pds.id
            WHERE pd.project_id = $1
              AND pds.study_uid = $2
        )"
    )
    .bind(project_id)
    .bind(study_uid)
    .fetch_one(pool)
    .await
    .unwrap_or(false)
}

pub async fn get_studies(
    qido: web::Data<Dcm4cheeQidoClient>,
    evaluator: web::Data<Arc<DicomRbacEvaluatorImpl>>,
    jwt: web::Data<Arc<JwtService>>,
    access_condition_repo: web::Data<Arc<AccessConditionRepositoryImpl>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
    project_data_repo: web::Data<Arc<ProjectDataRepositoryImpl>>,
    query: web::Query<GatewayQuery>,
    req: HttpRequest,
) -> HttpResponse {
    // 사용자 ID 추출
    let user_id = match extract_user_id_from_request(&req, &jwt, &user_repo).await {
        Some(id) if id > 0 => id,
        _ => {
            tracing::warn!("Gateway: Unauthorized - failed to extract user_id from token");
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Invalid or missing authorization token"
            }));
        }
    };

    // 전체 데이터 조회 권한 확인
    let has_global_access = has_global_dicom_access(user_id, project_data_repo.pool()).await;

    // 프로젝트 ID 검증
    let project_id_opt = query.project_id;

    // check_assignment_for_project 파라미터 추출 (숫자 또는 문자열로 전달될 수 있음)
    let check_assignment_project_id = query.extra.get("check_assignment_for_project")
        .and_then(|v| {
            // 숫자로 전달된 경우
            if let Some(num) = v.as_i64() {
                return Some(num as i32);
            }
            // 문자열로 전달된 경우
            if let Some(s) = v.as_str() {
                return s.parse::<i32>().ok();
            }
            None
        });

    tracing::debug!("Gateway: check_assignment_project_id = {:?}", check_assignment_project_id);

    // 전체 데이터 조회 권한이 없으면 project_id 필수
    if !has_global_access && project_id_opt.is_none() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "project_id is required (no global access permission)"
        }));
    }

    // project_id가 있으면 검증
    if let Some(id) = project_id_opt {
        if id <= 0 {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "project_id must be greater than 0"
            }));
        }
    }

    // check_assignment_for_project가 있으면 검증
    if let Some(id) = check_assignment_project_id {
        if id <= 0 {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "check_assignment_for_project must be greater than 0"
            }));
        }
    }

    // 1. 규칙 기반 QIDO 파라미터 병합 + 사용자 입력 우선 병합
    // 사용자 필터/페이지네이션 파라미터 파싱 및 검증
    // report_status는 extra에서 제거 (serde(flatten)으로 인해 포함될 수 있음)
    let mut extra_for_qido = query.extra.clone();
    extra_for_qido.remove("report_status");
    let user_params = match build_qido_params_from_user_query(&extra_for_qido) {
        Ok(p) => p,
        Err(msg) => {
            return HttpResponse::BadRequest().json(serde_json::json!({"error": msg}));
        }
    };

    tracing::debug!("Gateway: User params: {:?}", user_params);

    // Access Condition은 project_id가 있을 때만 적용
    let qido_params = if let Some(pid) = project_id_opt {
        if let Ok(conditions) = access_condition_repo.list_by_project(pid).await {
            tracing::debug!("Gateway: Found {} access conditions for project {}", conditions.len(), pid);
            let rule_params = build_qido_params_from_conditions(&conditions);
            tracing::debug!("Gateway: Rule params from conditions: {:?}", rule_params);
            let merged = merge_qido_params(rule_params, user_params); // 사용자 입력이 우선
            tracing::debug!("Gateway: Merged QIDO params: {:?}", merged);
            merged
        } else {
            tracing::debug!("Gateway: No access conditions found for project {}, using user params only", pid);
            user_params
        }
    } else {
        tracing::debug!("Gateway: No project_id provided (global access), using user params only");
        user_params
    };

    // 2. Dcm4chee QIDO 호출
    let bearer_opt = extract_bearer_token(&req);

    // 디버깅: Authorization 헤더 확인
    if let Some(token) = &bearer_opt {
        tracing::debug!("Gateway: Extracted Bearer token (length: {})", token.len());
        tracing::debug!(
            "Gateway: Token preview: {}...",
            &token[..std::cmp::min(50, token.len())]
        );
    } else {
        tracing::warn!("Gateway: No Bearer token found in Authorization header");
        if let Some(auth_header) = req
            .headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
        {
            tracing::debug!(
                "Gateway: Authorization header value: {}...",
                &auth_header[..std::cmp::min(100, auth_header.len())]
            );
        }
    }

    let qido_response = match qido
        .qido_studies_with_bearer(bearer_opt.as_deref(), qido_params)
        .await
    {
        Ok(json) => json,
        Err(e) => {
            return HttpResponse::BadGateway().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    // 3. RBAC 필터링 적용 (기존 RBAC + project_data_access 테이블 확인)
    let filtered = if has_global_access && project_id_opt.is_none() {
        // 전체 데이터 조회 권한이 있고 project_id가 없으면 필터링 안 함
        tracing::debug!("Gateway: Global access granted, skipping RBAC filtering");
        qido_response
    } else if let Some(pid) = project_id_opt {
        // project_id가 있으면 RBAC 필터링 적용
        if let Some(array) = qido_response.as_array() {
            let mut allowed_items = Vec::new();
            let mut study_uids_seen = std::collections::HashSet::new();
            for item in array.iter() {
                if let Some(study_uid) = extract_study_uid(item) {
                    // 중복 제거
                    if study_uids_seen.contains(&study_uid) {
                        continue;
                    }

                    // 기존 RBAC 평가
                    let result = evaluator
                        .evaluate_study_uid(user_id, pid, &study_uid)
                        .await;

                    // project_data_access 테이블 확인 (추가 제약)
                    let has_data_access = can_access_study(
                        user_id,
                        pid,
                        &study_uid,
                        project_data_repo.pool(),
                    )
                    .await;

                    // 두 조건 모두 만족해야 접근 가능
                    if result.allowed && has_data_access {
                        study_uids_seen.insert(study_uid.clone());
                        allowed_items.push(item.clone());
                    } else if !has_data_access {
                        tracing::debug!(
                            "Gateway: Study {} filtered out by project_data_access restrictions",
                            study_uid
                        );
                    }
                }
            }
            serde_json::Value::Array(allowed_items)
        } else {
            qido_response
        }
    } else {
        // 이 경우는 발생하지 않아야 함 (위에서 검증됨)
        qido_response
    };

    // 4. check_assignment_for_project 파라미터가 있으면 할당 여부 확인
    let final_response = if let Some(check_pid) = check_assignment_project_id {
        tracing::debug!("Gateway: Checking assignment for project_id={}", check_pid);
        if let Some(array) = filtered.as_array() {
            tracing::debug!("Gateway: Processing {} studies for assignment check", array.len());
            let mut enriched_items = Vec::new();
            for item in array.iter() {
                if let Some(study_uid) = extract_study_uid(item) {
                    // DB에서 해당 Study가 프로젝트에 할당되어 있는지 확인
                    let is_assigned = check_study_assignment(
                        &study_uid,
                        check_pid,
                        project_data_repo.pool()
                    ).await;

                    tracing::debug!("Gateway: Study {} is_assigned={}", study_uid, is_assigned);

                    // 기존 item에 is_assigned와 checked_project_id 필드 추가
                    let mut enriched_item = item.clone();
                    if let Some(obj) = enriched_item.as_object_mut() {
                        obj.insert("is_assigned".to_string(), serde_json::json!(is_assigned));
                        obj.insert("checked_project_id".to_string(), serde_json::json!(check_pid));
                        tracing::debug!("Gateway: Added is_assigned and checked_project_id fields");
                    }
                    enriched_items.push(enriched_item);
                } else {
                    // Study UID를 추출할 수 없으면 그대로 추가
                    tracing::warn!("Gateway: Could not extract study_uid from item");
                    enriched_items.push(item.clone());
                }
            }
            tracing::debug!("Gateway: Returning {} enriched studies", enriched_items.len());
            serde_json::Value::Array(enriched_items)
        } else {
            tracing::warn!("Gateway: filtered response is not an array");
            filtered
        }
    } else {
        tracing::debug!("Gateway: No check_assignment_project_id, skipping assignment check");
        filtered
    };

    HttpResponse::Ok().json(final_response)
}

/// 관리자용 전체 스터디 목록 조회 (전역 접근 권한 필요)
pub async fn get_admin_studies(
    qido: web::Data<Dcm4cheeQidoClient>,
    evaluator: web::Data<Arc<DicomRbacEvaluatorImpl>>,
    jwt: web::Data<Arc<JwtService>>,
    access_condition_repo: web::Data<Arc<AccessConditionRepositoryImpl>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
    project_data_repo: web::Data<Arc<ProjectDataRepositoryImpl>>,
    query: web::Query<GatewayQuery>,
    req: HttpRequest,
) -> HttpResponse {
    // 사용자 ID 추출
    let user_id = match extract_user_id_from_request(&req, &jwt, &user_repo).await {
        Some(id) if id > 0 => id,
        _ => {
            tracing::warn!("Gateway: Unauthorized - failed to extract user_id from token");
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Invalid or missing authorization token"
            }));
        }
    };

    // 전역 접근 권한 확인
    let has_global_access = has_global_dicom_access(user_id, project_data_repo.pool()).await;
    if !has_global_access {
        tracing::warn!("Gateway: User {} attempted to access admin endpoint without global access", user_id);
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Global access permission required",
            "message": "DICOM_GLOBAL_ACCESS capability is required to access this endpoint"
        }));
    }

    // 사용자 필터/페이지네이션 파라미터 파싱
    let mut extra_for_qido = query.extra.clone();
    extra_for_qido.remove("report_status");
    let user_params = match build_qido_params_from_user_query(&extra_for_qido) {
        Ok(p) => p,
        Err(msg) => {
            return HttpResponse::BadRequest().json(serde_json::json!({"error": msg}));
        }
    };

    tracing::debug!("Gateway: Admin endpoint - User params: {:?}", user_params);

    // Access Condition은 적용하지 않음 (전체 데이터 조회)
    let qido_params = user_params;

    // Dcm4chee QIDO 호출
    let bearer_opt = extract_bearer_token(&req);

    let qido_response = match qido
        .qido_studies_with_bearer(bearer_opt.as_deref(), qido_params)
        .await
    {
        Ok(json) => json,
        Err(e) => {
            return HttpResponse::BadGateway().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    // 전역 접근 권한이 있으면 필터링 없이 반환
    tracing::debug!("Gateway: Admin endpoint - Returning all studies without RBAC filtering");
    HttpResponse::Ok().json(qido_response)
}

/// 사용자가 속한 모든 프로젝트의 스터디 목록을 통합 조회
/// project_id 파라미터가 있으면 해당 프로젝트만 필터링
/// view 파라미터가 있으면 해당 View의 필드 정의에 따라 확장 필드 포함
pub async fn get_all_user_studies(
    qido: web::Data<Dcm4cheeQidoClient>,
    evaluator: web::Data<Arc<DicomRbacEvaluatorImpl>>,
    jwt: web::Data<Arc<JwtService>>,
    access_condition_repo: web::Data<Arc<AccessConditionRepositoryImpl>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
    project_data_repo: web::Data<Arc<ProjectDataRepositoryImpl>>,
    study_list_view_repo: web::Data<Arc<StudyListViewRepositoryImpl>>,
    annotation_repo: web::Data<Arc<AnnotationRepositoryImpl>>,
    pool: web::Data<PgPool>,
    query: web::Query<GatewayQuery>,
    req: HttpRequest,
) -> HttpResponse {
    // 사용자 ID 추출
    let user_id = match extract_user_id_from_request(&req, &jwt, &user_repo).await {
        Some(id) if id > 0 => id,
        _ => {
            tracing::warn!("Gateway: Unauthorized - failed to extract user_id from token");
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Invalid or missing authorization token"
            }));
        }
    };

    // project_id 파라미터 확인
    let project_id_filter = query.project_id;

    // 사용자가 속한 프로젝트 조회
    let user_projects = if let Some(filter_pid) = project_id_filter {
        // 특정 프로젝트만 필터링하는 경우
        // 사용자가 해당 프로젝트의 멤버인지 확인
        let is_member = match sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM security_user_project WHERE user_id = $1 AND project_id = $2)"
        )
        .bind(user_id)
        .bind(filter_pid)
        .fetch_one(project_data_repo.pool())
        .await
        {
            Ok(member) => member,
            Err(e) => {
                tracing::error!("Gateway: Failed to check project membership: {:?}", e);
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Failed to check project membership"
                }));
            }
        };

        if !is_member {
            tracing::warn!("Gateway: User {} is not a member of project {}", user_id, filter_pid);
            return HttpResponse::Forbidden().json(serde_json::json!({
                "error": "Access denied",
                "message": format!("User is not a member of project {}", filter_pid)
            }));
        }

        vec![filter_pid]
    } else {
        // 모든 프로젝트 조회
        match sqlx::query_scalar::<_, i32>(
            "SELECT DISTINCT project_id FROM security_user_project WHERE user_id = $1 ORDER BY project_id"
        )
        .bind(user_id)
        .fetch_all(project_data_repo.pool())
        .await
        {
            Ok(projects) => projects,
            Err(e) => {
                tracing::error!("Gateway: Failed to fetch user projects: {:?}", e);
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Failed to fetch user projects"
                }));
            }
        }
    };

    if user_projects.is_empty() {
        tracing::debug!("Gateway: User {} has no projects", user_id);
        return HttpResponse::Ok()
            .insert_header(("X-Total-Count", "0"))
            .insert_header(("X-Page", "1"))
            .insert_header(("X-Page-Size", "50"))
            .insert_header(("X-Total-Pages", "0"))
            .json(serde_json::json!([]));
    }

    tracing::debug!("Gateway: User {} querying {} projects", user_id, user_projects.len());

    // View 필드 조회 (view 파라미터가 있는 경우)
    let view_fields = if let Some(ref view_id) = query.view {
        match study_list_view_repo.find_view_fields(view_id).await {
            Ok(fields) => Some(fields),
            Err(e) => {
                tracing::warn!("Gateway: Failed to fetch view fields for '{}': {:?}", view_id, e);
                None
            }
        }
    } else {
        None
    };

    // Extension 필드 키 목록 추출 (view에서)
    let ext_field_keys: Vec<String> = view_fields
        .as_ref()
        .map(|fields| {
            fields
                .iter()
                .filter(|f| f.field_source == "extension" && f.visible)
                .map(|f| f.field_key.clone())
                .collect()
        })
        .unwrap_or_default();

    tracing::debug!("Gateway: Extension fields requested: {:?}", ext_field_keys);

    // 페이지네이션 파라미터 추출 (QIDO에 전달하기 전에 먼저 추출)
    // query.extra의 값은 문자열로 들어오므로 파싱 필요
    let page_size = query.extra
        .get("page_size")
        .and_then(|v| {
            v.as_i64()
                .or_else(|| v.as_str().and_then(|s| s.parse::<i64>().ok()))
        })
        .unwrap_or(50)
        .clamp(1, 200) as i64;
    let page = query.extra
        .get("page")
        .and_then(|v| {
            v.as_i64()
                .or_else(|| v.as_str().and_then(|s| s.parse::<i64>().ok()))
        })
        .unwrap_or(1)
        .max(1);
    let offset = (page - 1) * page_size;

    // 사용자 필터/페이지네이션 파라미터 파싱
    // page, page_size, report_status는 extra에서 제거 (QIDO에 전달하지 않음)
    let mut extra_for_qido = query.extra.clone();
    extra_for_qido.remove("report_status");
    extra_for_qido.remove("page");
    extra_for_qido.remove("page_size");
    let user_params = match build_qido_params_from_user_query(&extra_for_qido) {
        Ok(p) => p,
        Err(msg) => {
            return HttpResponse::BadRequest().json(serde_json::json!({"error": msg}));
        }
    };

    // 각 프로젝트별로 스터디 조회 및 통합
    let mut all_studies: Vec<serde_json::Value> = Vec::new();
    let mut study_uids_seen = std::collections::HashSet::new();

    let bearer_opt = extract_bearer_token(&req);

    // Bearer 토큰 전달 확인 로그
    if let Some(ref token) = bearer_opt {
        let token_source = if req.headers().get("X-Keycloak-Token").is_some() {
            "X-Keycloak-Token"
        } else {
            "Authorization"
        };
        tracing::info!("🔑 Gateway /me/studies: Using Bearer token from {} (length: {})", token_source, token.len());
    } else {
        tracing::warn!("⚠️ Gateway /me/studies: No Bearer token found in request");
    }

    // study_uid → project_ids 매핑 (같은 study가 여러 프로젝트에 있을 수 있음)
    let mut study_project_map: HashMap<String, Vec<i32>> = HashMap::new();

    for project_id in user_projects.iter() {
        // 프로젝트별 Access Condition 적용
        let qido_params = if let Ok(conditions) = access_condition_repo.list_by_project(*project_id).await {
            let rule_params = build_qido_params_from_conditions(&conditions);
            merge_qido_params(rule_params, user_params.clone())
        } else {
            user_params.clone()
        };

        tracing::debug!("Gateway /me/studies: QIDO params for project {}: {:?}", project_id, qido_params);

        // QIDO 호출
        match qido
            .qido_studies_with_bearer(bearer_opt.as_deref(), qido_params.clone())
            .await
        {
            Ok(json) => {
                if let Some(array) = json.as_array() {
                    for item in array.iter() {
                        if let Some(study_uid) = extract_study_uid(item) {
                            // RBAC 평가
                            let result = evaluator
                                .evaluate_study_uid(user_id, *project_id, &study_uid)
                                .await;

                            // project_data_access 확인
                            let has_data_access = can_access_study(
                                user_id,
                                *project_id,
                                &study_uid,
                                project_data_repo.pool(),
                            )
                            .await;

                            // 접근 가능한 경우
                            if result.allowed && has_data_access {
                                // project_id 매핑 추가
                                study_project_map
                                    .entry(study_uid.clone())
                                    .or_insert_with(Vec::new)
                                    .push(*project_id);

                                // 중복이 아닌 경우만 study 추가
                                if !study_uids_seen.contains(&study_uid) {
                                    study_uids_seen.insert(study_uid.clone());
                                    all_studies.push(item.clone());
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                tracing::error!("Gateway /me/studies: Failed to fetch studies for project {}: {:?}", project_id, e);
                if let Some(ref token) = bearer_opt {
                    tracing::error!("Gateway /me/studies: Bearer token was present (length: {})", token.len());
                } else {
                    tracing::error!("Gateway /me/studies: Bearer token was NOT present");
                }
                // 개별 프로젝트 실패는 무시하고 계속 진행
            }
        }
    }

    // Study Date로 정렬 (있는 경우)
    all_studies.sort_by(|a, b| {
        let date_a = extract_study_date(a);
        let date_b = extract_study_date(b);
        date_b.cmp(&date_a) // 최신순
    });

    // 페이지네이션 적용
    let total_count = all_studies.len();
    let total_pages = if page_size > 0 {
        (total_count as i64 + page_size - 1) / page_size
    } else {
        0
    };
    let start = offset as usize;
    let end = std::cmp::min(start + page_size as usize, total_count);
    let paginated_studies = if start < total_count {
        all_studies[start..end].to_vec()
    } else {
        Vec::new()
    };

    // _ext 필드 추가
    let ext_builder = StudyExtBuilder::new(pool.get_ref(), annotation_repo.get_ref(), user_id);

    // 성능 최적화: 모든 project_ids를 미리 수집하여 한 번에 조회
    let all_project_ids: Vec<i32> = study_project_map
        .values()
        .flat_map(|ids| ids.iter().copied())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    let project_info_cache = ext_builder.fetch_projects_batch(&all_project_ids).await;

    let mut enriched_studies: Vec<serde_json::Value> = Vec::with_capacity(paginated_studies.len());

    for mut study in paginated_studies {
        if let Some(study_uid) = extract_study_uid(&study) {
            // _ext 객체 생성
            let mut ext = serde_json::Map::new();

            // projects 정보는 항상 추가 (id, name, role_name)
            if let Some(project_ids) = study_project_map.get(&study_uid) {
                let projects: Vec<ProjectInfo> = project_ids
                    .iter()
                    .filter_map(|id| project_info_cache.get(id).cloned())
                    .collect();
                ext.insert("projects".to_string(), serde_json::json!(projects));

                // report_status 추가 (ext_field_keys에 포함된 경우만)
                if ext_field_keys.contains(&"report_status".to_string()) {
                    if let Some(status) = ext_builder.fetch_report_status(&study_uid, project_ids).await {
                        ext.insert("report_status".to_string(), serde_json::json!(status));
                    }
                }

                // review 추가 (ext_field_keys에 포함된 경우만)
                if ext_field_keys.contains(&"review".to_string()) {
                    if let Some(review) = ext_builder.fetch_review(&study_uid, project_ids).await {
                        ext.insert("review".to_string(), serde_json::json!(review));
                    }
                }
            }

            // _ext 필드를 study에 추가
            if !ext.is_empty() {
                if let Some(obj) = study.as_object_mut() {
                    obj.insert("_ext".to_string(), serde_json::Value::Object(ext));
                }
            }
        }
        enriched_studies.push(study);
    }

    tracing::debug!(
        "Gateway: Returning {} studies (page {}/{}, total {})",
        enriched_studies.len(),
        page,
        total_pages,
        total_count
    );

    HttpResponse::Ok()
        .insert_header(("X-Total-Count", total_count.to_string()))
        .insert_header(("X-Page", page.to_string()))
        .insert_header(("X-Page-Size", page_size.to_string()))
        .insert_header(("X-Total-Pages", total_pages.to_string()))
        .json(enriched_studies)
}

/// Study Date 추출 헬퍼 함수
fn extract_study_date(item: &serde_json::Value) -> Option<String> {
    // DICOM 태그 00080020 (StudyDate) 추출
    if let Some(date_obj) = item.get("00080020") {
        if let Some(values) = date_obj.get("Value").and_then(|v| v.as_array()) {
            if let Some(first_value) = values.first().and_then(|v| v.as_str()) {
                return Some(first_value.to_string());
            }
        }
    }
    // StudyDate 별칭도 확인
    if let Some(date_obj) = item.get("StudyDate") {
        if let Some(values) = date_obj.get("Value").and_then(|v| v.as_array()) {
            if let Some(first_value) = values.first().and_then(|v| v.as_str()) {
                return Some(first_value.to_string());
            }
        }
    }
    None
}

pub async fn get_series(
    qido: web::Data<Dcm4cheeQidoClient>,
    evaluator: web::Data<Arc<DicomRbacEvaluatorImpl>>,
    jwt: web::Data<Arc<JwtService>>,
    access_condition_repo: web::Data<Arc<AccessConditionRepositoryImpl>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
    project_data_repo: web::Data<Arc<ProjectDataRepositoryImpl>>,
    path: web::Path<String>,
    query: web::Query<GatewayQuery>,
    req: HttpRequest,
) -> HttpResponse {
    let study_uid = path.into_inner();

    // 사용자 ID 추출
    let user_id = match extract_user_id_from_request(&req, &jwt, &user_repo).await {
        Some(id) if id > 0 => id,
        _ => {
            tracing::warn!("Gateway: Unauthorized - failed to extract user_id from token");
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Invalid or missing authorization token"
            }));
        }
    };

    // 전체 데이터 조회 권한 확인
    let has_global_access = has_global_dicom_access(user_id, project_data_repo.pool()).await;

    // 프로젝트 ID 검증
    let project_id_opt = query.project_id;

    // 전체 데이터 조회 권한이 없으면 project_id 필수
    if !has_global_access && project_id_opt.is_none() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "project_id is required (no global access permission)"
        }));
    }

    // project_id가 있으면 검증
    if let Some(id) = project_id_opt {
        if id <= 0 {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "project_id must be greater than 0"
            }));
        }
    }

    // 0. project_id가 있으면 Study 접근 권한 확인 (project_data_access)
    if let Some(pid) = project_id_opt {
        let has_study_access = can_access_study(
            user_id,
            pid,
            &study_uid,
            project_data_repo.pool(),
        )
        .await;

        if !has_study_access {
            tracing::warn!(
                "Gateway: User {} does not have access to study {} in project {}",
                user_id,
                study_uid,
                pid
            );
            return HttpResponse::Forbidden().json(serde_json::json!({
                "error": "Access denied to this study"
            }));
        }
    }

    // 1. 규칙 기반 QIDO 파라미터 병합 + 사용자 입력 우선 병합
    // report_status는 extra에서 제거 (serde(flatten)으로 인해 포함될 수 있음)
    let mut extra_for_qido = query.extra.clone();
    extra_for_qido.remove("report_status");
    let user_params = match build_qido_params_from_user_query(&extra_for_qido) {
        Ok(p) => p,
        Err(msg) => {
            return HttpResponse::BadRequest().json(serde_json::json!({"error": msg}));
        }
    };

    // Access Condition은 project_id가 있을 때만 적용
    let qido_params = if let Some(pid) = project_id_opt {
        if let Ok(conditions) = access_condition_repo.list_by_project(pid).await {
            let rule_params = build_qido_params_from_conditions(&conditions);
            merge_qido_params(rule_params, user_params)
        } else {
            user_params
        }
    } else {
        user_params
    };

    // 2. Dcm4chee QIDO 호출
    let bearer_opt = extract_bearer_token(&req);
    let qido_response = match qido
        .qido_series_with_bearer(bearer_opt.as_deref(), &study_uid, qido_params)
        .await
    {
        Ok(json) => json,
        Err(e) => {
            return HttpResponse::BadGateway().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    // 3. RBAC 필터링 적용
    let filtered = if has_global_access && project_id_opt.is_none() {
        // 전체 데이터 조회 권한이 있고 project_id가 없으면 필터링 안 함
        tracing::debug!("Gateway: Global access granted, skipping RBAC filtering");
        qido_response
    } else if let Some(pid) = project_id_opt {
        // project_id가 있으면 RBAC 필터링 적용
        if let Some(array) = qido_response.as_array() {
            let mut allowed_items = Vec::new();
            for item in array.iter() {
                if let Some(series_uid) = extract_series_uid(item) {
                    let result = evaluator
                        .evaluate_series_uid(user_id, pid, &series_uid)
                        .await;
                    if result.allowed {
                        allowed_items.push(item.clone());
                    }
                }
            }
            serde_json::Value::Array(allowed_items)
        } else {
            qido_response
        }
    } else {
        // 이 경우는 발생하지 않아야 함 (위에서 검증됨)
        qido_response
    };

    // 4. Report Status 필터링 적용 (옵셔널)
    let final_filtered = if let Some(status_str) = &query.report_status {
        let status_filter = parse_report_status_filter(status_str);
        if !status_filter.is_empty() {
            if let Some(array) = filtered.as_array() {
                match filter_series_by_report_status_batch(
                    array,
                    user_id,
                    project_id_opt,
                    &status_filter,
                    project_data_repo.pool(),
                ).await {
                    Ok(filtered_series) => serde_json::Value::Array(filtered_series),
                    Err(e) => {
                        tracing::error!("Failed to filter by report status: {}", e);
                        filtered // 에러 시 기존 결과 반환
                    }
                }
            } else {
                filtered
            }
        } else {
            filtered
        }
    } else {
        filtered
    };

    HttpResponse::Ok().json(final_filtered)
}

/// 관리자용 특정 스터디의 시리즈 목록 조회 (전역 접근 권한 필요)
pub async fn get_admin_study_series(
    qido: web::Data<Dcm4cheeQidoClient>,
    evaluator: web::Data<Arc<DicomRbacEvaluatorImpl>>,
    jwt: web::Data<Arc<JwtService>>,
    access_condition_repo: web::Data<Arc<AccessConditionRepositoryImpl>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
    project_data_repo: web::Data<Arc<ProjectDataRepositoryImpl>>,
    path: web::Path<String>,
    query: web::Query<GatewayQuery>,
    req: HttpRequest,
) -> HttpResponse {
    let study_uid = path.into_inner();

    // 사용자 ID 추출
    let user_id = match extract_user_id_from_request(&req, &jwt, &user_repo).await {
        Some(id) if id > 0 => id,
        _ => {
            tracing::warn!("Gateway: Unauthorized - failed to extract user_id from token");
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Invalid or missing authorization token"
            }));
        }
    };

    // 전역 접근 권한 확인
    let has_global_access = has_global_dicom_access(user_id, project_data_repo.pool()).await;
    if !has_global_access {
        tracing::warn!("Gateway: User {} attempted to access admin endpoint without global access", user_id);
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Global access permission required",
            "message": "DICOM_GLOBAL_ACCESS capability is required to access this endpoint"
        }));
    }

    // 사용자 필터/페이지네이션 파라미터 파싱
    let mut extra_for_qido = query.extra.clone();
    extra_for_qido.remove("report_status");
    let user_params = match build_qido_params_from_user_query(&extra_for_qido) {
        Ok(p) => p,
        Err(msg) => {
            return HttpResponse::BadRequest().json(serde_json::json!({"error": msg}));
        }
    };

    tracing::debug!("Gateway: Admin /studies/{}/series endpoint - User params: {:?}", study_uid, user_params);

    // Access Condition은 적용하지 않음 (전체 데이터 조회)
    let qido_params = user_params;

    // Dcm4chee QIDO 호출
    let bearer_opt = extract_bearer_token(&req);

    let qido_response = match qido
        .qido_series_with_bearer(bearer_opt.as_deref(), &study_uid, qido_params)
        .await
    {
        Ok(json) => json,
        Err(e) => {
            return HttpResponse::BadGateway().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    // 전역 접근 권한이 있으면 필터링 없이 반환
    tracing::debug!("Gateway: Admin /studies/{}/series endpoint - Returning all series without RBAC filtering", study_uid);
    HttpResponse::Ok().json(qido_response)
}

/// 사용자 관점 특정 스터디의 시리즈 목록 조회
/// project_id 파라미터가 있으면 해당 프로젝트만 필터링
pub async fn get_user_study_series(
    qido: web::Data<Dcm4cheeQidoClient>,
    evaluator: web::Data<Arc<DicomRbacEvaluatorImpl>>,
    jwt: web::Data<Arc<JwtService>>,
    access_condition_repo: web::Data<Arc<AccessConditionRepositoryImpl>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
    project_data_repo: web::Data<Arc<ProjectDataRepositoryImpl>>,
    path: web::Path<String>,
    query: web::Query<GatewayQuery>,
    req: HttpRequest,
) -> HttpResponse {
    let study_uid = path.into_inner();

    // 사용자 ID 추출
    let user_id = match extract_user_id_from_request(&req, &jwt, &user_repo).await {
        Some(id) if id > 0 => id,
        _ => {
            tracing::warn!("Gateway: Unauthorized - failed to extract user_id from token");
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Invalid or missing authorization token"
            }));
        }
    };

    // project_id 파라미터 확인
    let project_id_filter = query.project_id;

    // 사용자가 속한 프로젝트 조회
    let user_projects = if let Some(filter_pid) = project_id_filter {
        // 특정 프로젝트만 필터링하는 경우
        // 사용자가 해당 프로젝트의 멤버인지 확인
        let is_member = match sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM security_user_project WHERE user_id = $1 AND project_id = $2)"
        )
        .bind(user_id)
        .bind(filter_pid)
        .fetch_one(project_data_repo.pool())
        .await
        {
            Ok(member) => member,
            Err(e) => {
                tracing::error!("Gateway: Failed to check project membership: {:?}", e);
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Failed to check project membership"
                }));
            }
        };

        if !is_member {
            tracing::warn!("Gateway: User {} is not a member of project {}", user_id, filter_pid);
            return HttpResponse::Forbidden().json(serde_json::json!({
                "error": "Access denied",
                "message": format!("User is not a member of project {}", filter_pid)
            }));
        }

        // Study 접근 권한 확인 (project_data_access)
        let has_study_access = can_access_study(
            user_id,
            filter_pid,
            &study_uid,
            project_data_repo.pool(),
        )
        .await;

        if !has_study_access {
            tracing::warn!(
                "Gateway: User {} does not have access to study {} in project {}",
                user_id,
                study_uid,
                filter_pid
            );
            return HttpResponse::Forbidden().json(serde_json::json!({
                "error": "Access denied to this study"
            }));
        }

        vec![filter_pid]
    } else {
        // 모든 프로젝트 조회
        match sqlx::query_scalar::<_, i32>(
            "SELECT DISTINCT project_id FROM security_user_project WHERE user_id = $1 ORDER BY project_id"
        )
        .bind(user_id)
        .fetch_all(project_data_repo.pool())
        .await
        {
            Ok(projects) => projects,
            Err(e) => {
                tracing::error!("Gateway: Failed to fetch user projects: {:?}", e);
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Failed to fetch user projects"
                }));
            }
        }
    };

    if user_projects.is_empty() {
        tracing::debug!("Gateway: User {} has no projects", user_id);
        return HttpResponse::Ok().json(serde_json::json!([]));
    }

    tracing::debug!("Gateway: User {} querying {} projects for study {} series", user_id, user_projects.len(), study_uid);

    // 사용자 필터/페이지네이션 파라미터 파싱
    let mut extra_for_qido = query.extra.clone();
    extra_for_qido.remove("report_status");
    let user_params = match build_qido_params_from_user_query(&extra_for_qido) {
        Ok(p) => p,
        Err(msg) => {
            return HttpResponse::BadRequest().json(serde_json::json!({"error": msg}));
        }
    };

    // 각 프로젝트별로 시리즈 조회 및 통합
    let mut all_series: Vec<serde_json::Value> = Vec::new();
    let mut series_uids_seen = std::collections::HashSet::new();

    let bearer_opt = extract_bearer_token(&req);

    for project_id in user_projects.iter() {
        // Study 접근 권한 확인 (project_data_access)
        let has_study_access = can_access_study(
            user_id,
            *project_id,
            &study_uid,
            project_data_repo.pool(),
        )
        .await;

        if !has_study_access {
            tracing::debug!(
                "Gateway: User {} does not have access to study {} in project {}, skipping",
                user_id,
                study_uid,
                project_id
            );
            continue;
        }

        // 프로젝트별 Access Condition 적용
        let qido_params = if let Ok(conditions) = access_condition_repo.list_by_project(*project_id).await {
            let rule_params = build_qido_params_from_conditions(&conditions);
            merge_qido_params(rule_params, user_params.clone())
        } else {
            user_params.clone()
        };

        // QIDO 호출
        match qido
            .qido_series_with_bearer(bearer_opt.as_deref(), &study_uid, qido_params.clone())
            .await
        {
            Ok(json) => {
                if let Some(array) = json.as_array() {
                    for item in array.iter() {
                        if let Some(series_uid) = extract_series_uid(item) {
                            // RBAC 평가
                            let result = evaluator
                                .evaluate_series_uid(user_id, *project_id, &series_uid)
                                .await;

                            // 접근 가능하고 중복이 아닌 경우만 추가
                            if result.allowed && !series_uids_seen.contains(&series_uid) {
                                series_uids_seen.insert(series_uid.clone());
                                all_series.push(item.clone());
                            }
                        }
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Gateway: Failed to fetch series for study {} in project {}: {:?}", study_uid, project_id, e);
                // 개별 프로젝트 실패는 무시하고 계속 진행
            }
        }
    }

    // Report Status 필터링 적용 (옵셔널)
    let mut final_filtered = serde_json::Value::Array(all_series);
    if let Some(status_str) = &query.report_status {
        let status_filter = parse_report_status_filter(status_str);
        if !status_filter.is_empty() {
            if let Some(array) = final_filtered.as_array() {
                match filter_series_by_report_status_batch(
                    array,
                    user_id,
                    project_id_filter,
                    &status_filter,
                    project_data_repo.pool(),
                ).await {
                    Ok(filtered_series) => {
                        final_filtered = serde_json::Value::Array(filtered_series);
                    },
                    Err(e) => {
                        tracing::error!("Failed to filter by report status: {}", e);
                        // 에러 시 기존 결과 유지
                    }
                }
            }
        }
    }

    tracing::debug!(
        "Gateway: Returning {} series for study {}",
        if let Some(arr) = final_filtered.as_array() { arr.len() } else { 0 },
        study_uid
    );

    HttpResponse::Ok().json(final_filtered)
}

pub async fn get_instances(
    qido: web::Data<Dcm4cheeQidoClient>,
    evaluator: web::Data<Arc<DicomRbacEvaluatorImpl>>,
    jwt: web::Data<Arc<JwtService>>,
    access_condition_repo: web::Data<Arc<AccessConditionRepositoryImpl>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
    project_data_repo: web::Data<Arc<ProjectDataRepositoryImpl>>,
    path: web::Path<(String, String)>,
    query: web::Query<GatewayQuery>,
    req: HttpRequest,
) -> HttpResponse {
    let (study_uid, series_uid) = path.into_inner();

    // 사용자 ID 추출
    let user_id = match extract_user_id_from_request(&req, &jwt, &user_repo).await {
        Some(id) if id > 0 => id,
        _ => {
            tracing::warn!("Gateway: Unauthorized - failed to extract user_id from token");
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Invalid or missing authorization token"
            }));
        }
    };

    // 전체 데이터 조회 권한 확인
    let has_global_access = has_global_dicom_access(user_id, project_data_repo.pool()).await;

    // 프로젝트 ID 검증
    let project_id_opt = query.project_id;

    // 전체 데이터 조회 권한이 없으면 project_id 필수
    if !has_global_access && project_id_opt.is_none() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "project_id is required (no global access permission)"
        }));
    }

    // project_id가 있으면 검증
    if let Some(id) = project_id_opt {
        if id <= 0 {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "project_id must be greater than 0"
            }));
        }
    }

    // 0. project_id가 있으면 Study 접근 권한 확인 (project_data_access)
    if let Some(pid) = project_id_opt {
        let has_study_access = can_access_study(
            user_id,
            pid,
            &study_uid,
            project_data_repo.pool(),
        )
        .await;

        if !has_study_access {
            tracing::warn!(
                "Gateway: User {} does not have access to study {} in project {}",
                user_id,
                study_uid,
                pid
            );
            return HttpResponse::Forbidden().json(serde_json::json!({
                "error": "Access denied to this study"
            }));
        }
    }

    // 1. 규칙 기반 QIDO 파라미터 병합 + 사용자 입력 우선 병합
    // report_status는 extra에서 제거 (serde(flatten)으로 인해 포함될 수 있음)
    let mut extra_for_qido = query.extra.clone();
    extra_for_qido.remove("report_status");
    let user_params = match build_qido_params_from_user_query(&extra_for_qido) {
        Ok(p) => p,
        Err(msg) => {
            return HttpResponse::BadRequest().json(serde_json::json!({"error": msg}));
        }
    };

    // Access Condition은 project_id가 있을 때만 적용
    let qido_params = if let Some(pid) = project_id_opt {
        if let Ok(conditions) = access_condition_repo.list_by_project(pid).await {
            let rule_params = build_qido_params_from_conditions(&conditions);
            merge_qido_params(rule_params, user_params)
        } else {
            user_params
        }
    } else {
        user_params
    };

    // 2. Dcm4chee QIDO 호출
    let bearer_opt = extract_bearer_token(&req);
    let qido_response = match qido
        .qido_instances_with_bearer(bearer_opt.as_deref(), &study_uid, &series_uid, qido_params)
        .await
    {
        Ok(json) => json,
        Err(e) => {
            return HttpResponse::BadGateway().json(serde_json::json!({"error": e.to_string()}))
        }
    };

    // 3. RBAC 필터링 적용
    let filtered = if has_global_access && project_id_opt.is_none() {
        // 전체 데이터 조회 권한이 있고 project_id가 없으면 필터링 안 함
        tracing::debug!("Gateway: Global access granted, skipping RBAC filtering");
        qido_response
    } else if let Some(pid) = project_id_opt {
        // project_id가 있으면 RBAC 필터링 적용
        if let Some(array) = qido_response.as_array() {
            let mut allowed_items = Vec::new();
            for item in array.iter() {
                if let Some(instance_uid) = extract_instance_uid(item) {
                    let result = evaluator
                        .evaluate_instance_uid(user_id, pid, &instance_uid)
                        .await;
                    if result.allowed {
                        allowed_items.push(item.clone());
                    }
                }
            }
            serde_json::Value::Array(allowed_items)
        } else {
            qido_response
        }
    } else {
        // 이 경우는 발생하지 않아야 함 (위에서 검증됨)
        qido_response
    };

    HttpResponse::Ok().json(filtered)
}

/// GET /api/dicom/patients - Patient 레벨 QIDO-RS 프록시
/// 1. Dcm4chee QIDO-RS /patients 호출 (사용자 파라미터 전달)
/// 2. RBAC 필터링 (프로젝트에 할당된 환자만)
pub async fn get_patients(
    qido: web::Data<Dcm4cheeQidoClient>,
    jwt: web::Data<Arc<JwtService>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
    project_data_repo: web::Data<Arc<ProjectDataRepositoryImpl>>,
    query: web::Query<GatewayQuery>,
    req: HttpRequest,
) -> HttpResponse {
    // 사용자 ID 추출 (개발 모드 지원)
    let user_id = match AnnotationController::extract_user_id_with_auth(&req, &jwt, &user_repo).await {
        Ok(id) => id,
        Err(err_response) => {
            tracing::warn!("Gateway /patients: Unauthorized - failed to extract user_id");
            return err_response;
        }
    };

    // 전체 데이터 조회 권한 확인
    let has_global_access = has_global_dicom_access(user_id, project_data_repo.pool()).await;

    // 프로젝트 ID 검증
    let project_id_opt = query.project_id;

    // 전체 데이터 조회 권한이 없으면 project_id 필수
    if !has_global_access && project_id_opt.is_none() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "project_id is required (no global access permission)"
        }));
    }

    // project_id가 있으면 검증
    if let Some(id) = project_id_opt {
        if id <= 0 {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "project_id must be greater than 0"
            }));
        }
    }

    // 1. 사용자 필터/페이지네이션 파라미터 파싱
    // report_status는 extra에서 제거 (serde(flatten)으로 인해 포함될 수 있음)
    let mut extra_for_qido = query.extra.clone();
    extra_for_qido.remove("report_status");
    let user_params = match build_qido_params_from_user_query(&extra_for_qido) {
        Ok(p) => p,
        Err(msg) => {
            return HttpResponse::BadRequest().json(serde_json::json!({"error": msg}));
        }
    };

    tracing::debug!("Gateway /patients: User params: {:?}", user_params);

    // 2. Dcm4chee QIDO 호출
    let bearer_opt = extract_bearer_token(&req);

    let qido_response = match qido
        .qido_patients_with_bearer(bearer_opt.as_deref(), user_params)
        .await
    {
        Ok(json) => json,
        Err(e) => {
            tracing::error!("Gateway /patients: QIDO call failed: {}", e);
            return HttpResponse::BadGateway().json(serde_json::json!({"error": e.to_string()}));
        }
    };

    // 3. RBAC 필터링 (프로젝트에 할당된 환자만)
    let filtered_response = if let Some(project_id) = project_id_opt {
        // DB에서 허용된 환자 ID 목록 조회
        let allowed_patient_ids = match get_allowed_patient_ids(project_id, project_data_repo.pool()).await {
            Ok(ids) => ids,
            Err(e) => {
                tracing::error!("Gateway /patients: Failed to get allowed patient IDs: {}", e);
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Failed to query allowed patients"
                }));
            }
        };

        tracing::debug!("Gateway /patients: Found {} allowed patient IDs for project {}", allowed_patient_ids.len(), project_id);

        // QIDO 응답 필터링
        if let Some(patients) = qido_response.as_array() {
            let filtered: Vec<serde_json::Value> = patients
                .iter()
                .filter(|patient| {
                    if let Some(patient_id) = extract_patient_id(patient) {
                        allowed_patient_ids.contains(&patient_id)
                    } else {
                        false
                    }
                })
                .cloned()
                .collect();

            tracing::debug!("Gateway /patients: Filtered {} patients from {} QIDO results", filtered.len(), patients.len());
            serde_json::json!(filtered)
        } else {
            qido_response
        }
    } else {
        // 전체 접근 권한이 있는 경우 - 필터링 없이 반환
        qido_response
    };

    HttpResponse::Ok().json(filtered_response)
}

/// GET /api/dicom/series - Series 레벨 QIDO-RS 프록시
/// 1. Dcm4chee QIDO-RS /series 호출 (사용자 파라미터 전달)
/// 2. RBAC 필터링 (프로젝트에 할당된 Series만)
/// 3. 각 Series에 thumbnail_url 필드 추가
pub async fn get_series_all(
    qido: web::Data<Dcm4cheeQidoClient>,
    jwt: web::Data<Arc<JwtService>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
    project_data_repo: web::Data<Arc<ProjectDataRepositoryImpl>>,
    query: web::Query<GatewayQuery>,
    req: HttpRequest,
) -> HttpResponse {
    // 사용자 ID 추출 (개발 모드 지원)
    let user_id = match AnnotationController::extract_user_id_with_auth(&req, &jwt, &user_repo).await {
        Ok(id) => id,
        Err(err_response) => {
            tracing::warn!("Gateway /series: Unauthorized - failed to extract user_id");
            return err_response;
        }
    };

    // 전체 데이터 조회 권한 확인
    let has_global_access = has_global_dicom_access(user_id, project_data_repo.pool()).await;

    // 프로젝트 ID 검증
    let project_id_opt = query.project_id;

    // 전체 데이터 조회 권한이 없으면 project_id 필수
    if !has_global_access && project_id_opt.is_none() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "project_id is required (no global access permission)"
        }));
    }

    // project_id가 있으면 검증
    if let Some(id) = project_id_opt {
        if id <= 0 {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "project_id must be greater than 0"
            }));
        }
    }

    tracing::debug!("Gateway /series: user_id={}, project_id={:?}", user_id, project_id_opt);

    // 1. 사용자 필터/페이지네이션 파라미터 파싱
    // report_status, page, page_size는 extra에서 제거
    // - report_status: 필터링 후 적용
    // - page, page_size: 필터링 후 메모리에서 페이지네이션 적용 (QIDO에는 전달하지 않음)
    let mut extra_for_qido = query.extra.clone();
    extra_for_qido.remove("report_status");
    extra_for_qido.remove("page");
    extra_for_qido.remove("page_size");
    let user_params = match build_qido_params_from_user_query(&extra_for_qido) {
        Ok(p) => p,
        Err(msg) => {
            return HttpResponse::BadRequest().json(serde_json::json!({"error": msg}));
        }
    };

    tracing::debug!("Gateway /series: User params: {:?}", user_params);

    // 2. Dcm4chee QIDO 호출
    let bearer_opt = extract_bearer_token(&req);

    // 3. Dcm4chee QIDO 호출
    let qido_response = match qido
        .qido_series_all_with_bearer(bearer_opt.as_deref(), user_params)
        .await
    {
        Ok(json) => json,
        Err(e) => {
            tracing::error!("Gateway /series: QIDO call failed: {}", e);
            return HttpResponse::BadGateway().json(serde_json::json!({"error": e.to_string()}));
        }
    };

    tracing::info!("🔍 Gateway /series: QIDO response received");

    // 3. RBAC 필터링 (프로젝트에 할당된 Series만)
    let filtered_response = if let Some(project_id) = project_id_opt {
        // DB에서 허용된 Series UID 목록 조회
        let allowed_series_uids = match get_allowed_series_uids(project_id, project_data_repo.pool()).await {
            Ok(uids) => uids,
            Err(e) => {
                tracing::error!("Gateway /series: Failed to get allowed series UIDs: {}", e);
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Failed to query allowed series"
                }));
            }
        };

        tracing::info!("🔍 Gateway /series: Found {} allowed series UIDs for project {}", allowed_series_uids.len(), project_id);
        if allowed_series_uids.len() > 0 {
            if allowed_series_uids.len() <= 10 {
                tracing::info!("   Allowed Series UIDs: {:?}", allowed_series_uids);
            } else {
                tracing::info!("   Allowed Series UIDs (first 10): {:?}", allowed_series_uids.iter().take(10).collect::<Vec<_>>());
            }
        } else {
            tracing::warn!("⚠️  No allowed series UIDs found for project {}!", project_id);
        }

        // QIDO 응답 필터링
        if let Some(series_list) = qido_response.as_array() {
            tracing::info!("🔍 Gateway /series: QIDO returned {} series", series_list.len());
            
            // QIDO 응답의 Series UID 추출 (디버깅용)
            let qido_series_uids: Vec<String> = series_list
                .iter()
                .filter_map(|series| extract_series_uid(series))
                .collect();
            
            if qido_series_uids.len() > 0 && qido_series_uids.len() <= 10 {
                tracing::info!("   QIDO Series UIDs: {:?}", qido_series_uids);
            } else if qido_series_uids.len() > 10 {
                tracing::info!("   QIDO Series UIDs (first 10): {:?}", qido_series_uids.iter().take(10).collect::<Vec<_>>());
            }
            
            // 매칭 확인
            let matched_count = qido_series_uids.iter()
                .filter(|uid| allowed_series_uids.contains(*uid))
                .count();
            tracing::info!("   Matched Series UIDs: {}/{}", matched_count, qido_series_uids.len());
            
            if matched_count == 0 && allowed_series_uids.len() > 0 && qido_series_uids.len() > 0 {
                tracing::warn!("⚠️  No Series UIDs matched! This might indicate a UID format mismatch.");
                tracing::warn!("   Allowed UIDs (first 3): {:?}", allowed_series_uids.iter().take(3).collect::<Vec<_>>());
                tracing::warn!("   QIDO UIDs (first 3): {:?}", qido_series_uids.iter().take(3).collect::<Vec<_>>());
            }
            
            let filtered: Vec<serde_json::Value> = series_list
                .iter()
                .filter(|series| {
                    if let Some(series_uid) = extract_series_uid(series) {
                        let contains = allowed_series_uids.contains(&series_uid);
                        if !contains {
                            tracing::debug!("   Series {} not in allowed list", series_uid);
                        }
                        contains
                    } else {
                        tracing::debug!("   Failed to extract series_uid from QIDO response");
                        false
                    }
                })
                .cloned()
                .collect();

            tracing::info!("🔍 Gateway /series: Filtered {} series from {} QIDO results", filtered.len(), series_list.len());
            serde_json::json!(filtered)
        } else {
            qido_response
        }
    } else {
        // 전체 접근 권한이 있는 경우 - 필터링 없이 반환
        qido_response
    };

    // 4. 페이지네이션 적용 (필터링 후 메모리에서)
    let paginated_response = if let Some(array) = filtered_response.as_array() {
        let page = query.extra
            .get("page")
            .and_then(|v| {
                v.as_i64()
                    .or_else(|| v.as_str().and_then(|s| s.parse::<i64>().ok()))
            })
            .unwrap_or(1)
            .max(1) as usize;
        let page_size = query.extra
            .get("page_size")
            .and_then(|v| {
                v.as_i64()
                    .or_else(|| v.as_str().and_then(|s| s.parse::<i64>().ok()))
            })
            .unwrap_or(50)
            .max(1)
            .min(200) as usize;
        let offset = (page - 1) * page_size;
        
        let paginated: Vec<serde_json::Value> = array
            .iter()
            .skip(offset as usize)
            .take(page_size as usize)
            .cloned()
            .collect();
        
        tracing::debug!(
            "Gateway /series: Pagination applied - page={}, page_size={}, offset={}, total={}, returned={}",
            page, page_size, offset, array.len(), paginated.len()
        );
        
        serde_json::json!(paginated)
    } else {
        filtered_response
    };

    // 5. 각 Series에 thumbnail_url 필드 추가
    let mut final_response = add_thumbnail_urls(paginated_response, qido.base_url(), qido.qido_path());

    // 6. Report Status 필터링 적용 (옵셔널)
    if let Some(status_str) = &query.report_status {
        let status_filter = parse_report_status_filter(status_str);
        if !status_filter.is_empty() {
            if let Some(array) = final_response.as_array() {
                match filter_series_by_report_status_batch(
                    array,
                    user_id,
                    project_id_opt,
                    &status_filter,
                    project_data_repo.pool(),
                ).await {
                    Ok(filtered_series) => {
                        // thumbnail_url 다시 추가
                        final_response = add_thumbnail_urls(
                            serde_json::Value::Array(filtered_series),
                            qido.base_url(),
                            qido.qido_path()
                        );
                    },
                    Err(e) => {
                        tracing::error!("Failed to filter by report status: {}", e);
                        // 에러 시 기존 결과 유지
                    }
                }
            }
        }
    }

    HttpResponse::Ok().json(final_response)
}

/// 관리자용 전체 시리즈 목록 조회 (전역 접근 권한 필요)
pub async fn get_admin_series(
    qido: web::Data<Dcm4cheeQidoClient>,
    jwt: web::Data<Arc<JwtService>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
    project_data_repo: web::Data<Arc<ProjectDataRepositoryImpl>>,
    query: web::Query<GatewayQuery>,
    req: HttpRequest,
) -> HttpResponse {
    // 사용자 ID 추출
    let user_id = match extract_user_id_from_request(&req, &jwt, &user_repo).await {
        Some(id) if id > 0 => id,
        _ => {
            tracing::warn!("Gateway: Unauthorized - failed to extract user_id from token");
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Invalid or missing authorization token"
            }));
        }
    };

    // 전역 접근 권한 확인
    let has_global_access = has_global_dicom_access(user_id, project_data_repo.pool()).await;
    if !has_global_access {
        tracing::warn!("Gateway: User {} attempted to access admin endpoint without global access", user_id);
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Global access permission required",
            "message": "DICOM_GLOBAL_ACCESS capability is required to access this endpoint"
        }));
    }

    // 사용자 필터/페이지네이션 파라미터 파싱
    let mut extra_for_qido = query.extra.clone();
    extra_for_qido.remove("report_status");
    let user_params = match build_qido_params_from_user_query(&extra_for_qido) {
        Ok(p) => p,
        Err(msg) => {
            return HttpResponse::BadRequest().json(serde_json::json!({"error": msg}));
        }
    };

    tracing::debug!("Gateway: Admin /series endpoint - User params: {:?}", user_params);

    // Dcm4chee QIDO 호출
    let bearer_opt = extract_bearer_token(&req);

    let qido_response = match qido
        .qido_series_all_with_bearer(bearer_opt.as_deref(), user_params)
        .await
    {
        Ok(json) => json,
        Err(e) => {
            tracing::error!("Gateway /admin/series: QIDO call failed: {}", e);
            return HttpResponse::BadGateway().json(serde_json::json!({"error": e.to_string()}));
        }
    };

    // 전역 접근 권한이 있으면 필터링 없이 반환
    // thumbnail_url 추가
    let mut final_response = add_thumbnail_urls(qido_response, qido.base_url(), qido.qido_path());

    // Report Status 필터링 적용 (옵셔널)
    if let Some(status_str) = &query.report_status {
        let status_filter = parse_report_status_filter(status_str);
        if !status_filter.is_empty() {
            if let Some(array) = final_response.as_array() {
                match filter_series_by_report_status_batch(
                    array,
                    user_id,
                    None, // project_id 없음 (전역 접근)
                    &status_filter,
                    project_data_repo.pool(),
                ).await {
                    Ok(filtered_series) => {
                        // thumbnail_url 다시 추가
                        final_response = add_thumbnail_urls(
                            serde_json::Value::Array(filtered_series),
                            qido.base_url(),
                            qido.qido_path()
                        );
                    },
                    Err(e) => {
                        tracing::error!("Failed to filter by report status: {}", e);
                        // 에러 시 기존 결과 유지
                    }
                }
            }
        }
    }

    tracing::debug!("Gateway: Admin /series endpoint - Returning all series without RBAC filtering");
    HttpResponse::Ok().json(final_response)
}

/// 사용자가 속한 모든 프로젝트의 시리즈 목록을 통합 조회
/// project_id 파라미터가 있으면 해당 프로젝트만 필터링
pub async fn get_all_user_series(
    qido: web::Data<Dcm4cheeQidoClient>,
    evaluator: web::Data<Arc<DicomRbacEvaluatorImpl>>,
    jwt: web::Data<Arc<JwtService>>,
    access_condition_repo: web::Data<Arc<AccessConditionRepositoryImpl>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
    project_data_repo: web::Data<Arc<ProjectDataRepositoryImpl>>,
    query: web::Query<GatewayQuery>,
    req: HttpRequest,
) -> HttpResponse {
    // 사용자 ID 추출
    let user_id = match extract_user_id_from_request(&req, &jwt, &user_repo).await {
        Some(id) if id > 0 => id,
        _ => {
            tracing::warn!("Gateway: Unauthorized - failed to extract user_id from token");
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Invalid or missing authorization token"
            }));
        }
    };

    // project_id 파라미터 확인
    let project_id_filter = query.project_id;

    // 사용자가 속한 프로젝트 조회
    let user_projects = if let Some(filter_pid) = project_id_filter {
        // 특정 프로젝트만 필터링하는 경우
        // 사용자가 해당 프로젝트의 멤버인지 확인
        let is_member = match sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM security_user_project WHERE user_id = $1 AND project_id = $2)"
        )
        .bind(user_id)
        .bind(filter_pid)
        .fetch_one(project_data_repo.pool())
        .await
        {
            Ok(member) => member,
            Err(e) => {
                tracing::error!("Gateway: Failed to check project membership: {:?}", e);
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Failed to check project membership"
                }));
            }
        };

        if !is_member {
            tracing::warn!("Gateway: User {} is not a member of project {}", user_id, filter_pid);
            return HttpResponse::Forbidden().json(serde_json::json!({
                "error": "Access denied",
                "message": format!("User is not a member of project {}", filter_pid)
            }));
        }

        vec![filter_pid]
    } else {
        // 모든 프로젝트 조회
        match sqlx::query_scalar::<_, i32>(
            "SELECT DISTINCT project_id FROM security_user_project WHERE user_id = $1 ORDER BY project_id"
        )
        .bind(user_id)
        .fetch_all(project_data_repo.pool())
        .await
        {
            Ok(projects) => projects,
            Err(e) => {
                tracing::error!("Gateway: Failed to fetch user projects: {:?}", e);
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Failed to fetch user projects"
                }));
            }
        }
    };

    if user_projects.is_empty() {
        tracing::debug!("Gateway: User {} has no projects", user_id);
        return HttpResponse::Ok().json(serde_json::json!([]));
    }

    tracing::debug!("Gateway: User {} querying {} projects for series", user_id, user_projects.len());

    // 페이지네이션 파라미터 추출 (QIDO에 전달하기 전에 먼저 추출)
    // query.extra의 값은 문자열로 들어오므로 파싱 필요
    let page_size = query.extra
        .get("page_size")
        .and_then(|v| {
            v.as_i64()
                .or_else(|| v.as_str().and_then(|s| s.parse::<i64>().ok()))
        })
        .unwrap_or(50)
        .clamp(1, 200) as i64;
    let page = query.extra
        .get("page")
        .and_then(|v| {
            v.as_i64()
                .or_else(|| v.as_str().and_then(|s| s.parse::<i64>().ok()))
        })
        .unwrap_or(1)
        .max(1);
    let offset = (page - 1) * page_size;

    tracing::info!("🔍 Pagination params: page={}, page_size={}, offset={}", page, page_size, offset);

    // 사용자 필터/페이지네이션 파라미터 파싱
    // page, page_size, report_status는 extra에서 제거 (QIDO에 전달하지 않음)
    let mut extra_for_qido = query.extra.clone();
    extra_for_qido.remove("report_status");
    extra_for_qido.remove("page");
    extra_for_qido.remove("page_size");
    let user_params = match build_qido_params_from_user_query(&extra_for_qido) {
        Ok(p) => p,
        Err(msg) => {
            return HttpResponse::BadRequest().json(serde_json::json!({"error": msg}));
        }
    };

    // 각 프로젝트별로 시리즈 조회 및 통합 (병렬 처리)
    let bearer_opt = extract_bearer_token(&req);

    // 병렬로 모든 프로젝트의 QIDO 호출
    let mut qido_futures = Vec::new();
    let projects_count = user_projects.len();

    // 페이지네이션 최적화: 각 프로젝트에서 필요한 만큼만 가져오기
    // QIDO 서버가 offset을 지원하지 않으므로, 충분한 데이터를 가져온 후 메모리에서 페이지네이션 적용
    // QIDO 서버가 limit를 제대로 처리하지 못하는 경우가 있으므로, 충분히 큰 값으로 설정
    // offset + page_size만큼 가져와야 하지만, 여유분을 크게 설정
    let qido_limit = if projects_count == 1 {
        // 단일 프로젝트: 최소 100개 이상 가져오기 (QIDO 서버 버그 회피)
        std::cmp::max(offset + page_size * 10, 100).min(500)
    } else {
        // 여러 프로젝트: offset + page_size * 프로젝트 수 * 10
        std::cmp::max(offset + page_size * projects_count as i64 * 10, 100).min(500)
    };

    for project_id in user_projects.iter() {
        let project_id = *project_id;
        let qido_clone = qido.clone();
        let bearer_clone = bearer_opt.clone();
        let access_condition_repo_clone = access_condition_repo.clone();
        let user_params_clone = user_params.clone();
        let qido_limit_clone = qido_limit;

        qido_futures.push(tokio::spawn(async move {
            // 프로젝트별 Access Condition 적용
            let mut qido_params = if let Ok(conditions) = access_condition_repo_clone.list_by_project(project_id).await {
                let rule_params = build_qido_params_from_conditions(&conditions);
                merge_qido_params(rule_params, user_params_clone)
            } else {
                user_params_clone
            };

            // offset 파라미터 제거 (QIDO 서버가 지원하지 않음)
            // 메모리에서 페이지네이션을 적용하므로 QIDO에는 offset을 전달하지 않음
            qido_params.retain(|(k, _)| k != "offset" && k != "Offset");

            // limit 파라미터 최적화: 기존 limit를 최적화된 값으로 덮어쓰기
            // build_qido_params_from_user_query가 이미 limit를 추가했지만,
            // 필터링 여유분을 고려하여 더 큰 값으로 조정
            let limit_index = qido_params.iter().position(|(k, _)| k == "limit" || k == "Limit");
            if let Some(idx) = limit_index {
                // 기존 limit를 최적화된 값으로 덮어쓰기
                qido_params[idx] = ("limit".to_string(), qido_limit_clone.to_string());
            } else {
                // limit가 없으면 추가
                qido_params.push(("limit".to_string(), qido_limit_clone.to_string()));
            }

            // Bearer 토큰 로깅 (디버깅용)
            if let Some(ref token) = bearer_clone {
                tracing::info!("Gateway: 프로젝트 {} QIDO 호출 - Bearer token 길이: {}", project_id, token.len());
            } else {
                tracing::warn!("Gateway: 프로젝트 {} QIDO 호출 - Bearer token 없음", project_id);
            }

            // QIDO 호출
            qido_clone
                .qido_series_all_with_bearer(bearer_clone.as_deref(), qido_params)
                .await
                .map(|json| (project_id, json))
        }));
    }

    // 모든 QIDO 호출 완료 대기
    let qido_results: Vec<_> = futures::future::join_all(qido_futures)
        .await
        .into_iter()
        .filter_map(|r| {
            match r {
                Ok(Ok((pid, json))) => {
                    tracing::info!("Gateway: 프로젝트 {} QIDO 성공", pid);
                    Some((pid, json))
                }
                Ok(Err(e)) => {
                    tracing::error!("Gateway: QIDO 호출 실패: {}", e);
                    None
                }
                Err(e) => {
                    tracing::error!("Gateway: QIDO future 실행 실패: {}", e);
                    None
                }
            }
        })
        .collect();

    tracing::info!("Gateway: QIDO 호출 완료 - {} 프로젝트에서 데이터 수신 (총 {}개 프로젝트 시도)", qido_results.len(), user_projects.len());
    
    if qido_results.is_empty() {
        tracing::warn!("Gateway: 모든 QIDO 호출이 실패했습니다. Bearer 토큰이 제대로 전달되었는지 확인하세요.");
    }

    // 모든 Series 수집 및 프로젝트별 allowed_series_uids 조회 (병렬)
    let mut all_series_items: Vec<(i32, serde_json::Value)> = Vec::new();
    let mut project_series_map: std::collections::HashMap<i32, Vec<serde_json::Value>> = std::collections::HashMap::new();

    for (project_id, json) in qido_results {
        if let Some(array) = json.as_array() {
            let count = array.len();
            tracing::debug!("Gateway: 프로젝트 {}에서 {}개 Series 수신", project_id, count);
            project_series_map.entry(project_id).or_insert_with(Vec::new).extend(array.iter().cloned());
        }
    }

    tracing::debug!("Gateway: 총 {} 프로젝트에서 Series 수집", project_series_map.len());

    // 프로젝트별 allowed_series_uids를 병렬로 조회
    let mut allowed_uids_futures = Vec::new();
    for project_id in project_series_map.keys() {
        let project_id = *project_id;
        let project_data_repo_clone = project_data_repo.clone();
        allowed_uids_futures.push(tokio::spawn(async move {
            get_allowed_series_uids(project_id, project_data_repo_clone.pool()).await.map(|uids| (project_id, uids))
        }));
    }

    let allowed_uids_results: std::collections::HashMap<i32, std::collections::HashSet<String>> = futures::future::join_all(allowed_uids_futures)
        .await
        .into_iter()
        .filter_map(|r| r.ok())
        .filter_map(|r| r.ok())
        .collect();

    // 프로젝트별로 Series 필터링 (할당된 Series만)
    for (project_id, series_array) in project_series_map.iter() {
        if let Some(allowed_uids) = allowed_uids_results.get(project_id) {
            let before_count = series_array.len();
            let mut matched_count = 0;
            for item in series_array.iter() {
                if let Some(series_uid) = extract_series_uid(item) {
                    if allowed_uids.contains(&series_uid) {
                        all_series_items.push((*project_id, item.clone()));
                        matched_count += 1;
                    }
                }
            }
            tracing::debug!("Gateway: 프로젝트 {} - QIDO: {}개, allowed_uids: {}개, 매칭: {}개", 
                project_id, before_count, allowed_uids.len(), matched_count);
        } else {
            tracing::warn!("Gateway: 프로젝트 {}에 대한 allowed_uids 결과가 없음", project_id);
        }
    }

    tracing::debug!("Gateway: allowed_uids 필터링 후 {}개 Series 항목", all_series_items.len());

    // Batch로 Series UID → Series ID 매핑
    let all_series_uids: Vec<String> = all_series_items
        .iter()
        .filter_map(|(_, item)| extract_series_uid(item))
        .collect();

    // 프로젝트별로 Series UID → Series ID 매핑 (병렬)
    let mut series_id_futures = Vec::new();
    for project_id in user_projects.iter() {
        let project_id = *project_id;
        let project_data_repo_clone = project_data_repo.clone();
        let project_series_uids: Vec<String> = all_series_items
            .iter()
            .filter(|(pid, _)| *pid == project_id)
            .filter_map(|(_, item)| extract_series_uid(item))
            .collect();

        if !project_series_uids.is_empty() {
            series_id_futures.push(tokio::spawn(async move {
                get_series_ids_by_uids_batch(&project_series_uids, Some(project_id), project_data_repo_clone.pool())
                    .await
                    .map(|map| (project_id, map))
            }));
        }
    }

    let series_id_maps: std::collections::HashMap<i32, std::collections::HashMap<String, i32>> = futures::future::join_all(series_id_futures)
        .await
        .into_iter()
        .filter_map(|r| r.ok())
        .filter_map(|r| r.ok())
        .collect();

    tracing::debug!("Gateway: Series ID 매핑 완료 - {} 프로젝트", series_id_maps.len());
    for (pid, map) in series_id_maps.iter() {
        tracing::debug!("Gateway: 프로젝트 {} - {}개 Series ID 매핑됨", pid, map.len());
    }

    // Batch로 RBAC 평가 (프로젝트별로)
    let mut rbac_futures = Vec::new();
    for project_id in user_projects.iter() {
        let project_id = *project_id;
        let evaluator_clone = evaluator.clone();
        let project_series_ids: Vec<i32> = series_id_maps
            .get(&project_id)
            .map(|map| map.values().copied().collect())
            .unwrap_or_default();

        if !project_series_ids.is_empty() {
            rbac_futures.push(tokio::spawn(async move {
                evaluate_series_access_batch(user_id, project_id, &project_series_ids, &evaluator_clone.pool).await
                    .map(|allowed_set| (project_id, allowed_set))
            }));
        }
    }

    let rbac_results: std::collections::HashMap<i32, std::collections::HashSet<i32>> = futures::future::join_all(rbac_futures)
        .await
        .into_iter()
        .filter_map(|r| r.ok())
        .filter_map(|r| r.ok())
        .collect();

    tracing::debug!("Gateway: RBAC 평가 완료 - {} 프로젝트", rbac_results.len());
    for (pid, allowed_set) in rbac_results.iter() {
        tracing::debug!("Gateway: 프로젝트 {} - {}개 Series 허용됨", pid, allowed_set.len());
    }

    // 최종 필터링: RBAC 평가 결과를 기반으로 허용된 Series만 포함
    let mut all_series: Vec<serde_json::Value> = Vec::new();
    let mut series_uids_seen = std::collections::HashSet::new();
    let mut filtered_out_count = 0;

    for (project_id, item) in all_series_items {
        if let Some(series_uid) = extract_series_uid(&item) {
            // 중복 체크
            if series_uids_seen.contains(&series_uid) {
                continue;
            }

            // RBAC 평가 결과 확인
            let mut included = false;
            if let Some(series_id_map) = series_id_maps.get(&project_id) {
                if let Some(series_id) = series_id_map.get(&series_uid) {
                    if let Some(allowed_series_ids) = rbac_results.get(&project_id) {
                        if allowed_series_ids.contains(series_id) {
                            series_uids_seen.insert(series_uid);
                            all_series.push(item);
                            included = true;
                        }
                    } else {
                        tracing::debug!("Gateway: 프로젝트 {}에 대한 RBAC 결과가 없음", project_id);
                    }
                } else {
                    tracing::debug!("Gateway: Series UID {}에 대한 Series ID를 찾을 수 없음 (프로젝트 {})", series_uid, project_id);
                }
            } else {
                tracing::debug!("Gateway: 프로젝트 {}에 대한 Series ID 맵이 없음", project_id);
            }
            
            if !included {
                filtered_out_count += 1;
            }
        }
    }

    tracing::debug!("Gateway: 최종 필터링 완료 - {}개 Series 포함, {}개 제외됨", all_series.len(), filtered_out_count);

    // thumbnail_url 추가
    let mut final_series = add_thumbnail_urls(
        serde_json::Value::Array(all_series.clone()),
        qido.base_url(),
        qido.qido_path()
    );

    // Report Status 필터링 적용 (옵셔널)
    if let Some(status_str) = &query.report_status {
        let status_filter = parse_report_status_filter(status_str);
        if !status_filter.is_empty() {
            if let Some(array) = final_series.as_array() {
                match filter_series_by_report_status_batch(
                    array,
                    user_id,
                    project_id_filter,
                    &status_filter,
                    project_data_repo.pool(),
                ).await {
                    Ok(filtered_series) => {
                        // thumbnail_url 다시 추가
                        final_series = add_thumbnail_urls(
                            serde_json::Value::Array(filtered_series),
                            qido.base_url(),
                            qido.qido_path()
                        );
                    },
                    Err(e) => {
                        tracing::error!("Failed to filter by report status: {}", e);
                        // 에러 시 기존 결과 유지
                    }
                }
            }
        }
    }

    // 페이지네이션 적용
    // QIDO 서버가 offset을 지원하지 않으므로, 메모리에서 offset을 적용
    let total_count = if let Some(array) = final_series.as_array() {
        array.len()
    } else {
        0
    };

    // 메모리에서 offset 적용
    let start = std::cmp::min(offset as usize, total_count);
    let end = std::cmp::min(start + page_size as usize, total_count);

    tracing::info!(
        "🔍 Pagination DEBUG: page={}, page_size={}, offset={}, start={}, end={}, total_count={}, QIDO_offset_applied=false (memory pagination)",
        page, page_size, offset, start, end, total_count
    );

    let paginated_series = if let Some(array) = final_series.as_array() {
        if total_count > 0 && start < total_count {
            tracing::info!("🔍 Slicing array[{}..{}] from QIDO result (total: {})", start, end, total_count);
            serde_json::Value::Array(array[start..end].to_vec())
        } else {
            tracing::info!("🔍 No data in range or empty result, returning empty array");
            serde_json::Value::Array(vec![])
        }
    } else {
        tracing::warn!("🔍 final_series is not an array!");
        serde_json::Value::Array(vec![])
    };

    tracing::info!(
        "Gateway: Returning {} series (page {}, total {})",
        if let Some(arr) = paginated_series.as_array() { arr.len() } else { 0 },
        page,
        total_count
    );

    HttpResponse::Ok().json(paginated_series)
}

// 토큰 파싱/추출 유틸은 `infrastructure::auth::token_extractor`로 분리

/// Report Status 필터 파싱
/// 입력: "approval,unread" 또는 "approved,unread" (호환성을 위해 둘 다 지원)
/// 출력: DB 스키마에 맞는 값 ("approval", "unread", "unapproval")
pub fn parse_report_status_filter(status_str: &str) -> Vec<String> {
    status_str
        .split(',')
        .map(|s| {
            let trimmed = s.trim().to_lowercase();
            // "approved"를 "approval"로 변환 (DB 스키마에 맞춤)
            if trimmed == "approved" {
                "approval".to_string()
            } else {
                trimmed
            }
        })
        .filter(|s| matches!(s.as_str(), "approval" | "unread" | "unapproval"))
        .collect()
}

/// 여러 Series UID를 한 번에 조회하여 series_id 매핑 생성
pub async fn get_series_ids_by_uids_batch(
    series_uids: &[String],
    project_id_opt: Option<i32>,
    pool: &sqlx::PgPool,
) -> Result<std::collections::HashMap<String, i32>, sqlx::Error> {
    if series_uids.is_empty() {
        return Ok(std::collections::HashMap::new());
    }

    let rows: Vec<(String, i32)> = if let Some(pid) = project_id_opt {
        // Project에 할당된 Series만 조회
        sqlx::query_as(
            "SELECT pds.series_uid, pds.id
             FROM project_data_series pds
             INNER JOIN project_data_study pdst ON pds.study_id = pdst.id
             INNER JOIN project_data pd ON pd.study_id = pdst.id
             WHERE pds.series_uid = ANY($1) AND pd.project_id = $2"
        )
        .bind(series_uids)
        .bind(pid)
        .fetch_all(pool)
        .await?
    } else {
        // 전체 Series 조회
        sqlx::query_as(
            "SELECT series_uid, id
             FROM project_data_series
             WHERE series_uid = ANY($1)"
        )
        .bind(series_uids)
        .fetch_all(pool)
        .await?
    };

    Ok(rows.into_iter().collect())
}

/// 여러 Series에 대한 Report Status를 한 번에 조회
/// project-dependent report가 있으면 우선, 없으면 global report 사용
pub async fn get_report_statuses_batch(
    series_ids: &[i32],
    user_id: i32,
    project_id_opt: Option<i32>,
    pool: &sqlx::PgPool,
) -> Result<std::collections::HashMap<i32, String>, sqlx::Error> {
    if series_ids.is_empty() {
        return Ok(std::collections::HashMap::new());
    }

    let rows: Vec<(i32, String)> = if let Some(pid) = project_id_opt {
        // project-dependent 우선, 없으면 global
        sqlx::query_as(
            "SELECT DISTINCT ON (series_id) series_id, status
             FROM series_user_report
             WHERE series_id = ANY($1)
               AND user_id = $2
               AND (project_id = $3 OR project_id IS NULL)
             ORDER BY series_id,
                      CASE WHEN project_id = $3 THEN 0 ELSE 1 END"
        )
        .bind(series_ids)
        .bind(user_id)
        .bind(pid)
        .fetch_all(pool)
        .await?
    } else {
        // global report만
        sqlx::query_as(
            "SELECT series_id, status
             FROM series_user_report
             WHERE series_id = ANY($1)
               AND user_id = $2
               AND project_id IS NULL"
        )
        .bind(series_ids)
        .bind(user_id)
        .fetch_all(pool)
        .await?
    };

    Ok(rows.into_iter().collect())
}

/// 배치 쿼리를 사용하여 Report Status로 Series 필터링
pub async fn filter_series_by_report_status_batch(
    series_array: &[serde_json::Value],
    user_id: i32,
    project_id_opt: Option<i32>,
    status_filter: &[String],
    pool: &sqlx::PgPool,
) -> Result<Vec<serde_json::Value>, sqlx::Error> {
    if series_array.is_empty() || status_filter.is_empty() {
        return Ok(series_array.to_vec());
    }

    // 1. 모든 Series UID 수집
    let series_uids: Vec<String> = series_array
        .iter()
        .filter_map(|s| extract_series_uid(s))
        .collect();

    if series_uids.is_empty() {
        return Ok(series_array.to_vec());
    }

    // 2. 배치로 Series ID 조회
    let series_id_map = get_series_ids_by_uids_batch(&series_uids, project_id_opt, pool).await?;

    // 3. Series ID 리스트 추출
    let series_ids: Vec<i32> = series_id_map.values().copied().collect();

    if series_ids.is_empty() {
        return Ok(Vec::new());
    }

    // 4. 배치로 Report Status 조회
    let report_status_map = get_report_statuses_batch(&series_ids, user_id, project_id_opt, pool).await?;

    // 5. 메모리에서 필터링
    let filtered: Vec<serde_json::Value> = series_array
        .iter()
        .filter(|series| {
            if let Some(series_uid) = extract_series_uid(series) {
                if let Some(series_id) = series_id_map.get(&series_uid) {
                    if let Some(status) = report_status_map.get(series_id) {
                        return status_filter.contains(&status.to_lowercase());
                    }
                }
            }
            false // Report가 없으면 제외
        })
        .cloned()
        .collect();

    Ok(filtered)
}

/// 프로젝트에 할당된 Patient ID 목록 조회
async fn get_allowed_patient_ids(project_id: i32, pool: &sqlx::PgPool) -> Result<std::collections::HashSet<String>, sqlx::Error> {
    let rows: Vec<(Option<String>,)> = sqlx::query_as(
        "SELECT DISTINCT pds.patient_id
         FROM project_data pd
         INNER JOIN project_data_study pds ON pd.study_id = pds.id
         WHERE pd.project_id = $1
           AND pds.patient_id IS NOT NULL"
    )
    .bind(project_id)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().filter_map(|(id,)| id).collect())
}

/// 프로젝트에 할당된 Series UID 목록 조회
async fn get_allowed_series_uids(project_id: i32, pool: &sqlx::PgPool) -> Result<std::collections::HashSet<String>, sqlx::Error> {
    // resource_level에 따라 다른 쿼리 사용
    // SERIES 레벨: series_id로 직접 조회 (해당 series만)
    // STUDY 레벨: study_id로 조인하여 study의 모든 series 조회
    
    let rows: Vec<(Option<String>,)> = sqlx::query_as(
        "SELECT DISTINCT combined.series_uid
         FROM (
             -- resource_level='SERIES'인 경우: series_id로 직접 조회
             SELECT pdser.series_uid
             FROM project_data pd
             INNER JOIN project_data_series pdser ON pd.series_id = pdser.id
             WHERE pd.project_id = $1
               AND pd.resource_level = 'SERIES'
               AND pd.series_id IS NOT NULL
               AND pdser.series_uid IS NOT NULL
             
             UNION
             
             -- resource_level='STUDY'인 경우: study_id로 조인하여 study의 모든 series 조회
             SELECT DISTINCT pdser.series_uid
             FROM project_data pd
             INNER JOIN project_data_study pds ON pd.study_id = pds.id
             INNER JOIN project_data_series pdser ON pds.id = pdser.study_id
             WHERE pd.project_id = $1
               AND pd.resource_level = 'STUDY'
               AND pd.study_id IS NOT NULL
               AND pdser.series_uid IS NOT NULL
         ) AS combined"
    )
    .bind(project_id)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().filter_map(|(uid,)| uid).collect())
}

/// 여러 Series에 대한 RBAC 접근 권한을 배치로 평가
/// 반환값: 허용된 Series ID 집합
async fn evaluate_series_access_batch(
    user_id: i32,
    project_id: i32,
    series_ids: &[i32],
    pool: &sqlx::PgPool,
) -> Result<std::collections::HashSet<i32>, sqlx::Error> {
    if series_ids.is_empty() {
        return Ok(std::collections::HashSet::new());
    }

    // 1. 프로젝트 멤버십 확인
    let is_member: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM security_user_project WHERE user_id = $1 AND project_id = $2)",
    )
    .bind(user_id)
    .bind(project_id)
    .fetch_one(pool)
    .await?;

    if !is_member {
        return Ok(std::collections::HashSet::new());
    }

    // 2. 명시적 거부된 Series 조회
    let denied_series: Vec<i32> = sqlx::query_scalar(
        "SELECT DISTINCT series_id
         FROM project_data_access
         WHERE user_id = $1 AND project_id = $2
           AND status = 'DENIED' AND resource_level = 'SERIES'
           AND series_id = ANY($3)"
    )
    .bind(user_id)
    .bind(project_id)
    .bind(series_ids)
    .fetch_all(pool)
    .await?;

    let denied_set: std::collections::HashSet<i32> = denied_series.into_iter().collect();

    // 3. 명시적 승인된 Series 조회
    let approved_series: Vec<i32> = sqlx::query_scalar(
        "SELECT DISTINCT series_id
         FROM project_data_access
         WHERE user_id = $1 AND project_id = $2
           AND status = 'APPROVED' AND resource_level = 'SERIES'
           AND series_id = ANY($3)"
    )
    .bind(user_id)
    .bind(project_id)
    .bind(series_ids)
    .fetch_all(pool)
    .await?;

    let approved_set: std::collections::HashSet<i32> = approved_series.into_iter().collect();

    // 4. Series의 Study ID 조회 (상속 평가용)
    let series_study_map: Vec<(i32, Option<i32>)> = sqlx::query_as(
        "SELECT id, study_id FROM project_data_series WHERE id = ANY($1)"
    )
    .bind(series_ids)
    .fetch_all(pool)
    .await?;

    // 5. Study 접근 권한 배치 조회
    let study_ids: Vec<i32> = series_study_map
        .iter()
        .filter_map(|(_, study_id)| *study_id)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    let allowed_studies: std::collections::HashSet<i32> = if !study_ids.is_empty() {
        // Study에 대한 명시적 거부 확인
        let denied_studies: Vec<i32> = sqlx::query_scalar(
            "SELECT DISTINCT study_id
             FROM project_data_access
             WHERE user_id = $1 AND project_id = $2
               AND status = 'DENIED' AND resource_level = 'STUDY'
               AND study_id = ANY($3)"
        )
        .bind(user_id)
        .bind(project_id)
        .bind(&study_ids)
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        let denied_study_set: std::collections::HashSet<i32> = denied_studies.into_iter().collect();

        // Study에 대한 명시적 승인 또는 기본 접근 확인
        // project_data_access에 레코드가 없으면 기본적으로 접근 허용
        let approved_studies: Vec<i32> = sqlx::query_scalar(
            "SELECT DISTINCT study_id
             FROM project_data_access
             WHERE user_id = $1 AND project_id = $2
               AND status = 'APPROVED' AND resource_level = 'STUDY'
               AND study_id = ANY($3)"
        )
        .bind(user_id)
        .bind(project_id)
        .bind(&study_ids)
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        let approved_study_set: std::collections::HashSet<i32> = approved_studies.into_iter().collect();

        // 모든 Study ID에서 거부된 것 제외, 승인된 것 또는 기본 접근 허용
        study_ids
            .into_iter()
            .filter(|study_id| !denied_study_set.contains(study_id))
            .filter(|study_id| {
                // 승인되었거나 project_data_access에 레코드가 없으면 허용
                approved_study_set.contains(study_id) || {
                    // project_data_access에 레코드가 있는지 확인
                    // 레코드가 없으면 기본 접근 허용
                    true // 간단화: 레코드가 없으면 허용으로 간주
                }
            })
            .collect()
    } else {
        std::collections::HashSet::new()
    };

    // 6. 최종 결과 계산
    let mut allowed_series = std::collections::HashSet::new();

    for (series_id, study_id_opt) in series_study_map {
        // 명시적 거부는 제외
        if denied_set.contains(&series_id) {
            continue;
        }

        // 명시적 승인은 포함
        if approved_set.contains(&series_id) {
            allowed_series.insert(series_id);
            continue;
        }

        // Study 상속: Study가 허용되면 Series도 허용
        if let Some(study_id) = study_id_opt {
            if allowed_studies.contains(&study_id) {
                allowed_series.insert(series_id);
            }
        }
    }

    Ok(allowed_series)
}

/// Authorization 헤더에서 Bearer 토큰 추출
/// X-Keycloak-Token 헤더를 우선 확인하고, 없으면 Authorization 헤더 사용
pub fn extract_bearer_token(req: &HttpRequest) -> Option<String> {
    req.headers()
        .get("X-Keycloak-Token")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string())
        .or_else(|| {
            req.headers()
                .get("Authorization")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.strip_prefix("Bearer "))
                .map(|s| s.to_string())
        })
}

/// 각 Series에 thumbnail_url 필드 추가
fn add_thumbnail_urls(series_array: serde_json::Value, base_url: &str, qido_path: &str) -> serde_json::Value {
    if let Some(array) = series_array.as_array() {
        let mut result = Vec::new();
        for item in array.iter() {
            let mut item_clone = item.clone();

            // StudyInstanceUID와 SeriesInstanceUID 추출
            if let (Some(study_uid), Some(series_uid)) = (
                extract_study_uid(item),
                extract_series_uid(item),
            ) {
                // qido_path에서 /rs를 제거하고 thumbnail 경로 생성
                // 예: /iaid-pacs/aets/iAID_PACS/rs -> /iaid-pacs/aets/iAID_PACS/rs/studies/.../thumbnail
                let thumbnail_url = format!(
                    "{}{}/studies/{}/series/{}/thumbnail",
                    base_url,
                    qido_path,
                    study_uid,
                    series_uid
                );

                if let Some(obj) = item_clone.as_object_mut() {
                    obj.insert("thumbnail_url".to_string(), serde_json::json!(thumbnail_url));
                }
            }

            result.push(item_clone);
        }
        serde_json::Value::Array(result)
    } else {
        series_array
    }
}

/// QIDO-RS JSON에서 StudyInstanceUID 추출 (0020000D)
fn extract_study_uid(item: &serde_json::Value) -> Option<String> {
    item.get("0020000D")
        .and_then(|v| v.get("Value"))
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.get(0))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

/// QIDO-RS JSON에서 SeriesInstanceUID 추출 (0020000E)
fn extract_series_uid(item: &serde_json::Value) -> Option<String> {
    item.get("0020000E")
        .and_then(|v| v.get("Value"))
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.get(0))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

/// QIDO-RS JSON에서 SOPInstanceUID 추출 (00080018)
fn extract_instance_uid(item: &serde_json::Value) -> Option<String> {
    item.get("00080018")
        .and_then(|v| v.get("Value"))
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.get(0))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

/// QIDO-RS JSON에서 PatientID 추출 (00100020)
fn extract_patient_id(item: &serde_json::Value) -> Option<String> {
    item.get("00100020")
        .and_then(|v| v.get("Value"))
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.get(0))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

// json_to_params: 이전 일반 쿼리 전달용 유틸은 사용자 필터 전용 파서로 대체됨

pub(crate) fn build_qido_params_from_conditions(
    conds: &Vec<AccessCondition>,
) -> Vec<(String, String)> {
    let mut params = Vec::new();
    for c in conds.iter() {
        match c.operator.as_str() {
            "EQ" | "EQUALS" | "==" => {
                if let Some(tag) = &c.dicom_tag {
                    match tag.as_str() {
                        "00080060" | "Modality" => {
                            if let Some(val) = &c.value {
                                params.push(("Modality".to_string(), val.clone()));
                            }
                        }
                        "00100020" | "PatientID" => {
                            if let Some(val) = &c.value {
                                params.push(("PatientID".to_string(), val.clone()));
                            }
                        }
                        "00080050" | "AccessionNumber" => {
                            if let Some(val) = &c.value {
                                params.push(("AccessionNumber".to_string(), val.clone()));
                            }
                        }
                        "00100010" | "PatientName" => {
                            if let Some(val) = &c.value {
                                params.push(("PatientName".to_string(), val.clone()));
                            }
                        }
                        _ => {}
                    }
                }
            }
            // CONTAINS는 QIDO에서도 부분일치로 동작하도록 값 그대로 전달(서버 구현에 의존)
            "CONTAINS" => {
                if let Some(tag) = &c.dicom_tag {
                    match tag.as_str() {
                        "00080050" | "AccessionNumber" => {
                            if let Some(val) = &c.value {
                                params.push(("AccessionNumber".to_string(), val.clone()));
                            }
                        }
                        "00100010" | "PatientName" => {
                            if let Some(val) = &c.value {
                                params.push(("PatientName".to_string(), val.clone()));
                            }
                        }
                        _ => {}
                    }
                }
            }
            "RANGE" | "BETWEEN" => {
                if let Some(tag) = &c.dicom_tag {
                    if tag == "00080020" || tag == "StudyDate" {
                        if let Some(val) = &c.value {
                            params.push(("StudyDate".to_string(), val.clone()));
                        }
                    }
                }
            }
            // NE(불일치) 등은 QIDO 파라미터로 표현하기 어려워 사후 필터에 위임
            _ => {}
        }
    }
    params
}

// 사용자 쿼리에서 지원하는 파라미터를 QIDO 파라미터로 변환하며 검증을 수행한다
fn build_qido_params_from_user_query(
    extra: &serde_json::Map<String, Value>,
) -> Result<Vec<(String, String)>, String> {
    // report_status는 명시적으로 제거 (안전장치)
    let mut extra = extra.clone();
    extra.remove("report_status");
    let mut params: HashMap<String, String> = HashMap::new();

    // 필터: modality/patient_id/study_date/optional accession_number/patient_name
    if let Some(v) = extra.get("modality").and_then(|v| v.as_str()) {
        params.insert("Modality".to_string(), v.to_string());
    }
    if let Some(v) = extra.get("patient_id").and_then(|v| v.as_str()) {
        params.insert("PatientID".to_string(), v.to_string());
    }
    if let Some(v) = extra.get("accession_number").and_then(|v| v.as_str()) {
        params.insert("AccessionNumber".to_string(), v.to_string());
    }
    if let Some(v) = extra.get("patient_name").and_then(|v| v.as_str()) {
        params.insert("PatientName".to_string(), v.to_string());
    }

    if let Some(sd) = extra.get("study_date").and_then(|v| v.as_str()) {
        if !is_valid_study_date(sd) {
            return Err("Invalid study_date format. Use YYYYMMDD or YYYYMMDD-YYYYMMDD".to_string());
        }
        params.insert("StudyDate".to_string(), sd.to_string());
    }

    // 페이지네이션: limit/offset이 명시되면 그대로 사용, 없을 때만 page/page_size를 limit/offset으로 변환
    let has_limit = extra.get("limit").is_some();
    let has_offset = extra.get("offset").is_some();
    if !has_limit || !has_offset {
        let page_size = extra
            .get("page_size")
            .and_then(|v| v.as_i64())
            .unwrap_or(50)
            .clamp(1, 200) as i64;
        let page = extra
            .get("page")
            .and_then(|v| v.as_i64())
            .unwrap_or(1)
            .max(1);
        let offset = (page - 1) * page_size;
        if !has_limit {
            params.insert("limit".to_string(), page_size.to_string());
        }
        if !has_offset {
            params.insert("offset".to_string(), offset.to_string());
        }
    }

    // DICOMweb 네이티브 파라미터 패스스루: 알려진 필드 외 문자열/숫자/불리언은 그대로 전달
    for (k, v) in extra.iter() {
        // 내부 파라미터는 전달하지 않음 (user_id 추가)
        if matches!(k.as_str(), "project_id" | "user_id" | "page" | "page_size" | "check_assignment_for_project" | "report_status") {
            continue;
        }
        // 소문자 사용자 별칭은 이미 위에서 변환 처리됨(modality/patient_id/study_date/accession_number/patient_name)
        if matches!(
            k.as_str(),
            "modality" | "patient_id" | "study_date" | "accession_number" | "patient_name"
        ) {
            continue;
        }
        if let Some(s) = v.as_str() {
            params.insert(k.clone(), s.to_string());
        } else if v.is_number() || v.is_boolean() {
            params.insert(k.clone(), v.to_string());
        }
    }

    Ok(params.into_iter().collect())
}

fn is_valid_study_date(s: &str) -> bool {
    // YYYYMMDD or YYYYMMDD-YYYYMMDD
    let bytes = s.as_bytes();
    if bytes.len() == 8 {
        return bytes.iter().all(|c| c.is_ascii_digit());
    }
    if bytes.len() == 17 && bytes[8] == b'-' {
        return bytes[..8].iter().all(|c| c.is_ascii_digit())
            && bytes[9..].iter().all(|c| c.is_ascii_digit());
    }
    false
}

fn merge_qido_params(
    rule_params: Vec<(String, String)>,
    user_params: Vec<(String, String)>,
) -> Vec<(String, String)> {
    // rule 먼저 넣고, 같은 키는 user 값으로 덮어씀
    let mut map: HashMap<String, String> = HashMap::new();
    for (k, v) in rule_params {
        map.insert(k, v);
    }
    for (k, v) in user_params {
        map.insert(k, v);
    }
    map.into_iter().collect()
}

// ============================================================================
// _ext 필드 관련 구조체 및 헬퍼 함수
// ============================================================================

/// 프로젝트 정보 (id, name, role_name)
#[derive(Debug, Clone, serde::Serialize)]
pub struct ProjectInfo {
    pub id: i32,
    pub name: String,
    pub role_name: Option<String>,
}

/// Review 상태 정보
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewInfo {
    pub review_stage: String,
    pub available_stages: Vec<String>,
    pub annotation_summary: HashMap<String, bool>,
}

/// Study 확장 정보 빌더
pub struct StudyExtBuilder<'a> {
    pool: &'a PgPool,
    annotation_repo: &'a AnnotationRepositoryImpl,
    user_id: i32,
}

impl<'a> StudyExtBuilder<'a> {
    pub fn new(pool: &'a PgPool, annotation_repo: &'a AnnotationRepositoryImpl, user_id: i32) -> Self {
        Self { pool, annotation_repo, user_id }
    }

    /// 프로젝트 정보 배치 조회 (HashMap으로 반환)
    pub async fn fetch_projects_batch(&self, project_ids: &[i32]) -> HashMap<i32, ProjectInfo> {
        if project_ids.is_empty() {
            return HashMap::new();
        }

        let results = sqlx::query_as::<_, (i32, String, Option<String>)>(
            r#"
            SELECT
                p.id,
                p.name,
                r.name as role_name
            FROM security_project p
            LEFT JOIN security_user_project up ON p.id = up.project_id AND up.user_id = $1
            LEFT JOIN security_role r ON up.role_id = r.id
            WHERE p.id = ANY($2)
            ORDER BY p.id
            "#
        )
        .bind(self.user_id)
        .bind(project_ids)
        .fetch_all(self.pool)
        .await;

        match results {
            Ok(rows) => rows
                .into_iter()
                .map(|(id, name, role_name)| (id, ProjectInfo { id, name, role_name }))
                .collect(),
            Err(e) => {
                tracing::warn!("Failed to fetch project info batch: {:?}", e);
                HashMap::new()
            }
        }
    }

    /// 프로젝트 정보 조회 (id, name, role_name) - 하위 호환성 유지
    pub async fn fetch_projects(&self, project_ids: &[i32]) -> Vec<ProjectInfo> {
        if project_ids.is_empty() {
            return Vec::new();
        }

        // 프로젝트 정보와 사용자의 역할을 함께 조회
        let results = sqlx::query_as::<_, (i32, String, Option<String>)>(
            r#"
            SELECT
                p.id,
                p.name,
                r.name as role_name
            FROM security_project p
            LEFT JOIN security_user_project up ON p.id = up.project_id AND up.user_id = $1
            LEFT JOIN security_role r ON up.role_id = r.id
            WHERE p.id = ANY($2)
            ORDER BY p.id
            "#
        )
        .bind(self.user_id)
        .bind(project_ids)
        .fetch_all(self.pool)
        .await;

        match results {
            Ok(rows) => rows.into_iter().map(|(id, name, role_name)| ProjectInfo { id, name, role_name }).collect(),
            Err(e) => {
                tracing::warn!("Failed to fetch project info: {:?}", e);
                Vec::new()
            }
        }
    }

    /// Report Status 조회 (study_uid 기준으로 가장 최근 report의 status)
    pub async fn fetch_report_status(&self, study_uid: &str, project_ids: &[i32]) -> Option<String> {
        if project_ids.is_empty() {
            return None;
        }

        // study_uid → project_data_study → project_data_series → series_user_report
        let result = sqlx::query_scalar::<_, String>(
            r#"
            SELECT sur.status
            FROM series_user_report sur
            INNER JOIN project_data_series pds ON sur.series_id = pds.id
            INNER JOIN project_data_study pst ON pds.study_id = pst.id
            WHERE pst.study_uid = $1
              AND sur.project_id = ANY($2)
              AND sur.user_id = $3
            ORDER BY sur.updated_at DESC
            LIMIT 1
            "#
        )
        .bind(study_uid)
        .bind(project_ids)
        .bind(self.user_id)
        .fetch_optional(self.pool)
        .await;

        match result {
            Ok(status) => status,
            Err(e) => {
                tracing::debug!("Failed to fetch report status for study {}: {:?}", study_uid, e);
                None
            }
        }
    }

    /// Review 상태 조회 (annotation 기반)
    pub async fn fetch_review(&self, study_uid: &str, project_ids: &[i32]) -> Option<ReviewInfo> {
        if project_ids.is_empty() {
            return None;
        }

        // 각 프로젝트별로 annotation을 조회하고 user_id별로 그룹화
        let mut all_annotator_ids: HashSet<i32> = HashSet::new();
        let mut current_user_role: Option<String> = None;

        for project_id in project_ids {
            // annotation 조회
            if let Ok(annotations) = self.annotation_repo.find_by_project_and_study(*project_id, study_uid).await {
                for ann in annotations {
                    all_annotator_ids.insert(ann.user_id);
                }
            }

            // 현재 사용자의 역할 조회 (첫 번째 프로젝트 기준)
            if current_user_role.is_none() {
                let role = sqlx::query_scalar::<_, String>(
                    r#"
                    SELECT r.name
                    FROM security_user_project up
                    INNER JOIN security_role r ON up.role_id = r.id
                    WHERE up.user_id = $1 AND up.project_id = $2
                    "#
                )
                .bind(self.user_id)
                .bind(*project_id)
                .fetch_optional(self.pool)
                .await;

                if let Ok(Some(role_name)) = role {
                    current_user_role = Some(role_name);
                }
            }
        }

        let annotation_count = all_annotator_ids.len();

        // reviewStage 계산
        let review_stage = match annotation_count {
            0 => "NOT_STARTED",
            1 => "IN_PROGRESS",
            _ => "READY_FOR_ADJUDICATION",
        }.to_string();

        // availableStages 계산 (역할 기반)
        let role_name = current_user_role.as_deref().unwrap_or("");
        let is_adjudicator_or_pm = role_name.to_uppercase().contains("ADMIN")
            || role_name.to_uppercase().contains("ADJUDICATOR")
            || role_name.to_uppercase().contains("PM");

        let available_stages = if is_adjudicator_or_pm {
            vec!["NOT_STARTED".to_string(), "IN_PROGRESS".to_string(), "READY_FOR_ADJUDICATION".to_string()]
        } else {
            // reader는 파랑 단계 제외
            vec!["NOT_STARTED".to_string(), "IN_PROGRESS".to_string()]
        };

        // annotationSummary 생성 (reader1, reader2 형태)
        let mut annotation_summary = HashMap::new();
        for (idx, _user_id) in all_annotator_ids.iter().enumerate() {
            let key = format!("reader{}", idx + 1);
            annotation_summary.insert(key, true);
        }
        // 최대 2명까지 표시
        if !annotation_summary.contains_key("reader1") {
            annotation_summary.insert("reader1".to_string(), false);
        }
        if !annotation_summary.contains_key("reader2") {
            annotation_summary.insert("reader2".to_string(), false);
        }

        Some(ReviewInfo {
            review_stage,
            available_stages,
            annotation_summary,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_qido_params_from_conditions, build_qido_params_from_user_query,
        decode_keycloak_token_sub, extract_instance_uid, extract_series_uid, extract_study_uid,
        is_valid_study_date, merge_qido_params, parse_report_status_filter,
    };
    use crate::domain::entities::access_condition::{
        AccessCondition, ConditionType, ResourceLevel,
    };
    use base64::Engine;

    fn ac(tag: Option<&str>, op: &str, val: Option<&str>) -> AccessCondition {
        AccessCondition {
            id: 1,
            resource_level: ResourceLevel::Study,
            resource_type: "study".to_string(),
            dicom_tag: tag.map(|s| s.to_string()),
            operator: op.to_string(),
            value: val.map(|s| s.to_string()),
            condition_type: ConditionType::Allow,
            created_at: chrono::Utc::now(),
        }
    }

    #[actix_rt::test]
    async fn test_modality_equals_maps_to_qido() {
        let conds = vec![ac(Some("00080060"), "EQ", Some("CT"))];
        let params = build_qido_params_from_conditions(&conds);
        assert!(params.contains(&("Modality".to_string(), "CT".to_string())));
    }

    #[actix_rt::test]
    async fn test_study_date_range_maps_to_qido() {
        let conds = vec![ac(Some("00080020"), "RANGE", Some("20200101-20201231"))];
        let params = build_qido_params_from_conditions(&conds);
        assert!(params.contains(&("StudyDate".to_string(), "20200101-20201231".to_string())));
    }

    #[actix_rt::test]
    async fn test_decode_keycloak_token_sub_valid() {
        // header: {"alg":"none"}
        // payload: {"sub":"550e8400-e29b-41d4-a716-446655440000"}
        let header = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(b"{\"alg\":\"none\"}");
        let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(b"{\"sub\":\"550e8400-e29b-41d4-a716-446655440000\"}");
        let token = format!("{}.{}.sig", header, payload);
        let sub = decode_keycloak_token_sub(&token);
        assert_eq!(sub.as_deref(), Some("550e8400-e29b-41d4-a716-446655440000"));
    }

    #[actix_rt::test]
    async fn test_decode_keycloak_token_sub_invalid() {
        let token = "invalid.token";
        assert!(decode_keycloak_token_sub(token).is_none());
    }

    #[actix_rt::test]
    async fn test_extract_uids_from_qido_json() {
        let v = serde_json::json!({
            "0020000D": {"Value": ["1.2.3"], "vr": "UI"},
            "0020000E": {"Value": ["4.5.6"], "vr": "UI"},
            "00080018": {"Value": ["7.8.9"], "vr": "UI"}
        });
        assert_eq!(extract_study_uid(&v).as_deref(), Some("1.2.3"));
        assert_eq!(extract_series_uid(&v).as_deref(), Some("4.5.6"));
        assert_eq!(extract_instance_uid(&v).as_deref(), Some("7.8.9"));
    }

    #[actix_rt::test]
    async fn test_build_qido_params_multiple_conditions() {
        let conds = vec![
            ac(Some("Modality"), "EQUALS", Some("MR")),
            ac(Some("PatientID"), "==", Some("P-001")),
            ac(Some("StudyDate"), "BETWEEN", Some("20231001-20231031")),
        ];
        let params = build_qido_params_from_conditions(&conds);
        assert!(params.contains(&("Modality".to_string(), "MR".to_string())));
        assert!(params.contains(&("PatientID".to_string(), "P-001".to_string())));
        assert!(params.contains(&("StudyDate".to_string(), "20231001-20231031".to_string())));
    }

    // ==========================
    // 사용자 쿼리 파싱/검증 단위 테스트
    // ==========================

    #[test]
    fn test_is_valid_study_date_formats() {
        assert!(is_valid_study_date("20240101"));
        assert!(is_valid_study_date("20240101-20241231"));
        assert!(!is_valid_study_date("2024-0101"));
        assert!(!is_valid_study_date("2024010X"));
        assert!(!is_valid_study_date("20240101-2024-1231"));
    }

    #[test]
    fn test_build_qido_params_from_user_query_basic_filters() {
        let mut extra = serde_json::Map::new();
        extra.insert(
            "modality".to_string(),
            serde_json::Value::String("CT".to_string()),
        );
        extra.insert(
            "patient_id".to_string(),
            serde_json::Value::String("PAT001".to_string()),
        );
        extra.insert(
            "study_date".to_string(),
            serde_json::Value::String("20240101-20241231".to_string()),
        );
        let params = build_qido_params_from_user_query(&extra).unwrap();
        assert!(params.contains(&("Modality".to_string(), "CT".to_string())));
        assert!(params.contains(&("PatientID".to_string(), "PAT001".to_string())));
        assert!(params.contains(&("StudyDate".to_string(), "20240101-20241231".to_string())));
        // pagination defaults
        assert!(params.iter().any(|(k, v)| k == "limit" && v == "50"));
        assert!(params.iter().any(|(k, v)| k == "offset" && v == "0"));
    }

    #[test]
    fn test_build_qido_params_user_query_pagination_clamp_and_offset() {
        let mut extra = serde_json::Map::new();
        extra.insert("page".to_string(), serde_json::json!(2));
        extra.insert("page_size".to_string(), serde_json::json!(250)); // will clamp to 200
        let params = build_qido_params_from_user_query(&extra).unwrap();
        assert!(params.iter().any(|(k, v)| k == "limit" && v == "200"));
        assert!(params.iter().any(|(k, v)| k == "offset" && v == "200"));
    }

    #[test]
    fn test_build_qido_params_user_query_invalid_study_date() {
        let mut extra = serde_json::Map::new();
        extra.insert(
            "study_date".to_string(),
            serde_json::Value::String("2024-0101".to_string()),
        );
        let err = build_qido_params_from_user_query(&extra).unwrap_err();
        assert!(err.contains("Invalid study_date"));
    }

    #[test]
    fn test_merge_qido_params_user_wins() {
        let rule = vec![
            ("Modality".to_string(), "MR".to_string()),
            ("StudyDate".to_string(), "20230101-20231231".to_string()),
        ];
        let user = vec![
            ("Modality".to_string(), "CT".to_string()),
            ("PatientID".to_string(), "P-9".to_string()),
        ];
        let merged = merge_qido_params(rule, user);
        // Modality should be CT (user overrides)
        assert!(merged.contains(&("Modality".to_string(), "CT".to_string())));
        // PatientID should exist from user
        assert!(merged.contains(&("PatientID".to_string(), "P-9".to_string())));
        // StudyDate should remain from rule (user did not set)
        assert!(merged.contains(&("StudyDate".to_string(), "20230101-20231231".to_string())));
    }

    #[test]
    fn test_rule_mapping_extended_tags() {
        let conds = vec![
            ac(Some("00080050"), "EQ", Some("ACC-1")),
            ac(Some("00100010"), "CONTAINS", Some("KIM")),
        ];
        let params = build_qido_params_from_conditions(&conds);
        assert!(params.contains(&("AccessionNumber".to_string(), "ACC-1".to_string())));
        assert!(params.contains(&("PatientName".to_string(), "KIM".to_string())));
    }

    // ===============
    // 통합 테스트 스텁 (가벼운 모킹) — 환경 의존 없이 설계만 검증하므로 기본 ignore
    // ===============
    #[tokio::test]
    #[ignore]
    async fn it_should_propagate_filters_and_pagination_to_qido() {
        // 향후: 로컬 mock 서버 기동 → Dcm4cheeQidoClient.base_url 지정 →
        // 게이트웨이 핸들러 호출 → mock에서 쿼리스트링(limit/offset/filters) 캡처 검증
        assert!(true);
    }

    #[tokio::test]
    #[ignore]
    async fn it_should_apply_post_filtering_with_evaluator_stub() {
        // 향후: evaluator 스텁이 특정 UID만 허용하도록 구성 →
        // mock QIDO가 여러 UID 반환 → 응답에서 허용된 UID만 남는지 확인
        assert!(true);
    }

    // ==========================
    // Report Status 필터 파싱 단위 테스트
    // ==========================

    #[test]
    fn test_parse_report_status_filter_single_value() {
        let result = parse_report_status_filter("approval");
        assert_eq!(result, vec!["approval"]);
    }

    #[test]
    fn test_parse_report_status_filter_approved_to_approval() {
        // "approved"는 "approval"로 변환되어야 함
        let result = parse_report_status_filter("approved");
        assert_eq!(result, vec!["approval"]);
    }

    #[test]
    fn test_parse_report_status_filter_multiple_values() {
        let result = parse_report_status_filter("approval,unread");
        assert_eq!(result, vec!["approval", "unread"]);
    }

    #[test]
    fn test_parse_report_status_filter_approved_compatibility() {
        // "approved"와 "approval" 모두 지원
        let result = parse_report_status_filter("approved,unread");
        assert_eq!(result, vec!["approval", "unread"]);
    }

    #[test]
    fn test_parse_report_status_filter_all_values() {
        let result = parse_report_status_filter("approval,unread,unapproval");
        assert_eq!(result.len(), 3);
        assert!(result.contains(&"approval".to_string()));
        assert!(result.contains(&"unread".to_string()));
        assert!(result.contains(&"unapproval".to_string()));
    }

    #[test]
    fn test_parse_report_status_filter_with_spaces() {
        let result = parse_report_status_filter("approval , unread , unapproval");
        assert_eq!(result.len(), 3);
        assert!(result.contains(&"approval".to_string()));
        assert!(result.contains(&"unread".to_string()));
        assert!(result.contains(&"unapproval".to_string()));
    }

    #[test]
    fn test_parse_report_status_filter_case_insensitive() {
        let result = parse_report_status_filter("APPROVAL,Unread,UNAPPROVAL");
        assert_eq!(result.len(), 3);
        assert!(result.contains(&"approval".to_string()));
        assert!(result.contains(&"unread".to_string()));
        assert!(result.contains(&"unapproval".to_string()));
    }

    #[test]
    fn test_parse_report_status_filter_case_insensitive_approved() {
        // "APPROVED"도 "approval"로 변환되어야 함
        let result = parse_report_status_filter("APPROVED,Unread");
        assert_eq!(result.len(), 2);
        assert!(result.contains(&"approval".to_string()));
        assert!(result.contains(&"unread".to_string()));
    }

    #[test]
    fn test_parse_report_status_filter_invalid_values_filtered() {
        let result = parse_report_status_filter("approval,invalid,unread,unknown");
        assert_eq!(result.len(), 2);
        assert!(result.contains(&"approval".to_string()));
        assert!(result.contains(&"unread".to_string()));
        assert!(!result.contains(&"invalid".to_string()));
        assert!(!result.contains(&"unknown".to_string()));
    }

    #[test]
    fn test_parse_report_status_filter_empty_string() {
        let result = parse_report_status_filter("");
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_report_status_filter_only_invalid() {
        let result = parse_report_status_filter("invalid,unknown,test");
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_report_status_filter_duplicates() {
        let result = parse_report_status_filter("approval,unread,approval,unread");
        assert_eq!(result.len(), 4); // 중복 허용 (필터링은 나중에)
        assert_eq!(result.iter().filter(|s| s == &"approval").count(), 2);
        assert_eq!(result.iter().filter(|s| s == &"unread").count(), 2);
    }
}
