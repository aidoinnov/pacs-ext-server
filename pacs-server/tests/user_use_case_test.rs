use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, Utc, TimeZone};

use pacs_server::application::dto::user_dto::{
    CreateUserRequest, UserResponse, AddProjectMemberRequest, UserProjectsResponse, ProjectSummary
};
use pacs_server::application::use_cases::UserUseCase;
use pacs_server::domain::entities::{User, Project};
use pacs_server::domain::services::UserService;
use pacs_server::domain::ServiceError;

// Mock UserService for testing
#[derive(Clone)]
struct MockUserService {
    users: std::collections::HashMap<i32, User>,
    user_projects: std::collections::HashMap<i32, Vec<Project>>,
    next_id: i32,
}

impl MockUserService {
    fn new() -> Self {
        Self {
            users: std::collections::HashMap::new(),
            user_projects: std::collections::HashMap::new(),
            next_id: 1,
        }
    }

    fn add_user(&mut self, user: User) {
        self.users.insert(user.id, user);
    }

    fn add_user_project(&mut self, user_id: i32, project: Project) {
        self.user_projects.entry(user_id).or_insert_with(Vec::new).push(project);
    }
}

#[async_trait::async_trait]
impl UserService for MockUserService {
    async fn create_user(&self, username: String, email: String, keycloak_id: Uuid) -> Result<User, ServiceError> {
        let user = User {
            id: self.next_id,
            keycloak_id,
            username,
            email,
            created_at: Utc.timestamp_opt(1640995200, 0).unwrap(),
        };
        Ok(user)
    }

    async fn get_user_by_id(&self, id: i32) -> Result<User, ServiceError> {
        self.users.get(&id)
            .ok_or(ServiceError::NotFound("User not found".into()))
            .map(|u| u.clone())
    }

    async fn get_user_by_keycloak_id(&self, keycloak_id: Uuid) -> Result<User, ServiceError> {
        self.users.values()
            .find(|u| u.keycloak_id == keycloak_id)
            .ok_or(ServiceError::NotFound("User not found".into()))
            .map(|u| u.clone())
    }

    async fn get_user_by_username(&self, username: &str) -> Result<User, ServiceError> {
        self.users.values()
            .find(|u| u.username == username)
            .ok_or(ServiceError::NotFound("User not found".into()))
            .map(|u| u.clone())
    }

    async fn delete_user(&self, id: i32) -> Result<(), ServiceError> {
        if self.users.contains_key(&id) {
            Ok(())
        } else {
            Err(ServiceError::NotFound("User not found".into()))
        }
    }

    async fn user_exists(&self, keycloak_id: Uuid) -> Result<bool, ServiceError> {
        Ok(self.users.values().any(|u| u.keycloak_id == keycloak_id))
    }

    async fn add_user_to_project(&self, user_id: i32, project_id: i32) -> Result<(), ServiceError> {
        if !self.users.contains_key(&user_id) {
            return Err(ServiceError::NotFound("User not found".into()));
        }
        Ok(())
    }

    async fn remove_user_from_project(&self, user_id: i32, project_id: i32) -> Result<(), ServiceError> {
        if !self.users.contains_key(&user_id) {
            return Err(ServiceError::NotFound("User not found".into()));
        }
        Ok(())
    }

    async fn get_user_projects(&self, user_id: i32) -> Result<Vec<Project>, ServiceError> {
        Ok(self.user_projects.get(&user_id).cloned().unwrap_or_default())
    }

    async fn is_project_member(&self, user_id: i32, project_id: i32) -> Result<bool, ServiceError> {
        Ok(self.user_projects.get(&user_id)
            .map(|projects| projects.iter().any(|p| p.id == project_id))
            .unwrap_or(false))
    }
}

fn create_test_user() -> User {
    User {
        id: 1,
        keycloak_id: Uuid::new_v4(),
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        created_at: Utc.timestamp_opt(1640995200, 0).unwrap(),
    }
}

fn create_test_project() -> Project {
    Project {
        id: 1,
        name: "Test Project".to_string(),
        description: Some("Test Description".to_string()),
        is_active: true,
        created_at: Utc.timestamp_opt(1640995200, 0).unwrap(),
    }
}

#[tokio::test]
async fn test_user_use_case_create_user_success() {
    let mock_user_service = MockUserService::new();
    let user_use_case = UserUseCase::new(mock_user_service);

    let create_request = CreateUserRequest {
        username: "newuser".to_string(),
        email: "newuser@example.com".to_string(),
        keycloak_id: Uuid::new_v4(),
    };

    let result = user_use_case.create_user(create_request).await;
    assert!(result.is_ok());

    let user_response = result.unwrap();
    assert_eq!(user_response.username, "newuser");
    assert_eq!(user_response.email, "newuser@example.com");
    assert!(!user_response.keycloak_id.is_nil());
    assert!(user_response.id > 0);
}

