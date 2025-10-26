use async_trait::async_trait;
use sqlx::PgPool;
use crate::domain::entities::{Project, NewProject, UpdateProject};
use crate::application::dto::project_dto::ProjectListQuery;

#[async_trait]
pub trait ProjectRepository: Send + Sync {
    async fn find_by_id(&self, id: i32) -> Result<Option<Project>, sqlx::Error>;
    async fn find_by_name(&self, name: &str) -> Result<Option<Project>, sqlx::Error>;
    async fn find_all(&self) -> Result<Vec<Project>, sqlx::Error>;
    async fn find_active(&self) -> Result<Vec<Project>, sqlx::Error>;
    async fn create(&self, new_project: NewProject) -> Result<Project, sqlx::Error>;
    async fn update(&self, id: i32, update: &UpdateProject) -> Result<Option<Project>, sqlx::Error>;
    async fn set_active(&self, id: i32, is_active: bool) -> Result<bool, sqlx::Error>;
    async fn delete(&self, id: i32) -> Result<bool, sqlx::Error>;
    
    // 페이지네이션 메서드
    async fn find_with_pagination(
        &self,
        page: i32,
        page_size: i32,
        sort_by: &str,
        sort_order: &str,
    ) -> Result<Vec<Project>, sqlx::Error>;
    
    async fn find_active_with_pagination(
        &self,
        page: i32,
        page_size: i32,
        sort_by: &str,
        sort_order: &str,
    ) -> Result<Vec<Project>, sqlx::Error>;
    
    async fn find_with_filter(
        &self,
        query: &ProjectListQuery,
    ) -> Result<Vec<Project>, sqlx::Error>;
    
    async fn count_all(&self) -> Result<i64, sqlx::Error>;
    
    async fn count_active(&self) -> Result<i64, sqlx::Error>;
    
    async fn count_with_filter(
        &self,
        query: &ProjectListQuery,
    ) -> Result<i64, sqlx::Error>;
    
    fn pool(&self) -> &PgPool;
}
