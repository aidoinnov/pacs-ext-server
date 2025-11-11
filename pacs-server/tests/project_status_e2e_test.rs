use actix_web::{test, web, App};
use pacs_server::application::use_cases::project_use_case::ProjectUseCase;
use pacs_server::domain::services::ProjectServiceImpl;
use pacs_server::infrastructure::repositories::{
    ProjectRepositoryImpl, RoleRepositoryImpl, UserRepositoryImpl,
};
use pacs_server::presentation::controllers::project_controller;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use uuid::Uuid;

/// E2E 테스트: 프로젝트 생성 → 메타데이터 조회 → 상태 변경 전체 플로우
#[actix_web::test]
#[ignore]
async fn test_e2e_project_lifecycle_with_metadata() {
    // Given: 테스트 앱 및 데이터베이스 설정
    let (app, pool) = setup_test_app().await;

    // Step 1: 메타데이터 조회 (프로젝트 생성 전)
    println!("\n=== Step 1: 메타데이터 조회 ===");
    let req = test::TestRequest::get()
        .uri("/api/projects/meta")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "메타데이터 조회 실패");

    let metadata: serde_json::Value = test::read_body_json(resp).await;
    let available_statuses = metadata["available_statuses"].as_array().unwrap();
    
    println!("✅ 사용 가능한 상태 목록:");
    for status in available_statuses {
        println!("  - {} ({}): {}", 
            status["value"].as_str().unwrap(),
            status["label"].as_str().unwrap(),
            status["description"].as_str().unwrap()
        );
    }

    assert_eq!(available_statuses.len(), 5, "5개의 상태가 있어야 함");

    // Step 2: 프로젝트 생성 (PREPARING 상태로 시작)
    println!("\n=== Step 2: 프로젝트 생성 ===");
    let project_name = format!("E2E Test Project {}", Uuid::new_v4());
    let create_json = serde_json::json!({
        "name": project_name,
        "description": "E2E 테스트용 프로젝트",
        "sponsor": "Test Sponsor",
        "start_date": "2025-01-01",
        "end_date": "2025-12-31",
        "auto_complete": false
    });

    let req = test::TestRequest::post()
        .uri("/api/projects")
        .set_json(&create_json)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201, "프로젝트 생성 실패");

    let project: serde_json::Value = test::read_body_json(resp).await;
    let project_id = project["id"].as_i64().unwrap() as i32;
    
    println!("✅ 프로젝트 생성 완료: ID={}, 상태={}", 
        project_id, 
        project["status"].as_str().unwrap()
    );
    assert_eq!(project["status"], "Preparing", "초기 상태는 PREPARING이어야 함");

    // Step 3: 프로젝트 조회
    println!("\n=== Step 3: 프로젝트 조회 ===");
    let req = test::TestRequest::get()
        .uri(&format!("/api/projects/{}", project_id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "프로젝트 조회 실패");

    let project: serde_json::Value = test::read_body_json(resp).await;
    println!("✅ 프로젝트 조회 완료: {}", project["name"].as_str().unwrap());

    // Step 4: 상태 변경 (PREPARING → IN_PROGRESS)
    println!("\n=== Step 4: 상태 변경 (PREPARING → IN_PROGRESS) ===");
    let update_json = serde_json::json!({
        "status": "IN_PROGRESS",
        "end_date": ""
    });

    let req = test::TestRequest::put()
        .uri(&format!("/api/projects/{}", project_id))
        .set_json(&update_json)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "상태 변경 실패");

    let project: serde_json::Value = test::read_body_json(resp).await;
    println!("✅ 상태 변경 완료: {}", project["status"].as_str().unwrap());
    assert_eq!(project["status"], "InProgress", "상태가 IN_PROGRESS로 변경되어야 함");

    // Step 5: 상태 변경 (IN_PROGRESS → COMPLETED)
    println!("\n=== Step 5: 상태 변경 (IN_PROGRESS → COMPLETED) ===");
    let update_json = serde_json::json!({
        "status": "COMPLETED",
        "end_date": ""
    });

    let req = test::TestRequest::put()
        .uri(&format!("/api/projects/{}", project_id))
        .set_json(&update_json)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "상태 변경 실패");

    let project: serde_json::Value = test::read_body_json(resp).await;
    println!("✅ 상태 변경 완료: {}", project["status"].as_str().unwrap());
    assert_eq!(project["status"], "Completed", "상태가 COMPLETED로 변경되어야 함");

    // Step 6: 최종 상태 확인
    println!("\n=== Step 6: 최종 상태 확인 ===");
    let req = test::TestRequest::get()
        .uri(&format!("/api/projects/{}", project_id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    let project: serde_json::Value = test::read_body_json(resp).await;
    
    println!("✅ 최종 프로젝트 상태:");
    println!("  - ID: {}", project["id"]);
    println!("  - 이름: {}", project["name"].as_str().unwrap());
    println!("  - 상태: {}", project["status"].as_str().unwrap());
    println!("  - 스폰서: {}", project["sponsor"].as_str().unwrap());

    assert_eq!(project["status"], "Completed");

    // Cleanup
    cleanup_project(&pool, project_id).await;
    println!("\n✅ E2E 테스트 완료!");
}

/// E2E 테스트: 메타데이터를 활용한 동적 상태 변경
#[actix_web::test]
#[ignore]
async fn test_e2e_dynamic_status_change_using_metadata() {
    // Given: 테스트 앱 설정
    let (app, pool) = setup_test_app().await;

    println!("\n=== E2E: 메타데이터 기반 동적 상태 변경 ===");

    // Step 1: 메타데이터에서 사용 가능한 상태 목록 가져오기
    let req = test::TestRequest::get()
        .uri("/api/projects/meta")
        .to_request();

    let resp = test::call_service(&app, req).await;
    let metadata: serde_json::Value = test::read_body_json(resp).await;
    let available_statuses = metadata["available_statuses"].as_array().unwrap();

    println!("✅ 메타데이터에서 {} 개의 상태 발견", available_statuses.len());

    // Step 2: 프로젝트 생성
    let project_name = format!("Dynamic Status Test {}", Uuid::new_v4());
    let create_json = serde_json::json!({
        "name": project_name,
        "description": "동적 상태 변경 테스트",
        "sponsor": "Test Sponsor",
        "start_date": "2025-01-01",
        "end_date": "2025-12-31",
        "auto_complete": false
    });

    let req = test::TestRequest::post()
        .uri("/api/projects")
        .set_json(&create_json)
        .to_request();

    let resp = test::call_service(&app, req).await;
    let project: serde_json::Value = test::read_body_json(resp).await;
    let project_id = project["id"].as_i64().unwrap() as i32;

    println!("✅ 프로젝트 생성: ID={}", project_id);

    // Step 3: 메타데이터의 각 상태로 변경 시도 (유효한 전환만)
    let valid_transitions = vec!["IN_PROGRESS", "ON_HOLD", "COMPLETED"];

    for status_value in valid_transitions {
        // 메타데이터에서 해당 상태 정보 찾기
        let status_meta = available_statuses
            .iter()
            .find(|s| s["value"].as_str().unwrap() == status_value)
            .expect(&format!("메타데이터에 {} 상태가 없음", status_value));

        println!("\n→ 상태 변경 시도: {} ({})", 
            status_meta["value"].as_str().unwrap(),
            status_meta["label"].as_str().unwrap()
        );

        let update_json = serde_json::json!({
            "status": status_value,
            "end_date": ""
        });

        let req = test::TestRequest::put()
            .uri(&format!("/api/projects/{}", project_id))
            .set_json(&update_json)
            .to_request();

        let resp = test::call_service(&app, req).await;
        
        if resp.status().is_success() {
            let project: serde_json::Value = test::read_body_json(resp).await;
            println!("  ✅ 성공: 현재 상태 = {}", project["status"].as_str().unwrap());
        } else {
            println!("  ⚠️  실패: {}", resp.status());
        }
    }

    // Cleanup
    cleanup_project(&pool, project_id).await;
    println!("\n✅ 동적 상태 변경 테스트 완료!");
}

/// E2E 테스트: 잘못된 상태 값 처리
#[actix_web::test]
#[ignore]
async fn test_e2e_invalid_status_handling() {
    // Given: 테스트 앱 설정
    let (app, pool) = setup_test_app().await;

    println!("\n=== E2E: 잘못된 상태 값 처리 ===");

    // Step 1: 프로젝트 생성
    let project_name = format!("Invalid Status Test {}", Uuid::new_v4());
    let create_json = serde_json::json!({
        "name": project_name,
        "description": "잘못된 상태 처리 테스트",
        "sponsor": "Test Sponsor",
        "start_date": "2025-01-01",
        "end_date": "2025-12-31",
        "auto_complete": false
    });

    let req = test::TestRequest::post()
        .uri("/api/projects")
        .set_json(&create_json)
        .to_request();

    let resp = test::call_service(&app, req).await;
    let project: serde_json::Value = test::read_body_json(resp).await;
    let project_id = project["id"].as_i64().unwrap() as i32;
    let original_status = project["status"].as_str().unwrap().to_string();

    println!("✅ 프로젝트 생성: ID={}, 초기 상태={}", project_id, original_status);

    // Step 2: 잘못된 상태 값으로 변경 시도
    let invalid_statuses = vec!["INVALID", "UNKNOWN", "TEST", ""];

    for invalid_status in invalid_statuses {
        println!("\n→ 잘못된 상태로 변경 시도: '{}'", invalid_status);

        let update_json = serde_json::json!({
            "status": invalid_status,
            "end_date": ""
        });

        let req = test::TestRequest::put()
            .uri(&format!("/api/projects/{}", project_id))
            .set_json(&update_json)
            .to_request();

        let resp = test::call_service(&app, req).await;
        let project: serde_json::Value = test::read_body_json(resp).await;
        
        println!("  현재 상태: {}", project["status"].as_str().unwrap());
        // 잘못된 값은 무시되고 상태가 변경되지 않아야 함
    }

    // Cleanup
    cleanup_project(&pool, project_id).await;
    println!("\n✅ 잘못된 상태 처리 테스트 완료!");
}

// ========================================
// 헬퍼 함수
// ========================================

/// 테스트 앱 및 데이터베이스 설정
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

/// 테스트 프로젝트 정리
async fn cleanup_project(pool: &sqlx::PgPool, project_id: i32) {
    let _ = sqlx::query("DELETE FROM security_project WHERE id = $1")
        .bind(project_id)
        .execute(pool)
        .await;
}

