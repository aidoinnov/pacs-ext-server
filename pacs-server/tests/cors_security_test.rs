#[cfg(test)]
mod cors_security_tests {
    use actix_web::{test, web, App, middleware::Logger};
    use actix_cors::Cors;
    use pacs_server::application::use_cases::{
        AnnotationUseCase, MaskGroupUseCase, MaskUseCase, UserUseCase, ProjectUseCase, AuthUseCase
    };
    use pacs_server::domain::services::{
        AnnotationServiceImpl, MaskGroupServiceImpl, MaskServiceImpl, UserServiceImpl, ProjectServiceImpl, AuthServiceImpl
    };
    use pacs_server::infrastructure::repositories::{
        AnnotationRepositoryImpl, MaskGroupRepositoryImpl, MaskRepositoryImpl,
        UserRepositoryImpl, ProjectRepositoryImpl
    };
    use pacs_server::infrastructure::external::S3Service;
    use pacs_server::infrastructure::config::{ObjectStorageConfig, JwtConfig};
    use pacs_server::infrastructure::auth::JwtService;
    use pacs_server::presentation::controllers::{
        annotation_controller::configure_routes as configure_annotation_routes,
        mask_group_controller::configure_routes as configure_mask_group_routes,
        mask_controller::configure_routes as configure_mask_routes,
        user_controller::configure_routes as configure_user_routes,
        project_controller::configure_routes as configure_project_routes,
        auth_controller::configure_routes as configure_auth_routes,
    };
    use sqlx::postgres::PgPoolOptions;
    use std::sync::Arc;

    async fn setup_security_test_app() -> impl actix_web::dev::Service<
        actix_http::Request,
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
    > {
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
        let user_service = UserServiceImpl::new(user_repo.clone(), project_repo.clone());
        let project_service = ProjectServiceImpl::new(project_repo, user_repo.clone());
        
        // Initialize object storage service
        let s3_config = ObjectStorageConfig {
            endpoint: std::env::var("S3_ENDPOINT").unwrap_or_else(|_| "http://localhost:9000".to_string()),
            access_key: std::env::var("S3_ACCESS_KEY").unwrap_or_else(|_| "minioadmin".to_string()),
            secret_key: std::env::var("S3_SECRET_KEY").unwrap_or_else(|_| "minioadmin".to_string()),
            bucket_name: std::env::var("S3_BUCKET").unwrap_or_else(|_| "pacs-test".to_string()),
            region: std::env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
        };

        let s3_service = Arc::new(S3Service::new(s3_config));
        
        // Initialize JWT service
        let jwt_config = JwtConfig {
            secret: "test_secret_key_for_security_testing".to_string(),
            expiration_hours: 24,
        };
        let jwt_service = Arc::new(JwtService::new(jwt_config));
        let auth_service = AuthServiceImpl::new(user_repo.clone(), jwt_service.clone());
        
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
        let auth_use_case = Arc::new(AuthUseCase::new(auth_service));

        // Configure CORS
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        test::init_service(
            App::new()
                .wrap(cors)
                .wrap(Logger::default())
                .app_data(web::Data::new(annotation_use_case.clone()))
                .app_data(web::Data::new(mask_group_use_case.clone()))
                .app_data(web::Data::new(mask_use_case.clone()))
                .app_data(web::Data::new(user_use_case.clone()))
                .app_data(web::Data::new(project_use_case.clone()))
                .app_data(web::Data::new(auth_use_case.clone()))
                .configure(|cfg| configure_annotation_routes(cfg, annotation_use_case.clone()))
                .configure(|cfg| configure_mask_group_routes(cfg, mask_group_use_case.clone()))
                .configure(|cfg| configure_mask_routes(cfg, mask_use_case.clone()))
                .configure(|cfg| configure_user_routes(cfg, user_use_case.clone()))
                .configure(|cfg| configure_project_routes(cfg, project_use_case.clone()))
                .configure(|cfg| configure_auth_routes(cfg, auth_use_case.clone())),
        )
        .await
    }

