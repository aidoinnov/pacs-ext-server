use actix_web::{test, web, App};
use std::sync::Arc;
use mockall::mock;
use async_trait::async_trait;

use pacs_server::{
    application::{
        dto::project_user_matrix_dto::*,
        use_cases::ProjectUserMatrixUseCase,
    },
    domain::{
        entities::{Project, ProjectStatus, User},
        services::{ProjectService, UserService},
        ServiceError,
    },
    presentation::controllers::project_user_matrix_controller,
};

// Mock 서비스들 생성
mock! {
    ProjectService {}
    
    #[async_trait]
    impl ProjectService for ProjectService {
        async fn get_projects_with_status_filter(
            &self,
            statuses: Option<Vec<ProjectStatus>>,
            project_ids: Option<Vec<i32>>,
            page: i32,
            page_size: i32,
        ) -> Result<(Vec<Project>, i64), ServiceError>;
        
        async fn get_user_project_roles_matrix(
            &self,
            project_ids: Vec<i32>,
            user_ids: Vec<i32>,
        ) -> Result<Vec<crate::domain::services::project_service::UserProjectRoleInfo>, ServiceError>;
    }
}

mock! {
    UserService {}
    
    #[async_trait]
    impl UserService for UserService {
        async fn get_users_with_filter(
            &self,
            user_ids: Option<Vec<i32>>,
            page: i32,
            page_size: i32,
        ) -> Result<(Vec<User>, i64), ServiceError>;
    }
}

/// 데이터베이스 연결 에러 테스트
#[tokio::test]
async fn test_matrix_database_connection_error() {
    // Given: 데이터베이스 연결 에러를 시뮬레이션하는 Mock
    let mut mock_project_service = MockProjectService::new();
    let mut mock_user_service = MockUserService::new();
    
    // ProjectService에서 데이터베이스 연결 에러 발생
    mock_project_service
        .expect_get_projects_with_status_filter()
        .returning(|_, _, _, _| Err(ServiceError::DatabaseError("Connection failed".to_string())));
    
    let use_case = ProjectUserMatrixUseCase::new(
        Arc::new(mock_project_service),
        Arc::new(mock_user_service),
    );
    
    let app = App::new()
        .app_data(web::Data::new(use_case.clone()))
        .service(
            web::scope("/api")
                .configure(|cfg| project_user_matrix_controller::configure_routes(cfg, use_case.clone()))
        );
    
    // When: 매트릭스 API 호출
    let req = test::TestRequest::get()
        .uri("/api/project-user-matrix")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // Then: 500 Internal Server Error가 반환되어야 함
    assert_eq!(resp.status(), 500);
}

/// 사용자 서비스 에러 테스트
#[tokio::test]
async fn test_matrix_user_service_error() {
    let mut mock_project_service = MockProjectService::new();
    let mut mock_user_service = MockUserService::new();
    
    let test_projects = create_test_projects();
    
    mock_project_service
        .expect_get_projects_with_status_filter()
        .returning(move |_, _, _, _| Ok((test_projects.clone(), 2)));
    
    // UserService에서 에러 발생
    mock_user_service
        .expect_get_users_with_filter()
        .returning(|_, _, _| Err(ServiceError::DatabaseError("User service error".to_string())));
    
    let use_case = ProjectUserMatrixUseCase::new(
        Arc::new(mock_project_service),
        Arc::new(mock_user_service),
    );
    
    let app = App::new()
        .app_data(web::Data::new(use_case.clone()))
        .service(
            web::scope("/api")
                .configure(|cfg| project_user_matrix_controller::configure_routes(cfg, use_case.clone()))
        );
    
    // When: 매트릭스 API 호출
    let req = test::TestRequest::get()
        .uri("/api/project-user-matrix")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // Then: 500 Internal Server Error가 반환되어야 함
    assert_eq!(resp.status(), 500);
}

/// 잘못된 쿼리 파라미터 테스트
#[tokio::test]
async fn test_matrix_invalid_query_parameters() {
    let mut mock_project_service = MockProjectService::new();
    let mut mock_user_service = MockUserService::new();
    
    let test_projects = create_test_projects();
    let test_users = create_test_users();
    let test_relationships = create_test_relationships();
    
    mock_project_service
        .expect_get_projects_with_status_filter()
        .returning(move |_, _, _, _| Ok((test_projects.clone(), 2)));
    
    mock_project_service
        .expect_get_user_project_roles_matrix()
        .returning(move |_, _| Ok(test_relationships.clone()));
    
    mock_user_service
        .expect_get_users_with_filter()
        .returning(move |_, _, _| Ok((test_users.clone(), 2)));
    
    let use_case = ProjectUserMatrixUseCase::new(
        Arc::new(mock_project_service),
        Arc::new(mock_user_service),
    );
    
    let app = App::new()
        .app_data(web::Data::new(use_case.clone()))
        .service(
            web::scope("/api")
                .configure(|cfg| project_user_matrix_controller::configure_routes(cfg, use_case.clone()))
        );
    
    // When: 잘못된 쿼리 파라미터로 요청
    let test_cases = vec![
        "/api/project-user-matrix?project_page=0", // 잘못된 페이지 번호
        "/api/project-user-matrix?project_page_size=-1", // 잘못된 페이지 크기
        "/api/project-user-matrix?project_page=abc", // 잘못된 페이지 타입
        "/api/project-user-matrix?project_statuses=INVALID_STATUS", // 잘못된 상태
    ];
    
    for uri in test_cases {
        let req = test::TestRequest::get()
            .uri(uri)
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        
        // Then: 400 Bad Request 또는 기본값 적용으로 200 OK
        assert!(resp.status().is_client_error() || resp.status().is_success());
    }
}

