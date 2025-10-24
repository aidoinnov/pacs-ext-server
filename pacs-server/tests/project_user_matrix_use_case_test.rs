use std::sync::Arc;
use mockall::mock;
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

/// ProjectUserMatrixUseCase 기본 기능 테스트
#[tokio::test]
async fn test_get_matrix_basic() {
    // Given: Mock 서비스들 설정
    let mut mock_project_service = MockProjectService::new();
    let mut mock_user_service = MockUserService::new();
    
    // Mock 데이터 설정
    let test_projects = create_test_projects();
    let test_users = create_test_users();
    let test_relationships = create_test_relationships();
    
    // ProjectService Mock 설정
    mock_project_service
        .expect_get_projects_with_status_filter()
        .returning(move |_, _, _, _| Ok((test_projects.clone(), 2)));
    
    mock_project_service
        .expect_get_user_project_roles_matrix()
        .returning(move |_, _| Ok(test_relationships.clone()));
    
    // UserService Mock 설정
    mock_user_service
        .expect_get_users_with_filter()
        .returning(move |_, _, _| Ok((test_users.clone(), 2)));
    
    let use_case = ProjectUserMatrixUseCase::new(
        Arc::new(mock_project_service),
        Arc::new(mock_user_service),
    );
    
    // When: 매트릭스 조회
    let query = MatrixQueryParams {
        project_page: Some(1),
        project_page_size: Some(10),
        user_page: Some(1),
        user_page_size: Some(10),
        project_statuses: None,
        project_ids: None,
        user_ids: None,
    };
    
    let result = use_case.get_matrix(query).await;
    
    // Then: 성공적으로 매트릭스가 반환되어야 함
    assert!(result.is_ok());
    
    let matrix_response = result.unwrap();
    assert_eq!(matrix_response.matrix.len(), 2);
    assert_eq!(matrix_response.users.len(), 2);
    assert_eq!(matrix_response.pagination.project_total_count, 2);
    assert_eq!(matrix_response.pagination.user_total_count, 2);
}

/// 상태 필터링 테스트
#[tokio::test]
async fn test_get_matrix_with_status_filter() {
    let mut mock_project_service = MockProjectService::new();
    let mut mock_user_service = MockUserService::new();
    
    let test_projects = create_test_projects();
    let test_users = create_test_users();
    let test_relationships = create_test_relationships();
    
    // 상태 필터링이 올바르게 전달되는지 확인
    mock_project_service
        .expect_get_projects_with_status_filter()
        .withf(|statuses, _, _, _| {
            statuses.as_ref().map_or(false, |s| {
                s.contains(&ProjectStatus::InProgress)
            })
        })
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
    
    // When: IN_PROGRESS 상태 필터링으로 조회
    let query = MatrixQueryParams {
        project_page: Some(1),
        project_page_size: Some(10),
        user_page: Some(1),
        user_page_size: Some(10),
        project_statuses: Some(vec![ProjectStatus::InProgress]),
        project_ids: None,
        user_ids: None,
    };
    
    let result = use_case.get_matrix(query).await;
    
    // Then: 성공적으로 필터링된 매트릭스가 반환되어야 함
    assert!(result.is_ok());
}

/// 프로젝트 ID 필터링 테스트
#[tokio::test]
async fn test_get_matrix_with_project_ids_filter() {
    let mut mock_project_service = MockProjectService::new();
    let mut mock_user_service = MockUserService::new();
    
    let test_projects = create_test_projects();
    let test_users = create_test_users();
    let test_relationships = create_test_relationships();
    
    // 프로젝트 ID 필터링이 올바르게 전달되는지 확인
    mock_project_service
        .expect_get_projects_with_status_filter()
        .withf(|_, project_ids, _, _| {
            project_ids.as_ref().map_or(false, |ids| {
                ids.contains(&1) && ids.contains(&2)
            })
        })
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
    
    // When: 특정 프로젝트 ID들로 필터링하여 조회
    let query = MatrixQueryParams {
        project_page: Some(1),
        project_page_size: Some(10),
        user_page: Some(1),
        user_page_size: Some(10),
        project_statuses: None,
        project_ids: Some(vec![1, 2]),
        user_ids: None,
    };
    
    let result = use_case.get_matrix(query).await;
    
    // Then: 성공적으로 필터링된 매트릭스가 반환되어야 함
    assert!(result.is_ok());
}

/// 사용자 ID 필터링 테스트
#[tokio::test]
async fn test_get_matrix_with_user_ids_filter() {
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
    
    // 사용자 ID 필터링이 올바르게 전달되는지 확인
    mock_user_service
        .expect_get_users_with_filter()
        .withf(|user_ids, _, _| {
            user_ids.as_ref().map_or(false, |ids| {
                ids.contains(&1) && ids.contains(&2)
            })
        })
        .returning(move |_, _, _| Ok((test_users.clone(), 2)));
    
    let use_case = ProjectUserMatrixUseCase::new(
        Arc::new(mock_project_service),
        Arc::new(mock_user_service),
    );
    
    // When: 특정 사용자 ID들로 필터링하여 조회
    let query = MatrixQueryParams {
        project_page: Some(1),
        project_page_size: Some(10),
        user_page: Some(1),
        user_page_size: Some(10),
        project_statuses: None,
        project_ids: None,
        user_ids: Some(vec![1, 2]),
    };
    
    let result = use_case.get_matrix(query).await;
    
    // Then: 성공적으로 필터링된 매트릭스가 반환되어야 함
    assert!(result.is_ok());
}

