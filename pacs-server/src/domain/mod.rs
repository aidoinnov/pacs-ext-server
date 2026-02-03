pub mod entities;
pub mod errors;
pub mod repositories;
pub mod reporting;
pub mod services;
pub mod sw_information;
pub mod template;
pub mod view_selection;

// ServiceError를 직접 정의
#[derive(Debug, Clone)]
pub enum ServiceError {
    NotFound(String),
    AlreadyExists(String),
    ValidationError(String),
    ValidationFailed(String),
    DatabaseError(String),
    Unauthorized(String),
    Forbidden(String),
    ExternalServiceError(String),
    /// 버전 충돌 에러 (Optimistic Locking)
    /// 클라이언트가 제공한 버전과 서버의 현재 버전이 일치하지 않음
    VersionConflict {
        current_version: i32,
        client_version: i32,
    },
}

impl std::fmt::Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceError::NotFound(msg) => write!(f, "Not found: {}", msg),
            ServiceError::AlreadyExists(msg) => write!(f, "Already exists: {}", msg),
            ServiceError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            ServiceError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
            ServiceError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            ServiceError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            ServiceError::Forbidden(msg) => write!(f, "Forbidden: {}", msg),
            ServiceError::ExternalServiceError(msg) => write!(f, "External service error: {}", msg),
            ServiceError::VersionConflict {
                current_version,
                client_version,
            } => write!(
                f,
                "Version conflict: current version is {}, but client version is {}",
                current_version, client_version
            ),
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
            ServiceError::NotFound(_) => {
                actix_web::HttpResponse::NotFound().json(serde_json::json!({
                    "error": self.to_string()
                }))
            }
            ServiceError::AlreadyExists(_) => {
                actix_web::HttpResponse::Conflict().json(serde_json::json!({
                    "error": self.to_string()
                }))
            }
            ServiceError::ValidationError(_) | ServiceError::ValidationFailed(_) => {
                actix_web::HttpResponse::BadRequest().json(serde_json::json!({
                    "error": self.to_string()
                }))
            }
            ServiceError::Unauthorized(_) => {
                actix_web::HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": self.to_string()
                }))
            }
            ServiceError::Forbidden(_) => {
                actix_web::HttpResponse::Forbidden().json(serde_json::json!({
                    "error": self.to_string()
                }))
            }
            ServiceError::VersionConflict {
                current_version,
                client_version,
            } => {
                actix_web::HttpResponse::Conflict().json(serde_json::json!({
                    "error": "Version conflict",
                    "message": self.to_string(),
                    "current_version": current_version,
                    "client_version": client_version,
                }))
            }
            _ => actix_web::HttpResponse::InternalServerError().json(serde_json::json!({
                "error": self.to_string()
            })),
        }
    }
}
