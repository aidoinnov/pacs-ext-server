use actix_web::{test, web, App};
use serde_json::json;
use std::sync::Arc;

use pacs_server::{
    application::{
        dto::project_user_matrix_dto::*,
        use_cases::ProjectUserMatrixUseCase,
    },
    domain::{
        entities::{Project, ProjectStatus, User},
        services::{ProjectService, UserService},
    },
    infrastructure::{
        config::Settings,
        repositories::{
            ProjectRepositoryImpl,
            UserRepositoryImpl,
        },
    },
    presentation::controllers::project_user_matrix_controller,
};

/// Project User Matrix API 상태 필터링 테스트
#[tokio::test]
async fn test_matrix_status_filtering() {
    // Given: 다양한 상태의 프로젝트들이 있는 상황
    let settings = Settings::default();
    let pool = sqlx::PgPool::connect(&settings.database.url).await.unwrap();
    
    // 테스트 데이터 준비
    setup_test_data(&pool).await;
    
    // When: 특정 상태의 프로젝트만 필터링하여 매트릭스 조회
    let response = test::TestRequest::get()
        .uri("/api/project-user-matrix?project_statuses=IN_PROGRESS")
        .to_request()
        .send_request(&create_test_app().await)
        .await;
    
    // Then: IN_PROGRESS 상태의 프로젝트만 반환되어야 함
    assert!(response.status().is_success());
    
    let matrix_response: ProjectUserMatrixResponse = test::read_body_json(response).await;
    
    // 모든 프로젝트가 IN_PROGRESS 상태인지 확인
    for project_row in &matrix_response.matrix {
        assert_eq!(project_row.status, "IN_PROGRESS");
    }
    
    // 프로젝트 개수가 예상과 일치하는지 확인
    assert!(matrix_response.matrix.len() > 0);
}

/// 여러 상태 필터링 테스트
#[tokio::test]
async fn test_matrix_multiple_status_filtering() {
    let settings = Settings::default();
    let pool = sqlx::PgPool::connect(&settings.database.url).await.unwrap();
    
    setup_test_data(&pool).await;
    
    // When: 여러 상태의 프로젝트를 필터링
    let response = test::TestRequest::get()
        .uri("/api/project-user-matrix?project_statuses=IN_PROGRESS,COMPLETED")
        .to_request()
        .send_request(&create_test_app().await)
        .await;
    
    // Then: IN_PROGRESS 또는 COMPLETED 상태의 프로젝트만 반환
    assert!(response.status().is_success());
    
    let matrix_response: ProjectUserMatrixResponse = test::read_body_json(response).await;
    
    for project_row in &matrix_response.matrix {
        assert!(
            project_row.status == "IN_PROGRESS" || 
            project_row.status == "COMPLETED"
        );
    }
}

/// 상태 필터링 없이 전체 조회 테스트
#[tokio::test]
async fn test_matrix_no_status_filtering() {
    let settings = Settings::default();
    let pool = sqlx::PgPool::connect(&settings.database.url).await.unwrap();
    
    setup_test_data(&pool).await;
    
    // When: 상태 필터 없이 전체 조회
    let response = test::TestRequest::get()
        .uri("/api/project-user-matrix")
        .to_request()
        .send_request(&create_test_app().await)
        .await;
    
    // Then: 모든 상태의 프로젝트가 반환되어야 함
    assert!(response.status().is_success());
    
    let matrix_response: ProjectUserMatrixResponse = test::read_body_json(response).await;
    
    // 다양한 상태의 프로젝트가 포함되어 있는지 확인
    let statuses: std::collections::HashSet<String> = matrix_response
        .matrix
        .iter()
        .map(|row| row.status.clone())
        .collect();
    
    assert!(statuses.len() > 1, "다양한 상태의 프로젝트가 있어야 함");
}

/// 잘못된 상태 필터링 테스트
#[tokio::test]
async fn test_matrix_invalid_status_filtering() {
    let settings = Settings::default();
    let pool = sqlx::PgPool::connect(&settings.database.url).await.unwrap();
    
    setup_test_data(&pool).await;
    
    // When: 잘못된 상태 값으로 필터링
    let response = test::TestRequest::get()
        .uri("/api/project-user-matrix?project_statuses=INVALID_STATUS")
        .to_request()
        .send_request(&create_test_app().await)
        .await;
    
    // Then: 400 Bad Request 또는 빈 결과 반환
    // (구현에 따라 다를 수 있음)
    if response.status().is_success() {
        let matrix_response: ProjectUserMatrixResponse = test::read_body_json(response).await;
        assert_eq!(matrix_response.matrix.len(), 0);
    } else {
        assert_eq!(response.status(), 400);
    }
}

