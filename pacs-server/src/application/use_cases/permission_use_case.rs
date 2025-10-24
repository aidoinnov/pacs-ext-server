use crate::application::dto::{
    CreateRoleRequest, RoleResponse, PermissionResponse,
    RolePermissionsResponse, ProjectPermissionsResponse, ResourcePermissionsResponse,
    RoleWithPermissionsResponse, RolesWithPermissionsListResponse,
};
use crate::application::dto::permission_dto::AssignPermissionRequest;
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

    /// Global 역할 목록 조회 (권한 정보 포함, 페이지네이션)
    pub async fn get_global_roles_with_permissions(
        &self,
        page: Option<i32>,
        page_size: Option<i32>,
    ) -> Result<RolesWithPermissionsListResponse, ServiceError> {
        let page = page.unwrap_or(1).max(1);
        let page_size = page_size.unwrap_or(20).clamp(1, 100);
        let offset = (page - 1) * page_size;
        
        // 전체 Global 역할 조회
        let all_roles = self.permission_service.get_global_roles().await?;
        let total_count = all_roles.len() as i64;
        
        // 페이지네이션 적용
        let paginated_roles: Vec<_> = all_roles
            .into_iter()
            .skip(offset as usize)
            .take(page_size as usize)
            .collect();
        
        // 각 역할의 권한 조회
        let mut roles_with_permissions = Vec::new();
        for role in paginated_roles {
            let permissions = self.permission_service
                .get_role_permissions(role.id)
                .await?;
            
            roles_with_permissions.push(RoleWithPermissionsResponse {
                id: role.id,
                name: role.name,
                description: role.description,
                scope: role.scope,
                permissions: permissions
                    .into_iter()
                    .map(|p| PermissionResponse {
                        id: p.id,
                        resource_type: p.resource_type,
                        action: p.action,
                    })
                    .collect(),
            });
        }
        
        let total_pages = ((total_count + page_size as i64 - 1) / page_size as i64) as i32;
        
        Ok(RolesWithPermissionsListResponse {
            roles: roles_with_permissions,
            total_count,
            page,
            page_size,
            total_pages,
        })
    }
}
