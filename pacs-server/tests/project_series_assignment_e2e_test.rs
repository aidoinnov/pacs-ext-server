use actix_web::{test, web, App};
use pacs_server::application::dto::project_data_access_dto::{
    AssignSeriesToProjectRequest, AssignStudyToProjectRequest,
};
use pacs_server::application::use_cases::project_data_access_use_case::ProjectDataAccessUseCase;
use pacs_server::domain::services::ProjectServiceImpl;
use pacs_server::infrastructure::repositories::{
    ProjectDataAccessRepositoryImpl, ProjectDataRepositoryImpl, ProjectRepositoryImpl,
    RoleRepositoryImpl, UserRepositoryImpl,
};
use pacs_server::infrastructure::services::ProjectDataServiceImpl;
use pacs_server::presentation::controllers::project_data_access_controller;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use uuid::Uuid;

// ========================================
// E2E 테스트: 전체 워크플로우 검증
// ========================================

/// E2E 테스트 1: 프로젝트 생성 → Series 할당 → 조회 → 삭제 전체 흐름
#[actix_web::test]
#[ignore] // 실제 DB 필요
async fn test_complete_series_assignment_workflow() {
    // Given: 테스트 앱 설정
    let (app, pool) = setup_test_app().await;

    // Step 1: 프로젝트 생성
    let project_id = create_test_project(&pool).await;
    println!("✅ Step 1: Created project with ID: {}", project_id);

    // Step 2: Study 할당
    let study_uid = format!("1.2.840.113619.2.1.1.{}", Uuid::new_v4().as_u128());
    let study_req = AssignStudyToProjectRequest {
        study_uid: study_uid.clone(),
        study_description: Some("CT Chest with Contrast".to_string()),
        patient_id: Some("P12345".to_string()),
        patient_name: Some("John Doe".to_string()),
        study_date: Some("2024-01-15".to_string()),
        modality: Some("CT".to_string()),
    };

    let req = test::TestRequest::post()
        .uri(&format!("/api/projects/{}/studies/assign", project_id))
        .set_json(&study_req)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Study assignment should succeed");

    let body: serde_json::Value = test::read_body_json(resp).await;
    let study_id = body["study_id"].as_i64().unwrap();
    println!("✅ Step 2: Assigned study with ID: {}", study_id);

    // Step 3: Series 할당 (3개)
    let series_uids: Vec<String> = (1..=3)
        .map(|i| format!("1.2.840.113619.2.1.2.{}.{}", Uuid::new_v4().as_u128(), i))
        .collect();

    for (i, series_uid) in series_uids.iter().enumerate() {
        let series_req = AssignSeriesToProjectRequest {
            study_uid: study_uid.clone(),
            series_uid: series_uid.clone(),
            series_description: Some(format!("Axial CT {}mm", (i + 1) * 5)),
            modality: Some("CT".to_string()),
            series_number: Some((i + 1) as i32),
        };

        let req = test::TestRequest::post()
            .uri(&format!("/api/projects/{}/series/assign", project_id))
            .set_json(&series_req)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200, "Series {} assignment should succeed", i + 1);

        let body: serde_json::Value = test::read_body_json(resp).await;
        println!(
            "✅ Step 3.{}: Assigned series with ID: {}",
            i + 1,
            body["series_id"].as_i64().unwrap()
        );
    }

    // Step 4: 프로젝트 Studies 목록 조회
    let req = test::TestRequest::get()
        .uri(&format!("/api/project-data/{}/studies", project_id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Studies list should be retrieved");

    let response: serde_json::Value = test::read_body_json(resp).await;
    let studies = response["studies"].as_array().expect("studies should be an array");
    assert_eq!(studies.len(), 1, "Should have 1 study");
    assert_eq!(
        studies[0]["study_uid"].as_str().unwrap(),
        study_uid,
        "Study UID should match"
    );
    println!("✅ Step 4: Retrieved {} studies", studies.len());

    // Step 5: Study의 Series 목록 조회
    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/project-data/{}/studies/{}/series",
            project_id, study_id
        ))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Series list should be retrieved");

    let response: serde_json::Value = test::read_body_json(resp).await;
    let series_list = response["series"].as_array().expect("series should be an array");
    assert_eq!(series_list.len(), 3, "Should have 3 series");

    // Series UID 확인
    let retrieved_series_uids: Vec<String> = series_list
        .iter()
        .filter_map(|s| s["series"]["series_uid"].as_str().map(|s| s.to_string()))
        .collect();

    for series_uid in &series_uids {
        assert!(
            retrieved_series_uids.contains(series_uid),
            "Series UID {} should be in the list",
            series_uid
        );
    }
    println!("✅ Step 5: Retrieved {} series", series_list.len());

    // Step 6: 데이터베이스 직접 확인
    let db_series_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM project_data pd
         WHERE pd.project_id = $1 AND pd.resource_level = 'SERIES'",
    )
    .bind(project_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(db_series_count, 3, "Database should have 3 series mappings");
    println!("✅ Step 6: Verified {} series in database", db_series_count);

    // Step 7: 정리
    cleanup_test_data(&pool, project_id).await;
    println!("✅ Step 7: Cleaned up test data");

    println!("\n🎉 E2E Test Complete: All steps passed!");
}

