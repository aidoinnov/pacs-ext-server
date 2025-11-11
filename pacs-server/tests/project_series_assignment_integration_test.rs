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
// 통합 테스트: Series 할당
// ========================================

/// 통합 테스트 1: Series를 프로젝트에 성공적으로 할당
#[actix_web::test]
#[ignore] // 실제 DB 필요
async fn test_assign_series_to_project_success() {
    // Given: 테스트 앱 및 프로젝트 설정
    let (app, pool) = setup_test_app().await;
    let project_id = create_test_project(&pool).await;

    // When: Series 할당 요청
    let study_uid = format!("1.2.840.113619.2.1.1.{}", Uuid::new_v4().as_u128());
    let series_uid = format!("1.2.840.113619.2.1.2.{}.1", Uuid::new_v4().as_u128());

    let req_body = AssignSeriesToProjectRequest {
        study_uid: study_uid.clone(),
        series_uid: series_uid.clone(),
        series_description: Some("Axial CT 5mm".to_string()),
        modality: Some("CT".to_string()),
        series_number: Some(1),
    };

    let req = test::TestRequest::post()
        .uri(&format!("/api/projects/{}/series/assign", project_id))
        .set_json(&req_body)
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Then: 성공 응답 확인
    assert_eq!(resp.status(), 200, "Series assignment should succeed");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["success"].as_bool().unwrap(), true);
    assert!(body["series_id"].as_i64().is_some());
    assert!(body["message"].as_str().unwrap().contains("assigned"));

    // 데이터베이스에서 확인
    let series_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM project_data pd
         INNER JOIN project_data_series pds ON pd.series_id = pds.id
         WHERE pd.project_id = $1 AND pds.series_uid = $2",
    )
    .bind(project_id)
    .bind(&series_uid)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(series_count, 1, "Series should be assigned to project");

    // Cleanup
    cleanup_test_data(&pool, project_id).await;
}

/// 통합 테스트 2: 중복 Series 할당 시 409 Conflict 반환
#[actix_web::test]
#[ignore]
async fn test_assign_series_duplicate_returns_409() {
    // Given: 프로젝트 및 이미 할당된 Series
    let (app, pool) = setup_test_app().await;
    let project_id = create_test_project(&pool).await;

    let study_uid = format!("1.2.840.113619.2.1.1.{}", Uuid::new_v4().as_u128());
    let series_uid = format!("1.2.840.113619.2.1.2.{}.1", Uuid::new_v4().as_u128());

    // 첫 번째 할당 (성공)
    let req_body = AssignSeriesToProjectRequest {
        study_uid: study_uid.clone(),
        series_uid: series_uid.clone(),
        series_description: Some("Axial CT 5mm".to_string()),
        modality: Some("CT".to_string()),
        series_number: Some(1),
    };

    let req1 = test::TestRequest::post()
        .uri(&format!("/api/projects/{}/series/assign", project_id))
        .set_json(&req_body)
        .to_request();

    let resp1 = test::call_service(&app, req1).await;
    assert_eq!(resp1.status(), 200, "First assignment should succeed");

    // When: 동일한 Series를 다시 할당 시도
    let req2 = test::TestRequest::post()
        .uri(&format!("/api/projects/{}/series/assign", project_id))
        .set_json(&req_body)
        .to_request();

    let resp2 = test::call_service(&app, req2).await;

    // Then: 409 Conflict 반환
    assert_eq!(
        resp2.status(),
        409,
        "Duplicate assignment should return 409 Conflict"
    );

    let body: serde_json::Value = test::read_body_json(resp2).await;
    assert!(body["error"]
        .as_str()
        .unwrap()
        .to_lowercase()
        .contains("already"));

    // Cleanup
    cleanup_test_data(&pool, project_id).await;
}

/// 통합 테스트 3: 존재하지 않는 프로젝트에 할당 시 404 반환
#[actix_web::test]
#[ignore]
async fn test_assign_series_nonexistent_project_returns_404() {
    // Given: 테스트 앱 설정
    let (app, _pool) = setup_test_app().await;
    let nonexistent_project_id = 999999;

    // When: 존재하지 않는 프로젝트에 Series 할당 시도
    let req_body = AssignSeriesToProjectRequest {
        study_uid: format!("1.2.840.113619.2.1.1.{}", Uuid::new_v4().as_u128()),
        series_uid: format!("1.2.840.113619.2.1.2.{}.1", Uuid::new_v4().as_u128()),
        series_description: Some("Axial CT 5mm".to_string()),
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

    // Then: 404 Not Found 반환
    assert_eq!(
        resp.status(),
        404,
        "Nonexistent project should return 404"
    );

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["error"]
        .as_str()
        .unwrap()
        .to_lowercase()
        .contains("not found"));
}

