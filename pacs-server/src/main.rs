use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;

mod application;
mod domain;
mod infrastructure;
mod presentation;

use application::use_cases::{
    AuthUseCase, UserUseCase, ProjectUseCase, PermissionUseCase, AccessControlUseCase,
};
use domain::services::{
    AuthServiceImpl, UserServiceImpl, ProjectServiceImpl, PermissionServiceImpl,
    AccessControlServiceImpl,
};
use infrastructure::repositories::{
    UserRepositoryImpl, ProjectRepositoryImpl, RoleRepositoryImpl, PermissionRepositoryImpl,
    AccessLogRepositoryImpl,
};
use infrastructure::auth::JwtService;
use infrastructure::config::JwtConfig;
use presentation::controllers::{
    auth_controller, user_controller, project_controller, permission_controller,
    access_control_controller,
};

async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "pacs-extension-server"
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://admin:admin123@localhost:5432/pacs_db".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    // Initialize repositories
    let user_repo = UserRepositoryImpl::new(pool.clone());
    let project_repo = ProjectRepositoryImpl::new(pool.clone());
    let role_repo = RoleRepositoryImpl::new(pool.clone());
    let permission_repo = PermissionRepositoryImpl::new(pool.clone());
    let access_log_repo = AccessLogRepositoryImpl::new(pool.clone());

    // Initialize JWT service
    let jwt_config = JwtConfig {
        secret: std::env::var("JWT_SECRET").unwrap_or_else(|_| "default_secret_key".to_string()),
        expiration_hours: 24,
    };
    let jwt_service = JwtService::new(&jwt_config);

    // Initialize services
    let auth_service = AuthServiceImpl::new(user_repo.clone(), jwt_service);
    let user_service = UserServiceImpl::new(user_repo.clone(), project_repo.clone());
    let project_service = ProjectServiceImpl::new(project_repo.clone(), user_repo.clone(), role_repo.clone());
    let permission_service = PermissionServiceImpl::new(permission_repo.clone(), role_repo.clone());
    let access_control_service = AccessControlServiceImpl::new(
        access_log_repo,
        user_repo.clone(),
        project_repo,
        role_repo,
        permission_repo,
    );

    // Initialize use cases
    let auth_use_case = Arc::new(AuthUseCase::new(auth_service));
    let user_use_case = Arc::new(UserUseCase::new(user_service));
    let project_use_case = Arc::new(ProjectUseCase::new(project_service));
    let permission_use_case = Arc::new(PermissionUseCase::new(permission_service));
    let access_control_use_case = Arc::new(AccessControlUseCase::new(access_control_service));

    println!("Starting PACS Extension Server on http://0.0.0.0:8080");

    HttpServer::new(move || {
        App::new()
            .route("/health", web::get().to(health_check))
            .configure(|cfg| auth_controller::configure_routes(cfg, auth_use_case.clone()))
            .configure(|cfg| user_controller::configure_routes(cfg, user_use_case.clone()))
            .configure(|cfg| project_controller::configure_routes(cfg, project_use_case.clone()))
            .configure(|cfg| permission_controller::configure_routes(cfg, permission_use_case.clone()))
            .configure(|cfg| access_control_controller::configure_routes(cfg, access_control_use_case.clone()))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