/// 빈 결과 처리 테스트
#[tokio::test]
async fn test_matrix_empty_results() {
    let mut mock_project_service = MockProjectService::new();
    let mut mock_user_service = MockUserService::new();
    
    // 빈 결과 반환
    mock_project_service
        .expect_get_projects_with_status_filter()
        .returning(|_, _, _, _| Ok((vec![], 0)));
    
    mock_user_service
        .expect_get_users_with_filter()
        .returning(|_, _, _| Ok((vec![], 0)));
    
    let use_case = ProjectUserMatrixUseCase::new(
        Arc::new(mock_project_service),
        Arc::new(mock_user_service),
    );
    
    let app = App::new()
        .app_data(web::Data::new(use_case.clone()))
        .service(
            web::scope("/api")
                .configure(|cfg| project_user_matrix_controller::configure_routes(cfg, use_case.clone()))
        );
    
    // When: 매트릭스 API 호출
    let req = test::TestRequest::get()
        .uri("/api/project-user-matrix")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // Then: 성공적으로 빈 결과가 반환되어야 함
    assert!(resp.status().is_success());
    
    let matrix_response: ProjectUserMatrixResponse = test::read_body_json(resp).await;
    assert_eq!(matrix_response.matrix.len(), 0);
    assert_eq!(matrix_response.users.len(), 0);
    assert_eq!(matrix_response.pagination.project_total_count, 0);
    assert_eq!(matrix_response.pagination.user_total_count, 0);
}

/// 메모리 부족 시뮬레이션 테스트
#[tokio::test]
async fn test_matrix_memory_error() {
    let mut mock_project_service = MockProjectService::new();
    let mut mock_user_service = MockUserService::new();
    
    // 메모리 부족 에러 시뮬레이션
    mock_project_service
        .expect_get_projects_with_status_filter()
        .returning(|_, _, _, _| Err(ServiceError::DatabaseError("Out of memory".to_string())));
    
    let use_case = ProjectUserMatrixUseCase::new(
        Arc::new(mock_project_service),
        Arc::new(mock_user_service),
    );
    
    let app = App::new()
        .app_data(web::Data::new(use_case.clone()))
        .service(
            web::scope("/api")
                .configure(|cfg| project_user_matrix_controller::configure_routes(cfg, use_case.clone()))
        );
    
    // When: 매트릭스 API 호출
    let req = test::TestRequest::get()
        .uri("/api/project-user-matrix")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // Then: 500 Internal Server Error가 반환되어야 함
    assert_eq!(resp.status(), 500);
}

/// 타임아웃 에러 테스트
#[tokio::test]
async fn test_matrix_timeout_error() {
    let mut mock_project_service = MockProjectService::new();
    let mut mock_user_service = MockUserService::new();
    
    // 타임아웃 에러 시뮬레이션
    mock_project_service
        .expect_get_projects_with_status_filter()
        .returning(|_, _, _, _| Err(ServiceError::DatabaseError("Query timeout".to_string())));
    
    let use_case = ProjectUserMatrixUseCase::new(
        Arc::new(mock_project_service),
        Arc::new(mock_user_service),
    );
    
    let app = App::new()
        .app_data(web::Data::new(use_case.clone()))
        .service(
            web::scope("/api")
                .configure(|cfg| project_user_matrix_controller::configure_routes(cfg, use_case.clone()))
        );
    
    // When: 매트릭스 API 호출
    let req = test::TestRequest::get()
        .uri("/api/project-user-matrix")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // Then: 500 Internal Server Error가 반환되어야 함
    assert_eq!(resp.status(), 500);
}

/// 권한 에러 테스트
#[tokio::test]
async fn test_matrix_permission_error() {
    let mut mock_project_service = MockProjectService::new();
    let mut mock_user_service = MockUserService::new();
    
    // 권한 에러 시뮬레이션
    mock_project_service
        .expect_get_projects_with_status_filter()
        .returning(|_, _, _, _| Err(ServiceError::PermissionDenied("Access denied".to_string())));
    
    let use_case = ProjectUserMatrixUseCase::new(
        Arc::new(mock_project_service),
        Arc::new(mock_user_service),
    );
    
    let app = App::new()
        .app_data(web::Data::new(use_case.clone()))
        .service(
            web::scope("/api")
                .configure(|cfg| project_user_matrix_controller::configure_routes(cfg, use_case.clone()))
        );
    
    // When: 매트릭스 API 호출
    let req = test::TestRequest::get()
        .uri("/api/project-user-matrix")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // Then: 403 Forbidden이 반환되어야 함
    assert_eq!(resp.status(), 403);
}

