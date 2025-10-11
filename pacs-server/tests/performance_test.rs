#[cfg(test)]
mod performance_tests {
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
    use sqlx::postgres::PgPoolOptions;
    use std::sync::Arc;
    use std::time::{Duration, Instant};
    use tokio::time::timeout;
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
            .max_connections(10) // 성능 테스트를 위해 더 많은 연결 허용
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
        let mask_group_service = MaskGroupServiceImpl::new(mask_group_repo, user_repo.clone(), project_repo.clone());
        let mask_service = MaskServiceImpl::new(mask_repo, mask_group_repo.clone());
        
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
            // Simulate network delay
            tokio::time::sleep(Duration::from_millis(10)).await;
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
            tokio::time::sleep(Duration::from_millis(5)).await;
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
            tokio::time::sleep(Duration::from_millis(15)).await;
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
            tokio::time::sleep(Duration::from_millis(8)).await;
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
            tokio::time::sleep(Duration::from_millis(12)).await;
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
            tokio::time::sleep(Duration::from_millis(6)).await;
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
        .bind("perftestuser")
        .bind("perftest@example.com")
        .fetch_one(pool.as_ref())
        .await
        .expect("Failed to create test user");

        let user_id: i32 = user_result.get("id");

        // Create test project
        let project_result = sqlx::query(
            "INSERT INTO security_project (name, description) VALUES ($1, $2) RETURNING id"
        )
        .bind("Performance Test Project")
        .bind("Performance Test Description")
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
        .bind("perf_tool")
        .bind("1.0.0")
        .bind("perf_viewer")
        .bind(serde_json::json!({"type": "circle", "x": 100, "y": 200, "radius": 50}))
        .bind("Performance test annotation")
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
    async fn test_large_file_upload_performance() {
        let (app, pool) = setup_test_app().await;
        let (annotation_id, user_id, project_id) = create_test_data(&pool).await;

        // Create mask group
        let mask_group_req = CreateMaskGroupRequest {
            annotation_id: annotation_id,
            group_name: Some("Large File Test Group".to_string()),
            model_name: Some("LargeFileModel".to_string()),
            version: Some("1.0.0".to_string()),
            modality: Some("CT".to_string()),
            slice_count: 1000, // Large number of slices
            mask_type: "segmentation".to_string(),
            description: Some("Large file performance test".to_string()),
        };

        let start = Instant::now();
        let req = test::TestRequest::post()
            .uri(&format!("/api/annotations/{}/mask-groups", annotation_id))
            .set_json(&mask_group_req)
            .to_request();

        let resp = test::call_service(&app, req).await;
        let duration = start.elapsed();
        
        assert_eq!(resp.status(), 201);
        assert!(duration < Duration::from_millis(500), "Mask group creation took too long: {:?}", duration);

        // Cleanup
        cleanup_test_data(&pool, user_id, project_id).await;
    }

    #[actix_web::test]
    async fn test_concurrent_upload_performance() {
        let (app, pool) = setup_test_app().await;
        let (annotation_id, user_id, project_id) = create_test_data(&pool).await;

        // Create mask group first
        let mask_group_req = CreateMaskGroupRequest {
            annotation_id: annotation_id,
            group_name: Some("Concurrent Test Group".to_string()),
            model_name: Some("ConcurrentModel".to_string()),
            version: Some("1.0.0".to_string()),
            modality: Some("CT".to_string()),
            slice_count: 100,
            mask_type: "segmentation".to_string(),
            description: Some("Concurrent upload test".to_string()),
        };

        let req = test::TestRequest::post()
            .uri(&format!("/api/annotations/{}/mask-groups", annotation_id))
            .set_json(&mask_group_req)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201);

        let body: serde_json::Value = test::read_body_json(resp).await;
        let mask_group_id = body["id"].as_i64().unwrap() as i32;

        // Test concurrent mask creation
        let start = Instant::now();
        let mut handles = vec![];

