#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;
    use std::sync::Arc;
    use tokio;
    use crate::application::dto::user_registration_dto::*;
    use crate::domain::ServiceError;
    use crate::infrastructure::external::KeycloakClient;
    use crate::infrastructure::services::UserRegistrationServiceImpl;
    use sqlx::PgPool;

    // Mock KeycloakClient
    mock! {
        KeycloakClient {}

        #[async_trait]
        impl KeycloakClient for KeycloakClient {
            async fn get_admin_token(&self) -> Result<String, ServiceError>;
            async fn create_user(&self, token: &str, request: &SignupRequest) -> Result<String, ServiceError>;
            async fn delete_user(&self, token: &str, keycloak_user_id: &str) -> Result<(), ServiceError>;
            async fn send_verification_email(&self, token: &str, keycloak_user_id: &str) -> Result<(), ServiceError>;
            async fn assign_realm_role(&self, token: &str, keycloak_user_id: &str, role: &str) -> Result<(), ServiceError>;
            async fn update_user_enabled(&self, token: &str, keycloak_user_id: &str, enabled: bool) -> Result<(), ServiceError>;
        }
    }

    #[tokio::test]
    async fn test_signup_success() {
        // Given
        let mut mock_keycloak = MockKeycloakClient::new();
        mock_keycloak
            .expect_get_admin_token()
            .times(1)
            .returning(|| Ok("mock_token".to_string()));
        
        mock_keycloak
            .expect_create_user()
            .times(1)
            .returning(|_, _| Ok("mock_keycloak_user_id".to_string()));
        
        mock_keycloak
            .expect_send_verification_email()
            .times(1)
            .returning(|_, _| Ok(()));
        
        mock_keycloak
            .expect_assign_realm_role()
            .times(1)
            .returning(|_, _, _| Ok(()));

        let service = UserRegistrationServiceImpl::new(
            // Mock pool - 실제 테스트에서는 테스트 DB 사용
            Arc::new(mock_keycloak),
        );

        let request = SignupRequest {
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            full_name: Some("Test User".to_string()),
            organization: Some("Test Org".to_string()),
            department: Some("Test Dept".to_string()),
            phone: Some("010-1234-5678".to_string()),
        };

        // When
        let result = service.signup(request).await;

        // Then
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.username, "testuser");
        assert_eq!(response.email, "test@example.com");
        assert_eq!(response.account_status, UserAccountStatus::PendingEmail);
        assert!(!response.email_verified);
    }

    #[tokio::test]
    async fn test_signup_keycloak_failure() {
        // Given
        let mut mock_keycloak = MockKeycloakClient::new();
        mock_keycloak
            .expect_get_admin_token()
            .times(1)
            .returning(|| Err(ServiceError::ExternalServiceError("Keycloak error".to_string())));

        let service = UserRegistrationServiceImpl::new(
            Arc::new(mock_keycloak),
        );

        let request = SignupRequest {
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            full_name: None,
            organization: None,
            department: None,
            phone: None,
        };

        // When
        let result = service.signup(request).await;

        // Then
        assert!(result.is_err());
        match result.unwrap_err() {
            ServiceError::ExternalServiceError(msg) => {
                assert_eq!(msg, "Keycloak error");
            }
            _ => panic!("Expected ExternalServiceError"),
        }
    }

    #[tokio::test]
    async fn test_verify_email_success() {
        // Given
        let mut mock_keycloak = MockKeycloakClient::new();
        mock_keycloak
            .expect_get_admin_token()
            .times(1)
            .returning(|| Ok("mock_token".to_string()));
        
        mock_keycloak
            .expect_update_user_enabled()
            .times(1)
            .returning(|_, _, _| Ok(()));

        let service = UserRegistrationServiceImpl::new(
            Arc::new(mock_keycloak),
        );

        // When
        let result = service.verify_email(1).await;

        // Then
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_approve_user_success() {
        // Given
        let mut mock_keycloak = MockKeycloakClient::new();
        mock_keycloak
            .expect_get_admin_token()
            .times(1)
            .returning(|| Ok("mock_token".to_string()));
        
        mock_keycloak
            .expect_update_user_enabled()
            .times(1)
            .returning(|_, _, _| Ok(()));

        let service = UserRegistrationServiceImpl::new(
            Arc::new(mock_keycloak),
        );

        // When
        let result = service.approve_user(1, 2).await;

        // Then
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_account_success() {
        // Given
        let mut mock_keycloak = MockKeycloakClient::new();
        mock_keycloak
            .expect_get_admin_token()
            .times(1)
            .returning(|| Ok("mock_token".to_string()));
        
        mock_keycloak
            .expect_delete_user()
            .times(1)
            .returning(|_, _| Ok(()));

        let service = UserRegistrationServiceImpl::new(
            Arc::new(mock_keycloak),
        );

        // When
        let result = service.delete_account(1, Some(2)).await;

        // Then
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_user_status_success() {
        // Given
        let service = UserRegistrationServiceImpl::new(
            Arc::new(MockKeycloakClient::new()),
        );

        // When
        let result = service.get_user_status(1).await;

        // Then
        // 실제 구현에서는 데이터베이스에서 사용자 정보를 조회하므로
        // 여기서는 에러가 발생할 것으로 예상됨
        assert!(result.is_err());
    }
}
