#[cfg(test)]
mod mask_controller_tests {
    use actix_web::{test, web, App};
    use pacs_server::application::dto::mask_dto::{
        CreateMaskRequest, UpdateMaskRequest, DownloadUrlRequest
    };
    use pacs_server::application::use_cases::MaskUseCase;
    use pacs_server::domain::services::{MaskServiceImpl, MaskGroupServiceImpl};
    use pacs_server::application::services::SignedUrlServiceImpl;
    use pacs_server::infrastructure::repositories::{
        MaskRepositoryImpl, MaskGroupRepositoryImpl, AnnotationRepositoryImpl, UserRepositoryImpl, ProjectRepositoryImpl
    };
    use pacs_server::presentation::controllers::mask_controller::configure_routes;
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

        let mask_repo = MaskRepositoryImpl::new(pool.clone());
        let mask_group_repo = MaskGroupRepositoryImpl::new(pool.clone());
        let annotation_repo = AnnotationRepositoryImpl::new(pool.clone());
        let user_repo = UserRepositoryImpl::new(pool.clone());
        let project_repo = ProjectRepositoryImpl::new(pool.clone());

        let pool = Arc::new(pool);
        let mask_service = MaskServiceImpl::new(Arc::new(mask_repo), Arc::new(mask_group_repo.clone()), Arc::new(user_repo.clone()));
        let mask_group_service = MaskGroupServiceImpl::new(Arc::new(mask_group_repo), Arc::new(annotation_repo), Arc::new(user_repo));
        
        // Mock SignedUrlService for testing
        let signed_url_service = Arc::new(MockSignedUrlService::new());
        
