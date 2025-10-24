use actix_web::{test, web, App};
use std::sync::Arc;
use serde_json::json;
use async_trait::async_trait;

use pacs_server::{
    application::{
        dto::project_user_matrix_dto::*,
        use_cases::ProjectUserMatrixUseCase,
    },
    domain::{
        entities::{Project, User, ProjectStatus},
        services::{
            project_service::{ProjectService, UserProjectRoleInfo},
            user_service::UserService,
        },
        ServiceError,
    },
    presentation::controllers::project_user_matrix_controller::get_matrix,
};

// Mock ProjectService
mockall::mock! {
    ProjectService {}
    #[async_trait]
    impl ProjectService for ProjectService {
        async fn create_project(&self, new_project: &pacs_server::domain::entities::NewProject) -> Result<pacs_server::domain::entities::Project, ServiceError> { todo!() }
        async fn get_project_by_name(&self, name: &str) -> Result<pacs_server::domain::entities::Project, ServiceError> { todo!() }
        async fn get_all_projects(&self, page: i32, page_size: i32) -> Result<(Vec<pacs_server::domain::entities::Project>, i64), ServiceError> { todo!() }
        async fn update_project(&self, id: i32, new_project: pacs_server::domain::entities::NewProject) -> Result<pacs_server::domain::entities::Project, ServiceError> { todo!() }
        async fn activate_project(&self, id: i32) -> Result<pacs_server::domain::entities::Project, ServiceError> { todo!() }
        async fn deactivate_project(&self, id: i32) -> Result<pacs_server::domain::entities::Project, ServiceError> { todo!() }
        async fn delete_project(&self, id: i32) -> Result<(), ServiceError> { todo!() }
        async fn get_project_members(&self, project_id: i32) -> Result<Vec<pacs_server::domain::entities::User>, ServiceError> { todo!() }
        async fn count_project_members(&self, project_id: i32) -> Result<i64, ServiceError> { todo!() }
        async fn get_project_members_with_roles(&self, project_id: i32, page: i32, page_size: i32) -> Result<(Vec<pacs_server::application::dto::project_user_dto::UserWithRoleResponse>, i64), ServiceError> { todo!() }
        async fn assign_role_to_project(&self, project_id: i32, role_id: i32, user_id: i32) -> Result<bool, ServiceError> { todo!() }
        async fn remove_role_from_project(&self, project_id: i32, role_id: i32, user_id: i32) -> Result<bool, ServiceError> { todo!() }
        async fn get_project_roles(&self, project_id: i32) -> Result<Vec<pacs_server::domain::entities::Role>, ServiceError> { todo!() }
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
        ) -> Result<Vec<UserProjectRoleInfo>, ServiceError>;
    }
}

// Mock UserService
mockall::mock! {
    UserService {}
    #[async_trait]
    impl UserService for UserService {
        async fn create_user(&self, new_user: &pacs_server::application::dto::CreateUserRequest) -> Result<pacs_server::domain::entities::User, ServiceError> { todo!() }
        async fn get_user_by_id(&self, id: i32) -> Result<pacs_server::domain::entities::User, ServiceError> { todo!() }
        async fn get_user_by_keycloak_id(&self, keycloak_id: uuid::Uuid) -> Result<pacs_server::domain::entities::User, ServiceError> { todo!() }
        async fn get_user_by_username(&self, username: &str) -> Result<pacs_server::domain::entities::User, ServiceError> { todo!() }
        async fn get_all_users(&self, page: i32, page_size: i32) -> Result<(Vec<pacs_server::domain::entities::User>, i64), ServiceError> { todo!() }
        async fn update_user(&self, id: i32, update_user: &pacs_server::application::dto::UpdateUserRequest) -> Result<pacs_server::domain::entities::User, ServiceError> { todo!() }
        async fn delete_user(&self, id: i32) -> Result<(), ServiceError> { todo!() }
        async fn user_exists(&self, keycloak_id: uuid::Uuid) -> Result<bool, ServiceError> { todo!() }
        async fn add_user_to_project(&self, user_id: i32, project_id: i32) -> Result<bool, ServiceError> { todo!() }
        async fn remove_user_from_project(&self, user_id: i32, project_id: i32) -> Result<bool, ServiceError> { todo!() }
        async fn get_user_projects(&self, user_id: i32) -> Result<Vec<pacs_server::domain::entities::Project>, ServiceError> { todo!() }
        async fn is_project_member(&self, user_id: i32, project_id: i32) -> Result<bool, ServiceError> { todo!() }
        async fn get_users_with_filter(
            &self,
            user_ids: Option<Vec<i32>>,
            page: i32,
            page_size: i32,
        ) -> Result<(Vec<User>, i64), ServiceError>;
    }
}