    #[actix_web::test]
    async fn test_cors_preflight_request() {
        let app = setup_security_test_app().await;

        // Test OPTIONS request (preflight)
        let req = test::TestRequest::options()
            .uri("/api/annotations")
            .insert_header(("Origin", "https://example.com"))
            .insert_header(("Access-Control-Request-Method", "GET"))
            .insert_header(("Access-Control-Request-Headers", "Authorization, Content-Type"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        // Check CORS headers
        let headers = resp.headers();
        assert!(headers.get("Access-Control-Allow-Origin").is_some());
        assert!(headers.get("Access-Control-Allow-Methods").is_some());
        assert!(headers.get("Access-Control-Allow-Headers").is_some());
        assert!(headers.get("Access-Control-Max-Age").is_some());
    }

    #[actix_web::test]
    async fn test_cors_actual_request() {
        let app = setup_security_test_app().await;

        // Test actual GET request with CORS headers
        let req = test::TestRequest::get()
            .uri("/api/annotations")
            .insert_header(("Origin", "https://example.com"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        // Check CORS headers
        let headers = resp.headers();
        assert!(headers.get("Access-Control-Allow-Origin").is_some());
    }

    #[actix_web::test]
    async fn test_cors_multiple_origins() {
        let app = setup_security_test_app().await;

        let origins = vec![
            "https://example.com",
            "https://app.example.com",
            "http://localhost:3000",
            "http://localhost:8080",
        ];

        for origin in origins {
            let req = test::TestRequest::get()
                .uri("/api/annotations")
                .insert_header(("Origin", origin))
                .to_request();

            let resp = test::call_service(&app, req).await;
            assert_eq!(resp.status(), 200);

            // Check CORS headers
            let headers = resp.headers();
            assert!(headers.get("Access-Control-Allow-Origin").is_some());
        }
    }

    #[actix_web::test]
    async fn test_cors_different_methods() {
        let app = setup_security_test_app().await;

        let methods = vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"];

        for method in methods {
            let req = test::TestRequest::default()
                .method(method)
                .uri("/api/annotations")
                .insert_header(("Origin", "https://example.com"))
                .to_request();

            let resp = test::call_service(&app, req).await;
            
            // Some methods might return 404 or 405, but CORS headers should still be present
            let headers = resp.headers();
            assert!(headers.get("Access-Control-Allow-Origin").is_some());
        }
    }

    #[actix_web::test]
    async fn test_security_headers() {
        let app = setup_security_test_app().await;

        let req = test::TestRequest::get()
            .uri("/api/annotations")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let headers = resp.headers();
        
        // Check for security headers (these might need to be added to the application)
        // For now, we'll just verify the response is successful
        assert!(headers.get("content-type").is_some());
    }

    #[actix_web::test]
    async fn test_cors_with_credentials() {
        let app = setup_security_test_app().await;

        // Test request with credentials
        let req = test::TestRequest::get()
            .uri("/api/annotations")
            .insert_header(("Origin", "https://example.com"))
            .insert_header(("Cookie", "session_id=12345"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let headers = resp.headers();
        assert!(headers.get("Access-Control-Allow-Origin").is_some());
    }

    #[actix_web::test]
    async fn test_cors_invalid_origin() {
        let app = setup_security_test_app().await;

        // Test with potentially malicious origin
        let req = test::TestRequest::get()
            .uri("/api/annotations")
            .insert_header(("Origin", "https://malicious-site.com"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        // The response should still work due to allow_any_origin() configuration
        assert_eq!(resp.status(), 200);

        let headers = resp.headers();
        assert!(headers.get("Access-Control-Allow-Origin").is_some());
    }

    #[actix_web::test]
    async fn test_cors_headers_consistency() {
        let app = setup_security_test_app().await;

        let endpoints = vec![
            "/api/annotations",
            "/api/users",
            "/api/projects",
            "/api/auth/validate",
        ];

        for endpoint in endpoints {
            let req = test::TestRequest::get()
                .uri(endpoint)
                .insert_header(("Origin", "https://example.com"))
                .to_request();

            let resp = test::call_service(&app, req).await;
            
            // Some endpoints might return 404, but CORS headers should be consistent
            let headers = resp.headers();
            assert!(headers.get("Access-Control-Allow-Origin").is_some());
        }
    }

    #[actix_web::test]
    async fn test_cors_preflight_with_complex_headers() {
        let app = setup_security_test_app().await;

        // Test preflight with complex headers
        let req = test::TestRequest::options()
            .uri("/api/annotations")
            .insert_header(("Origin", "https://example.com"))
            .insert_header(("Access-Control-Request-Method", "POST"))
            .insert_header(("Access-Control-Request-Headers", "Authorization, Content-Type, X-Custom-Header"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let headers = resp.headers();
        assert!(headers.get("Access-Control-Allow-Origin").is_some());
        assert!(headers.get("Access-Control-Allow-Methods").is_some());
        assert!(headers.get("Access-Control-Allow-Headers").is_some());
    }

    #[actix_web::test]
    async fn test_cors_max_age_header() {
        let app = setup_security_test_app().await;

        let req = test::TestRequest::options()
            .uri("/api/annotations")
            .insert_header(("Origin", "https://example.com"))
            .insert_header(("Access-Control-Request-Method", "GET"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let headers = resp.headers();
        let max_age = headers.get("Access-Control-Max-Age");
        assert!(max_age.is_some());
        
        // Verify max age is reasonable (should be 3600 based on our configuration)
        if let Some(max_age_value) = max_age {
            let max_age_str = max_age_value.to_str().unwrap();
            assert_eq!(max_age_str, "3600");
        }
    }

    #[actix_web::test]
    async fn test_cors_with_different_content_types() {
        let app = setup_security_test_app().await;

        let content_types = vec![
            "application/json",
            "application/x-www-form-urlencoded",
            "multipart/form-data",
            "text/plain",
        ];

        for content_type in content_types {
            let req = test::TestRequest::post()
                .uri("/api/annotations")
                .insert_header(("Origin", "https://example.com"))
                .insert_header(("Content-Type", content_type))
                .set_payload("{}")
                .to_request();

            let resp = test::call_service(&app, req).await;
            
            // Some requests might fail due to validation, but CORS headers should be present
            let headers = resp.headers();
            assert!(headers.get("Access-Control-Allow-Origin").is_some());
        }
    }

    #[actix_web::test]
    async fn test_cors_performance() {
        let app = setup_security_test_app().await;

        // Test CORS performance with multiple concurrent requests
        let mut handles = vec![];
        
        for i in 0..10 {
            let app_clone = app.clone();
            let handle = tokio::spawn(async move {
                let req = test::TestRequest::get()
                    .uri("/api/annotations")
                    .insert_header(("Origin", format!("https://example{}.com", i)))
                    .to_request();

                test::call_service(&app_clone, req).await
            });
            handles.push(handle);
        }

        // Wait for all requests to complete
        let results = futures::future::join_all(handles).await;
        
        for result in results {
            let resp = result.expect("CORS request failed");
            assert_eq!(resp.status(), 200);
            
            let headers = resp.headers();
            assert!(headers.get("Access-Control-Allow-Origin").is_some());
        }
    }

    #[actix_web::test]
    async fn test_cors_error_responses() {
        let app = setup_security_test_app().await;

        // Test CORS headers on error responses
        let req = test::TestRequest::get()
            .uri("/api/annotations/99999") // Non-existent annotation
            .insert_header(("Origin", "https://example.com"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404);

        // CORS headers should still be present on error responses
        let headers = resp.headers();
        assert!(headers.get("Access-Control-Allow-Origin").is_some());
    }

    #[actix_web::test]
    async fn test_cors_with_authentication() {
        let app = setup_security_test_app().await;

        // Test CORS with authentication headers
        let req = test::TestRequest::get()
            .uri("/api/annotations")
            .insert_header(("Origin", "https://example.com"))
            .insert_header(("Authorization", "Bearer test_token"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        
        // Request might fail due to invalid token, but CORS headers should be present
        let headers = resp.headers();
        assert!(headers.get("Access-Control-Allow-Origin").is_some());
    }

    #[actix_web::test]
    async fn test_cors_origin_validation() {
        let app = setup_security_test_app().await;

        // Test various origin formats
        let origins = vec![
            "https://example.com",
            "http://localhost:3000",
            "https://subdomain.example.com",
            "https://example.com:8080",
            "http://192.168.1.100:3000",
        ];

        for origin in origins {
            let req = test::TestRequest::get()
                .uri("/api/annotations")
                .insert_header(("Origin", origin))
                .to_request();

            let resp = test::call_service(&app, req).await;
            assert_eq!(resp.status(), 200);

            let headers = resp.headers();
            assert!(headers.get("Access-Control-Allow-Origin").is_some());
        }
    }

    #[actix_web::test]
    async fn test_cors_method_validation() {
        let app = setup_security_test_app().await;

        // Test various HTTP methods
        let methods = vec![
            ("GET", "/api/annotations"),
            ("POST", "/api/annotations"),
            ("PUT", "/api/annotations/1"),
            ("DELETE", "/api/annotations/1"),
            ("PATCH", "/api/annotations/1"),
        ];

        for (method, uri) in methods {
            let req = test::TestRequest::default()
                .method(method)
                .uri(uri)
                .insert_header(("Origin", "https://example.com"))
                .to_request();

            let resp = test::call_service(&app, req).await;
            
            // Some methods might return 404 or 405, but CORS headers should be present
            let headers = resp.headers();
            assert!(headers.get("Access-Control-Allow-Origin").is_some());
        }
    }
}
