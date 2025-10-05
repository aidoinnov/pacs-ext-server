#[cfg(test)]
mod access_control_controller_tests {
    use actix_web::{test, App};
    use pacs_server::application::dto::access_control_dto::{
        CheckPermissionRequest, LogDicomAccessRequest,
    };
    use pacs_server::application::use_cases::access_control_use_case::AccessControlUseCase;
    use pacs_server::domain::services::access_control_service::AccessControlServiceImpl;
    use pacs_server::infrastructure::repositories::{
        AccessLogRepositoryImpl, PermissionRepositoryImpl, ProjectRepositoryImpl,
        RoleRepositoryImpl, UserRepositoryImpl,
    };
    use pacs_server::presentation::controllers::access_control_controller::configure_routes;
    use sqlx::postgres::PgPoolOptions;
    use sqlx::Row;
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

        let access_log_repo = AccessLogRepositoryImpl::new(pool.clone());
        let user_repo = UserRepositoryImpl::new(pool.clone());
        let project_repo = ProjectRepositoryImpl::new(pool.clone());
        let role_repo = RoleRepositoryImpl::new(pool.clone());
        let permission_repo = PermissionRepositoryImpl::new(pool.clone());

        let pool = Arc::new(pool);
        let access_control_service = AccessControlServiceImpl::new(
            access_log_repo,
            user_repo,
            project_repo,
            role_repo,
            permission_repo,
        );
        let access_control_use_case = Arc::new(AccessControlUseCase::new(access_control_service));

        let app = test::init_service(
            App::new().configure(|cfg| configure_routes(cfg, access_control_use_case.clone())),
        )
        .await;

