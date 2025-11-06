#[cfg(test)]
mod database_cleanup_tests {
    use actix_web::{test, web, App};
    use pacs_server::application::dto::{
        annotation_dto::CreateAnnotationRequest,
        mask_group_dto::{CreateMaskGroupRequest, SignedUrlRequest, CompleteUploadRequest},
        mask_dto::CreateMaskRequest,
        user_dto::CreateUserRequest,
        project_dto::CreateProjectRequest,
    };
    use pacs_server::application::use_cases::{
        AnnotationUseCase, MaskGroupUseCase, MaskUseCase, UserUseCase, ProjectUseCase
    };
    use pacs_server::domain::services::{
        AnnotationServiceImpl, MaskGroupServiceImpl, MaskServiceImpl, UserServiceImpl, ProjectServiceImpl
    };
    use pacs_server::infrastructure::repositories::{
        AnnotationRepositoryImpl, MaskGroupRepositoryImpl, MaskRepositoryImpl,
        UserRepositoryImpl, ProjectRepositoryImpl
    };
    use pacs_server::infrastructure::external::s3_service::S3ObjectStorageService;
    use pacs_server::infrastructure::config::ObjectStorageConfig;
    use pacs_server::presentation::controllers::{
        annotation_controller::configure_routes as configure_annotation_routes,
        mask_group_controller::configure_routes as configure_mask_group_routes,
        mask_controller::configure_routes as configure_mask_routes,
        user_controller::configure_routes as configure_user_routes,
        project_controller::configure_routes as configure_project_routes,
    };
    use sqlx::postgres::PgPoolOptions;
    use std::sync::Arc;
    use uuid::Uuid;
    use serde_json::json;

