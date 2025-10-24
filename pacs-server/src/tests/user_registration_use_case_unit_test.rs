#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use mockall::mock;
    use std::sync::Arc;
    use tokio;
    use crate::application::dto::user_registration_dto::*;
    use crate::application::use_cases::UserRegistrationUseCase;
    use crate::domain::ServiceError;
    use crate::domain::services::UserRegistrationService;
    use crate::domain::entities::{User, UserAccountStatus, NewUserAuditLog};

    // Mock UserRegistrationService
    mock! {
        UserRegistrationService {}

        #[async_trait]
        impl UserRegistrationService for UserRegistrationService {
            async fn signup(&self, username: String, email: String, password: String, full_name: Option<String>, organization: Option<String>, department: Option<String>, phone: Option<String>) -> Result<User, ServiceError>;
            async fn verify_email(&self, user_id: i32) -> Result<(), ServiceError>;
            async fn approve_user(&self, user_id: i32, admin_id: i32) -> Result<(), ServiceError>;
            async fn delete_account(&self, user_id: i32, actor_id: Option<i32>) -> Result<(), ServiceError>;
            async fn log_audit(&self, log: NewUserAuditLog) -> Result<(), ServiceError>;
        }
    }

    #[tokio::test]
    async fn test_signup_success() {
        // Given
        let mut mock_service = MockUserRegistrationService::new();
        let expected_user = User {
            id: 1,
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            account_status: UserAccountStatus::PendingEmail,
            email_verified: false,
            email_verification_token: None,
            email_verification_expires_at: None,
            approved_by: None,
            approved_at: None,
            suspended_at: None,
            suspended_reason: None,
            deleted_at: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        mock_service
            .expect_signup()
            .times(1)
            .returning(move |_, _, _, _, _, _, _| Ok(expected_user.clone()));

        let use_case = UserRegistrationUseCase::new(Arc::new(mock_service));

        let request = SignupRequest {
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            full_name: Some("Test User".to_string()),
            organization: None,
            department: None,
            phone: None,
        };

        // When
        let result = use_case.signup(request).await;

        // Then
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.username, "testuser");
        assert_eq!(response.email, "test@example.com");
        assert_eq!(response.account_status, UserAccountStatus::PendingEmail);
    }

    #[tokio::test]
    async fn test_signup_service_error() {
        // Given
        let mut mock_service = MockUserRegistrationService::new();
        mock_service
            .expect_signup()
            .times(1)
            .returning(|_, _, _, _, _, _, _| Err(ServiceError::ValidationError("Invalid email".to_string())));

        let use_case = UserRegistrationUseCase::new(Arc::new(mock_service));

        let request = SignupRequest {
            username: "testuser".to_string(),
            email: "invalid-email".to_string(),
            password: "password123".to_string(),
            full_name: None,
            organization: None,
            department: None,
            phone: None,
        };

        // When
        let result = use_case.signup(request).await;

        // Then
        assert!(result.is_err());
        match result.unwrap_err() {
            ServiceError::ValidationError(msg) => {
                assert_eq!(msg, "Invalid email");
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[tokio::test]
    async fn test_verify_email_success() {
        // Given
        let mut mock_service = MockUserRegistrationService::new();
        mock_service
            .expect_verify_email()
            .times(1)
            .returning(|_| Ok(()));

        let use_case = UserRegistrationUseCase::new(Arc::new(mock_service));

        // When
        let result = use_case.verify_email(1).await;

        // Then
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_approve_user_success() {
        // Given
        let mut mock_service = MockUserRegistrationService::new();
        mock_service
            .expect_approve_user()
            .times(1)
            .returning(|_, _| Ok(()));

        let use_case = UserRegistrationUseCase::new(Arc::new(mock_service));

        // When
        let result = use_case.approve_user(1, 2).await;

        // Then
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_account_success() {
        // Given
        let mut mock_service = MockUserRegistrationService::new();
        mock_service
            .expect_delete_account()
            .times(1)
            .returning(|_, _| Ok(()));

        let use_case = UserRegistrationUseCase::new(Arc::new(mock_service));

        // When
        let result = use_case.delete_account(1, Some(2)).await;

        // Then
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_log_audit_success() {
        // Given
        let mut mock_service = MockUserRegistrationService::new();
        mock_service
            .expect_log_audit()
            .times(1)
            .returning(|_| Ok(()));

        let use_case = UserRegistrationUseCase::new(Arc::new(mock_service));

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
        let result = use_case.log_audit(audit_log).await;

        // Then
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_log_audit_error() {
        // Given
        let mut mock_service = MockUserRegistrationService::new();
        mock_service
            .expect_log_audit()
            .times(1)
            .returning(|_| Err(ServiceError::DatabaseError("Database error".to_string())));

        let use_case = UserRegistrationUseCase::new(Arc::new(mock_service));

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
        let result = use_case.log_audit(audit_log).await;

        // Then
        assert!(result.is_err());
        match result.unwrap_err() {
            ServiceError::DatabaseError(msg) => {
                assert_eq!(msg, "Database error");
            }
            _ => panic!("Expected DatabaseError"),
        }
    }
}
