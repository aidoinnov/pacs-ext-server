#[cfg(test)]
mod user_controller_tests {
    use actix_web::{test, web, App};
    use pacs_server::application::dto::user_dto::CreateUserRequest;
    use pacs_server::application::use_cases::user_use_case::UserUseCase;
    use pacs_server::domain::services::user_service::UserServiceImpl;
    use pacs_server::infrastructure::repositories::{UserRepositoryImpl, ProjectRepositoryImpl};
    use pacs_server::presentation::controllers::user_controller::configure_routes;
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
        let project_repo = ProjectRepositoryImpl::new(pool.clone());

        let pool = Arc::new(pool);
        let user_service = UserServiceImpl::new(user_repo, project_repo);
        let user_use_case = Arc::new(UserUseCase::new(user_service));

        let app = test::init_service(
            App::new().configure(|cfg| configure_routes(cfg, user_use_case.clone())),
        )
        .await;

        (app, pool)
    }

    #[actix_web::test]
    async fn test_create_user_success() {
        let (app, pool) = setup_test_app().await;

        let keycloak_id = Uuid::new_v4();
        let create_req = CreateUserRequest {
            keycloak_id,
            username: "newuser".to_string(),
            email: "newuser@example.com".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/users")
            .set_json(&create_req)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201);

        // Cleanup
        sqlx::query("DELETE FROM security_user WHERE keycloak_id = $1")
            .bind(keycloak_id)
            .execute(&*pool)
            .await
            .ok();
    }

    #[actix_web::test]
    async fn test_get_user_by_id() {
        let (app, pool) = setup_test_app().await;

        // Create test user
        let keycloak_id = Uuid::new_v4();
        let result = sqlx::query(
            "INSERT INTO security_user (keycloak_id, username, email) VALUES ($1, $2, $3) RETURNING id"
        )
        .bind(keycloak_id)
        .bind("getuser")
        .bind("get@example.com")
        .fetch_one(&*pool)
        .await
        .expect("Failed to create test user");

        use sqlx::Row;
        let user_id: i32 = result.get("id");

        let req = test::TestRequest::get()
            .uri(&format!("/users/{}", user_id))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        // Cleanup
        sqlx::query("DELETE FROM security_user WHERE id = $1")
            .bind(user_id)
            .execute(&*pool)
            .await
            .ok();
    }

    #[actix_web::test]
    async fn test_get_user_not_found() {
        let (app, _pool) = setup_test_app().await;

        let req = test::TestRequest::get()
            .uri("/users/999999")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404);
    }

    #[actix_web::test]
    async fn test_get_user_by_username() {
        let (app, pool) = setup_test_app().await;

        // Create test user
        let keycloak_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO security_user (keycloak_id, username, email) VALUES ($1, $2, $3)"
        )
        .bind(keycloak_id)
        .bind("usernameuser")
        .bind("username@example.com")
        .execute(&*pool)
        .await
        .expect("Failed to create test user");

        let req = test::TestRequest::get()
            .uri("/users/username/usernameuser")
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
    async fn test_get_user_by_username_not_found() {
        let (app, _pool) = setup_test_app().await;

        let req = test::TestRequest::get()
            .uri("/users/username/nonexistentuser")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404);
    }
}
