//! # Health Check Server
//! 
//! 최소한의 헬스체크 기능을 제공하는 뼈대 서버입니다.
//! Clean Architecture 패턴을 따르며 확장 가능한 구조를 유지합니다.
//! 
//! ## 주요 기능
//! - 서버 상태 확인 (Health Check)
//! - CORS 지원
//! - 구조화된 로깅
//! - 환경별 설정 관리
//! - OpenAPI 문서화 (선택사항)

use actix_web::{web, App, HttpServer, middleware::Logger, Result, HttpResponse, Responder};
use std::io;

// 모듈 선언
mod domain;
mod infrastructure;
mod presentation;

// 설정 및 미들웨어
use infrastructure::config::Settings;
use infrastructure::middleware::configure_cors;

// 컨트롤러
use presentation::controllers::health_controller;

/// 서버 상태 확인을 위한 기본 헬스체크 엔드포인트
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
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "health-check-server",
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        "version": env!("CARGO_PKG_VERSION")
    }))
}

/// PACS Extension Server의 메인 함수
/// 
/// 이 함수는 서버의 전체 생명주기를 관리합니다:
/// 1. 환경 변수 로드
/// 2. 설정 로드
/// 3. 로깅 초기화
/// 4. HTTP 서버 시작
/// 5. Graceful shutdown 처리
/// 
/// # 반환값
/// - `Ok(())`: 서버가 정상적으로 종료됨
/// - `Err(io::Error)`: 서버 시작 또는 실행 중 오류 발생
/// 
/// # 환경 변수
/// - `RUN_MODE`: 실행 모드 (development, production)
/// - `HOST`: 서버 호스트 (기본값: 0.0.0.0)
/// - `PORT`: 서버 포트 (기본값: 8080)
/// - `LOG_LEVEL`: 로그 레벨 (기본값: info)
/// - `CORS_ENABLED`: CORS 활성화 여부 (기본값: true)
#[actix_rt::main]
async fn main() -> io::Result<()> {
    // .env 파일에서 환경 변수 로드
    dotenvy::dotenv().ok();

    // 서버 초기화 시작 메시지 출력
    println!("\n{}", "=".repeat(80));
    println!("🚀 Health Check Server - Initialization");
    println!("{}\n", "=".repeat(80));

    // 설정 로드
    print!("⚙️  Loading configuration... ");
    let settings = Settings::new()
        .or_else(|_| {
            println!("⚠️  Config files not found, using environment variable defaults");
            Settings::with_env_defaults()
        })
        .expect("Failed to load configuration");
    println!("✅ Done");

    // 로깅 초기화 (간단한 로깅)
    print!("📝 Initializing logging... ");
    // 간단한 로깅 - 환경 변수로 로그 레벨 제어
    std::env::set_var("RUST_LOG", &settings.logging.level);
    println!("✅ Done (Level: {})", settings.logging.level);

    // CORS 설정
    print!("🌐 Configuring CORS... ");
    let cors_enabled = settings.cors.enabled;
    println!("✅ {} (Origins: {:?})", 
        if cors_enabled { "Enabled" } else { "Disabled" }, 
        settings.cors.allowed_origins
    );

    println!("\n{}", "=".repeat(80));
    println!("✨ Server Ready!");
    println!("{}", "=".repeat(80));
    println!("🌐 Server URL:    http://{}:{}", settings.server.host, settings.server.port);
    println!("❤️  Health Check:  http://{}:{}/health", settings.server.host, settings.server.port);
    println!("🔌 API Endpoints: http://{}:{}/api/", settings.server.host, settings.server.port);
    println!("{}\n", "=".repeat(80));

    // Graceful shutdown을 위한 signal handler 설정
    let shutdown_signal = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
        println!("\n🛑 Received shutdown signal, starting graceful shutdown...");
    };
    
    HttpServer::new(move || {
        App::new()
            // 로깅 미들웨어
            .wrap(Logger::default())
            // CORS 미들웨어
            .wrap(configure_cors(&settings.cors))
            // 기본 헬스체크 엔드포인트
            .route("/health", web::get().to(health_check))
            // API 라우트
            .service(
                web::scope("/api")
                    .configure(health_controller::configure_routes)
            )
    })
    .bind((settings.server.host.as_str(), settings.server.port))?
    .workers(settings.server.workers)
    .shutdown_timeout(30) // 30초 graceful shutdown timeout
    .run()
    .await?;
    
    // Graceful shutdown 완료
    println!("✅ Server shutdown completed");
    Ok(())
}
