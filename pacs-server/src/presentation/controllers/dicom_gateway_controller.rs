use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;
use base64::Engine;

use crate::domain::services::DicomRbacEvaluator;
use crate::infrastructure::external::Dcm4cheeQidoClient;
use crate::infrastructure::auth::{Claims, JwtService};
use crate::infrastructure::repositories::{AccessConditionRepositoryImpl, UserRepositoryImpl};
use crate::domain::repositories::{AccessConditionRepository, UserRepository};
use crate::domain::entities::access_condition::AccessCondition;
use crate::infrastructure::services::DicomRbacEvaluatorImpl;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct GatewayQuery {
    #[serde(default)]
    pub project_id: Option<i32>,
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
) {
    cfg.service(
        web::scope("/dicom")
            .app_data(web::Data::new(qido_client))
            .app_data(web::Data::new(evaluator))
            .app_data(web::Data::new(jwt_service))
            .app_data(web::Data::new(access_condition_repo))
            .app_data(web::Data::new(user_repo))
            .route("/ping", web::get().to(|| async { HttpResponse::Ok().finish() }))
            .route("/studies_raw", web::get().to(get_studies_raw))
            .route("/deps", web::get().to(debug_deps))
            .route("/studies", web::get().to(get_studies))
            .route(
                "/studies/{study_uid}/series",
                web::get().to(get_series),
            )
            .route(
                "/studies/{study_uid}/series/{series_uid}/instances",
                web::get().to(get_instances),
            )
    );
}
pub async fn get_studies_raw(
    qido: web::Data<Dcm4cheeQidoClient>, 
    req: HttpRequest,
) -> HttpResponse {
    let bearer_opt = req.headers().get("Authorization").and_then(|h| h.to_str().ok()).and_then(|s| s.strip_prefix("Bearer ")).map(|s| s.to_string());
    match qido.qido_studies_with_bearer(bearer_opt.as_deref(), vec![("limit".to_string(), "1".to_string())]).await {
        Ok(json) => HttpResponse::Ok().json(json),
        Err(e) => HttpResponse::BadGateway().json(serde_json::json!({"error": e.to_string()})),
    }
}

async fn debug_deps(req: HttpRequest) -> HttpResponse {
    let has_qido = req.app_data::<web::Data<Dcm4cheeQidoClient>>().is_some();
    let has_eval = req.app_data::<web::Data<Arc<DicomRbacEvaluatorImpl>>>().is_some();
    let has_eval_plain = req.app_data::<web::Data<DicomRbacEvaluatorImpl>>().is_some();
    let has_jwt = req.app_data::<web::Data<Arc<JwtService>>>().is_some();
    let has_jwt_plain = req.app_data::<web::Data<JwtService>>().is_some();
    let has_ac = req.app_data::<web::Data<Arc<AccessConditionRepositoryImpl>>>().is_some();
    let has_ac_plain = req.app_data::<web::Data<AccessConditionRepositoryImpl>>().is_some();
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

pub async fn get_studies(
    qido: web::Data<Dcm4cheeQidoClient>,
    evaluator: web::Data<Arc<DicomRbacEvaluatorImpl>>,
    jwt: web::Data<Arc<JwtService>>,
    access_condition_repo: web::Data<Arc<AccessConditionRepositoryImpl>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
    query: web::Query<GatewayQuery>,
    req: HttpRequest,
) -> HttpResponse {
    // 프로젝트 ID 검증
    let project_id = match query.project_id {
        Some(id) if id > 0 => id,
        _ => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "project_id is required and must be greater than 0"
            }));
        }
    };
    
    // 사용자 ID 추출
    let user_id = match extract_user_id_from_token(&req, &jwt, &user_repo).await {
        Some(id) if id > 0 => id,
        _ => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Invalid or missing authorization token"
            }));
        }
    };
    
    // 1. 규칙 기반 QIDO 파라미터 병합
    let mut qido_params = json_to_params(query.extra.clone().into());
    if let Ok(conditions) = access_condition_repo.list_by_project(project_id).await {
        let rule_params = build_qido_params_from_conditions(&conditions);
        qido_params.extend(rule_params);
    }
    
    // 2. Dcm4chee QIDO 호출
    let bearer_opt = req.headers().get("Authorization").and_then(|h| h.to_str().ok()).and_then(|s| s.strip_prefix("Bearer ")).map(|s| s.to_string());
    
    // 디버깅: Authorization 헤더 확인
    if let Some(token) = &bearer_opt {
        tracing::debug!("Gateway: Extracted Bearer token (length: {})", token.len());
        tracing::debug!("Gateway: Token preview: {}...", &token[..std::cmp::min(50, token.len())]);
    } else {
        tracing::warn!("Gateway: No Bearer token found in Authorization header");
        if let Some(auth_header) = req.headers().get("Authorization").and_then(|h| h.to_str().ok()) {
            tracing::debug!("Gateway: Authorization header value: {}...", &auth_header[..std::cmp::min(100, auth_header.len())]);
        }
    }
    
    let qido_response = match qido.qido_studies_with_bearer(bearer_opt.as_deref(), qido_params).await {
        Ok(json) => json,
        Err(e) => return HttpResponse::BadGateway().json(serde_json::json!({"error": e.to_string()})),
    };
    
    // 3. RBAC 필터링 적용
    let filtered = if let Some(array) = qido_response.as_array() {
        let mut allowed_items = Vec::new();
        for item in array.iter() {
            if let Some(study_uid) = extract_study_uid(item) {
                let result = evaluator.evaluate_study_uid(user_id, project_id, &study_uid).await;
                if result.allowed {
                    allowed_items.push(item.clone());
                }
            }
        }
        serde_json::Value::Array(allowed_items)
    } else {
        qido_response
    };
    
    HttpResponse::Ok().json(filtered)
}

