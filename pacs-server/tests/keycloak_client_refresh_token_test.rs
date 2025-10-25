use std::sync::Arc;
use mockito::mock;
use serde_json::json;
use tokio;

use pacs_server::infrastructure::external::KeycloakClient;
use pacs_server::infrastructure::config::KeycloakConfig;
use pacs_server::domain::ServiceError;

#[tokio::test]
async fn test_refresh_access_token_success() {
    // Given
    let mut mock_server = mockito::Server::new_async().await;
    let mock_url = mock_server.url();
    
    let config = KeycloakConfig {
        url: mock_url,
        realm: "test-realm".to_string(),
        client_id: "test-client".to_string(),
        client_secret: "test-secret".to_string(),
        admin_username: "admin".to_string(),
        admin_password: "password".to_string(),
    };
    
    let client = KeycloakClient::new(config);
    
    let refresh_token = "test-refresh-token";
    let expected_response = json!({
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
        .with_body(expected_response.to_string())
        .create_async()
        .await;
    
    // When
    let result = client.refresh_access_token(refresh_token).await;
    
    // Then
    assert!(result.is_ok());
    let token_response = result.unwrap();
    assert_eq!(token_response.access_token, "new-access-token");
    assert_eq!(token_response.refresh_token, "new-refresh-token");
    assert_eq!(token_response.expires_in, 3600);
    assert_eq!(token_response.refresh_expires_in, 7200);
    assert_eq!(token_response.token_type, "Bearer");
}

#[tokio::test]
async fn test_refresh_access_token_invalid_token() {
    // Given
    let mut mock_server = mockito::Server::new_async().await;
    let mock_url = mock_server.url();
    
    let config = KeycloakConfig {
        url: mock_url,
        realm: "test-realm".to_string(),
        client_id: "test-client".to_string(),
        client_secret: "test-secret".to_string(),
        admin_username: "admin".to_string(),
        admin_password: "password".to_string(),
    };
    
    let client = KeycloakClient::new(config);
    
    let invalid_refresh_token = "invalid-refresh-token";
    let error_response = json!({
        "error": "invalid_grant",
        "error_description": "Invalid refresh token"
    });
    
    let _mock = mock_server
        .mock("POST", "/realms/test-realm/protocol/openid-connect/token")
        .with_status(400)
        .with_header("content-type", "application/json")
        .with_body(error_response.to_string())
        .create_async()
        .await;
    
    // When
    let result = client.refresh_access_token(invalid_refresh_token).await;
    
    // Then
    assert!(result.is_err());
    match result.unwrap_err() {
        ServiceError::ExternalServiceError(msg) => {
            assert!(msg.contains("Refresh token failed"));
        }
        _ => panic!("Expected ExternalServiceError"),
    }
}

#[tokio::test]
async fn test_refresh_access_token_network_error() {
    // Given
    let config = KeycloakConfig {
        url: "http://invalid-url".to_string(),
        realm: "test-realm".to_string(),
        client_id: "test-client".to_string(),
        client_secret: "test-secret".to_string(),
        admin_username: "admin".to_string(),
        admin_password: "password".to_string(),
    };
    
    let client = KeycloakClient::new(config);
    let refresh_token = "test-refresh-token";
    
    // When
    let result = client.refresh_access_token(refresh_token).await;
    
    // Then
    assert!(result.is_err());
    match result.unwrap_err() {
        ServiceError::ExternalServiceError(msg) => {
            assert!(msg.contains("Refresh token request failed"));
        }
        _ => panic!("Expected ExternalServiceError"),
    }
}

#[tokio::test]
async fn test_refresh_access_token_malformed_response() {
    // Given
    let mut mock_server = mockito::Server::new_async().await;
    let mock_url = mock_server.url();
    
    let config = KeycloakConfig {
        url: mock_url,
        realm: "test-realm".to_string(),
        client_id: "test-client".to_string(),
        client_secret: "test-secret".to_string(),
        admin_username: "admin".to_string(),
        admin_password: "password".to_string(),
    };
    
    let client = KeycloakClient::new(config);
    let refresh_token = "test-refresh-token";
    
    let _mock = mock_server
        .mock("POST", "/realms/test-realm/protocol/openid-connect/token")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body("invalid json")
        .create_async()
        .await;
    
    // When
    let result = client.refresh_access_token(refresh_token).await;
    
    // Then
    assert!(result.is_err());
    match result.unwrap_err() {
        ServiceError::ExternalServiceError(msg) => {
            assert!(msg.contains("Failed to parse refresh token response"));
        }
        _ => panic!("Expected ExternalServiceError"),
    }
}

#[tokio::test]
async fn test_refresh_access_token_request_parameters() {
    // Given
    let mut mock_server = mockito::Server::new_async().await;
    let mock_url = mock_server.url();
    
    let config = KeycloakConfig {
        url: mock_url,
        realm: "test-realm".to_string(),
        client_id: "test-client".to_string(),
        client_secret: "test-secret".to_string(),
        admin_username: "admin".to_string(),
        admin_password: "password".to_string(),
    };
    
    let client = KeycloakClient::new(config);
    let refresh_token = "test-refresh-token";
    
    let expected_response = json!({
        "access_token": "new-access-token",
        "refresh_token": "new-refresh-token",
        "expires_in": 3600,
        "refresh_expires_in": 7200,
        "token_type": "Bearer"
    });
    
    let _mock = mock_server
        .mock("POST", "/realms/test-realm/protocol/openid-connect/token")
        .match_header("content-type", "application/x-www-form-urlencoded")
        .match_body("grant_type=refresh_token&refresh_token=test-refresh-token&client_id=test-client")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(expected_response.to_string())
        .create_async()
        .await;
    
    // When
    let result = client.refresh_access_token(refresh_token).await;
    
    // Then
    assert!(result.is_ok());
}
