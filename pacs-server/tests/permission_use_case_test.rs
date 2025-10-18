use std::collections::HashMap;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

use pacs_server::application::use_cases::PermissionUseCase;
use pacs_server::application::dto::{
    CreateRoleRequest, RoleResponse, PermissionResponse, AssignPermissionRequest,
    RolePermissionsResponse, ProjectPermissionsResponse, ResourcePermissionsResponse,
};
use pacs_server::domain::entities::{Role, Permission, RoleScope};
use pacs_server::domain::services::PermissionService;
use pacs_server::domain::ServiceError;

// Mock PermissionService for testing
#[derive(Clone)]
struct MockPermissionService {
    roles: HashMap<i32, Role>,
    permissions: HashMap<i32, Permission>,
    role_permissions: HashMap<i32, Vec<i32>>, // role_id -> permission_ids
    project_permissions: HashMap<i32, Vec<i32>>, // project_id -> permission_ids
}

impl MockPermissionService {
    fn new() -> Self {
        Self {
            roles: HashMap::new(),
            permissions: HashMap::new(),
            role_permissions: HashMap::new(),
            project_permissions: HashMap::new(),
        }
    }

    fn add_role(&mut self, role: Role) {
        self.roles.insert(role.id, role);
    }

    fn add_permission(&mut self, permission: Permission) {
        self.permissions.insert(permission.id, permission);
    }

    fn assign_permission_to_role(&mut self, role_id: i32, permission_id: i32) {
        self.role_permissions.entry(role_id).or_insert_with(Vec::new).push(permission_id);
    }

    fn assign_permission_to_project(&mut self, project_id: i32, permission_id: i32) {
        self.project_permissions.entry(project_id).or_insert_with(Vec::new).push(permission_id);
    }
}

#[async_trait]
impl PermissionService for MockPermissionService {
    async fn get_permissions_for_resource(&self, resource_type: &str) -> Result<Vec<Permission>, ServiceError> {
        let permissions: Vec<Permission> = self.permissions
            .values()
            .filter(|p| p.resource_type == resource_type)
            .cloned()
            .collect();
        Ok(permissions)
    }

    async fn validate_permission_exists(&self, resource_type: &str, action: &str) -> Result<bool, ServiceError> {
        let exists = self.permissions
            .values()
            .any(|p| p.resource_type == resource_type && p.action == action);
        Ok(exists)
    }

    async fn get_roles_by_scope(&self, scope: RoleScope) -> Result<Vec<Role>, ServiceError> {
        let scope_str = match scope {
            RoleScope::Global => "GLOBAL",
            RoleScope::Project => "PROJECT",
        };
        let roles: Vec<Role> = self.roles
            .values()
            .filter(|r| r.scope == scope_str)
            .cloned()
            .collect();
        Ok(roles)
    }

    async fn get_global_roles(&self) -> Result<Vec<Role>, ServiceError> {
        self.get_roles_by_scope(RoleScope::Global).await
    }

    async fn get_project_roles(&self) -> Result<Vec<Role>, ServiceError> {
        self.get_roles_by_scope(RoleScope::Project).await
    }

    async fn create_role(&self, name: String, scope: RoleScope, description: Option<String>) -> Result<Role, ServiceError> {
        let scope_str = match scope {
            RoleScope::Global => "GLOBAL",
            RoleScope::Project => "PROJECT",
        };
        let role = Role {
            id: (self.roles.len() + 1) as i32,
            name,
            description,
            scope: scope_str.to_string(),
            created_at: chrono::Utc::now().naive_utc(),
        };
        Ok(role)
    }

    async fn get_role(&self, role_id: i32) -> Result<Role, ServiceError> {
        self.roles.get(&role_id)
            .cloned()
            .ok_or_else(|| ServiceError::NotFound("Role not found".to_string()))
    }

    async fn assign_permission_to_role(&self, role_id: i32, permission_id: i32) -> Result<(), ServiceError> {
        if !self.roles.contains_key(&role_id) {
            return Err(ServiceError::NotFound("Role not found".to_string()));
        }
        if !self.permissions.contains_key(&permission_id) {
            return Err(ServiceError::NotFound("Permission not found".to_string()));
        }
        // Note: In a real implementation, this would update the database
        // For testing, we'll just return Ok since the mock doesn't persist changes
        Ok(())
    }

    async fn remove_permission_from_role(&self, role_id: i32, permission_id: i32) -> Result<(), ServiceError> {
        if !self.roles.contains_key(&role_id) {
            return Err(ServiceError::NotFound("Role not found".to_string()));
        }
        Ok(())
    }

    async fn get_role_permissions(&self, role_id: i32) -> Result<Vec<Permission>, ServiceError> {
        if !self.roles.contains_key(&role_id) {
            return Err(ServiceError::NotFound("Role not found".to_string()));
        }
        
        // For testing, return all permissions if role exists
        // In a real implementation, this would query the database for role-specific permissions
        let permissions: Vec<Permission> = self.permissions.values().cloned().collect();
        Ok(permissions)
    }

    async fn assign_permission_to_project(&self, project_id: i32, permission_id: i32) -> Result<(), ServiceError> {
        if !self.permissions.contains_key(&permission_id) {
            return Err(ServiceError::NotFound("Permission not found".to_string()));
        }
        Ok(())
    }

    async fn remove_permission_from_project(&self, project_id: i32, permission_id: i32) -> Result<(), ServiceError> {
        Ok(())
    }

    async fn get_project_permissions(&self, project_id: i32) -> Result<Vec<Permission>, ServiceError> {
        // For testing, return all permissions for any project
        // In a real implementation, this would query the database for project-specific permissions
        let permissions: Vec<Permission> = self.permissions.values().cloned().collect();
        Ok(permissions)
    }
}