        for i in 0..10 {
            let app_clone = app.clone();
            let annotation_id_clone = annotation_id;
            let mask_group_id_clone = mask_group_id;
            
            let handle = tokio::spawn(async move {
                let mask_req = CreateMaskRequest {
                    mask_group_id: mask_group_id_clone,
                    file_path: format!("masks/concurrent_test_{}.png", i),
                    mime_type: Some("image/png".to_string()),
                    slice_index: Some(i),
                    sop_instance_uid: Some(format!("1.2.3.4.5.6.7.8.9.1.{}", i)),
                    label_name: Some(format!("concurrent_label_{}", i)),
                    file_size: Some(1024 * 1024), // 1MB per file
                    checksum: Some(format!("md5-checksum-{}", i)),
                    width: Some(512),
                    height: Some(512),
                };

                let req = test::TestRequest::post()
                    .uri(&format!("/api/annotations/{}/mask-groups/{}/masks", annotation_id_clone, mask_group_id_clone))
                    .set_json(&mask_req)
                    .to_request();

                test::call_service(&app_clone, req).await
            });
            
            handles.push(handle);
        }

        // Wait for all concurrent operations to complete
        let results = futures::future::join_all(handles).await;
        let duration = start.elapsed();

        // Verify all operations succeeded
        for result in results {
            let resp = result.expect("Concurrent operation failed");
            assert_eq!(resp.status(), 201);
        }

        assert!(duration < Duration::from_secs(5), "Concurrent uploads took too long: {:?}", duration);
        println!("Concurrent upload performance: {:?} for 10 operations", duration);

