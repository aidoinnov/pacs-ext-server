use crate::application::dto::{
    LoginRequest, LoginResponse, RefreshTokenRequest, RefreshTokenResponse, VerifyTokenResponse,
    FindUsernameResponse, ResetPasswordResponse, mask_email,
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
    pub async fn refresh_token(&self, request: RefreshTokenRequest) -> Result<RefreshTokenResponse, ServiceError> {
        // Keycloak의 refresh token endpoint를 통해 토큰 갱신
        self.auth_service.refresh_token_with_keycloak(&request.refresh_token).await
    }

    /// 로그아웃
    pub async fn logout(&self, token: &str) -> Result<(), ServiceError> {
        self.auth_service.logout(token).await
    }

    /// 아이디 찾기
    pub async fn find_username(&self, email: &str) -> Result<FindUsernameResponse, ServiceError> {
        // TODO: AuthService를 확장하여 find_by_email을 추가해야 함
        Err(ServiceError::ValidationError("아이디 찾기는 아직 구현되지 않았습니다.".into()))
    }

    /// 비밀번호 재설정
    pub async fn reset_password(
        &self,
        username: &str,
        email: &str,
        new_password: &str,
    ) -> Result<ResetPasswordResponse, ServiceError> {
        // TODO: AuthService를 확장하여 필요한 메서드를 추가해야 함
        Err(ServiceError::ValidationError("비밀번호 재설정은 아직 구현되지 않았습니다.".into()))
    }
}