pub async fn get_series(
    qido: web::Data<Dcm4cheeQidoClient>,
    evaluator: web::Data<Arc<DicomRbacEvaluatorImpl>>,
    jwt: web::Data<Arc<JwtService>>,
    access_condition_repo: web::Data<Arc<AccessConditionRepositoryImpl>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
    path: web::Path<String>,
    query: web::Query<GatewayQuery>,
    req: HttpRequest,
) -> HttpResponse {
    let study_uid = path.into_inner();
    let project_id = match query.project_id {
        Some(id) if id > 0 => id,
        _ => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "project_id is required and must be greater than 0"
            }));
        }
    };
    let user_id = match extract_user_id_from_token(&req, &jwt, &user_repo).await {
        Some(id) if id > 0 => id,
        _ => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Invalid or missing authorization token"
            }));
        }
    };
    
    // 1. 규칙 기반 QIDO 파라미터 병합
    let mut qido_params = json_to_params(query.extra.clone().into());
    if let Ok(conditions) = access_condition_repo.list_by_project(project_id).await {
        let rule_params = build_qido_params_from_conditions(&conditions);
        qido_params.extend(rule_params);
    }
    
    // 2. Dcm4chee QIDO 호출
    let bearer_opt = req.headers().get("Authorization").and_then(|h| h.to_str().ok()).and_then(|s| s.strip_prefix("Bearer ")).map(|s| s.to_string());
    let qido_response = match qido.qido_series_with_bearer(bearer_opt.as_deref(), &study_uid, qido_params).await {
        Ok(json) => json,
        Err(e) => return HttpResponse::BadGateway().json(serde_json::json!({"error": e.to_string()})),
    };
    
    // 3. RBAC 필터링 적용
    let filtered = if let Some(array) = qido_response.as_array() {
        let mut allowed_items = Vec::new();
        for item in array.iter() {
            if let Some(series_uid) = extract_series_uid(item) {
                let result = evaluator.evaluate_series_uid(user_id, project_id, &series_uid).await;
                if result.allowed {
                    allowed_items.push(item.clone());
                }
            }
        }
        serde_json::Value::Array(allowed_items)
    } else {
        qido_response
    };
    
    HttpResponse::Ok().json(filtered)
}

pub async fn get_instances(
    qido: web::Data<Dcm4cheeQidoClient>,
    evaluator: web::Data<Arc<DicomRbacEvaluatorImpl>>,
    jwt: web::Data<Arc<JwtService>>,
    access_condition_repo: web::Data<Arc<AccessConditionRepositoryImpl>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
    path: web::Path<(String, String)>,
    query: web::Query<GatewayQuery>,
    req: HttpRequest,
) -> HttpResponse {
    let (study_uid, series_uid) = path.into_inner();
    let project_id = match query.project_id {
        Some(id) if id > 0 => id,
        _ => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "project_id is required and must be greater than 0"
            }));
        }
    };
    let user_id = match extract_user_id_from_token(&req, &jwt, &user_repo).await {
        Some(id) if id > 0 => id,
        _ => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Invalid or missing authorization token"
            }));
        }
    };
    
    // 1. 규칙 기반 QIDO 파라미터 병합
    let mut qido_params = json_to_params(query.extra.clone().into());
    if let Ok(conditions) = access_condition_repo.list_by_project(project_id).await {
        let rule_params = build_qido_params_from_conditions(&conditions);
        qido_params.extend(rule_params);
    }
    
    // 2. Dcm4chee QIDO 호출
    let bearer_opt = req.headers().get("Authorization").and_then(|h| h.to_str().ok()).and_then(|s| s.strip_prefix("Bearer ")).map(|s| s.to_string());
    let qido_response = match qido.qido_instances_with_bearer(bearer_opt.as_deref(), &study_uid, &series_uid, qido_params).await {
        Ok(json) => json,
        Err(e) => return HttpResponse::BadGateway().json(serde_json::json!({"error": e.to_string()})),
    };
    
    // 3. RBAC 필터링 적용
    let filtered = if let Some(array) = qido_response.as_array() {
        let mut allowed_items = Vec::new();
        for item in array.iter() {
            if let Some(instance_uid) = extract_instance_uid(item) {
                let result = evaluator.evaluate_instance_uid(user_id, project_id, &instance_uid).await;
                if result.allowed {
                    allowed_items.push(item.clone());
                }
            }
        }
        serde_json::Value::Array(allowed_items)
    } else {
        qido_response
    };
    
    HttpResponse::Ok().json(filtered)
}

