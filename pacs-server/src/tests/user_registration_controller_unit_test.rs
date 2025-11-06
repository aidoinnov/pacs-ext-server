#[cfg(test)]
mod tests {
    use crate::application::dto::user_registration_dto::*;
    use crate::application::use_cases::UserRegistrationUseCase;
    use crate::domain::entities::{NewUserAuditLog, User, UserAccountStatus};
    use crate::domain::services::UserRegistrationService;
    use crate::domain::ServiceError;
    use actix_web::{test, web, App};
    use async_trait::async_trait;
    use serde_json::json;
    use std::sync::Arc;
    use tokio;

    // Mock UserRegistrationService for controller tests
    use mockall::mock;
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
    async fn test_signup_endpoint_success() {
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
            updated_at: Some(chrono::Utc::now()),
            full_name: None,
            organization: None,
            department: None,
            phone: None,
            keycloak_id: uuid::Uuid::new_v4(),
        };

        mock_service
            .expect_signup()
            .times(1)
            .returning(move |_, _, _, _, _, _, _| Ok(expected_user.clone()));

        let use_case = Arc::new(UserRegistrationUseCase::new(mock_service));
        let app = test::init_service(App::new().service(web::scope("/api").configure(|cfg| {
            crate::presentation::controllers::user_registration_controller::configure_routes(
                cfg,
                use_case.clone(),
            )
        })))
        .await;

        let request_body = json!({
            "username": "testuser",
            "email": "test@example.com",
            "password": "Password123",
            "full_name": "Test User",
            "organization": "Test Org",
            "department": "Test Dept",
            "phone": "010-1234-5678"
        });

        // When
        let req = test::TestRequest::post()
            .uri("/api/auth/signup")
            .set_json(&request_body)
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Then
        assert_eq!(resp.status(), 201);
    }

    #[tokio::test]
    async fn test_verify_email_endpoint_success() {
        // Given
        let mut mock_service = MockUserRegistrationService::new();
        mock_service
            .expect_verify_email()
            .times(1)
            .returning(|_| Ok(()));

        let use_case = Arc::new(UserRegistrationUseCase::new(mock_service));
        let app = test::init_service(App::new().service(web::scope("/api").configure(|cfg| {
            crate::presentation::controllers::user_registration_controller::configure_routes(
                cfg,
                use_case.clone(),
            )
        })))
        .await;

        let request_body = json!({
            "user_id": 1,
            "token": "test_token"
        });

        // When
        let req = test::TestRequest::post()
            .uri("/api/auth/verify-email")
            .set_json(&request_body)
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Then
        assert_eq!(resp.status(), 200);
    }

    #[tokio::test]
    async fn test_approve_user_endpoint_success() {
        // Given
        let mut mock_service = MockUserRegistrationService::new();
        mock_service
            .expect_approve_user()
            .times(1)
            .returning(|_, _| Ok(()));

        let use_case = Arc::new(UserRegistrationUseCase::new(mock_service));
        let app = test::init_service(App::new().service(web::scope("/api").configure(|cfg| {
            crate::presentation::controllers::user_registration_controller::configure_routes(
                cfg,
                use_case.clone(),
            )
        })))
        .await;

        let request_body = json!({
            "user_id": 1
        });

        // When
        let req = test::TestRequest::post()
            .uri("/api/admin/users/approve")
            .set_json(&request_body)
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Then
        assert_eq!(resp.status(), 200);
    }

    #[tokio::test]
    async fn test_delete_account_endpoint_success() {
        // Given
        let mut mock_service = MockUserRegistrationService::new();
        mock_service
            .expect_delete_account()
            .times(1)
            .returning(|_, _| Ok(()));

        let use_case = Arc::new(UserRegistrationUseCase::new(mock_service));
        let app = test::init_service(App::new().service(web::scope("/api").configure(|cfg| {
            crate::presentation::controllers::user_registration_controller::configure_routes(
                cfg,
                use_case.clone(),
            )
        })))
        .await;

        // When
        let req = test::TestRequest::delete().uri("/api/users/1").to_request();

        let resp = test::call_service(&app, req).await;

        // Then
        assert_eq!(resp.status(), 200);
    }
}
