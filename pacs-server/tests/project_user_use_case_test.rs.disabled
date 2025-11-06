use std::sync::Arc;
use pacs_server::application::use_cases::project_user_use_case::ProjectUserUseCase;
use pacs_server::application::dto::project_user_dto::{
    UserWithRoleResponse, ProjectWithRoleResponse, ProjectMembersResponse, UserProjectsResponse
};
use pacs_server::domain::services::{ProjectService, UserService};
use pacs_server::domain::errors::ServiceError;
use async_trait::async_trait;
use mockall::mock;

// Mock ProjectService
mock! {
    ProjectService {}

    #[async_trait]
    impl ProjectService for ProjectService {
        async fn create_project(&self, name: String, description: Option<String>) -> Result<pacs_server::domain::entities::Project, ServiceError>;
        async fn get_project(&self, id: i32) -> Result<pacs_server::domain::entities::Project, ServiceError>;
        async fn get_project_by_name(&self, name: &str) -> Result<pacs_server::domain::entities::Project, ServiceError>;
        async fn get_all_projects(&self) -> Result<Vec<pacs_server::domain::entities::Project>, ServiceError>;
        async fn get_active_projects(&self) -> Result<Vec<pacs_server::domain::entities::Project>, ServiceError>;
        async fn activate_project(&self, id: i32) -> Result<pacs_server::domain::entities::Project, ServiceError>;
        async fn deactivate_project(&self, id: i32) -> Result<pacs_server::domain::entities::Project, ServiceError>;
        async fn delete_project(&self, id: i32) -> Result<(), ServiceError>;
        async fn get_project_members(&self, project_id: i32) -> Result<Vec<pacs_server::domain::entities::User>, ServiceError>;
        async fn count_project_members(&self, project_id: i32) -> Result<i64, ServiceError>;
        async fn assign_role_to_project(&self, project_id: i32, role_id: i32) -> Result<(), ServiceError>;
        async fn remove_role_from_project(&self, project_id: i32, role_id: i32) -> Result<(), ServiceError>;
        async fn get_project_roles(&self, project_id: i32) -> Result<Vec<pacs_server::domain::entities::Role>, ServiceError>;
        async fn get_project_members_with_roles(&self, project_id: i32, page: i32, page_size: i32) -> Result<(Vec<UserWithRoleResponse>, i64), ServiceError>;
        async fn assign_user_role_in_project(&self, project_id: i32, user_id: i32, role_id: i32) -> Result<(), ServiceError>;
    }
}

// Mock UserService
mock! {
    UserService {}

    #[async_trait]
    impl UserService for UserService {
        async fn create_user(&self, username: String, email: String, keycloak_id: uuid::Uuid, full_name: Option<String>, organization: Option<String>, department: Option<String>, phone: Option<String>) -> Result<pacs_server::domain::entities::User, ServiceError>;
        async fn get_user_by_id(&self, id: i32) -> Result<pacs_server::domain::entities::User, ServiceError>;
        async fn get_user_by_keycloak_id(&self, keycloak_id: uuid::Uuid) -> Result<pacs_server::domain::entities::User, ServiceError>;
        async fn get_user_by_username(&self, username: &str) -> Result<pacs_server::domain::entities::User, ServiceError>;
        async fn update_user(&self, update_user: pacs_server::domain::entities::UpdateUser) -> Result<pacs_server::domain::entities::User, ServiceError>;
        async fn delete_user(&self, id: i32) -> Result<(), ServiceError>;
        async fn user_exists(&self, keycloak_id: uuid::Uuid) -> Result<bool, ServiceError>;
        async fn add_user_to_project(&self, user_id: i32, project_id: i32) -> Result<(), ServiceError>;
        async fn remove_user_from_project(&self, user_id: i32, project_id: i32) -> Result<(), ServiceError>;
        async fn get_user_projects(&self, user_id: i32) -> Result<Vec<pacs_server::domain::entities::Project>, ServiceError>;
        async fn is_project_member(&self, user_id: i32, project_id: i32) -> Result<bool, ServiceError>;
        async fn get_user_projects_with_roles(&self, user_id: i32, page: i32, page_size: i32) -> Result<(Vec<ProjectWithRoleResponse>, i64), ServiceError>;
    }
}

#[tokio::test]
async fn test_get_project_members_with_roles_success() {
    let mut mock_project_service = MockProjectService::new();
    let mock_user_service = MockUserService::new();
    
    let user = UserWithRoleResponse {
        user_id: 1,
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        full_name: Some("Test User".to_string()),
        role_id: Some(2),
        role_name: Some("Admin".to_string()),
        role_scope: Some("GLOBAL".to_string()),
    };
    
    mock_project_service
        .expect_get_project_members_with_roles()
        .with(mockall::predicate::eq(1), mockall::predicate::eq(1), mockall::predicate::eq(20))
        .return_once(|_, _, _| Ok((vec![user], 1)));
    
    let use_case = ProjectUserUseCase::new(
        Arc::new(mock_project_service),
        Arc::new(mock_user_service),
    );
    
    let result = use_case.get_project_members_with_roles(1, Some(1), Some(20)).await;
    
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.users.len(), 1);
    assert_eq!(response.total_count, 1);
    assert_eq!(response.page, 1);
    assert_eq!(response.page_size, 20);
    assert_eq!(response.total_pages, 1);
}

