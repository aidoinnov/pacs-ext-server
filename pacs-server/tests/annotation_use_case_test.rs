#[cfg(test)]
mod annotation_use_case_tests {
    use actix_web::{test, App};
    use pacs_server::application::dto::annotation_dto::{
        CreateAnnotationRequest, UpdateAnnotationRequest
    };
    use pacs_server::application::use_cases::AnnotationUseCase;
    use pacs_server::domain::services::AnnotationServiceImpl;
    use pacs_server::infrastructure::repositories::{
        AnnotationRepositoryImpl, UserRepositoryImpl, ProjectRepositoryImpl,
    };
    use pacs_server::presentation::controllers::annotation_controller;
    use sqlx::postgres::PgPoolOptions;
    use std::sync::Arc;
    use uuid::Uuid;
    use serde_json::json;

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

        let annotation_repo = AnnotationRepositoryImpl::new(pool.clone());
        let user_repo = UserRepositoryImpl::new(pool.clone());
        let project_repo = ProjectRepositoryImpl::new(pool.clone());

        let pool = Arc::new(pool);
        let annotation_service = AnnotationServiceImpl::new(annotation_repo, user_repo, project_repo);
        let annotation_use_case = Arc::new(AnnotationUseCase::new(annotation_service));

        let app = test::init_service(
            App::new().configure(|cfg| annotation_controller::configure_routes(cfg, annotation_use_case.clone())),
        )
        .await;

