use std::sync::Arc;
use std::time::Instant;
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
async fn test_refresh_token_performance_single_request() {
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
    
    // Mock Keycloak response with fast response time
    let keycloak_response = json!({
        "access_token": "new-access-token",
        "refresh_token": "new-refresh-token",
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
    let start = Instant::now();
    
    let req = test::TestRequest::post()
        .uri("/api/auth/refresh")
        .set_json(&request_body)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    let duration = start.elapsed();
    
    // Then
    assert_eq!(resp.status(), 200);
    
    // Performance assertion: should complete within 1 second
    assert!(duration.as_millis() < 1000, "Request took too long: {:?}", duration);
    
    println!("Single request duration: {:?}", duration);
}

#[tokio::test]
async fn test_refresh_token_performance_concurrent_requests() {
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
        "access_token": "new-access-token",
        "refresh_token": "new-refresh-token",
        "expires_in": 3600,
        "refresh_expires_in": 7200,
        "token_type": "Bearer"
    });
    
    let _mock = mock_server
        .mock("POST", "/realms/test-realm/protocol/openid-connect/token")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(keycloak_response.to_string())
        .expect(10) // Expect 10 concurrent requests
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
    
    // When
    let start = Instant::now();
    
    let mut handles = vec![];
    for i in 0..10 {
        // Note: App cloning is not supported in actix-web, using sequential requests instead
        let handle = tokio::spawn(async move {
            let request_body = RefreshTokenRequest {
                refresh_token: format!("test-refresh-token-{}", i),
            };
            
            let req = test::TestRequest::post()
                .uri("/api/auth/refresh")
                .set_json(&request_body)
                .to_request();
            
            test::call_service(&app, req).await
        });
        handles.push(handle);
    }
    
    let mut success_count = 0;
    for handle in handles {
        let resp = handle.await.unwrap();
        if resp.status() == 200 {
            success_count += 1;
        }
    }
    
    let duration = start.elapsed();
    
    // Then
    assert_eq!(success_count, 10, "Not all concurrent requests succeeded");
    
    // Performance assertion: 10 concurrent requests should complete within 2 seconds
    assert!(duration.as_millis() < 2000, "Concurrent requests took too long: {:?}", duration);
    
    println!("10 concurrent requests duration: {:?}", duration);
    println!("Average time per request: {:?}", duration / 10);
}

#[tokio::test]
async fn test_refresh_token_performance_high_load() {
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
        "access_token": "new-access-token",
        "refresh_token": "new-refresh-token",
        "expires_in": 3600,
        "refresh_expires_in": 7200,
        "token_type": "Bearer"
    });
    
    let _mock = mock_server
        .mock("POST", "/realms/test-realm/protocol/openid-connect/token")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(keycloak_response.to_string())
        .expect(100) // Expect 100 requests
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
    
    // When
    let start = Instant::now();
    
    let mut handles = vec![];
    for i in 0..100 {
        // Note: App cloning is not supported in actix-web, using sequential requests instead
        let handle = tokio::spawn(async move {
            let request_body = RefreshTokenRequest {
                refresh_token: format!("test-refresh-token-{}", i),
            };
            
            let req = test::TestRequest::post()
                .uri("/api/auth/refresh")
                .set_json(&request_body)
                .to_request();
            
            test::call_service(&app, req).await
        });
        handles.push(handle);
    }
    
    let mut success_count = 0;
    let mut error_count = 0;
    for handle in handles {
        let resp = handle.await.unwrap();
        if resp.status() == 200 {
            success_count += 1;
        } else {
            error_count += 1;
        }
    }
    
    let duration = start.elapsed();
    
    // Then
    println!("High load test results:");
    println!("  Total requests: 100");
    println!("  Successful: {}", success_count);
    println!("  Errors: {}", error_count);
    println!("  Total duration: {:?}", duration);
    println!("  Requests per second: {:.2}", 100.0 / duration.as_secs_f64());
    
    // Performance assertion: 100 requests should complete within 10 seconds
    assert!(duration.as_secs() < 10, "High load test took too long: {:?}", duration);
    
    // At least 90% should succeed
    assert!(success_count >= 90, "Too many requests failed: {} errors out of 100", error_count);
}
