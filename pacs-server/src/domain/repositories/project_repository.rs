use async_trait::async_trait;
use sqlx::PgPool;
use crate::domain::entities::{Project, NewProject};

#[async_trait]
pub trait ProjectRepository: Send + Sync {
    async fn find_by_id(&self, id: i32) -> Result<Option<Project>, sqlx::Error>;
    async fn find_by_name(&self, name: &str) -> Result<Option<Project>, sqlx::Error>;
    async fn find_all(&self) -> Result<Vec<Project>, sqlx::Error>;
    async fn find_active(&self) -> Result<Vec<Project>, sqlx::Error>;
    async fn create(&self, new_project: NewProject) -> Result<Project, sqlx::Error>;
    async fn update(&self, id: i32, new_project: NewProject) -> Result<Option<Project>, sqlx::Error>;
    async fn set_active(&self, id: i32, is_active: bool) -> Result<bool, sqlx::Error>;
    async fn delete(&self, id: i32) -> Result<bool, sqlx::Error>;
    fn pool(&self) -> &PgPool;
}
