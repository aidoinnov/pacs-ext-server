use std::sync::Arc;
use crate::domain::ServiceError;
use crate::domain::services::{ProjectService, UserService, ProjectDataService};
use crate::application::dto::project_user_dto::{
    ProjectMembersResponse, UserProjectsResponse, RoleAssignmentResponse, BatchRoleAssignmentResponse, FailedAssignment,
    AddMemberRequest, AddMemberResponse, RemoveMemberResponse, MembershipResponse
};

/// 프로젝트-사용자 역할 관리 Use Case
pub struct ProjectUserUseCase<P, U, D>
where
    P: ProjectService,
    U: UserService,
    D: ProjectDataService,
{
    project_service: Arc<P>,
    user_service: Arc<U>,
    project_data_service: Arc<D>,
}

impl<P, U, D> ProjectUserUseCase<P, U, D>
where
    P: ProjectService,
    U: UserService,
    D: ProjectDataService,
{
    pub fn new(project_service: Arc<P>, user_service: Arc<U>, project_data_service: Arc<D>) -> Self {
        Self {
            project_service,
            user_service,
            project_data_service,
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

    /// 프로젝트에 멤버 추가
    pub async fn add_member_to_project(
        &self,
        project_id: i32,
        request: AddMemberRequest,
    ) -> Result<AddMemberResponse, ServiceError> {
        // 프로젝트 존재 확인
        self.project_service.get_project(project_id).await?;

        // 사용자를 프로젝트에 추가
        self.user_service
            .add_user_to_project_with_role(request.user_id, project_id, request.role_id)
            .await?;

        // ✅ 프로젝트의 모든 데이터에 대한 기본 접근 권한 자동 부여
        let _ = self.project_data_service
            .grant_default_access_to_user(project_id, request.user_id)
            .await
            .map_err(|e| {
                // 로깅만 하고 계속 진행 (access 권한은 옵셔널)
                eprintln!("Warning: Failed to grant default access to user: {}", e);
                e
            });

        // 추가된 멤버의 역할 정보 조회
        let membership = self.user_service
            .get_project_membership(request.user_id, project_id)
            .await?;

        let (role_name, role_id) = match membership {
            Some(m) => (
                m.role_name.unwrap_or_else(|| "Unknown".to_string()),
                m.role_id.unwrap_or(0)
            ),
            None => ("Unknown".to_string(), 0)
        };

        Ok(AddMemberResponse {
            message: "Member added to project successfully".to_string(),
            user_id: request.user_id,
            project_id,
            role_id,
            role_name,
        })
    }

    /// 프로젝트에서 멤버 제거
    pub async fn remove_member_from_project(
        &self,
        project_id: i32,
        user_id: i32,
    ) -> Result<RemoveMemberResponse, ServiceError> {
        // 프로젝트 존재 확인
        self.project_service.get_project(project_id).await?;

        // 사용자를 프로젝트에서 제거
        self.user_service
            .remove_user_from_project(user_id, project_id)
            .await?;

        Ok(RemoveMemberResponse {
            message: "Member removed from project successfully".to_string(),
            user_id,
            project_id,
        })
    }

    /// 프로젝트 멤버십 확인
    pub async fn check_project_membership(
        &self,
        project_id: i32,
        user_id: i32,
    ) -> Result<MembershipResponse, ServiceError> {
        // 프로젝트 존재 확인
        self.project_service.get_project(project_id).await?;

        // 멤버십 정보 조회
        let membership = self.user_service
            .get_project_membership(user_id, project_id)
            .await?;

        Ok(membership.unwrap_or(MembershipResponse {
            is_member: false,
            role_id: None,
            role_name: None,
            joined_at: None,
        }))
    }
}
