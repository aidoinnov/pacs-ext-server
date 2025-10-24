use crate::application::dto::{
    CreateProjectRequest, ProjectResponse, ProjectListResponse, ProjectAssignRoleRequest,
    MemberInfo, ProjectRolesResponse, RoleInfo,
};
use crate::application::dto::project_dto::ProjectMembersResponse;
use crate::domain::services::ProjectService;
use crate::domain::ServiceError;

/// 프로젝트 관리 유스케이스
pub struct ProjectUseCase<P: ProjectService> {
    project_service: P,
}

impl<P: ProjectService> ProjectUseCase<P> {
    pub fn new(project_service: P) -> Self {
        Self { project_service }
    }

    /// 프로젝트 생성
    pub async fn create_project(&self, request: CreateProjectRequest) -> Result<ProjectResponse, ServiceError> {
        let project = self
            .project_service
            .create_project(request.name, request.description)
            .await?;

        Ok(ProjectResponse {
            id: project.id,
            name: project.name,
            description: project.description,
            is_active: project.is_active,
            created_at: project.created_at,
        })
    }

    /// 프로젝트 조회
    pub async fn get_project(&self, project_id: i32) -> Result<ProjectResponse, ServiceError> {
        let project = self.project_service.get_project(project_id).await?;

        Ok(ProjectResponse {
            id: project.id,
            name: project.name,
            description: project.description,
            is_active: project.is_active,
            created_at: project.created_at,
        })
    }

    /// 모든 프로젝트 조회
    pub async fn get_all_projects(&self) -> Result<ProjectListResponse, ServiceError> {
        let projects = self.project_service.get_all_projects().await?;
        let total = projects.len();

        let project_responses = projects
            .into_iter()
            .map(|p| ProjectResponse {
                id: p.id,
                name: p.name,
                description: p.description,
                is_active: p.is_active,
                created_at: p.created_at,
            })
            .collect();

        Ok(ProjectListResponse {
            projects: project_responses,
            total,
        })
    }

    /// 활성화된 프로젝트만 조회
    pub async fn get_active_projects(&self) -> Result<ProjectListResponse, ServiceError> {
        let projects = self.project_service.get_active_projects().await?;
        let total = projects.len();

        let project_responses = projects
            .into_iter()
            .map(|p| ProjectResponse {
                id: p.id,
                name: p.name,
                description: p.description,
                is_active: p.is_active,
                created_at: p.created_at,
            })
            .collect();

        Ok(ProjectListResponse {
            projects: project_responses,
            total,
        })
    }

    /// 프로젝트 활성화
    pub async fn activate_project(&self, project_id: i32) -> Result<ProjectResponse, ServiceError> {
        let project = self.project_service.activate_project(project_id).await?;

        Ok(ProjectResponse {
            id: project.id,
            name: project.name,
            description: project.description,
            is_active: project.is_active,
            created_at: project.created_at,
        })
    }

    /// 프로젝트 비활성화
    pub async fn deactivate_project(&self, project_id: i32) -> Result<ProjectResponse, ServiceError> {
        let project = self.project_service.deactivate_project(project_id).await?;

        Ok(ProjectResponse {
            id: project.id,
            name: project.name,
            description: project.description,
            is_active: project.is_active,
            created_at: project.created_at,
        })
    }

    /// 프로젝트 삭제
    pub async fn delete_project(&self, project_id: i32) -> Result<(), ServiceError> {
        self.project_service.delete_project(project_id).await
    }

    /// 프로젝트 멤버 목록 조회
    pub async fn get_project_members(&self, project_id: i32) -> Result<ProjectMembersResponse, ServiceError> {
        let members = self.project_service.get_project_members(project_id).await?;
        let count = self.project_service.count_project_members(project_id).await?;

        let member_infos = members
            .into_iter()
            .map(|m| MemberInfo {
                id: m.id,
                username: m.username,
                email: m.email,
                joined_at: m.created_at,
            })
            .collect();

        Ok(ProjectMembersResponse {
            project_id,
            members: member_infos,
            total: count,
        })
    }

    /// 프로젝트에 역할 할당
    pub async fn assign_role(&self, project_id: i32, request: ProjectAssignRoleRequest) -> Result<(), ServiceError> {
        self.project_service
            .assign_role_to_project(project_id, request.role_id)
            .await
    }

    /// 프로젝트에서 역할 제거
    pub async fn remove_role(&self, project_id: i32, role_id: i32) -> Result<(), ServiceError> {
        self.project_service
            .remove_role_from_project(project_id, role_id)
            .await
    }

    /// 프로젝트 역할 목록 조회
    pub async fn get_project_roles(&self, project_id: i32) -> Result<ProjectRolesResponse, ServiceError> {
        let roles = self.project_service.get_project_roles(project_id).await?;

        let role_infos = roles
            .into_iter()
            .map(|r| RoleInfo {
                id: r.id,
                name: r.name,
                description: r.description,
                scope: r.scope,
            })
            .collect();

        Ok(ProjectRolesResponse {
            project_id,
            roles: role_infos,
        })
    }
}
