use async_trait::async_trait;
use sqlx::PgPool;
use crate::domain::entities::{Role, NewRole, RoleScope};
use crate::domain::repositories::RoleRepository;

#[derive(Clone)]
pub struct RoleRepositoryImpl {
    pool: PgPool,
}

impl RoleRepositoryImpl {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RoleRepository for RoleRepositoryImpl {
    async fn find_by_id(&self, id: i32) -> Result<Option<Role>, sqlx::Error> {
        sqlx::query_as::<_, Role>(
            "SELECT id, name, description, scope, created_at
             FROM security_role
             WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    async fn find_by_name(&self, name: &str) -> Result<Option<Role>, sqlx::Error> {
        sqlx::query_as::<_, Role>(
            "SELECT id, name, description, scope, created_at
             FROM security_role
             WHERE name = $1"
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
    }

    async fn find_all(&self) -> Result<Vec<Role>, sqlx::Error> {
        sqlx::query_as::<_, Role>(
            "SELECT id, name, description, scope, created_at
             FROM security_role
             ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await
    }

    async fn find_by_scope(&self, scope: &str) -> Result<Vec<Role>, sqlx::Error> {
        sqlx::query_as::<_, Role>(
            "SELECT id, name, description, scope, created_at
             FROM security_role
             WHERE scope = $1
             ORDER BY created_at DESC"
        )
        .bind(scope)
        .fetch_all(&self.pool)
        .await
    }

    async fn create(&self, new_role: NewRole) -> Result<Role, sqlx::Error> {
        let scope_str = match new_role.scope {
            RoleScope::Global => "GLOBAL",
            RoleScope::Project => "PROJECT",
        };

        sqlx::query_as::<_, Role>(
            "INSERT INTO security_role (name, description, scope)
             VALUES ($1, $2, $3)
             RETURNING id, name, description, scope, created_at"
        )
        .bind(new_role.name)
        .bind(new_role.description)
        .bind(scope_str)
        .fetch_one(&self.pool)
        .await
    }

    async fn update(&self, id: i32, new_role: NewRole) -> Result<Option<Role>, sqlx::Error> {
        let scope_str = match new_role.scope {
            RoleScope::Global => "GLOBAL",
            RoleScope::Project => "PROJECT",
        };

        sqlx::query_as::<_, Role>(
            "UPDATE security_role
             SET name = $2, description = $3, scope = $4
             WHERE id = $1
             RETURNING id, name, description, scope, created_at"
        )
        .bind(id)
        .bind(new_role.name)
        .bind(new_role.description)
        .bind(scope_str)
        .fetch_optional(&self.pool)
        .await
    }

    async fn delete(&self, id: i32) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM security_role WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    fn pool(&self) -> &PgPool {
        &self.pool
    }
}