        (app, pool)
    }

    async fn cleanup_test_data(pool: &sqlx::Pool<sqlx::Postgres>) {
        // 관계 테이블 먼저 삭제 (FK 제약)
        sqlx::query("DELETE FROM annotation_annotation_history").execute(pool).await.unwrap();
        sqlx::query("DELETE FROM annotation_annotation").execute(pool).await.unwrap();
        sqlx::query("DELETE FROM security_access_log").execute(pool).await.unwrap();
        sqlx::query("DELETE FROM security_user_project").execute(pool).await.unwrap();
        sqlx::query("DELETE FROM security_user").execute(pool).await.unwrap();
        sqlx::query("DELETE FROM security_project").execute(pool).await.unwrap();
    }

    #[actix_web::test]
    async fn test_create_annotation_use_case() {
        let (app, pool) = setup_test_app().await;
        cleanup_test_data(&pool).await;

        // Create test user and project
        let keycloak_id = Uuid::new_v4();
        let user_result = sqlx::query(
            "INSERT INTO security_user (keycloak_id, username, email) VALUES ($1, $2, $3) RETURNING id"
        )
        .bind(keycloak_id)
        .bind(&format!("testuser_{}", keycloak_id))
        .bind(&format!("test_{}@example.com", keycloak_id))
        .fetch_one(&*pool)
        .await
        .expect("Failed to create test user");

        let project_result = sqlx::query(
            "INSERT INTO security_project (name, description) VALUES ($1, $2) RETURNING id"
        )
        .bind(&format!("Test Project {}", keycloak_id))
        .bind(&format!("Test Description {}", keycloak_id))
        .fetch_one(&*pool)
        .await
        .expect("Failed to create test project");

        use sqlx::Row;
        let user_id: i32 = user_result.get("id");
        let project_id: i32 = project_result.get("id");

        // Add user to project
        sqlx::query(
            "INSERT INTO security_user_project (user_id, project_id) VALUES ($1, $2)"
        )
        .bind(user_id)
        .bind(project_id)
        .execute(&*pool)
        .await
        .expect("Failed to add user to project");

        let create_req = CreateAnnotationRequest {
            study_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.1".to_string(),
            series_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.2".to_string(),
            sop_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.3".to_string(),
            tool_name: Some("test_tool".to_string()),
            tool_version: Some("1.0.0".to_string()),
            viewer_software: Some("test_viewer".to_string()),
            annotation_data: json!({"type": "circle", "x": 100, "y": 200, "radius": 50}),
            description: Some("Test annotation".to_string()),
        };

        let req = test::TestRequest::post()
            .uri("/annotations")
            .set_json(&create_req)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201);

        // Verify annotation was created
        let annotation_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM annotation_annotation WHERE user_id = $1"
        )
        .bind(user_id)
        .fetch_one(&*pool)
        .await
        .expect("Failed to count annotations");

        assert_eq!(annotation_count, 1);

        cleanup_test_data(&pool).await;
    }

    #[actix_web::test]
    async fn test_get_annotation_use_case() {
        let (app, pool) = setup_test_app().await;
        cleanup_test_data(&pool).await;

        // Create test user and project
        let keycloak_id = Uuid::new_v4();
        let user_result = sqlx::query(
            "INSERT INTO security_user (keycloak_id, username, email) VALUES ($1, $2, $3) RETURNING id"
        )
        .bind(keycloak_id)
        .bind(&format!("testuser_{}", keycloak_id))
        .bind(&format!("test_{}@example.com", keycloak_id))
        .fetch_one(&*pool)
        .await
        .expect("Failed to create test user");

        let project_result = sqlx::query(
            "INSERT INTO security_project (name, description) VALUES ($1, $2) RETURNING id"
        )
        .bind(&format!("Test Project {}", keycloak_id))
        .bind(&format!("Test Description {}", keycloak_id))
        .fetch_one(&*pool)
        .await
        .expect("Failed to create test project");

        use sqlx::Row;
        let user_id: i32 = user_result.get("id");
        let project_id: i32 = project_result.get("id");

        // Add user to project
        sqlx::query(
            "INSERT INTO security_user_project (user_id, project_id) VALUES ($1, $2)"
        )
        .bind(user_id)
        .bind(project_id)
        .execute(&*pool)
        .await
        .expect("Failed to add user to project");

        // Create annotation
        let annotation_result = sqlx::query(
            "INSERT INTO annotation_annotation (project_id, user_id, study_uid, series_uid, instance_uid, tool_name, data, is_shared) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id"
        )
        .bind(project_id)
        .bind(user_id)
        .bind("1.2.3.4.5")
        .bind("1.2.3.4.6")
        .bind("1.2.3.4.7")
        .bind("test_tool")
        .bind(json!({"type": "test"}))
        .bind(false)
        .fetch_one(&*pool)
        .await
        .expect("Failed to create test annotation");

        let annotation_id: i32 = annotation_result.get("id");

        let req = test::TestRequest::get()
            .uri(&format!("/annotations/{}", annotation_id))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        cleanup_test_data(&pool).await;
    }

    #[actix_web::test]
    async fn test_update_annotation_use_case() {
        let (app, pool) = setup_test_app().await;
        cleanup_test_data(&pool).await;

        // Create test user and project
        let keycloak_id = Uuid::new_v4();
        let user_result = sqlx::query(
            "INSERT INTO security_user (keycloak_id, username, email) VALUES ($1, $2, $3) RETURNING id"
        )
        .bind(keycloak_id)
        .bind(&format!("testuser_{}", keycloak_id))
        .bind(&format!("test_{}@example.com", keycloak_id))
        .fetch_one(&*pool)
        .await
        .expect("Failed to create test user");

        let project_result = sqlx::query(
            "INSERT INTO security_project (name, description) VALUES ($1, $2) RETURNING id"
        )
        .bind(&format!("Test Project {}", keycloak_id))
        .bind(&format!("Test Description {}", keycloak_id))
        .fetch_one(&*pool)
        .await
        .expect("Failed to create test project");

        use sqlx::Row;
        let user_id: i32 = user_result.get("id");
        let project_id: i32 = project_result.get("id");

        // Add user to project
        sqlx::query(
            "INSERT INTO security_user_project (user_id, project_id) VALUES ($1, $2)"
        )
        .bind(user_id)
        .bind(project_id)
        .execute(&*pool)
        .await
        .expect("Failed to add user to project");

        // Create annotation
        let annotation_result = sqlx::query(
            "INSERT INTO annotation_annotation (project_id, user_id, study_uid, series_uid, instance_uid, tool_name, data, is_shared) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id"
        )
        .bind(project_id)
        .bind(user_id)
        .bind("1.2.3.4.5")
        .bind("1.2.3.4.6")
        .bind("1.2.3.4.7")
        .bind("test_tool")
        .bind(json!({"type": "test"}))
        .bind(false)
        .fetch_one(&*pool)
        .await
        .expect("Failed to create test annotation");

        let annotation_id: i32 = annotation_result.get("id");

        let update_req = UpdateAnnotationRequest {
            tool_name: Some("updated_tool".to_string()),
            tool_version: Some("2.0.0".to_string()),
            viewer_software: Some("updated_viewer".to_string()),
            annotation_data: Some(json!({"type": "updated", "x": 200, "y": 300})),
            description: Some("Updated annotation".to_string()),
        };

        let req = test::TestRequest::put()
            .uri(&format!("/annotations/{}", annotation_id))
            .set_json(&update_req)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        cleanup_test_data(&pool).await;
    }

    #[actix_web::test]
    async fn test_delete_annotation_use_case() {
        let (app, pool) = setup_test_app().await;
        cleanup_test_data(&pool).await;

        // Create test user and project
        let keycloak_id = Uuid::new_v4();
        let user_result = sqlx::query(
            "INSERT INTO security_user (keycloak_id, username, email) VALUES ($1, $2, $3) RETURNING id"
        )
        .bind(keycloak_id)
        .bind(&format!("testuser_{}", keycloak_id))
        .bind(&format!("test_{}@example.com", keycloak_id))
        .fetch_one(&*pool)
        .await
        .expect("Failed to create test user");

        let project_result = sqlx::query(
            "INSERT INTO security_project (name, description) VALUES ($1, $2) RETURNING id"
        )
        .bind(&format!("Test Project {}", keycloak_id))
        .bind(&format!("Test Description {}", keycloak_id))
        .fetch_one(&*pool)
        .await
        .expect("Failed to create test project");

        use sqlx::Row;
        let user_id: i32 = user_result.get("id");
        let project_id: i32 = project_result.get("id");

        // Add user to project
        sqlx::query(
            "INSERT INTO security_user_project (user_id, project_id) VALUES ($1, $2)"
        )
        .bind(user_id)
        .bind(project_id)
        .execute(&*pool)
        .await
        .expect("Failed to add user to project");

        // Create annotation
        let annotation_result = sqlx::query(
            "INSERT INTO annotation_annotation (project_id, user_id, study_uid, series_uid, instance_uid, tool_name, data, is_shared) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id"
        )
        .bind(project_id)
        .bind(user_id)
        .bind("1.2.3.4.5")
        .bind("1.2.3.4.6")
        .bind("1.2.3.4.7")
        .bind("test_tool")
        .bind(json!({"type": "test"}))
        .bind(false)
        .fetch_one(&*pool)
        .await
        .expect("Failed to create test annotation");

        let annotation_id: i32 = annotation_result.get("id");

        let req = test::TestRequest::delete()
            .uri(&format!("/annotations/{}", annotation_id))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        // Verify annotation was deleted
        let annotation_exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM annotation_annotation WHERE id = $1)"
        )
        .bind(annotation_id)
        .fetch_one(&*pool)
        .await
        .expect("Failed to check annotation existence");

        assert!(!annotation_exists);

        cleanup_test_data(&pool).await;
    }

    #[actix_web::test]
    async fn test_list_annotations_use_case() {
        let (app, pool) = setup_test_app().await;
        cleanup_test_data(&pool).await;

        // Create test user and project
        let keycloak_id = Uuid::new_v4();
        let user_result = sqlx::query(
            "INSERT INTO security_user (keycloak_id, username, email) VALUES ($1, $2, $3) RETURNING id"
        )
        .bind(keycloak_id)
        .bind(&format!("testuser_{}", keycloak_id))
        .bind(&format!("test_{}@example.com", keycloak_id))
        .fetch_one(&*pool)
        .await
        .expect("Failed to create test user");

        let project_result = sqlx::query(
            "INSERT INTO security_project (name, description) VALUES ($1, $2) RETURNING id"
        )
        .bind(&format!("Test Project {}", keycloak_id))
        .bind(&format!("Test Description {}", keycloak_id))
        .fetch_one(&*pool)
        .await
        .expect("Failed to create test project");

        use sqlx::Row;
        let user_id: i32 = user_result.get("id");
        let project_id: i32 = project_result.get("id");

        // Add user to project
        sqlx::query(
            "INSERT INTO security_user_project (user_id, project_id) VALUES ($1, $2)"
        )
        .bind(user_id)
        .bind(project_id)
        .execute(&*pool)
        .await
        .expect("Failed to add user to project");

        // Create multiple annotations
        for i in 0..3 {
            sqlx::query(
                "INSERT INTO annotation_annotation (project_id, user_id, study_uid, series_uid, instance_uid, tool_name, data, is_shared) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
            )
            .bind(project_id)
            .bind(user_id)
            .bind(format!("1.2.3.4.{}", i))
            .bind(format!("1.2.3.5.{}", i))
            .bind(format!("1.2.3.6.{}", i))
            .bind("test_tool")
            .bind(json!({"type": "test", "index": i}))
            .bind(false)
            .execute(&*pool)
            .await
            .expect("Failed to create test annotation");
        }

        let req = test::TestRequest::get()
            .uri("/annotations")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        cleanup_test_data(&pool).await;
    }

    #[actix_web::test]
    async fn test_annotation_not_found_use_case() {
        let (app, _pool) = setup_test_app().await;

        let req = test::TestRequest::get()
            .uri("/annotations/999999")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404);
    }

    #[actix_web::test]
    async fn test_annotation_validation_use_case() {
        let (app, pool) = setup_test_app().await;
        cleanup_test_data(&pool).await;

        // Test with invalid JSON data
        let invalid_req = serde_json::json!({
            "study_instance_uid": "",
            "series_instance_uid": "1.2.3.4.5",
            "sop_instance_uid": "1.2.3.4.6",
            "annotation_data": "invalid_json",
            "description": "Test"
        });

        let req = test::TestRequest::post()
            .uri("/annotations")
            .set_json(&invalid_req)
            .to_request();

        let resp = test::call_service(&app, req).await;
        // Should return 400 for invalid request
        assert!(resp.status() == 400 || resp.status() == 422);

        cleanup_test_data(&pool).await;
    }
}
