use crate::application::dto::{
    mask_email, FindUsernameResponse, LoginRequest, LoginResponse, RefreshTokenRequest,
    RefreshTokenResponse, ResetPasswordResponse, VerifyTokenResponse,
};
use crate::domain::services::AuthService;
use crate::domain::ServiceError;

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

    /// 토큰 갱신 (Keycloak 사용)
    pub async fn refresh_token(
        &self,
        request: RefreshTokenRequest,
    ) -> Result<RefreshTokenResponse, ServiceError> {
        // Keycloak의 refresh token endpoint를 통해 토큰 갱신
        self.auth_service
            .refresh_token_with_keycloak(&request.refresh_token)
            .await
    }

    /// 로그아웃
    pub async fn logout(&self, token: &str) -> Result<(), ServiceError> {
        self.auth_service.logout(token).await
    }

    /// 아이디 찾기
    pub async fn find_username(&self, email: &str) -> Result<FindUsernameResponse, ServiceError> {
        let user = self.auth_service.find_username_by_email(email).await?;

        // 이메일 마스킹
        let masked_email = mask_email(&user.email);

        Ok(FindUsernameResponse {
            username: user.username,
            masked_email,
            message: "아이디를 찾았습니다.".to_string(),
        })
    }

    /// 비밀번호 재설정
    pub async fn reset_password(
        &self,
        username: &str,
        email: &str,
        new_password: &str,
    ) -> Result<ResetPasswordResponse, ServiceError> {
        self.auth_service
            .reset_password_by_credentials(username, email, new_password)
            .await?;

        Ok(ResetPasswordResponse {
            message: "비밀번호가 성공적으로 재설정되었습니다.".to_string(),
        })
    }
}
