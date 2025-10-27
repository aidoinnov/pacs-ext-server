use std::sync::Arc;
use uuid::Uuid;
use chrono::{NaiveDateTime, Utc};

use pacs_server::application::dto::auth_dto::{LoginRequest, LoginResponse, VerifyTokenResponse, RefreshTokenResponse};
use pacs_server::application::use_cases::AuthUseCase;
use pacs_server::domain::entities::User;
use pacs_server::domain::services::AuthService;
use pacs_server::domain::ServiceError;

// Mock AuthService for testing
#[derive(Clone)]
struct MockAuthService {
    users: std::collections::HashMap<Uuid, User>,
    tokens: std::collections::HashMap<String, User>,
}

impl MockAuthService {
    fn new() -> Self {
        Self {
            users: std::collections::HashMap::new(),
            tokens: std::collections::HashMap::new(),
        }
    }

    fn add_user(&mut self, user: User) {
        self.users.insert(user.keycloak_id, user);
    }

    fn add_token(&mut self, token: String, user: User) {
        self.tokens.insert(token, user);
    }
}

#[async_trait::async_trait]
impl AuthService for MockAuthService {
    async fn login(&self, keycloak_id: Uuid, username: String, email: String) -> Result<pacs_server::domain::services::AuthResponse, ServiceError> {
        let user = self.users.get(&keycloak_id)
            .ok_or(ServiceError::NotFound("User not found".into()))?;
        
        let token = format!("token_{}_{}", keycloak_id, username);
        
        Ok(pacs_server::domain::services::AuthResponse {
            user: user.clone(),
            token,
        })
    }

    async fn verify_and_get_user(&self, token: &str) -> Result<User, ServiceError> {
        self.tokens.get(token)
            .ok_or(ServiceError::Unauthorized("Invalid token".into()))
            .map(|u| u.clone())
    }

    async fn refresh_token(&self, user: &User) -> Result<String, ServiceError> {
        Ok(format!("refreshed_token_{}_{}", user.keycloak_id, user.username))
    }

    async fn logout(&self, _token: &str) -> Result<(), ServiceError> {
        Ok(())
    }

    async fn refresh_token_with_keycloak(&self, _refresh_token: &str) -> Result<RefreshTokenResponse, ServiceError> {
        Err(ServiceError::ValidationError("Not implemented in mock".into()))
    }

    async fn reset_user_password(&self, _keycloak_user_id: &str, _new_password: &str) -> Result<(), ServiceError> {
        Ok(())
    }

    async fn find_username_by_email(&self, _email: &str) -> Result<User, ServiceError> {
        Err(ServiceError::NotFound("User not found".into()))
    }

    async fn reset_password_by_credentials(&self, _username: &str, _email: &str, _new_password: &str) -> Result<(), ServiceError> {
        Ok(())
    }
}

fn create_test_user() -> User {
    User {
        id: 1,
        keycloak_id: Uuid::new_v4(),
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        full_name: None,
        organization: None,
        department: None,
        phone: None,
        created_at: Utc::now(),
        updated_at: None,
        account_status: pacs_server::domain::entities::UserAccountStatus::Active,
        email_verified: true,
        email_verification_token: None,
        email_verification_expires_at: None,
        approved_by: None,
        approved_at: None,
        suspended_at: None,
        suspended_reason: None,
        deleted_at: None,
    }
}

#[tokio::test]
async fn test_auth_use_case_login_success() {
    let mut mock_auth_service = MockAuthService::new();
    let user = create_test_user();
    mock_auth_service.add_user(user.clone());
    
    let auth_use_case = AuthUseCase::new(mock_auth_service);

    let login_request = LoginRequest {
        keycloak_id: user.keycloak_id,
        username: user.username.clone(),
        email: user.email.clone(),
    };

    let result = auth_use_case.login(login_request).await;
    assert!(result.is_ok());

    let login_response = result.unwrap();
    assert_eq!(login_response.user_id, user.id);
    assert_eq!(login_response.keycloak_id, user.keycloak_id);
    assert_eq!(login_response.username, user.username);
    assert_eq!(login_response.email, user.email);
    assert_eq!(login_response.token_type, "Bearer");
    assert_eq!(login_response.expires_in, 24 * 60 * 60);
    assert!(!login_response.token.is_empty());
}