#[tokio::test]
async fn test_user_use_case_get_user_by_id_success() {
    let mut mock_user_service = MockUserService::new();
    let user = create_test_user();
    mock_user_service.add_user(user.clone());
    
    let user_use_case = UserUseCase::new(mock_user_service);

    let result = user_use_case.get_user_by_id(user.id).await;
    assert!(result.is_ok());

    let user_response = result.unwrap();
    assert_eq!(user_response.id, user.id);
    assert_eq!(user_response.username, user.username);
    assert_eq!(user_response.email, user.email);
    assert_eq!(user_response.keycloak_id, user.keycloak_id);
}

#[tokio::test]
async fn test_user_use_case_get_user_by_id_not_found() {
    let mock_user_service = MockUserService::new();
    let user_use_case = UserUseCase::new(mock_user_service);

    let result = user_use_case.get_user_by_id(999).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_user_use_case_get_user_by_username_success() {
    let mut mock_user_service = MockUserService::new();
    let user = create_test_user();
    mock_user_service.add_user(user.clone());
    
    let user_use_case = UserUseCase::new(mock_user_service);

    let result = user_use_case.get_user_by_username(&user.username).await;
    assert!(result.is_ok());

    let user_response = result.unwrap();
    assert_eq!(user_response.username, user.username);
    assert_eq!(user_response.email, user.email);
}

#[tokio::test]
async fn test_user_use_case_get_user_by_username_not_found() {
    let mock_user_service = MockUserService::new();
    let user_use_case = UserUseCase::new(mock_user_service);

    let result = user_use_case.get_user_by_username("nonexistent").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_user_use_case_delete_user_success() {
    let mut mock_user_service = MockUserService::new();
    let user = create_test_user();
    mock_user_service.add_user(user.clone());
    
    let user_use_case = UserUseCase::new(mock_user_service);

    let result = user_use_case.delete_user(user.id).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_user_use_case_delete_user_not_found() {
    let mock_user_service = MockUserService::new();
    let user_use_case = UserUseCase::new(mock_user_service);

    let result = user_use_case.delete_user(999).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_user_use_case_add_project_member_success() {
    let mut mock_user_service = MockUserService::new();
    // Add a user to the mock service
    let user = create_test_user();
    mock_user_service.users.insert(user.id, user);
    
    let user_use_case = UserUseCase::new(mock_user_service);

    let add_member_request = AddProjectMemberRequest {
        user_id: 1,
        project_id: 1,
    };

    let result = user_use_case.add_project_member(add_member_request).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_user_use_case_add_project_member_user_not_found() {
    let mock_user_service = MockUserService::new();
    let user_use_case = UserUseCase::new(mock_user_service);

    let add_member_request = AddProjectMemberRequest {
        user_id: 999,
        project_id: 1,
    };

    let result = user_use_case.add_project_member(add_member_request).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_user_use_case_get_user_projects_success() {
    let mut mock_user_service = MockUserService::new();
    let user = create_test_user();
    let project = create_test_project();
    mock_user_service.add_user(user.clone());
    mock_user_service.add_user_project(user.id, project.clone());
    
    let user_use_case = UserUseCase::new(mock_user_service);

    let result = user_use_case.get_user_projects(user.id).await;
    assert!(result.is_ok());

    let projects_response = result.unwrap();
    assert_eq!(projects_response.projects.len(), 1);
    assert_eq!(projects_response.projects[0].id, project.id);
    assert_eq!(projects_response.projects[0].name, project.name);
}

#[tokio::test]
async fn test_user_use_case_get_user_projects_empty() {
    let mut mock_user_service = MockUserService::new();
    let user = create_test_user();
    mock_user_service.add_user(user.clone());
    
    let user_use_case = UserUseCase::new(mock_user_service);

    let result = user_use_case.get_user_projects(user.id).await;
    assert!(result.is_ok());

    let projects_response = result.unwrap();
    assert_eq!(projects_response.projects.len(), 0);
}

#[tokio::test]
async fn test_user_response_structure() {
    let mut mock_user_service = MockUserService::new();
    let user = create_test_user();
    mock_user_service.add_user(user.clone());
    
    let user_use_case = UserUseCase::new(mock_user_service);

    let user_response = user_use_case.get_user_by_id(user.id).await.unwrap();
    
    // Verify all required fields are present
    assert!(user_response.id > 0);
    assert!(!user_response.keycloak_id.is_nil());
    assert!(!user_response.username.is_empty());
    assert!(!user_response.email.is_empty());
    assert!(user_response.created_at.timestamp() > 0);
}

#[tokio::test]
async fn test_project_summary_structure() {
    let mut mock_user_service = MockUserService::new();
    let user = create_test_user();
    let project = create_test_project();
    mock_user_service.add_user(user.clone());
    mock_user_service.add_user_project(user.id, project.clone());
    
    let user_use_case = UserUseCase::new(mock_user_service);

    let projects_response = user_use_case.get_user_projects(user.id).await.unwrap();
    let project_summary = &projects_response.projects[0];
    
    // Verify all required fields are present
    assert!(project_summary.id > 0);
    assert!(!project_summary.name.is_empty());
    assert!(project_summary.is_active);
}
