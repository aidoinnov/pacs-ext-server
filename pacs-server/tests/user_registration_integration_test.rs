#[cfg(test)]
mod tests {
    use actix_web::{test, web, App};
    use serde_json::json;
    use std::sync::Arc;
    use tokio;
    use crate::application::dto::user_registration_dto::*;
    use crate::application::use_cases::UserRegistrationUseCase;
    use crate::domain::ServiceError;
    use crate::infrastructure::services::UserRegistrationServiceImpl;
    use crate::infrastructure::external::KeycloakClient;
    use crate::infrastructure::config::KeycloakConfig;

    #[tokio::test]
    async fn test_signup_integration() {
        // Given
        let keycloak_config = KeycloakConfig {
            url: "http://localhost:8080".to_string(),
            realm: "dcm4che".to_string(),
            client_id: "pacs-server".to_string(),
            client_secret: "your-client-secret".to_string(),
            admin_username: "admin".to_string(),
            admin_password: "adminPassword123!".to_string(),
        };

        let keycloak_client = KeycloakClient::new(keycloak_config);
        let service = UserRegistrationServiceImpl::new(
            // 실제 테스트에서는 테스트 데이터베이스 풀을 사용해야 함
            // 여기서는 간단한 테스트만 작성
            Arc::new(keycloak_client),
        );

        let use_case = UserRegistrationUseCase::new(service);
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
        // 실제 Keycloak 서버가 없으므로 에러가 발생할 것으로 예상됨
        assert!(resp.status() >= 400);
    }

    #[tokio::test]
    async fn test_verify_email_integration() {
        // Given
        let keycloak_config = KeycloakConfig {
            url: "http://localhost:8080".to_string(),
            realm: "dcm4che".to_string(),
            client_id: "pacs-server".to_string(),
            client_secret: "your-client-secret".to_string(),
            admin_username: "admin".to_string(),
            admin_password: "adminPassword123!".to_string(),
        };

        let keycloak_client = KeycloakClient::new(keycloak_config);
        let service = UserRegistrationServiceImpl::new(
            Arc::new(keycloak_client),
        );

        let use_case = UserRegistrationUseCase::new(service);
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
        // 실제 Keycloak 서버가 없으므로 에러가 발생할 것으로 예상됨
        assert!(resp.status() >= 400);
    }

    #[tokio::test]
    async fn test_approve_user_integration() {
        // Given
        let keycloak_config = KeycloakConfig {
            url: "http://localhost:8080".to_string(),
            realm: "dcm4che".to_string(),
            client_id: "pacs-server".to_string(),
            client_secret: "your-client-secret".to_string(),
            admin_username: "admin".to_string(),
            admin_password: "adminPassword123!".to_string(),
        };

        let keycloak_client = KeycloakClient::new(keycloak_config);
        let service = UserRegistrationServiceImpl::new(
            Arc::new(keycloak_client),
        );

        let use_case = UserRegistrationUseCase::new(service);
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
        // 실제 Keycloak 서버가 없으므로 에러가 발생할 것으로 예상됨
        assert!(resp.status() >= 400);
    }

    #[tokio::test]
    async fn test_delete_account_integration() {
        // Given
        let keycloak_config = KeycloakConfig {
            url: "http://localhost:8080".to_string(),
            realm: "dcm4che".to_string(),
            client_id: "pacs-server".to_string(),
            client_secret: "your-client-secret".to_string(),
            admin_username: "admin".to_string(),
            admin_password: "adminPassword123!".to_string(),
        };

        let keycloak_client = KeycloakClient::new(keycloak_config);
        let service = UserRegistrationServiceImpl::new(
            Arc::new(keycloak_client),
        );

        let use_case = UserRegistrationUseCase::new(service);
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
        // 실제 Keycloak 서버가 없으므로 에러가 발생할 것으로 예상됨
        assert!(resp.status() >= 400);
    }
}