/// E2E 테스트 2: 프로젝트 격리 검증 (다른 프로젝트의 데이터는 조회 불가)
#[actix_web::test]
#[ignore]
async fn test_project_data_isolation() {
    // Given: 2개의 프로젝트 생성
    let (app, pool) = setup_test_app().await;
    let project_a_id = create_test_project(&pool).await;
    let project_b_id = create_test_project(&pool).await;

    println!("✅ Created Project A: {}", project_a_id);
    println!("✅ Created Project B: {}", project_b_id);

    // Step 1: Project A에 Series 할당
    let study_uid_a = format!("1.2.840.113619.2.1.1.{}", Uuid::new_v4().as_u128());
    let series_uid_a = format!("1.2.840.113619.2.1.2.{}.1", Uuid::new_v4().as_u128());

    let req_a = AssignSeriesToProjectRequest {
        study_uid: study_uid_a.clone(),
        series_uid: series_uid_a.clone(),
        series_description: Some("Project A Series".to_string()),
        modality: Some("CT".to_string()),
        series_number: Some(1),
    };

    let req = test::TestRequest::post()
        .uri(&format!("/api/projects/{}/series/assign", project_a_id))
        .set_json(&req_a)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
    println!("✅ Assigned series to Project A");

    // Step 2: Project B에 다른 Series 할당
    let study_uid_b = format!("1.2.840.113619.2.1.1.{}", Uuid::new_v4().as_u128());
    let series_uid_b = format!("1.2.840.113619.2.1.2.{}.1", Uuid::new_v4().as_u128());

    let req_b = AssignSeriesToProjectRequest {
        study_uid: study_uid_b.clone(),
        series_uid: series_uid_b.clone(),
        series_description: Some("Project B Series".to_string()),
        modality: Some("MR".to_string()),
        series_number: Some(1),
    };

    let req = test::TestRequest::post()
        .uri(&format!("/api/projects/{}/series/assign", project_b_id))
        .set_json(&req_b)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
    println!("✅ Assigned series to Project B");

    // Step 3: Project A 조회 → Project A의 데이터만 반환
    let req = test::TestRequest::get()
        .uri(&format!("/api/project-data/{}/studies", project_a_id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let response_a: serde_json::Value = test::read_body_json(resp).await;
    let studies_a = response_a["studies"].as_array().expect("studies should be an array");
    assert_eq!(studies_a.len(), 1, "Project A should have 1 study");

    let study_uids_a: Vec<String> = studies_a
        .iter()
        .filter_map(|s| s["study_uid"].as_str().map(|s| s.to_string()))
        .collect();

    assert!(
        study_uids_a.contains(&study_uid_a),
        "Project A should contain its own study"
    );
    assert!(
        !study_uids_a.contains(&study_uid_b),
        "Project A should NOT contain Project B's study"
    );
    println!("✅ Project A isolation verified");

    // Step 4: Project B 조회 → Project B의 데이터만 반환
    let req = test::TestRequest::get()
        .uri(&format!("/api/project-data/{}/studies", project_b_id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let response_b: serde_json::Value = test::read_body_json(resp).await;
    let studies_b = response_b["studies"].as_array().expect("studies should be an array");
    assert_eq!(studies_b.len(), 1, "Project B should have 1 study");

    let study_uids_b: Vec<String> = studies_b
        .iter()
        .filter_map(|s| s["study_uid"].as_str().map(|s| s.to_string()))
        .collect();

    assert!(
        study_uids_b.contains(&study_uid_b),
        "Project B should contain its own study"
    );
    assert!(
        !study_uids_b.contains(&study_uid_a),
        "Project B should NOT contain Project A's study"
    );
    println!("✅ Project B isolation verified");

    // Cleanup
    cleanup_test_data(&pool, project_a_id).await;
    cleanup_test_data(&pool, project_b_id).await;
    println!("✅ Cleaned up test data");

    println!("\n🎉 E2E Test Complete: Project isolation verified!");
}

/// E2E 테스트 3: 에러 처리 전체 흐름 (404, 409)
#[actix_web::test]
#[ignore]
async fn test_error_handling_workflow() {
    // Given: 테스트 앱 설정
    let (app, pool) = setup_test_app().await;
    let project_id = create_test_project(&pool).await;

    println!("✅ Created project: {}", project_id);

    // Step 1: 존재하지 않는 프로젝트에 할당 시도 → 404
    let nonexistent_project_id = 999999;
    let req_body = AssignSeriesToProjectRequest {
        study_uid: format!("1.2.840.113619.2.1.1.{}", Uuid::new_v4().as_u128()),
        series_uid: format!("1.2.840.113619.2.1.2.{}.1", Uuid::new_v4().as_u128()),
        series_description: Some("Test Series".to_string()),
        modality: Some("CT".to_string()),
        series_number: Some(1),
    };

    let req = test::TestRequest::post()
        .uri(&format!(
            "/api/projects/{}/series/assign",
            nonexistent_project_id
        ))
        .set_json(&req_body)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404, "Should return 404 for nonexistent project");
    println!("✅ Step 1: 404 error handled correctly");

    // Step 2: 정상 할당
    let study_uid = format!("1.2.840.113619.2.1.1.{}", Uuid::new_v4().as_u128());
    let series_uid = format!("1.2.840.113619.2.1.2.{}.1", Uuid::new_v4().as_u128());

    let req_body = AssignSeriesToProjectRequest {
        study_uid: study_uid.clone(),
        series_uid: series_uid.clone(),
        series_description: Some("Test Series".to_string()),
        modality: Some("CT".to_string()),
        series_number: Some(1),
    };

    let req = test::TestRequest::post()
        .uri(&format!("/api/projects/{}/series/assign", project_id))
        .set_json(&req_body)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "First assignment should succeed");
    println!("✅ Step 2: Series assigned successfully");

    // Step 3: 중복 할당 시도 → 409
    let req = test::TestRequest::post()
        .uri(&format!("/api/projects/{}/series/assign", project_id))
        .set_json(&req_body)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 409, "Should return 409 for duplicate assignment");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(
        body["error"]
            .as_str()
            .unwrap()
            .to_lowercase()
            .contains("already"),
        "Error message should indicate duplicate"
    );
    println!("✅ Step 3: 409 error handled correctly");

    // Cleanup
    cleanup_test_data(&pool, project_id).await;
    println!("✅ Cleaned up test data");

    println!("\n🎉 E2E Test Complete: Error handling verified!");
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
        .unwrap_or_else(|_| {
            "postgres://pacs_extension_admin:PacsExtension2024@localhost:5456/pacs_extension"
                .to_string()
        });

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    // 서비스 및 UseCase 설정
    let user_repo = UserRepositoryImpl::new(pool.clone());
    let project_repo = ProjectRepositoryImpl::new(pool.clone());
    let role_repo = RoleRepositoryImpl::new(pool.clone());
    let project_data_repo = Arc::new(ProjectDataRepositoryImpl::new(pool.clone()));
    let project_data_access_repo = Arc::new(ProjectDataAccessRepositoryImpl::new(pool.clone()));

    let project_service = Arc::new(ProjectServiceImpl::new(project_repo, user_repo, role_repo));
    let project_data_service = ProjectDataServiceImpl::new(
        project_data_repo.clone(),
        project_data_access_repo.clone(),
    );

    // ProjectDataService trait object로 변환
    let project_data_service_arc: Arc<dyn pacs_server::domain::services::ProjectDataService> =
        Arc::new(project_data_service);

    let use_case = Arc::new(ProjectDataAccessUseCase::new(
        project_data_service_arc.clone(),
        project_service.clone(),
    ));

    // 앱 설정
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(use_case.clone()))
            .service(web::scope("/api").configure(|cfg| {
                project_data_access_controller::configure_routes(cfg, use_case.clone())
            })),
    )
    .await;

    (app, pool)
}

