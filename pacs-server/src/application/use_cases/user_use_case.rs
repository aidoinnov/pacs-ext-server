use crate::application::dto::{
    CreateUserRequest, UserResponse, UserListResponse, AddProjectMemberRequest,
    UserProjectsResponse, ProjectSummary,
};
use crate::domain::services::{UserService, ServiceError};

/// 사용자 관리 유스케이스
pub struct UserUseCase<U: UserService> {
    user_service: U,
}

impl<U: UserService> UserUseCase<U> {
    pub fn new(user_service: U) -> Self {
        Self { user_service }
    }

    /// 사용자 생성
    pub async fn create_user(&self, request: CreateUserRequest) -> Result<UserResponse, ServiceError> {
        let user = self
            .user_service
            .create_user(request.username, request.email, request.keycloak_id)
            .await?;

        Ok(UserResponse {
            id: user.id,
            keycloak_id: user.keycloak_id,
            username: user.username,
            email: user.email,
            created_at: user.created_at,
        })
    }

    /// 사용자 조회 (ID)
    pub async fn get_user_by_id(&self, user_id: i32) -> Result<UserResponse, ServiceError> {
        let user = self.user_service.get_user_by_id(user_id).await?;

        Ok(UserResponse {
            id: user.id,
            keycloak_id: user.keycloak_id,
            username: user.username,
            email: user.email,
            created_at: user.created_at,
        })
    }

    /// 사용자 조회 (Username)
    pub async fn get_user_by_username(&self, username: &str) -> Result<UserResponse, ServiceError> {
        let user = self.user_service.get_user_by_username(username).await?;

        Ok(UserResponse {
            id: user.id,
            keycloak_id: user.keycloak_id,
            username: user.username,
            email: user.email,
            created_at: user.created_at,
        })
    }

    /// 사용자 삭제
    pub async fn delete_user(&self, user_id: i32) -> Result<(), ServiceError> {
        self.user_service.delete_user(user_id).await
    }

    /// 프로젝트 멤버 추가
    pub async fn add_project_member(&self, request: AddProjectMemberRequest) -> Result<(), ServiceError> {
        self.user_service
            .add_user_to_project(request.user_id, request.project_id)
            .await
    }

    /// 프로젝트 멤버 제거
    pub async fn remove_project_member(&self, user_id: i32, project_id: i32) -> Result<(), ServiceError> {
        self.user_service
            .remove_user_from_project(user_id, project_id)
            .await
    }

    /// 사용자 프로젝트 목록 조회
    pub async fn get_user_projects(&self, user_id: i32) -> Result<UserProjectsResponse, ServiceError> {
        let projects = self.user_service.get_user_projects(user_id).await?;

        let project_summaries = projects
            .into_iter()
            .map(|p| ProjectSummary {
                id: p.id,
                name: p.name,
                description: p.description,
                is_active: p.is_active,
            })
            .collect();

        Ok(UserProjectsResponse {
            user_id,
            projects: project_summaries,
        })
    }

    /// 프로젝트 멤버십 확인
    pub async fn is_project_member(&self, user_id: i32, project_id: i32) -> Result<bool, ServiceError> {
        self.user_service.is_project_member(user_id, project_id).await
    }
}
