//! # 유저-프로젝트 매트릭스 Use Case 모듈
//! 
//! 이 모듈은 유저 중심 매트릭스 API를 위한 비즈니스 로직을 제공합니다.
//! 유저를 행으로, 프로젝트를 열로 표시하는 매트릭스 데이터를 생성합니다.

use std::sync::Arc;
use crate::application::dto::user_project_matrix_dto::{
    UserProjectMatrixResponse, UserProjectMatrixRow, ProjectRoleCell, ProjectInfo,
    UserProjectMatrixPagination, UserProjectMatrixQueryParams
};
use crate::domain::services::{UserService, ProjectService};
use crate::domain::ServiceError;

/// 유저-프로젝트 매트릭스 Use Case
/// 
/// 유저 중심 매트릭스 데이터를 생성하는 비즈니스 로직을 담당합니다.
#[derive(Clone)]
pub struct UserProjectMatrixUseCase<U, P>
where
    U: UserService,
    P: ProjectService,
{
    user_service: Arc<U>,
    project_service: Arc<P>,
}

impl<U, P> UserProjectMatrixUseCase<U, P>
where
    U: UserService,
    P: ProjectService,
{
    /// 새로운 Use Case 인스턴스 생성
    pub fn new(user_service: Arc<U>, project_service: Arc<P>) -> Self {
        Self {
            user_service,
            project_service,
        }
    }

    /// 유저-프로젝트 매트릭스 조회
    /// 
    /// 유저를 행으로, 프로젝트를 열로 표시하는 매트릭스 데이터를 생성합니다.
    /// 이중 페이지네이션(유저/프로젝트)과 다양한 필터링 옵션을 지원합니다.
    pub async fn get_matrix(
        &self,
        params: UserProjectMatrixQueryParams,
    ) -> Result<UserProjectMatrixResponse, ServiceError> {
        // 파라미터 기본값 설정
        let user_page = params.user_page.unwrap_or(1);
        let user_page_size = params.user_page_size.unwrap_or(10).min(50);
        let project_page = params.project_page.unwrap_or(1);
        let project_page_size = params.project_page_size.unwrap_or(10).min(50);
        let user_sort_by = params.user_sort_by.unwrap_or_else(|| "username".to_string());
        let user_sort_order = params.user_sort_order.unwrap_or_else(|| "asc".to_string());

        // 1. 유저 목록 조회 (정렬, 필터링, 페이지네이션)
        let (users, user_total_count) = self.user_service
            .get_users_with_sorting(
                user_page,
                user_page_size,
                &user_sort_by,
                &user_sort_order,
                params.user_search.as_deref(),
                params.user_ids.as_deref(),
            )
            .await?;

        // 2. 프로젝트 목록 조회 (필터링, 페이지네이션)
        let (projects, project_total_count) = self.project_service
            .get_projects_with_status_filter(
                None, // project_status는 현재 지원하지 않음
                params.project_ids,
                project_page,
                project_page_size,
            )
            .await?;

        // 3. 각 유저의 프로젝트 역할 매핑 조회
        let mut matrix_rows = Vec::new();
        
        for user in users {
            let mut project_roles = Vec::new();
            
            for project in &projects {
                // 유저의 프로젝트 역할 조회
                let membership = self.user_service
                    .get_project_membership(user.id, project.id)
                    .await?;
                
                let project_role = ProjectRoleCell {
                    project_id: project.id,
                    project_name: project.name.clone(),
                    role_id: membership.as_ref().and_then(|m| m.role_id),
                    role_name: membership.as_ref().and_then(|m| m.role_name.clone()),
                };
                
                project_roles.push(project_role);
            }
            
            let matrix_row = UserProjectMatrixRow {
                user_id: user.id,
                username: user.username.clone(),
                email: user.email.clone(),
                full_name: user.full_name.clone(),
                project_roles,
            };
            
            matrix_rows.push(matrix_row);
        }

        // 4. 프로젝트 정보 목록 생성 (열 헤더용)
        let project_infos: Vec<ProjectInfo> = projects
            .into_iter()
            .map(|project| ProjectInfo {
                project_id: project.id,
                project_name: project.name,
                description: project.description,
                status: format!("{:?}", project.status),
            })
            .collect();

        // 5. 페이지네이션 정보 계산
        let user_total_pages = ((user_total_count as f64) / (user_page_size as f64)).ceil() as i32;
        let project_total_pages = ((project_total_count as f64) / (project_page_size as f64)).ceil() as i32;

        let pagination = UserProjectMatrixPagination {
            user_page,
            user_page_size,
            user_total_count,
            user_total_pages,
            project_page,
            project_page_size,
            project_total_count,
            project_total_pages,
        };

        // 6. 최종 응답 구성
        Ok(UserProjectMatrixResponse {
            matrix: matrix_rows,
            projects: project_infos,
            pagination,
        })
    }
}
