use async_trait::async_trait;
use crate::domain::entities::{Project, NewProject};
use crate::domain::repositories::ProjectRepository;
use super::user_service::ServiceError;

/// 프로젝트 관리 도메인 서비스
#[async_trait]
pub trait ProjectService: Send + Sync {
    /// 프로젝트 생성
    async fn create_project(&self, name: String, description: Option<String>) -> Result<Project, ServiceError>;

    /// 프로젝트 조회
    async fn get_project(&self, id: i32) -> Result<Project, ServiceError>;

    /// 프로젝트 이름으로 조회
    async fn get_project_by_name(&self, name: &str) -> Result<Project, ServiceError>;

    /// 모든 프로젝트 조회
    async fn get_all_projects(&self) -> Result<Vec<Project>, ServiceError>;

    /// 활성화된 프로젝트만 조회
    async fn get_active_projects(&self) -> Result<Vec<Project>, ServiceError>;

    /// 프로젝트 활성화
    async fn activate_project(&self, id: i32) -> Result<Project, ServiceError>;

    /// 프로젝트 비활성화
    async fn deactivate_project(&self, id: i32) -> Result<Project, ServiceError>;

    /// 프로젝트 삭제
    async fn delete_project(&self, id: i32) -> Result<(), ServiceError>;
}

pub struct ProjectServiceImpl<R: ProjectRepository> {
    project_repository: R,
}

impl<R: ProjectRepository> ProjectServiceImpl<R> {
    pub fn new(project_repository: R) -> Self {
        Self { project_repository }
    }
}

#[async_trait]
impl<R: ProjectRepository> ProjectService for ProjectServiceImpl<R> {
    async fn create_project(&self, name: String, description: Option<String>) -> Result<Project, ServiceError> {
        // 프로젝트 이름 중복 체크
        if let Some(_) = self.project_repository.find_by_name(&name).await? {
            return Err(ServiceError::AlreadyExists("Project name already exists".into()));
        }

        // 프로젝트 이름 검증
        if name.trim().is_empty() {
            return Err(ServiceError::ValidationError("Project name cannot be empty".into()));
        }

        if name.len() > 255 {
            return Err(ServiceError::ValidationError("Project name too long (max 255 characters)".into()));
        }

        let new_project = NewProject {
            name,
            description,
        };

        Ok(self.project_repository.create(new_project).await?)
    }

    async fn get_project(&self, id: i32) -> Result<Project, ServiceError> {
        self.project_repository
            .find_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound("Project not found".into()))
    }

    async fn get_project_by_name(&self, name: &str) -> Result<Project, ServiceError> {
        self.project_repository
            .find_by_name(name)
            .await?
            .ok_or(ServiceError::NotFound("Project not found".into()))
    }

    async fn get_all_projects(&self) -> Result<Vec<Project>, ServiceError> {
        Ok(self.project_repository.find_all().await?)
    }

    async fn get_active_projects(&self) -> Result<Vec<Project>, ServiceError> {
        Ok(self.project_repository.find_active().await?)
    }

    async fn activate_project(&self, id: i32) -> Result<Project, ServiceError> {
        let updated = self.project_repository.set_active(id, true).await?;
        if updated {
            self.get_project(id).await
        } else {
            Err(ServiceError::NotFound("Project not found".into()))
        }
    }

    async fn deactivate_project(&self, id: i32) -> Result<Project, ServiceError> {
        let updated = self.project_repository.set_active(id, false).await?;
        if updated {
            self.get_project(id).await
        } else {
            Err(ServiceError::NotFound("Project not found".into()))
        }
    }

    async fn delete_project(&self, id: i32) -> Result<(), ServiceError> {
        let deleted = self.project_repository.delete(id).await?;
        if deleted {
            Ok(())
        } else {
            Err(ServiceError::NotFound("Project not found".into()))
        }
    }
}