        let mask_use_case = Arc::new(MaskUseCase::new(
            Arc::new(mask_service),
            Arc::new(mask_group_service),
            signed_url_service,
        ));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(mask_use_case.clone()))
                .configure(|cfg| configure_routes(cfg, mask_use_case.clone())),
        )
        .await;

        (app, pool)
    }

    // Mock SignedUrlService for testing
    use pacs_server::application::services::{SignedUrlService, SignedUrlError, SignedUrlResponse};
    use async_trait::async_trait;
    use std::collections::HashMap;

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

    async fn create_test_data(pool: &sqlx::Pool<sqlx::Postgres>) -> (i32, i32, i32) {
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
        println!("DEBUG: Created user_id = {}", user_id);

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
        println!("DEBUG: Created annotation_id = {}, user_id = {}", annotation_id, user_id);

        // Create test mask group
        let mask_group_result = sqlx::query(
            "INSERT INTO annotation_mask_group (annotation_id, group_name, model_name, version, modality, slice_count, mask_type, description, created_by) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING id"
        )
        .bind(annotation_id)
        .bind("Test Mask Group")
        .bind("Test Model")
        .bind("1.0.0")
        .bind("CT")
        .bind(100)
        .bind("segmentation")
        .bind("Test description")
        .bind(user_id)
        .fetch_one(pool)
        .await
        .expect("Failed to create test mask group");

        let mask_group_id: i32 = mask_group_result.get("id");

        (annotation_id, mask_group_id, user_id)
    }

    async fn cleanup_test_data(pool: &sqlx::Pool<sqlx::Postgres>, user_id: i32, project_id: i32) {
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
        
        sqlx::query("DELETE FROM security_project WHERE id = $1")
            .bind(project_id)
            .execute(pool)
            .await
            .ok();
    }

    #[actix_web::test]
    async fn test_create_mask_success() {
        let (app, pool) = setup_test_app().await;
        let (annotation_id, mask_group_id, user_id) = create_test_data(&pool).await;

        let create_req = CreateMaskRequest {
            mask_group_id: mask_group_id,
            file_path: "masks/test.png".to_string(),
            mime_type: "image/png".to_string(),
            slice_index: Some(1),
            sop_instance_uid: Some("1.2.3.4.5.6.7.8.9.1.1".to_string()),
            label_name: Some("lung_nodule".to_string()),
            file_size: Some(102400),
            checksum: Some("md5-checksum-12345".to_string()),
            width: Some(512),
            height: Some(512),
        };

        let req = test::TestRequest::post()
            .uri(&format!("/api/annotations/{}/mask-groups/{}/masks", annotation_id, mask_group_id))
            .insert_header(("X-User-ID", user_id.to_string()))
            .set_json(&create_req)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201);

        // Cleanup
        cleanup_test_data(&pool, user_id, 1).await;
    }

    #[actix_web::test]
    async fn test_get_mask_success() {
        let (app, pool) = setup_test_app().await;
        let (annotation_id, mask_group_id, user_id) = create_test_data(&pool).await;

        // Create a test mask first
        let create_req = CreateMaskRequest {
            mask_group_id: mask_group_id,
            file_path: "masks/test.png".to_string(),
            mime_type: "image/png".to_string(),
            slice_index: Some(1),
            sop_instance_uid: Some("1.2.3.4.5.6.7.8.9.1.1".to_string()),
            label_name: Some("lung_nodule".to_string()),
            file_size: Some(102400),
            checksum: Some("md5-checksum-12345".to_string()),
            width: Some(512),
            height: Some(512),
        };

        let create_req = test::TestRequest::post()
            .uri(&format!("/api/annotations/{}/mask-groups/{}/masks", annotation_id, mask_group_id))
            .insert_header(("X-User-ID", user_id.to_string()))
            .set_json(&create_req)
            .to_request();

        let create_resp = test::call_service(&app, create_req).await;
        assert_eq!(create_resp.status(), 201);

        let body: serde_json::Value = test::read_body_json(create_resp).await;
        let mask_id = body["id"].as_i64().unwrap() as i32;

        // Get the created mask
        let req = test::TestRequest::get()
            .uri(&format!("/api/annotations/{}/mask-groups/{}/masks/{}", annotation_id, mask_group_id, mask_id))
            .insert_header(("X-User-ID", user_id.to_string()))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        // Cleanup
        cleanup_test_data(&pool, user_id, 1).await;
    }

    #[actix_web::test]
    async fn test_get_mask_not_found() {
        let (app, pool) = setup_test_app().await;
        let (annotation_id, mask_group_id, user_id) = create_test_data(&pool).await;

        let req = test::TestRequest::get()
            .uri(&format!("/api/annotations/{}/mask-groups/{}/masks/999999", annotation_id, mask_group_id))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404);

        // Cleanup
        cleanup_test_data(&pool, user_id, 1).await;
    }

    #[actix_web::test]
    async fn test_update_mask_success() {
        let (app, pool) = setup_test_app().await;
        let (annotation_id, mask_group_id, user_id) = create_test_data(&pool).await;

        // Create a test mask first
        let create_req = CreateMaskRequest {
            mask_group_id: mask_group_id,
            file_path: "masks/test.png".to_string(),
            mime_type: "image/png".to_string(),
            slice_index: Some(1),
            sop_instance_uid: Some("1.2.3.4.5.6.7.8.9.1.1".to_string()),
            label_name: Some("lung_nodule".to_string()),
            file_size: Some(102400),
            checksum: Some("md5-checksum-12345".to_string()),
            width: Some(512),
            height: Some(512),
        };

        let create_req = test::TestRequest::post()
            .uri(&format!("/api/annotations/{}/mask-groups/{}/masks", annotation_id, mask_group_id))
            .insert_header(("X-User-ID", user_id.to_string()))
            .set_json(&create_req)
            .to_request();

        let create_resp = test::call_service(&app, create_req).await;
        assert_eq!(create_resp.status(), 201);

        let body: serde_json::Value = test::read_body_json(create_resp).await;
        let mask_id = body["id"].as_i64().unwrap() as i32;

        // Update the mask
        let update_req = UpdateMaskRequest {
            file_path: Some("masks/updated.png".to_string()),
            mime_type: Some("image/png".to_string()),
            slice_index: Some(2),
            sop_instance_uid: Some("1.2.3.4.5.6.7.8.9.1.2".to_string()),
            label_name: Some("updated_label".to_string()),
            file_size: Some(204800),
            checksum: Some("md5-checksum-67890".to_string()),
            width: Some(1024),
            height: Some(1024),
        };

        let req = test::TestRequest::put()
            .uri(&format!("/api/annotations/{}/mask-groups/{}/masks/{}", annotation_id, mask_group_id, mask_id))
            .insert_header(("X-User-ID", user_id.to_string()))
            .set_json(&update_req)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        // Cleanup
        cleanup_test_data(&pool, user_id, 1).await;
    }

    #[actix_web::test]
    async fn test_delete_mask_success() {
        let (app, pool) = setup_test_app().await;
        let (annotation_id, mask_group_id, user_id) = create_test_data(&pool).await;

        // Create a test mask first
        let create_req = CreateMaskRequest {
            mask_group_id: mask_group_id,
            file_path: "masks/test.png".to_string(),
            mime_type: "image/png".to_string(),
            slice_index: Some(1),
            sop_instance_uid: Some("1.2.3.4.5.6.7.8.9.1.1".to_string()),
            label_name: Some("lung_nodule".to_string()),
            file_size: Some(102400),
            checksum: Some("md5-checksum-12345".to_string()),
            width: Some(512),
            height: Some(512),
        };

        let create_req = test::TestRequest::post()
            .uri(&format!("/api/annotations/{}/mask-groups/{}/masks", annotation_id, mask_group_id))
            .insert_header(("X-User-ID", user_id.to_string()))
            .set_json(&create_req)
            .to_request();

        let create_resp = test::call_service(&app, create_req).await;
        assert_eq!(create_resp.status(), 201);

        let body: serde_json::Value = test::read_body_json(create_resp).await;
        let mask_id = body["id"].as_i64().unwrap() as i32;

        // Delete the mask
        let req = test::TestRequest::delete()
            .uri(&format!("/api/annotations/{}/mask-groups/{}/masks/{}", annotation_id, mask_group_id, mask_id))
            .insert_header(("X-User-ID", user_id.to_string()))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 204);

        // Cleanup
        cleanup_test_data(&pool, user_id, 1).await;
    }

    #[actix_web::test]
    async fn test_list_masks_success() {
        let (app, pool) = setup_test_app().await;
        let (annotation_id, mask_group_id, user_id) = create_test_data(&pool).await;

        // Create a test mask first
        let create_req = CreateMaskRequest {
            mask_group_id: mask_group_id,
            file_path: "masks/test.png".to_string(),
            mime_type: "image/png".to_string(),
            slice_index: Some(1),
            sop_instance_uid: Some("1.2.3.4.5.6.7.8.9.1.1".to_string()),
            label_name: Some("lung_nodule".to_string()),
            file_size: Some(102400),
            checksum: Some("md5-checksum-12345".to_string()),
            width: Some(512),
            height: Some(512),
        };

        let create_req = test::TestRequest::post()
            .uri(&format!("/api/annotations/{}/mask-groups/{}/masks", annotation_id, mask_group_id))
            .insert_header(("X-User-ID", user_id.to_string()))
            .set_json(&create_req)
            .to_request();

        let create_resp = test::call_service(&app, create_req).await;
        assert_eq!(create_resp.status(), 201);

        // List masks
        let req = test::TestRequest::get()
            .uri(&format!("/api/annotations/{}/mask-groups/{}/masks", annotation_id, mask_group_id))
            .insert_header(("X-User-ID", user_id.to_string()))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        // Cleanup
        cleanup_test_data(&pool, user_id, 1).await;
    }

    #[actix_web::test]
    async fn test_generate_download_url_success() {
        let (app, pool) = setup_test_app().await;
        let (annotation_id, mask_group_id, user_id) = create_test_data(&pool).await;

        // Create a test mask first
        let create_req = CreateMaskRequest {
            mask_group_id: mask_group_id,
            file_path: "masks/test.png".to_string(),
            mime_type: "image/png".to_string(),
            slice_index: Some(1),
            sop_instance_uid: Some("1.2.3.4.5.6.7.8.9.1.1".to_string()),
            label_name: Some("lung_nodule".to_string()),
            file_size: Some(102400),
            checksum: Some("md5-checksum-12345".to_string()),
            width: Some(512),
            height: Some(512),
        };

        let create_req = test::TestRequest::post()
            .uri(&format!("/api/annotations/{}/mask-groups/{}/masks", annotation_id, mask_group_id))
            .insert_header(("X-User-ID", user_id.to_string()))
            .set_json(&create_req)
            .to_request();

        let create_resp = test::call_service(&app, create_req).await;
        assert_eq!(create_resp.status(), 201);

        // Generate download URL
        let download_req = DownloadUrlRequest {
            mask_id: 1,
            file_path: "masks/test.png".to_string(),
            expires_in: Some(3600),
        };

        let req = test::TestRequest::post()
            .uri(&format!("/api/annotations/{}/mask-groups/{}/masks/1/download-url", annotation_id, mask_group_id))
            .insert_header(("X-User-ID", user_id.to_string()))
            .set_json(&download_req)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        // Cleanup
        cleanup_test_data(&pool, user_id, 1).await;
    }

    #[actix_web::test]
    async fn test_get_mask_stats_success() {
        let (app, pool) = setup_test_app().await;
        let (annotation_id, mask_group_id, user_id) = create_test_data(&pool).await;

        // Create a test mask first
        let create_req = CreateMaskRequest {
            mask_group_id: mask_group_id,
            file_path: "masks/test.png".to_string(),
            mime_type: "image/png".to_string(),
            slice_index: Some(1),
            sop_instance_uid: Some("1.2.3.4.5.6.7.8.9.1.1".to_string()),
            label_name: Some("lung_nodule".to_string()),
            file_size: Some(102400),
            checksum: Some("md5-checksum-12345".to_string()),
            width: Some(512),
            height: Some(512),
        };

        let create_req = test::TestRequest::post()
            .uri(&format!("/api/annotations/{}/mask-groups/{}/masks", annotation_id, mask_group_id))
            .insert_header(("X-User-ID", user_id.to_string()))
            .set_json(&create_req)
            .to_request();

        let create_resp = test::call_service(&app, create_req).await;
        assert_eq!(create_resp.status(), 201);

        // Get mask stats
        let url = format!("/api/annotations/{}/mask-groups/{}/masks/stats", annotation_id, mask_group_id);
        println!("DEBUG: get_mask_stats URL = {}", url);
        let req = test::TestRequest::get()
            .uri(&url)
            .insert_header(("X-User-ID", user_id.to_string()))
            .to_request();

        let resp = test::call_service(&app, req).await;
        println!("DEBUG: get_mask_stats response status = {}", resp.status());
        assert_eq!(resp.status(), 200);

        // Cleanup
        cleanup_test_data(&pool, user_id, 1).await;
    }
}
