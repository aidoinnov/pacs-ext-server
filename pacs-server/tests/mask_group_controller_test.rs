#[cfg(test)]
mod mask_group_controller_tests {
    use actix_web::{test, web, App};
    use pacs_server::application::dto::mask_group_dto::{
        CreateMaskGroupRequest, UpdateMaskGroupRequest, SignedUrlRequest, CompleteUploadRequest
    };
    use pacs_server::application::use_cases::MaskGroupUseCase;
    use pacs_server::domain::services::MaskGroupServiceImpl;
    use pacs_server::infrastructure::repositories::{
        MaskGroupRepositoryImpl, AnnotationRepositoryImpl, UserRepositoryImpl, ProjectRepositoryImpl
    };
    use pacs_server::presentation::controllers::mask_group_controller::configure_routes;
    use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use uuid::Uuid;
use sqlx::Row;

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

        let mask_group_repo = MaskGroupRepositoryImpl::new(pool.clone());
        let annotation_repo = AnnotationRepositoryImpl::new(pool.clone());
        let user_repo = UserRepositoryImpl::new(pool.clone());
        let project_repo = ProjectRepositoryImpl::new(pool.clone());

        let pool = Arc::new(pool);
        let mask_group_service = MaskGroupServiceImpl::new(Arc::new(mask_group_repo), Arc::new(annotation_repo), Arc::new(user_repo));
        
        // Mock SignedUrlService for testing
        let signed_url_service = Arc::new(MockSignedUrlService::new());
        
