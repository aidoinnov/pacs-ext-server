#[cfg(test)]
mod api_documentation_tests {
    use actix_web::{test, web, App, middleware::Logger};
    use utoipa::OpenApi;
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
    use pacs_server::ApiDoc;
    use sqlx::postgres::PgPoolOptions;
    use std::sync::Arc;
    use serde_json::Value;

    async fn setup_documentation_test_app() -> impl actix_web::dev::Service<
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
            secret: "test_secret_key_for_documentation_testing".to_string(),
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

        // Generate OpenAPI documentation
        let openapi = ApiDoc::openapi();

        test::init_service(
            App::new()
                .wrap(Logger::default())
                .app_data(web::Data::new(annotation_use_case.clone()))
                .app_data(web::Data::new(mask_group_use_case.clone()))
                .app_data(web::Data::new(mask_use_case.clone()))
                .app_data(web::Data::new(user_use_case.clone()))
                .app_data(web::Data::new(project_use_case.clone()))
                .app_data(web::Data::new(auth_use_case.clone()))
                .service(
                    utoipa_swagger_ui::SwaggerUi::new("/swagger-ui/{_:.*}")
                        .url("/api-docs/openapi.json", openapi.clone()),
                )
                .service(
                    web::resource("/api-docs/openapi.json")
                        .route(web::get().to(move || async move { openapi })),
                )
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
    async fn test_openapi_json_endpoint() {
        let app = setup_documentation_test_app().await;

        let req = test::TestRequest::get()
            .uri("/api-docs/openapi.json")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        // Verify content type
        let content_type = resp.headers().get("content-type").unwrap();
        assert!(content_type.to_str().unwrap().contains("application/json"));

        // Parse and validate OpenAPI JSON
        let body: Value = test::read_body_json(resp).await;
        
        // Check required OpenAPI fields
        assert!(body["openapi"].is_string());
        assert!(body["info"].is_object());
        assert!(body["paths"].is_object());
        assert!(body["components"].is_object());

        // Check API info
        let info = &body["info"];
        assert!(info["title"].is_string());
        assert!(info["version"].is_string());
        assert!(info["description"].is_string());
    }

    #[actix_web::test]
    async fn test_swagger_ui_endpoint() {
        let app = setup_documentation_test_app().await;

        let req = test::TestRequest::get()
            .uri("/swagger-ui/")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        // Verify content type
        let content_type = resp.headers().get("content-type").unwrap();
        assert!(content_type.to_str().unwrap().contains("text/html"));
    }

    #[actix_web::test]
    async fn test_openapi_paths_validation() {
        let app = setup_documentation_test_app().await;

        let req = test::TestRequest::get()
            .uri("/api-docs/openapi.json")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body: Value = test::read_body_json(resp).await;
        let paths = &body["paths"];

        // Check that all expected API endpoints are documented
        let expected_paths = vec![
            "/api/annotations",
            "/api/annotations/{id}",
            "/api/annotations/{annotation_id}/mask-groups",
            "/api/annotations/{annotation_id}/mask-groups/{id}",
            "/api/annotations/{annotation_id}/mask-groups/{mask_group_id}/masks",
            "/api/annotations/{annotation_id}/mask-groups/{mask_group_id}/masks/{id}",
            "/api/users",
            "/api/users/{id}",
            "/api/projects",
            "/api/projects/{id}",
            "/api/auth/validate",
            "/api/auth/refresh",
        ];

        for expected_path in expected_paths {
            assert!(paths.get(expected_path).is_some(), "Path {} not found in OpenAPI spec", expected_path);
        }
    }

    #[actix_web::test]
    async fn test_openapi_components_validation() {
        let app = setup_documentation_test_app().await;

        let req = test::TestRequest::get()
            .uri("/api-docs/openapi.json")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body: Value = test::read_body_json(resp).await;
        let components = &body["components"];

        // Check that schemas are defined
        assert!(components["schemas"].is_object());
        let schemas = &components["schemas"];

        // Check for key DTOs
        let expected_schemas = vec![
            "CreateAnnotationRequest",
            "UpdateAnnotationRequest",
            "AnnotationResponse",
            "CreateMaskGroupRequest",
            "UpdateMaskGroupRequest",
            "MaskGroupResponse",
            "CreateMaskRequest",
            "UpdateMaskRequest",
            "MaskResponse",
            "CreateUserRequest",
            "UpdateUserRequest",
            "UserResponse",
            "CreateProjectRequest",
            "UpdateProjectRequest",
            "ProjectResponse",
            "LoginRequest",
            "RegisterRequest",
            "AuthResponse",
        ];

        for expected_schema in expected_schemas {
            assert!(schemas.get(expected_schema).is_some(), "Schema {} not found in OpenAPI spec", expected_schema);
        }
    }

    #[actix_web::test]
    async fn test_openapi_http_methods_validation() {
        let app = setup_documentation_test_app().await;

        let req = test::TestRequest::get()
            .uri("/api-docs/openapi.json")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body: Value = test::read_body_json(resp).await;
        let paths = &body["paths"];

        // Check that each path has the expected HTTP methods
        let path_methods = vec![
            ("/api/annotations", vec!["get", "post"]),
            ("/api/annotations/{id}", vec!["get", "put", "delete"]),
            ("/api/annotations/{annotation_id}/mask-groups", vec!["get", "post"]),
            ("/api/annotations/{annotation_id}/mask-groups/{id}", vec!["get", "put", "delete"]),
            ("/api/annotations/{annotation_id}/mask-groups/{mask_group_id}/masks", vec!["get", "post"]),
            ("/api/annotations/{annotation_id}/mask-groups/{mask_group_id}/masks/{id}", vec!["get", "put", "delete"]),
            ("/api/users", vec!["get", "post"]),
            ("/api/users/{id}", vec!["get", "put", "delete"]),
            ("/api/projects", vec!["get", "post"]),
            ("/api/projects/{id}", vec!["get", "put", "delete"]),
            ("/api/auth/validate", vec!["get"]),
            ("/api/auth/refresh", vec!["post"]),
        ];

        for (path, expected_methods) in path_methods {
            if let Some(path_obj) = paths.get(path) {
                for method in expected_methods {
                    assert!(path_obj.get(method).is_some(), "Method {} not found for path {}", method, path);
                }
            }
        }
    }

    #[actix_web::test]
    async fn test_openapi_request_response_schemas() {
        let app = setup_documentation_test_app().await;

        let req = test::TestRequest::get()
            .uri("/api-docs/openapi.json")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body: Value = test::read_body_json(resp).await;
        let paths = &body["paths"];

        // Check that request and response schemas are properly defined
        let annotation_post = &paths["/api/annotations"]["post"];
        assert!(annotation_post["requestBody"].is_object());
        assert!(annotation_post["responses"].is_object());

        let annotation_get = &paths["/api/annotations"]["get"];
        assert!(annotation_get["responses"].is_object());
        assert!(annotation_get["responses"]["200"].is_object());
    }

    #[actix_web::test]
    async fn test_openapi_parameters_validation() {
        let app = setup_documentation_test_app().await;

        let req = test::TestRequest::get()
            .uri("/api-docs/openapi.json")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body: Value = test::read_body_json(resp).await;
        let paths = &body["paths"];

        // Check that path parameters are properly defined
        let annotation_by_id = &paths["/api/annotations/{id}"];
        assert!(annotation_by_id["parameters"].is_array());
        
        let parameters = annotation_by_id["parameters"].as_array().unwrap();
        assert!(!parameters.is_empty());
        
        // Check that the id parameter is defined
        let id_param = parameters.iter().find(|p| p["name"] == "id");
        assert!(id_param.is_some());
    }

    #[actix_web::test]
    async fn test_openapi_security_schemes() {
        let app = setup_documentation_test_app().await;

        let req = test::TestRequest::get()
            .uri("/api-docs/openapi.json")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body: Value = test::read_body_json(resp).await;
        let components = &body["components"];

        // Check that security schemes are defined
        if let Some(security_schemes) = components.get("securitySchemes") {
            assert!(security_schemes.is_object());
            
            // Check for JWT security scheme
            if let Some(jwt_scheme) = security_schemes.get("BearerAuth") {
                assert_eq!(jwt_scheme["type"], "http");
                assert_eq!(jwt_scheme["scheme"], "bearer");
                assert_eq!(jwt_scheme["bearerFormat"], "JWT");
            }
        }
    }

    #[actix_web::test]
    async fn test_openapi_tags_validation() {
        let app = setup_documentation_test_app().await;

        let req = test::TestRequest::get()
            .uri("/api-docs/openapi.json")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body: Value = test::read_body_json(resp).await;

        // Check that tags are defined
        if let Some(tags) = body.get("tags") {
            assert!(tags.is_array());
            
            let expected_tags = vec![
                "Annotations",
                "Mask Groups",
                "Masks",
                "Users",
                "Projects",
                "Authentication",
            ];

            let tag_names: Vec<String> = tags.as_array().unwrap()
                .iter()
                .map(|tag| tag["name"].as_str().unwrap().to_string())
                .collect();

            for expected_tag in expected_tags {
                assert!(tag_names.contains(&expected_tag.to_string()), "Tag {} not found", expected_tag);
            }
        }
    }

    #[actix_web::test]
    async fn test_openapi_examples_validation() {
        let app = setup_documentation_test_app().await;

        let req = test::TestRequest::get()
            .uri("/api-docs/openapi.json")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body: Value = test::read_body_json(resp).await;
        let components = &body["components"];
        let schemas = &components["schemas"];

        // Check that examples are provided for key schemas
        let schemas_with_examples = vec![
            "CreateAnnotationRequest",
            "CreateMaskGroupRequest",
            "CreateMaskRequest",
            "CreateUserRequest",
            "CreateProjectRequest",
        ];

        for schema_name in schemas_with_examples {
            if let Some(schema) = schemas.get(schema_name) {
                // Check if the schema has properties with examples
                if let Some(properties) = schema.get("properties") {
                    let has_examples = properties.as_object().unwrap()
                        .values()
                        .any(|prop| prop.get("example").is_some());
                    
                    assert!(has_examples, "Schema {} should have examples", schema_name);
                }
            }
        }
    }

    #[actix_web::test]
    async fn test_openapi_error_responses() {
        let app = setup_documentation_test_app().await;

        let req = test::TestRequest::get()
            .uri("/api-docs/openapi.json")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body: Value = test::read_body_json(resp).await;
        let paths = &body["paths"];

        // Check that error responses are documented
        let annotation_get = &paths["/api/annotations"]["get"];
        let responses = &annotation_get["responses"];

        // Check for common error status codes
        let error_codes = vec!["400", "401", "403", "404", "500"];
        for error_code in error_codes {
            if let Some(error_response) = responses.get(error_code) {
                assert!(error_response["description"].is_string());
            }
        }
    }

    #[actix_web::test]
    async fn test_swagger_ui_static_files() {
        let app = setup_documentation_test_app().await;

        // Test various Swagger UI static files
        let static_files = vec![
            "/swagger-ui/index.html",
            "/swagger-ui/swagger-ui-bundle.js",
            "/swagger-ui/swagger-ui-standalone-preset.js",
            "/swagger-ui/swagger-ui.css",
        ];

        for file_path in static_files {
            let req = test::TestRequest::get()
                .uri(file_path)
                .to_request();

            let resp = test::call_service(&app, req).await;
            // Some files might return 404, but the main index should work
            if file_path == "/swagger-ui/index.html" {
                assert_eq!(resp.status(), 200);
            }
        }
    }

    #[actix_web::test]
    async fn test_openapi_schema_validation() {
        let app = setup_documentation_test_app().await;

        let req = test::TestRequest::get()
            .uri("/api-docs/openapi.json")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body: Value = test::read_body_json(resp).await;

        // Validate OpenAPI version
        let openapi_version = body["openapi"].as_str().unwrap();
        assert!(openapi_version.starts_with("3."), "OpenAPI version should be 3.x");

        // Validate info object
        let info = &body["info"];
        assert!(info["title"].is_string());
        assert!(info["version"].is_string());
        assert!(info["description"].is_string());

        // Validate paths object
        let paths = &body["paths"];
        assert!(paths.is_object());
        assert!(!paths.as_object().unwrap().is_empty());

        // Validate components object
        let components = &body["components"];
        assert!(components.is_object());
        assert!(components["schemas"].is_object());
    }

    #[actix_web::test]
    async fn test_openapi_performance() {
        let app = setup_documentation_test_app().await;

        // Test OpenAPI endpoint performance
        let start = std::time::Instant::now();
        
        let req = test::TestRequest::get()
            .uri("/api-docs/openapi.json")
            .to_request();

        let resp = test::call_service(&app, req).await;
        let duration = start.elapsed();

        assert_eq!(resp.status(), 200);
        assert!(duration < std::time::Duration::from_millis(100), "OpenAPI endpoint too slow: {:?}", duration);
    }

    #[actix_web::test]
    async fn test_openapi_content_negotiation() {
        let app = setup_documentation_test_app().await;

        // Test with different Accept headers
        let accept_headers = vec![
            "application/json",
            "application/vnd.oai.openapi+json",
            "application/vnd.oai.openapi",
            "*/*",
        ];

        for accept_header in accept_headers {
            let req = test::TestRequest::get()
                .uri("/api-docs/openapi.json")
                .insert_header(("Accept", accept_header))
                .to_request();

            let resp = test::call_service(&app, req).await;
            assert_eq!(resp.status(), 200);
        }
    }
}
