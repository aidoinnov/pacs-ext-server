//! 공통 에러 처리 유틸리티

use actix_web::HttpResponse;
use serde_json::json;

use crate::domain::ServiceError;

/// ServiceError를 HttpResponse로 변환
pub fn handle_service_error(error: ServiceError) -> HttpResponse {
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
        _ => HttpResponse::InternalServerError().json(json!({
            "error": "Internal Server Error",
            "message": "An unexpected error occurred"
        })),
    }
}
