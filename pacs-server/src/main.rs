//! # PACS Extension Server
//!
//! PACS(의료영상저장전송시스템) 확장 서버의 메인 엔트리 포인트입니다.
//! 이 서버는 의료 영상 어노테이션, 마스크 관리, 사용자 인증 등의 기능을 제공합니다.
//!
//! ## 아키텍처
//! - **Clean Architecture** 패턴을 따르며, 도메인 중심의 설계를 채택합니다.
//! - **Repository Pattern**을 통해 데이터 접근을 추상화합니다.
//! - **Use Case Pattern**을 통해 비즈니스 로직을 캡슐화합니다.
//!
//! ## 주요 기능
//! - 사용자 인증 및 권한 관리
//! - 프로젝트 및 어노테이션 관리
//! - 마스크 그룹 및 개별 마스크 관리
//! - 객체 저장소 연동 (AWS S3, MinIO)
//! - RESTful API 제공
//! - OpenAPI 문서화

// 환경 변수 로딩
use dotenvy::dotenv;

// 웹 프레임워크 및 HTTP 관련 모듈
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
// PostgreSQL 데이터베이스 연결 풀 옵션
use sqlx::postgres::PgPoolOptions;
// Redis 클라이언트 (현재 비활성화)
// use redis::Client as RedisClient;
// 스레드 안전한 참조 카운팅 포인터
use std::sync::Arc;
// OpenAPI 문서 생성
use utoipa::OpenApi;
// Swagger UI 서비스
use utoipa_swagger_ui::SwaggerUi;

// 애플리케이션 레이어 모듈 (Use Case, Service 등)
mod application;
// 도메인 레이어 모듈 (Entity, Repository, Service 인터페이스 등)
mod domain;
// 인프라스트럭처 레이어 모듈 (데이터베이스, 외부 서비스 등)
mod infrastructure;
// 프레젠테이션 레이어 모듈 (Controller, DTO 등)
mod presentation;

// 애플리케이션 레이어 - Use Case 인터페이스들
use application::use_cases::{
    AccessControlUseCase, AnnotationUseCase, AuthUseCase, MaskGroupUseCase, MaskUseCase,
    PermissionUseCase, ProjectDataAccessUseCase, ProjectUseCase, ProjectUserMatrixUseCase,
    ProjectUserUseCase, RolePermissionMatrixUseCase, RoleCapabilityMatrixUseCase, UserRegistrationUseCase, UserUseCase,
    UserProjectMatrixUseCase,
};

// 도메인 레이어 - 서비스 구현체들
use domain::services::{
    AccessControlServiceImpl, AnnotationServiceImpl, AuthServiceImpl, MaskGroupServiceImpl,
    MaskServiceImpl, PermissionServiceImpl, ProjectServiceImpl, UserServiceImpl,
};

// 인프라스트럭처 레이어 - 리포지토리 구현체들
use infrastructure::external::KeycloakClient;
use infrastructure::repositories::{
    AccessLogRepositoryImpl, AnnotationRepositoryImpl, CapabilityRepositoryImpl, MaskGroupRepositoryImpl, MaskRepositoryImpl,
    PermissionRepositoryImpl, ProjectDataAccessRepositoryImpl, ProjectDataRepositoryImpl,
    ProjectRepositoryImpl, RoleRepositoryImpl, UserRepositoryImpl,
};
use infrastructure::services::{CapabilityServiceImpl, ProjectDataServiceImpl, UserRegistrationServiceImpl};

// JWT 인증 서비스
use infrastructure::auth::JwtService;
// 서명된 URL 및 객체 저장소 서비스
use application::services::{ObjectStorageServiceFactory, SignedUrlServiceImpl};
// 설정 관련 구조체들
use infrastructure::config::{JwtConfig, Settings};
// 미들웨어 (캐시 헤더, CORS)
use infrastructure::middleware::{configure_cors, CacheHeaders};
// 프레젠테이션 레이어 - 컨트롤러들
use presentation::controllers::{
    access_control_controller, annotation_controller, auth_controller, mask_controller,
    mask_group_controller, project_controller, role_controller,
    project_data_access_controller, project_user_controller, project_user_matrix_controller,
    user_project_matrix_controller,
    role_permission_matrix_controller, user_controller, user_registration_controller,
};
// OpenAPI 문서 생성
use presentation::openapi::ApiDoc;

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
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "pacs-extension-server"
    }))
}

