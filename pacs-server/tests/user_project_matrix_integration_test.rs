use actix_web::{test, web, App};
use pacs_server::application::use_cases::user_project_matrix_use_case::UserProjectMatrixUseCase;
use pacs_server::domain::services::{ProjectServiceImpl, UserServiceImpl};
use pacs_server::infrastructure::repositories::{ProjectRepositoryImpl, UserRepositoryImpl, RoleRepositoryImpl};
use pacs_server::presentation::controllers::user_project_matrix_controller;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use uuid::Uuid;

/// 통합 테스트: 전체 사용자 × 전체 프로젝트 매트릭스 조회
/// 
/// 조건1: 전체 사용자 × 전체 프로젝트
/// 조건2: 역할 값이 있어야 함 (테스트 데이터에 역할이 할당되어 있음)
#[actix_web::test]
#[ignore]
async fn test_user_project_matrix_full_integration() {
    // Given: 데이터베이스 연결 및 테스트 데이터 준비
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL")
        .or_else(|_| std::env::var("APP_DATABASE_URL"))
        .expect("DATABASE_URL or APP_DATABASE_URL not set for test");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    // 테스트 데이터 생성
    let (user_ids, project_ids) = setup_test_data(&pool).await;

    // 서비스 및 UseCase 설정
    let user_repo1 = UserRepositoryImpl::new(pool.clone());
    let user_repo2 = UserRepositoryImpl::new(pool.clone());
    let project_repo1 = ProjectRepositoryImpl::new(pool.clone());
    let project_repo2 = ProjectRepositoryImpl::new(pool.clone());
    let role_repo = RoleRepositoryImpl::new(pool.clone());

    let user_service = Arc::new(UserServiceImpl::new(user_repo1, project_repo1));
    let project_service = Arc::new(ProjectServiceImpl::new(project_repo2, user_repo2, role_repo));

    let use_case = Arc::new(UserProjectMatrixUseCase::new(
        user_service,
        project_service,
    ));

    // 앱 설정
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(use_case.clone()))
            .service(
                web::scope("/api")
                    .configure(|cfg| {
                        user_project_matrix_controller::configure_routes(cfg, use_case.clone())
                    })
            )
    )
    .await;

    // When: 전체 매트릭스 API 호출 (조건1: 전체 사용자 × 전체 프로젝트)
    let req = test::TestRequest::get()
        .uri("/api/user-project-matrix?user_page=1&user_page_size=10&project_page=1&project_page_size=10")
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Then: 성공 응답 확인
    assert!(resp.status().is_success(), "API should return success");

    let body: serde_json::Value = test::read_body_json(resp).await;

    // 매트릭스 구조 검증
    assert!(body["matrix"].is_array(), "matrix should be an array");
    assert!(body["projects"].is_array(), "projects should be an array");
    assert!(body["pagination"].is_object(), "pagination should be an object");

    let matrix = body["matrix"].as_array().unwrap();
    let projects = body["projects"].as_array().unwrap();

    // 조건1 검증: 전체 사용자 × 전체 프로젝트
    assert!(matrix.len() > 0, "matrix should contain users");
    assert!(projects.len() > 0, "projects should be present");

    // 조건2 검증: 역할 값이 있어야 함
    let mut has_role_assigned = false;
    for user_row in matrix {
        let project_roles = user_row["project_roles"].as_array().unwrap();
        for cell in project_roles {
            if !cell["role_name"].is_null() {
                has_role_assigned = true;
                
                // 역할 정보 검증
                assert!(cell["role_id"].is_number(), "role_id should be a number when role is assigned");
                assert!(cell["role_name"].is_string(), "role_name should be a string when role is assigned");
                assert!(cell["project_id"].is_number(), "project_id should be present");
                assert!(cell["project_name"].is_string(), "project_name should be present");
            }
        }
    }

    assert!(has_role_assigned, "조건2: At least one user should have a role assigned to a project");

    // 페이지네이션 정보 검증
    assert!(body["pagination"]["user_total_count"].as_i64().unwrap() > 0);
    assert!(body["pagination"]["project_total_count"].as_i64().unwrap() > 0);

    // 정리
    cleanup_test_data(&pool, &user_ids, &project_ids).await;
}

/// 통합 테스트: 유저 검색 및 정렬
#[actix_web::test]
#[ignore]
async fn test_user_project_matrix_with_search_and_sort() {
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL")
        .or_else(|_| std::env::var("APP_DATABASE_URL"))
        .expect("DATABASE_URL or APP_DATABASE_URL not set for test");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    let (user_ids, project_ids) = setup_test_data(&pool).await;

    let user_repo1 = UserRepositoryImpl::new(pool.clone());
    let user_repo2 = UserRepositoryImpl::new(pool.clone());
    let project_repo1 = ProjectRepositoryImpl::new(pool.clone());
    let project_repo2 = ProjectRepositoryImpl::new(pool.clone());
    let role_repo = RoleRepositoryImpl::new(pool.clone());

    let user_service = Arc::new(UserServiceImpl::new(user_repo1, project_repo1));
    let project_service = Arc::new(ProjectServiceImpl::new(project_repo2, user_repo2, role_repo));

    let use_case = Arc::new(UserProjectMatrixUseCase::new(
        user_service,
        project_service,
    ));

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(use_case.clone()))
            .service(
                web::scope("/api")
                    .configure(|cfg| {
                        user_project_matrix_controller::configure_routes(cfg, use_case.clone())
                    })
            )
    )
    .await;

    // When: 검색 및 정렬 파라미터와 함께 호출
    let req = test::TestRequest::get()
        .uri("/api/user-project-matrix?user_search=test&user_sort_by=username&user_sort_order=asc")
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Then: 성공 응답 확인
    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["matrix"].is_array());

    cleanup_test_data(&pool, &user_ids, &project_ids).await;
}

