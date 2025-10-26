use crate::application::dto::{
    CreateProjectRequest, UpdateProjectRequest, ProjectResponse, ProjectListResponse, 
    ProjectAssignRoleRequest, MemberInfo, ProjectRolesResponse,
};
use crate::application::dto::project_dto::{RoleInfo, ProjectMembersResponse, ProjectListQuery, PaginationInfo};
use crate::domain::services::ProjectService;
use crate::domain::{ServiceError, entities::project::{NewProject, UpdateProject, ProjectStatus}};

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
        let new_project = NewProject {
            name: request.name,
            description: request.description,
            sponsor: request.sponsor,
            start_date: request.start_date,
            end_date: request.end_date,
            auto_complete: request.auto_complete.unwrap_or(false),
        };

        let project = self.project_service.create_project(new_project).await?;
        Ok(project.into())
    }

    /// 프로젝트 조회
    pub async fn get_project(&self, project_id: i32) -> Result<ProjectResponse, ServiceError> {
        let project = self.project_service.get_project(project_id).await?;
        Ok(project.into())
    }

    /// 모든 프로젝트 조회 (페이지네이션 및 필터링 지원)
    pub async fn get_all_projects(&self, query: ProjectListQuery) -> Result<ProjectListResponse, ServiceError> {
        let page = query.page.unwrap_or(1);
        let page_size = query.page_size.unwrap_or(20);
        let sort_by = query.sort_by.as_deref().unwrap_or("created_at");
        let sort_order = query.sort_order.as_deref().unwrap_or("desc");
        
        // 필터가 있는지 확인 (status, sponsor, 날짜 범위 중 하나라도 있으면 필터 사용)
        let has_filters = query.status.is_some() || 
                          query.sponsor.is_some() || 
                          query.start_date_from.is_some() || 
                          query.start_date_to.is_some() ||
                          query.end_date_from.is_some() ||
                          query.end_date_to.is_some();
        
        let (projects, total) = if has_filters {
            // 필터링 사용
            let projects = self.project_service.get_projects_with_filter(&query).await?;
            let total = self.project_service.count_projects_with_filter(&query).await?;
            (projects, total)
        } else {
            // 단순 페이지네이션
            let projects = self.project_service.get_projects_paginated(page, page_size, sort_by, sort_order).await?;
            let total = self.project_service.count_all_projects().await?;
            (projects, total)
        };
        
        let total_pages = if total > 0 {
            ((total as f64) / (page_size as f64)).ceil() as i32
        } else {
            0
        };
        
        Ok(ProjectListResponse {
            projects: projects.into_iter().map(|p| p.into()).collect(),
            pagination: PaginationInfo {
                page,
                page_size,
                total,
                total_pages,
            },
        })
    }

    /// 활성화된 프로젝트만 조회 (페이지네이션 지원)
    pub async fn get_active_projects(
        &self,
        page: i32,
        page_size: i32,
        sort_by: &str,
        sort_order: &str,
    ) -> Result<ProjectListResponse, ServiceError> {
        let projects = self.project_service.get_active_projects_paginated(page, page_size, sort_by, sort_order).await?;
        let total = self.project_service.count_active_projects().await?;
        
        let total_pages = if total > 0 {
            ((total as f64) / (page_size as f64)).ceil() as i32
        } else {
            0
        };
        
        Ok(ProjectListResponse {
            projects: projects.into_iter().map(|p| p.into()).collect(),
            pagination: PaginationInfo {
                page,
                page_size,
                total,
                total_pages,
            },
        })
    }

    /// 프로젝트 수정
    pub async fn update_project(&self, project_id: i32, request: UpdateProjectRequest) -> Result<ProjectResponse, ServiceError> {
        let mut update = UpdateProject {
            name: request.name,
            description: request.description,
            sponsor: request.sponsor,
            start_date: request.start_date,
            end_date: request.end_date,
            status: None, // TODO: 문자열을 enum으로 변환
            auto_complete: request.auto_complete,
            is_active: request.is_active,
        };

        // 문자열 status를 ProjectStatus enum으로 변환
        if let Some(status_str) = request.status {
            update.status = match status_str.as_str() {
                "Planning" | "PLANNING" => Some(ProjectStatus::Planning),
                "Active" | "ACTIVE" => Some(ProjectStatus::Active),
                "Completed" | "COMPLETED" => Some(ProjectStatus::Completed),
                "Suspended" | "SUSPENDED" => Some(ProjectStatus::Suspended),
                "Cancelled" | "CANCELLED" => Some(ProjectStatus::Cancelled),
                "PendingCompletion" | "PENDING_COMPLETION" => Some(ProjectStatus::PendingCompletion),
                "OverPlanning" | "OVER_PLANNING" => Some(ProjectStatus::OverPlanning),
                _ => None,
            };
        }

        let project = self.project_service.update_project(project_id, update).await?;
        Ok(project.into())
    }

    /// 프로젝트 활성화
    pub async fn activate_project(&self, project_id: i32) -> Result<ProjectResponse, ServiceError> {
        let project = self.project_service.activate_project(project_id).await?;
        Ok(project.into())
    }

    /// 프로젝트 비활성화
    pub async fn deactivate_project(&self, project_id: i32) -> Result<ProjectResponse, ServiceError> {
        let project = self.project_service.deactivate_project(project_id).await?;
        Ok(project.into())
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
