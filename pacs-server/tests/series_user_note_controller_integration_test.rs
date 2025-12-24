use actix_web::{test, web, App};
use pacs_server::application::dto::series_user_note_dto::CreateOrUpdateSeriesNoteRequest;
use pacs_server::application::use_cases::SeriesUserNoteUseCase;
use pacs_server::domain::services::{SeriesUserNoteService, SeriesUserNoteServiceImpl};
use pacs_server::infrastructure::auth::JwtService;
use pacs_server::infrastructure::repositories::{
    ProjectDataRepositoryImpl, ProjectRepositoryImpl, SeriesUserNoteRepositoryImpl,
    UserRepositoryImpl,
};
use pacs_server::presentation::controllers::series_user_note_controller;
use sqlx::PgPool;
use std::sync::Arc;

async fn get_test_pool() -> PgPool {
    let database_url = std::env::var("APP_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .unwrap_or_else(|_| {
            "postgres://pacs_extension_admin:PacsExtension2024@localhost:5456/pacs_extension"
                .to_string()
        });

    PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

async fn setup_test_data(pool: &PgPool) -> (i32, i32, i32) {
    // 1. 사용자 생성
    let user_id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO security_user (username, email, password_hash, is_active, full_name)
         VALUES ($1, $2, 'hashed_password', true, $3)
         RETURNING id",
    )
    .bind(format!("test_user_{}", uuid::Uuid::new_v4()))
    .bind(format!("test_{}@example.com", uuid::Uuid::new_v4()))
    .bind("테스트 사용자".to_string())
    .fetch_one(pool)
    .await
    .expect("Failed to create test user");

    // 2. 프로젝트 생성
    let project_id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO security_project (project_name, description, status, is_active)
         VALUES ($1, 'Test Project', 'ACTIVE', true)
         RETURNING id",
    )
    .bind(format!("test_project_{}", uuid::Uuid::new_v4()))
    .fetch_one(pool)
    .await
    .expect("Failed to create test project");

    // 3. 사용자를 프로젝트 멤버로 추가
    sqlx::query(
        "INSERT INTO security_user_project (user_id, project_id)
         VALUES ($1, $2)",
    )
    .bind(user_id)
    .bind(project_id)
    .execute(pool)
    .await
    .expect("Failed to add user to project");

    // 4. Study 생성
    let study_id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO project_data_study (study_uid, study_description)
         VALUES ($1, 'Test Study')
         RETURNING id",
    )
    .bind(format!("1.2.840.113619.2.1.1.{}", uuid::Uuid::new_v4()))
    .fetch_one(pool)
    .await
    .expect("Failed to create test study");

    // 5. Series 생성
    let series_id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO project_data_series (study_id, series_uid, series_description, modality)
         VALUES ($1, $2, 'Test Series', 'CT')
         RETURNING id",
    )
    .bind(study_id)
    .bind(format!("1.2.840.113619.2.1.2.{}", uuid::Uuid::new_v4()))
    .fetch_one(pool)
    .await
    .expect("Failed to create test series");

    (user_id, project_id, series_id)
}

async fn cleanup_test_data(pool: &PgPool, user_id: i32, project_id: i32, series_id: i32) {
    sqlx::query("DELETE FROM series_user_note WHERE series_id = $1")
        .bind(series_id)
        .execute(pool)
        .await
        .ok();
    sqlx::query("DELETE FROM project_data_series WHERE id = $1")
        .bind(series_id)
        .execute(pool)
        .await
        .ok();
    sqlx::query("DELETE FROM project_data_study WHERE id IN (SELECT study_id FROM project_data_series WHERE id = $1)")
        .bind(series_id)
        .execute(pool)
        .await
        .ok();
    sqlx::query("DELETE FROM security_user_project WHERE user_id = $1 AND project_id = $2")
        .bind(user_id)
        .bind(project_id)
        .execute(pool)
        .await
        .ok();
    sqlx::query("DELETE FROM security_user WHERE id = $1")
        .bind(user_id)
        .execute(pool)
        .await
        .ok();
    sqlx::query("DELETE FROM security_project WHERE id = $1")
        .bind(project_id)
        .execute(pool)
        .await
        .ok();
}

