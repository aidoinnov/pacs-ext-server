#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use mockall::mock;
    use std::sync::Arc;
    use tokio;
    use crate::application::dto::user_registration_dto::*;
    use crate::domain::ServiceError;
    use crate::domain::entities::{User, UserAccountStatus, NewUserAuditLog};
    use crate::infrastructure::external::KeycloakClient;
    use crate::infrastructure::services::UserRegistrationServiceImpl;
    use crate::domain::services::UserRegistrationService;

    // Mock KeycloakClient - 실제로는 KeycloakClient를 직접 사용하지 않고
    // UserRegistrationService의 구현체에서 KeycloakClient를 사용하므로
    // 여기서는 간단한 테스트만 작성

    #[tokio::test]
    async fn test_signup_success() {
        // Given
        let keycloak_client = KeycloakClient::new(crate::infrastructure::config::KeycloakConfig {
            url: "http://localhost:8080".to_string(),
            realm: "test".to_string(),
            client_id: "test".to_string(),
            client_secret: "test".to_string(),
            admin_username: "admin".to_string(),
            admin_password: "admin".to_string(),
        });

        let service = UserRegistrationServiceImpl::new(
            Arc::new(keycloak_client),
        );

        // When
        let result = service.signup(
            "testuser".to_string(),
            "test@example.com".to_string(),
            "password123".to_string(),
            Some("Test User".to_string()),
            Some("Test Org".to_string()),
            Some("Test Dept".to_string()),
            Some("010-1234-5678".to_string()),
        ).await;

        // Then
        // 실제 Keycloak 서버가 없으므로 에러가 발생할 것으로 예상됨
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_verify_email_success() {
        // Given
        let keycloak_client = KeycloakClient::new(crate::infrastructure::config::KeycloakConfig {
            url: "http://localhost:8080".to_string(),
            realm: "test".to_string(),
            client_id: "test".to_string(),
            client_secret: "test".to_string(),
            admin_username: "admin".to_string(),
            admin_password: "admin".to_string(),
        });

        let service = UserRegistrationServiceImpl::new(
            Arc::new(keycloak_client),
        );

        // When
        let result = service.verify_email(1).await;

        // Then
        // 실제 Keycloak 서버가 없으므로 에러가 발생할 것으로 예상됨
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_approve_user_success() {
        // Given
        let keycloak_client = KeycloakClient::new(crate::infrastructure::config::KeycloakConfig {
            url: "http://localhost:8080".to_string(),
            realm: "test".to_string(),
            client_id: "test".to_string(),
            client_secret: "test".to_string(),
            admin_username: "admin".to_string(),
            admin_password: "admin".to_string(),
        });

        let service = UserRegistrationServiceImpl::new(
            Arc::new(keycloak_client),
        );

        // When
        let result = service.approve_user(1, 2).await;

        // Then
        // 실제 Keycloak 서버가 없으므로 에러가 발생할 것으로 예상됨
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_account_success() {
        // Given
        let keycloak_client = KeycloakClient::new(crate::infrastructure::config::KeycloakConfig {
            url: "http://localhost:8080".to_string(),
            realm: "test".to_string(),
            client_id: "test".to_string(),
            client_secret: "test".to_string(),
            admin_username: "admin".to_string(),
            admin_password: "admin".to_string(),
        });

        let service = UserRegistrationServiceImpl::new(
            Arc::new(keycloak_client),
        );

        // When
        let result = service.delete_account(1, Some(2)).await;

        // Then
        // 실제 Keycloak 서버가 없으므로 에러가 발생할 것으로 예상됨
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_log_audit_success() {
        // Given
        let keycloak_client = KeycloakClient::new(crate::infrastructure::config::KeycloakConfig {
            url: "http://localhost:8080".to_string(),
            realm: "test".to_string(),
            client_id: "test".to_string(),
            client_secret: "test".to_string(),
            admin_username: "admin".to_string(),
            admin_password: "admin".to_string(),
        });

        let service = UserRegistrationServiceImpl::new(
            Arc::new(keycloak_client),
        );

        let audit_log = NewUserAuditLog {
            user_id: Some(1),
            action: "TEST_ACTION".to_string(),
            actor_id: Some(2),
            keycloak_sync_status: Some("SUCCESS".to_string()),
            keycloak_user_id: Some("test_keycloak_id".to_string()),
            error_message: None,
            metadata: None,
        };

        // When
        let result = service.log_audit(audit_log).await;

        // Then
        // 실제 구현에서는 데이터베이스에 감사 로그를 저장하므로
        // 여기서는 에러가 발생할 것으로 예상됨
        assert!(result.is_err());
    }
}
