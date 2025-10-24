//! # 프로젝트-사용자 매트릭스 Use Case 모듈
//! 
//! 이 모듈은 프로젝트-사용자 역할 매트릭스 조회를 위한 Use Case를 정의합니다.
//! 매트릭스는 관리 UI에서 테이블 형태로 보여주기 위한 것으로,
//! 열은 사용자 목록, 행은 프로젝트 목록이며, 각 셀에는 해당 프로젝트에서 사용자의 역할이 표시됩니다.

use std::sync::Arc;
use crate::domain::ServiceError;
use crate::domain::services::{ProjectService, UserService};
use crate::domain::entities::ProjectStatus;
use crate::application::dto::project_user_matrix_dto::*;

/// 프로젝트-사용자 매트릭스 Use Case
/// 
/// 프로젝트와 사용자 데이터를 조합하여 매트릭스 형태의 데이터를 생성합니다.
pub struct ProjectUserMatrixUseCase<P, U>
where
    P: ProjectService,
    U: UserService,
{
    project_service: Arc<P>,
    user_service: Arc<U>,
}

impl<P, U> ProjectUserMatrixUseCase<P, U>
where
    P: ProjectService,
    U: UserService,
{
    /// 새로운 매트릭스 Use Case 인스턴스 생성
    pub fn new(project_service: Arc<P>, user_service: Arc<U>) -> Self {
        Self {
            project_service,
            user_service,
        }
    }

    /// 프로젝트-사용자 역할 매트릭스 조회
    /// 
    /// # Parameters
    /// - `params`: 매트릭스 조회 파라미터 (페이지네이션, 필터 등)
    /// 
    /// # Returns
    /// - `ProjectUserMatrixResponse`: 매트릭스 데이터와 페이지네이션 정보
    pub async fn get_matrix(
        &self,
        params: MatrixQueryParams,
    ) -> Result<ProjectUserMatrixResponse, ServiceError> {
        // 파라미터 검증 및 기본값 설정
        let project_page = params.project_page.unwrap_or(1).max(1);
        let project_page_size = params.project_page_size.unwrap_or(10).clamp(1, 50);
        let user_page = params.user_page.unwrap_or(1).max(1);
        let user_page_size = params.user_page_size.unwrap_or(10).clamp(1, 50);

        // 프로젝트 상태 파싱
        let statuses = params.project_status.as_ref().map(|statuses| {
            statuses
                .iter()
                .filter_map(|s| match s.as_str() {
                    "PREPARING" => Some(ProjectStatus::Preparing),
                    "IN_PROGRESS" => Some(ProjectStatus::InProgress),
                    "COMPLETED" => Some(ProjectStatus::Completed),
                    "ON_HOLD" => Some(ProjectStatus::OnHold),
                    "CANCELLED" => Some(ProjectStatus::Cancelled),
                    _ => None,
                })
                .collect()
        });

        // 필터된 프로젝트 조회 (페이지네이션)
        let (projects, project_total_count) = self
            .project_service
            .get_projects_with_status_filter(
                statuses,
                params.project_ids.clone(),
                project_page,
                project_page_size,
            )
            .await?;

        // 필터된 사용자 조회 (페이지네이션)
        let (users, user_total_count) = self
            .user_service
            .get_users_with_filter(
                params.user_ids.clone(),
                user_page,
                user_page_size,
            )
            .await?;

        // 프로젝트나 사용자가 비어있으면 빈 매트릭스 반환
        if projects.is_empty() || users.is_empty() {
            return Ok(ProjectUserMatrixResponse {
                matrix: vec![],
                users: vec![],
                pagination: MatrixPagination {
                    project_page,
                    project_page_size,
                    project_total_count,
                    project_total_pages: 0,
                    user_page,
                    user_page_size,
                    user_total_count,
                    user_total_pages: 0,
                },
            });
        }

        // 매트릭스용 사용자-프로젝트-역할 관계 조회
        let project_ids: Vec<i32> = projects.iter().map(|p| p.id).collect();
        let user_ids: Vec<i32> = users.iter().map(|u| u.id).collect();

        let relationships = self
            .project_service
            .get_user_project_roles_matrix(project_ids.clone(), user_ids.clone())
            .await?;

        // 매트릭스 행 구성
        let matrix: Vec<ProjectUserMatrixRow> = projects
            .iter()
            .map(|project| {
                let user_roles: Vec<UserRoleCell> = users
                    .iter()
                    .map(|user| {
                        // 해당 사용자-프로젝트 조합의 역할 정보 찾기
                        let role_info = relationships
                            .iter()
                            .find(|r| r.project_id == project.id && r.user_id == user.id);

                        UserRoleCell {
                            user_id: user.id,
                            username: user.username.clone(),
                            email: user.email.clone(),
                            role_id: role_info.and_then(|r| r.role_id),
                            role_name: role_info.and_then(|r| r.role_name.clone()),
                        }
                    })
                    .collect();

                ProjectUserMatrixRow {
                    project_id: project.id,
                    project_name: project.name.clone(),
                    description: project.description.clone(),
                    status: format!("{:?}", project.status).to_uppercase(),
                    user_roles,
                }
            })
            .collect();

        // 사용자 정보 목록 구성 (열 헤더용)
        let user_info: Vec<UserInfo> = users
            .iter()
            .map(|u| UserInfo {
                user_id: u.id,
                username: u.username.clone(),
                email: u.email.clone(),
                full_name: u.full_name.clone(),
            })
            .collect();

        // 페이지네이션 계산
        let project_total_pages =
            ((project_total_count + project_page_size as i64 - 1) / project_page_size as i64) as i32;
        let user_total_pages =
            ((user_total_count + user_page_size as i64 - 1) / user_page_size as i64) as i32;

        Ok(ProjectUserMatrixResponse {
            matrix,
            users: user_info,
            pagination: MatrixPagination {
                project_page,
                project_page_size,
                project_total_count,
                project_total_pages,
                user_page,
                user_page_size,
                user_total_count,
                user_total_pages,
            },
        })
    }
}
