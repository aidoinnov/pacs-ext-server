use std::sync::Arc;
use actix_web::{test, web, App};
use serde_json::json;
use mockito::mock;

use pacs_server::presentation::controllers::auth_controller::configure_routes;
use pacs_server::application::use_cases::auth_use_case::AuthUseCase;
use pacs_server::domain::services::auth_service::AuthServiceImpl;
use pacs_server::infrastructure::repositories::UserRepositoryImpl;
use pacs_server::infrastructure::auth::JwtService;
use pacs_server::infrastructure::external::KeycloakClient;
use pacs_server::infrastructure::config::{KeycloakConfig, JwtConfig};
use pacs_server::application::use_cases::user_registration_use_case::UserRegistrationUseCase;
use pacs_server::infrastructure::services::UserRegistrationServiceImpl;
use pacs_server::application::dto::auth_dto::RefreshTokenRequest;

#[tokio::test]
async fn test_refresh_token_integration_success() {
    // Given
    let mut mock_server = mockito::Server::new_async().await;
    let mock_url = mock_server.url();
    
    let keycloak_config = KeycloakConfig {
        url: mock_url,
        realm: "test-realm".to_string(),
        client_id: "test-client".to_string(),
        client_secret: "test-secret".to_string(),
        admin_username: "admin".to_string(),
        admin_password: "password".to_string(),
    };
    
    let jwt_config = JwtConfig {
        secret: "test-secret-key-for-jwt-token-generation".to_string(),
        expiration_hours: 24,
    };
    
    // Mock Keycloak response
    let keycloak_response = json!({
        "access_token": "new-access-token-from-keycloak",
        "refresh_token": "new-refresh-token-from-keycloak",
        "expires_in": 3600,
        "refresh_expires_in": 7200,
        "token_type": "Bearer"
    });
    
    let _mock = mock_server
        .mock("POST", "/realms/test-realm/protocol/openid-connect/token")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(keycloak_response.to_string())
        .create();
    
    // Create real services
    let keycloak_client = Arc::new(KeycloakClient::new(keycloak_config));
    let jwt_service = JwtService::new(&jwt_config);
    
    // Mock database (we'll use a simple in-memory setup for integration test)
    // In a real integration test, you'd use a test database
    let mock_user_repo = UserRepositoryImpl::new(sqlx::PgPool::connect("postgresql://test:test@localhost/test").await.unwrap());
    let auth_service = AuthServiceImpl::new(mock_user_repo, jwt_service, keycloak_client.clone());
    let auth_use_case = Arc::new(AuthUseCase::new(auth_service));
    
    // Mock user registration service
    let user_registration_service = UserRegistrationServiceImpl::new(
        sqlx::PgPool::connect("postgresql://test:test@localhost/test").await.unwrap(),
        (*keycloak_client).clone(),
    );
    let user_registration_use_case = Arc::new(UserRegistrationUseCase::new(user_registration_service));
    
    let app = test::init_service(
        App::new()
            .configure(|cfg| configure_routes(cfg, auth_use_case, user_registration_use_case))
    ).await;
    
    let request_body = RefreshTokenRequest {
        refresh_token: "valid-refresh-token".to_string(),
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
    assert_eq!(body["token"], "new-access-token-from-keycloak");
    assert_eq!(body["token_type"], "Bearer");
    assert_eq!(body["expires_in"], 3600);
}

#[tokio::test]
async fn test_refresh_token_integration_keycloak_error() {
    // Given
    let mut mock_server = mockito::Server::new_async().await;
    let mock_url = mock_server.url();
    
    let keycloak_config = KeycloakConfig {
        url: mock_url,
        realm: "test-realm".to_string(),
        client_id: "test-client".to_string(),
        client_secret: "test-secret".to_string(),
        admin_username: "admin".to_string(),
        admin_password: "password".to_string(),
    };
    
    let jwt_config = JwtConfig {
        secret: "test-secret-key-for-jwt-token-generation".to_string(),
        expiration_hours: 24,
    };
    
    // Mock Keycloak error response
    let error_response = json!({
        "error": "invalid_grant",
        "error_description": "Invalid refresh token"
    });
    
    let _mock = mock_server
        .mock("POST", "/realms/test-realm/protocol/openid-connect/token")
        .with_status(400)
        .with_header("content-type", "application/json")
        .with_body(error_response.to_string())
        .create();
    
    // Create real services
    let keycloak_client = Arc::new(KeycloakClient::new(keycloak_config));
    let jwt_service = JwtService::new(&jwt_config);
    
    // Mock database
    let mock_user_repo = UserRepositoryImpl::new(sqlx::PgPool::connect("postgresql://test:test@localhost/test").await.unwrap());
    let auth_service = AuthServiceImpl::new(mock_user_repo, jwt_service, keycloak_client.clone());
    let auth_use_case = Arc::new(AuthUseCase::new(auth_service));
    
    // Mock user registration service
    let user_registration_service = UserRegistrationServiceImpl::new(
        sqlx::PgPool::connect("postgresql://test:test@localhost/test").await.unwrap(),
        (*keycloak_client).clone(),
    );
    let user_registration_use_case = Arc::new(UserRegistrationUseCase::new(user_registration_service));
    
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
async fn test_refresh_token_integration_network_timeout() {
    // Given
    let keycloak_config = KeycloakConfig {
        url: "http://timeout-server".to_string(), // This will cause a timeout
        realm: "test-realm".to_string(),
        client_id: "test-client".to_string(),
        client_secret: "test-secret".to_string(),
        admin_username: "admin".to_string(),
        admin_password: "password".to_string(),
    };
    
    let jwt_config = JwtConfig {
        secret: "test-secret-key-for-jwt-token-generation".to_string(),
        expiration_hours: 24,
    };
    
    // Create real services
    let keycloak_client = Arc::new(KeycloakClient::new(keycloak_config));
    let jwt_service = JwtService::new(&jwt_config);
    
    // Mock database
    let mock_user_repo = UserRepositoryImpl::new(sqlx::PgPool::connect("postgresql://test:test@localhost/test").await.unwrap());
    let auth_service = AuthServiceImpl::new(mock_user_repo, jwt_service, keycloak_client.clone());
    let auth_use_case = Arc::new(AuthUseCase::new(auth_service));
    
    // Mock user registration service
    let user_registration_service = UserRegistrationServiceImpl::new(
        sqlx::PgPool::connect("postgresql://test:test@localhost/test").await.unwrap(),
        (*keycloak_client).clone(),
    );
    let user_registration_use_case = Arc::new(UserRegistrationUseCase::new(user_registration_service));
    
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
async fn test_refresh_token_integration_malformed_response() {
    // Given
    let mut mock_server = mockito::Server::new_async().await;
    let mock_url = mock_server.url();
    
    let keycloak_config = KeycloakConfig {
        url: mock_url,
        realm: "test-realm".to_string(),
        client_id: "test-client".to_string(),
        client_secret: "test-secret".to_string(),
        admin_username: "admin".to_string(),
        admin_password: "password".to_string(),
    };
    
    let jwt_config = JwtConfig {
        secret: "test-secret-key-for-jwt-token-generation".to_string(),
        expiration_hours: 24,
    };
    
    // Mock malformed response
    let _mock = mock_server
        .mock("POST", "/realms/test-realm/protocol/openid-connect/token")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body("invalid json response")
        .create();
    
    // Create real services
    let keycloak_client = Arc::new(KeycloakClient::new(keycloak_config));
    let jwt_service = JwtService::new(&jwt_config);
    
    // Mock database
    let mock_user_repo = UserRepositoryImpl::new(sqlx::PgPool::connect("postgresql://test:test@localhost/test").await.unwrap());
    let auth_service = AuthServiceImpl::new(mock_user_repo, jwt_service, keycloak_client.clone());
    let auth_use_case = Arc::new(AuthUseCase::new(auth_service));
    
    // Mock user registration service
    let user_registration_service = UserRegistrationServiceImpl::new(
        sqlx::PgPool::connect("postgresql://test:test@localhost/test").await.unwrap(),
        (*keycloak_client).clone(),
    );
    let user_registration_use_case = Arc::new(UserRegistrationUseCase::new(user_registration_service));
    
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
