use std::collections::HashMap;
use std::sync::Arc;
use crate::domain::services::PermissionService;
use crate::domain::ServiceError;
use crate::application::dto::role_permission_matrix_dto::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Arc;
    use mockall::mock;
    use crate::domain::entities::{Role, Permission, RoleScope};
    use crate::domain::ServiceError;

    // Mock PermissionService
    mock! {
        PermissionService {}

        #[async_trait::async_trait]
        impl crate::domain::services::PermissionService for PermissionService {
            async fn get_permissions_for_resource(&self, resource_type: &str) -> Result<Vec<Permission>, ServiceError>;
            async fn validate_permission_exists(&self, resource_type: &str, action: &str) -> Result<bool, ServiceError>;
            async fn create_role(&self, name: String, scope: RoleScope, description: Option<String>) -> Result<Role, ServiceError>;
            async fn get_role(&self, id: i32) -> Result<Role, ServiceError>;
            async fn get_roles_by_scope(&self, scope: RoleScope) -> Result<Vec<Role>, ServiceError>;
            async fn get_global_roles(&self) -> Result<Vec<Role>, ServiceError>;
            async fn get_project_roles(&self) -> Result<Vec<Role>, ServiceError>;
            async fn assign_permission_to_role(&self, role_id: i32, permission_id: i32) -> Result<(), ServiceError>;
            async fn remove_permission_from_role(&self, role_id: i32, permission_id: i32) -> Result<(), ServiceError>;
            async fn get_role_permissions(&self, role_id: i32) -> Result<Vec<Permission>, ServiceError>;
            async fn assign_permission_to_project(&self, project_id: i32, permission_id: i32) -> Result<(), ServiceError>;
            async fn remove_permission_from_project(&self, project_id: i32, permission_id: i32) -> Result<(), ServiceError>;
            async fn get_project_permissions(&self, project_id: i32) -> Result<Vec<Permission>, ServiceError>;
            async fn get_global_role_permission_matrix(&self) -> Result<(Vec<Role>, Vec<Permission>, Vec<(i32, i32)>), ServiceError>;
            async fn get_project_role_permission_matrix(&self, project_id: i32) -> Result<(Vec<Role>, Vec<Permission>, Vec<(i32, i32)>), ServiceError>;
        }
    }

    fn create_test_roles() -> Vec<Role> {
        vec![
            Role {
                id: 1,
                name: "Admin".to_string(),
                description: Some("Administrator role".to_string()),
                scope: "GLOBAL".to_string(),
                created_at: chrono::Utc::now(),
            },
            Role {
                id: 2,
                name: "User".to_string(),
                description: Some("Regular user role".to_string()),
                scope: "GLOBAL".to_string(),
                created_at: chrono::Utc::now(),
            },
        ]
    }

    fn create_test_permissions() -> Vec<Permission> {
        vec![
            Permission {
                id: 1,
                resource_type: "USER".to_string(),
                action: "READ".to_string(),
            },
            Permission {
                id: 2,
                resource_type: "USER".to_string(),
                action: "WRITE".to_string(),
            },
            Permission {
                id: 3,
                resource_type: "PROJECT".to_string(),
                action: "READ".to_string(),
            },
        ]
    }

    fn create_test_assignments() -> Vec<(i32, i32)> {
        vec![
            (1, 1), // Admin has USER:READ
            (1, 2), // Admin has USER:WRITE
            (1, 3), // Admin has PROJECT:READ
            (2, 1), // User has USER:READ
        ]
    }

    #[tokio::test]
    async fn test_get_global_matrix_success() {
        let mut mock_service = MockPermissionService::new();
        let roles = create_test_roles();
        let permissions = create_test_permissions();
        let assignments = create_test_assignments();

        mock_service
            .expect_get_global_role_permission_matrix()
            .times(1)
            .returning(move || Ok((roles.clone(), permissions.clone(), assignments.clone())));

        let use_case = RolePermissionMatrixUseCase::new(Arc::new(mock_service));
        let result = use_case.get_global_matrix().await;

        assert!(result.is_ok());
        let matrix = result.unwrap();
        
        // Check roles
        assert_eq!(matrix.roles.len(), 2);
        assert_eq!(matrix.roles[0].name, "Admin");
        assert_eq!(matrix.roles[1].name, "User");

        // Check permissions by category
        assert!(matrix.permissions_by_category.contains_key("USER"));
        assert!(matrix.permissions_by_category.contains_key("PROJECT"));
        assert_eq!(matrix.permissions_by_category["USER"].len(), 2);
        assert_eq!(matrix.permissions_by_category["PROJECT"].len(), 1);

        // Check assignments
        assert_eq!(matrix.assignments.len(), 6); // 2 roles * 3 permissions
    }

    #[tokio::test]
    async fn test_get_global_matrix_service_error() {
        let mut mock_service = MockPermissionService::new();
        
        mock_service
            .expect_get_global_role_permission_matrix()
            .times(1)
            .returning(|| Err(ServiceError::DatabaseError("Database connection failed".into())));

        let use_case = RolePermissionMatrixUseCase::new(Arc::new(mock_service));
        let result = use_case.get_global_matrix().await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ServiceError::DatabaseError(msg) => assert_eq!(msg, "Database connection failed"),
            _ => panic!("Expected DatabaseError"),
        }
    }

    #[tokio::test]
    async fn test_get_project_matrix_success() {
        let mut mock_service = MockPermissionService::new();
        let roles = create_test_roles();
        let permissions = create_test_permissions();
        let assignments = create_test_assignments();

        mock_service
            .expect_get_project_role_permission_matrix()
            .times(1)
            .returning(move |_| Ok((roles.clone(), permissions.clone(), assignments.clone())));

        let use_case = RolePermissionMatrixUseCase::new(Arc::new(mock_service));
        let result = use_case.get_project_matrix(1).await;

        assert!(result.is_ok());
        let matrix = result.unwrap();
        
        assert_eq!(matrix.roles.len(), 2);
        assert_eq!(matrix.assignments.len(), 6);
    }

    #[tokio::test]
    async fn test_get_project_matrix_not_found() {
        let mut mock_service = MockPermissionService::new();
        
        mock_service
            .expect_get_project_role_permission_matrix()
            .times(1)
            .returning(|_| Err(ServiceError::NotFound("Project not found".into())));

        let use_case = RolePermissionMatrixUseCase::new(Arc::new(mock_service));
        let result = use_case.get_project_matrix(999).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ServiceError::NotFound(msg) => assert_eq!(msg, "Project not found"),
            _ => panic!("Expected NotFound error"),
        }
    }

    #[tokio::test]
    async fn test_update_permission_assignment_assign() {
        let mut mock_service = MockPermissionService::new();
        
        mock_service
            .expect_assign_permission_to_role()
            .times(1)
            .with(mockall::predicate::eq(1), mockall::predicate::eq(2))
            .returning(|_, _| Ok(()));

        let use_case = RolePermissionMatrixUseCase::new(Arc::new(mock_service));
        let result = use_case.update_permission_assignment(1, 2, true).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_permission_assignment_remove() {
        let mut mock_service = MockPermissionService::new();
        
        mock_service
            .expect_remove_permission_from_role()
            .times(1)
            .with(mockall::predicate::eq(1), mockall::predicate::eq(2))
            .returning(|_, _| Ok(()));

        let use_case = RolePermissionMatrixUseCase::new(Arc::new(mock_service));
        let result = use_case.update_permission_assignment(1, 2, false).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_permission_assignment_error() {
        let mut mock_service = MockPermissionService::new();
        
        mock_service
            .expect_assign_permission_to_role()
            .times(1)
            .returning(|_, _| Err(ServiceError::ValidationError("Invalid role or permission".into())));

        let use_case = RolePermissionMatrixUseCase::new(Arc::new(mock_service));
        let result = use_case.update_permission_assignment(1, 2, true).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ServiceError::ValidationError(msg) => assert_eq!(msg, "Invalid role or permission"),
            _ => panic!("Expected ValidationError"),
        }
    }
}

