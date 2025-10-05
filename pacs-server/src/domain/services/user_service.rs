use async_trait::async_trait;
use uuid::Uuid;
use crate::domain::entities::{User, NewUser, Project};
use crate::domain::repositories::{UserRepository, ProjectRepository};

/// 사용자 관리 도메인 서비스
#[async_trait]
pub trait UserService: Send + Sync {
    /// 사용자 생성
    async fn create_user(&self, username: String, email: String, keycloak_id: Uuid) -> Result<User, ServiceError>;

    /// 사용자 조회 (ID)
    async fn get_user_by_id(&self, id: i32) -> Result<User, ServiceError>;

    /// 사용자 조회 (Keycloak ID)
    async fn get_user_by_keycloak_id(&self, keycloak_id: Uuid) -> Result<User, ServiceError>;

    /// 사용자 조회 (Username)
    async fn get_user_by_username(&self, username: &str) -> Result<User, ServiceError>;

    /// 사용자 삭제
    async fn delete_user(&self, id: i32) -> Result<(), ServiceError>;

    /// 사용자 존재 여부 확인
    async fn user_exists(&self, keycloak_id: Uuid) -> Result<bool, ServiceError>;

    // === 프로젝트 멤버십 관리 ===

    /// 사용자를 프로젝트에 추가
    async fn add_user_to_project(&self, user_id: i32, project_id: i32) -> Result<(), ServiceError>;

    /// 프로젝트에서 사용자 제거
    async fn remove_user_from_project(&self, user_id: i32, project_id: i32) -> Result<(), ServiceError>;

    /// 사용자가 속한 프로젝트 목록 조회
    async fn get_user_projects(&self, user_id: i32) -> Result<Vec<Project>, ServiceError>;

    /// 사용자가 프로젝트 멤버인지 확인
    async fn is_project_member(&self, user_id: i32, project_id: i32) -> Result<bool, ServiceError>;
}

pub struct UserServiceImpl<U, P>
where
    U: UserRepository,
    P: ProjectRepository,
{
    user_repository: U,
    project_repository: P,
}

impl<U, P> UserServiceImpl<U, P>
where
    U: UserRepository,
    P: ProjectRepository,
{
    pub fn new(user_repository: U, project_repository: P) -> Self {
        Self {
            user_repository,
            project_repository,
        }
    }
}

