use actix_web::{test, web, App};
use chrono::NaiveDate;
use pacs_server::application::dto::project_dto::{CreateProjectRequest, UpdateProjectRequest};
use pacs_server::application::use_cases::project_use_case::ProjectUseCase;
use pacs_server::domain::entities::project::ProjectStatus;
use pacs_server::domain::services::ProjectServiceImpl;
use pacs_server::infrastructure::repositories::{
    ProjectRepositoryImpl, RoleRepositoryImpl, UserRepositoryImpl,
};
use pacs_server::presentation::controllers::project_controller;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use uuid::Uuid;

/// 통합 테스트: 프로젝트 생성 시 PREPARING 상태로 시작
#[actix_web::test]
#[ignore]
async fn test_project_created_with_preparing_status() {
    // Given: 테스트 앱 설정
    let (app, pool) = setup_test_app().await;

    // When: 프로젝트 생성
    let project_name = format!("Test Project {}", Uuid::new_v4());
    let create_req = CreateProjectRequest {
        name: project_name.clone(),
        description: Some("Test project for status verification".to_string()),
        sponsor: "Test Sponsor".to_string(),
        start_date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        end_date: Some(NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()),
        auto_complete: Some(false),
    };

    let req = test::TestRequest::post()
        .uri("/api/projects")
        .set_json(&create_req)
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Then: 성공 응답 확인
    assert_eq!(resp.status(), 201, "Project creation should succeed");

    let body: serde_json::Value = test::read_body_json(resp).await;
    let project_id = body["id"].as_i64().unwrap() as i32;

    // 상태가 PREPARING인지 확인
    assert_eq!(
        body["status"].as_str().unwrap(),
        "Preparing",
        "New project should have PREPARING status"
    );

    // Cleanup
    cleanup_project(&pool, project_id).await;
}

