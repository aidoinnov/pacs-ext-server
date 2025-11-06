use std::sync::Arc;
use chrono::{DateTime, Utc};

use pacs_server::application::dto::project_dto::{
    CreateProjectRequest, ProjectResponse, ProjectListResponse, ProjectAssignRoleRequest,
    ProjectMembersResponse, MemberInfo, ProjectRolesResponse, RoleInfo
};
use pacs_server::application::use_cases::ProjectUseCase;
use pacs_server::domain::entities::{Project, User, Role, RoleScope};
use pacs_server::domain::services::ProjectService;
use pacs_server::domain::ServiceError;

// Mock ProjectService for testing
#[derive(Clone)]
struct MockProjectService {
    projects: std::collections::HashMap<i32, Project>,
    project_members: std::collections::HashMap<i32, Vec<User>>,
    project_roles: std::collections::HashMap<i32, Vec<Role>>,
    next_id: i32,
}

impl MockProjectService {
    fn new() -> Self {
        Self {
            projects: std::collections::HashMap::new(),
            project_members: std::collections::HashMap::new(),
            project_roles: std::collections::HashMap::new(),
            next_id: 1,
        }
    }

    fn add_project(&mut self, project: Project) {
        self.projects.insert(project.id, project);
    }

    fn add_project_member(&mut self, project_id: i32, user: User) {
        self.project_members.entry(project_id).or_insert_with(Vec::new).push(user);
    }

    fn add_project_role(&mut self, project_id: i32, role: Role) {
        self.project_roles.entry(project_id).or_insert_with(Vec::new).push(role);
    }
}

#[async_trait::async_trait]
impl ProjectService for MockProjectService {
    async fn create_project(&self, name: String, description: Option<String>) -> Result<Project, ServiceError> {
        let project = Project {
            id: self.next_id,
            name,
            description,
            is_active: true,
            created_at: Utc::timestamp_opt(1640995200, 0).unwrap(),
        };
        Ok(project)
    }

    async fn get_project(&self, id: i32) -> Result<Project, ServiceError> {
        self.projects.get(&id)
            .ok_or(ServiceError::NotFound("Project not found".into()))
            .map(|p| p.clone())
    }

    async fn get_project_by_name(&self, name: &str) -> Result<Project, ServiceError> {
        self.projects.values()
            .find(|p| p.name == name)
            .ok_or(ServiceError::NotFound("Project not found".into()))
            .map(|p| p.clone())
    }

    async fn get_all_projects(&self) -> Result<Vec<Project>, ServiceError> {
        Ok(self.projects.values().cloned().collect())
    }

    async fn get_active_projects(&self) -> Result<Vec<Project>, ServiceError> {
        Ok(self.projects.values().filter(|p| p.is_active).cloned().collect())
    }

    async fn activate_project(&self, id: i32) -> Result<Project, ServiceError> {
        self.projects.get(&id)
            .ok_or(ServiceError::NotFound("Project not found".into()))
            .map(|p| Project { is_active: true, ..p.clone() })
    }

    async fn deactivate_project(&self, id: i32) -> Result<Project, ServiceError> {
        self.projects.get(&id)
            .ok_or(ServiceError::NotFound("Project not found".into()))
            .map(|p| Project { is_active: false, ..p.clone() })
    }

    async fn delete_project(&self, id: i32) -> Result<(), ServiceError> {
        if self.projects.contains_key(&id) {
            Ok(())
        } else {
            Err(ServiceError::NotFound("Project not found".into()))
        }
    }

    async fn get_project_members(&self, project_id: i32) -> Result<Vec<User>, ServiceError> {
        Ok(self.project_members.get(&project_id).cloned().unwrap_or_default())
    }

    async fn count_project_members(&self, project_id: i32) -> Result<i64, ServiceError> {
        Ok(self.project_members.get(&project_id).map(|m| m.len() as i64).unwrap_or(0))
    }

    async fn assign_role_to_project(&self, project_id: i32, role_id: i32) -> Result<(), ServiceError> {
        if !self.projects.contains_key(&project_id) {
            return Err(ServiceError::NotFound("Project not found".into()));
        }
        Ok(())
    }

