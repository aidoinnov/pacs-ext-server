use async_trait::async_trait;
use uuid::Uuid;
use sqlx::PgPool;
use crate::domain::entities::{User, NewUser, UpdateUser};

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: i32) -> Result<Option<User>, sqlx::Error>;
    async fn find_by_keycloak_id(&self, keycloak_id: Uuid) -> Result<Option<User>, sqlx::Error>;
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, sqlx::Error>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error>;
    async fn find_all(&self) -> Result<Vec<User>, sqlx::Error>;
    async fn create(&self, new_user: NewUser) -> Result<User, sqlx::Error>;
    async fn update(&self, update_user: &UpdateUser) -> Result<User, sqlx::Error>;
    async fn delete(&self, id: i32) -> Result<bool, sqlx::Error>;
    fn pool(&self) -> &PgPool;
}
