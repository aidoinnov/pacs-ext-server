use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::application::use_cases::data_access_check_use_case::DataAccessCheckUseCase;
use crate::infrastructure::auth::token_extractor::extract_user_id_from_request;
use crate::infrastructure::auth::JwtService;
use crate::infrastructure::repositories::UserRepositoryImpl;

/// 데이터 접근 권한 확인 요청
#[derive(Debug, Deserialize)]
pub struct DataAccessCheckRequest {
    /// Study UID (필수)
    pub study_uid: String,
    /// Series UID (선택)
    pub series_uid: Option<String>,
    /// Project ID (선택) - 특정 프로젝트에 대한 접근 권한만 확인
    pub project_id: Option<i32>,
}

/// 프로젝트별 접근 정보
#[derive(Debug, Serialize)]
pub struct ProjectAccessInfo {
    pub project_id: i32,
    pub project_name: String,
    pub access_level: String, // "STUDY", "SERIES"
    pub reason: String,       // "approved", "member", "denied"
}

/// 데이터 접근 권한 확인 응답
#[derive(Debug, Serialize)]
pub struct DataAccessCheckResponse {
    pub accessible: bool,
    pub projects: Vec<ProjectAccessInfo>,
}

/// 데이터 접근 권한 확인 API
///
/// POST /api/v1/access/check
///
/// 사용자가 특정 Study/Series에 접근 가능한지 확인합니다.
///
/// # 로직
/// 1. 사용자가 속한 모든 프로젝트 조회
/// 2. 각 프로젝트에서 해당 Study/Series 접근 권한 확인
///    - 프로젝트 멤버십 확인
///    - project_data_access 테이블 확인
///    - RBAC 평가
/// 3. 접근 가능한 프로젝트 목록 반환
pub async fn check_data_access(
    http_req: HttpRequest,
    req: web::Json<DataAccessCheckRequest>,
    use_case: web::Data<Arc<DataAccessCheckUseCase>>,
    jwt: web::Data<Arc<JwtService>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
) -> HttpResponse {
    // 1. 사용자 ID 추출
    let user_id = match extract_user_id_from_request(&http_req, &jwt, &user_repo).await {
        Some(id) if id > 0 => id,
        _ => {
            tracing::error!("Failed to extract user_id from request");
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Unauthorized",
                "message": "Invalid or missing authentication token"
            }));
        }
    };

    // 2. 입력 검증
    if req.study_uid.trim().is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Bad Request",
            "message": "study_uid is required and cannot be empty"
        }));
    }

    // 3. UseCase 호출
    match use_case
        .check_access(user_id, &req.study_uid, req.series_uid.as_deref(), req.project_id)
        .await
    {
        Ok(result) => {
            let response = DataAccessCheckResponse {
                accessible: !result.projects.is_empty(),
                projects: result
                    .projects
                    .into_iter()
                    .map(|p| ProjectAccessInfo {
                        project_id: p.project_id,
                        project_name: p.project_name,
                        access_level: p.access_level,
                        reason: p.reason,
                    })
                    .collect(),
            };

            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            tracing::error!(
                "Error checking data access for user {} on study {}: {:?}",
                user_id,
                req.study_uid,
                e
            );
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Internal Server Error",
                "message": e.to_string()
            }))
        }
    }
}

/// 라우트 설정
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/v1/dicom/access")
            .route("/check", web::post().to(check_data_access)),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_validation() {
        let req = DataAccessCheckRequest {
            study_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.1".to_string(),
            series_uid: None,
            project_id: None,
        };
        assert!(!req.study_uid.is_empty());
    }

    #[test]
    fn test_request_with_series() {
        let req = DataAccessCheckRequest {
            study_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.1".to_string(),
            series_uid: Some("1.2.840.113619.2.55.3.604688119.868.1234567890.2".to_string()),
            project_id: Some(1),
        };
        assert!(req.series_uid.is_some());
    }
}

