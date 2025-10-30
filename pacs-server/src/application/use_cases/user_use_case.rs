use crate::application::dto::{
    CreateUserRequest, UpdateUserRequest, UserResponse, AddProjectMemberRequest,
    ProjectSummary,
};
use crate::application::dto::user_dto::UserProjectsResponse;
use crate::domain::services::UserService;
use crate::domain::ServiceError;
use crate::domain::entities::User;

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
            .create_user(
                request.username, 
                request.email, 
                request.keycloak_id,
                request.full_name,
                request.organization,
                request.department,
                request.phone,
            )
            .await?;

        Ok(UserResponse::from(user))
    }

    /// 사용자 조회 (ID)
    pub async fn get_user_by_id(&self, user_id: i32) -> Result<UserResponse, ServiceError> {
        let user = self.user_service.get_user_by_id(user_id).await?;
        Ok(UserResponse::from(user))
    }

    /// 사용자 조회 (Username)
    pub async fn get_user_by_username(&self, username: &str) -> Result<UserResponse, ServiceError> {
        let user = self.user_service.get_user_by_username(username).await?;
        Ok(UserResponse::from(user))
    }

    /// 사용자 목록 조회 (페이지네이션, 정렬, 검색 지원)
    pub async fn list_users(
        &self,
        page: i32,
        page_size: i32,
        sort_by: &str,
        sort_order: &str,
        search: Option<&str>,
    ) -> Result<(Vec<User>, i64), ServiceError> {
        // 유저 서비스에 구현된 get_users_with_sorting 메서드 사용
        let result = self.user_service
            .get_users_with_sorting(page, page_size, sort_by, sort_order, search, None)
            .await?;
        Ok(result)
    }

    /// 사용자 정보 업데이트
    pub async fn update_user(&self, user_id: i32, request: UpdateUserRequest) -> Result<UserResponse, ServiceError> {
        let mut update_user = crate::domain::entities::UpdateUser::new(user_id);
        
        if let Some(email) = request.email {
            update_user = update_user.with_email(email);
        }
        if let Some(full_name) = request.full_name {
            update_user = update_user.with_full_name(full_name);
        }
        if let Some(organization) = request.organization {
            update_user = update_user.with_organization(organization);
        }
        if let Some(department) = request.department {
            update_user = update_user.with_department(department);
        }
        if let Some(phone) = request.phone {
            update_user = update_user.with_phone(phone);
        }

        let user = self.user_service.update_user(update_user).await?;
        Ok(UserResponse::from(user))
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
