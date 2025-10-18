#[cfg(test)]
mod annotation_controller_tests {
    use actix_web::{test, web, App};
    use pacs_server::application::dto::annotation_dto::CreateAnnotationRequest;
    use pacs_server::application::use_cases::AnnotationUseCase;
    use pacs_server::domain::services::AnnotationServiceImpl;
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

    async fn create_test_data(pool: &Arc<sqlx::Pool<sqlx::Postgres>>) -> (i32, i32) {
        use sqlx::Row;
        
        // Create test user with unique username and email
        let keycloak_id = Uuid::new_v4();
        let unique_suffix = keycloak_id.to_string()[..8].to_string();
        let username = format!("testuser_{}", unique_suffix);
        let email = format!("test_{}@example.com", unique_suffix);
        
        let user_result = sqlx::query(
            "INSERT INTO security_user (keycloak_id, username, email) VALUES ($1, $2, $3) RETURNING id"
        )
        .bind(keycloak_id)
        .bind(&username)
        .bind(&email)
        .fetch_one(pool.as_ref())
        .await
        .expect("Failed to create test user");

        let user_id: i32 = user_result.get("id");

        // Create test project with unique name
        let project_name = format!("Test Project {}", unique_suffix);
        let project_result = sqlx::query(
            "INSERT INTO security_project (name, description) VALUES ($1, $2) RETURNING id"
        )
        .bind(&project_name)
        .bind("Test Description")
        .fetch_one(pool.as_ref())
        .await
        .expect("Failed to create test project");

        let project_id: i32 = project_result.get("id");

        // Add user to project
        sqlx::query(
            "INSERT INTO security_user_project (user_id, project_id) VALUES ($1, $2)"
        )
        .bind(user_id)
        .bind(project_id)
        .execute(pool.as_ref())
        .await
        .expect("Failed to add user to project");

        (user_id, project_id)
    }

    async fn cleanup_test_data(pool: &Arc<sqlx::Pool<sqlx::Postgres>>, user_id: i32, project_id: i32) {
        // Clean up in reverse order of dependencies
        sqlx::query("DELETE FROM annotation_annotation WHERE user_id = $1")
            .bind(user_id)
            .execute(pool.as_ref())
            .await
            .ok();
        
        sqlx::query("DELETE FROM security_user_project WHERE user_id = $1")
            .bind(user_id)
            .execute(pool.as_ref())
            .await
            .ok();
        
        sqlx::query("DELETE FROM security_user WHERE id = $1")
            .bind(user_id)
            .execute(pool.as_ref())
            .await
            .ok();
        
        sqlx::query("DELETE FROM security_project WHERE id = $1")
            .bind(project_id)
            .execute(pool.as_ref())
            .await
            .ok();
    }

    #[actix_web::test]
    async fn test_create_annotation_use_case() {
        let (_, pool) = setup_test_app().await;
        let (user_id, project_id) = create_test_data(&pool).await;

        // Test the use case directly
        let annotation_repo = AnnotationRepositoryImpl::new(pool.as_ref().clone());
        let user_repo = UserRepositoryImpl::new(pool.as_ref().clone());
        let project_repo = ProjectRepositoryImpl::new(pool.as_ref().clone());
        let annotation_service = AnnotationServiceImpl::new(annotation_repo, user_repo, project_repo);
        let annotation_use_case = AnnotationUseCase::new(annotation_service);

        let create_req = CreateAnnotationRequest {
            user_id: Some(user_id),
            project_id: Some(project_id),
            study_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.1".to_string(),
            series_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.2".to_string(),
            sop_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.3".to_string(),
            annotation_data: serde_json::json!({"type": "circle", "x": 100, "y": 200, "radius": 50}),
            viewer_software: Some("OHIF Viewer".to_string()),
            tool_name: Some("Circle Tool".to_string()),
            tool_version: Some("2.1.0".to_string()),
            description: Some("Test annotation".to_string()),
        };

        let result = annotation_use_case.create_annotation(create_req, user_id, project_id).await;
        assert!(result.is_ok());

        // Cleanup
        cleanup_test_data(&pool, user_id, project_id).await;
    }

