//! # Health Controller
//! 
//! 헬스체크 관련 HTTP 요청을 처리하는 컨트롤러입니다.

use actix_web::{web, HttpResponse, Responder, Result};
use serde_json::json;

use crate::application::use_cases::HealthCheckUseCase;
use crate::application::dto::{HealthCheckResponse, ServerInfoResponse, BasicResponse, EndpointInfo};
use crate::domain::errors::DomainError;

/// 헬스체크 컨트롤러
pub struct HealthController {
    health_check_use_case: HealthCheckUseCase,
}

impl HealthController {
    /// 새로운 헬스체크 컨트롤러 생성
    pub fn new(health_check_use_case: HealthCheckUseCase) -> Self {
        Self {
            health_check_use_case,
        }
    }
}

/// 헬스체크 엔드포인트
/// 
/// # 반환값
/// - `200 OK`: 서버가 정상적으로 동작 중
/// - `500 Internal Server Error`: 서버에 문제가 있음
/// 
/// # 사용 예시
/// ```bash
/// curl http://localhost:8080/api/health
/// ```
pub async fn health_check(
    data: web::Data<HealthController>,
) -> Result<HttpResponse, actix_web::Error> {
    match data.health_check_use_case.check_health().await {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(DomainError::UnhealthyServer { message }) => {
            Ok(HttpResponse::InternalServerError().json(json!({
                "status": "unhealthy",
                "error": message,
                "timestamp": chrono::Utc::now().to_rfc3339()
            })))
        }
        Err(err) => {
            tracing::error!("Health check failed: {}", err);
            Ok(HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "error": "Internal server error",
                "timestamp": chrono::Utc::now().to_rfc3339()
            })))
        }
    }
}

/// 서버 정보 엔드포인트
/// 
/// # 반환값
/// - `200 OK`: 서버 정보 반환
/// - `500 Internal Server Error`: 서버에 문제가 있음
/// 
/// # 사용 예시
/// ```bash
/// curl http://localhost:8080/api/info
/// ```
pub async fn server_info(
    data: web::Data<HealthController>,
) -> Result<HttpResponse, actix_web::Error> {
    match data.health_check_use_case.get_server_info().await {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(err) => {
            tracing::error!("Server info failed: {}", err);
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to get server info",
                "timestamp": chrono::Utc::now().to_rfc3339()
            })))
        }
    }
}

/// 기본 응답 엔드포인트
/// 
/// # 반환값
/// - `200 OK`: 기본 응답 반환
pub async fn basic_response() -> Result<HttpResponse, actix_web::Error> {
    let response = BasicResponse {
        message: "Welcome to Basic PACS Server API".to_string(),
        endpoints: EndpointInfo {
            health: "/api/health".to_string(),
            info: "/api/info".to_string(),
            api: "/api/".to_string(),
        },
        timestamp: chrono::Utc::now(),
    };
    
    Ok(HttpResponse::Ok().json(response))
}

/// 라우트 설정
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg
        .route("/health", web::get().to(health_check))
        .route("/info", web::get().to(server_info))
        .route("/", web::get().to(basic_response));
}