/// 잘못된 JSON 응답 테스트
#[tokio::test]
async fn test_matrix_invalid_json_response() {
    let mut mock_project_service = MockProjectService::new();
    let mut mock_user_service = MockUserService::new();
    
    let test_projects = create_test_projects();
    let test_users = create_test_users();
    let test_relationships = create_test_relationships();
    
    mock_project_service
        .expect_get_projects_with_status_filter()
        .returning(move |_, _, _, _| Ok((test_projects.clone(), 2)));
    
    mock_project_service
        .expect_get_user_project_roles_matrix()
        .returning(move |_, _| Ok(test_relationships.clone()));
    
    mock_user_service
        .expect_get_users_with_filter()
        .returning(move |_, _, _| Ok((test_users.clone(), 2)));
    
    let use_case = ProjectUserMatrixUseCase::new(
        Arc::new(mock_project_service),
        Arc::new(mock_user_service),
    );
    
    let app = App::new()
        .app_data(web::Data::new(use_case.clone()))
        .service(
            web::scope("/api")
                .configure(|cfg| project_user_matrix_controller::configure_routes(cfg, use_case.clone()))
        );
    
    // When: 매트릭스 API 호출
    let req = test::TestRequest::get()
        .uri("/api/project-user-matrix")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // Then: 성공적으로 JSON이 반환되어야 함
    assert!(resp.status().is_success());
    
    // JSON 파싱이 성공하는지 확인
    let matrix_response: Result<ProjectUserMatrixResponse, _> = test::read_body_json(resp).await;
    assert!(matrix_response.is_ok());
}

/// 대용량 데이터 에러 테스트
#[tokio::test]
async fn test_matrix_large_data_error() {
    let mut mock_project_service = MockProjectService::new();
    let mut mock_user_service = MockUserService::new();
    
    // 대용량 데이터로 인한 에러 시뮬레이션
    mock_project_service
        .expect_get_projects_with_status_filter()
        .returning(|_, _, _, _| Err(ServiceError::DatabaseError("Result set too large".to_string())));
    
    let use_case = ProjectUserMatrixUseCase::new(
        Arc::new(mock_project_service),
        Arc::new(mock_user_service),
    );
    
    let app = App::new()
        .app_data(web::Data::new(use_case.clone()))
        .service(
            web::scope("/api")
                .configure(|cfg| project_user_matrix_controller::configure_routes(cfg, use_case.clone()))
        );
    
    // When: 매트릭스 API 호출
    let req = test::TestRequest::get()
        .uri("/api/project-user-matrix")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // Then: 500 Internal Server Error가 반환되어야 함
    assert_eq!(resp.status(), 500);
}

/// 네트워크 에러 테스트
#[tokio::test]
async fn test_matrix_network_error() {
    let mut mock_project_service = MockProjectService::new();
    let mut mock_user_service = MockUserService::new();
    
    // 네트워크 에러 시뮬레이션
    mock_project_service
        .expect_get_projects_with_status_filter()
        .returning(|_, _, _, _| Err(ServiceError::DatabaseError("Network unreachable".to_string())));
    
    let use_case = ProjectUserMatrixUseCase::new(
        Arc::new(mock_project_service),
        Arc::new(mock_user_service),
    );
    
    let app = App::new()
        .app_data(web::Data::new(use_case.clone()))
        .service(
            web::scope("/api")
                .configure(|cfg| project_user_matrix_controller::configure_routes(cfg, use_case.clone()))
        );
    
    // When: 매트릭스 API 호출
    let req = test::TestRequest::get()
        .uri("/api/project-user-matrix")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // Then: 500 Internal Server Error가 반환되어야 함
    assert_eq!(resp.status(), 500);
}

/// 테스트 데이터 생성 함수들
fn create_test_projects() -> Vec<Project> {
    vec![
        Project {
            id: 1,
            name: "Test Project 1".to_string(),
            description: Some("Test Description".to_string()),
            is_active: true,
            status: ProjectStatus::InProgress,
            created_at: chrono::Utc::now(),
        },
    ]
}

fn create_test_users() -> Vec<User> {
    vec![
        User {
            id: 1,
            keycloak_id: uuid::Uuid::new_v4(),
            username: "testuser1".to_string(),
            email: "user1@example.com".to_string(),
            full_name: Some("Test User 1".to_string()),
            organization: Some("Test Org".to_string()),
            department: Some("Test Dept".to_string()),
            phone: Some("010-1234-5678".to_string()),
            created_at: chrono::Utc::now(),
            updated_at: Some(chrono::Utc::now()),
        },
    ]
}

fn create_test_relationships() -> Vec<crate::domain::services::project_service::UserProjectRoleInfo> {
    vec![
        crate::domain::services::project_service::UserProjectRoleInfo {
            project_id: 1,
            user_id: 1,
            role_id: None,
            role_name: None,
        },
    ]
}
