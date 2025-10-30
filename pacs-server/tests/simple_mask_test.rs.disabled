use actix_web::{test, web, App};
use sqlx::PgPool;
use std::sync::Arc;
use pacs_server::{
    application::{
        dto::{
            annotation_dto::CreateAnnotationRequest,
            mask_group_dto::CreateMaskGroupRequest,
        },
        use_cases::{
            annotation_use_case::AnnotationUseCase,
            mask_group_use_case::MaskGroupUseCase,
        },
    },
    domain::{
        entities::{NewAnnotation, NewMaskGroup},
        repositories::{
            annotation_repository::AnnotationRepository,
            mask_group_repository::MaskGroupRepository,
            user_repository::UserRepository,
            project_repository::ProjectRepository,
        },
        services::{
            annotation_service::AnnotationServiceImpl,
            mask_group_service::MaskGroupServiceImpl,
        },
    },
    infrastructure::{
        repositories::{
            annotation_repository_impl::AnnotationRepositoryImpl,
            mask_group_repository_impl::MaskGroupRepositoryImpl,
            user_repository_impl::UserRepositoryImpl,
            project_repository_impl::ProjectRepositoryImpl,
        },
    },
    presentation::controllers::{
        annotation_controller,
        mask_group_controller,
    },
};

// Mock SignedUrlService for testing
use pacs_server::application::services::signed_url_service::SignedUrlService;
use pacs_server::application::dto::signed_url_dto::{SignedUrlRequest, SignedUrlResponse};

#[derive(Clone)]
pub struct MockSignedUrlService;

impl MockSignedUrlService {
    pub fn new() -> Self {
        Self
    }
}

impl SignedUrlService for MockSignedUrlService {
    async fn generate_upload_url(&self, _request: SignedUrlRequest) -> Result<SignedUrlResponse, Box<dyn std::error::Error + Send + Sync>> {
        Ok(SignedUrlResponse {
            upload_url: "https://mock-s3.amazonaws.com/test-bucket/test-file.png?mock-signed-url".to_string(),
            download_url: "https://mock-s3.amazonaws.com/test-bucket/test-file.png".to_string(),
            expires_in: 3600,
            expires_at: "2025-01-18T13:00:00Z".to_string(),
        })
    }

    async fn generate_download_url(&self, _request: SignedUrlRequest) -> Result<SignedUrlResponse, Box<dyn std::error::Error + Send + Sync>> {
        Ok(SignedUrlResponse {
            upload_url: "https://mock-s3.amazonaws.com/test-bucket/test-file.png?mock-signed-url".to_string(),
            download_url: "https://mock-s3.amazonaws.com/test-bucket/test-file.png".to_string(),
            expires_in: 3600,
            expires_at: "2025-01-18T13:00:00Z".to_string(),
        })
    }
}

async fn setup_test_app() -> (App<impl actix_web::dev::ServiceFactory<actix_web::dev::ServiceRequest, Config = (), Response = actix_web::dev::ServiceResponse<actix_web::body::BoxBody>, Error = actix_web::Error, InitError = ()>, actix_web::body::BoxBody>, PgPool) {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://pacs_extension_admin:PacsExtension2024@localhost:5456/pacs_extension".to_string());
    
    let pool = sqlx::PgPool::connect(&database_url).await.unwrap();
    
    // Initialize repositories
    let annotation_repo = Arc::new(AnnotationRepositoryImpl::new(pool.clone()));
    let mask_group_repo = Arc::new(MaskGroupRepositoryImpl::new(pool.clone()));
    let user_repo = Arc::new(UserRepositoryImpl::new(pool.clone()));
    let project_repo = Arc::new(ProjectRepositoryImpl::new(pool.clone()));
    
    // Initialize services
    let annotation_service = Arc::new(AnnotationServiceImpl::new(
        annotation_repo.clone(),
        user_repo.clone(),
        project_repo.clone(),
    ));
    let mask_group_service = Arc::new(MaskGroupServiceImpl::new(
        mask_group_repo.clone(),
        annotation_repo.clone(),
        user_repo.clone(),
    ));
    
    // Mock SignedUrlService
    let signed_url_service = Arc::new(MockSignedUrlService::new());
    
    // Initialize use cases
    let annotation_use_case = Arc::new(AnnotationUseCase::new(annotation_service));
    let mask_group_use_case = Arc::new(MaskGroupUseCase::new(
        mask_group_service,
        signed_url_service,
    ));
    
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(annotation_use_case.clone()))
            .app_data(web::Data::new(mask_group_use_case.clone()))
            .service(
                web::scope("/api")
                    .configure(|cfg| annotation_controller::configure_routes(cfg, annotation_use_case))
                    .configure(|cfg| mask_group_controller::configure_routes(cfg, mask_group_use_case))
            ),
    )
    .await;
    
    (app, pool)
}