    async fn remove_role_from_project(&self, project_id: i32, role_id: i32) -> Result<(), ServiceError> {
        if !self.projects.contains_key(&project_id) {
            return Err(ServiceError::NotFound("Project not found".into()));
        }
        Ok(())
    }

    async fn get_project_roles(&self, project_id: i32) -> Result<Vec<Role>, ServiceError> {
        Ok(self.project_roles.get(&project_id).cloned().unwrap_or_default())
    }
}

fn create_test_project() -> Project {
    Project {
        id: 1,
        name: "Test Project".to_string(),
        description: Some("Test Description".to_string()),
        is_active: true,
        created_at: Utc::timestamp_opt(1640995200, 0).unwrap(),
    }
}

fn create_test_user() -> User {
    User {
        id: 1,
        keycloak_id: uuid::Uuid::new_v4(),
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        created_at: Utc::timestamp_opt(1640995200, 0).unwrap(),
    }
}

fn create_test_role() -> Role {
    Role {
        id: 1,
        name: "Admin".to_string(),
        description: Some("Administrator".to_string()),
        scope: "PROJECT".to_string(),
        created_at: Utc::timestamp_opt(1640995200, 0).unwrap(),
    }
}

#[tokio::test]
async fn test_project_use_case_create_project_success() {
    let mock_project_service = MockProjectService::new();
    let project_use_case = ProjectUseCase::new(mock_project_service);

    let create_request = CreateProjectRequest {
        name: "New Project".to_string(),
        description: Some("New Description".to_string()),
    };

    let result = project_use_case.create_project(create_request).await;
    assert!(result.is_ok());

    let project_response = result.unwrap();
    assert_eq!(project_response.name, "New Project");
    assert_eq!(project_response.description, Some("New Description".to_string()));
    assert!(project_response.is_active);
    assert!(project_response.id > 0);
}

#[tokio::test]
async fn test_project_use_case_create_project_without_description() {
    let mock_project_service = MockProjectService::new();
    let project_use_case = ProjectUseCase::new(mock_project_service);

    let create_request = CreateProjectRequest {
        name: "Simple Project".to_string(),
        description: None,
    };

    let result = project_use_case.create_project(create_request).await;
    assert!(result.is_ok());

    let project_response = result.unwrap();
    assert_eq!(project_response.name, "Simple Project");
    assert_eq!(project_response.description, None);
    assert!(project_response.is_active);
}

#[tokio::test]
async fn test_project_use_case_get_project_success() {
    let mut mock_project_service = MockProjectService::new();
    let project = create_test_project();
    mock_project_service.add_project(project.clone());
    
    let project_use_case = ProjectUseCase::new(mock_project_service);

    let result = project_use_case.get_project(project.id).await;
    assert!(result.is_ok());

    let project_response = result.unwrap();
    assert_eq!(project_response.id, project.id);
    assert_eq!(project_response.name, project.name);
    assert_eq!(project_response.description, project.description);
    assert_eq!(project_response.is_active, project.is_active);
}

