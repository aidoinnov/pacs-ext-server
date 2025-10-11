use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use sqlx::postgres::PgPoolOptions;
// use redis::Client as RedisClient;
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod application;
mod domain;
mod infrastructure;
mod presentation;

use application::use_cases::{
    AuthUseCase, UserUseCase, ProjectUseCase, PermissionUseCase, AccessControlUseCase,
    AnnotationUseCase, MaskGroupUseCase, MaskUseCase,
};
use domain::services::{
    AuthServiceImpl, UserServiceImpl, ProjectServiceImpl, PermissionServiceImpl,
    AccessControlServiceImpl, AnnotationServiceImpl, MaskGroupServiceImpl, MaskServiceImpl,
};
use infrastructure::repositories::{
    UserRepositoryImpl, ProjectRepositoryImpl, RoleRepositoryImpl, PermissionRepositoryImpl,
    AccessLogRepositoryImpl, AnnotationRepositoryImpl, MaskGroupRepositoryImpl, MaskRepositoryImpl,
};
use infrastructure::auth::JwtService;
use application::services::SignedUrlServiceImpl;
use infrastructure::config::{JwtConfig, Settings};
use infrastructure::middleware::{CacheHeaders, configure_cors};
use presentation::controllers::{
    auth_controller, user_controller, project_controller, permission_controller,
    access_control_controller, annotation_controller, mask_group_controller, mask_controller,
};
use presentation::openapi::ApiDoc;

