use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// 이메일 마스킹 함수
pub fn mask_email(email: &str) -> String {
    if let Some(at_pos) = email.find('@') {
        let (local, domain) = email.split_at(at_pos);
        let masked_local = if local.len() > 2 {
            format!("{}***", &local[..1])
        } else {
            "***".to_string()
        };
        format!("{}@{}", masked_local, domain)
    } else {
        email.to_string()
    }
}

/// 로그인 요청 DTO
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct LoginRequest {
    pub keycloak_id: Uuid,
    pub username: String,
    pub email: String,
}

/// 로그인 응답 DTO
#[derive(Debug, Deserialize, Serialize, ToSchema)]
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
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

/// 토큰 갱신 응답 DTO
#[derive(Debug, Deserialize, Serialize, ToSchema, Clone)]
pub struct RefreshTokenResponse {
    pub token: String,
    pub token_type: String,
    pub expires_in: i64,
}

/// 토큰 검증 응답 DTO
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct VerifyTokenResponse {
    pub user_id: i32,
    pub keycloak_id: Uuid,
    pub username: String,
    pub email: String,
    pub is_valid: bool,
}

/// 아이디 찾기 요청 DTO
#[derive(Debug, Deserialize, ToSchema)]
pub struct FindUsernameRequest {
    pub email: String,
}

/// 아이디 찾기 응답 DTO
#[derive(Debug, Serialize, ToSchema)]
pub struct FindUsernameResponse {
    pub username: String,
    pub masked_email: String,
    pub message: String,
}

/// 비밀번호 재설정 요청 DTO
#[derive(Debug, Deserialize, ToSchema)]
pub struct ResetPasswordRequest {
    pub username: String,
    pub email: String,
    pub new_password: String,
}

/// 비밀번호 재설정 응답 DTO
#[derive(Debug, Serialize, ToSchema)]
pub struct ResetPasswordResponse {
    pub message: String,
}
