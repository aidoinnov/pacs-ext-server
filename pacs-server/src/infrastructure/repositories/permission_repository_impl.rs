use async_trait::async_trait;
use sqlx::PgPool;
use crate::domain::entities::{Permission, NewPermission};
use crate::domain::repositories::PermissionRepository;

#[derive(Clone)]
pub struct PermissionRepositoryImpl {
    pool: PgPool,
}

impl PermissionRepositoryImpl {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PermissionRepository for PermissionRepositoryImpl {
    async fn find_by_id(&self, id: i32) -> Result<Option<Permission>, sqlx::Error> {
        sqlx::query_as::<_, Permission>(
            "SELECT id, resource_type, action
             FROM security_permission
             WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    async fn find_by_resource_and_action(&self, resource_type: &str, action: &str) -> Result<Option<Permission>, sqlx::Error> {
        sqlx::query_as::<_, Permission>(
            "SELECT id, resource_type, action
             FROM security_permission
             WHERE resource_type = $1 AND action = $2"
        )
        .bind(resource_type)
        .bind(action)
        .fetch_optional(&self.pool)
        .await
    }

    async fn find_all(&self) -> Result<Vec<Permission>, sqlx::Error> {
        sqlx::query_as::<_, Permission>(
            "SELECT id, resource_type, action
             FROM security_permission
             ORDER BY resource_type, action"
        )
        .fetch_all(&self.pool)
        .await
    }

    async fn find_by_resource_type(&self, resource_type: &str) -> Result<Vec<Permission>, sqlx::Error> {
        sqlx::query_as::<_, Permission>(
            "SELECT id, resource_type, action
             FROM security_permission
             WHERE resource_type = $1
             ORDER BY action"
        )
        .bind(resource_type)
        .fetch_all(&self.pool)
        .await
    }

    async fn create(&self, new_permission: NewPermission) -> Result<Permission, sqlx::Error> {
        sqlx::query_as::<_, Permission>(
            "INSERT INTO security_permission (resource_type, action)
             VALUES ($1, $2)
             RETURNING id, resource_type, action"
        )
        .bind(new_permission.resource_type)
        .bind(new_permission.action)
        .fetch_one(&self.pool)
        .await
    }

    async fn delete(&self, id: i32) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM security_permission WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    fn pool(&self) -> &PgPool {
        &self.pool
    }
}