async fn setup_test_app() -> (impl actix_web::dev::ServiceFactory<actix_web::dev::ServiceRequest, Config = (), Error = actix_web::Error, InitError = ()>, PgPool) {
    let pool = get_test_pool().await;

    let note_repo = SeriesUserNoteRepositoryImpl::new(pool.clone());
    let user_repo = UserRepositoryImpl::new(pool.clone());
    let project_repo = ProjectRepositoryImpl::new(pool.clone());
    let project_data_repo = Arc::new(ProjectDataRepositoryImpl::new(pool.clone()));

    let note_service = Arc::new(SeriesUserNoteServiceImpl::new(
        note_repo,
        user_repo.clone(),
        project_repo,
        project_data_repo,
    ));

    let use_case = Arc::new(SeriesUserNoteUseCase::new(
        note_service,
        Arc::new(user_repo.clone()),
    ));

    let jwt_service = Arc::new(JwtService::new(&pacs_server::infrastructure::config::JwtConfig {
        secret: "test_secret_key_for_testing_purposes_only".to_string(),
        expiration_hours: 24,
    }));

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(use_case.clone()))
            .app_data(web::Data::new(jwt_service.clone()))
            .app_data(web::Data::new(Arc::new(user_repo.clone())))
            .configure(|cfg| {
                series_user_note_controller::configure_routes(
                    cfg,
                    use_case,
                    jwt_service,
                    Arc::new(user_repo),
                )
            }),
    )
    .await;

    (app, pool)
}

/// 통합 테스트 1: 프로젝트 종속 Note 생성 성공
#[actix_web::test]
#[ignore]
async fn test_create_project_note_success() {
    let (app, pool) = setup_test_app().await;
    let (user_id, project_id, series_id) = setup_test_data(&pool).await;

    // When: 프로젝트 종속 Note 생성 요청
    let req_body = CreateOrUpdateSeriesNoteRequest {
        note: "프로젝트 메모입니다".to_string(),
    };

    let req = test::TestRequest::put()
        .uri(&format!(
            "/api/project-data/{}/series/{}/note",
            project_id, series_id
        ))
        .set_json(&req_body)
        .insert_header(("X-User-ID", user_id.to_string()))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Then: 성공 응답 확인
    assert_eq!(resp.status(), 200, "Note creation should succeed");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["success"].as_bool().unwrap(), true);
    assert!(body["note"].is_object());
    assert_eq!(
        body["note"]["note"].as_str().unwrap(),
        "프로젝트 메모입니다"
    );

    cleanup_test_data(&pool, user_id, project_id, series_id).await;
}

/// 통합 테스트 2: 프로젝트 종속 Note 조회 성공
#[actix_web::test]
#[ignore]
async fn test_get_project_note_success() {
    let (app, pool) = setup_test_app().await;
    let (user_id, project_id, series_id) = setup_test_data(&pool).await;

    // Given: Note 생성
    let note_repo = SeriesUserNoteRepositoryImpl::new(pool.clone());
    note_repo
        .create_or_update(series_id, user_id, Some(project_id), "조회할 메모".to_string())
        .await
        .unwrap();

    // When: Note 조회 요청
    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/project-data/{}/series/{}/note",
            project_id, series_id
        ))
        .insert_header(("X-User-ID", user_id.to_string()))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Then: 성공 응답 확인
    assert_eq!(resp.status(), 200, "Note retrieval should succeed");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["success"].as_bool().unwrap(), true);
    assert!(body["note"].is_object());
    assert_eq!(body["note"]["note"].as_str().unwrap(), "조회할 메모");

    cleanup_test_data(&pool, user_id, project_id, series_id).await;
}

/// 통합 테스트 3: 프로젝트 종속 Note 목록 조회
#[actix_web::test]
#[ignore]
async fn test_get_project_notes_list() {
    let (app, pool) = setup_test_app().await;
    let (user_id, project_id, series_id) = setup_test_data(&pool).await;

    // Given: 여러 Note 생성
    let note_repo = SeriesUserNoteRepositoryImpl::new(pool.clone());
    note_repo
        .create_or_update(series_id, user_id, Some(project_id), "첫 번째 메모".to_string())
        .await
        .unwrap();

    let user2_id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO security_user (username, email, password_hash, is_active)
         VALUES ($1, $2, 'hashed_password', true)
         RETURNING id",
    )
    .bind(format!("test_user2_{}", uuid::Uuid::new_v4()))
    .bind(format!("test2_{}@example.com", uuid::Uuid::new_v4()))
    .fetch_one(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO security_user_project (user_id, project_id)
         VALUES ($1, $2)",
    )
    .bind(user2_id)
    .bind(project_id)
    .execute(&pool)
    .await
    .unwrap();

    note_repo
        .create_or_update(series_id, user2_id, Some(project_id), "두 번째 메모".to_string())
        .await
        .unwrap();

    // When: Note 목록 조회 요청
    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/project-data/{}/series/{}/notes",
            project_id, series_id
        ))
        .insert_header(("X-User-ID", user_id.to_string()))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Then: 성공 응답 및 목록 확인
    assert_eq!(resp.status(), 200, "Notes list retrieval should succeed");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["success"].as_bool().unwrap(), true);
    assert!(body["notes"].is_array());
    assert_eq!(body["notes"].as_array().unwrap().len(), 2);

    cleanup_test_data(&pool, user_id, project_id, series_id).await;
    sqlx::query("DELETE FROM security_user_project WHERE user_id = $1 AND project_id = $2")
        .bind(user2_id)
        .bind(project_id)
        .execute(&pool)
        .await
        .ok();
    sqlx::query("DELETE FROM security_user WHERE id = $1")
        .bind(user2_id)
        .execute(&pool)
        .await
        .ok();
}