fn create_test_role() -> Role {
    Role {
        id: 1,
        name: "Test Role".to_string(),
        description: Some("Test role description".to_string()),
        scope: "PROJECT".to_string(),
        created_at: chrono::Utc::now().naive_utc(),
    }
}

fn create_test_permission() -> Permission {
    Permission {
        id: 1,
        resource_type: "annotation".to_string(),
        action: "read".to_string(),
    }
}

#[tokio::test]
async fn test_permission_use_case_create_role_success() {
    let mock_permission_service = MockPermissionService::new();
    let permission_use_case = PermissionUseCase::new(mock_permission_service);

    let request = CreateRoleRequest {
        name: "Test Role".to_string(),
        description: Some("Test role description".to_string()),
        scope: "PROJECT".to_string(),
    };

    let result = permission_use_case.create_role(request).await;
    assert!(result.is_ok());
    let role = result.unwrap();
    assert_eq!(role.name, "Test Role");
    assert_eq!(role.scope, "PROJECT");
}

#[tokio::test]
async fn test_permission_use_case_get_role_success() {
    let mut mock_permission_service = MockPermissionService::new();
    let role = create_test_role();
    mock_permission_service.add_role(role.clone());
    
    let permission_use_case = PermissionUseCase::new(mock_permission_service);

    let result = permission_use_case.get_role(role.id).await;
    assert!(result.is_ok());
    let retrieved_role = result.unwrap();
    assert_eq!(retrieved_role.name, role.name);
    assert_eq!(retrieved_role.scope, role.scope);
}

#[tokio::test]
async fn test_permission_use_case_get_role_not_found() {
    let mock_permission_service = MockPermissionService::new();
    let permission_use_case = PermissionUseCase::new(mock_permission_service);

    let result = permission_use_case.get_role(999).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        ServiceError::NotFound(_) => {},
        _ => panic!("Expected NotFound error"),
    }
}

#[tokio::test]
async fn test_permission_use_case_get_global_roles_success() {
    let mut mock_permission_service = MockPermissionService::new();
    let role1 = create_test_role();
    let role2 = Role {
        id: 2,
        name: "Admin Role".to_string(),
        description: Some("Admin role description".to_string()),
        scope: "GLOBAL".to_string(),
        created_at: chrono::Utc::now().naive_utc(),
    };
    mock_permission_service.add_role(role1);
    mock_permission_service.add_role(role2);
    
    let permission_use_case = PermissionUseCase::new(mock_permission_service);

    let result = permission_use_case.get_global_roles().await;
    assert!(result.is_ok());
    let roles = result.unwrap();
    assert_eq!(roles.len(), 1);
    assert_eq!(roles[0].scope, "GLOBAL");
}

#[tokio::test]
async fn test_permission_use_case_assign_permission_to_role_success() {
    let mut mock_permission_service = MockPermissionService::new();
    let role = create_test_role();
    let permission = create_test_permission();
    mock_permission_service.add_role(role);
    mock_permission_service.add_permission(permission);
    
    let permission_use_case = PermissionUseCase::new(mock_permission_service);

    let request = AssignPermissionRequest {
        permission_id: 1,
    };

    let result = permission_use_case.assign_permission_to_role(1, request).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_permission_use_case_assign_permission_to_role_not_found() {
    let mock_permission_service = MockPermissionService::new();
    let permission_use_case = PermissionUseCase::new(mock_permission_service);

    let request = AssignPermissionRequest {
        permission_id: 1,
    };

    let result = permission_use_case.assign_permission_to_role(999, request).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_permission_use_case_get_role_permissions_success() {
    let mut mock_permission_service = MockPermissionService::new();
    let role = create_test_role();
    let permission = create_test_permission();
    mock_permission_service.add_role(role);
    mock_permission_service.add_permission(permission);
    
    let permission_use_case = PermissionUseCase::new(mock_permission_service);

    let result = permission_use_case.get_role_permissions(1).await;
    assert!(result.is_ok());
    let permissions_response = result.unwrap();
    assert_eq!(permissions_response.role_id, 1);
    assert_eq!(permissions_response.permissions.len(), 1);
}

#[tokio::test]
async fn test_permission_use_case_assign_permission_to_project_success() {
    let mut mock_permission_service = MockPermissionService::new();
    let permission = create_test_permission();
    mock_permission_service.add_permission(permission);
    
    let permission_use_case = PermissionUseCase::new(mock_permission_service);

    let request = AssignPermissionRequest {
        permission_id: 1,
    };

    let result = permission_use_case.assign_permission_to_project(1, request).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_permission_use_case_get_project_permissions_success() {
    let mut mock_permission_service = MockPermissionService::new();
    let permission = create_test_permission();
    mock_permission_service.add_permission(permission);
    
    let permission_use_case = PermissionUseCase::new(mock_permission_service);

    let result = permission_use_case.get_project_permissions(1).await;
    assert!(result.is_ok());
    let permissions_response = result.unwrap();
    assert_eq!(permissions_response.permissions.len(), 1);
    assert_eq!(permissions_response.project_id, 1);
}

#[tokio::test]
async fn test_permission_use_case_get_permissions_for_resource_success() {
    let mut mock_permission_service = MockPermissionService::new();
    let permission = create_test_permission();
    mock_permission_service.add_permission(permission);
    
    let permission_use_case = PermissionUseCase::new(mock_permission_service);

    let result = permission_use_case.get_permissions_for_resource("annotation").await;
    assert!(result.is_ok());
    let permissions_response = result.unwrap();
    assert_eq!(permissions_response.permissions.len(), 1);
    assert_eq!(permissions_response.resource_type, "annotation");
}