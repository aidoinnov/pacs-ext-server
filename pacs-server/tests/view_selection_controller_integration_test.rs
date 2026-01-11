use actix_web::{test, web, App};
use pacs_server::application::dto::view_selection_dto::{CreateViewSelectionRequest, SelectedSeriesDto};
use pacs_server::application::use_cases::ViewSelectionUseCase;
use pacs_server::domain::view_selection::services::ViewSelectionService;
use pacs_server::infrastructure::auth::JwtService;
use pacs_server::infrastructure::redis::RedisClientFactory;
use pacs_server::infrastructure::repositories::{ProjectDataRepositoryImpl, UserRepositoryImpl};
use pacs_server::infrastructure::services::DicomRbacEvaluatorImpl;
use pacs_server::infrastructure::view_selection::{ViewSelectionRepositoryImpl, ViewSelectionServiceImpl};
use pacs_server::presentation::controllers::view_selection_controller;
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

async fn get_redis_connection() -> Option<Arc<pacs_server::infrastructure::redis::RedisConnection>> {
    let redis_url = std::env::var("APP_REDIS__URL")
        .or_else(|_| std::env::var("REDIS_URL"))
        .unwrap_or_else(|_| "redis://localhost:6379".to_string());

    match RedisClientFactory::create(&redis_url).await {
        Ok(conn) => Some(Arc::new(conn)),
        Err(e) => {
            eprintln!("⚠️  Redis connection failed: {} - Skipping integration tests", e);
            None
        }
    }
}

async fn setup_test_user(pool: &PgPool) -> i32 {
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

    user_id
}

async fn cleanup_test_user(pool: &PgPool, user_id: i32) {
    sqlx::query("DELETE FROM security_user WHERE id = $1")
        .bind(user_id)
        .execute(pool)
        .await
        .ok();
}

async fn setup_test_app() -> Option<(
    impl actix_web::dev::ServiceFactory<
        actix_web::dev::ServiceRequest,
        Config = (),
        Error = actix_web::Error,
        InitError = (),
    >,
    PgPool,
    i32,
)> {
    let pool = get_test_pool().await;
    let user_id = setup_test_user(&pool).await;

    let redis_conn = match get_redis_connection().await {
        Some(conn) => conn,
        None => return None,
    };

    let view_selection_repo = Arc::new(ViewSelectionRepositoryImpl::new(
        redis_conn,
        Some("test_view_selection:".to_string()),
    ));

    let view_selection_service = Arc::new(ViewSelectionServiceImpl::new(
        view_selection_repo.clone(),
        1800,
    ));

    let use_case = Arc::new(ViewSelectionUseCase::new(
        view_selection_service,
        1800,
    ));

    let jwt_service = Arc::new(JwtService::new(&pacs_server::infrastructure::config::JwtConfig {
        secret: "test_secret_key_for_testing_purposes_only".to_string(),
        expiration_hours: 24,
    }));

    let user_repo = Arc::new(UserRepositoryImpl::new(pool.clone()));
    let project_data_repo = Arc::new(ProjectDataRepositoryImpl::new(pool.clone()));
    let dicom_evaluator = Arc::new(DicomRbacEvaluatorImpl::new(pool.clone()));

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(use_case.clone()))
            .app_data(web::Data::new(jwt_service.clone()))
            .app_data(web::Data::new(Arc::new(user_repo.clone())))
            .app_data(web::Data::new(project_data_repo.clone()))
            .app_data(web::Data::new(dicom_evaluator.clone()))
            .service(
                web::resource("/api/v1/view-selections")
                    .route(web::post().to(
                        view_selection_controller::create_view_selection::<
                            ViewSelectionServiceImpl<ViewSelectionRepositoryImpl>,
                        >,
                    )),
            )
            .service(
                web::resource("/api/v1/view-selections/{selection_id}")
                    .route(web::get().to(
                        view_selection_controller::get_view_selection::<
                            ViewSelectionServiceImpl<ViewSelectionRepositoryImpl>,
                        >,
                    ))
                    .route(web::delete().to(
                        view_selection_controller::delete_view_selection::<
                            ViewSelectionServiceImpl<ViewSelectionRepositoryImpl>,
                        >,
                    )),
            ),
    )
    .await;

    Some((app, pool, user_id))
}

/// 통합 테스트 1: ViewSelection 생성 성공
#[actix_web::test]
#[ignore]
async fn test_create_view_selection_success() {
    let Some((app, pool, user_id)) = setup_test_app().await else {
        eprintln!("Skipping test - Redis not available");
        return;
    };

    let req_body = CreateViewSelectionRequest {
        series: vec![
            SelectedSeriesDto {
                study_uid: "1.2.840.113619.2.1.1.123".to_string(),
                series_uid: "1.2.840.113619.2.1.2.124".to_string(),
            },
            SelectedSeriesDto {
                study_uid: "1.2.840.113619.2.1.1.125".to_string(),
                series_uid: "1.2.840.113619.2.1.2.126".to_string(),
            },
        ],
    };

    let req = test::TestRequest::post()
        .uri("/api/v1/view-selections")
        .set_json(&req_body)
        .insert_header(("X-User-ID", user_id.to_string()))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 201, "Selection creation should succeed");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["selection_id"].is_string());
    assert!(body["selection_id"].as_str().unwrap().starts_with("sel_"));

    cleanup_test_user(&pool, user_id).await;
}

