use crate::application::dto::{
    CreateRoleRequest, RoleResponse, PermissionResponse, AssignPermissionRequest,
    RolePermissionsResponse, ProjectPermissionsResponse, ResourcePermissionsResponse,
};
use crate::domain::services::PermissionService;
use crate::domain::ServiceError;
use crate::domain::entities::RoleScope;

/// 권한 관리 유스케이스
pub struct PermissionUseCase<P: PermissionService> {
    permission_service: P,
}

impl<P: PermissionService> PermissionUseCase<P> {
    pub fn new(permission_service: P) -> Self {
        Self { permission_service }
    }

    /// 역할 생성
    pub async fn create_role(&self, request: CreateRoleRequest) -> Result<RoleResponse, ServiceError> {
        let scope = match request.scope.as_str() {
            "GLOBAL" => RoleScope::Global,
            "PROJECT" => RoleScope::Project,
            _ => return Err(ServiceError::ValidationError("Invalid scope. Must be GLOBAL or PROJECT".into())),
        };

        let role = self
            .permission_service
            .create_role(request.name, scope, request.description)
            .await?;

        Ok(RoleResponse {
            id: role.id,
            name: role.name,
            description: role.description,
            scope: role.scope,
        })
    }

    /// 역할 조회
    pub async fn get_role(&self, role_id: i32) -> Result<RoleResponse, ServiceError> {
        let role = self.permission_service.get_role(role_id).await?;

        Ok(RoleResponse {
            id: role.id,
            name: role.name,
            description: role.description,
            scope: role.scope,
        })
    }

    /// Global 역할 목록 조회
    pub async fn get_global_roles(&self) -> Result<Vec<RoleResponse>, ServiceError> {
        let roles = self.permission_service.get_global_roles().await?;

        Ok(roles
            .into_iter()
            .map(|r| RoleResponse {
                id: r.id,
                name: r.name,
                description: r.description,
                scope: r.scope,
            })
            .collect())
    }

    /// Project 역할 목록 조회
    pub async fn get_project_roles(&self) -> Result<Vec<RoleResponse>, ServiceError> {
        let roles = self.permission_service.get_project_roles().await?;

        Ok(roles
            .into_iter()
            .map(|r| RoleResponse {
                id: r.id,
                name: r.name,
                description: r.description,
                scope: r.scope,
            })
            .collect())
    }

    /// 역할에 권한 할당
    pub async fn assign_permission_to_role(
        &self,
        role_id: i32,
        request: AssignPermissionRequest,
    ) -> Result<(), ServiceError> {
        self.permission_service
            .assign_permission_to_role(role_id, request.permission_id)
            .await
    }

    /// 역할에서 권한 제거
    pub async fn remove_permission_from_role(
        &self,
        role_id: i32,
        permission_id: i32,
    ) -> Result<(), ServiceError> {
        self.permission_service
            .remove_permission_from_role(role_id, permission_id)
            .await
    }

    /// 역할 권한 목록 조회
    pub async fn get_role_permissions(&self, role_id: i32) -> Result<RolePermissionsResponse, ServiceError> {
        let role = self.permission_service.get_role(role_id).await?;
        let permissions = self.permission_service.get_role_permissions(role_id).await?;

        let permission_responses = permissions
            .into_iter()
            .map(|p| PermissionResponse {
                id: p.id,
                resource_type: p.resource_type,
                action: p.action,
            })
            .collect();

        Ok(RolePermissionsResponse {
            role_id: role.id,
            role_name: role.name,
            permissions: permission_responses,
        })
    }

    /// 프로젝트에 권한 할당
    pub async fn assign_permission_to_project(
        &self,
        project_id: i32,
        request: AssignPermissionRequest,
    ) -> Result<(), ServiceError> {
        self.permission_service
            .assign_permission_to_project(project_id, request.permission_id)
            .await
    }

    /// 프로젝트에서 권한 제거
    pub async fn remove_permission_from_project(
        &self,
        project_id: i32,
        permission_id: i32,
    ) -> Result<(), ServiceError> {
        self.permission_service
            .remove_permission_from_project(project_id, permission_id)
            .await
    }

    /// 프로젝트 권한 목록 조회
    pub async fn get_project_permissions(&self, project_id: i32) -> Result<ProjectPermissionsResponse, ServiceError> {
        let permissions = self
            .permission_service
            .get_project_permissions(project_id)
            .await?;

        let permission_responses = permissions
            .into_iter()
            .map(|p| PermissionResponse {
                id: p.id,
                resource_type: p.resource_type,
                action: p.action,
            })
            .collect();

        Ok(ProjectPermissionsResponse {
            project_id,
            permissions: permission_responses,
        })
    }

    /// 리소스별 권한 조회
    pub async fn get_permissions_for_resource(&self, resource_type: &str) -> Result<ResourcePermissionsResponse, ServiceError> {
        let permissions = self
            .permission_service
            .get_permissions_for_resource(resource_type)
            .await?;

        let permission_responses = permissions
            .into_iter()
            .map(|p| PermissionResponse {
                id: p.id,
                resource_type: p.resource_type.clone(),
                action: p.action,
            })
            .collect();

        Ok(ResourcePermissionsResponse {
            resource_type: resource_type.to_string(),
            permissions: permission_responses,
        })
    }
}
