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
    AnnotationUseCase,
};
use domain::services::{
    AuthServiceImpl, UserServiceImpl, ProjectServiceImpl, PermissionServiceImpl,
    AccessControlServiceImpl, AnnotationServiceImpl,
};
use infrastructure::repositories::{
    UserRepositoryImpl, ProjectRepositoryImpl, RoleRepositoryImpl, PermissionRepositoryImpl,
    AccessLogRepositoryImpl, AnnotationRepositoryImpl,
};
use infrastructure::auth::JwtService;
use infrastructure::config::JwtConfig;
use infrastructure::middleware::CacheHeaders;
use presentation::controllers::{
    auth_controller, user_controller, project_controller, permission_controller,
    access_control_controller, annotation_controller,
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

    // Database connection
    print!("📦 Connecting to PostgreSQL... ");
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://admin:admin123@localhost:5432/pacs_db".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
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
    let user_repo = UserRepositoryImpl::new(pool.clone());
    let project_repo = ProjectRepositoryImpl::new(pool.clone());
    let role_repo = RoleRepositoryImpl::new(pool.clone());
    let permission_repo = PermissionRepositoryImpl::new(pool.clone());
    let access_log_repo = AccessLogRepositoryImpl::new(pool.clone());
    let annotation_repo = AnnotationRepositoryImpl::new(pool.clone());
    println!("✅ Done");

    // Initialize JWT service
    print!("🔐 Initializing JWT service... ");
    let jwt_config = JwtConfig {
        secret: std::env::var("JWT_SECRET").unwrap_or_else(|_| "default_secret_key".to_string()),
        expiration_hours: 24,
    };
    let jwt_service = JwtService::new(&jwt_config);
    println!("✅ Done (TTL: {}h)", jwt_config.expiration_hours);

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
    println!("✅ Done");

    // Initialize use cases
    print!("📋 Initializing use cases... ");
    let auth_use_case = Arc::new(AuthUseCase::new(auth_service));
    let user_use_case = Arc::new(UserUseCase::new(user_service));
    let project_use_case = Arc::new(ProjectUseCase::new(project_service));
    let permission_use_case = Arc::new(PermissionUseCase::new(permission_service));
    let access_control_use_case = Arc::new(AccessControlUseCase::new(access_control_service));
    let annotation_use_case = Arc::new(AnnotationUseCase::new(annotation_service));
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
            )
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
