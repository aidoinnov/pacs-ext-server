#[cfg(test)]
mod permission_controller_tests {
    use actix_web::{test, App};
    use pacs_server::application::dto::permission_dto::CreateRoleRequest;
    use pacs_server::application::use_cases::permission_use_case::PermissionUseCase;
    use pacs_server::domain::services::permission_service::PermissionServiceImpl;
    use pacs_server::infrastructure::repositories::{
        PermissionRepositoryImpl, RoleRepositoryImpl,
    };
    use pacs_server::presentation::controllers::permission_controller::configure_routes;
    use sqlx::postgres::PgPoolOptions;
    use std::sync::Arc;

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

        let role_repo = RoleRepositoryImpl::new(pool.clone());
        let permission_repo = PermissionRepositoryImpl::new(pool.clone());

        let pool = Arc::new(pool);
        let permission_service = PermissionServiceImpl::new(permission_repo, role_repo);
        let permission_use_case = Arc::new(PermissionUseCase::new(permission_service));

        let app = test::init_service(
            App::new().configure(|cfg| configure_routes(cfg, permission_use_case.clone())),
        )
        .await;

        (app, pool)
    }

    #[actix_web::test]
    async fn test_create_role_success() {
        let (app, pool) = setup_test_app().await;

        let create_req = CreateRoleRequest {
            name: "TEST_ROLE".to_string(),
            scope: "GLOBAL".to_string(),
            description: Some("Test role".to_string()),
        };

        let req = test::TestRequest::post()
            .uri("/roles")
            .set_json(&create_req)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201);

        // Cleanup
        sqlx::query("DELETE FROM security_role WHERE name = $1")
            .bind("TEST_ROLE")
            .execute(&*pool)
            .await
            .ok();
    }

    #[actix_web::test]
    async fn test_get_role_by_id() {
        let (app, pool) = setup_test_app().await;

        // Create test role
        use sqlx::Row;
        let result = sqlx::query(
            "INSERT INTO security_role (name, scope, description) VALUES ($1, $2, $3) RETURNING id"
        )
        .bind("GET_ROLE")
        .bind("GLOBAL")
        .bind("Get role test")
        .fetch_one(&*pool)
        .await
        .expect("Failed to create test role");

        let role_id: i32 = result.get("id");

        let req = test::TestRequest::get()
            .uri(&format!("/roles/{}", role_id))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        // Cleanup
        sqlx::query("DELETE FROM security_role WHERE id = $1")
            .bind(role_id)
            .execute(&*pool)
            .await
            .ok();
    }

    #[actix_web::test]
    async fn test_get_role_not_found() {
        let (app, _pool) = setup_test_app().await;

        let req = test::TestRequest::get()
            .uri("/roles/999999")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404);
    }

    #[actix_web::test]
    async fn test_get_global_roles() {
        let (app, _pool) = setup_test_app().await;

        let req = test::TestRequest::get().uri("/roles/global").to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
    }

    #[actix_web::test]
    async fn test_get_project_roles() {
        let (app, _pool) = setup_test_app().await;

        let req = test::TestRequest::get()
            .uri("/roles/project")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
    }
}
