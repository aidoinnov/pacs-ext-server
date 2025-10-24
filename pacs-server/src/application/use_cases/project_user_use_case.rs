use std::sync::Arc;
use crate::domain::ServiceError;
use crate::domain::services::{ProjectService, UserService};
use crate::application::dto::project_user_dto::{
    ProjectMembersResponse, UserProjectsResponse, AssignRoleRequest, BatchAssignRolesRequest,
    RoleAssignmentResponse, BatchRoleAssignmentResponse, FailedAssignment
};
use crate::application::dto::permission_dto::PaginationQuery;

/// 프로젝트-사용자 역할 관리 Use Case
pub struct ProjectUserUseCase<P, U>
where
    P: ProjectService,
    U: UserService,
{
    project_service: Arc<P>,
    user_service: Arc<U>,
}

impl<P, U> ProjectUserUseCase<P, U>
where
    P: ProjectService,
    U: UserService,
{
    pub fn new(project_service: Arc<P>, user_service: Arc<U>) -> Self {
        Self {
            project_service,
            user_service,
        }
    }

    /// 프로젝트 멤버 목록 조회 (역할 정보 포함, 페이지네이션)
    pub async fn get_project_members_with_roles(
        &self,
        project_id: i32,
        page: Option<i32>,
        page_size: Option<i32>,
    ) -> Result<ProjectMembersResponse, ServiceError> {
        let page = page.unwrap_or(1).max(1);
        let page_size = page_size.unwrap_or(20).clamp(1, 100);
        
        let (users, total_count) = self.project_service
            .get_project_members_with_roles(project_id, page, page_size)
            .await?;
        
        let total_pages = ((total_count + page_size as i64 - 1) / page_size as i64) as i32;
        
        Ok(ProjectMembersResponse {
            members: users,
            total_count,
            page,
            page_size,
            total_pages,
        })
    }
    
    /// 사용자의 프로젝트 목록 조회 (역할 정보 포함, 페이지네이션)
    pub async fn get_user_projects_with_roles(
        &self,
        user_id: i32,
        page: Option<i32>,
        page_size: Option<i32>,
    ) -> Result<UserProjectsResponse, ServiceError> {
        let page = page.unwrap_or(1).max(1);
        let page_size = page_size.unwrap_or(20).clamp(1, 100);
        
        let (projects, total_count) = self.user_service
            .get_user_projects_with_roles(user_id, page, page_size)
            .await?;
        
        let total_pages = ((total_count + page_size as i64 - 1) / page_size as i64) as i32;
        
        Ok(UserProjectsResponse {
            projects,
            total_count,
            page,
            page_size,
            total_pages,
        })
    }
    
    /// 프로젝트 내 사용자에게 역할 할당 (개별)
    pub async fn assign_role_to_user(
        &self,
        project_id: i32,
        user_id: i32,
        role_id: i32,
    ) -> Result<RoleAssignmentResponse, ServiceError> {
        self.project_service
            .assign_user_role_in_project(project_id, user_id, role_id)
            .await?;

        Ok(RoleAssignmentResponse {
            message: "Role assigned successfully".to_string(),
            user_id,
            project_id,
            role_id,
        })
    }
    
    /// 프로젝트 내 여러 사용자에게 역할 일괄 할당
    pub async fn batch_assign_roles(
        &self,
        project_id: i32,
        assignments: Vec<(i32, i32)>, // (user_id, role_id)
    ) -> Result<BatchRoleAssignmentResponse, ServiceError> {
        let mut assigned_count = 0;
        let mut failed_assignments = Vec::new();

        for (user_id, role_id) in assignments {
            match self.project_service
                .assign_user_role_in_project(project_id, user_id, role_id)
                .await
            {
                Ok(_) => {
                    assigned_count += 1;
                }
                Err(e) => {
                    failed_assignments.push(FailedAssignment {
                        user_id,
                        role_id,
                        error: e.to_string(),
                    });
                }
            }
        }

        Ok(BatchRoleAssignmentResponse {
            message: format!("Batch role assignment completed. {} successful, {} failed", 
                           assigned_count, failed_assignments.len()),
            project_id,
            assigned_count,
            failed_assignments,
        })
    }

    /// 프로젝트 내 사용자의 역할 제거
    pub async fn remove_user_role(
        &self,
        project_id: i32,
        user_id: i32,
    ) -> Result<RoleAssignmentResponse, ServiceError> {
        // 역할을 NULL로 설정 (제거)
        self.project_service
            .assign_user_role_in_project(project_id, user_id, 0) // 0은 NULL 역할을 의미
            .await?;

        Ok(RoleAssignmentResponse {
            message: "User role removed successfully".to_string(),
            user_id,
            project_id,
            role_id: 0, // NULL 역할
        })
    }
}
