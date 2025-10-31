use crate::domain::entities::{NewRole, Role};
use async_trait::async_trait;
use sqlx::PgPool;

#[async_trait]
pub trait RoleRepository: Send + Sync {
    async fn find_by_id(&self, id: i32) -> Result<Option<Role>, sqlx::Error>;
    async fn find_by_name(&self, name: &str) -> Result<Option<Role>, sqlx::Error>;
    async fn find_all(&self) -> Result<Vec<Role>, sqlx::Error>;
    async fn find_by_scope(&self, scope: &str) -> Result<Vec<Role>, sqlx::Error>;
    async fn create(&self, new_role: NewRole) -> Result<Role, sqlx::Error>;
    async fn update(&self, id: i32, new_role: NewRole) -> Result<Option<Role>, sqlx::Error>;
    async fn delete(&self, id: i32) -> Result<bool, sqlx::Error>;
    fn pool(&self) -> &PgPool;
}
