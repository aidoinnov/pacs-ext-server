use crate::domain::entities::User;
use crate::domain::repositories::UserRepository;
use crate::domain::ServiceError;
use crate::infrastructure::auth::{Claims, JwtService};
use crate::infrastructure::external::KeycloakClient;
use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

/// 인증 도메인 서비스
#[async_trait]
pub trait AuthService: Send + Sync {
    /// 사용자 로그인 (Keycloak ID로)
    async fn login(
        &self,
        keycloak_id: Uuid,
        username: String,
        email: String,
    ) -> Result<AuthResponse, ServiceError>;

    /// 토큰 검증 및 사용자 조회
    async fn verify_and_get_user(&self, token: &str) -> Result<User, ServiceError>;

    /// 토큰 갱신
    async fn refresh_token(&self, user: &User) -> Result<String, ServiceError>;

    /// 로그아웃 (토큰 무효화) - 현재는 단순히 성공 반환
    async fn logout(&self, _token: &str) -> Result<(), ServiceError>;

    /// Keycloak을 사용한 토큰 갱신
    async fn refresh_token_with_keycloak(
        &self,
        refresh_token: &str,
    ) -> Result<crate::application::dto::auth_dto::RefreshTokenResponse, ServiceError>;

    /// 사용자 비밀번호 재설정 (Keycloak)
    async fn reset_user_password(
        &self,
        keycloak_user_id: &str,
        new_password: &str,
    ) -> Result<(), ServiceError>;

    /// 이메일로 사용자명 찾기
    async fn find_username_by_email(&self, email: &str) -> Result<User, ServiceError>;

    /// 사용자명과 이메일로 비밀번호 재설정
    async fn reset_password_by_credentials(
        &self,
        username: &str,
        email: &str,
        new_password: &str,
    ) -> Result<(), ServiceError>;
}

pub struct AuthServiceImpl<U: UserRepository> {
    user_repository: U,
    jwt_service: JwtService,
    keycloak_client: Arc<KeycloakClient>,
}

impl<U: UserRepository> AuthServiceImpl<U> {
    pub fn new(
        user_repository: U,
        jwt_service: JwtService,
        keycloak_client: Arc<KeycloakClient>,
    ) -> Self {
        Self {
            user_repository,
            jwt_service,
            keycloak_client,
        }
    }
}

#[async_trait]
impl<U: UserRepository> AuthService for AuthServiceImpl<U> {
    async fn login(
        &self,
        keycloak_id: Uuid,
        username: String,
        email: String,
    ) -> Result<AuthResponse, ServiceError> {
        // UPSERT 패턴으로 동시 로그인 Race condition 방지
        let user = sqlx::query_as::<_, crate::domain::entities::User>(
            "INSERT INTO security_user (keycloak_id, username, email)
             VALUES ($1, $2, $3)
             ON CONFLICT (keycloak_id) DO UPDATE
             SET username = EXCLUDED.username,
                 email = EXCLUDED.email
             RETURNING id, keycloak_id, username, email, created_at",
        )
        .bind(keycloak_id)
        .bind(&username)
        .bind(&email)
        .fetch_one(self.user_repository.pool())
        .await?;

        // JWT 토큰 생성
        let claims = Claims::new(
            user.id,
            user.keycloak_id,
            user.username.clone(),
            user.email.clone(),
            24, // 24시간 유효
        );

        let token = self
            .jwt_service
            .create_token(&claims)
            .map_err(|e| ServiceError::Unauthorized(format!("Failed to create token: {}", e)))?;

        Ok(AuthResponse { user, token })
    }

    async fn verify_and_get_user(&self, token: &str) -> Result<User, ServiceError> {
        // 토큰 검증
        let claims = self
            .jwt_service
            .validate_token(token)
            .map_err(|e| ServiceError::Unauthorized(format!("Invalid token: {}", e)))?;

        // Claims의 만료 여부 확인
        if claims.is_expired() {
            return Err(ServiceError::Unauthorized("Token has expired".into()));
        }

        // 사용자 ID로 사용자 조회
        let user_id = claims.user_id().map_err(|e| {
            ServiceError::ValidationError(format!("Invalid user ID in token: {}", e))
        })?;

        self.user_repository
            .find_by_id(user_id)
            .await?
            .ok_or(ServiceError::NotFound("User not found".into()))
    }

    async fn refresh_token(&self, user: &User) -> Result<String, ServiceError> {
        // 새로운 토큰 생성
        let claims = Claims::new(
            user.id,
            user.keycloak_id,
            user.username.clone(),
            user.email.clone(),
            24, // 24시간 유효
        );

        self.jwt_service
            .create_token(&claims)
            .map_err(|e| ServiceError::Unauthorized(format!("Failed to refresh token: {}", e)))
    }

    async fn logout(&self, _token: &str) -> Result<(), ServiceError> {
        // 실제 구현에서는 토큰 블랙리스트에 추가하거나 Redis 등에서 세션 제거
        // 현재는 단순히 성공 반환
        Ok(())
    }

    async fn refresh_token_with_keycloak(
        &self,
        refresh_token: &str,
    ) -> Result<crate::application::dto::auth_dto::RefreshTokenResponse, ServiceError> {
        // Keycloak의 refresh token endpoint 호출
        let keycloak_response = self
            .keycloak_client
            .refresh_access_token(refresh_token)
            .await?;

        // Keycloak 응답을 우리 DTO로 변환
        Ok(crate::application::dto::auth_dto::RefreshTokenResponse {
            token: keycloak_response.access_token,
            token_type: keycloak_response.token_type,
            expires_in: keycloak_response.expires_in,
        })
    }

    async fn reset_user_password(
        &self,
        keycloak_user_id: &str,
        new_password: &str,
    ) -> Result<(), ServiceError> {
        self.keycloak_client
            .reset_user_password(keycloak_user_id, new_password)
            .await
    }

    async fn find_username_by_email(&self, email: &str) -> Result<User, ServiceError> {
        self.user_repository
            .find_by_email(email)
            .await?
            .ok_or(ServiceError::NotFound(
                "해당 이메일로 등록된 사용자가 없습니다.".into(),
            ))
    }

    async fn reset_password_by_credentials(
        &self,
        username: &str,
        email: &str,
        new_password: &str,
    ) -> Result<(), ServiceError> {
        // 1. 비밀번호 강도 검증
        if new_password.len() < 8 {
            return Err(ServiceError::ValidationError(
                "비밀번호는 최소 8자 이상이어야 합니다.".into(),
            ));
        }

        // 2. 사용자 존재 확인 (username + email 일치 확인)
        let user = self
            .user_repository
            .find_by_username(username)
            .await?
            .ok_or(ServiceError::NotFound("사용자를 찾을 수 없습니다.".into()))?;

        if user.email != email {
            return Err(ServiceError::ValidationError(
                "이메일 정보가 일치하지 않습니다.".into(),
            ));
        }

        // 3. Keycloak 비밀번호 재설정
        self.keycloak_client
            .reset_user_password(&user.keycloak_id.to_string(), new_password)
            .await?;

        Ok(())
    }
}

/// 인증 응답
#[derive(Debug)]
pub struct AuthResponse {
    pub user: User,
    pub token: String,
}
