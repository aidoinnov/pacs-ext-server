use async_trait::async_trait;
use crate::domain::entities::{Project, NewProject, User, Role};
use crate::domain::repositories::{ProjectRepository, UserRepository, RoleRepository};
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

    // === 멤버 관리 ===

    /// 프로젝트의 멤버 목록 조회
    async fn get_project_members(&self, project_id: i32) -> Result<Vec<User>, ServiceError>;

    /// 프로젝트 멤버 수 조회
    async fn count_project_members(&self, project_id: i32) -> Result<i64, ServiceError>;

    // === 역할 관리 ===

    /// 프로젝트에 역할 할당
    async fn assign_role_to_project(&self, project_id: i32, role_id: i32) -> Result<(), ServiceError>;

    /// 프로젝트에서 역할 제거
    async fn remove_role_from_project(&self, project_id: i32, role_id: i32) -> Result<(), ServiceError>;

    /// 프로젝트에 할당된 역할 목록 조회
    async fn get_project_roles(&self, project_id: i32) -> Result<Vec<Role>, ServiceError>;
}

pub struct ProjectServiceImpl<P, U, R>
where
    P: ProjectRepository,
    U: UserRepository,
    R: RoleRepository,
{
    project_repository: P,
    user_repository: U,
    role_repository: R,
}

impl<P, U, R> ProjectServiceImpl<P, U, R>
where
    P: ProjectRepository,
    U: UserRepository,
    R: RoleRepository,
{
    pub fn new(project_repository: P, user_repository: U, role_repository: R) -> Self {
        Self {
            project_repository,
            user_repository,
            role_repository,
        }
    }
}

#[async_trait]
impl<P, U, R> ProjectService for ProjectServiceImpl<P, U, R>
where
    P: ProjectRepository,
    U: UserRepository,
    R: RoleRepository,
{
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

    // === 멤버 관리 구현 ===

    async fn get_project_members(&self, project_id: i32) -> Result<Vec<User>, ServiceError> {
        // 프로젝트 존재 확인
        if self.project_repository.find_by_id(project_id).await?.is_none() {
            return Err(ServiceError::NotFound("Project not found".into()));
        }

        let members = sqlx::query_as::<_, User>(
            "SELECT u.id, u.keycloak_id, u.username, u.email, u.created_at
             FROM security_user u
             INNER JOIN security_user_project up ON u.id = up.user_id
             WHERE up.project_id = $1
             ORDER BY u.username"
        )
        .bind(project_id)
        .fetch_all(self.project_repository.pool())
        .await?;

        Ok(members)
    }

    async fn count_project_members(&self, project_id: i32) -> Result<i64, ServiceError> {
        // 프로젝트 존재 확인
        if self.project_repository.find_by_id(project_id).await?.is_none() {
            return Err(ServiceError::NotFound("Project not found".into()));
        }

        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM security_user_project WHERE project_id = $1"
        )
        .bind(project_id)
        .fetch_one(self.project_repository.pool())
        .await?;

        Ok(count)
    }

    // === 역할 관리 구현 ===

    async fn assign_role_to_project(&self, project_id: i32, role_id: i32) -> Result<(), ServiceError> {
        // 프로젝트 존재 확인
        if self.project_repository.find_by_id(project_id).await?.is_none() {
            return Err(ServiceError::NotFound("Project not found".into()));
        }

        // 역할 존재 확인
        if self.role_repository.find_by_id(role_id).await?.is_none() {
            return Err(ServiceError::NotFound("Role not found".into()));
        }

        // 이미 할당되었는지 확인
        let exists = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM security_project_role WHERE project_id = $1 AND role_id = $2"
        )
        .bind(project_id)
        .bind(role_id)
        .fetch_one(self.project_repository.pool())
        .await?;

        if exists > 0 {
            return Err(ServiceError::AlreadyExists("Role already assigned to this project".into()));
        }

        // security_project_role 테이블에 추가
        sqlx::query(
            "INSERT INTO security_project_role (project_id, role_id) VALUES ($1, $2)"
        )
        .bind(project_id)
        .bind(role_id)
        .execute(self.project_repository.pool())
        .await?;

        Ok(())
    }

    async fn remove_role_from_project(&self, project_id: i32, role_id: i32) -> Result<(), ServiceError> {
        let result = sqlx::query(
            "DELETE FROM security_project_role WHERE project_id = $1 AND role_id = $2"
        )
        .bind(project_id)
        .bind(role_id)
        .execute(self.project_repository.pool())
        .await?;

        if result.rows_affected() > 0 {
            Ok(())
        } else {
            Err(ServiceError::NotFound("Role is not assigned to this project".into()))
        }
    }

    async fn get_project_roles(&self, project_id: i32) -> Result<Vec<Role>, ServiceError> {
        // 프로젝트 존재 확인
        if self.project_repository.find_by_id(project_id).await?.is_none() {
            return Err(ServiceError::NotFound("Project not found".into()));
        }

        let roles = sqlx::query_as::<_, Role>(
            "SELECT r.id, r.name, r.description, r.created_at
             FROM security_role r
             INNER JOIN security_project_role pr ON r.id = pr.role_id
             WHERE pr.project_id = $1
             ORDER BY r.name"
        )
        .bind(project_id)
        .fetch_all(self.project_repository.pool())
        .await?;

        Ok(roles)
    }
}