/// Keycloak Bearer 토큰에서 사용자 ID 추출
/// 1. 우리 JWT 서비스 토큰인 경우: 직접 검증
/// 2. Keycloak 토큰인 경우: 디코딩하여 `sub` 필드 추출 후 DB에서 사용자 찾기
async fn extract_user_id_from_token(
    req: &HttpRequest,
    jwt: &Arc<JwtService>,
    user_repo: &Arc<UserRepositoryImpl>,
) -> Option<i32> {
    let token = req.headers()
        .get("Authorization")?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")?;

    // 먼저 우리 JWT 서비스 토큰인지 시도
    if let Ok(claims) = jwt.validate_token(token) {
        return claims.user_id().ok();
    }

    // Keycloak 토큰인 경우: JWT payload 디코딩
    if let Some(keycloak_id_str) = decode_keycloak_token_sub(token) {
        if let Ok(keycloak_id) = Uuid::parse_str(&keycloak_id_str) {
            // DB에서 사용자 찾기
            if let Ok(Some(user)) = user_repo.find_by_keycloak_id(keycloak_id).await {
                return Some(user.id);
            }
        }
    }
    
    None
}

/// Keycloak JWT 토큰에서 `sub` 필드 추출 (서명 검증 없이 디코딩만)
fn decode_keycloak_token_sub(token: &str) -> Option<String> {
    // JWT는 세 부분으로 나뉨: header.payload.signature
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return None;
    }

    let payload = parts[1];
    
    // Base64URL 디코딩 (패딩 추가하여 디코딩 시도)
    let mut padded = payload.to_string();
    while padded.len() % 4 != 0 {
        padded.push('=');
    }
    let decoded = base64::engine::general_purpose::URL_SAFE
        .decode(&padded)
        .ok()?;
    
    // JSON 파싱
    let json: serde_json::Value = serde_json::from_slice(&decoded).ok()?;
    
    // `sub` 필드 추출
    json.get("sub")?.as_str().map(|s| s.to_string())
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

fn json_to_params(v: Value) -> Vec<(String, String)> {
    let mut params = Vec::new();
    if let Value::Object(map) = v {
        for (k, v) in map.into_iter() {
            if let Some(s) = v.as_str() {
                params.push((k, s.to_string()));
            } else if v.is_number() || v.is_boolean() {
                params.push((k, v.to_string()));
            }
        }
    }
    params
}

pub(crate) fn build_qido_params_from_conditions(conds: &Vec<AccessCondition>) -> Vec<(String, String)> {
    let mut params = Vec::new();
    for c in conds.iter() {
        match c.operator.as_str() {
            "EQ" | "EQUALS" | "==" => {
                if let Some(tag) = &c.dicom_tag {
                    match tag.as_str() {
                        "00080060" | "Modality" => {
                            if let Some(val) = &c.value { params.push(("Modality".to_string(), val.clone())); }
                        }
                        "00100020" | "PatientID" => {
                            if let Some(val) = &c.value { params.push(("PatientID".to_string(), val.clone())); }
                        }
                        _ => {}
                    }
                }
            }
            "RANGE" | "BETWEEN" => {
                if let Some(tag) = &c.dicom_tag {
                    if tag == "00080020" || tag == "StudyDate" {
                        if let Some(val) = &c.value { params.push(("StudyDate".to_string(), val.clone())); }
                    }
                }
            }
            _ => {}
        }
    }
    params
}

#[cfg(test)]
mod tests {
    use super::{build_qido_params_from_conditions, decode_keycloak_token_sub, extract_study_uid, extract_series_uid, extract_instance_uid};
    use crate::domain::entities::access_condition::{AccessCondition, ResourceLevel, ConditionType};
    use actix_web::{App, web, HttpResponse};
    use actix_web::test;
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
        let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(b"{\"sub\":\"550e8400-e29b-41d4-a716-446655440000\"}");
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
}


