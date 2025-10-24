use actix_web::{test, web, App};
use std::sync::Arc;
use sqlx::PgPool;

use pacs_server::{
    application::use_cases::ProjectUserMatrixUseCase,
    domain::services::{
        project_service::ProjectServiceImpl,
        user_service::UserServiceImpl,
    },
    infrastructure::repositories::{
        project_repository_impl::ProjectRepositoryImpl,
        user_repository_impl::UserRepositoryImpl,
        role_repository_impl::RoleRepositoryImpl,
    },
    presentation::controllers::project_user_matrix_controller,
};

/// 전체 매트릭스 API 통합 테스트
#[tokio::test]
async fn test_matrix_api_integration() {
    // Given: 실제 데이터베이스 연결
    let pool = setup_test_database().await;
    setup_test_data(&pool).await;
    
    // Repository, Service, Use Case 설정
    let project_repository = Arc::new(ProjectRepositoryImpl::new(pool.clone()));
    let user_repository = Arc::new(UserRepositoryImpl::new(pool.clone()));
    let role_repository = Arc::new(RoleRepositoryImpl::new(pool.clone()));
    
    let project_service = Arc::new(ProjectServiceImpl::new(
        project_repository.clone(),
        user_repository.clone(),
        role_repository.clone(),
    ));
    
    let user_service = Arc::new(UserServiceImpl::new(
        user_repository.clone(),
        project_repository.clone(),
    ));
    
    let matrix_use_case = Arc::new(ProjectUserMatrixUseCase::new(
        project_service.clone(),
        user_service.clone(),
    ));
    
    // 앱 설정
    let app = App::new()
        .app_data(web::Data::new(matrix_use_case.clone()))
        .service(
            web::scope("/api")
                .configure(|cfg| project_user_matrix_controller::configure_routes(cfg, matrix_use_case.clone()))
        );
    
    // When: 매트릭스 API 호출
    let req = test::TestRequest::get()
        .uri("/api/project-user-matrix")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // Then: 성공적으로 응답해야 함
    assert!(resp.status().is_success());
    
    let matrix_response: serde_json::Value = test::read_body_json(resp).await;
    
    // 응답 구조 확인
    assert!(matrix_response.get("matrix").is_some());
    assert!(matrix_response.get("users").is_some());
    assert!(matrix_response.get("pagination").is_some());
    
    // 매트릭스 데이터 확인
    let matrix = matrix_response.get("matrix").unwrap().as_array().unwrap();
    assert!(!matrix.is_empty());
    
    // 페이지네이션 확인
    let pagination = matrix_response.get("pagination").unwrap();
    assert!(pagination.get("project_total_count").unwrap().as_i64().unwrap() > 0);
    assert!(pagination.get("user_total_count").unwrap().as_i64().unwrap() > 0);
}

/// 상태 필터링 통합 테스트
#[tokio::test]
async fn test_matrix_status_filtering_integration() {
    let pool = setup_test_database().await;
    setup_test_data(&pool).await;
    
    let app = create_test_app(pool).await;
    
    // When: IN_PROGRESS 상태로 필터링
    let req = test::TestRequest::get()
        .uri("/api/project-user-matrix?project_statuses=IN_PROGRESS")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // Then: 성공적으로 응답해야 함
    assert!(resp.status().is_success());
    
    let matrix_response: serde_json::Value = test::read_body_json(resp).await;
    let matrix = matrix_response.get("matrix").unwrap().as_array().unwrap();
    
    // 모든 프로젝트가 IN_PROGRESS 상태인지 확인
    for project in matrix {
        let status = project.get("status").unwrap().as_str().unwrap();
        assert_eq!(status, "IN_PROGRESS");
    }
}

/// 페이지네이션 통합 테스트
#[tokio::test]
async fn test_matrix_pagination_integration() {
    let pool = setup_test_database().await;
    setup_large_test_data(&pool).await;
    
    let app = create_test_app(pool).await;
    
    // When: 첫 번째 페이지 조회
    let req1 = test::TestRequest::get()
        .uri("/api/project-user-matrix?project_page=1&project_page_size=2&user_page=1&user_page_size=2")
        .to_request();
    
    let resp1 = test::call_service(&app, req1).await;
    assert!(resp1.status().is_success());
    
    let matrix_response1: serde_json::Value = test::read_body_json(resp1).await;
    let matrix1 = matrix_response1.get("matrix").unwrap().as_array().unwrap();
    let users1 = matrix_response1.get("users").unwrap().as_array().unwrap();
    
    // 두 번째 페이지 조회
    let req2 = test::TestRequest::get()
        .uri("/api/project-user-matrix?project_page=2&project_page_size=2&user_page=1&user_page_size=2")
        .to_request();
    
    let resp2 = test::call_service(&app, req2).await;
    assert!(resp2.status().is_success());
    
    let matrix_response2: serde_json::Value = test::read_body_json(resp2).await;
    let matrix2 = matrix_response2.get("matrix").unwrap().as_array().unwrap();
    
    // Then: 페이지 간 중복이 없어야 함
    let page1_project_ids: std::collections::HashSet<i32> = matrix1
        .iter()
        .map(|p| p.get("project_id").unwrap().as_i64().unwrap() as i32)
        .collect();
    
    let page2_project_ids: std::collections::HashSet<i32> = matrix2
        .iter()
        .map(|p| p.get("project_id").unwrap().as_i64().unwrap() as i32)
        .collect();
    
    assert!(page1_project_ids.is_disjoint(&page2_project_ids));
    
    // 페이지 크기 확인
    assert!(matrix1.len() <= 2);
    assert!(matrix2.len() <= 2);
    assert!(users1.len() <= 2);
}