/// PACS Extension Server의 메인 함수
///
/// 이 함수는 서버의 전체 생명주기를 관리합니다:
/// 1. 환경 변수 로드
/// 2. 설정 로드
/// 3. 데이터베이스 연결 설정
/// 4. 서비스 및 리포지토리 초기화
/// 5. HTTP 서버 시작
/// 6. Graceful shutdown 처리
///
/// # 반환값
/// - `Ok(())`: 서버가 정상적으로 종료됨
/// - `Err(io::Error)`: 서버 시작 또는 실행 중 오류 발생
///
/// # 환경 변수
/// - `DATABASE_URL`: PostgreSQL 데이터베이스 연결 URL
/// - `JWT_SECRET`: JWT 토큰 서명을 위한 비밀키
/// - `S3_ACCESS_KEY`, `S3_SECRET_KEY`: AWS S3 접근 키
/// - `CACHE_ENABLED`: 캐시 활성화 여부 (기본값: true)
/// - `CACHE_TTL_SECONDS`: 캐시 TTL (기본값: 300초)
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 서버 초기화 시작 메시지 출력
    println!("\n{}", "=".repeat(80));
    println!("🚀 PACS Extension Server - Initialization");
    println!("{}\n", "=".repeat(80));

    // .env 파일에서 환경 변수 로드
    dotenvy::dotenv().ok();

    // 디버깅: 환경 변수 로딩 확인
    println!("🔍 환경 변수 로딩 확인:");
    println!(
        "   APP_OBJECT_STORAGE__ACCESS_KEY_ID: {}",
        std::env::var("APP_OBJECT_STORAGE__ACCESS_KEY_ID")
            .unwrap_or_else(|_| "NOT_FOUND".to_string())
    );
    println!(
        "   APP_OBJECT_STORAGE__SECRET_ACCESS_KEY: {}",
        std::env::var("APP_OBJECT_STORAGE__SECRET_ACCESS_KEY")
            .unwrap_or_else(|_| "NOT_FOUND".to_string())
    );

    // 애플리케이션 설정 로드
    print!("⚙️  Loading configuration... ");
    let settings = Settings::new()
        .or_else(|_| {
            println!("⚠️  Config files not found, using environment variable defaults");
            Settings::with_env_defaults()
        })
        .expect("Failed to load configuration");
    println!("✅ Done");

    // CORS(Cross-Origin Resource Sharing) 설정
    // 웹 브라우저에서 다른 도메인의 리소스에 접근할 수 있도록 허용하는 설정
    print!("🌐 Configuring CORS... ");
    let cors_enabled = settings.cors.enabled;
    println!(
        "✅ {} (Origins: {:?})",
        if cors_enabled { "Enabled" } else { "Disabled" },
        settings.cors.allowed_origins
    );

    // PostgreSQL 데이터베이스 연결 설정
    // 연결 풀을 사용하여 동시 연결 수를 제한하고 성능을 최적화
    print!("📦 Connecting to PostgreSQL... ");
    let database_url = settings.database_url();

    // 데이터베이스 연결 풀 생성
    // max_connections: 최대 동시 연결 수
    // min_connections: 최소 유지 연결 수
    let pool = PgPoolOptions::new()
        .max_connections(settings.database.max_connections)
        .min_connections(settings.database.min_connections)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    println!("✅ Connected");

    // Redis 연결 (현재 비활성화 상태)
    // 캐싱 및 세션 저장을 위한 Redis 연결 설정
    // 향후 캐싱 기능 구현 시 활성화 예정
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

    // 데이터 접근 계층(Repository) 초기화
    // 각 엔티티별로 데이터베이스 작업을 담당하는 리포지토리 생성
    print!("🔧 Initializing repositories... ");

    // 사용자 관련 데이터 접근을 위한 리포지토리
    let user_repo = UserRepositoryImpl::new(pool.clone());
    // 프로젝트 관련 데이터 접근을 위한 리포지토리
    let project_repo = ProjectRepositoryImpl::new(pool.clone());
    // 역할(Role) 관련 데이터 접근을 위한 리포지토리
    let role_repo = RoleRepositoryImpl::new(pool.clone());
    // 권한(Permission) 관련 데이터 접근을 위한 리포지토리
    let permission_repo = PermissionRepositoryImpl::new(pool.clone());
    // 접근 로그 관련 데이터 접근을 위한 리포지토리
    let access_log_repo = AccessLogRepositoryImpl::new(pool.clone());
    // 어노테이션 관련 데이터 접근을 위한 리포지토리
    let annotation_repo = AnnotationRepositoryImpl::new(pool.clone());
    // 마스크 그룹 관련 데이터 접근을 위한 리포지토리 (Arc로 래핑하여 공유 소유권)
    let mask_group_repo = Arc::new(MaskGroupRepositoryImpl::new(pool.clone()));
    // 마스크 관련 데이터 접근을 위한 리포지토리 (Arc로 래핑하여 공유 소유권)
    let mask_repo = Arc::new(MaskRepositoryImpl::new(pool.clone()));
    // 프로젝트 데이터 관련 데이터 접근을 위한 리포지토리
    let project_data_repo = Arc::new(ProjectDataRepositoryImpl::new(pool.clone()));
    // 프로젝트 데이터 접근 권한 관련 데이터 접근을 위한 리포지토리
    let project_data_access_repo = Arc::new(ProjectDataAccessRepositoryImpl::new(pool.clone()));
    println!("✅ Done");

    // JWT(JSON Web Token) 서비스 초기화
    // 사용자 인증을 위한 토큰 생성 및 검증 서비스
    print!("🔐 Initializing JWT service... ");
    let jwt_service = JwtService::new(&settings.jwt);
    println!("✅ Done (TTL: {}h)", settings.jwt.expiration_hours);

    // Keycloak 클라이언트 초기화
    print!("🔐 Initializing Keycloak client... ");
    let keycloak_client = Arc::new(KeycloakClient::new(settings.keycloak.clone()));
    println!("✅ Done (Realm: {})", settings.keycloak.realm);

    // 도메인 서비스 계층 초기화
    // 비즈니스 로직을 담당하는 서비스들을 생성
    print!("⚙️  Initializing domain services... ");

    // 인증 서비스: 로그인, 토큰 생성/검증 등
    let auth_service =
        AuthServiceImpl::new(user_repo.clone(), jwt_service, keycloak_client.clone());
    // 사용자 서비스: 사용자 CRUD, 프로젝트 멤버십 관리 등
    let user_service = UserServiceImpl::new(user_repo.clone(), project_repo.clone());
    // 프로젝트 서비스: 프로젝트 CRUD, 사용자 관리 등
    let project_service =
        ProjectServiceImpl::new(project_repo.clone(), user_repo.clone(), role_repo.clone());
    // 권한 서비스: 권한 CRUD, 역할-권한 매핑 등
    let permission_service: PermissionServiceImpl<PermissionRepositoryImpl, RoleRepositoryImpl> = 
        PermissionServiceImpl::new(permission_repo.clone(), role_repo.clone());
    // 접근 제어 서비스: 권한 검증, 접근 로그 기록 등
    let access_control_service = AccessControlServiceImpl::new(
        access_log_repo,
        user_repo.clone(),
        project_repo.clone(),
        role_repo,
        permission_repo,
    );
    // 어노테이션 서비스: 어노테이션 CRUD, 히스토리 관리 등
    let annotation_service: AnnotationServiceImpl<_, _, _> = AnnotationServiceImpl::new(
        annotation_repo.clone(),
        user_repo.clone(),
        project_repo.clone(),
    );
    // 마스크 그룹 서비스: 마스크 그룹 CRUD, 업로드 URL 생성 등
    let mask_group_service = Arc::new(MaskGroupServiceImpl::new(
        mask_group_repo.clone(),
        Arc::new(annotation_repo.clone()),
        Arc::new(user_repo.clone()),
    ));
    // 마스크 서비스: 개별 마스크 CRUD, 다운로드 URL 생성 등
    let mask_service = Arc::new(MaskServiceImpl::new(
        mask_repo.clone(),
        mask_group_repo.clone(),
        Arc::new(user_repo.clone()),
    ));
    // 프로젝트 데이터 서비스: 프로젝트 데이터 CRUD, 접근 권한 관리 등
    let project_data_service = Arc::new(ProjectDataServiceImpl::new(
        project_data_repo.clone(),
        project_data_access_repo.clone(),
    ));

    // 사용자 등록 서비스: 회원가입, 이메일 인증, 계정 삭제 등
    let user_registration_service =
        UserRegistrationServiceImpl::new(pool.clone(), (*keycloak_client).clone());
    // Initialize Object Storage service
    print!("☁️  Initializing Object Storage service... ");
    let object_storage = ObjectStorageServiceFactory::create(
        &settings.object_storage.provider,
        &settings.object_storage.bucket_name,
        &settings.object_storage.region,
        &settings.object_storage.endpoint,
        &settings.object_storage.access_key,
        &settings.object_storage.secret_key,
    )
    .await
    .map_err(|e| {
        eprintln!("❌ Failed to initialize Object Storage: {}", e);
        std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
    })?;
    let signed_url_service = Arc::new(SignedUrlServiceImpl::new(
        object_storage,
        settings.signed_url.default_ttl,
        settings.signed_url.max_ttl,
    ));
    println!("✅ Done (Provider: {})", settings.object_storage.provider);

    // Initialize use cases
    print!("📋 Initializing use cases... ");
    let auth_use_case = Arc::new(AuthUseCase::new(auth_service));
    let user_use_case = Arc::new(UserUseCase::new(user_service.clone()));
    let project_use_case = Arc::new(ProjectUseCase::new(project_service.clone()));
    let permission_use_case = Arc::new(PermissionUseCase::new(permission_service.clone()));
    let access_control_use_case = Arc::new(AccessControlUseCase::new(access_control_service));
    let annotation_use_case = Arc::new(AnnotationUseCase::new(annotation_service));
    let mask_group_use_case = Arc::new(MaskGroupUseCase::new(
        mask_group_service.clone(),
        signed_url_service.clone(),
    ));
    let mask_use_case = Arc::new(MaskUseCase::new(
        mask_service,
        mask_group_service.clone(),
        signed_url_service.clone(),
    ));
    let project_user_use_case = Arc::new(ProjectUserUseCase::new(
        Arc::new(project_service.clone()),
        Arc::new(user_service.clone()),
    ));
    let project_user_matrix_use_case = Arc::new(ProjectUserMatrixUseCase::new(
        Arc::new(project_service.clone()),
        Arc::new(user_service.clone()),
    ));
    let user_project_matrix_use_case = Arc::new(UserProjectMatrixUseCase::new(
        Arc::new(user_service.clone()),
        Arc::new(project_service.clone()),
    ));
    let role_permission_matrix_use_case = Arc::new(RolePermissionMatrixUseCase::new(Arc::new(
        permission_service.clone(),
    )));
    
    // Capability 서비스 및 Use Case 초기화
    let capability_repository = Arc::new(CapabilityRepositoryImpl::new(pool.clone()));
    let capability_service = Arc::new(CapabilityServiceImpl::new(capability_repository));
    let role_capability_matrix_use_case = Arc::new(RoleCapabilityMatrixUseCase::new(capability_service));
    
    let project_data_access_use_case =
        Arc::new(ProjectDataAccessUseCase::new(project_data_service.clone()));
    let user_registration_use_case =
        Arc::new(UserRegistrationUseCase::new(user_registration_service));
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
    println!(
        "✅ {} (TTL: {}s)",
        if cache_enabled { "Enabled" } else { "Disabled" },
        cache_ttl
    );

    // OpenAPI 문서 생성
    print!("📚 Generating OpenAPI documentation... ");
    let mut openapi = ApiDoc::openapi();
    openapi = presentation::openapi_extensions::extend_openapi(openapi);
    println!("✅ Done");

    println!("\n{}", "=".repeat(80));
    println!("✨ Server Ready!");
    println!("{}", "=".repeat(80));
    println!(
        "🌐 Server URL:    http://{}:{}",
        settings.server.host, settings.server.port
    );
    println!(
        "📖 Swagger UI:    http://{}:{}/swagger-ui/",
        settings.server.host, settings.server.port
    );
    println!(
        "❤️  Health Check:  http://{}:{}/health",
        settings.server.host, settings.server.port
    );
    println!(
        "🔌 API Endpoints: http://{}:{}/api/",
        settings.server.host, settings.server.port
    );
    println!("{}\n", "=".repeat(80));

    // Graceful shutdown을 위한 signal handler 설정
    let pool_for_shutdown = pool.clone();

    // Signal handler for graceful shutdown
    let shutdown_signal = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
        println!("\n🛑 Received shutdown signal, starting graceful shutdown...");

        // 데이터베이스 연결 풀 정리
        println!("📦 Closing database connection pool...");
        pool_for_shutdown.close().await;
        println!("✅ Database connections closed");
    };

    HttpServer::new(move || {
        App::new()
            // CORS middleware
            .wrap(configure_cors(&settings.cors))
            // Cache headers middleware
            .wrap(CacheHeaders::new(cache_enabled, cache_ttl))
            // Swagger UI (commented out for now)
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", openapi.clone()),
            )
            // Health check
            .route("/health", web::get().to(health_check))
            // API routes
            .service(
                web::scope("/api")
                    // ========================================
                    // 🔐 인증 관련 API (가장 먼저 등록)
                    // ========================================
                    .configure(|cfg| {
                        auth_controller::configure_routes(
                            cfg,
                            auth_use_case.clone(),
                            user_registration_use_case.clone(),
                        )
                    })
                    // ========================================
                    // 📊 프로젝트-사용자 매트릭스 API (먼저 등록)
                    // ========================================
                    .configure(|cfg| {
                        project_user_controller::configure_routes(
                            cfg,
                            project_user_use_case.clone(),
                        )
                    })
                    // ========================================
                    // 👥 사용자 관리 API
                    // ========================================
                    .configure(|cfg| user_controller::configure_routes(cfg, user_use_case.clone()))
                    // ========================================
                    // 🏗️ 프로젝트 관리 API
                    // ========================================
                    .configure(|cfg| {
                        project_controller::configure_routes(cfg, project_use_case.clone())
                    })
                    // ========================================
                    // 🔑 권한 관리 API (구체적인 경로 우선)
                    // ========================================
                    .configure(|cfg| {
                        role_permission_matrix_controller::configure_routes(
                            cfg,
                            role_permission_matrix_use_case.clone(),
                        )
                    })
                    .configure(|cfg| {
                        role_controller::configure_routes(
                            cfg,
                            permission_use_case.clone(),
                            role_capability_matrix_use_case.clone(),
                        )
                    })
                    .configure(|cfg| {
                        access_control_controller::configure_routes(
                            cfg,
                            access_control_use_case.clone(),
                        )
                    })
                    .configure(|cfg| {
                        project_user_matrix_controller::configure_routes(
                            cfg,
                            project_user_matrix_use_case.clone(),
                        )
                    })
                    .configure(|cfg| {
                        user_project_matrix_controller::configure_routes(
                            cfg,
                            user_project_matrix_use_case.clone(),
                        )
                    })
                    // ========================================
                    // 📁 데이터 접근 관리 API
                    // ========================================
                    .configure(|cfg| {
                        project_data_access_controller::configure_routes(
                            cfg,
                            project_data_access_use_case.clone(),
                        )
                    })
                    // ========================================
                    // 🎨 어노테이션 및 마스크 관리 API
                    // ========================================
                    .configure(|cfg| {
                        annotation_controller::configure_routes(cfg, annotation_use_case.clone())
                    })
                    .configure(|cfg| mask_controller::configure_routes(cfg, mask_use_case.clone()))
                    .configure(|cfg| {
                        mask_group_controller::configure_routes(cfg, mask_group_use_case.clone())
                    }),
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