/// 통합 테스트: 페이지네이션
#[actix_web::test]
#[ignore]
async fn test_user_project_matrix_pagination() {
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL")
        .or_else(|_| std::env::var("APP_DATABASE_URL"))
        .expect("DATABASE_URL or APP_DATABASE_URL not set for test");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    let (user_ids, project_ids) = setup_test_data(&pool).await;

    let user_repo1 = UserRepositoryImpl::new(pool.clone());
    let user_repo2 = UserRepositoryImpl::new(pool.clone());
    let project_repo1 = ProjectRepositoryImpl::new(pool.clone());
    let project_repo2 = ProjectRepositoryImpl::new(pool.clone());
    let role_repo = RoleRepositoryImpl::new(pool.clone());

    let user_service = Arc::new(UserServiceImpl::new(user_repo1, project_repo1));
    let project_service = Arc::new(ProjectServiceImpl::new(project_repo2, user_repo2, role_repo));

    let use_case = Arc::new(UserProjectMatrixUseCase::new(
        user_service,
        project_service,
    ));

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(use_case.clone()))
            .service(
                web::scope("/api")
                    .configure(|cfg| {
                        user_project_matrix_controller::configure_routes(cfg, use_case.clone())
                    })
            )
    )
    .await;

    // When: 페이지 크기 제한 테스트
    let req = test::TestRequest::get()
        .uri("/api/user-project-matrix?user_page=1&user_page_size=2&project_page=1&project_page_size=2")
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Then: 페이지네이션 정보 검증
    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    
    let matrix = body["matrix"].as_array().unwrap();
    assert!(matrix.len() <= 2, "user_page_size should limit results");

    let projects = body["projects"].as_array().unwrap();
    assert!(projects.len() <= 2, "project_page_size should limit results");

    assert_eq!(body["pagination"]["user_page"].as_i64().unwrap(), 1);
    assert_eq!(body["pagination"]["user_page_size"].as_i64().unwrap(), 2);
    assert_eq!(body["pagination"]["project_page"].as_i64().unwrap(), 1);
    assert_eq!(body["pagination"]["project_page_size"].as_i64().unwrap(), 2);

    cleanup_test_data(&pool, &user_ids, &project_ids).await;
}

/// 테스트 데이터 생성
async fn setup_test_data(pool: &sqlx::PgPool) -> (Vec<i32>, Vec<i32>) {
    let mut user_ids = Vec::new();
    let mut project_ids = Vec::new();

    // 테스트 유저 3명 생성
    for i in 1..=3 {
        let keycloak_id = Uuid::new_v4();
        let username = format!("test_user_matrix_{}_{}", i, Uuid::new_v4());
        let email = format!("{}@example.com", username);

        let rec: (i32,) = sqlx::query_as(
            "INSERT INTO security_user (keycloak_id, username, email) VALUES ($1, $2, $3) RETURNING id",
        )
        .bind(keycloak_id)
        .bind(&username)
        .bind(&email)
        .fetch_one(pool)
        .await
        .unwrap();

        user_ids.push(rec.0);
    }

    // 테스트 프로젝트 3개 생성
    for i in 1..=3 {
        let project_name = format!("Test Project Matrix {} {}", i, Uuid::new_v4());
        let description = format!("Test project for matrix integration test {} {}", i, Uuid::new_v4());

        let rec: (i32,) = sqlx::query_as(
            "INSERT INTO security_project (name, description, status) VALUES ($1, $2, 'IN_PROGRESS') RETURNING id",
        )
        .bind(&project_name)
        .bind(&description)
        .fetch_one(pool)
        .await
        .unwrap();

        project_ids.push(rec.0);
    }

    // 역할 조회 (조건2: 역할 값이 있어야 함)
    let role_id: (i32,) = match sqlx::query_as::<_, (i32,)>(
        "SELECT id FROM security_role WHERE name = 'PROJECT_ADMIN' LIMIT 1"
    )
    .fetch_one(pool)
    .await
    {
        Ok(id) => id,
        Err(_) => {
            // 역할이 없으면 생성
            sqlx::query_as::<_, (i32,)>(
                "INSERT INTO security_role (name, description) VALUES ('PROJECT_ADMIN', 'Project Administrator') RETURNING id"
            )
            .fetch_one(pool)
            .await
            .unwrap()
        }
    };

    // 유저-프로젝트 역할 할당 (조건2: 역할 값이 있어야 함)
    for user_id in &user_ids {
        for project_id in &project_ids {
            let _ = sqlx::query(
                "INSERT INTO security_user_project (user_id, project_id, role_id) VALUES ($1, $2, $3)
                 ON CONFLICT (user_id, project_id) DO NOTHING"
            )
            .bind(user_id)
            .bind(project_id)
            .bind(role_id.0)
            .execute(pool)
            .await;
        }
    }

    (user_ids, project_ids)
}

/// 테스트 데이터 정리
async fn cleanup_test_data(pool: &sqlx::PgPool, user_ids: &[i32], project_ids: &[i32]) {
    // 유저-프로젝트 관계 삭제
    for user_id in user_ids {
        let _ = sqlx::query("DELETE FROM security_user_project WHERE user_id = $1")
            .bind(user_id)
            .execute(pool)
            .await;
    }

    // 유저 삭제
    for user_id in user_ids {
        let _ = sqlx::query("DELETE FROM security_user WHERE id = $1")
            .bind(user_id)
            .execute(pool)
            .await;
    }

    // 프로젝트 삭제
    for project_id in project_ids {
        let _ = sqlx::query("DELETE FROM security_project WHERE id = $1")
            .bind(project_id)
            .execute(pool)
            .await;
    }
}