/// 테스트 프로젝트 생성
async fn create_test_project(pool: &sqlx::PgPool) -> i32 {
    let project_name = format!("E2E Test Project {}", Uuid::new_v4());
    let description = "E2E test project for series assignment";

    let rec: (i32,) = sqlx::query_as(
        "INSERT INTO security_project (name, description, sponsor, start_date, end_date, status)
         VALUES ($1, $2, 'Test Sponsor', CURRENT_DATE, CURRENT_DATE + INTERVAL '1 year', 'PREPARING'::project_status)
         RETURNING id",
    )
    .bind(&project_name)
    .bind(description)
    .fetch_one(pool)
    .await
    .expect("Failed to create test project");

    rec.0
}

/// 테스트 데이터 정리
async fn cleanup_test_data(pool: &sqlx::PgPool, project_id: i32) {
    // project_data 삭제
    let _ = sqlx::query("DELETE FROM project_data WHERE project_id = $1")
        .bind(project_id)
        .execute(pool)
        .await;

    // 프로젝트 삭제
    let _ = sqlx::query("DELETE FROM security_project WHERE id = $1")
        .bind(project_id)
        .execute(pool)
        .await;

    // 고아 Study/Series 정리
    let _ = sqlx::query(
        "DELETE FROM project_data_study WHERE id NOT IN (SELECT DISTINCT study_id FROM project_data WHERE study_id IS NOT NULL)"
    )
    .execute(pool)
    .await;

    let _ = sqlx::query(
        "DELETE FROM project_data_series WHERE id NOT IN (SELECT DISTINCT series_id FROM project_data WHERE series_id IS NOT NULL)"
    )
    .execute(pool)
    .await;
}