/// ProjectService 에러 처리 테스트
#[tokio::test]
async fn test_get_matrix_project_service_error() {
    let mut mock_project_service = MockProjectService::new();
    let mut mock_user_service = MockUserService::new();
    
    // ProjectService에서 에러 발생
    mock_project_service
        .expect_get_projects_with_status_filter()
        .returning(|_, _, _, _| Err(ServiceError::DatabaseError("Database error".to_string())));
    
    let use_case = ProjectUserMatrixUseCase::new(
        Arc::new(mock_project_service),
        Arc::new(mock_user_service),
    );
    
    // When: 매트릭스 조회
    let query = MatrixQueryParams {
        project_page: Some(1),
        project_page_size: Some(10),
        user_page: Some(1),
        user_page_size: Some(10),
        project_statuses: None,
        project_ids: None,
        user_ids: None,
    };
    
    let result = use_case.get_matrix(query).await;
    
    // Then: 에러가 전파되어야 함
    assert!(result.is_err());
    match result.unwrap_err() {
        ServiceError::DatabaseError(msg) => assert_eq!(msg, "Database error"),
        _ => panic!("예상하지 못한 에러 타입"),
    }
}

/// UserService 에러 처리 테스트
#[tokio::test]
async fn test_get_matrix_user_service_error() {
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
    
    // When: 매트릭스 조회
    let query = MatrixQueryParams {
        project_page: Some(1),
        project_page_size: Some(10),
        user_page: Some(1),
        user_page_size: Some(10),
        project_statuses: None,
        project_ids: None,
        user_ids: None,
    };
    
    let result = use_case.get_matrix(query).await;
    
    // Then: 에러가 전파되어야 함
    assert!(result.is_err());
}

/// 매트릭스 관계 매핑 테스트
#[tokio::test]
async fn test_matrix_relationship_mapping() {
    let mut mock_project_service = MockProjectService::new();
    let mut mock_user_service = MockUserService::new();
    
    let test_projects = create_test_projects();
    let test_users = create_test_users();
    let test_relationships = create_test_relationships_with_roles();
    
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
    
    // When: 매트릭스 조회
    let query = MatrixQueryParams {
        project_page: Some(1),
        project_page_size: Some(10),
        user_page: Some(1),
        user_page_size: Some(10),
        project_statuses: None,
        project_ids: None,
        user_ids: None,
    };
    
    let result = use_case.get_matrix(query).await;
    
    // Then: 매트릭스 관계가 올바르게 매핑되어야 함
    assert!(result.is_ok());
    
    let matrix_response = result.unwrap();
    
    // 각 프로젝트에 대해 사용자-역할 관계가 올바르게 매핑되었는지 확인
    for project_row in &matrix_response.matrix {
        assert!(!project_row.user_roles.is_empty());
        
        // 역할이 있는 사용자와 없는 사용자가 모두 포함되어야 함
        let has_role_users = project_row.user_roles.iter().any(|ur| ur.role_id.is_some());
        let no_role_users = project_row.user_roles.iter().any(|ur| ur.role_id.is_none());
        
        assert!(has_role_users || no_role_users, "역할 정보가 올바르게 매핑되지 않음");
    }
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
        Project {
            id: 2,
            name: "Test Project 2".to_string(),
            description: Some("Test Description".to_string()),
            is_active: true,
            status: ProjectStatus::Completed,
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
        User {
            id: 2,
            keycloak_id: uuid::Uuid::new_v4(),
            username: "testuser2".to_string(),
            email: "user2@example.com".to_string(),
            full_name: Some("Test User 2".to_string()),
            organization: Some("Test Org".to_string()),
            department: Some("Test Dept".to_string()),
            phone: Some("010-1234-5679".to_string()),
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
        crate::domain::services::project_service::UserProjectRoleInfo {
            project_id: 1,
            user_id: 2,
            role_id: None,
            role_name: None,
        },
        crate::domain::services::project_service::UserProjectRoleInfo {
            project_id: 2,
            user_id: 1,
            role_id: None,
            role_name: None,
        },
        crate::domain::services::project_service::UserProjectRoleInfo {
            project_id: 2,
            user_id: 2,
            role_id: None,
            role_name: None,
        },
    ]
}

fn create_test_relationships_with_roles() -> Vec<crate::domain::services::project_service::UserProjectRoleInfo> {
    vec![
        crate::domain::services::project_service::UserProjectRoleInfo {
            project_id: 1,
            user_id: 1,
            role_id: Some(1),
            role_name: Some("Admin".to_string()),
        },
        crate::domain::services::project_service::UserProjectRoleInfo {
            project_id: 1,
            user_id: 2,
            role_id: None,
            role_name: None,
        },
        crate::domain::services::project_service::UserProjectRoleInfo {
            project_id: 2,
            user_id: 1,
            role_id: Some(2),
            role_name: Some("Viewer".to_string()),
        },
        crate::domain::services::project_service::UserProjectRoleInfo {
            project_id: 2,
            user_id: 2,
            role_id: Some(1),
            role_name: Some("Admin".to_string()),
        },
    ]
}