/// 역할-권한 매트릭스 Use Case
pub struct RolePermissionMatrixUseCase {
    permission_service: Arc<dyn PermissionService>,
}

impl RolePermissionMatrixUseCase {
    pub fn new(permission_service: Arc<dyn PermissionService>) -> Self {
        Self {
            permission_service,
        }
    }

    /// 글로벌 역할-권한 매트릭스 조회
    pub async fn get_global_matrix(&self) -> Result<RolePermissionMatrixResponse, ServiceError> {
        let (roles, permissions, assignments) = self.permission_service
            .get_global_role_permission_matrix()
            .await?;

        // 역할 정보 변환
        let role_infos: Vec<RoleInfo> = roles
            .into_iter()
            .map(|role| RoleInfo {
                id: role.id,
                name: role.name,
                description: role.description,
                scope: role.scope,
            })
            .collect();

        // 권한을 카테고리별로 그룹화
        let mut permissions_by_category: HashMap<String, Vec<PermissionInfo>> = HashMap::new();
        for permission in permissions {
            let permission_info = PermissionInfo {
                id: permission.id,
                resource_type: permission.resource_type.clone(),
                action: permission.action,
            };
            
            permissions_by_category
                .entry(permission.resource_type)
                .or_insert_with(Vec::new)
                .push(permission_info);
        }

        // 할당 정보 변환
        let assignment_set: std::collections::HashSet<(i32, i32)> = assignments.into_iter().collect();
        let assignments: Vec<RolePermissionAssignment> = role_infos
            .iter()
            .flat_map(|role| {
                let assignment_set = assignment_set.clone();
                permissions_by_category
                    .values()
                    .flatten()
                    .map(move |permission| RolePermissionAssignment {
                        role_id: role.id,
                        permission_id: permission.id,
                        assigned: assignment_set.contains(&(role.id, permission.id)),
                    })
            })
            .collect();

        Ok(RolePermissionMatrixResponse {
            roles: role_infos,
            permissions_by_category,
            assignments,
        })
    }

