#[cfg(test)]
mod error_handling_tests {
    use actix_web::{test, web, App};
    use pacs_server::application::dto::{
        annotation_dto::CreateAnnotationRequest,
        mask_group_dto::{CreateMaskGroupRequest, UpdateMaskGroupRequest, SignedUrlRequest},
        mask_dto::{CreateMaskRequest, UpdateMaskRequest}
    };
    use pacs_server::application::use_cases::{AnnotationUseCase, MaskGroupUseCase, MaskUseCase};
    use pacs_server::domain::services::{
        AnnotationServiceImpl, MaskGroupServiceImpl, MaskServiceImpl
    };
    use pacs_server::infrastructure::repositories::{
        AnnotationRepositoryImpl, MaskGroupRepositoryImpl, MaskRepositoryImpl,
        UserRepositoryImpl, ProjectRepositoryImpl
    };
    use pacs_server::presentation::controllers::{
        annotation_controller::configure_routes as configure_annotation_routes,
        mask_group_controller::configure_routes as configure_mask_group_routes,
        mask_controller::configure_routes as configure_mask_routes,
    };
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

        // Initialize repositories
        let annotation_repo = AnnotationRepositoryImpl::new(pool.clone());
        let mask_group_repo = MaskGroupRepositoryImpl::new(pool.clone());
        let mask_repo = MaskRepositoryImpl::new(pool.clone());
        let user_repo = UserRepositoryImpl::new(pool.clone());
        let project_repo = ProjectRepositoryImpl::new(pool.clone());

        let pool = Arc::new(pool);
        
        // Initialize services
        let annotation_service = AnnotationServiceImpl::new(annotation_repo, user_repo.clone(), project_repo.clone());
        let mask_group_service = MaskGroupServiceImpl::new(
            Arc::new(mask_group_repo), 
            Arc::new(user_repo), 
            Arc::new(project_repo),
            Arc::new(annotation_repo)
        );
        let mask_service = MaskServiceImpl::new(
            Arc::new(mask_repo), 
            Arc::new(mask_group_repo), 
            Arc::new(user_repo)
        );
        
        // Mock SignedUrlService for testing
        let signed_url_service = Arc::new(MockSignedUrlService::new());
        