    #[actix_web::test]
    async fn test_create_annotation_with_new_fields_use_case() {
        let (_, pool) = setup_test_app().await;
        let (user_id, project_id) = create_test_data(&pool).await;

        // Test the use case directly
        let annotation_repo = AnnotationRepositoryImpl::new(pool.as_ref().clone());
        let user_repo = UserRepositoryImpl::new(pool.as_ref().clone());
        let project_repo = ProjectRepositoryImpl::new(pool.as_ref().clone());
        let annotation_service = AnnotationServiceImpl::new(annotation_repo, user_repo, project_repo);
        let annotation_use_case = AnnotationUseCase::new(annotation_service);

        let create_req = CreateAnnotationRequest {
            user_id: Some(user_id),
            project_id: Some(project_id),
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

        let result = annotation_use_case.create_annotation(create_req, user_id, project_id).await;
        assert!(result.is_ok());

        // Cleanup
        cleanup_test_data(&pool, user_id, project_id).await;
    }

    #[actix_web::test]
    async fn test_create_annotation_with_partial_new_fields_use_case() {
        let (_, pool) = setup_test_app().await;
        let (user_id, project_id) = create_test_data(&pool).await;

        // Test the use case directly
        let annotation_repo = AnnotationRepositoryImpl::new(pool.as_ref().clone());
        let user_repo = UserRepositoryImpl::new(pool.as_ref().clone());
        let project_repo = ProjectRepositoryImpl::new(pool.as_ref().clone());
        let annotation_service = AnnotationServiceImpl::new(annotation_repo, user_repo, project_repo);
        let annotation_use_case = AnnotationUseCase::new(annotation_service);

        let create_req = CreateAnnotationRequest {
            user_id: Some(user_id),
            project_id: Some(project_id),
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

        let result = annotation_use_case.create_annotation(create_req, user_id, project_id).await;
        assert!(result.is_ok());

        // Cleanup
        cleanup_test_data(&pool, user_id, project_id).await;
    }

    #[actix_web::test]
    async fn test_update_annotation_with_new_fields_use_case() {
        let (_, pool) = setup_test_app().await;
        let (user_id, project_id) = create_test_data(&pool).await;

        // Test the use case directly
        let annotation_repo = AnnotationRepositoryImpl::new(pool.as_ref().clone());
        let user_repo = UserRepositoryImpl::new(pool.as_ref().clone());
        let project_repo = ProjectRepositoryImpl::new(pool.as_ref().clone());
        let annotation_service = AnnotationServiceImpl::new(annotation_repo, user_repo, project_repo);
        let annotation_use_case = AnnotationUseCase::new(annotation_service);

        // First create an annotation
        let create_req = CreateAnnotationRequest {
            user_id: Some(user_id),
            project_id: Some(project_id),
            study_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.1".to_string(),
            series_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.2".to_string(),
            sop_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.3".to_string(),
            annotation_data: serde_json::json!({"type": "circle", "x": 100, "y": 200, "radius": 50}),
            viewer_software: Some("Original Viewer".to_string()),
            tool_name: Some("Original Tool".to_string()),
            tool_version: Some("1.0.0".to_string()),
            description: Some("Original description".to_string()),
        };

        let create_result = annotation_use_case.create_annotation(create_req, user_id, project_id).await;
        assert!(create_result.is_ok());

        let annotation = create_result.unwrap();
        let annotation_id = annotation.id;

        // Now update the annotation
        use pacs_server::application::dto::annotation_dto::UpdateAnnotationRequest;
        let update_req = UpdateAnnotationRequest {
            annotation_data: Some(serde_json::json!({
                "type": "rectangle",
                "x": 200,
                "y": 300,
                "width": 150,
                "height": 100,
                "color": "#00FF00",
                "label": "Updated Annotation"
            })),
            viewer_software: Some("Updated Viewer".to_string()),
            tool_name: Some("Updated Tool".to_string()),
            tool_version: Some("2.0.0".to_string()),
            description: Some("Updated description with new fields".to_string()),
        };

        let update_result = annotation_use_case.update_annotation(annotation_id, update_req).await;
        assert!(update_result.is_ok());

        // Cleanup
        cleanup_test_data(&pool, user_id, project_id).await;
    }

    #[tokio::test]
    async fn test_list_annotations_with_viewer_software_filter() {
        let (app, pool) = setup_test_app().await;
        
        // 테스트 데이터 생성
        let (user_id, project_id) = create_test_data(&pool).await;
        
        let annotation1 = CreateAnnotationRequest {
            user_id: Some(user_id),
            project_id: Some(project_id),
            study_instance_uid: "1.2.3.4.5".to_string(),
            series_instance_uid: "1.2.3.4.6".to_string(),
            sop_instance_uid: "1.2.3.4.7".to_string(),
            annotation_data: serde_json::json!({"type": "test1"}),
            description: Some("Test annotation 1".to_string()),
            tool_name: Some("test_tool".to_string()),
            tool_version: Some("1.0.0".to_string()),
            viewer_software: Some("OHIF Viewer".to_string()),
        };

        let annotation2 = CreateAnnotationRequest {
            user_id: Some(user_id),
            project_id: Some(project_id),
            study_instance_uid: "1.2.3.4.8".to_string(),
            series_instance_uid: "1.2.3.4.9".to_string(),
            sop_instance_uid: "1.2.3.4.10".to_string(),
            annotation_data: serde_json::json!({"type": "test2"}),
            description: Some("Test annotation 2".to_string()),
            tool_name: Some("test_tool".to_string()),
            tool_version: Some("1.0.0".to_string()),
            viewer_software: Some("DICOM Viewer".to_string()),
        };

        // 어노테이션들 생성
        let req1 = test::TestRequest::post()
            .uri("/api/annotations")
            .set_json(&annotation1)
            .to_request();
        let resp1 = test::call_service(&app, req1).await;
        assert_eq!(resp1.status(), 201);

        let req2 = test::TestRequest::post()
            .uri("/api/annotations")
            .set_json(&annotation2)
            .to_request();
        let resp2 = test::call_service(&app, req2).await;
        assert_eq!(resp2.status(), 201);

        // OHIF Viewer로 필터링 테스트
        let req = test::TestRequest::get()
            .uri("/api/annotations?viewer_software=OHIF%20Viewer")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["total"], 1);
        assert_eq!(body["annotations"][0]["viewer_software"], "OHIF Viewer");

        // DICOM Viewer로 필터링 테스트
        let req = test::TestRequest::get()
            .uri("/api/annotations?viewer_software=DICOM%20Viewer")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["total"], 1);
        assert_eq!(body["annotations"][0]["viewer_software"], "DICOM Viewer");

        // 필터 없이 모든 어노테이션 조회 테스트
        let req = test::TestRequest::get()
            .uri("/api/annotations")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["total"], 2);

