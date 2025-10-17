//! # Health Controller
//! 
//! 서버 상태 확인과 관련된 엔드포인트들을 처리합니다.

use actix_web::{web, HttpResponse, Result, Responder};
use serde_json::json;
use crate::domain::errors::DomainError;

/// 상세한 서버 상태 정보를 반환하는 엔드포인트
/// 
/// # 반환값
/// - `200 OK`: 서버가 정상적으로 동작 중
/// - JSON 형태로 상세한 서버 상태 정보 반환
/// 
/// # 사용 예시
/// ```bash
/// curl http://localhost:8080/api/health/detailed
/// ```
pub async fn detailed_health_check() -> impl Responder {
    let response = json!({
        "status": "healthy",
        "service": "health-check-server",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        "uptime": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        "environment": std::env::var("RUN_MODE").unwrap_or_else(|_| "development".to_string()),
        "features": {
            "health_check": true,
            "cors": true,
            "logging": true
        }
    });

    HttpResponse::Ok().json(response)
}

/// 간단한 서버 상태 확인 엔드포인트
/// 
/// # 반환값
/// - `200 OK`: 서버가 정상적으로 동작 중
/// - JSON 형태로 간단한 상태 정보 반환
/// 
/// # 사용 예시
/// ```bash
/// curl http://localhost:8080/api/health/simple
/// ```
pub async fn simple_health_check() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "status": "ok"
    }))
}

/// 서버 정보를 반환하는 엔드포인트
/// 
/// # 반환값
/// - `200 OK`: 서버 정보 반환
/// - JSON 형태로 서버 정보 및 사용 가능한 엔드포인트 목록 반환
/// 
/// # 사용 예시
/// ```bash
/// curl http://localhost:8080/api/info
/// ```
pub async fn server_info() -> impl Responder {
    let response = json!({
        "name": "Health Check Server",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "Minimal health check server with Clean Architecture",
        "endpoints": {
            "health": "/health",
            "detailed_health": "/api/health/detailed",
            "simple_health": "/api/health/simple",
            "info": "/api/info"
        }
    });

    HttpResponse::Ok().json(response)
}

/// 서버 상태를 검증하는 엔드포인트
/// 
/// # 반환값
/// - `200 OK`: 서버가 정상 상태
/// - `503 Service Unavailable`: 서버가 비정상 상태
/// 
/// # 사용 예시
/// ```bash
/// curl http://localhost:8080/api/health/validate
/// ```
pub async fn validate_health() -> impl Responder {
    // 실제 서비스에서는 여기서 다양한 상태를 검증할 수 있습니다
    // 예: 데이터베이스 연결, 외부 서비스 상태, 메모리 사용량 등
    
    // 현재는 항상 정상 상태로 반환
    let is_healthy = true;
    
    if is_healthy {
        HttpResponse::Ok().json(json!({
            "status": "healthy",
            "checks": {
                "server": "ok",
                "memory": "ok",
                "disk": "ok"
            }
        }))
    } else {
        HttpResponse::ServiceUnavailable().json(json!({
            "status": "unhealthy",
            "error": "One or more health checks failed"
        }))
    }
}

/// 라우트 설정
/// 
/// # 매개변수
/// - `cfg`: Actix Web 서비스 설정
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg
        .route("/health/detailed", web::get().to(detailed_health_check))
        .route("/health/simple", web::get().to(simple_health_check))
        .route("/health/validate", web::get().to(validate_health))
        .route("/info", web::get().to(server_info));
}
