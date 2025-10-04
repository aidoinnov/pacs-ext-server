use async_trait::async_trait;
use sqlx::PgPool;
use crate::domain::entities::{Project, NewProject};
use crate::domain::repositories::ProjectRepository;

pub struct ProjectRepositoryImpl {
    pool: PgPool,
}

impl ProjectRepositoryImpl {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProjectRepository for ProjectRepositoryImpl {
    async fn find_by_id(&self, id: i32) -> Result<Option<Project>, sqlx::Error> {
        sqlx::query_as::<_, Project>(
            "SELECT id, name, description, is_active, created_at
             FROM security_project
             WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    async fn find_by_name(&self, name: &str) -> Result<Option<Project>, sqlx::Error> {
        sqlx::query_as::<_, Project>(
            "SELECT id, name, description, is_active, created_at
             FROM security_project
             WHERE name = $1"
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
    }

    async fn find_all(&self) -> Result<Vec<Project>, sqlx::Error> {
        sqlx::query_as::<_, Project>(
            "SELECT id, name, description, is_active, created_at
             FROM security_project
             ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await
    }

    async fn find_active(&self) -> Result<Vec<Project>, sqlx::Error> {
        sqlx::query_as::<_, Project>(
            "SELECT id, name, description, is_active, created_at
             FROM security_project
             WHERE is_active = true
             ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await
    }

    async fn create(&self, new_project: NewProject) -> Result<Project, sqlx::Error> {
        sqlx::query_as::<_, Project>(
            "INSERT INTO security_project (name, description)
             VALUES ($1, $2)
             RETURNING id, name, description, is_active, created_at"
        )
        .bind(new_project.name)
        .bind(new_project.description)
        .fetch_one(&self.pool)
        .await
    }

    async fn update(&self, id: i32, new_project: NewProject) -> Result<Option<Project>, sqlx::Error> {
        sqlx::query_as::<_, Project>(
            "UPDATE security_project
             SET name = $2, description = $3
             WHERE id = $1
             RETURNING id, name, description, is_active, created_at"
        )
        .bind(id)
        .bind(new_project.name)
        .bind(new_project.description)
        .fetch_optional(&self.pool)
        .await
    }

    async fn set_active(&self, id: i32, is_active: bool) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "UPDATE security_project SET is_active = $2 WHERE id = $1"
        )
        .bind(id)
        .bind(is_active)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    async fn delete(&self, id: i32) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM security_project WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    fn pool(&self) -> &PgPool {
        &self.pool
    }
}
