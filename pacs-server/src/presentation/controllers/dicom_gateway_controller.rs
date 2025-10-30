use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;
use base64::Engine;

use crate::domain::services::DicomRbacEvaluator;
use crate::infrastructure::external::Dcm4cheeQidoClient;
use crate::infrastructure::auth::JwtService;
use crate::infrastructure::repositories::{AccessConditionRepositoryImpl, UserRepositoryImpl};
use crate::domain::repositories::{AccessConditionRepository, UserRepository};
use crate::domain::entities::access_condition::AccessCondition;
use crate::infrastructure::services::DicomRbacEvaluatorImpl;
use uuid::Uuid;
use std::collections::HashMap;

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
    
    // 1. 규칙 기반 QIDO 파라미터 병합 + 사용자 입력 우선 병합
    // 사용자 필터/페이지네이션 파라미터 파싱 및 검증
    let user_params = match build_qido_params_from_user_query(&query.extra) {
        Ok(p) => p,
        Err(msg) => {
            return HttpResponse::BadRequest().json(serde_json::json!({"error": msg}));
        }
    };
    let qido_params = if let Ok(conditions) = access_condition_repo.list_by_project(project_id).await {
        let rule_params = build_qido_params_from_conditions(&conditions);
        merge_qido_params(rule_params, user_params) // 사용자 입력이 우선
    } else {
        user_params
    };
    
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
    
    // 1. 규칙 기반 QIDO 파라미터 병합 + 사용자 입력 우선 병합
    let user_params = match build_qido_params_from_user_query(&query.extra) {
        Ok(p) => p,
        Err(msg) => {
            return HttpResponse::BadRequest().json(serde_json::json!({"error": msg}));
        }
    };
    let qido_params = if let Ok(conditions) = access_condition_repo.list_by_project(project_id).await {
        let rule_params = build_qido_params_from_conditions(&conditions);
        merge_qido_params(rule_params, user_params)
    } else {
        user_params
    };
    
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
    
    // 1. 규칙 기반 QIDO 파라미터 병합 + 사용자 입력 우선 병합
    let user_params = match build_qido_params_from_user_query(&query.extra) {
        Ok(p) => p,
        Err(msg) => {
            return HttpResponse::BadRequest().json(serde_json::json!({"error": msg}));
        }
    };
    let qido_params = if let Ok(conditions) = access_condition_repo.list_by_project(project_id).await {
        let rule_params = build_qido_params_from_conditions(&conditions);
        merge_qido_params(rule_params, user_params)
    } else {
        user_params
    };
    
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

// json_to_params: 이전 일반 쿼리 전달용 유틸은 사용자 필터 전용 파서로 대체됨

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
                        "00080050" | "AccessionNumber" => {
                            if let Some(val) = &c.value { params.push(("AccessionNumber".to_string(), val.clone())); }
                        }
                        "00100010" | "PatientName" => {
                            if let Some(val) = &c.value { params.push(("PatientName".to_string(), val.clone())); }
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
                            if let Some(val) = &c.value { params.push(("AccessionNumber".to_string(), val.clone())); }
                        }
                        "00100010" | "PatientName" => {
                            if let Some(val) = &c.value { params.push(("PatientName".to_string(), val.clone())); }
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
            // NE(불일치) 등은 QIDO 파라미터로 표현하기 어려워 사후 필터에 위임
            _ => {}
        }
    }
    params
}

// 사용자 쿼리에서 지원하는 파라미터를 QIDO 파라미터로 변환하며 검증을 수행한다
fn build_qido_params_from_user_query(extra: &serde_json::Map<String, Value>) -> Result<Vec<(String, String)>, String> {
    let mut params: HashMap<String, String> = HashMap::new();

    // 필터: modality/patient_id/study_date/optional accession_number/patient_name
    if let Some(v) = extra.get("modality").and_then(|v| v.as_str()) { params.insert("Modality".to_string(), v.to_string()); }
    if let Some(v) = extra.get("patient_id").and_then(|v| v.as_str()) { params.insert("PatientID".to_string(), v.to_string()); }
    if let Some(v) = extra.get("accession_number").and_then(|v| v.as_str()) { params.insert("AccessionNumber".to_string(), v.to_string()); }
    if let Some(v) = extra.get("patient_name").and_then(|v| v.as_str()) { params.insert("PatientName".to_string(), v.to_string()); }

    if let Some(sd) = extra.get("study_date").and_then(|v| v.as_str()) {
        if !is_valid_study_date(sd) { return Err("Invalid study_date format. Use YYYYMMDD or YYYYMMDD-YYYYMMDD".to_string()); }
        params.insert("StudyDate".to_string(), sd.to_string());
    }

    // 페이지네이션: limit/offset이 명시되면 그대로 사용, 없을 때만 page/page_size를 limit/offset으로 변환
    let has_limit = extra.get("limit").is_some();
    let has_offset = extra.get("offset").is_some();
    if !has_limit || !has_offset {
        let page_size = extra.get("page_size").and_then(|v| v.as_i64()).unwrap_or(50).clamp(1, 200) as i64;
        let page = extra.get("page").and_then(|v| v.as_i64()).unwrap_or(1).max(1);
        let offset = (page - 1) * page_size;
        if !has_limit { params.insert("limit".to_string(), page_size.to_string()); }
        if !has_offset { params.insert("offset".to_string(), offset.to_string()); }
    }

    // DICOMweb 네이티브 파라미터 패스스루: 알려진 필드 외 문자열/숫자/불리언은 그대로 전달
    for (k, v) in extra.iter() {
        // 내부 파라미터는 전달하지 않음
        if matches!(k.as_str(), "project_id" | "page" | "page_size") { continue; }
        // 소문자 사용자 별칭은 이미 위에서 변환 처리됨(modality/patient_id/study_date/accession_number/patient_name)
        if matches!(k.as_str(), "modality" | "patient_id" | "study_date" | "accession_number" | "patient_name") { continue; }
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
    if bytes.len() == 8 { return bytes.iter().all(|c| c.is_ascii_digit()); }
    if bytes.len() == 17 && bytes[8] == b'-' {
        return bytes[..8].iter().all(|c| c.is_ascii_digit()) && bytes[9..].iter().all(|c| c.is_ascii_digit());
    }
    false
}

fn merge_qido_params(rule_params: Vec<(String, String)>, user_params: Vec<(String, String)>) -> Vec<(String, String)> {
    // rule 먼저 넣고, 같은 키는 user 값으로 덮어씀
    let mut map: HashMap<String, String> = HashMap::new();
    for (k, v) in rule_params { map.insert(k, v); }
    for (k, v) in user_params { map.insert(k, v); }
    map.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::{build_qido_params_from_conditions, decode_keycloak_token_sub, extract_study_uid, extract_series_uid, extract_instance_uid, build_qido_params_from_user_query, is_valid_study_date, merge_qido_params};
    use crate::domain::entities::access_condition::{AccessCondition, ResourceLevel, ConditionType};
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
        extra.insert("modality".to_string(), serde_json::Value::String("CT".to_string()));
        extra.insert("patient_id".to_string(), serde_json::Value::String("PAT001".to_string()));
        extra.insert("study_date".to_string(), serde_json::Value::String("20240101-20241231".to_string()));
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
        extra.insert("study_date".to_string(), serde_json::Value::String("2024-0101".to_string()));
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
}


