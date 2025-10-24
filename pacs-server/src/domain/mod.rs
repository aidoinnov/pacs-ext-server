pub mod entities;
pub mod repositories;
pub mod services;
pub mod errors;

// ServiceError를 직접 정의
#[derive(Debug, Clone)]
pub enum ServiceError {
    NotFound(String),
    AlreadyExists(String),
    ValidationError(String),
    DatabaseError(String),
    Unauthorized(String),
    ExternalServiceError(String),
}

impl std::fmt::Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceError::NotFound(msg) => write!(f, "Not found: {}", msg),
            ServiceError::AlreadyExists(msg) => write!(f, "Already exists: {}", msg),
            ServiceError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            ServiceError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            ServiceError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            ServiceError::ExternalServiceError(msg) => write!(f, "External service error: {}", msg),
        }
    }
}

impl std::error::Error for ServiceError {}

impl From<reqwest::Error> for ServiceError {
    fn from(err: reqwest::Error) -> Self {
        ServiceError::ExternalServiceError(err.to_string())
    }
}

impl From<sqlx::Error> for ServiceError {
    fn from(err: sqlx::Error) -> Self {
        ServiceError::DatabaseError(err.to_string())
    }
}

impl actix_web::ResponseError for ServiceError {
    fn error_response(&self) -> actix_web::HttpResponse {
        match self {
            ServiceError::NotFound(_) => actix_web::HttpResponse::NotFound().json(serde_json::json!({
                "error": self.to_string()
            })),
            ServiceError::AlreadyExists(_) => actix_web::HttpResponse::Conflict().json(serde_json::json!({
                "error": self.to_string()
            })),
            ServiceError::ValidationError(_) => actix_web::HttpResponse::BadRequest().json(serde_json::json!({
                "error": self.to_string()
            })),
            ServiceError::Unauthorized(_) => actix_web::HttpResponse::Unauthorized().json(serde_json::json!({
                "error": self.to_string()
            })),
            _ => actix_web::HttpResponse::InternalServerError().json(serde_json::json!({
                "error": self.to_string()
            })),
        }
    }
}
