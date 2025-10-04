use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use crate::domain::entities::{User, NewUser};
use crate::domain::repositories::UserRepository;

pub struct UserRepositoryImpl {
    pool: PgPool,
}

impl UserRepositoryImpl {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for UserRepositoryImpl {
    async fn find_by_id(&self, id: i32) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "SELECT id, keycloak_id, username, email, created_at
             FROM security_user
             WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    async fn find_by_keycloak_id(&self, keycloak_id: Uuid) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "SELECT id, keycloak_id, username, email, created_at
             FROM security_user
             WHERE keycloak_id = $1"
        )
        .bind(keycloak_id)
        .fetch_optional(&self.pool)
        .await
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "SELECT id, keycloak_id, username, email, created_at
             FROM security_user
             WHERE username = $1"
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "SELECT id, keycloak_id, username, email, created_at
             FROM security_user
             WHERE email = $1"
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
    }

    async fn find_all(&self) -> Result<Vec<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "SELECT id, keycloak_id, username, email, created_at
             FROM security_user
             ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await
    }

    async fn create(&self, new_user: NewUser) -> Result<User, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "INSERT INTO security_user (keycloak_id, username, email)
             VALUES ($1, $2, $3)
             RETURNING id, keycloak_id, username, email, created_at"
        )
        .bind(new_user.keycloak_id)
        .bind(new_user.username)
        .bind(new_user.email)
        .fetch_one(&self.pool)
        .await
    }

    async fn delete(&self, id: i32) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM security_user WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}