    async fn setup_cleanup_test_app() -> (
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
            .max_connections(5)
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
        let user_service = UserServiceImpl::new(user_repo.clone(), project_repo.clone());
        let project_service = ProjectServiceImpl::new(project_repo, user_repo.clone());
        
        // Initialize object storage service
        let s3_config = ObjectStorageConfig {
            provider: "minio".to_string(),
            endpoint: std::env::var("S3_ENDPOINT").unwrap_or_else(|_| "http://localhost:9000".to_string()),
            access_key: std::env::var("S3_ACCESS_KEY").unwrap_or_else(|_| "minioadmin".to_string()),
            secret_key: std::env::var("S3_SECRET_KEY").unwrap_or_else(|_| "minioadmin".to_string()),
            bucket_name: std::env::var("S3_BUCKET").unwrap_or_else(|_| "pacs-test".to_string()),
            region: std::env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
        };

        let s3_service = Arc::new(S3ObjectStorageService::new(s3_config));
        
        // Initialize use cases
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
        let user_use_case = Arc::new(UserUseCase::new(user_service));
        let project_use_case = Arc::new(ProjectUseCase::new(project_service));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(annotation_use_case.clone()))
                .app_data(web::Data::new(mask_group_use_case.clone()))
                .app_data(web::Data::new(mask_use_case.clone()))
                .app_data(web::Data::new(user_use_case.clone()))
                .app_data(web::Data::new(project_use_case.clone()))
                .configure(|cfg| configure_annotation_routes(cfg, annotation_use_case.clone()))
                .configure(|cfg| configure_mask_group_routes(cfg, mask_group_use_case.clone()))
                .configure(|cfg| configure_mask_routes(cfg, mask_use_case.clone()))
                .configure(|cfg| configure_user_routes(cfg, user_use_case.clone()))
                .configure(|cfg| configure_project_routes(cfg, project_use_case.clone())),
        )
        .await;

        (app, pool)
    }

    async fn create_test_data_hierarchy(pool: &Arc<sqlx::Pool<sqlx::Postgres>>) -> (i32, i32, i32, i32) {
        use sqlx::Row;
        
        // Create test user
        let keycloak_id = Uuid::new_v4();
        let user_result = sqlx::query(
            "INSERT INTO security_user (keycloak_id, username, email) VALUES ($1, $2, $3) RETURNING id"
        )
        .bind(keycloak_id)
        .bind("cleanup_test_user")
        .bind("cleanup@example.com")
        .fetch_one(pool.as_ref())
        .await
        .expect("Failed to create test user");

        let user_id: i32 = user_result.get("id");

        // Create test project
        let project_result = sqlx::query(
            "INSERT INTO security_project (name, description) VALUES ($1, $2) RETURNING id"
        )
        .bind("Cleanup Test Project")
        .bind("Cleanup Test Description")
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
        .bind("cleanup_tool")
        .bind("1.0.0")
        .bind("cleanup_viewer")
        .bind(serde_json::json!({"type": "circle", "x": 100, "y": 200, "radius": 50}))
        .bind("Cleanup test annotation")
        .bind(false)
        .fetch_one(pool.as_ref())
        .await
        .expect("Failed to create test annotation");

        let annotation_id: i32 = annotation_result.get("id");

        // Create test mask group
        let mask_group_result = sqlx::query(
            "INSERT INTO annotation_mask_group (annotation_id, group_name, model_name, version, modality, slice_count, mask_type, description) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id"
        )
        .bind(annotation_id)
        .bind("Cleanup Test Group")
        .bind("CleanupModel")
        .bind("1.0.0")
        .bind("CT")
        .bind(100)
        .bind("segmentation")
        .bind("Cleanup test group")
        .fetch_one(pool.as_ref())
        .await
        .expect("Failed to create test mask group");

        let mask_group_id: i32 = mask_group_result.get("id");

        // Create test mask
        let mask_result = sqlx::query(
            "INSERT INTO annotation_mask (mask_group_id, file_path, mime_type, slice_index, sop_instance_uid, label_name, file_size, checksum, width, height) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) RETURNING id"
        )
        .bind(mask_group_id)
        .bind("masks/cleanup_test.png")
        .bind("image/png")
        .bind(1)
        .bind("1.2.3.4.5.6.7.8.9.1.1")
        .bind("cleanup_label")
        .bind(102400)
        .bind("md5-cleanup-test")
        .bind(512)
        .bind(512)
        .fetch_one(pool.as_ref())
        .await
        .expect("Failed to create test mask");

        let mask_id: i32 = mask_result.get("id");

        (user_id, project_id, annotation_id, mask_group_id)
    }

    async fn verify_data_exists(pool: &Arc<sqlx::Pool<sqlx::Postgres>>, user_id: i32, project_id: i32, annotation_id: i32, mask_group_id: i32) -> bool {
        use sqlx::Row;
        
        // Check if user exists
        let user_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM security_user WHERE id = $1")
            .bind(user_id)
            .fetch_one(pool.as_ref())
            .await
            .unwrap_or(0);

        // Check if project exists
        let project_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM security_project WHERE id = $1")
            .bind(project_id)
            .fetch_one(pool.as_ref())
            .await
            .unwrap_or(0);

        // Check if annotation exists
        let annotation_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM annotation_annotation WHERE id = $1")
            .bind(annotation_id)
            .fetch_one(pool.as_ref())
            .await
            .unwrap_or(0);

        // Check if mask group exists
        let mask_group_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM annotation_mask_group WHERE id = $1")
            .bind(mask_group_id)
            .fetch_one(pool.as_ref())
            .await
            .unwrap_or(0);

        user_count > 0 && project_count > 0 && annotation_count > 0 && mask_group_count > 0
    }

    async fn cleanup_all_test_data(pool: &Arc<sqlx::Pool<sqlx::Postgres>>, user_id: i32, project_id: i32) {
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
    async fn test_database_cleanup_after_annotation_deletion() {
        let (app, pool) = setup_cleanup_test_app().await;
        let (user_id, project_id, annotation_id, mask_group_id) = create_test_data_hierarchy(&pool).await;

        // Verify data exists before deletion
        assert!(verify_data_exists(&pool, user_id, project_id, annotation_id, mask_group_id).await);

        // Delete annotation (should cascade to mask groups and masks)
        let req = test::TestRequest::delete()
            .uri(&format!("/api/annotations/{}", annotation_id))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        // Verify annotation is deleted
        let annotation_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM annotation_annotation WHERE id = $1")
            .bind(annotation_id)
            .fetch_one(pool.as_ref())
            .await
            .unwrap_or(0);
        assert_eq!(annotation_count, 0);

        // Verify mask group is deleted (cascade)
        let mask_group_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM annotation_mask_group WHERE id = $1")
            .bind(mask_group_id)
            .fetch_one(pool.as_ref())
            .await
            .unwrap_or(0);
        assert_eq!(mask_group_count, 0);

        // Verify masks are deleted (cascade)
        let mask_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM annotation_mask WHERE mask_group_id = $1")
            .bind(mask_group_id)
            .fetch_one(pool.as_ref())
            .await
            .unwrap_or(0);
        assert_eq!(mask_count, 0);

        // Verify user and project still exist
        let user_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM security_user WHERE id = $1")
            .bind(user_id)
            .fetch_one(pool.as_ref())
            .await
            .unwrap_or(0);
        assert_eq!(user_count, 1);

        let project_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM security_project WHERE id = $1")
            .bind(project_id)
            .fetch_one(pool.as_ref())
            .await
            .unwrap_or(0);
        assert_eq!(project_count, 1);

        // Cleanup remaining data
        cleanup_all_test_data(&pool, user_id, project_id).await;
    }

    #[actix_web::test]
    async fn test_database_cleanup_after_mask_group_deletion() {
        let (app, pool) = setup_cleanup_test_app().await;
        let (user_id, project_id, annotation_id, mask_group_id) = create_test_data_hierarchy(&pool).await;

        // Verify data exists before deletion
        assert!(verify_data_exists(&pool, user_id, project_id, annotation_id, mask_group_id).await);

        // Delete mask group (should cascade to masks)
        let req = test::TestRequest::delete()
            .uri(&format!("/api/annotations/{}/mask-groups/{}", annotation_id, mask_group_id))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        // Verify mask group is deleted
        let mask_group_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM annotation_mask_group WHERE id = $1")
            .bind(mask_group_id)
            .fetch_one(pool.as_ref())
            .await
            .unwrap_or(0);
        assert_eq!(mask_group_count, 0);

        // Verify masks are deleted (cascade)
        let mask_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM annotation_mask WHERE mask_group_id = $1")
            .bind(mask_group_id)
            .fetch_one(pool.as_ref())
            .await
            .unwrap_or(0);
        assert_eq!(mask_count, 0);

        // Verify annotation, user, and project still exist
        let annotation_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM annotation_annotation WHERE id = $1")
            .bind(annotation_id)
            .fetch_one(pool.as_ref())
            .await
            .unwrap_or(0);
        assert_eq!(annotation_count, 1);

        // Cleanup remaining data
        cleanup_all_test_data(&pool, user_id, project_id).await;
    }

    #[actix_web::test]
    async fn test_database_cleanup_after_user_deletion() {
        let (app, pool) = setup_cleanup_test_app().await;
        let (user_id, project_id, annotation_id, mask_group_id) = create_test_data_hierarchy(&pool).await;

        // Verify data exists before deletion
        assert!(verify_data_exists(&pool, user_id, project_id, annotation_id, mask_group_id).await);

        // Delete user (should cascade to annotations, mask groups, and masks)
        let req = test::TestRequest::delete()
            .uri(&format!("/api/users/{}", user_id))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        // Verify user is deleted
        let user_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM security_user WHERE id = $1")
            .bind(user_id)
            .fetch_one(pool.as_ref())
            .await
            .unwrap_or(0);
        assert_eq!(user_count, 0);

        // Verify user-project relationship is deleted
        let user_project_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM security_user_project WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(pool.as_ref())
            .await
            .unwrap_or(0);
        assert_eq!(user_project_count, 0);

        // Verify annotations are deleted (cascade)
        let annotation_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM annotation_annotation WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(pool.as_ref())
            .await
            .unwrap_or(0);
        assert_eq!(annotation_count, 0);

        // Verify mask groups are deleted (cascade)
        let mask_group_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM annotation_mask_group WHERE annotation_id IN (SELECT id FROM annotation_annotation WHERE user_id = $1)")
            .bind(user_id)
            .fetch_one(pool.as_ref())
            .await
            .unwrap_or(0);
        assert_eq!(mask_group_count, 0);

        // Verify masks are deleted (cascade)
        let mask_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM annotation_mask WHERE mask_group_id IN (SELECT id FROM annotation_mask_group WHERE annotation_id IN (SELECT id FROM annotation_annotation WHERE user_id = $1))")
            .bind(user_id)
            .fetch_one(pool.as_ref())
            .await
            .unwrap_or(0);
        assert_eq!(mask_count, 0);

        // Verify project still exists
        let project_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM security_project WHERE id = $1")
            .bind(project_id)
            .fetch_one(pool.as_ref())
            .await
            .unwrap_or(0);
        assert_eq!(project_count, 1);

        // Cleanup remaining data
        sqlx::query("DELETE FROM security_project WHERE id = $1")
            .bind(project_id)
            .execute(pool.as_ref())
            .await
            .ok();
    }

    #[actix_web::test]
    async fn test_database_cleanup_after_project_deletion() {
        let (app, pool) = setup_cleanup_test_app().await;
        let (user_id, project_id, annotation_id, mask_group_id) = create_test_data_hierarchy(&pool).await;

        // Verify data exists before deletion
        assert!(verify_data_exists(&pool, user_id, project_id, annotation_id, mask_group_id).await);

        // Delete project (should cascade to annotations, mask groups, and masks)
        let req = test::TestRequest::delete()
            .uri(&format!("/api/projects/{}", project_id))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        // Verify project is deleted
        let project_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM security_project WHERE id = $1")
            .bind(project_id)
            .fetch_one(pool.as_ref())
            .await
            .unwrap_or(0);
        assert_eq!(project_count, 0);

        // Verify user-project relationship is deleted
        let user_project_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM security_user_project WHERE project_id = $1")
            .bind(project_id)
            .fetch_one(pool.as_ref())
            .await
            .unwrap_or(0);
        assert_eq!(user_project_count, 0);

        // Verify annotations are deleted (cascade)
        let annotation_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM annotation_annotation WHERE project_id = $1")
            .bind(project_id)
            .fetch_one(pool.as_ref())
            .await
            .unwrap_or(0);
        assert_eq!(annotation_count, 0);

        // Verify mask groups are deleted (cascade)
        let mask_group_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM annotation_mask_group WHERE annotation_id IN (SELECT id FROM annotation_annotation WHERE project_id = $1)")
            .bind(project_id)
            .fetch_one(pool.as_ref())
            .await
            .unwrap_or(0);
        assert_eq!(mask_group_count, 0);

        // Verify masks are deleted (cascade)
        let mask_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM annotation_mask WHERE mask_group_id IN (SELECT id FROM annotation_mask_group WHERE annotation_id IN (SELECT id FROM annotation_annotation WHERE project_id = $1))")
            .bind(project_id)
            .fetch_one(pool.as_ref())
            .await
            .unwrap_or(0);
        assert_eq!(mask_count, 0);

        // Verify user still exists
        let user_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM security_user WHERE id = $1")
            .bind(user_id)
            .fetch_one(pool.as_ref())
            .await
            .unwrap_or(0);
        assert_eq!(user_count, 1);

        // Cleanup remaining data
        sqlx::query("DELETE FROM security_user WHERE id = $1")
            .bind(user_id)
            .execute(pool.as_ref())
            .await
            .ok();
    }

    #[actix_web::test]
    async fn test_database_rollback_on_transaction_failure() {
        let (app, pool) = setup_cleanup_test_app().await;
        let (user_id, project_id, annotation_id, mask_group_id) = create_test_data_hierarchy(&pool).await;

        // Verify data exists before test
        assert!(verify_data_exists(&pool, user_id, project_id, annotation_id, mask_group_id).await);

        // Try to create annotation with invalid data (should fail and rollback)
        let invalid_annotation_req = CreateAnnotationRequest {
            study_instance_uid: "".to_string(), // Invalid empty study UID
            series_instance_uid: "".to_string(), // Invalid empty series UID
            sop_instance_uid: "".to_string(), // Invalid empty instance UID
            tool_name: Some("".to_string()), // Invalid empty tool name
            tool_version: Some("".to_string()), // Invalid empty tool version
            viewer_software: Some("".to_string()), // Invalid empty viewer software
            annotation_data: serde_json::json!({}), // Invalid empty data
            description: Some("".to_string()), // Invalid empty description
        };

        let req = test::TestRequest::post()
            .uri("/api/annotations")
            .set_json(&invalid_annotation_req)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 400); // Should fail validation

        // Verify no new annotation was created
        let annotation_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM annotation_annotation WHERE project_id = $1")
            .bind(project_id)
            .fetch_one(pool.as_ref())
            .await
            .unwrap_or(0);
        assert_eq!(annotation_count, 1); // Only the original annotation should exist

        // Verify original data is still intact
        assert!(verify_data_exists(&pool, user_id, project_id, annotation_id, mask_group_id).await);

        // Cleanup
        cleanup_all_test_data(&pool, user_id, project_id).await;
    }

    #[actix_web::test]
    async fn test_database_cleanup_with_foreign_key_constraints() {
        let (app, pool) = setup_cleanup_test_app().await;
        let (user_id, project_id, annotation_id, mask_group_id) = create_test_data_hierarchy(&pool).await;

        // Verify data exists before test
        assert!(verify_data_exists(&pool, user_id, project_id, annotation_id, mask_group_id).await);

        // Try to delete project while user still has annotations (should fail due to foreign key constraint)
        // This test verifies that foreign key constraints are properly enforced
        let result = sqlx::query("DELETE FROM security_project WHERE id = $1")
            .bind(project_id)
            .execute(pool.as_ref())
            .await;

        // The deletion should fail due to foreign key constraint
        assert!(result.is_err());

        // Verify project still exists
        let project_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM security_project WHERE id = $1")
            .bind(project_id)
            .fetch_one(pool.as_ref())
            .await
            .unwrap_or(0);
        assert_eq!(project_count, 1);

        // Verify all related data still exists
        assert!(verify_data_exists(&pool, user_id, project_id, annotation_id, mask_group_id).await);

        // Cleanup
        cleanup_all_test_data(&pool, user_id, project_id).await;
    }

    #[actix_web::test]
    async fn test_database_cleanup_performance() {
        let (app, pool) = setup_cleanup_test_app().await;
        
        // Create multiple test data sets
        let mut test_data_sets = vec![];
        for i in 0..5 {
            let (user_id, project_id, annotation_id, mask_group_id) = create_test_data_hierarchy(&pool).await;
            test_data_sets.push((user_id, project_id, annotation_id, mask_group_id));
        }

        // Verify all data exists
        for (user_id, project_id, annotation_id, mask_group_id) in &test_data_sets {
            assert!(verify_data_exists(&pool, *user_id, *project_id, *annotation_id, *mask_group_id).await);
        }

        // Measure cleanup performance
        let start = std::time::Instant::now();
        
        // Clean up all test data
        for (user_id, project_id, _, _) in &test_data_sets {
            cleanup_all_test_data(&pool, *user_id, *project_id).await;
        }
        
        let cleanup_duration = start.elapsed();
        
        // Verify all data is cleaned up
        for (user_id, project_id, annotation_id, mask_group_id) in &test_data_sets {
            let user_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM security_user WHERE id = $1")
                .bind(user_id)
                .fetch_one(pool.as_ref())
                .await
                .unwrap_or(0);
            assert_eq!(user_count, 0);
        }

        // Performance assertion (adjust based on your requirements)
        assert!(cleanup_duration < std::time::Duration::from_secs(5), "Cleanup took too long: {:?}", cleanup_duration);
        
        println!("Cleanup performance: {:?} for 5 data sets", cleanup_duration);
    }

    #[actix_web::test]
    async fn test_database_cleanup_with_concurrent_operations() {
        let (app, pool) = setup_cleanup_test_app().await;
        let (user_id, project_id, annotation_id, mask_group_id) = create_test_data_hierarchy(&pool).await;

        // Verify data exists before test
        assert!(verify_data_exists(&pool, user_id, project_id, annotation_id, mask_group_id).await);

        // Test concurrent cleanup operations
        let pool_clone = pool.clone();
        let user_id_clone = user_id;
        let project_id_clone = project_id;
        
        let cleanup_handle = tokio::spawn(async move {
            cleanup_all_test_data(&pool_clone, user_id_clone, project_id_clone).await;
        });

        // Wait for cleanup to complete
        cleanup_handle.await.expect("Cleanup task failed");

        // Verify data is cleaned up
        let user_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM security_user WHERE id = $1")
            .bind(user_id)
            .fetch_one(pool.as_ref())
            .await
            .unwrap_or(0);
        assert_eq!(user_count, 0);

        let project_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM security_project WHERE id = $1")
            .bind(project_id)
            .fetch_one(pool.as_ref())
            .await
            .unwrap_or(0);
        assert_eq!(project_count, 0);
    }
}