    /// 프로젝트별 역할-권한 매트릭스 조회
    pub async fn get_project_matrix(&self, project_id: i32) -> Result<RolePermissionMatrixResponse, ServiceError> {
        let (roles, permissions, assignments) = self.permission_service
            .get_project_role_permission_matrix(project_id)
            .await?;

        // 역할 정보 변환
        let role_infos: Vec<RoleInfo> = roles
            .into_iter()
            .map(|role| RoleInfo {
                id: role.id,
                name: role.name,
                description: role.description,
                scope: role.scope,
            })
            .collect();

        // 권한을 카테고리별로 그룹화
        let mut permissions_by_category: HashMap<String, Vec<PermissionInfo>> = HashMap::new();
        for permission in permissions {
            let permission_info = PermissionInfo {
                id: permission.id,
                resource_type: permission.resource_type.clone(),
                action: permission.action,
            };
            
            permissions_by_category
                .entry(permission.resource_type)
                .or_insert_with(Vec::new)
                .push(permission_info);
        }

        // 할당 정보 변환
        let assignment_set: std::collections::HashSet<(i32, i32)> = assignments.into_iter().collect();
        let assignments: Vec<RolePermissionAssignment> = role_infos
            .iter()
            .flat_map(|role| {
                let assignment_set = assignment_set.clone();
                permissions_by_category
                    .values()
                    .flatten()
                    .map(move |permission| RolePermissionAssignment {
                        role_id: role.id,
                        permission_id: permission.id,
                        assigned: assignment_set.contains(&(role.id, permission.id)),
                    })
            })
            .collect();

        Ok(RolePermissionMatrixResponse {
            roles: role_infos,
            permissions_by_category,
            assignments,
        })
    }

    /// 개별 권한 할당/제거
    pub async fn update_permission_assignment(
        &self,
        role_id: i32,
        permission_id: i32,
        assign: bool,
    ) -> Result<(), ServiceError> {
        if assign {
            self.permission_service
                .assign_permission_to_role(role_id, permission_id)
                .await
        } else {
            self.permission_service
                .remove_permission_from_role(role_id, permission_id)
                .await
        }
    }
}