/// 통합 테스트 2: ViewSelection 조회 성공
#[actix_web::test]
#[ignore]
async fn test_get_view_selection_success() {
    let Some((app, pool, user_id)) = setup_test_app().await else {
        eprintln!("Skipping test - Redis not available");
        return;
    };

    // Given: Selection 생성
    let req_body = CreateViewSelectionRequest {
        series: vec![SelectedSeriesDto {
            study_uid: "1.2.840.113619.2.1.1.123".to_string(),
            series_uid: "1.2.840.113619.2.1.2.124".to_string(),
        }],
    };

    let create_req = test::TestRequest::post()
        .uri("/api/v1/view-selections")
        .set_json(&req_body)
        .insert_header(("X-User-ID", user_id.to_string()))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), 201);

    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let selection_id = create_body["selection_id"].as_str().unwrap();

    // When: Selection 조회
    let get_req = test::TestRequest::get()
        .uri(&format!("/api/v1/view-selections/{}", selection_id))
        .insert_header(("X-User-ID", user_id.to_string()))
        .to_request();

    let get_resp = test::call_service(&app, get_req).await;

    // Then: 성공 응답 확인
    assert_eq!(get_resp.status(), 200, "Selection retrieval should succeed");

    let body: serde_json::Value = test::read_body_json(get_resp).await;
    assert_eq!(body["selection_id"].as_str().unwrap(), selection_id);
    assert_eq!(body["series"].as_array().unwrap().len(), 1);
    assert_eq!(body["user_id"].as_i64().unwrap(), user_id as i64);

    cleanup_test_user(&pool, user_id).await;
}

/// 통합 테스트 3: ViewSelection 조회 실패 (존재하지 않음)
#[actix_web::test]
#[ignore]
async fn test_get_view_selection_not_found() {
    let Some((app, pool, user_id)) = setup_test_app().await else {
        eprintln!("Skipping test - Redis not available");
        return;
    };

    let req = test::TestRequest::get()
        .uri("/api/v1/view-selections/sel_nonexistent")
        .insert_header(("X-User-ID", user_id.to_string()))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 404, "Should return 404 for non-existent selection");

    cleanup_test_user(&pool, user_id).await;
}

/// 통합 테스트 4: ViewSelection 삭제 성공
#[actix_web::test]
#[ignore]
async fn test_delete_view_selection_success() {
    let Some((app, pool, user_id)) = setup_test_app().await else {
        eprintln!("Skipping test - Redis not available");
        return;
    };

    // Given: Selection 생성
    let req_body = CreateViewSelectionRequest {
        series: vec![SelectedSeriesDto {
            study_uid: "1.2.840.113619.2.1.1.123".to_string(),
            series_uid: "1.2.840.113619.2.1.2.124".to_string(),
        }],
    };

    let create_req = test::TestRequest::post()
        .uri("/api/v1/view-selections")
        .set_json(&req_body)
        .insert_header(("X-User-ID", user_id.to_string()))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    let create_body: serde_json::Value = test::read_body_json(create_resp).await;
    let selection_id = create_body["selection_id"].as_str().unwrap();

    // When: Selection 삭제
    let delete_req = test::TestRequest::delete()
        .uri(&format!("/api/v1/view-selections/{}", selection_id))
        .insert_header(("X-User-ID", user_id.to_string()))
        .to_request();

    let delete_resp = test::call_service(&app, delete_req).await;

    // Then: 성공 응답 확인
    assert_eq!(delete_resp.status(), 204, "Selection deletion should succeed");

    // 삭제 후 조회 시 404가 나와야 함
    let get_req = test::TestRequest::get()
        .uri(&format!("/api/v1/view-selections/{}", selection_id))
        .insert_header(("X-User-ID", user_id.to_string()))
        .to_request();

    let get_resp = test::call_service(&app, get_req).await;
    assert_eq!(get_resp.status(), 404, "Deleted selection should not be found");

    cleanup_test_user(&pool, user_id).await;
}

/// 통합 테스트 5: 빈 Series 목록으로 생성 시도 (실패)
#[actix_web::test]
#[ignore]
async fn test_create_view_selection_empty_series() {
    let Some((app, pool, user_id)) = setup_test_app().await else {
        eprintln!("Skipping test - Redis not available");
        return;
    };

    let req_body = CreateViewSelectionRequest {
        series: vec![],
    };

    let req = test::TestRequest::post()
        .uri("/api/v1/view-selections")
        .set_json(&req_body)
        .insert_header(("X-User-ID", user_id.to_string()))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 400, "Should return 400 for empty series list");

    cleanup_test_user(&pool, user_id).await;
}


