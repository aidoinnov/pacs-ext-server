use async_trait::async_trait;
use uuid::Uuid;
use crate::domain::entities::{User, NewUser};
use crate::domain::repositories::UserRepository;

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
}

pub struct UserServiceImpl<R: UserRepository> {
    user_repository: R,
}

impl<R: UserRepository> UserServiceImpl<R> {
    pub fn new(user_repository: R) -> Self {
        Self { user_repository }
    }
}

#[async_trait]
impl<R: UserRepository> UserService for UserServiceImpl<R> {
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