/// 포괄적인 통합 테스트 - 기본 매트릭스 조회
#[actix_web::test]
async fn test_matrix_basic_integration() {
    // Given: Mock 서비스들이 정상 데이터를 반환하도록 설정
    let mut mock_project_service = MockProjectService::new();
    let mut mock_user_service = MockUserService::new();

    // 프로젝트 데이터 설정
    let projects = vec![
        Project {
            id: 1,
            name: "Test Project 1".to_string(),
            description: Some("Test Description 1".to_string()),
            is_active: true,
            status: ProjectStatus::InProgress,
            created_at: chrono::Utc::now(),
        },
        Project {
            id: 2,
            name: "Test Project 2".to_string(),
            description: Some("Test Description 2".to_string()),
            is_active: true,
            status: ProjectStatus::Completed,
            created_at: chrono::Utc::now(),
        },
    ];

    // 사용자 데이터 설정
    let users = vec![
        User {
            id: 1,
            keycloak_id: uuid::Uuid::new_v4(),
            username: "user1".to_string(),
            email: "user1@example.com".to_string(),
            full_name: Some("User One".to_string()),
            organization: Some("Test Org".to_string()),
            department: Some("Test Dept".to_string()),
            phone: Some("010-1234-5678".to_string()),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
        User {
            id: 2,
            keycloak_id: uuid::Uuid::new_v4(),
            username: "user2".to_string(),
            email: "user2@example.com".to_string(),
            full_name: Some("User Two".to_string()),
            organization: Some("Test Org".to_string()),
            department: Some("Test Dept".to_string()),
            phone: Some("010-2345-6789".to_string()),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
    ];

    // 역할 관계 데이터 설정
    let relationships = vec![
        UserProjectRoleInfo {
            project_id: 1,
            user_id: 1,
            role_id: Some(1),
            role_name: Some("Admin".to_string()),
        },
        UserProjectRoleInfo {
            project_id: 1,
            user_id: 2,
            role_id: Some(2),
            role_name: Some("Viewer".to_string()),
        },
        UserProjectRoleInfo {
            project_id: 2,
            user_id: 1,
            role_id: None,
            role_name: None,
        },
        UserProjectRoleInfo {
            project_id: 2,
            user_id: 2,
            role_id: Some(1),
            role_name: Some("Admin".to_string()),
        },
    ];

    // Mock 설정
    mock_project_service
        .expect_get_projects_with_status_filter()
        .returning(move |_, _, _, _| Ok((projects.clone(), 2)));
    
    mock_user_service
        .expect_get_users_with_filter()
        .returning(move |_, _, _| Ok((users.clone(), 2)));
    
    mock_project_service
        .expect_get_user_project_roles_matrix()
        .returning(move |_, _| Ok(relationships.clone()));

    let use_case = Arc::new(ProjectUserMatrixUseCase::new(
        Arc::new(mock_project_service),
        Arc::new(mock_user_service),
    ));

    let app = test::init_service(
        App::new()
            .app_data(web::Data::from(use_case.clone()))
            .service(web::resource("/api/project-user-matrix").to(get_matrix))
    ).await;

    // When: 매트릭스 API 호출
    let req = test::TestRequest::get()
        .uri("/api/project-user-matrix")
        .to_request();
    let resp = test::call_service(&app, req).await;

    // Then: 성공적인 응답이 반환되어야 함
    assert!(resp.status().is_success());
    
    let response_body: ProjectUserMatrixResponse = test::read_body_json(resp).await;
    
    // 매트릭스 구조 검증
    assert_eq!(response_body.matrix.len(), 2);
    assert_eq!(response_body.users.len(), 2);
    
    // 첫 번째 프로젝트 검증
    let project1 = &response_body.matrix[0];
    assert_eq!(project1.project_id, 1);
    assert_eq!(project1.project_name, "Test Project 1");
    assert_eq!(project1.status, ProjectStatus::InProgress);
    assert_eq!(project1.user_roles.len(), 2);
    
    // 사용자 역할 검증
    let user1_role = project1.user_roles.iter().find(|ur| ur.user_id == 1).unwrap();
    assert_eq!(user1_role.username, "user1");
    assert_eq!(user1_role.role_name, Some("Admin".to_string()));
    
    let user2_role = project1.user_roles.iter().find(|ur| ur.user_id == 2).unwrap();
    assert_eq!(user2_role.username, "user2");
    assert_eq!(user2_role.role_name, Some("Viewer".to_string()));
    
    // 페이지네이션 검증
    assert_eq!(response_body.pagination.project_total_count, 2);
    assert_eq!(response_body.pagination.user_total_count, 2);
}

/// 상태 필터링 통합 테스트
#[actix_web::test]
async fn test_matrix_status_filtering_integration() {
    // Given: Mock 서비스 설정
    let mut mock_project_service = MockProjectService::new();
    let mut mock_user_service = MockUserService::new();

    let in_progress_projects = vec![
        Project {
            id: 1,
            name: "In Progress Project".to_string(),
            description: Some("Description".to_string()),
            is_active: true,
            status: ProjectStatus::InProgress,
            created_at: chrono::Utc::now(),
        },
    ];

    let users = vec![
        User {
            id: 1,
            keycloak_id: uuid::Uuid::new_v4(),
            username: "user1".to_string(),
            email: "user1@example.com".to_string(),
            full_name: Some("User One".to_string()),
            organization: Some("Test Org".to_string()),
            department: Some("Test Dept".to_string()),
            phone: Some("010-1234-5678".to_string()),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
    ];

    let relationships = vec![
        UserProjectRoleInfo {
            project_id: 1,
            user_id: 1,
            role_id: Some(1),
            role_name: Some("Admin".to_string()),
        },
    ];

    // Mock 설정 - IN_PROGRESS 상태만 필터링
    mock_project_service
        .expect_get_projects_with_status_filter()
        .withf(|statuses, _, _, _| {
            statuses.as_ref().map_or(false, |s| s.contains(&ProjectStatus::InProgress))
        })
        .returning(move |_, _, _, _| Ok((in_progress_projects.clone(), 1)));
    
    mock_user_service
        .expect_get_users_with_filter()
        .returning(move |_, _, _| Ok((users.clone(), 1)));
    
    mock_project_service
        .expect_get_user_project_roles_matrix()
        .returning(move |_, _| Ok(relationships.clone()));

    let use_case = Arc::new(ProjectUserMatrixUseCase::new(
        Arc::new(mock_project_service),
        Arc::new(mock_user_service),
    ));

    let app = test::init_service(
        App::new()
            .app_data(web::Data::from(use_case.clone()))
            .service(web::resource("/api/project-user-matrix").to(get_matrix))
    ).await;

    // When: IN_PROGRESS 상태로 필터링
    let req = test::TestRequest::get()
        .uri("/api/project-user-matrix?project_status=IN_PROGRESS")
        .to_request();
    let resp = test::call_service(&app, req).await;

    // Then: IN_PROGRESS 프로젝트만 반환되어야 함
    assert!(resp.status().is_success());
    
    let response_body: ProjectUserMatrixResponse = test::read_body_json(resp).await;
    
    assert_eq!(response_body.matrix.len(), 1);
    assert_eq!(response_body.matrix[0].status, ProjectStatus::InProgress);
    assert_eq!(response_body.pagination.project_total_count, 1);
}

/// 페이지네이션 통합 테스트
#[actix_web::test]
async fn test_matrix_pagination_integration() {
    // Given: Mock 서비스 설정
    let mut mock_project_service = MockProjectService::new();
    let mut mock_user_service = MockUserService::new();

    let projects = vec![
        Project {
            id: 1,
            name: "Project 1".to_string(),
            description: Some("Description".to_string()),
            is_active: true,
            status: ProjectStatus::InProgress,
            created_at: chrono::Utc::now(),
        },
        Project {
            id: 2,
            name: "Project 2".to_string(),
            description: Some("Description".to_string()),
            is_active: true,
            status: ProjectStatus::InProgress,
            created_at: chrono::Utc::now(),
        },
    ];

    let users = vec![
        User {
            id: 1,
            keycloak_id: uuid::Uuid::new_v4(),
            username: "user1".to_string(),
            email: "user1@example.com".to_string(),
            full_name: Some("User One".to_string()),
            organization: Some("Test Org".to_string()),
            department: Some("Test Dept".to_string()),
            phone: Some("010-1234-5678".to_string()),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
    ];

    let relationships = vec![
        UserProjectRoleInfo {
            project_id: 1,
            user_id: 1,
            role_id: Some(1),
            role_name: Some("Admin".to_string()),
        },
        UserProjectRoleInfo {
            project_id: 2,
            user_id: 1,
            role_id: Some(1),
            role_name: Some("Admin".to_string()),
        },
    ];

    // Mock 설정 - 페이지네이션 파라미터 검증
    mock_project_service
        .expect_get_projects_with_status_filter()
        .withf(|_, _, page, page_size| *page == 1 && *page_size == 2)
        .returning(move |_, _, _, _| Ok((projects.clone(), 2)));
    
    mock_user_service
        .expect_get_users_with_filter()
        .withf(|_, page, page_size| *page == 1 && *page_size == 1)
        .returning(move |_, _, _| Ok((users.clone(), 1)));
    
    mock_project_service
        .expect_get_user_project_roles_matrix()
        .returning(move |_, _| Ok(relationships.clone()));

    let use_case = Arc::new(ProjectUserMatrixUseCase::new(
        Arc::new(mock_project_service),
        Arc::new(mock_user_service),
    ));

    let app = test::init_service(
        App::new()
            .app_data(web::Data::from(use_case.clone()))
            .service(web::resource("/api/project-user-matrix").to(get_matrix))
    ).await;

    // When: 페이지네이션 파라미터와 함께 호출
    let req = test::TestRequest::get()
        .uri("/api/project-user-matrix?project_page=1&project_page_size=2&user_page=1&user_page_size=1")
        .to_request();
    let resp = test::call_service(&app, req).await;

    // Then: 페이지네이션이 올바르게 적용되어야 함
    assert!(resp.status().is_success());
    
    let response_body: ProjectUserMatrixResponse = test::read_body_json(resp).await;
    
    assert_eq!(response_body.matrix.len(), 2);
    assert_eq!(response_body.users.len(), 1);
    assert_eq!(response_body.pagination.project_page, 1);
    assert_eq!(response_body.pagination.project_page_size, 2);
    assert_eq!(response_body.pagination.user_page, 1);
    assert_eq!(response_body.pagination.user_page_size, 1);
}

/// 에러 처리 통합 테스트
#[actix_web::test]
async fn test_matrix_error_handling_integration() {
    // Given: Mock 서비스가 에러를 반환하도록 설정
    let mut mock_project_service = MockProjectService::new();
    let mut mock_user_service = MockUserService::new();

    mock_project_service
        .expect_get_projects_with_status_filter()
        .returning(|_, _, _, _| Err(ServiceError::InternalError("Database connection failed".to_string())));

    let use_case = Arc::new(ProjectUserMatrixUseCase::new(
        Arc::new(mock_project_service),
        Arc::new(mock_user_service),
    ));

    let app = test::init_service(
        App::new()
            .app_data(web::Data::from(use_case.clone()))
            .service(web::resource("/api/project-user-matrix").to(get_matrix))
    ).await;

    // When: 매트릭스 API 호출
    let req = test::TestRequest::get()
        .uri("/api/project-user-matrix")
        .to_request();
    let resp = test::call_service(&app, req).await;

    // Then: 500 Internal Server Error가 반환되어야 함
    assert!(resp.status().is_internal_server_error());
}

/// 복합 필터링 통합 테스트
#[actix_web::test]
async fn test_matrix_complex_filtering_integration() {
    // Given: Mock 서비스 설정
    let mut mock_project_service = MockProjectService::new();
    let mut mock_user_service = MockUserService::new();

    let projects = vec![
        Project {
            id: 1,
            name: "Filtered Project".to_string(),
            description: Some("Description".to_string()),
            is_active: true,
            status: ProjectStatus::InProgress,
            created_at: chrono::Utc::now(),
        },
    ];

    let users = vec![
        User {
            id: 1,
            keycloak_id: uuid::Uuid::new_v4(),
            username: "filtered_user".to_string(),
            email: "filtered@example.com".to_string(),
            full_name: Some("Filtered User".to_string()),
            organization: Some("Test Org".to_string()),
            department: Some("Test Dept".to_string()),
            phone: Some("010-1234-5678".to_string()),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
    ];

    let relationships = vec![
        UserProjectRoleInfo {
            project_id: 1,
            user_id: 1,
            role_id: Some(1),
            role_name: Some("Admin".to_string()),
        },
    ];

    // Mock 설정 - 복합 필터링 검증
    mock_project_service
        .expect_get_projects_with_status_filter()
        .withf(|statuses, project_ids, page, page_size| {
            statuses.as_ref().map_or(false, |s| s.contains(&ProjectStatus::InProgress)) &&
            project_ids.as_ref().map_or(false, |ids| ids.contains(&1)) &&
            *page == 1 && *page_size == 10
        })
        .returning(move |_, _, _, _| Ok((projects.clone(), 1)));
    
    mock_user_service
        .expect_get_users_with_filter()
        .withf(|user_ids, page, page_size| {
            user_ids.as_ref().map_or(false, |ids| ids.contains(&1)) &&
            *page == 1 && *page_size == 10
        })
        .returning(move |_, _, _| Ok((users.clone(), 1)));
    
    mock_project_service
        .expect_get_user_project_roles_matrix()
        .returning(move |_, _| Ok(relationships.clone()));

    let use_case = Arc::new(ProjectUserMatrixUseCase::new(
        Arc::new(mock_project_service),
        Arc::new(mock_user_service),
    ));

    let app = test::init_service(
        App::new()
            .app_data(web::Data::from(use_case.clone()))
            .service(web::resource("/api/project-user-matrix").to(get_matrix))
    ).await;

    // When: 복합 필터링 파라미터와 함께 호출
    let req = test::TestRequest::get()
        .uri("/api/project-user-matrix?project_status=IN_PROGRESS&project_ids=1&user_ids=1")
        .to_request();
    let resp = test::call_service(&app, req).await;

    // Then: 필터링된 결과가 반환되어야 함
    assert!(resp.status().is_success());
    
    let response_body: ProjectUserMatrixResponse = test::read_body_json(resp).await;
    
    assert_eq!(response_body.matrix.len(), 1);
    assert_eq!(response_body.users.len(), 1);
    assert_eq!(response_body.matrix[0].project_id, 1);
    assert_eq!(response_body.users[0].user_id, 1);
}