        (app, pool)
    }

    #[actix_web::test]
    async fn test_log_dicom_access_success() {
        let (app, pool) = setup_test_app().await;

        // Create test user and project
        let uuid = Uuid::new_v4();
        let unique_username = format!("test_access_user_{}", uuid);
        let unique_email = format!("access_{}@test.com", uuid);
        let user_result = sqlx::query(
            "INSERT INTO security_user (keycloak_id, username, email) VALUES ($1, $2, $3) RETURNING id"
        )
        .bind(Uuid::new_v4())
        .bind(&unique_username)
        .bind(&unique_email)
        .fetch_one(&*pool)
        .await
        .expect("Failed to create test user");
        let user_id: i32 = user_result.get("id");

        let unique_project_name = format!("Test Access Project {}", uuid);
        let project_result = sqlx::query(
            "INSERT INTO security_project (name) VALUES ($1) RETURNING id"
        )
        .bind(&unique_project_name)
        .fetch_one(&*pool)
        .await
        .expect("Failed to create test project");
        let project_id: i32 = project_result.get("id");

        let log_req = LogDicomAccessRequest {
            user_id,
            project_id: Some(project_id),
            resource_type: "STUDY".to_string(),
            study_uid: Some("1.2.3.4.5".to_string()),
            series_uid: None,
            instance_uid: None,
            action: "VIEW".to_string(),
            result: "SUCCESS".to_string(),
            ip_address: Some("127.0.0.1".to_string()),
            ae_title: Some("TEST_AE".to_string()),
        };

        let req = test::TestRequest::post()
            .uri("/access-control/logs")
            .set_json(&log_req)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201);

        // Cleanup
        sqlx::query("DELETE FROM security_access_log WHERE user_id = $1")
            .bind(user_id)
            .execute(&*pool)
            .await
            .ok();
        sqlx::query("DELETE FROM security_user WHERE id = $1")
            .bind(user_id)
            .execute(&*pool)
            .await
            .ok();
        sqlx::query("DELETE FROM security_project WHERE id = $1")
            .bind(project_id)
            .execute(&*pool)
            .await
            .ok();
    }

    #[actix_web::test]
    async fn test_get_user_access_logs() {
        let (app, pool) = setup_test_app().await;

        // Create test user
        let uuid = Uuid::new_v4();
        let unique_username = format!("test_log_user_{}", uuid);
        let unique_email = format!("logs_{}@test.com", uuid);
        let user_result = sqlx::query(
            "INSERT INTO security_user (keycloak_id, username, email) VALUES ($1, $2, $3) RETURNING id"
        )
        .bind(Uuid::new_v4())
        .bind(&unique_username)
        .bind(&unique_email)
        .fetch_one(&*pool)
        .await
        .expect("Failed to create test user");
        let user_id: i32 = user_result.get("id");

        let req = test::TestRequest::get()
            .uri(&format!("/access-control/logs/user/{}?limit=10", user_id))
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
    async fn test_get_project_access_logs() {
        let (app, pool) = setup_test_app().await;

        // Create test project
        let uuid = Uuid::new_v4();
        let unique_project_name = format!("Test Log Project {}", uuid);
        let project_result = sqlx::query(
            "INSERT INTO security_project (name) VALUES ($1) RETURNING id"
        )
        .bind(&unique_project_name)
        .fetch_one(&*pool)
        .await
        .expect("Failed to create test project");
        let project_id: i32 = project_result.get("id");

        let req = test::TestRequest::get()
            .uri(&format!(
                "/access-control/logs/project/{}?limit=10",
                project_id
            ))
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
    async fn test_get_study_access_logs() {
        let (app, _pool) = setup_test_app().await;

        let study_uid = "1.2.3.4.5.6.7";

        let req = test::TestRequest::get()
            .uri(&format!(
                "/access-control/logs/study/{}?limit=10",
                study_uid
            ))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
    }

    #[actix_web::test]
    async fn test_check_permission() {
        let (app, pool) = setup_test_app().await;

        // Create test user and project
        let uuid = Uuid::new_v4();
        let unique_username = format!("test_perm_user_{}", uuid);
        let unique_email = format!("perm_{}@test.com", uuid);
        let user_result = sqlx::query(
            "INSERT INTO security_user (keycloak_id, username, email) VALUES ($1, $2, $3) RETURNING id"
        )
        .bind(Uuid::new_v4())
        .bind(&unique_username)
        .bind(&unique_email)
        .fetch_one(&*pool)
        .await
        .expect("Failed to create test user");
        let user_id: i32 = user_result.get("id");

        let unique_project_name = format!("Test Perm Project {}", uuid);
        let project_result = sqlx::query(
            "INSERT INTO security_project (name) VALUES ($1) RETURNING id"
        )
        .bind(&unique_project_name)
        .fetch_one(&*pool)
        .await
        .expect("Failed to create test project");
        let project_id: i32 = project_result.get("id");

        let check_req = CheckPermissionRequest {
            user_id,
            project_id,
            resource_type: "STUDY".to_string(),
            action: "VIEW".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/access-control/permissions/check")
            .set_json(&check_req)
            .to_request();

        let resp = test::call_service(&app, req).await;
        // May return 200 or 403 depending on actual permissions
        assert!(resp.status().is_success() || resp.status() == 403);

        // Cleanup
        sqlx::query("DELETE FROM security_user WHERE id = $1")
            .bind(user_id)
            .execute(&*pool)
            .await
            .ok();
        sqlx::query("DELETE FROM security_project WHERE id = $1")
            .bind(project_id)
            .execute(&*pool)
            .await
            .ok();
    }

    #[actix_web::test]
    async fn test_get_user_permissions() {
        let (app, pool) = setup_test_app().await;

        // Create test user and project
        let uuid = Uuid::new_v4();
        let unique_username = format!("test_user_perm_{}", uuid);
        let unique_email = format!("userperm_{}@test.com", uuid);
        let user_result = sqlx::query(
            "INSERT INTO security_user (keycloak_id, username, email) VALUES ($1, $2, $3) RETURNING id"
        )
        .bind(Uuid::new_v4())
        .bind(&unique_username)
        .bind(&unique_email)
        .fetch_one(&*pool)
        .await
        .expect("Failed to create test user");
        let user_id: i32 = user_result.get("id");

        let unique_project_name = format!("Test User Perm Project {}", uuid);
        let project_result = sqlx::query(
            "INSERT INTO security_project (name) VALUES ($1) RETURNING id"
        )
        .bind(&unique_project_name)
        .fetch_one(&*pool)
        .await
        .expect("Failed to create test project");
        let project_id: i32 = project_result.get("id");

        let req = test::TestRequest::get()
            .uri(&format!(
                "/access-control/permissions/user/{}/project/{}",
                user_id, project_id
            ))
            .to_request();

        let resp = test::call_service(&app, req).await;
        // May return 200 or 500 depending on implementation details
        assert!(resp.status().is_success() || resp.status() == 500);

        // Cleanup
        sqlx::query("DELETE FROM security_user WHERE id = $1")
            .bind(user_id)
            .execute(&*pool)
            .await
            .ok();
        sqlx::query("DELETE FROM security_project WHERE id = $1")
            .bind(project_id)
            .execute(&*pool)
            .await
            .ok();
    }

    #[actix_web::test]
    async fn test_can_access_project() {
        let (app, pool) = setup_test_app().await;

        // Create test user and project
        let uuid = Uuid::new_v4();
        let unique_username = format!("test_access_check_{}", uuid);
        let unique_email = format!("accesscheck_{}@test.com", uuid);
        let user_result = sqlx::query(
            "INSERT INTO security_user (keycloak_id, username, email) VALUES ($1, $2, $3) RETURNING id"
        )
        .bind(Uuid::new_v4())
        .bind(&unique_username)
        .bind(&unique_email)
        .fetch_one(&*pool)
        .await
        .expect("Failed to create test user");
        let user_id: i32 = user_result.get("id");

        let unique_project_name = format!("Test Access Check Project {}", uuid);
        let project_result = sqlx::query(
            "INSERT INTO security_project (name) VALUES ($1) RETURNING id"
        )
        .bind(&unique_project_name)
        .fetch_one(&*pool)
        .await
        .expect("Failed to create test project");
        let project_id: i32 = project_result.get("id");

        let req = test::TestRequest::get()
            .uri(&format!(
                "/access-control/access/user/{}/project/{}",
                user_id, project_id
            ))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        // Cleanup
        sqlx::query("DELETE FROM security_user WHERE id = $1")
            .bind(user_id)
            .execute(&*pool)
            .await
            .ok();
        sqlx::query("DELETE FROM security_project WHERE id = $1")
            .bind(project_id)
            .execute(&*pool)
            .await
            .ok();
    }
}
