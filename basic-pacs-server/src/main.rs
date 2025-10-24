//! # Basic PACS Server
//! 
//! Health check만 포함하는 기초 백엔드 서버입니다.
//! Clean Architecture 패턴의 기본 구조를 보여줍니다.

use actix_web::{web, App, HttpResponse, HttpServer, Responder, middleware::Logger};
use serde_json::json;
use std::collections::HashMap;
use std::io;

// 모듈 선언
mod domain;
mod application;
mod infrastructure;
mod presentation;

// 설정 및 미들웨어
use infrastructure::config::Settings;
use infrastructure::middleware::configure_cors;

// 컨트롤러
use presentation::controllers::health_controller;
use application::use_cases::HealthCheckUseCase;
use domain::services::HealthCheckServiceImpl;
use std::sync::Arc;

/// 서버 상태 확인을 위한 헬스체크 엔드포인트
/// 
/// # 반환값
/// - `200 OK`: 서버가 정상적으로 동작 중
/// - JSON 형태로 서버 상태 정보 반환
/// 
/// # 사용 예시
/// ```bash
/// curl http://localhost:8080/health
/// ```
async fn health_check() -> impl Responder {
    let mut status = HashMap::new();
    status.insert("status", "healthy");
    status.insert("service", "basic-pacs-server");
    status.insert("version", env!("CARGO_PKG_VERSION"));
    status.insert("timestamp", &chrono::Utc::now().to_rfc3339());
    
    HttpResponse::Ok().json(status)
}

/// 서버 정보를 반환하는 엔드포인트
async fn server_info() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "name": "Basic PACS Server",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "Health check만 포함하는 기초 백엔드 서버",
        "architecture": "Clean Architecture",
        "framework": "Actix Web",
        "language": "Rust",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// 메인 함수
/// 
/// 이 함수는 서버의 전체 생명주기를 관리합니다:
/// 1. 환경 변수 로드
/// 2. 설정 로드
/// 3. 로깅 초기화
/// 4. HTTP 서버 시작
/// 5. Graceful shutdown 처리
#[actix_web::main]
async fn main() -> io::Result<()> {
    // 환경 변수 로드
    dotenvy::dotenv().ok();
    
    // 로깅 초기화
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    
    // 설정 로드
    let settings = Settings::new()
        .or_else(|_| {
            tracing::warn!("Config files not found, using environment variable defaults");
            Settings::with_env_defaults()
        })
        .expect("Failed to load configuration");
    
    println!("\n{}", "=".repeat(80));
    println!("🚀 Basic PACS Server Starting...");
    println!("{}", "=".repeat(80));
    println!("🌐 Server URL:    http://{}:{}", settings.server.host, settings.server.port);
    println!("❤️  Health Check:  http://{}:{}/health", settings.server.host, settings.server.port);
    println!("ℹ️  Server Info:   http://{}:{}/info", settings.server.host, settings.server.port);
    println!("🔌 API Endpoints: http://{}:{}/api/", settings.server.host, settings.server.port);
    println!("{}\n", "=".repeat(80));
    
    // Graceful shutdown을 위한 signal handler
    let shutdown_signal = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
        println!("\n🛑 Received shutdown signal, starting graceful shutdown...");
    };
    
    // 서비스 초기화
    let health_check_service = Box::new(HealthCheckServiceImpl::new());
    let health_check_use_case = HealthCheckUseCase::new(health_check_service);
    let health_controller = Arc::new(health_controller::HealthController::new(health_check_use_case));

    // HTTP 서버 시작
    HttpServer::new(move || {
        App::new()
            // 로깅 미들웨어
            .wrap(Logger::default())
            // CORS 미들웨어
            .wrap(configure_cors(&settings.cors))
            // 컨트롤러 데이터 주입
            .app_data(web::Data::from(health_controller.clone()))
            // Health check 엔드포인트
            .route("/health", web::get().to(health_check))
            // 서버 정보 엔드포인트
            .route("/info", web::get().to(server_info))
            // 기본 라우트
            .route("/", web::get().to(|| async {
                HttpResponse::Ok().json(json!({
                    "message": "Welcome to Basic PACS Server",
                    "endpoints": {
                        "health": "/health",
                        "info": "/info",
                        "api": "/api/"
                    },
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }))
            }))
            // API 라우트
            .service(
                web::scope("/api")
                    .configure(health_controller::configure_routes)
            )
    })
    .bind((settings.server.host.as_str(), settings.server.port))?
    .workers(settings.server.workers)
    .shutdown_timeout(30)
    .run()
    .await?;
    
    println!("✅ Server shutdown completed");
    Ok(())
}
