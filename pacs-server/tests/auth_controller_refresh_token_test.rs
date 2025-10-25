use std::sync::Arc;
use actix_web::{test, web, App};
use serde_json::json;
use mockall::mock;
use async_trait::async_trait;

use pacs_server::presentation::controllers::auth_controller::configure_routes;
use pacs_server::application::use_cases::auth_use_case::AuthUseCase;
use pacs_server::application::dto::auth_dto::{RefreshTokenRequest, RefreshTokenResponse};
use pacs_server::domain::services::auth_service::AuthService;
use pacs_server::domain::ServiceError;
use pacs_server::application::use_cases::user_registration_use_case::UserRegistrationUseCase;
use pacs_server::infrastructure::services::UserRegistrationServiceImpl;

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

// Mock UserRegistrationUseCase
mock! {
    UserRegistrationUseCase {}

    #[async_trait]
    impl pacs_server::application::use_cases::user_registration_use_case::UserRegistrationUseCase for UserRegistrationUseCase {
        async fn signup(&self, request: pacs_server::application::dto::user_registration_dto::SignupRequest) -> Result<pacs_server::application::dto::user_registration_dto::SignupResponse, ServiceError>;
        async fn verify_email(&self, user_id: i32) -> Result<pacs_server::application::dto::user_registration_dto::VerifyEmailResponse, ServiceError>;
        async fn approve_user(&self, user_id: i32, admin_id: i32) -> Result<pacs_server::application::dto::user_registration_dto::ApproveUserResponse, ServiceError>;
        async fn delete_account(&self, user_id: i32, actor_id: Option<i32>) -> Result<pacs_server::application::dto::user_registration_dto::DeleteAccountResponse, ServiceError>;
    }
}

#[tokio::test]
async fn test_refresh_token_endpoint_success() {
    // Given
    let mut mock_auth_service = MockAuthServiceImpl::new();
    let mut mock_user_registration_use_case = MockUserRegistrationUseCase::new();
    
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
    
    let auth_use_case = Arc::new(AuthUseCase::new(mock_auth_service));
    let user_registration_use_case = Arc::new(mock_user_registration_use_case);
    
    let app = test::init_service(
        App::new()
            .configure(|cfg| configure_routes(cfg, auth_use_case, user_registration_use_case))
    ).await;
    
    let request_body = RefreshTokenRequest {
        refresh_token: "test-refresh-token".to_string(),
    };
    
    // When
    let req = test::TestRequest::post()
        .uri("/api/auth/refresh")
        .set_json(&request_body)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // Then
    assert_eq!(resp.status(), 200);
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["token"], "new-access-token");
    assert_eq!(body["token_type"], "Bearer");
    assert_eq!(body["expires_in"], 3600);
}

#[tokio::test]
async fn test_refresh_token_endpoint_invalid_token() {
    // Given
    let mut mock_auth_service = MockAuthServiceImpl::new();
    let mut mock_user_registration_use_case = MockUserRegistrationUseCase::new();
    
    mock_auth_service
        .expect_refresh_token_with_keycloak()
        .times(1)
        .with(mockall::predicate::eq("invalid-refresh-token"))
        .returning(|_| Err(ServiceError::ExternalServiceError("Invalid refresh token".to_string())));
    
    let auth_use_case = Arc::new(AuthUseCase::new(mock_auth_service));
    let user_registration_use_case = Arc::new(mock_user_registration_use_case);
    
    let app = test::init_service(
        App::new()
            .configure(|cfg| configure_routes(cfg, auth_use_case, user_registration_use_case))
    ).await;
    
    let request_body = RefreshTokenRequest {
        refresh_token: "invalid-refresh-token".to_string(),
    };
    
    // When
    let req = test::TestRequest::post()
        .uri("/api/auth/refresh")
        .set_json(&request_body)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // Then
    assert_eq!(resp.status(), 401);
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["error"].as_str().unwrap().contains("Token refresh failed"));
}

#[tokio::test]
async fn test_refresh_token_endpoint_malformed_json() {
    // Given
    let mut mock_auth_service = MockAuthServiceImpl::new();
    let mut mock_user_registration_use_case = MockUserRegistrationUseCase::new();
    
    let auth_use_case = Arc::new(AuthUseCase::new(mock_auth_service));
    let user_registration_use_case = Arc::new(mock_user_registration_use_case);
    
    let app = test::init_service(
        App::new()
            .configure(|cfg| configure_routes(cfg, auth_use_case, user_registration_use_case))
    ).await;
    
    // When
    let req = test::TestRequest::post()
        .uri("/api/auth/refresh")
        .set_header("content-type", "application/json")
        .set_payload("invalid json")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // Then
    assert_eq!(resp.status(), 400);
}

#[tokio::test]
async fn test_refresh_token_endpoint_missing_refresh_token() {
    // Given
    let mut mock_auth_service = MockAuthServiceImpl::new();
    let mut mock_user_registration_use_case = MockUserRegistrationUseCase::new();
    
    let auth_use_case = Arc::new(AuthUseCase::new(mock_auth_service));
    let user_registration_use_case = Arc::new(mock_user_registration_use_case);
    
    let app = test::init_service(
        App::new()
            .configure(|cfg| configure_routes(cfg, auth_use_case, user_registration_use_case))
    ).await;
    
    let request_body = json!({});
    
    // When
    let req = test::TestRequest::post()
        .uri("/api/auth/refresh")
        .set_json(&request_body)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // Then
    assert_eq!(resp.status(), 400);
}

#[tokio::test]
async fn test_refresh_token_endpoint_network_error() {
    // Given
    let mut mock_auth_service = MockAuthServiceImpl::new();
    let mut mock_user_registration_use_case = MockUserRegistrationUseCase::new();
    
    mock_auth_service
        .expect_refresh_token_with_keycloak()
        .times(1)
        .with(mockall::predicate::eq("test-refresh-token"))
        .returning(|_| Err(ServiceError::ExternalServiceError("Network error".to_string())));
    
    let auth_use_case = Arc::new(AuthUseCase::new(mock_auth_service));
    let user_registration_use_case = Arc::new(mock_user_registration_use_case);
    
    let app = test::init_service(
        App::new()
            .configure(|cfg| configure_routes(cfg, auth_use_case, user_registration_use_case))
    ).await;
    
    let request_body = RefreshTokenRequest {
        refresh_token: "test-refresh-token".to_string(),
    };
    
    // When
    let req = test::TestRequest::post()
        .uri("/api/auth/refresh")
        .set_json(&request_body)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // Then
    assert_eq!(resp.status(), 401);
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["error"].as_str().unwrap().contains("Token refresh failed"));
}

#[tokio::test]
async fn test_refresh_token_endpoint_wrong_method() {
    // Given
    let mut mock_auth_service = MockAuthServiceImpl::new();
    let mut mock_user_registration_use_case = MockUserRegistrationUseCase::new();
    
    let auth_use_case = Arc::new(AuthUseCase::new(mock_auth_service));
    let user_registration_use_case = Arc::new(mock_user_registration_use_case);
    
    let app = test::init_service(
        App::new()
            .configure(|cfg| configure_routes(cfg, auth_use_case, user_registration_use_case))
    ).await;
    
    // When
    let req = test::TestRequest::get()
        .uri("/api/auth/refresh")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // Then
    assert_eq!(resp.status(), 405); // Method Not Allowed
}