#[tokio::test]
async fn test_auth_use_case_login_user_not_found() {
    let mock_auth_service = MockAuthService::new();
    let auth_use_case = AuthUseCase::new(mock_auth_service);

    let login_request = LoginRequest {
        keycloak_id: Uuid::new_v4(),
        username: "nonexistent".to_string(),
        email: "nonexistent@example.com".to_string(),
    };

    let result = auth_use_case.login(login_request).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_auth_use_case_verify_token_success() {
    let mut mock_auth_service = MockAuthService::new();
    let user = create_test_user();
    let token = "valid_token_123";
    mock_auth_service.add_token(token.to_string(), user.clone());
    
    let auth_use_case = AuthUseCase::new(mock_auth_service);

    let result = auth_use_case.verify_token(token).await;
    assert!(result.is_ok());

    let verify_response = result.unwrap();
    assert_eq!(verify_response.user_id, user.id);
    assert_eq!(verify_response.keycloak_id, user.keycloak_id);
    assert_eq!(verify_response.username, user.username);
    assert_eq!(verify_response.email, user.email);
    assert!(verify_response.is_valid);
}

#[tokio::test]
async fn test_auth_use_case_verify_token_invalid() {
    let mock_auth_service = MockAuthService::new();
    let auth_use_case = AuthUseCase::new(mock_auth_service);

    let result = auth_use_case.verify_token("invalid_token").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_auth_use_case_refresh_token_success() {
    let mut mock_auth_service = MockAuthService::new();
    let user = create_test_user();
    let token = "valid_token_123";
    mock_auth_service.add_token(token.to_string(), user.clone());
    
    let auth_use_case = AuthUseCase::new(mock_auth_service);

    let result = auth_use_case.refresh_token(token).await;
    assert!(result.is_ok());

    let refresh_response = result.unwrap();
    assert_eq!(refresh_response.token_type, "Bearer");
    assert_eq!(refresh_response.expires_in, 24 * 60 * 60);
    assert!(!refresh_response.token.is_empty());
    assert!(refresh_response.token.starts_with("refreshed_token_"));
}

#[tokio::test]
async fn test_auth_use_case_refresh_token_invalid() {
    let mock_auth_service = MockAuthService::new();
    let auth_use_case = AuthUseCase::new(mock_auth_service);

    let result = auth_use_case.refresh_token("invalid_token").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_auth_use_case_logout_success() {
    let mock_auth_service = MockAuthService::new();
    let auth_use_case = AuthUseCase::new(mock_auth_service);

    let result = auth_use_case.logout("any_token").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_login_response_structure() {
    let mut mock_auth_service = MockAuthService::new();
    let user = create_test_user();
    mock_auth_service.add_user(user.clone());
    
    let auth_use_case = AuthUseCase::new(mock_auth_service);

    let login_request = LoginRequest {
        keycloak_id: user.keycloak_id,
        username: user.username.clone(),
        email: user.email.clone(),
    };

    let login_response = auth_use_case.login(login_request).await.unwrap();
    
    // Verify all required fields are present
    assert!(login_response.user_id > 0);
    assert!(!login_response.keycloak_id.is_nil());
    assert!(!login_response.username.is_empty());
    assert!(!login_response.email.is_empty());
    assert!(!login_response.token.is_empty());
    assert_eq!(login_response.token_type, "Bearer");
    assert!(login_response.expires_in > 0);
}

#[tokio::test]
async fn test_verify_token_response_structure() {
    let mut mock_auth_service = MockAuthService::new();
    let user = create_test_user();
    let token = "valid_token_123";
    mock_auth_service.add_token(token.to_string(), user.clone());
    
    let auth_use_case = AuthUseCase::new(mock_auth_service);

    let verify_response = auth_use_case.verify_token(token).await.unwrap();
    
    // Verify all required fields are present
    assert!(verify_response.user_id > 0);
    assert!(!verify_response.keycloak_id.is_nil());
    assert!(!verify_response.username.is_empty());
    assert!(!verify_response.email.is_empty());
    assert!(verify_response.is_valid);
}

#[tokio::test]
async fn test_refresh_token_response_structure() {
    let mut mock_auth_service = MockAuthService::new();
    let user = create_test_user();
    let token = "valid_token_123";
    mock_auth_service.add_token(token.to_string(), user.clone());
    
    let auth_use_case = AuthUseCase::new(mock_auth_service);

    let refresh_response = auth_use_case.refresh_token(token).await.unwrap();
    
    // Verify all required fields are present
    assert!(!refresh_response.token.is_empty());
    assert_eq!(refresh_response.token_type, "Bearer");
    assert!(refresh_response.expires_in > 0);
}
