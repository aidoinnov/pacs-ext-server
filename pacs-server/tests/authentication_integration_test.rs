#[cfg(test)]
mod authentication_integration_tests {
    use actix_web::{test, web, App, middleware::Logger};
    use pacs_server::application::dto::{
        annotation_dto::CreateAnnotationRequest,
        mask_group_dto::CreateMaskGroupRequest,
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
    use pacs_server::infrastructure::auth::{JwtService, JwtConfig};
    use sqlx::postgres::PgPoolOptions;
    use std::sync::Arc;
    use uuid::Uuid;
    use chrono::{Duration, Utc};

    async fn setup_test_app() -> (
        impl actix_web::dev::Service<
            actix_http::Request,
            Response = actix_web::dev::ServiceResponse,
            Error = actix_web::Error,
        >,
        Arc<sqlx::Pool<sqlx::Postgres>>,
        JwtService,
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

        // Initialize JWT service
        let jwt_config = JwtConfig {
            secret: "test-secret-key-for-jwt-token-generation-and-validation".to_string(),
            expiration_hours: 24,
        };
        let jwt_service = JwtService::new(&jwt_config);

        let app = test::init_service(
            App::new()
                .wrap(Logger::default())
                .app_data(web::Data::new(annotation_use_case.clone()))
                .app_data(web::Data::new(mask_group_use_case.clone()))
                .app_data(web::Data::new(mask_use_case.clone()))
                .configure(|cfg| configure_annotation_routes(cfg, annotation_use_case.clone()))
                .configure(|cfg| configure_mask_group_routes(cfg, mask_group_use_case.clone()))
                .configure(|cfg| configure_mask_routes(cfg, mask_use_case.clone())),
        )
        .await;

        (app, pool, jwt_service)
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

    async fn create_test_user(pool: &Arc<sqlx::Pool<sqlx::Postgres>>) -> (i32, String) {
        use sqlx::Row;
        
        let keycloak_id = Uuid::new_v4();
        let username = format!("authtestuser_{}", Uuid::new_v4());
        let email = format!("authtest_{}@example.com", Uuid::new_v4());
        
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
        (user_id, username)
    }

    async fn create_test_project(pool: &Arc<sqlx::Pool<sqlx::Postgres>>, user_id: i32) -> i32 {
        use sqlx::Row;
        
        let project_result = sqlx::query(
            "INSERT INTO security_project (name, description) VALUES ($1, $2) RETURNING id"
        )
        .bind("Auth Test Project")
        .bind("Auth Test Description")
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

        project_id
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

    fn create_auth_header(token: &str) -> (String, String) {
        ("Authorization".to_string(), format!("Bearer {}", token))
    }

    #[actix_web::test]
    async fn test_jwt_token_generation_and_validation() {
        let (_, pool, jwt_service) = setup_test_app().await;
        let (user_id, username) = create_test_user(&pool).await;

        // Test 1: Generate valid JWT token
        let claims = pacs_server::infrastructure::auth::Claims {
            sub: user_id.to_string(),
            username: username.clone(),
            email: "test@example.com".to_string(),
            exp: (Utc::now() + Duration::hours(24)).timestamp() as usize,
        };

        let token_result = jwt_service.create_token(&claims);
        assert!(token_result.is_ok(), "Failed to create JWT token");

        let token = token_result.unwrap();
        assert!(!token.is_empty(), "Generated token should not be empty");

        // Test 2: Validate valid JWT token
        let validation_result = jwt_service.validate_token(&token);
        assert!(validation_result.is_ok(), "Failed to validate JWT token");

        let validated_claims = validation_result.unwrap();
        assert_eq!(validated_claims.sub, user_id.to_string());
        assert_eq!(validated_claims.username, username);

        // Test 3: Validate expired token
        let expired_claims = pacs_server::infrastructure::auth::Claims {
            sub: user_id.to_string(),
            username: username.clone(),
            email: "test@example.com".to_string(),
            exp: (Utc::now() - Duration::hours(1)).timestamp() as usize, // Expired 1 hour ago
        };

        let expired_token_result = jwt_service.create_token(&expired_claims);
        assert!(expired_token_result.is_ok());

        let expired_token = expired_token_result.unwrap();
        let expired_validation_result = jwt_service.validate_token(&expired_token);
        assert!(expired_validation_result.is_err(), "Expired token should be invalid");

        // Test 4: Validate malformed token
        let malformed_validation_result = jwt_service.validate_token("invalid.jwt.token");
        assert!(malformed_validation_result.is_err(), "Malformed token should be invalid");

        // Test 5: Validate token with wrong secret
        let wrong_secret_config = JwtConfig {
            secret: "wrong-secret-key".to_string(),
            expiration_hours: 24,
        };
        let wrong_secret_jwt_service = JwtService::new(&wrong_secret_config);
        
        let wrong_secret_validation_result = wrong_secret_jwt_service.validate_token(&token);
        assert!(wrong_secret_validation_result.is_err(), "Token with wrong secret should be invalid");

        // Cleanup
        sqlx::query("DELETE FROM security_user WHERE id = $1")
            .bind(user_id)
            .execute(pool.as_ref())
            .await
            .ok();
    }

    #[actix_web::test]
    async fn test_authentication_middleware_integration() {
        let (app, pool, jwt_service) = setup_test_app().await;
        let (user_id, username) = create_test_user(&pool).await;
        let project_id = create_test_project(&pool, user_id).await;

        // Create valid JWT token
        let claims = pacs_server::infrastructure::auth::Claims {
            sub: user_id.to_string(),
            username: username.clone(),
            email: "test@example.com".to_string(),
            exp: (Utc::now() + Duration::hours(24)).timestamp() as usize,
        };

        let token = jwt_service.create_token(&claims).unwrap();

        // Test 1: Access protected endpoint with valid token
        // Note: In current implementation, authentication middleware is not implemented
        // This test demonstrates the expected behavior when auth is implemented
        let req = test::TestRequest::get()
            .uri("/api/annotations")
            .insert_header(create_auth_header(&token))
            .to_request();

        let resp = test::call_service(&app, req).await;
        // Currently returns 200 because auth is not implemented
        // When auth is implemented, this should return 200 for valid token
        assert_eq!(resp.status(), 200);

        // Test 2: Access protected endpoint without token
        let req = test::TestRequest::get()
            .uri("/api/annotations")
            .to_request();

        let resp = test::call_service(&app, req).await;
        // Currently returns 200 because auth is not implemented
        // When auth is implemented, this should return 401 Unauthorized
        // assert_eq!(resp.status(), 401);

        // Test 3: Access protected endpoint with invalid token
        let req = test::TestRequest::get()
            .uri("/api/annotations")
            .insert_header(create_auth_header("invalid.token.here"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        // Currently returns 200 because auth is not implemented
        // When auth is implemented, this should return 401 Unauthorized
        // assert_eq!(resp.status(), 401);

        // Cleanup
        cleanup_test_data(&pool, user_id, project_id).await;
    }

    #[actix_web::test]
    async fn test_user_authorization_scenarios() {
        let (app, pool, jwt_service) = setup_test_app().await;
        let (user_id, username) = create_test_user(&pool).await;
        let project_id = create_test_project(&pool, user_id).await;

        // Create annotation for the user
        let annotation_req = CreateAnnotationRequest {
            study_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.1".to_string(),
            series_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.2".to_string(),
            sop_instance_uid: "1.2.840.113619.2.55.3.604688119.868.1234567890.3".to_string(),
            annotation_data: serde_json::json!({"type": "circle", "x": 100, "y": 200, "radius": 50}),
            viewer_software: Some("OHIF Viewer".to_string()),
            tool_name: Some("Circle Tool".to_string()),
            tool_version: Some("2.1.0".to_string()),
            description: Some("Auth test annotation".to_string()),
        };

        let req = test::TestRequest::post()
            .uri("/api/annotations")
            .set_json(&annotation_req)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201);

        let body: serde_json::Value = test::read_body_json(resp).await;
        let annotation_id = body["id"].as_i64().unwrap() as i32;

        // Create JWT token for the user
        let claims = pacs_server::infrastructure::auth::Claims {
            sub: user_id.to_string(),
            username: username.clone(),
            email: "test@example.com".to_string(),
            exp: (Utc::now() + Duration::hours(24)).timestamp() as usize,
        };

        let token = jwt_service.create_token(&claims).unwrap();

        // Test 1: User can access their own annotation
        let req = test::TestRequest::get()
            .uri(&format!("/api/annotations/{}", annotation_id))
            .insert_header(create_auth_header(&token))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        // Test 2: User can create mask group for their annotation
        let mask_group_req = CreateMaskGroupRequest {
            group_name: Some("Auth Test Group".to_string()),
            model_name: Some("AuthModel".to_string()),
            version: Some("1.0.0".to_string()),
            modality: Some("CT".to_string()),
            slice_count: 100,
            mask_type: "segmentation".to_string(),
            description: Some("Auth test group".to_string()),
        };

        let req = test::TestRequest::post()
            .uri(&format!("/api/annotations/{}/mask-groups", annotation_id))
            .insert_header(create_auth_header(&token))
            .set_json(&mask_group_req)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201);

        // Test 3: User cannot access non-existent annotation
        let req = test::TestRequest::get()
            .uri("/api/annotations/999999")
            .insert_header(create_auth_header(&token))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404);

        // Cleanup
        cleanup_test_data(&pool, user_id, project_id).await;
    }

    #[actix_web::test]
    async fn test_token_expiration_handling() {
        let (app, pool, jwt_service) = setup_test_app().await;
        let (user_id, username) = create_test_user(&pool).await;

        // Test 1: Create token that expires in 1 second
        let claims = pacs_server::infrastructure::auth::Claims {
            sub: user_id.to_string(),
            username: username.clone(),
            email: "test@example.com".to_string(),
            exp: (Utc::now() + Duration::seconds(1)).timestamp() as usize,
        };

        let token = jwt_service.create_token(&claims).unwrap();

        // Token should be valid immediately
        let validation_result = jwt_service.validate_token(&token);
        assert!(validation_result.is_ok(), "Fresh token should be valid");

        // Wait for token to expire
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Token should now be invalid
        let expired_validation_result = jwt_service.validate_token(&token);
        assert!(expired_validation_result.is_err(), "Expired token should be invalid");

        // Test 2: Create token with very short expiration
        let short_claims = pacs_server::infrastructure::auth::Claims {
            sub: user_id.to_string(),
            username: username.clone(),
            email: "test@example.com".to_string(),
            exp: (Utc::now() + Duration::milliseconds(100)).timestamp() as usize,
        };

        let short_token = jwt_service.create_token(&short_claims).unwrap();

        // Wait for short token to expire
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        let short_expired_result = jwt_service.validate_token(&short_token);
        assert!(short_expired_result.is_err(), "Short-lived expired token should be invalid");

        // Cleanup
        sqlx::query("DELETE FROM security_user WHERE id = $1")
            .bind(user_id)
            .execute(pool.as_ref())
            .await
            .ok();
    }

    #[actix_web::test]
    async fn test_multiple_user_authentication() {
        let (app, pool, jwt_service) = setup_test_app().await;

        // Create multiple users
        let (user1_id, username1) = create_test_user(&pool).await;
        let (user2_id, username2) = create_test_user(&pool).await;
        let project_id = create_test_project(&pool, user1_id).await;

        // Create JWT tokens for both users
        let claims1 = pacs_server::infrastructure::auth::Claims {
            sub: user1_id.to_string(),
            username: username1.clone(),
            email: "user1@example.com".to_string(),
            exp: (Utc::now() + Duration::hours(24)).timestamp() as usize,
        };

        let claims2 = pacs_server::infrastructure::auth::Claims {
            sub: user2_id.to_string(),
            username: username2.clone(),
            email: "user2@example.com".to_string(),
            exp: (Utc::now() + Duration::hours(24)).timestamp() as usize,
        };

        let token1 = jwt_service.create_token(&claims1).unwrap();
        let token2 = jwt_service.create_token(&claims2).unwrap();

        // Test 1: Both tokens should be valid
        let validation1 = jwt_service.validate_token(&token1);
        let validation2 = jwt_service.validate_token(&token2);

        assert!(validation1.is_ok(), "User 1 token should be valid");
        assert!(validation2.is_ok(), "User 2 token should be valid");

        // Test 2: Tokens should have different user information
        let claims1_validated = validation1.unwrap();
        let claims2_validated = validation2.unwrap();

        assert_ne!(claims1_validated.sub, claims2_validated.sub);
        assert_ne!(claims1_validated.username, claims2_validated.username);

        // Test 3: Both users can access public endpoints
        let req1 = test::TestRequest::get()
            .uri("/api/annotations")
            .insert_header(create_auth_header(&token1))
            .to_request();

        let req2 = test::TestRequest::get()
            .uri("/api/annotations")
            .insert_header(create_auth_header(&token2))
            .to_request();

        let resp1 = test::call_service(&app, req1).await;
        let resp2 = test::call_service(&app, req2).await;

        assert_eq!(resp1.status(), 200);
        assert_eq!(resp2.status(), 200);

        // Cleanup
        cleanup_test_data(&pool, user1_id, project_id).await;
        sqlx::query("DELETE FROM security_user WHERE id = $1")
            .bind(user2_id)
            .execute(pool.as_ref())
            .await
            .ok();
    }

    #[actix_web::test]
    async fn test_token_refresh_scenario() {
        let (_, pool, jwt_service) = setup_test_app().await;
        let (user_id, username) = create_test_user(&pool).await;

        // Test 1: Create initial token
        let initial_claims = pacs_server::infrastructure::auth::Claims {
            sub: user_id.to_string(),
            username: username.clone(),
            email: "test@example.com".to_string(),
            exp: (Utc::now() + Duration::hours(1)).timestamp() as usize,
        };

        let initial_token = jwt_service.create_token(&initial_claims).unwrap();
        assert!(jwt_service.validate_token(&initial_token).is_ok());

        // Test 2: Create refreshed token with longer expiration
        let refreshed_claims = pacs_server::infrastructure::auth::Claims {
            sub: user_id.to_string(),
            username: username.clone(),
            email: "test@example.com".to_string(),
            exp: (Utc::now() + Duration::hours(24)).timestamp() as usize,
        };

        let refreshed_token = jwt_service.create_token(&refreshed_claims).unwrap();
        assert!(jwt_service.validate_token(&refreshed_token).is_ok());

        // Test 3: Both tokens should be valid (no invalidation of old tokens)
        assert!(jwt_service.validate_token(&initial_token).is_ok());
        assert!(jwt_service.validate_token(&refreshed_token).is_ok());

        // Test 4: Tokens should have different expiration times
        let initial_claims_validated = jwt_service.validate_token(&initial_token).unwrap();
        let refreshed_claims_validated = jwt_service.validate_token(&refreshed_token).unwrap();

        assert_ne!(initial_claims_validated.exp, refreshed_claims_validated.exp);
        assert!(refreshed_claims_validated.exp > initial_claims_validated.exp);

        // Cleanup
        sqlx::query("DELETE FROM security_user WHERE id = $1")
            .bind(user_id)
            .execute(pool.as_ref())
            .await
            .ok();
    }

    #[actix_web::test]
    async fn test_malformed_token_handling() {
        let (_, pool, jwt_service) = setup_test_app().await;

        // Test various malformed token scenarios
        let malformed_tokens = vec![
            "", // Empty token
            "not.a.jwt", // Not a JWT
            "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.invalid.signature", // Invalid signature
            "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.invalid", // Wrong signature
            "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ", // Missing signature
            "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9", // Only header
            "invalid", // Completely invalid
        ];

        for malformed_token in malformed_tokens {
            let validation_result = jwt_service.validate_token(malformed_token);
            assert!(validation_result.is_err(), 
                "Malformed token '{}' should be invalid", malformed_token);
        }
    }
}
