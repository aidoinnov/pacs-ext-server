use async_trait::async_trait;
use crate::domain::entities::{Permission, Role, RoleScope};
use crate::domain::repositories::{PermissionRepository, RoleRepository};
use super::user_service::ServiceError;

/// 권한 관리 도메인 서비스
#[async_trait]
pub trait PermissionService: Send + Sync {
    /// 특정 리소스 타입에 대한 모든 권한 조회
    async fn get_permissions_for_resource(&self, resource_type: &str) -> Result<Vec<Permission>, ServiceError>;

    /// 역할에 필요한 권한이 존재하는지 확인
    async fn validate_permission_exists(&self, resource_type: &str, action: &str) -> Result<bool, ServiceError>;

    /// 역할 생성 (Global 또는 Project scope)
    async fn create_role(&self, name: String, scope: RoleScope, description: Option<String>) -> Result<Role, ServiceError>;

    /// 역할 조회
    async fn get_role(&self, id: i32) -> Result<Role, ServiceError>;

    /// Scope별 역할 조회
    async fn get_roles_by_scope(&self, scope: RoleScope) -> Result<Vec<Role>, ServiceError>;

    /// Global 역할만 조회
    async fn get_global_roles(&self) -> Result<Vec<Role>, ServiceError>;

    /// Project 역할만 조회
    async fn get_project_roles(&self) -> Result<Vec<Role>, ServiceError>;
}

pub struct PermissionServiceImpl<P: PermissionRepository, R: RoleRepository> {
    permission_repository: P,
    role_repository: R,
}

impl<P: PermissionRepository, R: RoleRepository> PermissionServiceImpl<P, R> {
    pub fn new(permission_repository: P, role_repository: R) -> Self {
        Self {
            permission_repository,
            role_repository,
        }
    }
}

#[async_trait]
impl<P: PermissionRepository, R: RoleRepository> PermissionService for PermissionServiceImpl<P, R> {
    async fn get_permissions_for_resource(&self, resource_type: &str) -> Result<Vec<Permission>, ServiceError> {
        Ok(self.permission_repository.find_by_resource_type(resource_type).await?)
    }

    async fn validate_permission_exists(&self, resource_type: &str, action: &str) -> Result<bool, ServiceError> {
        Ok(self.permission_repository
            .find_by_resource_and_action(resource_type, action)
            .await?
            .is_some())
    }

    async fn create_role(&self, name: String, scope: RoleScope, description: Option<String>) -> Result<Role, ServiceError> {
        // 역할 이름 검증
        if name.trim().is_empty() {
            return Err(ServiceError::ValidationError("Role name cannot be empty".into()));
        }

        if name.len() > 100 {
            return Err(ServiceError::ValidationError("Role name too long (max 100 characters)".into()));
        }

        // 같은 이름의 역할이 이미 존재하는지 확인
        if let Some(_) = self.role_repository.find_by_name(&name).await? {
            return Err(ServiceError::AlreadyExists("Role name already exists".into()));
        }

        let new_role = crate::domain::entities::NewRole {
            name,
            scope,
            description,
        };

        Ok(self.role_repository.create(new_role).await?)
    }

    async fn get_role(&self, id: i32) -> Result<Role, ServiceError> {
        self.role_repository
            .find_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound("Role not found".into()))
    }

    async fn get_roles_by_scope(&self, scope: RoleScope) -> Result<Vec<Role>, ServiceError> {
        Ok(self.role_repository.find_by_scope(scope.as_str()).await?)
    }

    async fn get_global_roles(&self) -> Result<Vec<Role>, ServiceError> {
        Ok(self.role_repository.find_by_scope(RoleScope::Global.as_str()).await?)
    }

    async fn get_project_roles(&self) -> Result<Vec<Role>, ServiceError> {
        Ok(self.role_repository.find_by_scope(RoleScope::Project.as_str()).await?)
    }
}