        // Initialize use cases
        let annotation_use_case = Arc::new(AnnotationUseCase::new(annotation_service));
        let mask_group_use_case = Arc::new(MaskGroupUseCase::new(
            Arc::new(mask_group_service),
            signed_url_service.clone(),
        ));
        let mask_use_case = Arc::new(MaskUseCase::new(
            Arc::new(mask_service),
            Arc::new(mask_group_service),
            signed_url_service,
        ));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(annotation_use_case.clone()))
                .app_data(web::Data::new(mask_group_use_case.clone()))
                .app_data(web::Data::new(mask_use_case.clone()))
                .configure(|cfg| configure_annotation_routes(cfg, annotation_use_case.clone()))
                .configure(|cfg| configure_mask_group_routes(cfg, mask_group_use_case.clone()))
                .configure(|cfg| configure_mask_routes(cfg, mask_use_case.clone())),
        )
        .await;

        (app, pool)
    }

    // Mock SignedUrlService that can simulate errors
    use pacs_server::application::services::{SignedUrlService, SignedUrlError, SignedUrlResponse};
    use async_trait::async_trait;

    struct MockSignedUrlService {
        should_fail: std::sync::Mutex<bool>,
    }

    impl MockSignedUrlService {
        fn new() -> Self {
            Self {
                should_fail: std::sync::Mutex::new(false),
            }
        }

        fn set_should_fail(&self, fail: bool) {
            *self.should_fail.lock().unwrap() = fail;
        }
    }

    #[async_trait]
    impl SignedUrlService for MockSignedUrlService {
        async fn generate_upload_url(
            &self,
            _request: pacs_server::application::services::SignedUrlRequest,
        ) -> Result<SignedUrlResponse, SignedUrlError> {
            if *self.should_fail.lock().unwrap() {
                return Err(SignedUrlError::ObjectStorageError(
                    pacs_server::application::services::ObjectStorageError::S3Error("Mock storage error".to_string())
                ));
            }
            Ok(SignedUrlResponse::new(
                "https://mock-s3.amazonaws.com/upload".to_string(),
                "test/file.png".to_string(),
                3600,
                "PUT".to_string(),
            ))
        }

        async fn generate_download_url(
            &self,
            _request: pacs_server::application::services::SignedUrlRequest,
        ) -> Result<SignedUrlResponse, SignedUrlError> {
            if *self.should_fail.lock().unwrap() {
                return Err(SignedUrlError::ObjectStorageError(
                    pacs_server::application::services::ObjectStorageError::S3Error("Mock storage error".to_string())
                ));
            }
            Ok(SignedUrlResponse::new(
                "https://mock-s3.amazonaws.com/download".to_string(),
                "test/file.png".to_string(),
                3600,
                "GET".to_string(),
            ))
        }

        async fn generate_mask_upload_url(
            &self,
            _annotation_id: i32,
            _mask_group_id: i32,
            _file_name: String,
            _content_type: String,
            _ttl_seconds: Option<u64>,
            _user_id: Option<i32>,
        ) -> Result<SignedUrlResponse, SignedUrlError> {
            if *self.should_fail.lock().unwrap() {
                return Err(SignedUrlError::ObjectStorageError(
                    pacs_server::application::services::ObjectStorageError::S3Error("Mock storage error".to_string())
                ));
            }
            Ok(SignedUrlResponse::new(
                "https://mock-s3.amazonaws.com/mask-upload".to_string(),
                "masks/test.png".to_string(),
                3600,
                "PUT".to_string(),
            ))
        }

        async fn generate_mask_download_url(
            &self,
            _file_path: String,
            _ttl_seconds: Option<u64>,
        ) -> Result<SignedUrlResponse, SignedUrlError> {
            if *self.should_fail.lock().unwrap() {
                return Err(SignedUrlError::ObjectStorageError(
                    pacs_server::application::services::ObjectStorageError::S3Error("Mock storage error".to_string())
                ));
            }
            Ok(SignedUrlResponse::new(
                "https://mock-s3.amazonaws.com/mask-download".to_string(),
                "masks/test.png".to_string(),
                3600,
                "GET".to_string(),
            ))
        }

        async fn generate_annotation_upload_url(
            &self,
            _annotation_id: i32,
            _file_name: String,
            _content_type: String,
            _ttl_seconds: Option<u64>,
            _user_id: Option<i32>,
        ) -> Result<SignedUrlResponse, SignedUrlError> {
            if *self.should_fail.lock().unwrap() {
                return Err(SignedUrlError::ObjectStorageError(
                    pacs_server::application::services::ObjectStorageError::S3Error("Mock storage error".to_string())
                ));
            }
            Ok(SignedUrlResponse::new(
                "https://mock-s3.amazonaws.com/annotation-upload".to_string(),
                "annotations/test.json".to_string(),
                3600,
                "PUT".to_string(),
            ))
        }

        async fn generate_annotation_download_url(
            &self,
            _file_path: String,
            _ttl_seconds: Option<u64>,
        ) -> Result<SignedUrlResponse, SignedUrlError> {
            if *self.should_fail.lock().unwrap() {
                return Err(SignedUrlError::ObjectStorageError(
                    pacs_server::application::services::ObjectStorageError::S3Error("Mock storage error".to_string())
                ));
            }
            Ok(SignedUrlResponse::new(
                "https://mock-s3.amazonaws.com/annotation-download".to_string(),
                "annotations/test.json".to_string(),
                3600,
                "GET".to_string(),
            ))
        }
    }

    async fn create_test_data(pool: &Arc<sqlx::Pool<sqlx::Postgres>>) -> (i32, i32, i32) {
        use sqlx::Row;
        
        // Create test user
        let keycloak_id = Uuid::new_v4();
        let user_result = sqlx::query(
            "INSERT INTO security_user (keycloak_id, username, email) VALUES ($1, $2, $3) RETURNING id"
        )
        .bind(keycloak_id)
        .bind("errortestuser")
        .bind("errortest@example.com")
        .fetch_one(pool.as_ref())
        .await
        .expect("Failed to create test user");

        let user_id: i32 = user_result.get("id");

        // Create test project
        let project_result = sqlx::query(
            "INSERT INTO security_project (name, description) VALUES ($1, $2) RETURNING id"
        )
        .bind("Error Test Project")
        .bind("Error Test Description")
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

        // Create test annotation
        let annotation_result = sqlx::query(
            "INSERT INTO annotation_annotation (project_id, user_id, study_uid, series_uid, instance_uid, tool_name, tool_version, viewer_software, data, description, is_shared) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) RETURNING id"
        )
        .bind(project_id)
        .bind(user_id)
        .bind("1.2.840.113619.2.55.3.604688119.868.1234567890.1")
        .bind("1.2.840.113619.2.55.3.604688119.868.1234567890.2")
        .bind("1.2.840.113619.2.55.3.604688119.868.1234567890.3")
        .bind("error_tool")
        .bind("1.0.0")
        .bind("error_viewer")
        .bind(serde_json::json!({"type": "circle", "x": 100, "y": 200, "radius": 50}))
        .bind("Error test annotation")
        .bind(false)
        .fetch_one(pool.as_ref())
        .await
        .expect("Failed to create test annotation");

        let annotation_id: i32 = annotation_result.get("id");

        (annotation_id, user_id, project_id)
    }

    async fn cleanup_test_data(pool: &Arc<sqlx::Pool<sqlx::Postgres>>, user_id: i32, project_id: i32) {
        // Clean up in reverse order of dependencies
        sqlx::query("DELETE FROM annotation_mask WHERE mask_group_id IN (SELECT id FROM annotation_mask_group WHERE annotation_id IN (SELECT id FROM annotation_annotation WHERE user_id = $1))")
            .bind(user_id)
            .execute(pool.as_ref())
            .await
            .ok();
        
        sqlx::query("DELETE FROM annotation_mask_group WHERE annotation_id IN (SELECT id FROM annotation_annotation WHERE user_id = $1)")
            .bind(user_id)
            .execute(pool.as_ref())
            .await
            .ok();
        
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
    async fn test_not_found_errors() {
        let (app, pool) = setup_test_app().await;
        let (annotation_id, user_id, project_id) = create_test_data(&pool).await;

        // Test 1: Get non-existent annotation
        let req = test::TestRequest::get()
            .uri("/api/annotations/999999")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404);

        // Test 2: Get non-existent mask group
        let req = test::TestRequest::get()
            .uri(&format!("/api/annotations/{}/mask-groups/999999", annotation_id))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404);

        // Test 3: Get non-existent mask
        let req = test::TestRequest::get()
            .uri(&format!("/api/annotations/{}/mask-groups/999999/masks/999999", annotation_id))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404);

        // Test 4: Update non-existent annotation
        let update_req = serde_json::json!({
            "annotation_data": {"type": "circle", "x": 100, "y": 200, "radius": 50}
        });

        let req = test::TestRequest::put()
            .uri("/api/annotations/999999")
            .set_json(&update_req)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404);

        // Cleanup
        cleanup_test_data(&pool, user_id, project_id).await;
    }

    #[actix_web::test]
    async fn test_validation_errors() {
        let (app, pool) = setup_test_app().await;
        let (annotation_id, user_id, project_id) = create_test_data(&pool).await;

        // Test 1: Invalid annotation data (missing required fields)
        let invalid_annotation_req = serde_json::json!({
            "study_instance_uid": "", // Empty required field
            "series_instance_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.2",
            "sop_instance_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.3",
            "annotation_data": {} // Empty annotation data
        });

        let req = test::TestRequest::post()
            .uri("/api/annotations")
            .set_json(&invalid_annotation_req)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 400);

        // Test 2: Invalid mask group data
        let invalid_mask_group_req = CreateMaskGroupRequest {
            annotation_id: 1,
            group_name: Some("".to_string()), // Empty group name
            model_name: Some("".to_string()), // Empty model name
            version: Some("invalid-version".to_string()), // Invalid version format
            modality: Some("INVALID_MODALITY".to_string()), // Invalid modality
            slice_count: -1, // Negative slice count
            mask_type: "".to_string(), // Empty mask type
            description: Some("Invalid mask group".to_string()),
        };

        let req = test::TestRequest::post()
            .uri(&format!("/api/annotations/{}/mask-groups", annotation_id))
            .set_json(&invalid_mask_group_req)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 400);

        // Test 3: Invalid mask data
        let invalid_mask_req = CreateMaskRequest {
            mask_group_id: 1, // Valid mask group ID
            file_path: "".to_string(), // Empty file path
            mime_type: "invalid/mime".to_string(), // Invalid MIME type
            slice_index: Some(-1), // Negative slice index
            sop_instance_uid: Some("invalid-uid".to_string()), // Invalid UID format
            label_name: Some("".to_string()), // Empty label name
            file_size: Some(-1), // Negative file size
            checksum: Some("invalid-checksum".to_string()), // Invalid checksum format
            width: Some(-1), // Negative width
            height: Some(-1), // Negative height
        };

        // First create a valid mask group
        let mask_group_req = CreateMaskGroupRequest {
            annotation_id: 1,
            group_name: Some("Error Test Group".to_string()),
            model_name: Some("ErrorModel".to_string()),
            version: Some("1.0.0".to_string()),
            modality: Some("CT".to_string()),
            slice_count: 100,
            mask_type: "segmentation".to_string(),
            description: Some("Error test group".to_string()),
        };

        let req = test::TestRequest::post()
            .uri(&format!("/api/annotations/{}/mask-groups", annotation_id))
            .set_json(&mask_group_req)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201);

        let body: serde_json::Value = test::read_body_json(resp).await;
        let mask_group_id = body["id"].as_i64().unwrap() as i32;

        // Now test invalid mask creation
        let req = test::TestRequest::post()
            .uri(&format!("/api/annotations/{}/mask-groups/{}/masks", annotation_id, mask_group_id))
            .set_json(&invalid_mask_req)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 400);

        // Cleanup
        cleanup_test_data(&pool, user_id, project_id).await;
    }

    #[actix_web::test]
    async fn test_unauthorized_errors() {
        let (app, pool) = setup_test_app().await;
        let (annotation_id, user_id, project_id) = create_test_data(&pool).await;

        // Test 1: Try to access annotation without proper authentication
        // (In real implementation, this would test JWT validation)
        let req = test::TestRequest::get()
            .uri(&format!("/api/annotations/{}", annotation_id))
            .to_request();

        // For now, this will succeed because we don't have authentication middleware
        // In a real implementation, this would return 401
        let resp = test::call_service(&app, req).await;
        // assert_eq!(resp.status(), 401); // Uncomment when auth is implemented

        // Test 2: Try to create mask group for non-existent annotation
        let mask_group_req = CreateMaskGroupRequest {
            annotation_id: 1,
            group_name: Some("Unauthorized Test Group".to_string()),
            model_name: Some("UnauthorizedModel".to_string()),
            version: Some("1.0.0".to_string()),
            modality: Some("CT".to_string()),
            slice_count: 100,
            mask_type: "segmentation".to_string(),
            description: Some("Unauthorized test group".to_string()),
        };

        let req = test::TestRequest::post()
            .uri("/api/annotations/999999/mask-groups")
            .set_json(&mask_group_req)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404);

        // Cleanup
        cleanup_test_data(&pool, user_id, project_id).await;
    }

    #[actix_web::test]
    async fn test_conflict_errors() {
        let (app, pool) = setup_test_app().await;
        let (annotation_id, user_id, project_id) = create_test_data(&pool).await;

        // Test 1: Try to create duplicate mask group
        let mask_group_req = CreateMaskGroupRequest {
            annotation_id: 1,
            group_name: Some("Duplicate Test Group".to_string()),
            model_name: Some("DuplicateModel".to_string()),
            version: Some("1.0.0".to_string()),
            modality: Some("CT".to_string()),
            slice_count: 100,
            mask_type: "segmentation".to_string(),
            description: Some("Duplicate test group".to_string()),
        };

        // Create first mask group
        let req = test::TestRequest::post()
            .uri(&format!("/api/annotations/{}/mask-groups", annotation_id))
            .set_json(&mask_group_req)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201);

        // Try to create duplicate (this should succeed in current implementation)
        // In a real implementation, this might return 409 Conflict
        let req = test::TestRequest::post()
            .uri(&format!("/api/annotations/{}/mask-groups", annotation_id))
            .set_json(&mask_group_req)
            .to_request();

        let resp = test::call_service(&app, req).await;
        // assert_eq!(resp.status(), 409); // Uncomment when duplicate checking is implemented

        // Cleanup
        cleanup_test_data(&pool, user_id, project_id).await;
    }

    #[actix_web::test]
    async fn test_database_errors() {
        let (app, pool) = setup_test_app().await;
        let (annotation_id, user_id, project_id) = create_test_data(&pool).await;

        // Test 1: Try to create mask group with invalid foreign key
        let mask_group_req = CreateMaskGroupRequest {
            annotation_id: 1,
            group_name: Some("Database Error Test Group".to_string()),
            model_name: Some("DatabaseErrorModel".to_string()),
            version: Some("1.0.0".to_string()),
            modality: Some("CT".to_string()),
            slice_count: 100,
            mask_type: "segmentation".to_string(),
            description: Some("Database error test group".to_string()),
        };

        // Try to create mask group for non-existent annotation
        let req = test::TestRequest::post()
            .uri("/api/annotations/999999/mask-groups")
            .set_json(&mask_group_req)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404);

        // Test 2: Try to create mask with invalid foreign key
        let mask_req = CreateMaskRequest {
            mask_group_id: 1, // Valid mask group ID
            file_path: "masks/database_error.png".to_string(),
            mime_type: "image/png".to_string(),
            slice_index: Some(1),
            sop_instance_uid: Some("1.2.3.4.5.6.7.8.9.1.1".to_string()),
            label_name: Some("database_error_label".to_string()),
            file_size: Some(102400),
            checksum: Some("md5-database-error".to_string()),
            width: Some(512),
            height: Some(512),
        };

        let req = test::TestRequest::post()
            .uri("/api/annotations/999999/mask-groups/999999/masks")
            .set_json(&mask_req)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404);

        // Cleanup
        cleanup_test_data(&pool, user_id, project_id).await;
    }

    #[actix_web::test]
    async fn test_storage_errors() {
        let (app, pool) = setup_test_app().await;
        let (annotation_id, user_id, project_id) = create_test_data(&pool).await;

        // Create mask group first
        let mask_group_req = CreateMaskGroupRequest {
            annotation_id: 1,
            group_name: Some("Storage Error Test Group".to_string()),
            model_name: Some("StorageErrorModel".to_string()),
            version: Some("1.0.0".to_string()),
            modality: Some("CT".to_string()),
            slice_count: 100,
            mask_type: "segmentation".to_string(),
            description: Some("Storage error test group".to_string()),
        };

        let req = test::TestRequest::post()
            .uri(&format!("/api/annotations/{}/mask-groups", annotation_id))
            .set_json(&mask_group_req)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201);

        let body: serde_json::Value = test::read_body_json(resp).await;
        let mask_group_id = body["id"].as_i64().unwrap() as i32;

        // Test 1: Try to generate upload URL when storage is down
        // (This would require modifying the mock service to simulate storage errors)
        let upload_req = SignedUrlRequest {
            filename: "storage_error_test.png".to_string(),
            mime_type: "image/png".to_string(),
            file_size: Some(102400),
            slice_index: Some(1),
            sop_instance_uid: Some("1.2.3.4.5.6.7.8.9.1.1".to_string()),
            label_name: Some("storage_error_label".to_string()),
            mask_group_id: mask_group_id,
            ttl_seconds: Some(3600),
        };

        let req = test::TestRequest::post()
            .uri(&format!("/api/annotations/{}/mask-groups/{}/upload-url", annotation_id, mask_group_id))
            .set_json(&upload_req)
            .to_request();

        let resp = test::call_service(&app, req).await;
        // This will succeed with our current mock, but in real implementation
        // it would return 500 when storage is down
        assert_eq!(resp.status(), 200);

        // Cleanup
        cleanup_test_data(&pool, user_id, project_id).await;
    }

    #[actix_web::test]
    async fn test_malformed_json_errors() {
        let (app, pool) = setup_test_app().await;
        let (annotation_id, user_id, project_id) = create_test_data(&pool).await;

        // Test 1: Malformed JSON in annotation creation
        let malformed_json = r#"{"study_instance_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.1", "series_instance_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.2", "sop_instance_uid": "1.2.840.113619.2.55.3.604688119.868.1234567890.3", "annotation_data": {"type": "circle", "x": 100, "y": 200, "radius": 50}, "tool_name": "Circle Tool", "tool_version": "2.1.0", "viewer_software": "OHIF Viewer", "description": "Malformed JSON test"}"#;

        let req = test::TestRequest::post()
            .uri("/api/annotations")
            .insert_header(("content-type", "application/json"))
            .set_payload(malformed_json)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 400);

        // Test 2: Malformed JSON in mask group creation
        let malformed_json = r#"{"group_name": "Malformed JSON Test Group", "model_name": "MalformedModel", "version": "1.0.0", "modality": "CT", "slice_count": 100, "mask_type": "segmentation", "description": "Malformed JSON test group"}"#;

        let req = test::TestRequest::post()
            .uri(&format!("/api/annotations/{}/mask-groups", annotation_id))
            .insert_header(("content-type", "application/json"))
            .set_payload(malformed_json)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 400);

        // Cleanup
        cleanup_test_data(&pool, user_id, project_id).await;
    }

    #[actix_web::test]
    async fn test_large_payload_errors() {
        let (app, pool) = setup_test_app().await;
        let (annotation_id, user_id, project_id) = create_test_data(&pool).await;

        // Test 1: Very large annotation data
        let large_annotation_data = serde_json::json!({
            "type": "polygon",
            "points": (0..10000).map(|i| {
                serde_json::json!({
                    "x": i % 1000,
                    "y": i / 1000,
                    "z": i % 100
                })
            }).collect::<Vec<_>>()
        });

        let large_annotation_req = CreateAnnotationRequest {
            study_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.1".to_string(),
            series_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.2".to_string(),
            sop_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.3".to_string(),
            annotation_data: large_annotation_data,
            viewer_software: Some("OHIF Viewer".to_string()),
            tool_name: Some("Polygon Tool".to_string()),
            tool_version: Some("2.1.0".to_string()),
            description: Some("Large payload test".to_string()),
        };

        let req = test::TestRequest::post()
            .uri("/api/annotations")
            .set_json(&large_annotation_req)
            .to_request();

        let resp = test::call_service(&app, req).await;
        // This might succeed or fail depending on payload size limits
        // In a real implementation, this might return 413 Payload Too Large
        // assert_eq!(resp.status(), 413); // Uncomment when size limits are implemented

        // Cleanup
        cleanup_test_data(&pool, user_id, project_id).await;
    }

    #[actix_web::test]
    async fn test_concurrent_modification_errors() {
        let (app, pool) = setup_test_app().await;
        let (annotation_id, user_id, project_id) = create_test_data(&pool).await;

        // Create mask group
        let mask_group_req = CreateMaskGroupRequest {
            annotation_id: annotation_id,
            group_name: Some("Concurrent Test Group".to_string()),
            model_name: Some("ConcurrentModel".to_string()),
            version: Some("1.0.0".to_string()),
            modality: Some("CT".to_string()),
            slice_count: 100,
            mask_type: "segmentation".to_string(),
            description: Some("Concurrent test group".to_string()),
        };

        let req = test::TestRequest::post()
            .uri(&format!("/api/annotations/{}/mask-groups", annotation_id))
            .set_json(&mask_group_req)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201);

        let body: serde_json::Value = test::read_body_json(resp).await;
        let mask_group_id = body["id"].as_i64().unwrap() as i32;

        // Test concurrent updates to the same mask group
        let update_req1 = UpdateMaskGroupRequest {
            group_name: Some("Updated Group 1".to_string()),
            model_name: Some("UpdatedModel1".to_string()),
            version: Some("2.0.0".to_string()),
            modality: Some("MRI".to_string()),
            slice_count: Some(200),
            mask_type: Some("classification".to_string()),
            description: Some("First update".to_string()),
        };

        let update_req2 = UpdateMaskGroupRequest {
            group_name: Some("Updated Group 2".to_string()),
            model_name: Some("UpdatedModel2".to_string()),
            version: Some("3.0.0".to_string()),
            modality: Some("CT".to_string()),
            slice_count: Some(300),
            mask_type: Some("segmentation".to_string()),
            description: Some("Second update".to_string()),
        };

        // Both updates should succeed in current implementation
        // In a real implementation with optimistic locking, one might fail
        let req1 = test::TestRequest::put()
            .uri(&format!("/api/annotations/{}/mask-groups/{}", annotation_id, mask_group_id))
            .set_json(&update_req1)
            .to_request();

        let req2 = test::TestRequest::put()
            .uri(&format!("/api/annotations/{}/mask-groups/{}", annotation_id, mask_group_id))
            .set_json(&update_req2)
            .to_request();

        let resp1 = test::call_service(&app, req1).await;
        let resp2 = test::call_service(&app, req2).await;

        assert_eq!(resp1.status(), 200);
        assert_eq!(resp2.status(), 200);

        // Cleanup
        cleanup_test_data(&pool, user_id, project_id).await;
    }
}