        let mask_group_use_case = Arc::new(MaskGroupUseCase::new(
            Arc::new(mask_group_service),
            signed_url_service,
        ));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(mask_group_use_case.clone()))
                .configure(|cfg| configure_routes(cfg, mask_group_use_case.clone())),
        )
        .await;

        (app, pool)
    }

    // Mock SignedUrlService for testing
    use pacs_server::application::services::{SignedUrlService, SignedUrlError, SignedUrlResponse};
    use async_trait::async_trait;

    struct MockSignedUrlService;

    impl MockSignedUrlService {
        fn new() -> Self {
            Self
        }
    }

    #[async_trait]
    impl SignedUrlService for MockSignedUrlService {
        async fn generate_upload_url(
            &self,
            _request: pacs_server::application::services::SignedUrlRequest,
        ) -> Result<SignedUrlResponse, SignedUrlError> {
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
            Ok(SignedUrlResponse::new(
                "https://mock-s3.amazonaws.com/annotation-download".to_string(),
                "annotations/test.json".to_string(),
                3600,
                "GET".to_string(),
            ))
        }
    }

    async fn create_test_data(pool: &sqlx::Pool<sqlx::Postgres>) -> (i32, i32) {
        // Create test user
        let keycloak_id = Uuid::new_v4();
        let username = format!("testuser_{}", keycloak_id);
        let email = format!("test_{}@example.com", keycloak_id);
        let user_result = sqlx::query(
            "INSERT INTO security_user (keycloak_id, username, email) VALUES ($1, $2, $3) RETURNING id"
        )
        .bind(keycloak_id)
        .bind(&username)
        .bind(&email)
        .fetch_one(pool)
        .await
        .expect("Failed to create test user");

        let user_id: i32 = user_result.get("id");

        // Create test project
        let project_name = format!("Test Project {}", keycloak_id);
        let project_description = format!("Test Description {}", keycloak_id);
        let project_result = sqlx::query(
            "INSERT INTO security_project (name, description) VALUES ($1, $2) RETURNING id"
        )
        .bind(&project_name)
        .bind(&project_description)
        .fetch_one(pool)
        .await
        .expect("Failed to create test project");

        let project_id: i32 = project_result.get("id");

        // Add user to project
        sqlx::query(
            "INSERT INTO security_user_project (user_id, project_id) VALUES ($1, $2)"
        )
        .bind(user_id)
        .bind(project_id)
        .execute(pool)
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
        .bind("test_tool")
        .bind("1.0.0")
        .bind("test_viewer")
        .bind(serde_json::json!({"type": "circle", "x": 100, "y": 200, "radius": 50}))
        .bind("Test annotation")
        .bind(false)
        .fetch_one(pool)
        .await
        .expect("Failed to create test annotation");

        let annotation_id: i32 = annotation_result.get("id");

        (annotation_id, user_id)
    }

    async fn cleanup_test_data(pool: &sqlx::Pool<sqlx::Postgres>, user_id: i32) {
        // Clean up in reverse order of dependencies
        sqlx::query("DELETE FROM annotation_mask WHERE mask_group_id IN (SELECT id FROM annotation_mask_group WHERE annotation_id IN (SELECT id FROM annotation_annotation WHERE user_id = $1))")
            .bind(user_id)
            .execute(pool)
            .await
            .ok();
        
        sqlx::query("DELETE FROM annotation_mask_group WHERE annotation_id IN (SELECT id FROM annotation_annotation WHERE user_id = $1)")
            .bind(user_id)
            .execute(pool)
            .await
            .ok();
        
        sqlx::query("DELETE FROM annotation_annotation WHERE user_id = $1")
            .bind(user_id)
            .execute(pool)
            .await
            .ok();
        
        sqlx::query("DELETE FROM security_user_project WHERE user_id = $1")
            .bind(user_id)
            .execute(pool)
            .await
            .ok();
        
        sqlx::query("DELETE FROM security_user WHERE id = $1")
            .bind(user_id)
            .execute(pool)
            .await
            .ok();
        
        sqlx::query("DELETE FROM security_project WHERE id IN (SELECT project_id FROM annotation_annotation WHERE user_id = $1)")
            .bind(user_id)
            .execute(pool)
            .await
            .ok();
    }

    #[actix_web::test]
    async fn test_create_mask_group_success() {
        let (app, pool) = setup_test_app().await;
        let (annotation_id, user_id) = create_test_data(&pool).await;

        let create_req = CreateMaskGroupRequest {
            annotation_id: annotation_id,
            group_name: Some("Test Mask Group".to_string()),
            model_name: Some("Test Model".to_string()),
            version: Some("1.0.0".to_string()),
            modality: Some("CT".to_string()),
            slice_count: 100,
            mask_type: "segmentation".to_string(),
            description: Some("Test description".to_string()),
        };

        let req = test::TestRequest::post()
            .uri(&format!("/api/annotations/{}/mask-groups", annotation_id))
            .set_json(&create_req)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201);

        // Cleanup
        cleanup_test_data(&pool, user_id).await;
    }

    #[actix_web::test]
    async fn test_get_mask_group_success() {
        let (app, pool) = setup_test_app().await;
        let (annotation_id, user_id) = create_test_data(&pool).await;

        // Create a test mask group first
        let create_req = CreateMaskGroupRequest {
            annotation_id: annotation_id,
            group_name: Some("Test Mask Group".to_string()),
            model_name: Some("Test Model".to_string()),
            version: Some("1.0.0".to_string()),
            modality: Some("CT".to_string()),
            slice_count: 100,
            mask_type: "segmentation".to_string(),
            description: Some("Test description".to_string()),
        };

        let create_req = test::TestRequest::post()
            .uri(&format!("/api/annotations/{}/mask-groups", annotation_id))
            .set_json(&create_req)
            .to_request();

        let create_resp = test::call_service(&app, create_req).await;
        assert_eq!(create_resp.status(), 201);

        // Get the created mask group
        let req = test::TestRequest::get()
            .uri(&format!("/api/annotations/{}/mask-groups/1", annotation_id))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        // Cleanup
        cleanup_test_data(&pool, user_id).await;
    }

    #[actix_web::test]
    async fn test_get_mask_group_not_found() {
        let (app, pool) = setup_test_app().await;
        let (annotation_id, user_id) = create_test_data(&pool).await;

        let req = test::TestRequest::get()
            .uri(&format!("/api/annotations/{}/mask-groups/999999", annotation_id))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404);

        // Cleanup
        cleanup_test_data(&pool, user_id).await;
    }

    #[actix_web::test]
    async fn test_update_mask_group_success() {
        let (app, pool) = setup_test_app().await;
        let (annotation_id, user_id) = create_test_data(&pool).await;

        // Create a test mask group first
        let create_req = CreateMaskGroupRequest {
            annotation_id: annotation_id,
            group_name: Some("Test Mask Group".to_string()),
            model_name: Some("Test Model".to_string()),
            version: Some("1.0.0".to_string()),
            modality: Some("CT".to_string()),
            slice_count: 100,
            mask_type: "segmentation".to_string(),
            description: Some("Test description".to_string()),
        };

        let create_req = test::TestRequest::post()
            .uri(&format!("/api/annotations/{}/mask-groups", annotation_id))
            .set_json(&create_req)
            .to_request();

        let create_resp = test::call_service(&app, create_req).await;
        assert_eq!(create_resp.status(), 201);

        // Update the mask group
        let update_req = UpdateMaskGroupRequest {
            group_name: Some("Updated Mask Group".to_string()),
            model_name: Some("Updated Model".to_string()),
            version: Some("2.0.0".to_string()),
            modality: Some("MRI".to_string()),
            slice_count: Some(200),
            mask_type: Some("classification".to_string()),
            description: Some("Updated description".to_string()),
        };

        let req = test::TestRequest::put()
            .uri(&format!("/api/annotations/{}/mask-groups/1", annotation_id))
            .set_json(&update_req)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        // Cleanup
        cleanup_test_data(&pool, user_id).await;
    }

    #[actix_web::test]
    async fn test_delete_mask_group_success() {
        let (app, pool) = setup_test_app().await;
        let (annotation_id, user_id) = create_test_data(&pool).await;

        // Create a test mask group first
        let create_req = CreateMaskGroupRequest {
            annotation_id: annotation_id,
            group_name: Some("Test Mask Group".to_string()),
            model_name: Some("Test Model".to_string()),
            version: Some("1.0.0".to_string()),
            modality: Some("CT".to_string()),
            slice_count: 100,
            mask_type: "segmentation".to_string(),
            description: Some("Test description".to_string()),
        };

        let create_req = test::TestRequest::post()
            .uri(&format!("/api/annotations/{}/mask-groups", annotation_id))
            .set_json(&create_req)
            .to_request();

        let create_resp = test::call_service(&app, create_req).await;
        assert_eq!(create_resp.status(), 201);

        // Delete the mask group
        let req = test::TestRequest::delete()
            .uri(&format!("/api/annotations/{}/mask-groups/1", annotation_id))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        // Cleanup
        cleanup_test_data(&pool, user_id).await;
    }

    #[actix_web::test]
    async fn test_list_mask_groups_success() {
        let (app, pool) = setup_test_app().await;
        let (annotation_id, user_id) = create_test_data(&pool).await;

        // Create a test mask group first
        let create_req = CreateMaskGroupRequest {
            annotation_id: annotation_id,
            group_name: Some("Test Mask Group".to_string()),
            model_name: Some("Test Model".to_string()),
            version: Some("1.0.0".to_string()),
            modality: Some("CT".to_string()),
            slice_count: 100,
            mask_type: "segmentation".to_string(),
            description: Some("Test description".to_string()),
        };

        let create_req = test::TestRequest::post()
            .uri(&format!("/api/annotations/{}/mask-groups", annotation_id))
            .set_json(&create_req)
            .to_request();

        let create_resp = test::call_service(&app, create_req).await;
        assert_eq!(create_resp.status(), 201);

        // List mask groups
        let req = test::TestRequest::get()
            .uri(&format!("/api/annotations/{}/mask-groups", annotation_id))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        // Cleanup
        cleanup_test_data(&pool, user_id).await;
    }

    #[actix_web::test]
    async fn test_generate_upload_url_success() {
        let (app, pool) = setup_test_app().await;
        let (annotation_id, user_id) = create_test_data(&pool).await;

        // Create a test mask group first
        let create_req = CreateMaskGroupRequest {
            annotation_id: annotation_id,
            group_name: Some("Test Mask Group".to_string()),
            model_name: Some("Test Model".to_string()),
            version: Some("1.0.0".to_string()),
            modality: Some("CT".to_string()),
            slice_count: 100,
            mask_type: "segmentation".to_string(),
            description: Some("Test description".to_string()),
        };

        let create_req = test::TestRequest::post()
            .uri(&format!("/api/annotations/{}/mask-groups", annotation_id))
            .set_json(&create_req)
            .to_request();

        let create_resp = test::call_service(&app, create_req).await;
        assert_eq!(create_resp.status(), 201);

        let body: serde_json::Value = test::read_body_json(create_resp).await;
        let mask_group_id = body["id"].as_i64().unwrap() as i32;

        // Generate upload URL
        let upload_req = SignedUrlRequest {
            mask_group_id: mask_group_id,
            filename: "test.png".to_string(),
            mime_type: "image/png".to_string(),
            file_size: Some(102400),
            slice_index: Some(1),
            sop_instance_uid: Some("1.2.3.4.5.6.7.8.9.1.1".to_string()),
            label_name: Some("test_label".to_string()),
            ttl_seconds: Some(3600),
        };

        let req = test::TestRequest::post()
            .uri(&format!("/api/annotations/{}/mask-groups/1/upload-url", annotation_id))
            .set_json(&upload_req)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        // Cleanup
        cleanup_test_data(&pool, user_id).await;
    }

    #[actix_web::test]
    async fn test_complete_upload_success() {
        let (app, pool) = setup_test_app().await;
        let (annotation_id, user_id) = create_test_data(&pool).await;

        // Create a test mask group first
        let create_req = CreateMaskGroupRequest {
            annotation_id: annotation_id,
            group_name: Some("Test Mask Group".to_string()),
            model_name: Some("Test Model".to_string()),
            version: Some("1.0.0".to_string()),
            modality: Some("CT".to_string()),
            slice_count: 100,
            mask_type: "segmentation".to_string(),
            description: Some("Test description".to_string()),
        };

        let create_req = test::TestRequest::post()
            .uri(&format!("/api/annotations/{}/mask-groups", annotation_id))
            .set_json(&create_req)
            .to_request();

        let create_resp = test::call_service(&app, create_req).await;
        assert_eq!(create_resp.status(), 201);

        let body: serde_json::Value = test::read_body_json(create_resp).await;
        let mask_group_id = body["id"].as_i64().unwrap() as i32;

        // Complete upload
        let complete_req = CompleteUploadRequest {
            mask_group_id: mask_group_id,
            slice_count: 100,
            labels: vec!["liver".to_string(), "spleen".to_string()],
            uploaded_files: vec!["file1.png".to_string(), "file2.png".to_string()],
        };

        let req = test::TestRequest::post()
            .uri(&format!("/api/annotations/{}/mask-groups/1/complete-upload", annotation_id))
            .set_json(&complete_req)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        // Cleanup
        cleanup_test_data(&pool, user_id).await;
    }
}
