use crate::domain::entities::{NewPermission, Permission};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

#[async_trait]
pub trait PermissionRepository: Send + Sync {
    async fn find_by_id(&self, id: i32) -> Result<Option<Permission>, sqlx::Error>;
    async fn find_by_resource_and_action(
        &self,
        resource_type: &str,
        action: &str,
    ) -> Result<Option<Permission>, sqlx::Error>;
    async fn find_all(&self) -> Result<Vec<Permission>, sqlx::Error>;
    async fn find_by_resource_type(
        &self,
        resource_type: &str,
    ) -> Result<Vec<Permission>, sqlx::Error>;
    async fn create(&self, new_permission: NewPermission) -> Result<Permission, sqlx::Error>;
    async fn delete(&self, id: i32) -> Result<bool, sqlx::Error>;
    async fn get_all_permissions_updated_at(&self) -> Result<DateTime<Utc>, sqlx::Error>;
    fn pool(&self) -> &PgPool;
}