        // Cleanup
        cleanup_test_data(&pool, user_id, project_id).await;
    }

    #[actix_web::test]
    async fn test_bulk_mask_creation_performance() {
        let (app, pool) = setup_test_app().await;
        let (annotation_id, user_id, project_id) = create_test_data(&pool).await;

        // Create mask group
        let mask_group_req = CreateMaskGroupRequest {
            annotation_id: annotation_id,
            group_name: Some("Bulk Test Group".to_string()),
            model_name: Some("BulkModel".to_string()),
            version: Some("1.0.0".to_string()),
            modality: Some("CT".to_string()),
            slice_count: 100,
            mask_type: "segmentation".to_string(),
            description: Some("Bulk creation test".to_string()),
        };

        let req = test::TestRequest::post()
            .uri(&format!("/api/annotations/{}/mask-groups", annotation_id))
            .set_json(&mask_group_req)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201);

        let body: serde_json::Value = test::read_body_json(resp).await;
        let mask_group_id = body["id"].as_i64().unwrap() as i32;

        // Test bulk mask creation
        let start = Instant::now();
        let mut success_count = 0;

        for i in 0..50 {
            let mask_req = CreateMaskRequest {
                mask_group_id: mask_group_id,
                file_path: format!("masks/bulk_test_{}.png", i),
                mime_type: Some("image/png".to_string()),
                slice_index: Some(i),
                sop_instance_uid: Some(format!("1.2.3.4.5.6.7.8.9.1.{}", i)),
                label_name: Some(format!("bulk_label_{}", i)),
                file_size: Some(512 * 1024), // 512KB per file
                checksum: Some(format!("md5-bulk-{}", i)),
                width: Some(256),
                height: Some(256),
            };

            let req = test::TestRequest::post()
                .uri(&format!("/api/annotations/{}/mask-groups/{}/masks", annotation_id, mask_group_id))
                .set_json(&mask_req)
                .to_request();

            let resp = test::call_service(&app, req).await;
            if resp.status() == 201 {
                success_count += 1;
            }
        }

        let duration = start.elapsed();
        
        assert_eq!(success_count, 50, "Not all bulk operations succeeded");
        assert!(duration < Duration::from_secs(10), "Bulk creation took too long: {:?}", duration);
        println!("Bulk creation performance: {:?} for 50 masks", duration);

        // Cleanup
        cleanup_test_data(&pool, user_id, project_id).await;
    }

    #[actix_web::test]
    async fn test_signed_url_generation_performance() {
        let (app, pool) = setup_test_app().await;
        let (annotation_id, user_id, project_id) = create_test_data(&pool).await;

        // Create mask group
        let mask_group_req = CreateMaskGroupRequest {
            annotation_id: annotation_id,
            group_name: Some("URL Test Group".to_string()),
            model_name: Some("URLModel".to_string()),
            version: Some("1.0.0".to_string()),
            modality: Some("CT".to_string()),
            slice_count: 100,
            mask_type: "segmentation".to_string(),
            description: Some("URL generation test".to_string()),
        };

        let req = test::TestRequest::post()
            .uri(&format!("/api/annotations/{}/mask-groups", annotation_id))
            .set_json(&mask_group_req)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201);

        let body: serde_json::Value = test::read_body_json(resp).await;
        let mask_group_id = body["id"].as_i64().unwrap() as i32;

        // Test signed URL generation performance
        let start = Instant::now();
        let mut url_count = 0;

        for i in 0..20 {
            let upload_req = SignedUrlRequest {
                filename: format!("test_file_{}.png", i),
                mime_type: "image/png".to_string(),
                file_size: Some(102400),
                slice_index: Some(i),
                sop_instance_uid: Some(format!("1.2.3.4.5.6.7.8.9.1.{}", i)),
                label_name: Some(format!("test_label_{}", i)),
                mask_group_id: mask_group_id,
                ttl_seconds: Some(3600),
            };

            let req = test::TestRequest::post()
                .uri(&format!("/api/annotations/{}/mask-groups/{}/upload-url", annotation_id, mask_group_id))
                .set_json(&upload_req)
                .to_request();

            let resp = test::call_service(&app, req).await;
            if resp.status() == 200 {
                url_count += 1;
            }
        }

        let duration = start.elapsed();
        
        assert_eq!(url_count, 20, "Not all URL generations succeeded");
        assert!(duration < Duration::from_secs(3), "URL generation took too long: {:?}", duration);
        println!("URL generation performance: {:?} for 20 URLs", duration);

        // Cleanup
        cleanup_test_data(&pool, user_id, project_id).await;
    }

    #[actix_web::test]
    async fn test_database_query_performance() {
        let (app, pool) = setup_test_app().await;
        let (annotation_id, user_id, project_id) = create_test_data(&pool).await;

        // Create multiple mask groups and masks for query performance testing
        let mut mask_group_ids = vec![];
        
        for i in 0..10 {
            let mask_group_req = CreateMaskGroupRequest {
                annotation_id: annotation_id,
                group_name: Some(format!("Query Test Group {}", i)),
                model_name: Some(format!("QueryModel{}", i)),
                version: Some("1.0.0".to_string()),
                modality: Some("CT".to_string()),
                slice_count: 50,
                mask_type: "segmentation".to_string(),
                description: Some(format!("Query test group {}", i)),
            };

            let req = test::TestRequest::post()
                .uri(&format!("/api/annotations/{}/mask-groups", annotation_id))
                .set_json(&mask_group_req)
                .to_request();

            let resp = test::call_service(&app, req).await;
            assert_eq!(resp.status(), 201);

            let body: serde_json::Value = test::read_body_json(resp).await;
            let mask_group_id = body["id"].as_i64().unwrap() as i32;
            mask_group_ids.push(mask_group_id);

            // Add some masks to each group
            for j in 0..5 {
                let mask_req = CreateMaskRequest {
                    mask_group_id: mask_group_id,
                    file_path: format!("masks/query_test_{}_{}.png", i, j),
                    mime_type: Some("image/png".to_string()),
                    slice_index: Some(j),
                    sop_instance_uid: Some(format!("1.2.3.4.5.6.7.8.9.{}.{}", i, j)),
                    label_name: Some(format!("query_label_{}_{}", i, j)),
                    file_size: Some(256 * 1024),
                    checksum: Some(format!("md5-query-{}-{}", i, j)),
                    width: Some(128),
                    height: Some(128),
                };

                let req = test::TestRequest::post()
                    .uri(&format!("/api/annotations/{}/mask-groups/{}/masks", annotation_id, mask_group_id))
                    .set_json(&mask_req)
                    .to_request();

                let resp = test::call_service(&app, req).await;
                assert_eq!(resp.status(), 201);
            }
        }

        // Test query performance
        let start = Instant::now();
        
        // Test mask group listing
        let req = test::TestRequest::get()
            .uri(&format!("/api/annotations/{}/mask-groups", annotation_id))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        // Test mask listing for each group
        for mask_group_id in &mask_group_ids {
            let req = test::TestRequest::get()
                .uri(&format!("/api/annotations/{}/mask-groups/{}/masks", annotation_id, mask_group_id))
                .to_request();

            let resp = test::call_service(&app, req).await;
            assert_eq!(resp.status(), 200);
        }

        let duration = start.elapsed();
        
        assert!(duration < Duration::from_secs(2), "Database queries took too long: {:?}", duration);
        println!("Database query performance: {:?} for 10 groups with 5 masks each", duration);

        // Cleanup
        cleanup_test_data(&pool, user_id, project_id).await;
    }

    #[actix_web::test]
    async fn test_memory_usage_under_load() {
        let (app, pool) = setup_test_app().await;
        let (annotation_id, user_id, project_id) = create_test_data(&pool).await;

        // Create mask group
        let mask_group_req = CreateMaskGroupRequest {
            annotation_id: annotation_id,
            group_name: Some("Memory Test Group".to_string()),
            model_name: Some("MemoryModel".to_string()),
            version: Some("1.0.0".to_string()),
            modality: Some("CT".to_string()),
            slice_count: 1000,
            mask_type: "segmentation".to_string(),
            description: Some("Memory usage test".to_string()),
        };

        let req = test::TestRequest::post()
            .uri(&format!("/api/annotations/{}/mask-groups", annotation_id))
            .set_json(&mask_group_req)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201);

        let body: serde_json::Value = test::read_body_json(resp).await;
        let mask_group_id = body["id"].as_i64().unwrap() as i32;

        // Simulate high load with many concurrent operations
        let start = Instant::now();
        let mut handles = vec![];

        for i in 0..100 {
            let app_clone = app.clone();
            let annotation_id_clone = annotation_id;
            let mask_group_id_clone = mask_group_id;
            
            let handle = tokio::spawn(async move {
                // Mix of different operations
                match i % 4 {
                    0 => {
                        // Create mask
                        let mask_req = CreateMaskRequest {
                            mask_group_id: mask_group_id,
                            file_path: format!("masks/memory_test_{}.png", i),
                            mime_type: Some("image/png".to_string()),
                            slice_index: Some(i),
                            sop_instance_uid: Some(format!("1.2.3.4.5.6.7.8.9.1.{}", i)),
                            label_name: Some(format!("memory_label_{}", i)),
                            file_size: Some(1024 * 1024), // 1MB
                            checksum: Some(format!("md5-memory-{}", i)),
                            width: Some(512),
                            height: Some(512),
                        };

                        let req = test::TestRequest::post()
                            .uri(&format!("/api/annotations/{}/mask-groups/{}/masks", annotation_id_clone, mask_group_id_clone))
                            .set_json(&mask_req)
                            .to_request();

                        test::call_service(&app_clone, req).await
                    },
                    1 => {
                        // Generate upload URL
                        let upload_req = SignedUrlRequest {
                            filename: format!("upload_{}.png", i),
                            mime_type: "image/png".to_string(),
                            file_size: Some(102400),
                            slice_index: Some(i),
                            sop_instance_uid: Some(format!("1.2.3.4.5.6.7.8.9.1.{}", i)),
                            label_name: Some(format!("upload_label_{}", i)),
                            mask_group_id: mask_group_id,
                            ttl_seconds: Some(3600),
                        };

                        let req = test::TestRequest::post()
                            .uri(&format!("/api/annotations/{}/mask-groups/{}/upload-url", annotation_id_clone, mask_group_id_clone))
                            .set_json(&upload_req)
                            .to_request();

                        test::call_service(&app_clone, req).await
                    },
                    2 => {
                        // List masks
                        let req = test::TestRequest::get()
                            .uri(&format!("/api/annotations/{}/mask-groups/{}/masks", annotation_id_clone, mask_group_id_clone))
                            .to_request();

                        test::call_service(&app_clone, req).await
                    },
                    _ => {
                        // Get mask group
                        let req = test::TestRequest::get()
                            .uri(&format!("/api/annotations/{}/mask-groups/{}", annotation_id_clone, mask_group_id_clone))
                            .to_request();

                        test::call_service(&app_clone, req).await
                    }
                }
            });
            
            handles.push(handle);
        }

        // Wait for all operations to complete with timeout
        let result = timeout(Duration::from_secs(30), futures::future::join_all(handles)).await;
        let duration = start.elapsed();

        match result {
            Ok(results) => {
                let mut success_count = 0;
                for result in results {
                    if let Ok(resp) = result {
                        if resp.status() == 200 || resp.status() == 201 {
                            success_count += 1;
                        }
                    }
                }
                
                println!("Memory load test: {} successful operations out of 100 in {:?}", success_count, duration);
                assert!(success_count > 80, "Too many operations failed under load: {}", success_count);
            },
            Err(_) => {
                panic!("Memory load test timed out after 30 seconds");
            }
        }

        // Cleanup
        cleanup_test_data(&pool, user_id, project_id).await;
    }
}