/// 페이지네이션과 상태 필터링 조합 테스트
#[tokio::test]
async fn test_matrix_status_filtering_with_pagination() {
    let settings = Settings::default();
    let pool = sqlx::PgPool::connect(&settings.database.url).await.unwrap();
    
    setup_test_data(&pool).await;
    
    // When: 상태 필터링과 페이지네이션을 함께 사용
    let response = test::TestRequest::get()
        .uri("/api/project-user-matrix?project_statuses=IN_PROGRESS&project_page=1&project_page_size=2")
        .to_request()
        .send_request(&create_test_app().await)
        .await;
    
    // Then: IN_PROGRESS 상태의 프로젝트만 2개씩 반환
    assert!(response.status().is_success());
    
    let matrix_response: ProjectUserMatrixResponse = test::read_body_json(response).await;
    
    // 모든 프로젝트가 IN_PROGRESS 상태인지 확인
    for project_row in &matrix_response.matrix {
        assert_eq!(project_row.status, "IN_PROGRESS");
    }
    
    // 페이지 크기가 2 이하인지 확인
    assert!(matrix_response.matrix.len() <= 2);
    
    // 페이지네이션 정보 확인
    assert_eq!(matrix_response.pagination.project_page, 1);
    assert_eq!(matrix_response.pagination.project_page_size, 2);
}

/// 상태별 프로젝트 개수 확인 테스트
#[tokio::test]
async fn test_matrix_status_count_verification() {
    let settings = Settings::default();
    let pool = sqlx::PgPool::connect(&settings.database.url).await.unwrap();
    
    setup_test_data(&pool).await;
    
    // When: 각 상태별로 조회
    let in_progress_response = test::TestRequest::get()
        .uri("/api/project-user-matrix?project_statuses=IN_PROGRESS")
        .to_request()
        .send_request(&create_test_app().await)
        .await;
    
    let completed_response = test::TestRequest::get()
        .uri("/api/project-user-matrix?project_statuses=COMPLETED")
        .to_request()
        .send_request(&create_test_app().await)
        .await;
    
    let all_response = test::TestRequest::get()
        .uri("/api/project-user-matrix")
        .to_request()
        .send_request(&create_test_app().await)
        .await;
    
    // Then: 각 상태별 개수의 합이 전체 개수와 일치해야 함
    let in_progress_matrix: ProjectUserMatrixResponse = test::read_body_json(in_progress_response).await;
    let completed_matrix: ProjectUserMatrixResponse = test::read_body_json(completed_response).await;
    let all_matrix: ProjectUserMatrixResponse = test::read_body_json(all_response).await;
    
    let in_progress_count = in_progress_matrix.pagination.project_total_count;
    let completed_count = completed_matrix.pagination.project_total_count;
    let all_count = all_matrix.pagination.project_total_count;
    
    // IN_PROGRESS + COMPLETED = 전체 (다른 상태가 없는 경우)
    // 또는 IN_PROGRESS + COMPLETED <= 전체 (다른 상태가 있는 경우)
    assert!(in_progress_count + completed_count <= all_count);
}

/// 테스트 데이터 설정
async fn setup_test_data(pool: &sqlx::PgPool) {
    // 테스트용 프로젝트 생성 (다양한 상태)
    let projects = vec![
        ("Test Project 1", "IN_PROGRESS"),
        ("Test Project 2", "COMPLETED"),
        ("Test Project 3", "PREPARING"),
        ("Test Project 4", "IN_PROGRESS"),
        ("Test Project 5", "ON_HOLD"),
    ];
    
    for (name, status) in projects {
        sqlx::query!(
            "INSERT INTO security_project (name, description, is_active, status) 
             VALUES ($1, 'Test Description', true, $2::project_status)",
            name,
            status
        )
        .execute(pool)
        .await
        .unwrap();
    }
    
    // 테스트용 사용자 생성
    let users = vec![
        ("testuser1", "user1@example.com"),
        ("testuser2", "user2@example.com"),
        ("testuser3", "user3@example.com"),
    ];
    
    for (username, email) in users {
        sqlx::query!(
            "INSERT INTO security_user (keycloak_id, username, email) 
             VALUES (gen_random_uuid(), $1, $2)",
            username,
            email
        )
        .execute(pool)
        .await
        .unwrap();
    }
}

/// 테스트용 앱 생성
async fn create_test_app() -> impl actix_web::dev::Service<actix_web::dev::ServiceRequest, Response = actix_web::dev::ServiceResponse, Error = actix_web::Error> {
    let settings = Settings::default();
    let pool = sqlx::PgPool::connect(&settings.database.url).await.unwrap();
    
    // Repository 생성
    let project_repository = Arc::new(ProjectRepositoryImpl::new(pool.clone()));
    let user_repository = Arc::new(UserRepositoryImpl::new(pool.clone()));
    
    // Service 생성
    let project_service = Arc::new(
        pacs_server::domain::services::project_service::ProjectServiceImpl::new(
            project_repository.clone(),
            user_repository.clone(),
            Arc::new(pacs_server::infrastructure::repositories::role_repository_impl::RoleRepositoryImpl::new(pool.clone())),
        )
    );
    let user_service = Arc::new(
        pacs_server::domain::services::user_service::UserServiceImpl::new(
            user_repository.clone(),
            project_repository.clone(),
        )
    );
    
    // Use Case 생성
    let matrix_use_case = Arc::new(ProjectUserMatrixUseCase::new(
        project_service.clone(),
        user_service.clone(),
    ));
    
    // 앱 생성
    App::new()
        .app_data(web::Data::new(matrix_use_case))
        .service(
            web::scope("/api")
                .configure(|cfg| project_user_matrix_controller::configure_routes(cfg, matrix_use_case.clone()))
        )
}
