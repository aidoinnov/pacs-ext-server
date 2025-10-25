#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App};
    use serde_json::json;
    use std::sync::Arc;
    use tokio;
    use crate::application::dto::user_registration_dto::*;
    use crate::application::use_cases::UserRegistrationUseCase;
    use crate::domain::ServiceError;
    use crate::domain::services::UserRegistrationService;
    use crate::infrastructure::services::UserRegistrationServiceImpl;
    use crate::infrastructure::external::KeycloakClient;

    // Mock UserRegistrationService for controller tests
    use mockall::mock;
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
    async fn test_signup_endpoint_success() {
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
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(use_case))
                .service(
                    web::scope("/api")
                        .route("/auth/signup", web::post().to(crate::presentation::controllers::user_registration_controller::signup))
                )
        ).await;

        let request_body = json!({
            "username": "testuser",
            "email": "test@example.com",
            "password": "password123",
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
        assert_eq!(resp.status(), 200);
        
        let body: UserStatusResponse = test::read_body_json(resp).await;
        assert_eq!(body.username, "testuser");
        assert_eq!(body.email, "test@example.com");
        assert_eq!(body.account_status, UserAccountStatus::PendingEmail);
    }

    #[tokio::test]
    async fn test_signup_endpoint_validation_error() {
        // Given
        let mut mock_service = MockUserRegistrationService::new();
        mock_service
            .expect_signup()
            .times(1)
            .returning(|_| Err(ServiceError::ValidationError("Invalid email format".to_string())));

        let use_case = UserRegistrationUseCase::new(Arc::new(mock_service));
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(use_case))
                .service(
                    web::scope("/api")
                        .route("/auth/signup", web::post().to(crate::presentation::controllers::user_registration_controller::signup))
                )
        ).await;

        let request_body = json!({
            "username": "testuser",
            "email": "invalid-email",
            "password": "password123"
        });

        // When
        let req = test::TestRequest::post()
            .uri("/api/auth/signup")
            .set_json(&request_body)
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Then
        assert_eq!(resp.status(), 400);
    }

    #[tokio::test]
    async fn test_verify_email_endpoint_success() {
        // Given
        let mut mock_service = MockUserRegistrationService::new();
        mock_service
            .expect_verify_email()
            .times(1)
            .returning(|_| Ok(()));

        let use_case = UserRegistrationUseCase::new(Arc::new(mock_service));
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(use_case))
                .service(
                    web::scope("/api")
                        .route("/auth/verify-email", web::post().to(crate::presentation::controllers::user_registration_controller::verify_email))
                )
        ).await;

        let request_body = json!({
            "user_id": 1,
            "token": "verification_token"
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

        let use_case = UserRegistrationUseCase::new(Arc::new(mock_service));
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(use_case))
                .service(
                    web::scope("/api")
                        .route("/admin/users/approve", web::post().to(crate::presentation::controllers::user_registration_controller::approve_user))
                )
        ).await;

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

        let use_case = UserRegistrationUseCase::new(Arc::new(mock_service));
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(use_case))
                .service(
                    web::scope("/api")
                        .route("/users/{user_id}", web::delete().to(crate::presentation::controllers::user_registration_controller::delete_account))
                )
        ).await;

        // When
        let req = test::TestRequest::delete()
            .uri("/api/users/1")
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Then
        assert_eq!(resp.status(), 200);
    }

    #[tokio::test]
    async fn test_get_user_status_endpoint_success() {
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
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(use_case))
                .service(
                    web::scope("/api")
                        .route("/users/{user_id}/status", web::get().to(crate::presentation::controllers::user_registration_controller::get_user_status))
                )
        ).await;

        // When
        let req = test::TestRequest::get()
            .uri("/api/users/1/status")
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Then
        assert_eq!(resp.status(), 200);
        
        let body: UserStatusResponse = test::read_body_json(resp).await;
        assert_eq!(body.user_id, 1);
        assert_eq!(body.username, "testuser");
        assert_eq!(body.account_status, UserAccountStatus::Active);
    }

    #[tokio::test]
    async fn test_get_user_status_endpoint_not_found() {
        // Given
        let mut mock_service = MockUserRegistrationService::new();
        mock_service
            .expect_get_user_status()
            .times(1)
            .returning(|_| Err(ServiceError::NotFound("User not found".to_string())));

        let use_case = UserRegistrationUseCase::new(Arc::new(mock_service));
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(use_case))
                .service(
                    web::scope("/api")
                        .route("/users/{user_id}/status", web::get().to(crate::presentation::controllers::user_registration_controller::get_user_status))
                )
        ).await;

        // When
        let req = test::TestRequest::get()
            .uri("/api/users/999/status")
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Then
        assert_eq!(resp.status(), 404);
    }
}
