use actix_web::{test, web, App};
use pacs_server::application::use_cases::project_use_case::ProjectUseCase;
use pacs_server::domain::services::ProjectServiceImpl;
use pacs_server::infrastructure::repositories::{
    ProjectRepositoryImpl, RoleRepositoryImpl, UserRepositoryImpl,
};
use pacs_server::presentation::controllers::project_controller;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;

/// 통합 테스트: 프로젝트 메타데이터 조회
#[actix_web::test]
#[ignore]
async fn test_get_project_metadata() {
    // Given: 테스트 앱 설정
    let app = setup_test_app().await;

    // When: 메타데이터 조회
    let req = test::TestRequest::get()
        .uri("/api/projects/meta")
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Then: 성공 응답 확인
    assert_eq!(resp.status(), 200, "Metadata request should succeed");

    let body: serde_json::Value = test::read_body_json(resp).await;

    // available_statuses 필드 확인
    assert!(
        body.get("available_statuses").is_some(),
        "Response should have available_statuses field"
    );

    let statuses = body["available_statuses"].as_array().unwrap();
    assert_eq!(statuses.len(), 5, "Should have 5 status options");

    // 각 상태 확인
    let status_values: Vec<&str> = statuses
        .iter()
        .map(|s| s["value"].as_str().unwrap())
        .collect();

    assert!(status_values.contains(&"PREPARING"), "Should have PREPARING");
    assert!(status_values.contains(&"IN_PROGRESS"), "Should have IN_PROGRESS");
    assert!(status_values.contains(&"COMPLETED"), "Should have COMPLETED");
    assert!(status_values.contains(&"ON_HOLD"), "Should have ON_HOLD");
    assert!(status_values.contains(&"CANCELLED"), "Should have CANCELLED");
}

/// 통합 테스트: 메타데이터 응답 구조 검증
#[actix_web::test]
#[ignore]
async fn test_project_metadata_response_structure() {
    // Given: 테스트 앱 설정
    let app = setup_test_app().await;

    // When: 메타데이터 조회
    let req = test::TestRequest::get()
        .uri("/api/projects/meta")
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Then: 응답 구조 확인
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    let statuses = body["available_statuses"].as_array().unwrap();

    // 첫 번째 상태 객체 구조 확인
    let first_status = &statuses[0];
    
    assert!(
        first_status.get("value").is_some(),
        "Status should have 'value' field"
    );
    assert!(
        first_status.get("label").is_some(),
        "Status should have 'label' field"
    );
    assert!(
        first_status.get("description").is_some(),
        "Status should have 'description' field"
    );

    // 값 타입 확인
    assert!(first_status["value"].is_string(), "value should be string");
    assert!(first_status["label"].is_string(), "label should be string");
    assert!(
        first_status["description"].is_string(),
        "description should be string"
    );
}

/// 통합 테스트: 메타데이터 내용 검증
#[actix_web::test]
#[ignore]
async fn test_project_metadata_content() {
    // Given: 테스트 앱 설정
    let app = setup_test_app().await;

    // When: 메타데이터 조회
    let req = test::TestRequest::get()
        .uri("/api/projects/meta")
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Then: 각 상태의 내용 확인
    let body: serde_json::Value = test::read_body_json(resp).await;
    let statuses = body["available_statuses"].as_array().unwrap();

    // PREPARING 상태 확인
    let preparing = statuses
        .iter()
        .find(|s| s["value"] == "PREPARING")
        .expect("Should have PREPARING status");
    
    assert_eq!(preparing["label"], "준비중");
    assert_eq!(preparing["description"], "프로젝트가 생성되었지만 아직 시작되지 않음");

    // IN_PROGRESS 상태 확인
    let in_progress = statuses
        .iter()
        .find(|s| s["value"] == "IN_PROGRESS")
        .expect("Should have IN_PROGRESS status");
    
    assert_eq!(in_progress["label"], "진행중");
    assert_eq!(in_progress["description"], "프로젝트가 활발히 진행 중");

    // COMPLETED 상태 확인
    let completed = statuses
        .iter()
        .find(|s| s["value"] == "COMPLETED")
        .expect("Should have COMPLETED status");
    
    assert_eq!(completed["label"], "완료");
    assert_eq!(completed["description"], "프로젝트가 성공적으로 완료됨");

    // ON_HOLD 상태 확인
    let on_hold = statuses
        .iter()
        .find(|s| s["value"] == "ON_HOLD")
        .expect("Should have ON_HOLD status");
    
    assert_eq!(on_hold["label"], "보류");
    assert_eq!(on_hold["description"], "프로젝트가 일시적으로 중단됨");

    // CANCELLED 상태 확인
    let cancelled = statuses
        .iter()
        .find(|s| s["value"] == "CANCELLED")
        .expect("Should have CANCELLED status");
    
    assert_eq!(cancelled["label"], "취소");
    assert_eq!(cancelled["description"], "프로젝트가 취소됨");
}

/// 통합 테스트: 메타데이터 엔드포인트는 인증 없이 접근 가능
#[actix_web::test]
#[ignore]
async fn test_project_metadata_no_auth_required() {
    // Given: 테스트 앱 설정
    let app = setup_test_app().await;

    // When: 인증 헤더 없이 메타데이터 조회
    let req = test::TestRequest::get()
        .uri("/api/projects/meta")
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Then: 성공 응답 (인증 불필요)
    assert_eq!(
        resp.status(),
        200,
        "Metadata endpoint should be accessible without authentication"
    );
}

// ========================================
// 헬퍼 함수
// ========================================

/// 테스트 앱 설정
async fn setup_test_app() -> impl actix_web::dev::Service<
    actix_http::Request,
    Response = actix_web::dev::ServiceResponse<actix_web::body::BoxBody>,
    Error = actix_web::Error,
> {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .or_else(|_| std::env::var("APP_DATABASE_URL"))
        .expect("DATABASE_URL or APP_DATABASE_URL not set for test");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    // 서비스 및 UseCase 설정
    let user_repo = UserRepositoryImpl::new(pool.clone());
    let project_repo = ProjectRepositoryImpl::new(pool.clone());
    let role_repo = RoleRepositoryImpl::new(pool.clone());

    let project_service = ProjectServiceImpl::new(project_repo, user_repo, role_repo);
    let project_use_case = Arc::new(ProjectUseCase::new(project_service));

    // 앱 설정
    test::init_service(
        App::new()
            .app_data(web::Data::new(project_use_case.clone()))
            .service(web::scope("/api").configure(|cfg| {
                project_controller::configure_routes(cfg, project_use_case.clone())
            })),
    )
    .await
}