#[tokio::test]
async fn test_get_project_members_with_roles_pagination() {
    let mut mock_project_service = MockProjectService::new();
    let mock_user_service = MockUserService::new();
    
    mock_project_service
        .expect_get_project_members_with_roles()
        .with(mockall::predicate::eq(1), mockall::predicate::eq(2), mockall::predicate::eq(10))
        .return_once(|_, _, _| Ok((vec![], 0)));
    
    let use_case = ProjectUserUseCase::new(
        Arc::new(mock_project_service),
        Arc::new(mock_user_service),
    );
    
    let result = use_case.get_project_members_with_roles(1, Some(2), Some(10)).await;
    
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.users.len(), 0);
    assert_eq!(response.total_count, 0);
    assert_eq!(response.page, 2);
    assert_eq!(response.page_size, 10);
    assert_eq!(response.total_pages, 0);
}

#[tokio::test]
async fn test_get_user_projects_with_roles_success() {
    let mock_project_service = MockProjectService::new();
    let mut mock_user_service = MockUserService::new();
    
    let project = ProjectWithRoleResponse {
        project_id: 1,
        project_name: "Test Project".to_string(),
        description: Some("Test Description".to_string()),
        is_active: true,
        role_id: Some(2),
        role_name: Some("Manager".to_string()),
        role_scope: Some("PROJECT".to_string()),
    };
    
    mock_user_service
        .expect_get_user_projects_with_roles()
        .with(mockall::predicate::eq(1), mockall::predicate::eq(1), mockall::predicate::eq(20))
        .return_once(|_, _, _| Ok((vec![project], 1)));
    
    let use_case = ProjectUserUseCase::new(
        Arc::new(mock_project_service),
        Arc::new(mock_user_service),
    );
    
    let result = use_case.get_user_projects_with_roles(1, Some(1), Some(20)).await;
    
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.projects.len(), 1);
    assert_eq!(response.total_count, 1);
    assert_eq!(response.page, 1);
    assert_eq!(response.page_size, 20);
    assert_eq!(response.total_pages, 1);
}

#[tokio::test]
async fn test_assign_role_to_user_success() {
    let mut mock_project_service = MockProjectService::new();
    let mock_user_service = MockUserService::new();
    
    mock_project_service
        .expect_assign_user_role_in_project()
        .with(mockall::predicate::eq(1), mockall::predicate::eq(2), mockall::predicate::eq(3))
        .return_once(|_, _, _| Ok(()));
    
    let use_case = ProjectUserUseCase::new(
        Arc::new(mock_project_service),
        Arc::new(mock_user_service),
    );
    
    let result = use_case.assign_role_to_user(1, 2, 3).await;
    
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.user_id, 2);
    assert_eq!(response.project_id, 1);
    assert_eq!(response.role_id, 3);
    assert!(response.message.contains("assigned successfully"));
}

#[tokio::test]
async fn test_batch_assign_roles_success() {
    let mut mock_project_service = MockProjectService::new();
    let mock_user_service = MockUserService::new();
    
    // First assignment succeeds
    mock_project_service
        .expect_assign_user_role_in_project()
        .with(mockall::predicate::eq(1), mockall::predicate::eq(2), mockall::predicate::eq(3))
        .return_once(|_, _, _| Ok(()));
    
    // Second assignment fails
    mock_project_service
        .expect_assign_user_role_in_project()
        .with(mockall::predicate::eq(1), mockall::predicate::eq(4), mockall::predicate::eq(5))
        .return_once(|_, _, _| Err(ServiceError::NotFound("User not found".to_string())));
    
    let use_case = ProjectUserUseCase::new(
        Arc::new(mock_project_service),
        Arc::new(mock_user_service),
    );
    
    let assignments = vec![(2, 3), (4, 5)];
    let result = use_case.batch_assign_roles(1, assignments).await;
    
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.assigned_count, 1);
    assert_eq!(response.failed_assignments.len(), 1);
    assert_eq!(response.failed_assignments[0].user_id, 4);
    assert_eq!(response.failed_assignments[0].role_id, 5);
}

#[tokio::test]
async fn test_remove_user_role_success() {
    let mut mock_project_service = MockProjectService::new();
    let mock_user_service = MockUserService::new();
    
    // Role removal (setting role_id to 0)
    mock_project_service
        .expect_assign_user_role_in_project()
        .with(mockall::predicate::eq(1), mockall::predicate::eq(2), mockall::predicate::eq(0))
        .return_once(|_, _, _| Ok(()));
    
    let use_case = ProjectUserUseCase::new(
        Arc::new(mock_project_service),
        Arc::new(mock_user_service),
    );
    
    let result = use_case.remove_user_role(1, 2).await;
    
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.user_id, 2);
    assert_eq!(response.project_id, 1);
    assert_eq!(response.role_id, 0);
    assert!(response.message.contains("removed successfully"));
}