/// 통합 테스트: PREPARING → IN_PROGRESS 상태 변경
#[actix_web::test]
#[ignore]
async fn test_project_status_preparing_to_in_progress() {
    // Given: 프로젝트 생성 (PREPARING 상태)
    let (app, pool) = setup_test_app().await;
    let project_id = create_test_project(&pool, "PREPARING").await;

    // When: IN_PROGRESS로 상태 변경
    let update_json = serde_json::json!({
        "status": "IN_PROGRESS",
        "end_date": "" // 빈 문자열로 None 처리
    });

    let req = test::TestRequest::put()
        .uri(&format!("/api/projects/{}", project_id))
        .set_json(&update_json)
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Then: 성공 응답 및 상태 확인
    assert_eq!(resp.status(), 200, "Status update should succeed");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(
        body["status"].as_str().unwrap(),
        "InProgress",
        "Status should be updated to IN_PROGRESS"
    );

    // 데이터베이스에서 직접 확인
    let db_status: String = sqlx::query_scalar(
        "SELECT status::text FROM security_project WHERE id = $1"
    )
    .bind(project_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(db_status, "IN_PROGRESS", "Database status should be IN_PROGRESS");

    // Cleanup
    cleanup_project(&pool, project_id).await;
}

/// 통합 테스트: IN_PROGRESS → ON_HOLD 상태 변경 (일시 중단)
#[actix_web::test]
#[ignore]
async fn test_project_status_in_progress_to_on_hold() {
    // Given: 진행중인 프로젝트
    let (app, pool) = setup_test_app().await;
    let project_id = create_test_project(&pool, "IN_PROGRESS").await;

    // When: ON_HOLD로 상태 변경
    let update_json = serde_json::json!({
        "status": "ON_HOLD",
        "end_date": ""
    });

    let req = test::TestRequest::put()
        .uri(&format!("/api/projects/{}", project_id))
        .set_json(&update_json)
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Then: 성공 응답 및 상태 확인
    assert_eq!(resp.status(), 200, "Status update should succeed");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(
        body["status"].as_str().unwrap(),
        "OnHold",
        "Status should be updated to ON_HOLD"
    );

    // Cleanup
    cleanup_project(&pool, project_id).await;
}

/// 통합 테스트: ON_HOLD → IN_PROGRESS 상태 변경 (재개)
#[actix_web::test]
#[ignore]
async fn test_project_status_on_hold_to_in_progress() {
    // Given: 보류 중인 프로젝트
    let (app, pool) = setup_test_app().await;
    let project_id = create_test_project(&pool, "ON_HOLD").await;

    // When: IN_PROGRESS로 상태 변경 (재개)
    let update_json = serde_json::json!({
        "status": "IN_PROGRESS",
        "end_date": ""
    });

    let req = test::TestRequest::put()
        .uri(&format!("/api/projects/{}", project_id))
        .set_json(&update_json)
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Then: 성공 응답 및 상태 확인
    assert_eq!(resp.status(), 200, "Status update should succeed");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(
        body["status"].as_str().unwrap(),
        "InProgress",
        "Status should be updated to IN_PROGRESS"
    );

    // Cleanup
    cleanup_project(&pool, project_id).await;
}

/// 통합 테스트: IN_PROGRESS → COMPLETED 상태 변경 (완료)
#[actix_web::test]
#[ignore]
async fn test_project_status_in_progress_to_completed() {
    // Given: 진행중인 프로젝트
    let (app, pool) = setup_test_app().await;
    let project_id = create_test_project(&pool, "IN_PROGRESS").await;

    // When: COMPLETED로 상태 변경
    let update_json = serde_json::json!({
        "status": "COMPLETED",
        "end_date": ""
    });

    let req = test::TestRequest::put()
        .uri(&format!("/api/projects/{}", project_id))
        .set_json(&update_json)
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Then: 성공 응답 및 상태 확인
    assert_eq!(resp.status(), 200, "Status update should succeed");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(
        body["status"].as_str().unwrap(),
        "Completed",
        "Status should be updated to COMPLETED"
    );

    // Cleanup
    cleanup_project(&pool, project_id).await;
}

/// 통합 테스트: IN_PROGRESS → CANCELLED 상태 변경 (취소)
#[actix_web::test]
#[ignore]
async fn test_project_status_in_progress_to_cancelled() {
    // Given: 진행중인 프로젝트
    let (app, pool) = setup_test_app().await;
    let project_id = create_test_project(&pool, "IN_PROGRESS").await;

    // When: CANCELLED로 상태 변경
    let update_json = serde_json::json!({
        "status": "CANCELLED",
        "end_date": ""
    });

    let req = test::TestRequest::put()
        .uri(&format!("/api/projects/{}", project_id))
        .set_json(&update_json)
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Then: 성공 응답 및 상태 확인
    assert_eq!(resp.status(), 200, "Status update should succeed");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(
        body["status"].as_str().unwrap(),
        "Cancelled",
        "Status should be updated to CANCELLED"
    );

    // Cleanup
    cleanup_project(&pool, project_id).await;
}

/// 통합 테스트: 잘못된 상태 값 입력 시 처리
#[actix_web::test]
#[ignore]
async fn test_project_status_invalid_value() {
    // Given: 진행중인 프로젝트
    let (app, pool) = setup_test_app().await;
    let project_id = create_test_project(&pool, "IN_PROGRESS").await;

    // 원래 상태 저장
    let original_status: String = sqlx::query_scalar(
        "SELECT status::text FROM security_project WHERE id = $1"
    )
    .bind(project_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    // When: 잘못된 상태 값으로 업데이트 시도
    let update_json = serde_json::json!({
        "status": "INVALID_STATUS",
        "end_date": ""
    });

    let req = test::TestRequest::put()
        .uri(&format!("/api/projects/{}", project_id))
        .set_json(&update_json)
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Then: 성공 응답이지만 상태는 변경되지 않음
    assert_eq!(resp.status(), 200, "Request should succeed");

    let _body: serde_json::Value = test::read_body_json(resp).await;

    // 상태가 변경되지 않았는지 확인
    let current_status: String = sqlx::query_scalar(
        "SELECT status::text FROM security_project WHERE id = $1"
    )
    .bind(project_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(
        current_status, original_status,
        "Status should not change with invalid value"
    );

    // Cleanup
    cleanup_project(&pool, project_id).await;
}

/// 통합 테스트: 여러 필드 동시 업데이트 (상태 포함)
#[actix_web::test]
#[ignore]
async fn test_project_status_update_with_other_fields() {
    // Given: 준비중인 프로젝트
    let (app, pool) = setup_test_app().await;
    let project_id = create_test_project(&pool, "PREPARING").await;

    // When: 여러 필드를 동시에 업데이트 (상태 포함)
    let update_json = serde_json::json!({
        "name": "Updated Project Name",
        "description": "Updated description",
        "sponsor": "Updated Sponsor",
        "status": "IN_PROGRESS",
        "auto_complete": true,
        "is_active": true,
        "end_date": ""
    });

    let req = test::TestRequest::put()
        .uri(&format!("/api/projects/{}", project_id))
        .set_json(&update_json)
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Then: 모든 필드가 업데이트되었는지 확인
    assert_eq!(resp.status(), 200, "Update should succeed");

    let body: serde_json::Value = test::read_body_json(resp).await;

    assert_eq!(body["name"].as_str().unwrap(), "Updated Project Name");
    assert_eq!(body["description"].as_str().unwrap(), "Updated description");
    assert_eq!(body["sponsor"].as_str().unwrap(), "Updated Sponsor");
    assert_eq!(body["status"].as_str().unwrap(), "InProgress");
    assert_eq!(body["auto_complete"].as_bool().unwrap(), true);
    assert_eq!(body["is_active"].as_bool().unwrap(), true);

    // Cleanup
    cleanup_project(&pool, project_id).await;
}

/// 통합 테스트: 대소문자 구분 없이 상태 변경
#[actix_web::test]
#[ignore]
async fn test_project_status_case_insensitive() {
    // Given: 준비중인 프로젝트
    let (app, pool) = setup_test_app().await;
    let project_id = create_test_project(&pool, "PREPARING").await;

    // When: 소문자 + 카멜케이스로 상태 변경
    let update_json = serde_json::json!({
        "status": "InProgress", // 카멜케이스
        "end_date": ""
    });

    let req = test::TestRequest::put()
        .uri(&format!("/api/projects/{}", project_id))
        .set_json(&update_json)
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Then: 성공 응답 및 상태 확인
    assert_eq!(resp.status(), 200, "Status update should succeed");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(
        body["status"].as_str().unwrap(),
        "InProgress",
        "Status should be updated regardless of case"
    );

    // Cleanup
    cleanup_project(&pool, project_id).await;
}

// ========================================
// 헬퍼 함수들
// ========================================

/// 테스트 앱 설정
async fn setup_test_app() -> (
    impl actix_web::dev::Service<
        actix_http::Request,
        Response = actix_web::dev::ServiceResponse<actix_web::body::BoxBody>,
        Error = actix_web::Error,
    >,
    sqlx::PgPool,
) {
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
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(project_use_case.clone()))
            .service(web::scope("/api").configure(|cfg| {
                project_controller::configure_routes(cfg, project_use_case.clone())
            })),
    )
    .await;

    (app, pool)
}

/// 테스트 프로젝트 생성 (특정 상태로)
async fn create_test_project(pool: &sqlx::PgPool, status: &str) -> i32 {
    let project_name = format!("Test Project {} {}", status, Uuid::new_v4());
    let description = format!("Test project with {} status", status);

    let rec: (i32,) = sqlx::query_as(
        "INSERT INTO security_project (name, description, sponsor, start_date, end_date, status)
         VALUES ($1, $2, 'Test Sponsor', CURRENT_DATE, CURRENT_DATE + INTERVAL '1 year', $3::project_status)
         RETURNING id",
    )
    .bind(&project_name)
    .bind(&description)
    .bind(status)
    .fetch_one(pool)
    .await
    .expect("Failed to create test project");

    rec.0
}

/// 테스트 프로젝트 정리
async fn cleanup_project(pool: &sqlx::PgPool, project_id: i32) {
    let _ = sqlx::query("DELETE FROM security_project WHERE id = $1")
        .bind(project_id)
        .execute(pool)
        .await;
}