/// 통합 테스트 4: 할당된 Series가 프로젝트 목록에 나타나는지 확인
#[actix_web::test]
#[ignore]
async fn test_assigned_series_appears_in_project_list() {
    // Given: 프로젝트 및 Series 할당
    let (app, pool) = setup_test_app().await;
    let project_id = create_test_project(&pool).await;

    let study_uid = format!("1.2.840.113619.2.1.1.{}", Uuid::new_v4().as_u128());
    let series_uid = format!("1.2.840.113619.2.1.2.{}.1", Uuid::new_v4().as_u128());

    // Series 할당
    let req_body = AssignSeriesToProjectRequest {
        study_uid: study_uid.clone(),
        series_uid: series_uid.clone(),
        series_description: Some("Axial CT 5mm".to_string()),
        modality: Some("CT".to_string()),
        series_number: Some(1),
    };

    let req1 = test::TestRequest::post()
        .uri(&format!("/api/projects/{}/series/assign", project_id))
        .set_json(&req_body)
        .to_request();

    let resp1 = test::call_service(&app, req1).await;
    assert_eq!(resp1.status(), 200);

    // When: 프로젝트 Studies 목록 조회
    let req2 = test::TestRequest::get()
        .uri(&format!("/api/project-data/{}/studies", project_id))
        .to_request();

    let resp2 = test::call_service(&app, req2).await;

    // Then: 할당된 Study가 목록에 포함되어 있는지 확인
    assert_eq!(resp2.status(), 200);

    let response: serde_json::Value = test::read_body_json(resp2).await;
    let studies = response["studies"].as_array().expect("studies should be an array");

    assert!(
        studies.len() > 0,
        "Assigned study should appear in project studies list"
    );

    let study_uids: Vec<String> = studies
        .iter()
        .filter_map(|s| s["study_uid"].as_str().map(|s| s.to_string()))
        .collect();

    assert!(
        study_uids.contains(&study_uid),
        "Study UID should be in the list"
    );

    // Cleanup
    cleanup_test_data(&pool, project_id).await;
}

// ========================================
// 통합 테스트: Study 할당
// ========================================

/// 통합 테스트 5: Study를 프로젝트에 성공적으로 할당
#[actix_web::test]
#[ignore]
async fn test_assign_study_to_project_success() {
    // Given: 테스트 앱 및 프로젝트 설정
    let (app, pool) = setup_test_app().await;
    let project_id = create_test_project(&pool).await;

    // When: Study 할당 요청
    let study_uid = format!("1.2.840.113619.2.1.1.{}", Uuid::new_v4().as_u128());

    let req_body = AssignStudyToProjectRequest {
        study_uid: study_uid.clone(),
        study_description: Some("CT Chest with Contrast".to_string()),
        patient_id: Some("P12345".to_string()),
        patient_name: Some("John Doe".to_string()),
        study_date: Some("2024-01-15".to_string()),
        modality: Some("CT".to_string()),
    };

    let req = test::TestRequest::post()
        .uri(&format!("/api/projects/{}/studies/assign", project_id))
        .set_json(&req_body)
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Then: 성공 응답 확인
    assert_eq!(resp.status(), 200, "Study assignment should succeed");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["success"].as_bool().unwrap(), true);
    assert!(body["study_id"].as_i64().is_some());

    // 데이터베이스에서 확인
    let study_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM project_data pd
         INNER JOIN project_data_study pds ON pd.study_id = pds.id
         WHERE pd.project_id = $1 AND pds.study_uid = $2",
    )
    .bind(project_id)
    .bind(&study_uid)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(study_count, 1, "Study should be assigned to project");

    // Cleanup
    cleanup_test_data(&pool, project_id).await;
}

/// 통합 테스트 6: 중복 Study 할당 시 409 Conflict 반환
#[actix_web::test]
#[ignore]
async fn test_assign_study_duplicate_returns_409() {
    // Given: 프로젝트 및 이미 할당된 Study
    let (app, pool) = setup_test_app().await;
    let project_id = create_test_project(&pool).await;

    let study_uid = format!("1.2.840.113619.2.1.1.{}", Uuid::new_v4().as_u128());

    let req_body = AssignStudyToProjectRequest {
        study_uid: study_uid.clone(),
        study_description: Some("CT Chest".to_string()),
        patient_id: Some("P12345".to_string()),
        patient_name: Some("John Doe".to_string()),
        study_date: Some("2024-01-15".to_string()),
        modality: Some("CT".to_string()),
    };

    // 첫 번째 할당 (성공)
    let req1 = test::TestRequest::post()
        .uri(&format!("/api/projects/{}/studies/assign", project_id))
        .set_json(&req_body)
        .to_request();

    let resp1 = test::call_service(&app, req1).await;
    assert_eq!(resp1.status(), 200);

    // When: 동일한 Study를 다시 할당 시도
    let req2 = test::TestRequest::post()
        .uri(&format!("/api/projects/{}/studies/assign", project_id))
        .set_json(&req_body)
        .to_request();

    let resp2 = test::call_service(&app, req2).await;

    // Then: 409 Conflict 반환
    assert_eq!(resp2.status(), 409);

    // Cleanup
    cleanup_test_data(&pool, project_id).await;
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
    let project_name = format!("Test Project {}", Uuid::new_v4());
    let description = "Test project for series assignment";

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
    // project_data 삭제 (CASCADE로 자동 삭제되지만 명시적으로)
    let _ = sqlx::query("DELETE FROM project_data WHERE project_id = $1")
        .bind(project_id)
        .execute(pool)
        .await;

    // 프로젝트 삭제
    let _ = sqlx::query("DELETE FROM security_project WHERE id = $1")
        .bind(project_id)
        .execute(pool)
        .await;

    // 고아 Study/Series 정리 (선택적)
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