async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "pacs-extension-server"
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();

    println!("\n{}", "=".repeat(80));
    println!("🚀 PACS Extension Server - Initialization");
    println!("{}\n", "=".repeat(80));

    // Load configuration
    print!("⚙️  Loading configuration... ");
    let settings = Settings::new().expect("Failed to load configuration");
    println!("✅ Done");

    // CORS configuration
    print!("🌐 Configuring CORS... ");
    let cors_enabled = settings.cors.enabled;
    println!("✅ {} (Origins: {:?})", 
        if cors_enabled { "Enabled" } else { "Disabled" }, 
        settings.cors.allowed_origins
    );

    // Database connection
    print!("📦 Connecting to PostgreSQL... ");
    let database_url = settings.database_url();

    let pool = PgPoolOptions::new()
        .max_connections(settings.database.max_connections)
        .min_connections(settings.database.min_connections)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    println!("✅ Connected");

    // Redis connection (commented out for now)
    // let redis_url = std::env::var("REDIS_URL")
    //     .unwrap_or_else(|_| "redis://:redis123@localhost:6379/0".to_string());
    // let redis_client = RedisClient::open(redis_url)
    //     .expect("Failed to create Redis client");
    // let mut redis_conn = redis_client.get_connection()
    //     .expect("Failed to connect to Redis");
    // redis::cmd("PING")
    //     .query::<String>(&mut redis_conn)
    //     .expect("Failed to ping Redis");
    // println!("Successfully connected to Redis");

    // Initialize repositories
    print!("🔧 Initializing repositories... ");
    let user_repo = Arc::new(UserRepositoryImpl::new(pool.clone()));
    let project_repo = Arc::new(ProjectRepositoryImpl::new(pool.clone()));
    let role_repo = Arc::new(RoleRepositoryImpl::new(pool.clone()));
    let permission_repo = Arc::new(PermissionRepositoryImpl::new(pool.clone()));
    let access_log_repo = Arc::new(AccessLogRepositoryImpl::new(pool.clone()));
    let annotation_repo = Arc::new(AnnotationRepositoryImpl::new(pool.clone()));
    let mask_group_repo = Arc::new(MaskGroupRepositoryImpl::new(pool.clone()));
    let mask_repo = Arc::new(MaskRepositoryImpl::new(pool.clone()));
    println!("✅ Done");

    // Initialize JWT service
    print!("🔐 Initializing JWT service... ");
    let jwt_service = JwtService::new(&settings.jwt);
    println!("✅ Done (TTL: {}h)", settings.jwt.expiration_hours);

    // Initialize services
    print!("⚙️  Initializing domain services... ");
    let auth_service = AuthServiceImpl::new(user_repo.clone(), jwt_service);
    let user_service = UserServiceImpl::new(user_repo.clone(), project_repo.clone());
    let project_service = ProjectServiceImpl::new(project_repo.clone(), user_repo.clone(), role_repo.clone());
    let permission_service = PermissionServiceImpl::new(permission_repo.clone(), role_repo.clone());
    let access_control_service = AccessControlServiceImpl::new(
        access_log_repo,
        user_repo.clone(),
        project_repo.clone(),
        role_repo,
        permission_repo,
    );
    let annotation_service = AnnotationServiceImpl::new(annotation_repo, user_repo.clone(), project_repo.clone());
    let mask_group_service = Arc::new(MaskGroupServiceImpl::new(
        mask_group_repo.clone(),
        annotation_repo.clone(),
        user_repo.clone(),
    ));
    let mask_service = Arc::new(MaskServiceImpl::new(
        mask_repo.clone(),
        mask_group_repo.clone(),
        user_repo.clone(),
    ));
    // TODO: Initialize ObjectStorageService with proper configuration
    // For now, we'll skip signed_url_service initialization
    println!("✅ Done");

    // Initialize use cases
    print!("📋 Initializing use cases... ");
    let auth_use_case = Arc::new(AuthUseCase::new(auth_service));
    let user_use_case = Arc::new(UserUseCase::new(user_service));
    let project_use_case = Arc::new(ProjectUseCase::new(project_service));
    let permission_use_case = Arc::new(PermissionUseCase::new(permission_service));
    let access_control_use_case = Arc::new(AccessControlUseCase::new(access_control_service));
    let annotation_use_case = Arc::new(AnnotationUseCase::new(annotation_service));
    // TODO: Initialize mask use cases after ObjectStorageService is configured
    // let mask_group_use_case = Arc::new(MaskGroupUseCase::new(mask_group_service, signed_url_service.clone()));
    // let mask_use_case = Arc::new(MaskUseCase::new(mask_service, mask_group_service.clone(), signed_url_service.clone()));
    println!("✅ Done");

    // Cache configuration
    print!("💾 Configuring cache... ");
    let cache_enabled = std::env::var("CACHE_ENABLED")
        .unwrap_or_else(|_| "true".to_string())
        .parse::<bool>()
        .unwrap_or(true);
    let cache_ttl = std::env::var("CACHE_TTL_SECONDS")
        .unwrap_or_else(|_| "300".to_string())
        .parse::<u64>()
        .unwrap_or(300);
    println!("✅ {} (TTL: {}s)", if cache_enabled { "Enabled" } else { "Disabled" }, cache_ttl);

    // OpenAPI 문서 생성
    print!("📚 Generating OpenAPI documentation... ");
    let mut openapi = ApiDoc::openapi();
    openapi = presentation::openapi_extensions::extend_openapi(openapi);
    println!("✅ Done");

    println!("\n{}", "=".repeat(80));
    println!("✨ Server Ready!");
    println!("{}", "=".repeat(80));
    println!("🌐 Server URL:    http://0.0.0.0:8080");
    println!("📖 Swagger UI:    http://0.0.0.0:8080/swagger-ui/");
    println!("❤️  Health Check:  http://0.0.0.0:8080/health");
    println!("🔌 API Endpoints: http://0.0.0.0:8080/api/");
    println!("{}\n", "=".repeat(80));

    HttpServer::new(move || {
        App::new()
            // CORS middleware
            .wrap(configure_cors(&settings.cors))
            // Cache headers middleware
            .wrap(CacheHeaders::new(cache_enabled, cache_ttl))
            // Swagger UI (commented out for now)
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", openapi.clone())
            )
            // Health check
            .route("/health", web::get().to(health_check))
            // API routes
            .service(
                web::scope("/api")
                    .configure(|cfg| auth_controller::configure_routes(cfg, auth_use_case.clone()))
                    .configure(|cfg| user_controller::configure_routes(cfg, user_use_case.clone()))
                    .configure(|cfg| project_controller::configure_routes(cfg, project_use_case.clone()))
                    .configure(|cfg| permission_controller::configure_routes(cfg, permission_use_case.clone()))
                    .configure(|cfg| access_control_controller::configure_routes(cfg, access_control_use_case.clone()))
                    .configure(|cfg| annotation_controller::configure_routes(cfg, annotation_use_case.clone()))
                    // TODO: Add mask routes after ObjectStorageService is configured
                    // .configure(|cfg| mask_group_controller::configure_routes(cfg, mask_group_use_case.clone()))
                    // .configure(|cfg| mask_controller::configure_routes(cfg, mask_use_case.clone()))
            )
    })
    .bind((settings.server.host.as_str(), settings.server.port))?
    .workers(settings.server.workers)
    .run()
    .await
}