/// 통합 테스트 4: 프로젝트 종속 Note 삭제
#[actix_web::test]
#[ignore]
async fn test_delete_project_note() {
    let (app, pool) = setup_test_app().await;
    let (user_id, project_id, series_id) = setup_test_data(&pool).await;

    // Given: Note 생성
    let note_repo = SeriesUserNoteRepositoryImpl::new(pool.clone());
    note_repo
        .create_or_update(series_id, user_id, Some(project_id), "삭제될 메모".to_string())
        .await
        .unwrap();

    // When: Note 삭제 요청
    let req = test::TestRequest::delete()
        .uri(&format!(
            "/api/project-data/{}/series/{}/note",
            project_id, series_id
        ))
        .insert_header(("X-User-ID", user_id.to_string()))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Then: 성공 응답 확인
    assert_eq!(resp.status(), 200, "Note deletion should succeed");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["success"].as_bool().unwrap(), true);

    cleanup_test_data(&pool, user_id, project_id, series_id).await;
}

/// 통합 테스트 5: 전역 Note 생성 성공
#[actix_web::test]
#[ignore]
async fn test_create_global_note_success() {
    let (app, pool) = setup_test_app().await;
    let (user_id, _project_id, series_id) = setup_test_data(&pool).await;

    // When: 전역 Note 생성 요청
    let req_body = CreateOrUpdateSeriesNoteRequest {
        note: "전역 메모입니다".to_string(),
    };

    let req = test::TestRequest::put()
        .uri(&format!("/api/series/{}/note", series_id))
        .set_json(&req_body)
        .insert_header(("X-User-ID", user_id.to_string()))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Then: 성공 응답 확인
    assert_eq!(resp.status(), 200, "Global note creation should succeed");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["success"].as_bool().unwrap(), true);
    assert!(body["note"].is_object());
    assert_eq!(body["note"]["note"].as_str().unwrap(), "전역 메모입니다");
    assert!(body["note"]["project_id"].is_null());

    cleanup_test_data(&pool, user_id, 0, series_id).await;
}

/// 통합 테스트 6: 전역 Note 조회 성공
#[actix_web::test]
#[ignore]
async fn test_get_global_note_success() {
    let (app, pool) = setup_test_app().await;
    let (user_id, _project_id, series_id) = setup_test_data(&pool).await;

    // Given: 전역 Note 생성
    let note_repo = SeriesUserNoteRepositoryImpl::new(pool.clone());
    note_repo
        .create_or_update(series_id, user_id, None, "전역 조회 메모".to_string())
        .await
        .unwrap();

    // When: 전역 Note 조회 요청
    let req = test::TestRequest::get()
        .uri(&format!("/api/series/{}/note", series_id))
        .insert_header(("X-User-ID", user_id.to_string()))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Then: 성공 응답 확인
    assert_eq!(resp.status(), 200, "Global note retrieval should succeed");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["success"].as_bool().unwrap(), true);
    assert!(body["note"].is_object());
    assert_eq!(body["note"]["note"].as_str().unwrap(), "전역 조회 메모");

    cleanup_test_data(&pool, user_id, 0, series_id).await;
}

/// 통합 테스트 7: 존재하지 않는 Note 조회 시 404
#[actix_web::test]
#[ignore]
async fn test_get_nonexistent_note_returns_404() {
    let (app, pool) = setup_test_app().await;
    let (user_id, project_id, series_id) = setup_test_data(&pool).await;

    // When: 존재하지 않는 Note 조회 요청
    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/project-data/{}/series/{}/note",
            project_id, series_id
        ))
        .insert_header(("X-User-ID", user_id.to_string()))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Then: 200 OK (Note가 없어도 success: true, note: null 반환)
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["success"].as_bool().unwrap(), true);
    assert!(body["note"].is_null());

    cleanup_test_data(&pool, user_id, project_id, series_id).await;
}