        // Cleanup
        cleanup_test_data(&pool, user_id, project_id).await;
    }

    #[tokio::test]
    async fn test_list_annotations_with_project_and_viewer_filter() {
        let (app, pool) = setup_test_app().await;
        
        // 테스트 데이터 생성
        let (user_id, project_id) = create_test_data(&pool).await;
        
        // 테스트 데이터 생성
        let annotation1 = CreateAnnotationRequest {
            user_id: Some(user_id),
            project_id: Some(project_id),
            study_instance_uid: "1.2.3.4.5".to_string(),
            series_instance_uid: "1.2.3.4.6".to_string(),
            sop_instance_uid: "1.2.3.4.7".to_string(),
            annotation_data: serde_json::json!({"type": "test1"}),
            description: Some("Test annotation 1".to_string()),
            tool_name: Some("test_tool".to_string()),
            tool_version: Some("1.0.0".to_string()),
            viewer_software: Some("OHIF Viewer".to_string()),
        };

        let annotation2 = CreateAnnotationRequest {
            user_id: Some(user_id),
            project_id: Some(project_id),
            study_instance_uid: "1.2.3.4.8".to_string(),
            series_instance_uid: "1.2.3.4.9".to_string(),
            sop_instance_uid: "1.2.3.4.10".to_string(),
            annotation_data: serde_json::json!({"type": "test2"}),
            description: Some("Test annotation 2".to_string()),
            tool_name: Some("test_tool".to_string()),
            tool_version: Some("1.0.0".to_string()),
            viewer_software: Some("DICOM Viewer".to_string()),
        };

        // 어노테이션들 생성
        let req1 = test::TestRequest::post()
            .uri("/api/annotations")
            .set_json(&annotation1)
            .to_request();
        let resp1 = test::call_service(&app, req1).await;
        assert_eq!(resp1.status(), 201);

        let req2 = test::TestRequest::post()
            .uri("/api/annotations")
            .set_json(&annotation2)
            .to_request();
        let resp2 = test::call_service(&app, req2).await;
        assert_eq!(resp2.status(), 201);

        // 프로젝트 ID와 OHIF Viewer로 필터링 테스트
        let req = test::TestRequest::get()
            .uri(&format!("/api/annotations?project_id={}&viewer_software=OHIF%20Viewer", project_id))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["total"], 1);
        assert_eq!(body["annotations"][0]["viewer_software"], "OHIF Viewer");

        // 프로젝트 ID와 DICOM Viewer로 필터링 테스트
        let req = test::TestRequest::get()
            .uri(&format!("/api/annotations?project_id={}&viewer_software=DICOM%20Viewer", project_id))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["total"], 1);
        assert_eq!(body["annotations"][0]["viewer_software"], "DICOM Viewer");

        // Cleanup
        cleanup_test_data(&pool, user_id, project_id).await;
    }

    #[tokio::test]
    async fn test_list_annotations_with_study_and_viewer_filter() {
        let (app, pool) = setup_test_app().await;
        
        // 테스트 데이터 생성
        let (user_id, project_id) = create_test_data(&pool).await;
        let study_uid = "1.2.3.4.5";
        
        // 테스트 데이터 생성
        let annotation1 = CreateAnnotationRequest {
            user_id: Some(user_id),
            project_id: Some(project_id),
            study_instance_uid: study_uid.to_string(),
            series_instance_uid: "1.2.3.4.6".to_string(),
            sop_instance_uid: "1.2.3.4.7".to_string(),
            annotation_data: serde_json::json!({"type": "test1"}),
            description: Some("Test annotation 1".to_string()),
            tool_name: Some("test_tool".to_string()),
            tool_version: Some("1.0.0".to_string()),
            viewer_software: Some("OHIF Viewer".to_string()),
        };

        let annotation2 = CreateAnnotationRequest {
            user_id: Some(user_id),
            project_id: Some(project_id),
            study_instance_uid: study_uid.to_string(),
            series_instance_uid: "1.2.3.4.8".to_string(),
            sop_instance_uid: "1.2.3.4.9".to_string(),
            annotation_data: serde_json::json!({"type": "test2"}),
            description: Some("Test annotation 2".to_string()),
            tool_name: Some("test_tool".to_string()),
            tool_version: Some("1.0.0".to_string()),
            viewer_software: Some("DICOM Viewer".to_string()),
        };

        // 어노테이션들 생성
        let req1 = test::TestRequest::post()
            .uri("/api/annotations")
            .set_json(&annotation1)
            .to_request();
        let resp1 = test::call_service(&app, req1).await;
        assert_eq!(resp1.status(), 201);

        let req2 = test::TestRequest::post()
            .uri("/api/annotations")
            .set_json(&annotation2)
            .to_request();
        let resp2 = test::call_service(&app, req2).await;
        assert_eq!(resp2.status(), 201);

        // Study UID와 OHIF Viewer로 필터링 테스트
        let req = test::TestRequest::get()
            .uri(&format!("/api/annotations?study_instance_uid={}&viewer_software=OHIF%20Viewer", study_uid))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["total"], 1);
        assert_eq!(body["annotations"][0]["viewer_software"], "OHIF Viewer");

        // Study UID와 DICOM Viewer로 필터링 테스트
        let req = test::TestRequest::get()
            .uri(&format!("/api/annotations?study_instance_uid={}&viewer_software=DICOM%20Viewer", study_uid))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["total"], 1);
        assert_eq!(body["annotations"][0]["viewer_software"], "DICOM Viewer");

        // Cleanup
        cleanup_test_data(&pool, user_id, project_id).await;
    }

    #[tokio::test]
    async fn test_list_annotations_with_nonexistent_viewer_filter() {
        let (app, pool) = setup_test_app().await;
        
        // 테스트 데이터 생성
        let (user_id, project_id) = create_test_data(&pool).await;
        
        // 테스트 데이터 생성
        let annotation = CreateAnnotationRequest {
            user_id: Some(user_id),
            project_id: Some(project_id),
            study_instance_uid: "1.2.3.4.5".to_string(),
            series_instance_uid: "1.2.3.4.6".to_string(),
            sop_instance_uid: "1.2.3.4.7".to_string(),
            annotation_data: serde_json::json!({"type": "test"}),
            description: Some("Test annotation".to_string()),
            tool_name: Some("test_tool".to_string()),
            tool_version: Some("1.0.0".to_string()),
            viewer_software: Some("OHIF Viewer".to_string()),
        };

        // 어노테이션 생성
        let req = test::TestRequest::post()
            .uri("/api/annotations")
            .set_json(&annotation)
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201);

        // 존재하지 않는 viewer_software로 필터링 테스트
        let req = test::TestRequest::get()
            .uri("/api/annotations?viewer_software=NonExistent%20Viewer")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["total"], 0);
        assert!(body["annotations"].as_array().unwrap().is_empty());

        // Cleanup
        cleanup_test_data(&pool, user_id, project_id).await;
    }
}