/// 프로젝트 ID 필터링 통합 테스트
#[tokio::test]
async fn test_matrix_project_ids_filtering_integration() {
    let pool = setup_test_database().await;
    let project_ids = setup_test_data(&pool).await;
    
    let app = create_test_app(pool).await;
    
    // When: 특정 프로젝트 ID들로 필터링
    let project_ids_str = project_ids.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(",");
    let req = test::TestRequest::get()
        .uri(&format!("/api/project-user-matrix?project_ids={}", project_ids_str))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // Then: 성공적으로 응답해야 함
    assert!(resp.status().is_success());
    
    let matrix_response: serde_json::Value = test::read_body_json(resp).await;
    let matrix = matrix_response.get("matrix").unwrap().as_array().unwrap();
    
    // 모든 프로젝트가 지정된 ID들 중 하나인지 확인
    for project in matrix {
        let project_id = project.get("project_id").unwrap().as_i64().unwrap() as i32;
        assert!(project_ids.contains(&project_id));
    }
    
    // 프로젝트 개수가 예상과 일치하는지 확인
    assert_eq!(matrix.len(), project_ids.len());
}

/// 사용자 ID 필터링 통합 테스트
#[tokio::test]
async fn test_matrix_user_ids_filtering_integration() {
    let pool = setup_test_database().await;
    let (_, user_ids) = setup_test_data(&pool).await;
    
    let app = create_test_app(pool).await;
    
    // When: 특정 사용자 ID들로 필터링
    let user_ids_str = user_ids.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(",");
    let req = test::TestRequest::get()
        .uri(&format!("/api/project-user-matrix?user_ids={}", user_ids_str))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // Then: 성공적으로 응답해야 함
    assert!(resp.status().is_success());
    
    let matrix_response: serde_json::Value = test::read_body_json(resp).await;
    let users = matrix_response.get("users").unwrap().as_array().unwrap();
    
    // 모든 사용자가 지정된 ID들 중 하나인지 확인
    for user in users {
        let user_id = user.get("user_id").unwrap().as_i64().unwrap() as i32;
        assert!(user_ids.contains(&user_id));
    }
    
    // 사용자 개수가 예상과 일치하는지 확인
    assert_eq!(users.len(), user_ids.len());
}

/// 복합 필터링 통합 테스트
#[tokio::test]
async fn test_matrix_complex_filtering_integration() {
    let pool = setup_test_database().await;
    let (project_ids, user_ids) = setup_test_data(&pool).await;
    
    let app = create_test_app(pool).await;
    
    // When: 상태, 프로젝트 ID, 사용자 ID를 모두 필터링
    let project_ids_str = project_ids.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(",");
    let user_ids_str = user_ids.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(",");
    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/project-user-matrix?project_statuses=IN_PROGRESS&project_ids={}&user_ids={}",
            project_ids_str, user_ids_str
        ))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // Then: 성공적으로 응답해야 함
    assert!(resp.status().is_success());
    
    let matrix_response: serde_json::Value = test::read_body_json(resp).await;
    let matrix = matrix_response.get("matrix").unwrap().as_array().unwrap();
    let users = matrix_response.get("users").unwrap().as_array().unwrap();
    
    // 모든 조건이 만족되는지 확인
    for project in matrix {
        let project_id = project.get("project_id").unwrap().as_i64().unwrap() as i32;
        let status = project.get("status").unwrap().as_str().unwrap();
        
        assert!(project_ids.contains(&project_id));
        assert_eq!(status, "IN_PROGRESS");
    }
    
    for user in users {
        let user_id = user.get("user_id").unwrap().as_i64().unwrap() as i32;
        assert!(user_ids.contains(&user_id));
    }
}