/// 통합 테스트 8: 프로젝트 멤버가 아닌 사용자로 Note 생성 시 403
#[actix_web::test]
#[ignore]
async fn test_create_note_with_non_member_returns_403() {
    let (app, pool) = setup_test_app().await;
    let (_user_id, project_id, series_id) = setup_test_data(&pool).await;

    // Given: 프로젝트 멤버가 아닌 사용자 생성
    let non_member_user_id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO security_user (username, email, password_hash, is_active)
         VALUES ($1, $2, 'hashed_password', true)
         RETURNING id",
    )
    .bind(format!("non_member_{}", uuid::Uuid::new_v4()))
    .bind(format!("non_member_{}@example.com", uuid::Uuid::new_v4()))
    .fetch_one(&pool)
    .await
    .unwrap();

    // When: 프로젝트 멤버가 아닌 사용자로 Note 생성 시도
    let req_body = CreateOrUpdateSeriesNoteRequest {
        note: "메모".to_string(),
    };

    let req = test::TestRequest::put()
        .uri(&format!(
            "/api/project-data/{}/series/{}/note",
            project_id, series_id
        ))
        .set_json(&req_body)
        .header("X-User-ID", non_member_user_id.to_string())
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Then: 401 또는 403 에러 반환
    assert!(
        resp.status() == 401 || resp.status() == 403,
        "Should return 401 or 403 for non-member"
    );

    sqlx::query("DELETE FROM security_user WHERE id = $1")
        .bind(non_member_user_id)
        .execute(&pool)
        .await
        .ok();
}

/// 통합 테스트 9: Note 업데이트 (PUT으로 동일한 Note 수정)
#[actix_web::test]
#[ignore]
async fn test_update_note() {
    let (app, pool) = setup_test_app().await;
    let (user_id, project_id, series_id) = setup_test_data(&pool).await;

    // Given: 초기 Note 생성
    let note_repo = SeriesUserNoteRepositoryImpl::new(pool.clone());
    note_repo
        .create_or_update(series_id, user_id, Some(project_id), "초기 메모".to_string())
        .await
        .unwrap();

    // When: Note 업데이트 요청
    let req_body = CreateOrUpdateSeriesNoteRequest {
        note: "업데이트된 메모".to_string(),
    };

    let req = test::TestRequest::put()
        .uri(&format!(
            "/api/project-data/{}/series/{}/note",
            project_id, series_id
        ))
        .set_json(&req_body)
        .insert_header(("X-User-ID", user_id.to_string()))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Then: 성공 응답 및 업데이트 확인
    assert_eq!(resp.status(), 200, "Note update should succeed");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["note"]["note"].as_str().unwrap(), "업데이트된 메모");

    cleanup_test_data(&pool, user_id, project_id, series_id).await;
}

/// 통합 테스트 10: 프로젝트별 Note와 전역 Note 분리 확인
#[actix_web::test]
#[ignore]
async fn test_project_and_global_notes_separation() {
    let (app, pool) = setup_test_app().await;
    let (user_id, project_id, series_id) = setup_test_data(&pool).await;

    // Given: 프로젝트별 Note와 전역 Note 생성
    let note_repo = SeriesUserNoteRepositoryImpl::new(pool.clone());
    note_repo
        .create_or_update(series_id, user_id, Some(project_id), "프로젝트 메모".to_string())
        .await
        .unwrap();
    note_repo
        .create_or_update(series_id, user_id, None, "전역 메모".to_string())
        .await
        .unwrap();

    // When: 프로젝트별 Note 조회
    let req1 = test::TestRequest::get()
        .uri(&format!(
            "/api/project-data/{}/series/{}/note",
            project_id, series_id
        ))
        .insert_header(("X-User-ID", user_id.to_string()))
        .to_request();

    let resp1 = test::call_service(&app, req1).await;
    let body1: serde_json::Value = test::read_body_json(resp1).await;
    assert_eq!(body1["note"]["note"].as_str().unwrap(), "프로젝트 메모");

    // When: 전역 Note 조회
    let req2 = test::TestRequest::get()
        .uri(&format!("/api/series/{}/note", series_id))
        .insert_header(("X-User-ID", user_id.to_string()))
        .to_request();

    let resp2 = test::call_service(&app, req2).await;
    let body2: serde_json::Value = test::read_body_json(resp2).await;
    assert_eq!(body2["note"]["note"].as_str().unwrap(), "전역 메모");

    cleanup_test_data(&pool, user_id, project_id, series_id).await;
}

