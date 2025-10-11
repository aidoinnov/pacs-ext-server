use async_trait::async_trait;
use crate::domain::entities::{Permission, Role, RoleScope};
use crate::domain::repositories::{PermissionRepository, RoleRepository};
use crate::domain::ServiceError;

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

    // === 권한 할당 관리 ===

    /// 역할에 권한 할당
    async fn assign_permission_to_role(&self, role_id: i32, permission_id: i32) -> Result<(), ServiceError>;

    /// 역할에서 권한 제거
    async fn remove_permission_from_role(&self, role_id: i32, permission_id: i32) -> Result<(), ServiceError>;

    /// 역할에 할당된 모든 권한 조회
    async fn get_role_permissions(&self, role_id: i32) -> Result<Vec<Permission>, ServiceError>;

    /// 프로젝트에 권한 할당
    async fn assign_permission_to_project(&self, project_id: i32, permission_id: i32) -> Result<(), ServiceError>;

    /// 프로젝트에서 권한 제거
    async fn remove_permission_from_project(&self, project_id: i32, permission_id: i32) -> Result<(), ServiceError>;

    /// 프로젝트에 할당된 모든 권한 조회
    async fn get_project_permissions(&self, project_id: i32) -> Result<Vec<Permission>, ServiceError>;
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

    // === 권한 할당 관리 구현 ===

    async fn assign_permission_to_role(&self, role_id: i32, permission_id: i32) -> Result<(), ServiceError> {
        // INSERT with ON CONFLICT - Race condition 방지
        let result = sqlx::query(
            "INSERT INTO security_role_permission (role_id, permission_id)
             SELECT $1, $2
             WHERE EXISTS(SELECT 1 FROM security_role WHERE id = $1)
               AND EXISTS(SELECT 1 FROM security_permission WHERE id = $2)
             ON CONFLICT (role_id, permission_id) DO NOTHING
             RETURNING role_id"
        )
        .bind(role_id)
        .bind(permission_id)
        .fetch_optional(self.role_repository.pool())
        .await?;

        match result {
            Some(_) => Ok(()),
            None => {
                // 실패 원인 파악
                if self.role_repository.find_by_id(role_id).await?.is_none() {
                    return Err(ServiceError::NotFound("Role not found".into()));
                }
                if self.permission_repository.find_by_id(permission_id).await?.is_none() {
                    return Err(ServiceError::NotFound("Permission not found".into()));
                }
                Err(ServiceError::AlreadyExists("Permission already assigned to this role".into()))
            }
        }
    }

    async fn remove_permission_from_role(&self, role_id: i32, permission_id: i32) -> Result<(), ServiceError> {
        let result = sqlx::query(
            "DELETE FROM security_role_permission WHERE role_id = $1 AND permission_id = $2"
        )
        .bind(role_id)
        .bind(permission_id)
        .execute(self.role_repository.pool())
        .await?;

        if result.rows_affected() > 0 {
            Ok(())
        } else {
            Err(ServiceError::NotFound("Permission is not assigned to this role".into()))
        }
    }

    async fn get_role_permissions(&self, role_id: i32) -> Result<Vec<Permission>, ServiceError> {
        // 역할 존재 확인
        if self.role_repository.find_by_id(role_id).await?.is_none() {
            return Err(ServiceError::NotFound("Role not found".into()));
        }

        let permissions = sqlx::query_as::<_, Permission>(
            "SELECT p.id, p.resource_type, p.action
             FROM security_permission p
             INNER JOIN security_role_permission rp ON p.id = rp.permission_id
             WHERE rp.role_id = $1
             ORDER BY p.resource_type, p.action"
        )
        .bind(role_id)
        .fetch_all(self.role_repository.pool())
        .await?;

        Ok(permissions)
    }

    async fn assign_permission_to_project(&self, project_id: i32, permission_id: i32) -> Result<(), ServiceError> {
        // INSERT with ON CONFLICT - Race condition 방지
        let result = sqlx::query(
            "INSERT INTO security_project_permission (project_id, permission_id)
             SELECT $1, $2
             WHERE EXISTS(SELECT 1 FROM security_project WHERE id = $1)
               AND EXISTS(SELECT 1 FROM security_permission WHERE id = $2)
             ON CONFLICT (project_id, permission_id) DO NOTHING
             RETURNING project_id"
        )
        .bind(project_id)
        .bind(permission_id)
        .fetch_optional(self.permission_repository.pool())
        .await?;

        match result {
            Some(_) => Ok(()),
            None => {
                // 실패 원인 파악
                let project_exists = sqlx::query_scalar::<_, bool>(
                    "SELECT EXISTS(SELECT 1 FROM security_project WHERE id = $1)"
                )
                .bind(project_id)
                .fetch_one(self.permission_repository.pool())
                .await?;

                if !project_exists {
                    return Err(ServiceError::NotFound("Project not found".into()));
                }

                if self.permission_repository.find_by_id(permission_id).await?.is_none() {
                    return Err(ServiceError::NotFound("Permission not found".into()));
                }

                Err(ServiceError::AlreadyExists("Permission already assigned to this project".into()))
            }
        }
    }

    async fn remove_permission_from_project(&self, project_id: i32, permission_id: i32) -> Result<(), ServiceError> {
        let result = sqlx::query(
            "DELETE FROM security_project_permission WHERE project_id = $1 AND permission_id = $2"
        )
        .bind(project_id)
        .bind(permission_id)
        .execute(self.permission_repository.pool())
        .await?;

        if result.rows_affected() > 0 {
            Ok(())
        } else {
            Err(ServiceError::NotFound("Permission is not assigned to this project".into()))
        }
    }

    async fn get_project_permissions(&self, project_id: i32) -> Result<Vec<Permission>, ServiceError> {
        let permissions = sqlx::query_as::<_, Permission>(
            "SELECT p.id, p.resource_type, p.action
             FROM security_permission p
             INNER JOIN security_project_permission pp ON p.id = pp.permission_id
             WHERE pp.project_id = $1
             ORDER BY p.resource_type, p.action"
        )
        .bind(project_id)
        .fetch_all(self.permission_repository.pool())
        .await?;

        Ok(permissions)
    }
}