#[async_trait]
impl<U, P> UserService for UserServiceImpl<U, P>
where
    U: UserRepository,
    P: ProjectRepository,
{
    async fn create_user(&self, username: String, email: String, keycloak_id: Uuid) -> Result<User, ServiceError> {
        // 중복 체크
        if let Some(_) = self.user_repository.find_by_keycloak_id(keycloak_id).await? {
            return Err(ServiceError::AlreadyExists("User with this keycloak_id already exists".into()));
        }

        if let Some(_) = self.user_repository.find_by_username(&username).await? {
            return Err(ServiceError::AlreadyExists("Username already taken".into()));
        }

        // 이메일 형식 검증
        if !email.contains('@') {
            return Err(ServiceError::ValidationError("Invalid email format".into()));
        }

        let new_user = NewUser {
            keycloak_id,
            username,
            email,
        };

        Ok(self.user_repository.create(new_user).await?)
    }

    async fn get_user_by_id(&self, id: i32) -> Result<User, ServiceError> {
        self.user_repository
            .find_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound("User not found".into()))
    }

    async fn get_user_by_keycloak_id(&self, keycloak_id: Uuid) -> Result<User, ServiceError> {
        self.user_repository
            .find_by_keycloak_id(keycloak_id)
            .await?
            .ok_or(ServiceError::NotFound("User not found".into()))
    }

    async fn get_user_by_username(&self, username: &str) -> Result<User, ServiceError> {
        self.user_repository
            .find_by_username(username)
            .await?
            .ok_or(ServiceError::NotFound("User not found".into()))
    }

    async fn delete_user(&self, id: i32) -> Result<(), ServiceError> {
        let deleted = self.user_repository.delete(id).await?;
        if deleted {
            Ok(())
        } else {
            Err(ServiceError::NotFound("User not found".into()))
        }
    }

    async fn user_exists(&self, keycloak_id: Uuid) -> Result<bool, ServiceError> {
        Ok(self.user_repository.find_by_keycloak_id(keycloak_id).await?.is_some())
    }

    // === 프로젝트 멤버십 관리 구현 ===

    async fn add_user_to_project(&self, user_id: i32, project_id: i32) -> Result<(), ServiceError> {
        // INSERT with ON CONFLICT - Race condition 방지
        let result = sqlx::query(
            "INSERT INTO security_user_project (user_id, project_id)
             SELECT $1, $2
             WHERE EXISTS(SELECT 1 FROM security_user WHERE id = $1)
               AND EXISTS(SELECT 1 FROM security_project WHERE id = $2)
             ON CONFLICT (user_id, project_id) DO NOTHING
             RETURNING user_id"
        )
        .bind(user_id)
        .bind(project_id)
        .fetch_optional(self.user_repository.pool())
        .await?;

        match result {
            Some(_) => Ok(()),
            None => {
                // 실패 원인 파악
                if self.user_repository.find_by_id(user_id).await?.is_none() {
                    return Err(ServiceError::NotFound("User not found".into()));
                }
                if self.project_repository.find_by_id(project_id).await?.is_none() {
                    return Err(ServiceError::NotFound("Project not found".into()));
                }
                Err(ServiceError::AlreadyExists("User is already a member of this project".into()))
            }
        }
    }

    async fn remove_user_from_project(&self, user_id: i32, project_id: i32) -> Result<(), ServiceError> {
        let result = sqlx::query(
            "DELETE FROM security_user_project WHERE user_id = $1 AND project_id = $2"
        )
        .bind(user_id)
        .bind(project_id)
        .execute(self.user_repository.pool())
        .await?;

        if result.rows_affected() > 0 {
            Ok(())
        } else {
            Err(ServiceError::NotFound("User is not a member of this project".into()))
        }
    }

    async fn get_user_projects(&self, user_id: i32) -> Result<Vec<Project>, ServiceError> {
        // 사용자 존재 확인
        if self.user_repository.find_by_id(user_id).await?.is_none() {
            return Err(ServiceError::NotFound("User not found".into()));
        }

        let projects = sqlx::query_as::<_, Project>(
            "SELECT p.id, p.name, p.description, p.is_active, p.created_at
             FROM security_project p
             INNER JOIN security_user_project up ON p.id = up.project_id
             WHERE up.user_id = $1
             ORDER BY p.name"
        )
        .bind(user_id)
        .fetch_all(self.user_repository.pool())
        .await?;

        Ok(projects)
    }

    async fn is_project_member(&self, user_id: i32, project_id: i32) -> Result<bool, ServiceError> {
        let result = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM security_user_project WHERE user_id = $1 AND project_id = $2"
        )
        .bind(user_id)
        .bind(project_id)
        .fetch_one(self.user_repository.pool())
        .await?;

        Ok(result > 0)
    }
}

#[derive(Debug)]
pub enum ServiceError {
    NotFound(String),
    AlreadyExists(String),
    ValidationError(String),
    DatabaseError(String),
    Unauthorized(String),
}

impl From<sqlx::Error> for ServiceError {
    fn from(err: sqlx::Error) -> Self {
        ServiceError::DatabaseError(err.to_string())
    }
}

impl std::fmt::Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceError::NotFound(msg) => write!(f, "Not found: {}", msg),
            ServiceError::AlreadyExists(msg) => write!(f, "Already exists: {}", msg),
            ServiceError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            ServiceError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            ServiceError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
        }
    }
}

impl std::error::Error for ServiceError {}
