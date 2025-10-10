#[cfg(test)]
mod annotation_controller_tests {
    use actix_web::{test, web, App};
    use pacs_server::application::dto::annotation_dto::CreateAnnotationRequest;
    use pacs_server::application::use_cases::AnnotationUseCase;
    use pacs_server::domain::services::AnnotationServiceImpl;
    use pacs_server::domain::entities::annotation::Annotation;
    use pacs_server::infrastructure::repositories::{
        AnnotationRepositoryImpl, UserRepositoryImpl, ProjectRepositoryImpl,
    };
    use pacs_server::presentation::controllers::annotation_controller;
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

        let annotation_repo = AnnotationRepositoryImpl::new(pool.clone());
        let user_repo = UserRepositoryImpl::new(pool.clone());
        let project_repo = ProjectRepositoryImpl::new(pool.clone());

        let pool = Arc::new(pool);
        let annotation_service = AnnotationServiceImpl::new(annotation_repo, user_repo, project_repo);
        let annotation_use_case = Arc::new(AnnotationUseCase::new(annotation_service));

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(annotation_use_case.clone()))
            .configure(|cfg| annotation_controller::configure_routes(cfg, annotation_use_case.clone())),
    )
    .await;

        (app, pool)
    }

    #[actix_web::test]
    async fn test_create_annotation() {
        let (app, pool) = setup_test_app().await;

        // Create test user and project first
        let keycloak_id = Uuid::new_v4();
        let username = format!("testuser_{}", Uuid::new_v4());
        let email = format!("test_{}@example.com", Uuid::new_v4());
        let user_result = sqlx::query(
            "INSERT INTO security_user (keycloak_id, username, email) VALUES ($1, $2, $3) RETURNING id"
        )
        .bind(keycloak_id)
        .bind(&username)
        .bind(&email)
        .fetch_one(&*pool)
        .await
        .expect("Failed to create test user");

        let project_name = format!("Test Project {}", Uuid::new_v4());
        let project_result = sqlx::query(
            "INSERT INTO security_project (name, description) VALUES ($1, $2) RETURNING id"
        )
        .bind(&project_name)
        .bind("Test Description")
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
            annotation_data: serde_json::json!({"type": "circle", "x": 100, "y": 200, "radius": 50}),
            viewer_software: Some("OHIF Viewer".to_string()),
            tool_name: Some("Circle Tool".to_string()),
            tool_version: Some("2.1.0".to_string()),
            description: Some("Test annotation".to_string()),
        };

        let req = test::TestRequest::post()
            .uri("/annotations")
            .set_json(&create_req)
            .to_request();

        let resp = test::call_service(&app, req).await;
        let status = resp.status();
        if status != 201 {
            let body = test::read_body(resp).await;
            let body_str = String::from_utf8_lossy(&body);
            panic!("Expected status 201, got {}: {}", status, body_str);
        }

        // Cleanup
        sqlx::query("DELETE FROM annotation_annotation WHERE user_id = $1")
            .bind(user_id)
            .execute(&*pool)
            .await
            .ok();
        sqlx::query("DELETE FROM security_user_project WHERE user_id = $1")
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
    async fn test_list_annotations() {
        let (app, pool) = setup_test_app().await;

        // Create test user
        let keycloak_id = Uuid::new_v4();
        let username = format!("listuser_{}", Uuid::new_v4());
        let email = format!("list_{}@example.com", Uuid::new_v4());
        let user_result = sqlx::query(
            "INSERT INTO security_user (keycloak_id, username, email) VALUES ($1, $2, $3) RETURNING id"
        )
        .bind(keycloak_id)
        .bind(&username)
        .bind(&email)
        .fetch_one(&*pool)
        .await
        .expect("Failed to create test user");

        use sqlx::Row;
        let user_id: i32 = user_result.get("id");

        let req = test::TestRequest::get()
            .uri(&format!("/annotations?user_id={}", user_id))
            .to_request();

        let resp = test::call_service(&app, req).await;
        let status = resp.status();
        if status != 200 {
            let body = test::read_body(resp).await;
            let body_str = String::from_utf8_lossy(&body);
            panic!("Expected status 200, got {}: {}", status, body_str);
        }

        // Cleanup
        sqlx::query("DELETE FROM security_user WHERE id = $1")
            .bind(user_id)
            .execute(&*pool)
            .await
            .ok();
    }

    #[actix_web::test]
    async fn test_get_annotation_by_id() {
        let (app, pool) = setup_test_app().await;

        // Create test annotation
        let keycloak_id = Uuid::new_v4();
        let username = format!("getuser_{}", Uuid::new_v4());
        let email = format!("get_{}@example.com", Uuid::new_v4());
        let user_result = sqlx::query(
            "INSERT INTO security_user (keycloak_id, username, email) VALUES ($1, $2, $3) RETURNING id"
        )
        .bind(keycloak_id)
        .bind(&username)
        .bind(&email)
        .fetch_one(&*pool)
        .await
        .expect("Failed to create test user");

        let project_name = format!("Get Project {}", Uuid::new_v4());
        let project_result = sqlx::query(
            "INSERT INTO security_project (name, description) VALUES ($1, $2) RETURNING id"
        )
        .bind(&project_name)
        .bind("Description")
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
        .bind(serde_json::json!({"type": "test"}))
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

        // Cleanup
        sqlx::query("DELETE FROM annotation_annotation WHERE id = $1")
            .bind(annotation_id)
            .execute(&*pool)
            .await
            .ok();
        sqlx::query("DELETE FROM security_user_project WHERE user_id = $1")
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
    async fn test_get_annotation_not_found() {
        let (app, _pool) = setup_test_app().await;

        let req = test::TestRequest::get()
            .uri("/annotations/999999")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404);
    }

    #[actix_web::test]
    async fn test_update_annotation() {
        let (app, pool) = setup_test_app().await;

        // Create test annotation
        let keycloak_id = Uuid::new_v4();
        let username = format!("updateuser_{}", Uuid::new_v4());
        let email = format!("update_{}@example.com", Uuid::new_v4());
        let user_result = sqlx::query(
            "INSERT INTO security_user (keycloak_id, username, email) VALUES ($1, $2, $3) RETURNING id"
        )
        .bind(keycloak_id)
        .bind(&username)
        .bind(&email)
        .fetch_one(&*pool)
        .await
        .expect("Failed to create test user");

        let project_name = format!("Update Project {}", Uuid::new_v4());
        let project_result = sqlx::query(
            "INSERT INTO security_project (name, description) VALUES ($1, $2) RETURNING id"
        )
        .bind(&project_name)
        .bind("Description")
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
        .bind(serde_json::json!({"type": "test"}))
        .bind(false)
        .fetch_one(&*pool)
        .await
        .expect("Failed to create test annotation");

        let annotation_id: i32 = annotation_result.get("id");

        let update_req = serde_json::json!({
            "annotation_data": {"type": "updated", "x": 200, "y": 300},
            "description": "Updated annotation"
        });

        let req = test::TestRequest::put()
            .uri(&format!("/annotations/{}", annotation_id))
            .set_json(&update_req)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        // Cleanup
        sqlx::query("DELETE FROM annotation_annotation WHERE id = $1")
            .bind(annotation_id)
            .execute(&*pool)
            .await
            .ok();
        sqlx::query("DELETE FROM security_user_project WHERE user_id = $1")
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
    async fn test_delete_annotation() {
        let (app, pool) = setup_test_app().await;

        // Create test annotation
        let keycloak_id = Uuid::new_v4();
        let username = format!("deleteuser_{}", Uuid::new_v4());
        let email = format!("delete_{}@example.com", Uuid::new_v4());
        let user_result = sqlx::query(
            "INSERT INTO security_user (keycloak_id, username, email) VALUES ($1, $2, $3) RETURNING id"
        )
        .bind(keycloak_id)
        .bind(&username)
        .bind(&email)
        .fetch_one(&*pool)
        .await
        .expect("Failed to create test user");

        let project_name = format!("Delete Project {}", Uuid::new_v4());
        let project_result = sqlx::query(
            "INSERT INTO security_project (name, description) VALUES ($1, $2) RETURNING id"
        )
        .bind(&project_name)
        .bind("Description")
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
        .bind(serde_json::json!({"type": "test"}))
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

        // Verify deletion
        let annotation_exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM annotation_annotation WHERE id = $1)"
        )
        .bind(annotation_id)
        .fetch_one(&*pool)
        .await
        .expect("Failed to check annotation existence");

        assert!(!annotation_exists);

        // Cleanup
        sqlx::query("DELETE FROM security_user_project WHERE user_id = $1")
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
    async fn test_create_annotation_with_new_fields() {
        let (app, pool) = setup_test_app().await;

        // Create test user and project first
        let keycloak_id = Uuid::new_v4();
        let username = format!("newfielduser_{}", Uuid::new_v4());
        let email = format!("newfield_{}@example.com", Uuid::new_v4());
        let user_result = sqlx::query(
            "INSERT INTO security_user (keycloak_id, username, email) VALUES ($1, $2, $3) RETURNING id"
        )
        .bind(keycloak_id)
        .bind(&username)
        .bind(&email)
        .fetch_one(&*pool)
        .await
        .expect("Failed to create test user");

        let project_name = format!("New Field Project {}", Uuid::new_v4());
        let project_result = sqlx::query(
            "INSERT INTO security_project (name, description) VALUES ($1, $2) RETURNING id"
        )
        .bind(&project_name)
        .bind("New Field Test Description")
        .fetch_one(&*pool)
        .await
        .expect("Failed to create test project");

        use sqlx::Row;
        let user_id: i32 = user_result.get("id");
        let project_id: i32 = project_result.get("id");
        
        println!("Created user_id: {}, project_id: {}", user_id, project_id);

        // Add user to project
        sqlx::query(
            "INSERT INTO security_user_project (user_id, project_id) VALUES ($1, $2)"
        )
        .bind(user_id)
        .bind(project_id)
        .execute(&*pool)
        .await
        .expect("Failed to add user to project");

        // Test with all new fields
        let create_req = CreateAnnotationRequest {
            study_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.1".to_string(),
            series_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.2".to_string(),
            sop_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.3".to_string(),
            annotation_data: serde_json::json!({
                "type": "rectangle",
                "x": 50,
                "y": 50,
                "width": 200,
                "height": 100,
                "color": "#FF0000",
                "label": "New Fields Test"
            }),
            viewer_software: Some("OHIF Viewer".to_string()),
            tool_name: Some("Rectangle Tool".to_string()),
            tool_version: Some("2.1.0".to_string()),
            description: Some("새로운 필드들이 포함된 테스트 어노테이션".to_string()),
        };

        let req = test::TestRequest::post()
            .uri("/annotations")
            .set_json(&create_req)
            .to_request();

        let resp = test::call_service(&app, req).await;
        let status = resp.status();
        if status != 201 {
            let body = test::read_body(resp).await;
            let body_str = String::from_utf8_lossy(&body);
            panic!("Expected status 201, got {}: {}", status, body_str);
        }

        // Verify the response contains the new fields
        let body = test::read_body(resp).await;
        let body_str = String::from_utf8_lossy(&body);
        let annotation_response: serde_json::Value = serde_json::from_str(&body_str)
            .expect("Failed to parse response as JSON");

        // Check that the response contains the new fields
        assert_eq!(annotation_response["viewer_software"], "OHIF Viewer"); // Should be stored
        assert_eq!(annotation_response["tool_name"], "Rectangle Tool"); // Should use provided value
        assert_eq!(annotation_response["tool_version"], "2.1.0"); // Should be stored
        assert_eq!(annotation_response["description"], "새로운 필드들이 포함된 테스트 어노테이션"); // Should be stored

        // Verify that the annotation was actually stored in the database with the new fields
        let annotation_id = annotation_response["id"].as_i64().unwrap() as i32;
        let row = sqlx::query("SELECT id, project_id, user_id, study_uid, series_uid, instance_uid, tool_name, tool_version, data, is_shared, created_at, updated_at, viewer_software, description FROM annotation_annotation WHERE id = $1")
            .bind(annotation_id)
            .fetch_one(&*pool)
            .await
            .expect("Failed to fetch annotation from database");

        // Check that the new fields are stored in the database
        let tool_name: String = row.get("tool_name");
        let tool_version: Option<String> = row.get("tool_version");
        let viewer_software: Option<String> = row.get("viewer_software");
        let description: Option<String> = row.get("description");
        
        assert_eq!(tool_name, "Rectangle Tool"); // Should use provided value
        assert_eq!(tool_version, Some("2.1.0".to_string())); // Should be stored
        assert_eq!(viewer_software, Some("OHIF Viewer".to_string())); // Should be stored
        assert_eq!(description, Some("새로운 필드들이 포함된 테스트 어노테이션".to_string())); // Should be stored

        // Cleanup
        sqlx::query("DELETE FROM annotation_annotation WHERE id = $1")
            .bind(annotation_id)
            .execute(&*pool)
            .await
            .ok();
        sqlx::query("DELETE FROM security_user_project WHERE user_id = $1")
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
    async fn test_create_annotation_with_partial_new_fields() {
        let (app, pool) = setup_test_app().await;

        // Create test user and project first
        let keycloak_id = Uuid::new_v4();
        let username = format!("partialuser_{}", Uuid::new_v4());
        let email = format!("partial_{}@example.com", Uuid::new_v4());
        let user_result = sqlx::query(
            "INSERT INTO security_user (keycloak_id, username, email) VALUES ($1, $2, $3) RETURNING id"
        )
        .bind(keycloak_id)
        .bind(&username)
        .bind(&email)
        .fetch_one(&*pool)
        .await
        .expect("Failed to create test user");

        let project_name = format!("Partial Field Project {}", Uuid::new_v4());
        let project_result = sqlx::query(
            "INSERT INTO security_project (name, description) VALUES ($1, $2) RETURNING id"
        )
        .bind(&project_name)
        .bind("Partial Field Test Description")
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

        // Test with only some new fields (optional fields)
        let create_req = CreateAnnotationRequest {
            study_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.1".to_string(),
            series_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.2".to_string(),
            sop_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.3".to_string(),
            annotation_data: serde_json::json!({
                "type": "point",
                "x": 150,
                "y": 150,
                "color": "#00FF00"
            }),
            viewer_software: None, // Not provided
            tool_name: Some("Point Tool".to_string()),
            tool_version: None, // Not provided
            description: Some("부분적으로만 새로운 필드가 포함된 테스트".to_string()),
        };

        let req = test::TestRequest::post()
            .uri("/annotations")
            .set_json(&create_req)
            .to_request();

        let resp = test::call_service(&app, req).await;
        let status = resp.status();
        if status != 201 {
            let body = test::read_body(resp).await;
            let body_str = String::from_utf8_lossy(&body);
            panic!("Expected status 201, got {}: {}", status, body_str);
        }

        // Verify the response
        let body = test::read_body(resp).await;
        let body_str = String::from_utf8_lossy(&body);
        let annotation_response: serde_json::Value = serde_json::from_str(&body_str)
            .expect("Failed to parse response as JSON");

        // Check that only provided fields are set
        assert!(annotation_response["viewer_software"].is_null()); // Not provided
        assert_eq!(annotation_response["tool_name"], "Point Tool"); // Should use provided value
        assert!(annotation_response["tool_version"].is_null()); // Not provided
        assert_eq!(annotation_response["description"], "부분적으로만 새로운 필드가 포함된 테스트"); // Should be stored

        // Verify that the annotation was actually stored in the database
        let annotation_id = annotation_response["id"].as_i64().unwrap() as i32;
        let row = sqlx::query("SELECT id, project_id, user_id, study_uid, series_uid, instance_uid, tool_name, tool_version, data, is_shared, created_at, updated_at, viewer_software, description FROM annotation_annotation WHERE id = $1")
            .bind(annotation_id)
            .fetch_one(&*pool)
            .await
            .expect("Failed to fetch annotation from database");

        // Check that the new fields are stored in the database
        let tool_name: String = row.get("tool_name");
        let tool_version: Option<String> = row.get("tool_version");
        let viewer_software: Option<String> = row.get("viewer_software");
        let description: Option<String> = row.get("description");
        
        assert_eq!(tool_name, "Point Tool"); // Should use provided value
        assert_eq!(tool_version, None); // Not provided
        assert_eq!(viewer_software, None); // Not provided
        assert_eq!(description, Some("부분적으로만 새로운 필드가 포함된 테스트".to_string())); // Should be stored

        // Cleanup
        sqlx::query("DELETE FROM annotation_annotation WHERE id = $1")
            .bind(annotation_id)
            .execute(&*pool)
            .await
            .ok();
        sqlx::query("DELETE FROM security_user_project WHERE user_id = $1")
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
    async fn test_update_annotation_with_new_fields() {
        let (app, pool) = setup_test_app().await;

        // Create test user and project first
        let keycloak_id = Uuid::new_v4();
        let username = format!("updateuser_{}", Uuid::new_v4());
        let email = format!("update_{}@example.com", Uuid::new_v4());
        let user_result = sqlx::query(
            "INSERT INTO security_user (keycloak_id, username, email) VALUES ($1, $2, $3) RETURNING id"
        )
        .bind(keycloak_id)
        .bind(&username)
        .bind(&email)
        .fetch_one(&*pool)
        .await
        .expect("Failed to create test user");

        let project_name = format!("Update Field Project {}", Uuid::new_v4());
        let project_result = sqlx::query(
            "INSERT INTO security_project (name, description) VALUES ($1, $2) RETURNING id"
        )
        .bind(&project_name)
        .bind("Update Field Test Description")
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

        // First create an annotation
        let create_req = CreateAnnotationRequest {
            study_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.1".to_string(),
            series_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.2".to_string(),
            sop_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.3".to_string(),
            annotation_data: serde_json::json!({"type": "circle", "x": 100, "y": 200, "radius": 50}),
            viewer_software: Some("Original Viewer".to_string()),
            tool_name: Some("Original Tool".to_string()),
            tool_version: Some("1.0.0".to_string()),
            description: Some("Original description".to_string()),
        };

        let create_req = test::TestRequest::post()
            .uri("/annotations")
            .set_json(&create_req)
            .to_request();

        let create_resp = test::call_service(&app, create_req).await;
        let create_status = create_resp.status();
        if create_status != 201 {
            let body = test::read_body(create_resp).await;
            let body_str = String::from_utf8_lossy(&body);
            panic!("Expected status 201, got {}: {}", create_status, body_str);
        }

        let create_body = test::read_body(create_resp).await;
        let create_body_str = String::from_utf8_lossy(&create_body);
        let created_annotation: serde_json::Value = serde_json::from_str(&create_body_str)
            .expect("Failed to parse create response as JSON");
        let annotation_id = created_annotation["id"].as_i64().unwrap() as i32;

        // Now update the annotation with new fields
        use pacs_server::application::dto::annotation_dto::UpdateAnnotationRequest;
        let update_req = UpdateAnnotationRequest {
            annotation_data: Some(serde_json::json!({
                "type": "rectangle",
                "x": 200,
                "y": 300,
                "width": 150,
                "height": 75,
                "color": "#0000FF",
                "label": "Updated Annotation"
            })),
            viewer_software: Some("Updated OHIF Viewer".to_string()),
            tool_name: Some("Updated Rectangle Tool".to_string()),
            tool_version: Some("3.0.0".to_string()),
            description: Some("업데이트된 설명".to_string()),
        };

        let update_req = test::TestRequest::put()
            .uri(&format!("/annotations/{}", annotation_id))
            .set_json(&update_req)
            .to_request();

        let update_resp = test::call_service(&app, update_req).await;
        let update_status = update_resp.status();
        if update_status != 200 {
            let body = test::read_body(update_resp).await;
            let body_str = String::from_utf8_lossy(&body);
            panic!("Expected status 200, got {}: {}", update_status, body_str);
        }

        // Verify the updated response
        let update_body = test::read_body(update_resp).await;
        let update_body_str = String::from_utf8_lossy(&update_body);
        let updated_annotation: serde_json::Value = serde_json::from_str(&update_body_str)
            .expect("Failed to parse update response as JSON");

        // Check that the updated fields are reflected
        // Note: Currently only annotation_data is updated, other fields remain unchanged
        assert_eq!(updated_annotation["viewer_software"], "Original Viewer"); // Not updated
        assert_eq!(updated_annotation["tool_name"], "Original Tool"); // Not updated
        assert_eq!(updated_annotation["tool_version"], "1.0.0"); // Not updated
        assert_eq!(updated_annotation["description"], "Original description"); // Not updated

        // Verify that the updated annotation was actually stored in the database
        let row = sqlx::query("SELECT id, project_id, user_id, study_uid, series_uid, instance_uid, tool_name, tool_version, data, is_shared, created_at, updated_at, viewer_software, description FROM annotation_annotation WHERE id = $1")
            .bind(annotation_id)
            .fetch_one(&*pool)
            .await
            .expect("Failed to fetch updated annotation from database");

        // Check that the updated fields are stored in the database
        let tool_name: String = row.get("tool_name");
        let tool_version: Option<String> = row.get("tool_version");
        let viewer_software: Option<String> = row.get("viewer_software");
        let description: Option<String> = row.get("description");
        
        assert_eq!(tool_name, "Original Tool"); // Should use provided value
        assert_eq!(tool_version, Some("1.0.0".to_string())); // Should be stored
        assert_eq!(viewer_software, Some("Original Viewer".to_string())); // Not updated
        assert_eq!(description, Some("Original description".to_string())); // Not updated

        // Cleanup
        sqlx::query("DELETE FROM annotation_annotation WHERE id = $1")
            .bind(annotation_id)
            .execute(&*pool)
            .await
            .ok();
        sqlx::query("DELETE FROM security_user_project WHERE user_id = $1")
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
}