/// 에러 처리 통합 테스트
#[tokio::test]
async fn test_matrix_error_handling_integration() {
    let pool = setup_test_database().await;
    let app = create_test_app(pool).await;
    
    // When: 잘못된 쿼리 파라미터로 요청
    let req = test::TestRequest::get()
        .uri("/api/project-user-matrix?project_page=0&project_page_size=-1")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // Then: 에러가 발생하거나 기본값이 적용되어야 함
    // (구현에 따라 다를 수 있음)
    if resp.status().is_success() {
        let matrix_response: serde_json::Value = test::read_body_json(resp).await;
        // 기본값이 적용되었는지 확인
        let pagination = matrix_response.get("pagination").unwrap();
        let project_page = pagination.get("project_page").unwrap().as_i64().unwrap();
        let project_page_size = pagination.get("project_page_size").unwrap().as_i64().unwrap();
        
        assert!(project_page >= 1);
        assert!(project_page_size >= 1);
    }
}

/// 헬스 체크 테스트
#[tokio::test]
async fn test_matrix_health_check() {
    let pool = setup_test_database().await;
    let app = create_test_app(pool).await;
    
    // When: 헬스 체크 요청
    let req = test::TestRequest::get()
        .uri("/health")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // Then: 성공적으로 응답해야 함
    assert!(resp.status().is_success());
}

/// 헬퍼 함수들
async fn setup_test_database() -> PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://admin:admin123@localhost:5432/pacs_db".to_string());
    
    sqlx::PgPool::connect(&database_url).await.unwrap()
}

async fn setup_test_data(pool: &PgPool) -> (Vec<i32>, Vec<i32>) {
    // 프로젝트 생성
    let project_result1 = sqlx::query!(
        "INSERT INTO security_project (name, description, is_active, status) 
         VALUES ('Integration Test Project 1', 'Test Description', true, 'IN_PROGRESS') 
         RETURNING id"
    )
    .fetch_one(pool)
    .await
    .unwrap();
    
    let project_result2 = sqlx::query!(
        "INSERT INTO security_project (name, description, is_active, status) 
         VALUES ('Integration Test Project 2', 'Test Description', true, 'COMPLETED') 
         RETURNING id"
    )
    .fetch_one(pool)
    .await
    .unwrap();
    
    let project_ids = vec![project_result1.id, project_result2.id];
    
    // 사용자 생성
    let user_result1 = sqlx::query!(
        "INSERT INTO security_user (keycloak_id, username, email) 
         VALUES (gen_random_uuid(), 'integration_user1', 'integration1@example.com') 
         RETURNING id"
    )
    .fetch_one(pool)
    .await
    .unwrap();
    
    let user_result2 = sqlx::query!(
        "INSERT INTO security_user (keycloak_id, username, email) 
         VALUES (gen_random_uuid(), 'integration_user2', 'integration2@example.com') 
         RETURNING id"
    )
    .fetch_one(pool)
    .await
    .unwrap();
    
    let user_ids = vec![user_result1.id, user_result2.id];
    
    (project_ids, user_ids)
}

async fn setup_large_test_data(pool: &PgPool) {
    // 대량의 테스트 데이터 생성
    for i in 1..=10 {
        sqlx::query!(
            "INSERT INTO security_project (name, description, is_active, status) 
             VALUES ($1, 'Large Test Description', true, 'IN_PROGRESS')",
            format!("Large Test Project {}", i)
        )
        .execute(pool)
        .await
        .unwrap();
    }
    
    for i in 1..=10 {
        sqlx::query!(
            "INSERT INTO security_user (keycloak_id, username, email) 
             VALUES (gen_random_uuid(), $1, $2)",
            format!("large_user_{}", i),
            format!("large{}@example.com", i)
        )
        .execute(pool)
        .await
        .unwrap();
    }
}

async fn create_test_app(pool: PgPool) -> impl actix_web::dev::Service<actix_web::dev::ServiceRequest, Response = actix_web::dev::ServiceResponse, Error = actix_web::Error> {
    let project_repository = Arc::new(ProjectRepositoryImpl::new(pool.clone()));
    let user_repository = Arc::new(UserRepositoryImpl::new(pool.clone()));
    let role_repository = Arc::new(RoleRepositoryImpl::new(pool.clone()));
    
    let project_service = Arc::new(ProjectServiceImpl::new(
        project_repository.clone(),
        user_repository.clone(),
        role_repository.clone(),
    ));
    
    let user_service = Arc::new(UserServiceImpl::new(
        user_repository.clone(),
        project_repository.clone(),
    ));
    
    let matrix_use_case = Arc::new(ProjectUserMatrixUseCase::new(
        project_service.clone(),
        user_service.clone(),
    ));
    
    App::new()
        .app_data(web::Data::new(matrix_use_case.clone()))
        .service(
            web::scope("/api")
                .configure(|cfg| project_user_matrix_controller::configure_routes(cfg, matrix_use_case.clone()))
        )
        .route("/health", web::get().to(|| async { "OK" }))
}
