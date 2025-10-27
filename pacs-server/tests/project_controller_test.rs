#[cfg(test)]
mod project_controller_tests {
    use actix_web::{test, App};
    use pacs_server::application::dto::project_dto::CreateProjectRequest;
    use pacs_server::application::use_cases::project_use_case::ProjectUseCase;
    use pacs_server::domain::services::project_service::ProjectServiceImpl;
    use pacs_server::infrastructure::repositories::{
        ProjectRepositoryImpl, RoleRepositoryImpl, UserRepositoryImpl,
    };
    use pacs_server::presentation::controllers::project_controller::configure_routes;
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

        let project_repo = ProjectRepositoryImpl::new(pool.clone());
        let user_repo = UserRepositoryImpl::new(pool.clone());
        let role_repo = RoleRepositoryImpl::new(pool.clone());

        let pool = Arc::new(pool);
        let project_service = ProjectServiceImpl::new(project_repo, user_repo, role_repo);
        let project_use_case = Arc::new(ProjectUseCase::new(project_service));

        let app = test::init_service(
            App::new().configure(|cfg| configure_routes(cfg, project_use_case.clone())),
        )
        .await;

        (app, pool)
    }

    #[actix_web::test]
    async fn test_create_project_success() {
        let (app, pool) = setup_test_app().await;

        let create_req = CreateProjectRequest {
            name: "Test Project".to_string(),
            description: Some("A test project".to_string()),
            sponsor: "Test Sponsor".to_string(),
            start_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            end_date: None,
            auto_complete: None,
        };

        let req = test::TestRequest::post()
            .uri("/projects")
            .set_json(&create_req)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201);

        // Cleanup
        sqlx::query("DELETE FROM security_project WHERE name = $1")
            .bind("Test Project")
            .execute(&*pool)
            .await
            .ok();
    }

    #[actix_web::test]
    async fn test_get_project_by_id() {
        let (app, pool) = setup_test_app().await;

        // Create test project
        use sqlx::Row;
        let result = sqlx::query(
            "INSERT INTO security_project (name, description) VALUES ($1, $2) RETURNING id"
        )
        .bind("Get Project")
        .bind("Description")
        .fetch_one(&*pool)
        .await
        .expect("Failed to create test project");

        let project_id: i32 = result.get("id");

        let req = test::TestRequest::get()
            .uri(&format!("/projects/{}", project_id))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        // Cleanup
        sqlx::query("DELETE FROM security_project WHERE id = $1")
            .bind(project_id)
            .execute(&*pool)
            .await
            .ok();
    }

    #[actix_web::test]
    async fn test_get_project_not_found() {
        let (app, _pool) = setup_test_app().await;

        let req = test::TestRequest::get()
            .uri("/projects/999999")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404);
    }

    #[actix_web::test]
    async fn test_list_projects() {
        let (app, _pool) = setup_test_app().await;

        let req = test::TestRequest::get().uri("/projects").to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
    }

    #[actix_web::test]
    async fn test_get_active_projects() {
        let (app, _pool) = setup_test_app().await;

        let req = test::TestRequest::get()
            .uri("/projects/active")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
    }
}
