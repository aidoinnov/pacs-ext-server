use crate::domain::entities::{NewRole, Role};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
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

    /// 모든 역할의 최신 변경 시점 조회 (ETag 생성용)
    async fn get_all_roles_updated_at(&self) -> Result<DateTime<Utc>, sqlx::Error>;

    /// 특정 scope의 역할 최신 변경 시점 조회 (ETag 생성용)
    async fn get_roles_updated_at_by_scope(&self, scope: &str) -> Result<DateTime<Utc>, sqlx::Error>;

    fn pool(&self) -> &PgPool;
}