#[tokio::test]
async fn test_project_use_case_get_project_not_found() {
    let mock_project_service = MockProjectService::new();
    let project_use_case = ProjectUseCase::new(mock_project_service);

    let result = project_use_case.get_project(999).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_project_use_case_get_all_projects_success() {
    let mut mock_project_service = MockProjectService::new();
    let project1 = create_test_project();
    let project2 = Project {
        id: 2,
        name: "Second Project".to_string(),
        description: None,
        is_active: true,
        created_at: Utc::timestamp_opt(1640995200, 0).unwrap(),
    };
    mock_project_service.add_project(project1);
    mock_project_service.add_project(project2);
    
    let project_use_case = ProjectUseCase::new(mock_project_service);

    let result = project_use_case.get_all_projects().await;
    assert!(result.is_ok());

    let projects_response = result.unwrap();
    assert_eq!(projects_response.projects.len(), 2);
    assert_eq!(projects_response.total, 2);
}

#[tokio::test]
async fn test_project_use_case_get_all_projects_empty() {
    let mock_project_service = MockProjectService::new();
    let project_use_case = ProjectUseCase::new(mock_project_service);

    let result = project_use_case.get_all_projects().await;
    assert!(result.is_ok());

    let projects_response = result.unwrap();
    assert_eq!(projects_response.projects.len(), 0);
    assert_eq!(projects_response.total, 0);
}

#[tokio::test]
async fn test_project_use_case_assign_role_success() {
    let mut mock_project_service = MockProjectService::new();
    let project = create_test_project();
    mock_project_service.add_project(project.clone());
    
    let project_use_case = ProjectUseCase::new(mock_project_service);

        let assign_request = ProjectAssignRoleRequest {
            role_id: 1,
        };

    let result = project_use_case.assign_role(project.id, assign_request).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_project_use_case_assign_role_project_not_found() {
    let mock_project_service = MockProjectService::new();
    let project_use_case = ProjectUseCase::new(mock_project_service);

    let assign_request = ProjectAssignRoleRequest {
        role_id: 1,
    };

    let result = project_use_case.assign_role(999, assign_request).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_project_use_case_get_project_members_success() {
    let mut mock_project_service = MockProjectService::new();
    let project = create_test_project();
    let user1 = create_test_user();
    let user2 = User {
        id: 2,
        keycloak_id: uuid::Uuid::new_v4(),
        username: "user2".to_string(),
        email: "user2@example.com".to_string(),
        created_at: Utc::timestamp_opt(1640995200, 0).unwrap(),
    };
    
    mock_project_service.add_project(project.clone());
    mock_project_service.add_project_member(project.id, user1.clone());
    mock_project_service.add_project_member(project.id, user2.clone());
    
    let project_use_case = ProjectUseCase::new(mock_project_service);

    let result = project_use_case.get_project_members(project.id).await;
    assert!(result.is_ok());

    let members_response = result.unwrap();
    assert_eq!(members_response.members.len(), 2);
    assert_eq!(members_response.total, 2);
}

#[tokio::test]
async fn test_project_use_case_get_project_roles_success() {
    let mut mock_project_service = MockProjectService::new();
    let project = create_test_project();
    let role = create_test_role();
    
    mock_project_service.add_project(project.clone());
    mock_project_service.add_project_role(project.id, role.clone());
    
    let project_use_case = ProjectUseCase::new(mock_project_service);

    let result = project_use_case.get_project_roles(project.id).await;
    assert!(result.is_ok());

    let roles_response = result.unwrap();
    assert_eq!(roles_response.roles.len(), 1);
    assert_eq!(roles_response.roles[0].id, role.id);
    assert_eq!(roles_response.roles[0].name, role.name);
}

#[tokio::test]
async fn test_project_response_structure() {
    let mut mock_project_service = MockProjectService::new();
    let project = create_test_project();
    mock_project_service.add_project(project.clone());
    
    let project_use_case = ProjectUseCase::new(mock_project_service);

    let project_response = project_use_case.get_project(project.id).await.unwrap();
    
    // Verify all required fields are present
    assert!(project_response.id > 0);
    assert!(!project_response.name.is_empty());
    assert!(project_response.created_at.timestamp() > 0);
}

#[tokio::test]
async fn test_member_info_structure() {
    let mut mock_project_service = MockProjectService::new();
    let project = create_test_project();
    let user = create_test_user();
    
    mock_project_service.add_project(project.clone());
    mock_project_service.add_project_member(project.id, user.clone());
    
    let project_use_case = ProjectUseCase::new(mock_project_service);

    let members_response = project_use_case.get_project_members(project.id).await.unwrap();
    let member_info = &members_response.members[0];
    
    // Verify all required fields are present
    assert!(member_info.id > 0);
    assert!(!member_info.username.is_empty());
    assert!(!member_info.email.is_empty());
    assert!(member_info.joined_at.timestamp() > 0);
}

#[tokio::test]
async fn test_role_info_structure() {
    let mut mock_project_service = MockProjectService::new();
    let project = create_test_project();
    let role = create_test_role();
    
    mock_project_service.add_project(project.clone());
    mock_project_service.add_project_role(project.id, role.clone());
    
    let project_use_case = ProjectUseCase::new(mock_project_service);

    let roles_response = project_use_case.get_project_roles(project.id).await.unwrap();
    let role_info = &roles_response.roles[0];
    
    // Verify all required fields are present
    assert!(role_info.id > 0);
    assert!(!role_info.name.is_empty());
}
