use std::sync::Arc;
use actix_web::{test, App};
use serde_json::json;
use mockall::mock;
use async_trait::async_trait;

use pacs_server::application::use_cases::auth_use_case::AuthUseCase;
use pacs_server::application::dto::auth_dto::{RefreshTokenRequest, RefreshTokenResponse};
use pacs_server::domain::services::auth_service::AuthService;
use pacs_server::domain::ServiceError;

// Mock AuthService
mock! {
    AuthServiceImpl {}

    #[async_trait]
    impl AuthService for AuthServiceImpl {
        async fn login(&self, keycloak_id: uuid::Uuid, username: String, email: String) -> Result<pacs_server::domain::services::auth_service::AuthResponse, ServiceError>;
        async fn verify_and_get_user(&self, token: &str) -> Result<pacs_server::domain::entities::User, ServiceError>;
        async fn refresh_token(&self, user: &pacs_server::domain::entities::User) -> Result<String, ServiceError>;
        async fn logout(&self, token: &str) -> Result<(), ServiceError>;
        async fn refresh_token_with_keycloak(&self, refresh_token: &str) -> Result<RefreshTokenResponse, ServiceError>;
    }
}

#[tokio::test]
async fn test_refresh_token_use_case_success() {
    // Given
    let mut mock_auth_service = MockAuthServiceImpl::new();
    
    let expected_response = RefreshTokenResponse {
        token: "new-access-token".to_string(),
        token_type: "Bearer".to_string(),
        expires_in: 3600,
    };
    
    mock_auth_service
        .expect_refresh_token_with_keycloak()
        .times(1)
        .with(mockall::predicate::eq("test-refresh-token"))
        .returning(move |_| Ok(expected_response.clone()));
    
    let auth_use_case = AuthUseCase::new(mock_auth_service);
    let request = RefreshTokenRequest {
        refresh_token: "test-refresh-token".to_string(),
    };
    
    // When
    let result = auth_use_case.refresh_token(request).await;
    
    // Then
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.token, "new-access-token");
    assert_eq!(response.token_type, "Bearer");
    assert_eq!(response.expires_in, 3600);
}

#[tokio::test]
async fn test_refresh_token_use_case_failure() {
    // Given
    let mut mock_auth_service = MockAuthServiceImpl::new();
    
    mock_auth_service
        .expect_refresh_token_with_keycloak()
        .times(1)
        .with(mockall::predicate::eq("invalid-refresh-token"))
        .returning(|_| Err(ServiceError::ExternalServiceError("Invalid refresh token".to_string())));
    
    let auth_use_case = AuthUseCase::new(mock_auth_service);
    let request = RefreshTokenRequest {
        refresh_token: "invalid-refresh-token".to_string(),
    };
    
    // When
    let result = auth_use_case.refresh_token(request).await;
    
    // Then
    assert!(result.is_err());
    match result.unwrap_err() {
        ServiceError::ExternalServiceError(msg) => {
            assert_eq!(msg, "Invalid refresh token");
        }
        _ => panic!("Expected ExternalServiceError"),
    }
}