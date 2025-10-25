#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;
    use std::sync::Arc;
    use tokio;
    use crate::application::dto::user_registration_dto::*;
    use crate::application::use_cases::UserRegistrationUseCase;
    use crate::domain::ServiceError;
    use crate::domain::services::UserRegistrationService;

    // Mock UserRegistrationService
    mock! {
        UserRegistrationService {}

        #[async_trait]
        impl UserRegistrationService for UserRegistrationService {
            async fn signup(&self, request: SignupRequest) -> Result<UserStatusResponse, ServiceError>;
            async fn verify_email(&self, user_id: i32) -> Result<(), ServiceError>;
            async fn approve_user(&self, user_id: i32, admin_id: i32) -> Result<(), ServiceError>;
            async fn delete_account(&self, user_id: i32, actor_id: Option<i32>) -> Result<(), ServiceError>;
            async fn get_user_status(&self, user_id: i32) -> Result<UserStatusResponse, ServiceError>;
        }
    }

    #[tokio::test]
    async fn test_signup_success() {
        // Given
        let mut mock_service = MockUserRegistrationService::new();
        let expected_response = UserStatusResponse {
            user_id: 1,
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            account_status: UserAccountStatus::PendingEmail,
            email_verified: false,
            approved_at: None,
            suspended_at: None,
            deleted_at: None,
        };

        mock_service
            .expect_signup()
            .times(1)
            .returning(move |_| Ok(expected_response.clone()));

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
            .returning(|_| Err(ServiceError::ValidationError("Invalid email".to_string())));

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
    async fn test_get_user_status_success() {
        // Given
        let mut mock_service = MockUserRegistrationService::new();
        let expected_response = UserStatusResponse {
            user_id: 1,
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            account_status: UserAccountStatus::Active,
            email_verified: true,
            approved_at: Some(chrono::Utc::now()),
            suspended_at: None,
            deleted_at: None,
        };

        mock_service
            .expect_get_user_status()
            .times(1)
            .returning(move |_| Ok(expected_response.clone()));

        let use_case = UserRegistrationUseCase::new(Arc::new(mock_service));

        // When
        let result = use_case.get_user_status(1).await;

        // Then
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.user_id, 1);
        assert_eq!(response.account_status, UserAccountStatus::Active);
        assert!(response.email_verified);
    }

    #[tokio::test]
    async fn test_get_user_status_not_found() {
        // Given
        let mut mock_service = MockUserRegistrationService::new();
        mock_service
            .expect_get_user_status()
            .times(1)
            .returning(|_| Err(ServiceError::NotFound("User not found".to_string())));

        let use_case = UserRegistrationUseCase::new(Arc::new(mock_service));

        // When
        let result = use_case.get_user_status(999).await;

        // Then
        assert!(result.is_err());
        match result.unwrap_err() {
            ServiceError::NotFound(msg) => {
                assert_eq!(msg, "User not found");
            }
            _ => panic!("Expected NotFound error"),
        }
    }
}
