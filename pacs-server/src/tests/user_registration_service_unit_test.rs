#[cfg(test)]
mod tests {
    use tokio;
    use crate::domain::ServiceError;
    use crate::infrastructure::external::KeycloakClient;
    use crate::infrastructure::services::UserRegistrationServiceImpl;
    use crate::domain::services::UserRegistrationService;
    use sqlx::PgPool;

    #[tokio::test]
    async fn test_signup_without_keycloak() {
        // Given
        let keycloak_client = KeycloakClient::new(crate::infrastructure::config::KeycloakConfig {
            url: "http://localhost:8080".to_string(),
            realm: "test".to_string(),
            client_id: "test".to_string(),
            client_secret: "test".to_string(),
            admin_username: "admin".to_string(),
            admin_password: "admin".to_string(),
        });

        // Lazy PgPool - 실제 연결하지 않음
        let opts = sqlx::postgres::PgConnectOptions::new()
            .host("localhost")
            .port(5432)
            .database("testdb")
            .username("user")
            .password("password");
        let pool = sqlx::postgres::PgPoolOptions::new().connect_lazy_with(opts);
        let service = UserRegistrationServiceImpl::new(pool, keycloak_client);

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
    async fn test_verify_email_without_db() {
        // Given
        let keycloak_client = KeycloakClient::new(crate::infrastructure::config::KeycloakConfig {
            url: "http://localhost:8080".to_string(),
            realm: "test".to_string(),
            client_id: "test".to_string(),
            client_secret: "test".to_string(),
            admin_username: "admin".to_string(),
            admin_password: "admin".to_string(),
        });

        let opts = sqlx::postgres::PgConnectOptions::new()
            .host("localhost")
            .port(5432)
            .database("testdb")
            .username("user")
            .password("password");
        let pool = sqlx::postgres::PgPoolOptions::new().connect_lazy_with(opts);
        let service = UserRegistrationServiceImpl::new(pool, keycloak_client);

        // When
        let result = service.verify_email(1).await;

        // Then
        // 실제 데이터베이스 연결이 없으므로 에러가 발생할 것으로 예상됨
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_approve_user_without_db() {
        // Given
        let keycloak_client = KeycloakClient::new(crate::infrastructure::config::KeycloakConfig {
            url: "http://localhost:8080".to_string(),
            realm: "test".to_string(),
            client_id: "test".to_string(),
            client_secret: "test".to_string(),
            admin_username: "admin".to_string(),
            admin_password: "admin".to_string(),
        });

        let opts = sqlx::postgres::PgConnectOptions::new()
            .host("localhost")
            .port(5432)
            .database("testdb")
            .username("user")
            .password("password");
        let pool = sqlx::postgres::PgPoolOptions::new().connect_lazy_with(opts);
        let service = UserRegistrationServiceImpl::new(pool, keycloak_client);

        // When
        let result = service.approve_user(1, 2).await;

        // Then
        // 실제 데이터베이스 연결이 없으므로 에러가 발생할 것으로 예상됨
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_account_without_db() {
        // Given
        let keycloak_client = KeycloakClient::new(crate::infrastructure::config::KeycloakConfig {
            url: "http://localhost:8080".to_string(),
            realm: "test".to_string(),
            client_id: "test".to_string(),
            client_secret: "test".to_string(),
            admin_username: "admin".to_string(),
            admin_password: "admin".to_string(),
        });

        let opts = sqlx::postgres::PgConnectOptions::new()
            .host("localhost")
            .port(5432)
            .database("testdb")
            .username("user")
            .password("password");
        let pool = sqlx::postgres::PgPoolOptions::new().connect_lazy_with(opts);
        let service = UserRegistrationServiceImpl::new(pool, keycloak_client);

        // When
        let result = service.delete_account(1, Some(2)).await;

        // Then
        // 실제 데이터베이스 연결이 없으므로 에러가 발생할 것으로 예상됨
        assert!(result.is_err());
    }
}