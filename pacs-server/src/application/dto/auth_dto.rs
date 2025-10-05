use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 로그인 요청 DTO
#[derive(Debug, Deserialize, Serialize)]
pub struct LoginRequest {
    pub keycloak_id: Uuid,
    pub username: String,
    pub email: String,
}

/// 로그인 응답 DTO
#[derive(Debug, Deserialize, Serialize)]
pub struct LoginResponse {
    pub user_id: i32,
    pub keycloak_id: Uuid,
    pub username: String,
    pub email: String,
    pub token: String,
    pub token_type: String, // "Bearer"
    pub expires_in: i64,    // seconds
}

/// 토큰 갱신 요청 DTO
#[derive(Debug, Deserialize, Serialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

/// 토큰 갱신 응답 DTO
#[derive(Debug, Deserialize, Serialize)]
pub struct RefreshTokenResponse {
    pub token: String,
    pub token_type: String,
    pub expires_in: i64,
}

/// 토큰 검증 응답 DTO
#[derive(Debug, Deserialize, Serialize)]
pub struct VerifyTokenResponse {
    pub user_id: i32,
    pub keycloak_id: Uuid,
    pub username: String,
    pub email: String,
    pub is_valid: bool,
}