async fn create_test_data(pool: &PgPool) -> (i32, i32) {
    use sqlx::Row;
    
    // Create test user
    let user_result = sqlx::query("INSERT INTO security_user (keycloak_id, username, email, created_at) VALUES ($1, $2, $3, NOW()) RETURNING id")
        .bind("test-keycloak-id")
        .bind("testuser")
        .bind("test@example.com")
        .fetch_one(pool)
        .await
        .unwrap();
    
    let user_id: i32 = user_result.get("id");
    
    // Create test project
    let project_result = sqlx::query("INSERT INTO project (name, description, is_active, created_at) VALUES ($1, $2, $3, NOW()) RETURNING id")
        .bind("Test Project")
        .bind("Test project for mask upload")
        .bind(true)
        .fetch_one(pool)
        .await
        .unwrap();
    
    let project_id: i32 = project_result.get("id");
    
    (project_id, user_id)
}

async fn cleanup_test_data(pool: &PgPool, user_id: i32, project_id: i32) {
    let _ = sqlx::query("DELETE FROM annotation_annotation WHERE project_id = $1")
        .bind(project_id)
        .execute(pool)
        .await;
    
    let _ = sqlx::query("DELETE FROM project WHERE id = $1")
        .bind(project_id)
        .execute(pool)
        .await;
    
    let _ = sqlx::query("DELETE FROM security_user WHERE id = $1")
        .bind(user_id)
        .execute(pool)
        .await;
}

#[actix_web::test]
async fn test_mask_upload_workflow() {
    let (app, pool) = setup_test_app().await;
    let (project_id, user_id) = create_test_data(&pool).await;
    
    // Step 1: Create annotation
    let annotation_req = CreateAnnotationRequest {
        project_id,
        user_id,
        study_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.1".to_string(),
        series_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.2".to_string(),
        sop_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.3".to_string(),
        tool_name: Some("AI Segmentation".to_string()),
        tool_version: Some("2.1.0".to_string()),
        viewer_software: Some("OHIF Viewer".to_string()),
        description: Some("AI가 생성한 간 마스크".to_string()),
        measurement_values: None,
        annotation_data: serde_json::json!({
            "type": "segmentation",
            "region": "liver"
        }),
    };
    
    let req = test::TestRequest::post()
        .uri("/api/annotations")
        .set_json(&annotation_req)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);
    
    let annotation_body: serde_json::Value = test::read_body_json(resp).await;
    let annotation_id = annotation_body["id"].as_i64().unwrap() as i32;
    
    // Step 2: Create mask group
    let mask_group_req = CreateMaskGroupRequest {
        annotation_id,
        group_name: Some("Liver Segmentation".to_string()),
        model_name: Some("UNet-3D".to_string()),
        version: Some("2.1.0".to_string()),
        modality: Some("CT".to_string()),
        slice_count: 120,
        mask_type: "segmentation".to_string(),
        description: Some("간 분할 마스크 그룹".to_string()),
    };
    
    let req = test::TestRequest::post()
        .uri(&format!("/api/annotations/{}/mask-groups", annotation_id))
        .set_json(&mask_group_req)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);
    
    let mask_group_body: serde_json::Value = test::read_body_json(resp).await;
    let mask_group_id = mask_group_body["id"].as_i64().unwrap() as i32;
    
    // Step 3: Request upload URL
    let upload_url_req = pacs_server::application::dto::signed_url_dto::SignedUrlRequest {
        file_path: "masks/liver_mask_001.png".to_string(),
        mime_type: "image/png".to_string(),
        expires_in: Some(3600),
        mask_group_id: Some(mask_group_id),
        label_name: Some("liver".to_string()),
        file_size: Some(1024000),
    };
    
    let req = test::TestRequest::post()
        .uri(&format!("/api/annotations/{}/mask-groups/{}/upload-url", annotation_id, mask_group_id))
        .set_json(&upload_url_req)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
    
    let upload_url_body: serde_json::Value = test::read_body_json(resp).await;
    assert!(upload_url_body["upload_url"].as_str().unwrap().contains("mock-s3"));
    
    // Cleanup
    cleanup_test_data(&pool, user_id, project_id).await;
}

#[actix_web::test]
async fn test_mask_upload_workflow_error_handling() {
    let (app, pool) = setup_test_app().await;
    let (project_id, user_id) = create_test_data(&pool).await;
    
    // Test 1: Try to create mask group for non-existent annotation
    let mask_group_req = CreateMaskGroupRequest {
        annotation_id: 999999, // Non-existent annotation ID
        group_name: Some("Test Group".to_string()),
        model_name: Some("Test Model".to_string()),
        version: Some("1.0.0".to_string()),
        modality: Some("CT".to_string()),
        slice_count: 100,
        mask_type: "segmentation".to_string(),
        description: Some("Test description".to_string()),
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
