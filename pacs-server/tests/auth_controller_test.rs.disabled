#[cfg(test)]
mod auth_controller_tests {
    use actix_web::{test, web, App};
    use pacs_server::application::dto::auth_dto::LoginRequest;
    use pacs_server::application::use_cases::auth_use_case::AuthUseCase;
    use pacs_server::domain::services::auth_service::AuthServiceImpl;
    use pacs_server::infrastructure::repositories::UserRepositoryImpl;
    use pacs_server::infrastructure::auth::jwt_service::JwtService;
    use pacs_server::presentation::controllers::auth_controller::configure_routes;
    use sqlx::postgres::PgPoolOptions;
    use std::sync::Arc;
    use uuid::Uuid;

    async fn setup_test_app() -> (
        impl actix_web::dev::Service<
            actix_http::Request,
            Response = actix_web::dev::ServiceResponse,
            Error = actix_web::Error,
        >,
        Arc<sqlx::Pool<sqlx::Postgres>>,
    ) {
        dotenvy::dotenv().ok();

        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://admin:admin123@localhost:5432/pacs_db".to_string());

        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await
            .expect("Failed to connect to test database");

        let user_repo = UserRepositoryImpl::new(pool.clone());

        let jwt_config = pacs_server::infrastructure::config::JwtConfig {
            secret: std::env::var("JWT_SECRET").unwrap_or_else(|_| "test_secret".to_string()),
            expiration_hours: 24,
        };
        let jwt_service = JwtService::new(&jwt_config);

        let pool = Arc::new(pool);
        let auth_service = AuthServiceImpl::new(user_repo, jwt_service);
        let auth_use_case = Arc::new(AuthUseCase::new(auth_service));

        let app = test::init_service(
            App::new().configure(|cfg| configure_routes(cfg, auth_use_case.clone())),
        )
        .await;

        (app, pool)
    }

    #[actix_web::test]
    async fn test_login_with_existing_user() {
        let (app, pool) = setup_test_app().await;

        // Create test user
        let keycloak_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO security_user (keycloak_id, username, email) VALUES ($1, $2, $3)"
        )
        .bind(keycloak_id)
        .bind("testuser")
        .bind("test@example.com")
        .execute(&*pool)
        .await
        .expect("Failed to create test user");

        let login_req = LoginRequest {
            keycloak_id,
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/auth/login")
            .set_json(&login_req)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        // Cleanup
        sqlx::query("DELETE FROM security_user WHERE keycloak_id = $1")
            .bind(keycloak_id)
            .execute(&*pool)
            .await
            .ok();
    }

    #[actix_web::test]
    async fn test_login_creates_new_user() {
        let (app, pool) = setup_test_app().await;

        let keycloak_id = Uuid::new_v4();
        let login_req = LoginRequest {
            keycloak_id,
            username: "newuser".to_string(),
            email: "new@example.com".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/auth/login")
            .set_json(&login_req)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        // Cleanup
        sqlx::query("DELETE FROM security_user WHERE keycloak_id = $1")
            .bind(keycloak_id)
            .execute(&*pool)
            .await
            .ok();
    }

    #[actix_web::test]
    async fn test_verify_token_invalid() {
        let (app, _pool) = setup_test_app().await;

        let req = test::TestRequest::get()
            .uri("/auth/verify/invalid_token_here")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 401);
    }
}
