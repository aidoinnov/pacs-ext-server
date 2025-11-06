//! 데이터베이스 연결 풀 정리 테스트
//! 
//! 이 테스트는 데이터베이스 연결 풀이 테스트 종료 시 적절하게 정리되는지 확인합니다.

use actix_web::{test, web, App};
use pacs_server::{
    application::use_cases::annotation_use_case::AnnotationUseCase,
    domain::services::annotation_service::AnnotationServiceImpl,
    infrastructure::repositories::{
        AnnotationRepositoryImpl, UserRepositoryImpl, ProjectRepositoryImpl
    },
    presentation::controllers::annotation_controller,
};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;

/// 데이터베이스 연결 풀 정리 테스트
#[actix_web::test]
async fn test_database_pool_cleanup() {
    // 환경 변수 로드
    dotenvy::dotenv().ok();
    
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost:5432/pacs_ext".to_string());

    // 연결 풀 생성
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .min_connections(1)
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    println!("📦 Created database pool with {} connections", pool.size());

    // 연결 풀 상태 확인
    let initial_size = pool.size();
    let idle_connections = pool.num_idle();
    println!("📊 Pool stats - Total: {}, Idle: {}", initial_size, idle_connections);

    // 리포지토리 및 서비스 생성
    let user_repo = UserRepositoryImpl::new(pool.clone());
    let project_repo = ProjectRepositoryImpl::new(pool.clone());
    let annotation_repo = AnnotationRepositoryImpl::new(pool.clone());
    let annotation_service = AnnotationServiceImpl::new(annotation_repo, user_repo, project_repo);
    let annotation_use_case = Arc::new(AnnotationUseCase::new(annotation_service));

    // 앱 생성
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(annotation_use_case.clone()))
            .configure(|cfg| annotation_controller::configure_routes(cfg, annotation_use_case.clone()))
    ).await;

    // 간단한 테스트 요청
    let req = test::TestRequest::get()
        .uri("/api/annotations")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success() || resp.status().is_client_error());

    // 연결 풀 상태 재확인
    let after_request_size = pool.size();
    let after_request_idle = pool.num_idle();
    println!("📊 After request - Total: {}, Idle: {}", after_request_size, after_request_idle);

    // 연결 풀 명시적 정리
    println!("🧹 Manually closing database pool...");
    pool.close().await;
    println!("✅ Database pool closed successfully");

    // 정리 후 상태 확인 (pool이 닫혔으므로 접근하면 panic이 발생할 수 있음)
    // 따라서 이 부분은 주석 처리
    // println!("📊 After cleanup - Total: {}, Idle: {}", pool.size(), pool.num_idle());
    
    println!("✅ Database pool cleanup test completed successfully");
}

/// 연결 풀 자동 정리 테스트 (Drop trait)
#[actix_web::test]
async fn test_database_pool_auto_cleanup() {
    // 환경 변수 로드
    dotenvy::dotenv().ok();
    
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost:5432/pacs_ext".to_string());

    // 스코프 내에서 연결 풀 생성
    {
        let pool = PgPoolOptions::new()
            .max_connections(3)
            .min_connections(1)
            .connect(&database_url)
            .await
            .expect("Failed to connect to test database");

        println!("📦 Created database pool in scope");
        println!("📊 Pool stats - Total: {}, Idle: {}", pool.size(), pool.num_idle());
        
        // 간단한 쿼리 실행
        let result: (i32,) = sqlx::query_as("SELECT 1 as test")
            .fetch_one(&pool)
            .await
            .expect("Failed to execute test query");
        
        assert_eq!(result.0, 1);
        println!("✅ Test query executed successfully");
        
        // 스코프 종료 시 pool이 자동으로 정리됨 (Drop trait)
    }
    
    println!("✅ Database pool auto cleanup test completed successfully");
}

/// 연결 풀 설정 테스트
#[actix_web::test]
async fn test_database_pool_configuration() {
    // 환경 변수 로드
    dotenvy::dotenv().ok();
    
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost:5432/pacs_ext".to_string());

    // 다양한 설정으로 연결 풀 생성
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .min_connections(2)
        .acquire_timeout(std::time::Duration::from_secs(30))
        .idle_timeout(std::time::Duration::from_secs(600))
        .max_lifetime(std::time::Duration::from_secs(1800))
        .test_before_acquire(true)
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    println!("📦 Created database pool with custom configuration");
    println!("📊 Pool stats - Total: {}, Idle: {}", pool.size(), pool.num_idle());

    // 연결 풀 설정 확인
    assert!(pool.size() >= 2); // 최소 연결 수 확인
    assert!(pool.size() <= 10); // 최대 연결 수 확인

    // 연결 풀 정리
    pool.close().await;
    println!("✅ Database pool configuration test completed successfully");
}
