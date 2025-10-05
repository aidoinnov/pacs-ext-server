use crate::application::dto::{
    LoginRequest, LoginResponse, RefreshTokenResponse, VerifyTokenResponse,
};
use crate::domain::services::{AuthService, ServiceError};

/// 인증 유스케이스
pub struct AuthUseCase<A: AuthService> {
    auth_service: A,
}

impl<A: AuthService> AuthUseCase<A> {
    pub fn new(auth_service: A) -> Self {
        Self { auth_service }
    }

    /// 로그인
    pub async fn login(&self, request: LoginRequest) -> Result<LoginResponse, ServiceError> {
        let auth_response = self
            .auth_service
            .login(request.keycloak_id, request.username, request.email)
            .await?;

        Ok(LoginResponse {
            user_id: auth_response.user.id,
            keycloak_id: auth_response.user.keycloak_id,
            username: auth_response.user.username,
            email: auth_response.user.email,
            token: auth_response.token,
            token_type: "Bearer".to_string(),
            expires_in: 24 * 60 * 60, // 24 hours in seconds
        })
    }

    /// 토큰 검증
    pub async fn verify_token(&self, token: &str) -> Result<VerifyTokenResponse, ServiceError> {
        let user = self.auth_service.verify_and_get_user(token).await?;

        Ok(VerifyTokenResponse {
            user_id: user.id,
            keycloak_id: user.keycloak_id,
            username: user.username,
            email: user.email,
            is_valid: true,
        })
    }

    /// 토큰 갱신
    pub async fn refresh_token(&self, token: &str) -> Result<RefreshTokenResponse, ServiceError> {
        // 먼저 토큰 검증
        let user = self.auth_service.verify_and_get_user(token).await?;

        // 새 토큰 생성
        let new_token = self.auth_service.refresh_token(&user).await?;

        Ok(RefreshTokenResponse {
            token: new_token,
            token_type: "Bearer".to_string(),
            expires_in: 24 * 60 * 60,
        })
    }

    /// 로그아웃
    pub async fn logout(&self, token: &str) -> Result<(), ServiceError> {
        self.auth_service.logout(token).await
    }
}
