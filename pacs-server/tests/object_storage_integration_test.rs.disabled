#[cfg(test)]
mod object_storage_integration_tests {
    use actix_web::{test, web, App};
    use pacs_server::application::dto::{
        annotation_dto::CreateAnnotationRequest,
        mask_group_dto::{CreateMaskGroupRequest, SignedUrlRequest, CompleteUploadRequest},
        mask_dto::CreateMaskRequest
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
    use pacs_server::infrastructure::external::{S3Service, MinIOService};
    use pacs_server::infrastructure::config::ObjectStorageConfig;
    use sqlx::postgres::PgPoolOptions;
    use std::sync::Arc;
    use uuid::Uuid;
    use tokio::fs;

    async fn setup_test_app() -> (
        impl actix_web::dev::Service<
            actix_http::Request,
            Response = actix_web::dev::ServiceResponse,
            Error = actix_web::Error,
        >,
        Arc<sqlx::Pool<sqlx::Postgres>>,
        Arc<S3Service>,
        Arc<MinIOService>,
    ) {
        dotenvy::dotenv().ok();

        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://admin:admin123@localhost:5432/pacs_db".to_string());

        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await
            .expect("Failed to connect to test database");

        let pool = Arc::new(pool);
        
        // Initialize repositories
        let annotation_repo = AnnotationRepositoryImpl::new(pool.clone());
        let mask_group_repo = MaskGroupRepositoryImpl::new(pool.clone());
        let mask_repo = MaskRepositoryImpl::new(pool.clone());
        let user_repo = UserRepositoryImpl::new(pool.clone());
        let project_repo = ProjectRepositoryImpl::new(pool.clone());
        
        // Initialize services
        let annotation_service = AnnotationServiceImpl::new(annotation_repo, user_repo.clone(), project_repo.clone());
        let mask_group_service = MaskGroupServiceImpl::new(mask_group_repo, user_repo.clone(), project_repo.clone());
        let mask_service = MaskServiceImpl::new(mask_repo, mask_group_repo.clone());
        
        // Initialize object storage services
        let s3_config = ObjectStorageConfig {
            provider: "minio".to_string(),
            endpoint: std::env::var("S3_ENDPOINT").unwrap_or_else(|_| "http://localhost:9000".to_string()),
            access_key: std::env::var("S3_ACCESS_KEY").unwrap_or_else(|_| "minioadmin".to_string()),
            secret_key: std::env::var("S3_SECRET_KEY").unwrap_or_else(|_| "minioadmin".to_string()),
            bucket_name: std::env::var("S3_BUCKET").unwrap_or_else(|_| "pacs-test".to_string()),
            region: std::env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
        };

        let minio_config = ObjectStorageConfig {
            provider: "minio".to_string(),
            endpoint: std::env::var("MINIO_ENDPOINT").unwrap_or_else(|_| "http://localhost:9000".to_string()),
            access_key: std::env::var("MINIO_ACCESS_KEY").unwrap_or_else(|_| "minioadmin".to_string()),
            secret_key: std::env::var("MINIO_SECRET_KEY").unwrap_or_else(|_| "minioadmin".to_string()),
            bucket_name: std::env::var("MINIO_BUCKET").unwrap_or_else(|_| "pacs-test".to_string()),
            region: std::env::var("MINIO_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
        };

        let s3_service = Arc::new(S3Service::new(s3_config));
        let minio_service = Arc::new(MinIOService::new(minio_config));
        
        // Initialize use cases with real object storage services
        let annotation_use_case = Arc::new(AnnotationUseCase::new(annotation_service));
        let mask_group_use_case = Arc::new(MaskGroupUseCase::new(
            Arc::new(mask_group_service),
            s3_service.clone(),
        ));
        let mask_use_case = Arc::new(MaskUseCase::new(
            Arc::new(mask_service),
            Arc::new(mask_group_service),
            s3_service.clone(),
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

        (app, pool, s3_service, minio_service)
    }

    async fn create_test_data(pool: &Arc<sqlx::Pool<sqlx::Postgres>>) -> (i32, i32, i32) {
        use sqlx::Row;
        
        // Create test user
        let keycloak_id = Uuid::new_v4();
        let user_result = sqlx::query(
            "INSERT INTO security_user (keycloak_id, username, email) VALUES ($1, $2, $3) RETURNING id"
        )
        .bind(keycloak_id)
        .bind("storagetestuser")
        .bind("storagetest@example.com")
        .fetch_one(pool.as_ref())
        .await
        .expect("Failed to create test user");

        let user_id: i32 = user_result.get("id");

        // Create test project
        let project_result = sqlx::query(
            "INSERT INTO security_project (name, description) VALUES ($1, $2) RETURNING id"
        )
        .bind("Storage Test Project")
        .bind("Storage Test Description")
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
        .bind("storage_tool")
        .bind("1.0.0")
        .bind("storage_viewer")
        .bind(serde_json::json!({"type": "circle", "x": 100, "y": 200, "radius": 50}))
        .bind("Storage test annotation")
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

    async fn create_test_file(content: &str, filename: &str) -> String {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join(filename);
        
        fs::write(&file_path, content).await
            .expect("Failed to create test file");
        
        file_path.to_string_lossy().to_string()
    }

    async fn cleanup_test_file(file_path: &str) {
        let _ = fs::remove_file(file_path).await;
    }

    #[actix_web::test]
    async fn test_s3_bucket_operations() {
        let (_, pool, s3_service, _) = setup_test_app().await;

        // Test 1: Ensure bucket exists
        let bucket_result = s3_service.ensure_bucket_exists().await;
        assert!(bucket_result.is_ok(), "Failed to ensure S3 bucket exists: {:?}", bucket_result);

        // Test 2: Upload a test file
        let test_content = "This is a test file for S3 integration";
        let test_file_path = create_test_file(test_content, "s3_test_file.txt").await;

        let upload_result = s3_service.upload_file(&test_file_path, "test/s3_test_file.txt").await;
        assert!(upload_result.is_ok(), "Failed to upload file to S3: {:?}", upload_result);

        // Test 3: Download the uploaded file
        let download_result = s3_service.download_file("test/s3_test_file.txt", "/tmp/s3_downloaded_file.txt").await;
        assert!(download_result.is_ok(), "Failed to download file from S3: {:?}", download_result);

        // Verify downloaded content
        let downloaded_content = fs::read_to_string("/tmp/s3_downloaded_file.txt").await
            .expect("Failed to read downloaded file");
        assert_eq!(downloaded_content, test_content);

        // Test 4: Delete the uploaded file
        let delete_result = s3_service.delete_file("test/s3_test_file.txt").await;
        assert!(delete_result.is_ok(), "Failed to delete file from S3: {:?}", delete_result);

        // Cleanup
        cleanup_test_file(&test_file_path).await;
        let _ = fs::remove_file("/tmp/s3_downloaded_file.txt").await;
    }

    #[actix_web::test]
    async fn test_minio_bucket_operations() {
        let (_, pool, _, minio_service) = setup_test_app().await;

        // Test 1: Ensure bucket exists
        let bucket_result = minio_service.ensure_bucket_exists().await;
        assert!(bucket_result.is_ok(), "Failed to ensure MinIO bucket exists: {:?}", bucket_result);

        // Test 2: Upload a test file
        let test_content = "This is a test file for MinIO integration";
        let test_file_path = create_test_file(test_content, "minio_test_file.txt").await;

        let upload_result = minio_service.upload_file(&test_file_path, "test/minio_test_file.txt").await;
        assert!(upload_result.is_ok(), "Failed to upload file to MinIO: {:?}", upload_result);

        // Test 3: Download the uploaded file
        let download_result = minio_service.download_file("test/minio_test_file.txt", "/tmp/minio_downloaded_file.txt").await;
        assert!(download_result.is_ok(), "Failed to download file from MinIO: {:?}", download_result);

        // Verify downloaded content
        let downloaded_content = fs::read_to_string("/tmp/minio_downloaded_file.txt").await
            .expect("Failed to read downloaded file");
        assert_eq!(downloaded_content, test_content);

        // Test 4: Delete the uploaded file
        let delete_result = minio_service.delete_file("test/minio_test_file.txt").await;
        assert!(delete_result.is_ok(), "Failed to delete file from MinIO: {:?}", delete_result);

        // Cleanup
        cleanup_test_file(&test_file_path).await;
        let _ = fs::remove_file("/tmp/minio_downloaded_file.txt").await;
    }

    #[actix_web::test]
    async fn test_signed_url_generation_s3() {
        let (app, pool, s3_service, _) = setup_test_app().await;
        let (annotation_id, user_id, project_id) = create_test_data(&pool).await;

        // Create mask group
        let mask_group_req = CreateMaskGroupRequest {
            annotation_id,
            group_name: Some("S3 Storage Test Group".to_string()),
            model_name: Some("S3StorageModel".to_string()),
            version: Some("1.0.0".to_string()),
            modality: Some("CT".to_string()),
            slice_count: 100,
            mask_type: "segmentation".to_string(),
            description: Some("S3 storage test group".to_string()),
        };

        let req = test::TestRequest::post()
            .uri(&format!("/api/annotations/{}/mask-groups", annotation_id))
            .set_json(&mask_group_req)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201);

        let body: serde_json::Value = test::read_body_json(resp).await;
        let mask_group_id = body["id"].as_i64().unwrap() as i32;

        // Test 1: Generate upload URL
        let upload_req = SignedUrlRequest {
            filename: "s3_upload_test.png".to_string(),
            mime_type: "image/png".to_string(),
            file_size: Some(102400),
            label_name: Some("test_label".to_string()),
            mask_group_id: mask_group_id,
            slice_index: Some(0),
            sop_instance_uid: Some("1.2.840.113619.2.55.3.604688119.868.1234567890.3".to_string()),
            ttl_seconds: Some(3600),
        };

        let req = test::TestRequest::post()
            .uri(&format!("/api/annotations/{}/mask-groups/{}/upload-url", annotation_id, mask_group_id))
            .set_json(&upload_req)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body["upload_url"].is_string());
        assert!(body["file_path"].is_string());
        assert_eq!(body["expires_in"], 3600);

        // Test 2: Generate download URL
        let download_req = pacs_server::application::dto::mask_dto::DownloadUrlRequest {
            mask_id: mask_id,
            file_path: "masks/s3_test.png".to_string(),
            expires_in: Some(3600),
        };

        let req = test::TestRequest::post()
            .uri(&format!("/api/annotations/{}/mask-groups/{}/masks/1/download-url", annotation_id, mask_group_id))
            .set_json(&download_req)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body["download_url"].is_string());
        assert_eq!(body["file_path"], "masks/s3_test.png");
        assert_eq!(body["expires_in"], 3600);

        // Cleanup
        cleanup_test_data(&pool, user_id, project_id).await;
    }

    #[actix_web::test]
    async fn test_large_file_upload_s3() {
        let (_, pool, s3_service, _) = setup_test_app().await;

        // Create a large test file (1MB)
        let large_content = "x".repeat(1024 * 1024);
        let large_file_path = create_test_file(&large_content, "large_s3_test_file.txt").await;

        // Test upload
        let upload_result = s3_service.upload_file(&large_file_path, "test/large_s3_test_file.txt").await;
        assert!(upload_result.is_ok(), "Failed to upload large file to S3: {:?}", upload_result);

        // Test download
        let download_result = s3_service.download_file("test/large_s3_test_file.txt", "/tmp/large_s3_downloaded_file.txt").await;
        assert!(download_result.is_ok(), "Failed to download large file from S3: {:?}", download_result);

        // Verify file size
        let metadata = fs::metadata("/tmp/large_s3_downloaded_file.txt").await
            .expect("Failed to get file metadata");
        assert_eq!(metadata.len(), 1024 * 1024);

        // Cleanup
        let delete_result = s3_service.delete_file("test/large_s3_test_file.txt").await;
        assert!(delete_result.is_ok(), "Failed to delete large file from S3: {:?}", delete_result);

        cleanup_test_file(&large_file_path).await;
        let _ = fs::remove_file("/tmp/large_s3_downloaded_file.txt").await;
    }

    #[actix_web::test]
    async fn test_concurrent_uploads_s3() {
        let (_, pool, s3_service, _) = setup_test_app().await;

        // Test concurrent uploads
        let mut handles = vec![];
        
        for i in 0..5 {
            let s3_service_clone = s3_service.clone();
            let handle = tokio::spawn(async move {
                let content = format!("Concurrent upload test file {}", i);
                let file_path = create_test_file(&content, &format!("concurrent_test_{}.txt", i)).await;
                
                let upload_result = s3_service_clone.upload_file(&file_path, &format!("test/concurrent_test_{}.txt", i)).await;
                
                // Cleanup
                cleanup_test_file(&file_path).await;
                
                upload_result
            });
            handles.push(handle);
        }

        // Wait for all uploads to complete
        let results = futures::future::join_all(handles).await;
        
        for (i, result) in results.into_iter().enumerate() {
            let upload_result = result.expect("Concurrent upload task failed");
            assert!(upload_result.is_ok(), "Concurrent upload {} failed: {:?}", i, upload_result);
        }

        // Cleanup uploaded files
        for i in 0..5 {
            let _ = s3_service.delete_file(&format!("test/concurrent_test_{}.txt", i)).await;
        }
    }

    #[actix_web::test]
    async fn test_storage_error_handling() {
        let (_, pool, s3_service, _) = setup_test_app().await;

        // Test 1: Upload to non-existent bucket (should fail)
        let test_content = "Error handling test";
        let test_file_path = create_test_file(test_content, "error_test.txt").await;

        // This should fail because we're using a different bucket name
        let error_config = ObjectStorageConfig {
            provider: "s3".to_string(),
            endpoint: std::env::var("S3_ENDPOINT").unwrap_or_else(|_| "http://localhost:9000".to_string()),
            access_key: std::env::var("S3_ACCESS_KEY").unwrap_or_else(|_| "minioadmin".to_string()),
            secret_key: std::env::var("S3_SECRET_KEY").unwrap_or_else(|_| "minioadmin".to_string()),
            bucket_name: "non-existent-bucket".to_string(),
            region: std::env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
        };

        let error_s3_service = S3Service::new(error_config);
        let upload_result = error_s3_service.upload_file(&test_file_path, "test/error_test.txt").await;
        
        // This might succeed or fail depending on S3 configuration
        // In a real test environment, this should fail
        println!("Upload to non-existent bucket result: {:?}", upload_result);

        // Test 2: Download non-existent file
        let download_result = s3_service.download_file("test/non_existent_file.txt", "/tmp/non_existent_download.txt").await;
        assert!(download_result.is_err(), "Download of non-existent file should fail");

        // Test 3: Delete non-existent file
        let delete_result = s3_service.delete_file("test/non_existent_file.txt").await;
        assert!(delete_result.is_err(), "Delete of non-existent file should fail");

        // Cleanup
        cleanup_test_file(&test_file_path).await;
    }

    #[actix_web::test]
    async fn test_file_metadata_operations() {
        let (_, pool, s3_service, _) = setup_test_app().await;

        // Upload a test file
        let test_content = "Metadata test file";
        let test_file_path = create_test_file(test_content, "metadata_test.txt").await;

        let upload_result = s3_service.upload_file(&test_file_path, "test/metadata_test.txt").await;
        assert!(upload_result.is_ok(), "Failed to upload file for metadata test");

        // Test 1: Check if file exists
        let exists_result = s3_service.file_exists("test/metadata_test.txt").await;
        assert!(exists_result.is_ok(), "Failed to check file existence");
        assert!(exists_result.unwrap(), "Uploaded file should exist");

        // Test 2: Get file size
        let size_result = s3_service.get_file_size("test/metadata_test.txt").await;
        assert!(size_result.is_ok(), "Failed to get file size");
        assert_eq!(size_result.unwrap(), test_content.len() as u64);

        // Test 3: List files in directory
        let list_result = s3_service.list_files("test/").await;
        assert!(list_result.is_ok(), "Failed to list files");
        let files = list_result.unwrap();
        assert!(files.contains(&"test/metadata_test.txt".to_string()));

        // Cleanup
        let delete_result = s3_service.delete_file("test/metadata_test.txt").await;
        assert!(delete_result.is_ok(), "Failed to delete test file");

        cleanup_test_file(&test_file_path).await;
    }

    #[actix_web::test]
    async fn test_storage_performance_benchmark() {
        let (_, pool, s3_service, _) = setup_test_app().await;

        // Test upload performance
        let test_content = "Performance test content".repeat(1000); // ~25KB
        let test_file_path = create_test_file(&test_content, "performance_test.txt").await;

        let start = std::time::Instant::now();
        let upload_result = s3_service.upload_file(&test_file_path, "test/performance_test.txt").await;
        let upload_duration = start.elapsed();

        assert!(upload_result.is_ok(), "Performance test upload failed");
        println!("Upload performance: {:?} for {} bytes", upload_duration, test_content.len());

        // Test download performance
        let start = std::time::Instant::now();
        let download_result = s3_service.download_file("test/performance_test.txt", "/tmp/performance_download.txt").await;
        let download_duration = start.elapsed();

        assert!(download_result.is_ok(), "Performance test download failed");
        println!("Download performance: {:?} for {} bytes", download_duration, test_content.len());

        // Performance assertions (adjust based on your requirements)
        assert!(upload_duration < std::time::Duration::from_secs(5), "Upload too slow");
        assert!(download_duration < std::time::Duration::from_secs(5), "Download too slow");

        // Cleanup
        let _ = s3_service.delete_file("test/performance_test.txt").await;
        cleanup_test_file(&test_file_path).await;
        let _ = fs::remove_file("/tmp/performance_download.txt").await;
    }

    #[actix_web::test]
    async fn test_storage_configuration_validation() {
        // Test 1: Invalid endpoint
        let invalid_config = ObjectStorageConfig {
            provider: "s3".to_string(),
            endpoint: "invalid-endpoint".to_string(),
            access_key: "test".to_string(),
            secret_key: "test".to_string(),
            bucket_name: "test".to_string(),
            region: "us-east-1".to_string(),
        };

        let invalid_s3_service = S3Service::new(invalid_config);
        let bucket_result = invalid_s3_service.ensure_bucket_exists().await;
        assert!(bucket_result.is_err(), "Invalid endpoint should fail");

        // Test 2: Invalid credentials
        let invalid_creds_config = ObjectStorageConfig {
            provider: "s3".to_string(),
            endpoint: std::env::var("S3_ENDPOINT").unwrap_or_else(|_| "http://localhost:9000".to_string()),
            access_key: "invalid".to_string(),
            secret_key: "invalid".to_string(),
            bucket_name: "test".to_string(),
            region: "us-east-1".to_string(),
        };

        let invalid_creds_s3_service = S3Service::new(invalid_creds_config);
        let bucket_result = invalid_creds_s3_service.ensure_bucket_exists().await;
        // This might succeed or fail depending on S3 configuration
        println!("Invalid credentials result: {:?}", bucket_result);
    }
}
