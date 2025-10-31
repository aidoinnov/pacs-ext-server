use crate::domain::entities::{NewUser, UpdateUser, User};
use crate::domain::repositories::UserRepository;
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
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
            "SELECT id, keycloak_id, username, email, full_name, organization, department, phone, 
                    created_at, updated_at, account_status, email_verified,
                    email_verification_token, email_verification_expires_at,
                    approved_by, approved_at, suspended_at, suspended_reason, deleted_at
             FROM security_user
             WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    async fn find_by_keycloak_id(&self, keycloak_id: Uuid) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "SELECT id, keycloak_id, username, email, full_name, organization, department, phone, 
                    created_at, updated_at, account_status, email_verified,
                    email_verification_token, email_verification_expires_at,
                    approved_by, approved_at, suspended_at, suspended_reason, deleted_at
             FROM security_user
             WHERE keycloak_id = $1",
        )
        .bind(keycloak_id)
        .fetch_optional(&self.pool)
        .await
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "SELECT id, keycloak_id, username, email, full_name, organization, department, phone, 
                    created_at, updated_at, account_status, email_verified,
                    email_verification_token, email_verification_expires_at,
                    approved_by, approved_at, suspended_at, suspended_reason, deleted_at
             FROM security_user
             WHERE username = $1",
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "SELECT id, keycloak_id, username, email, full_name, organization, department, phone, 
                    created_at, updated_at, account_status, email_verified,
                    email_verification_token, email_verification_expires_at,
                    approved_by, approved_at, suspended_at, suspended_reason, deleted_at
             FROM security_user
             WHERE email = $1",
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
    }

    async fn find_all(&self) -> Result<Vec<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "SELECT id, keycloak_id, username, email, full_name, organization, department, phone, 
                    created_at, updated_at, account_status, email_verified,
                    email_verification_token, email_verification_expires_at,
                    approved_by, approved_at, suspended_at, suspended_reason, deleted_at
             FROM security_user
             ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await
    }

    async fn create(&self, new_user: NewUser) -> Result<User, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "INSERT INTO security_user (keycloak_id, username, email, full_name, organization, department, phone)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             RETURNING id, keycloak_id, username, email, full_name, organization, department, phone, created_at, updated_at"
        )
        .bind(new_user.keycloak_id)
        .bind(new_user.username)
        .bind(new_user.email)
        .bind(new_user.full_name)
        .bind(new_user.organization)
        .bind(new_user.department)
        .bind(new_user.phone)
        .fetch_one(&self.pool)
        .await
    }

    async fn update(&self, update_user: &UpdateUser) -> Result<User, sqlx::Error> {
        // 모든 필드를 업데이트 (NULL인 경우 기존 값 유지)
        sqlx::query_as::<_, User>(
            "UPDATE security_user 
             SET email = COALESCE($2, email),
                 full_name = COALESCE($3, full_name),
                 organization = COALESCE($4, organization),
                 department = COALESCE($5, department),
                 phone = COALESCE($6, phone),
                 updated_at = CURRENT_TIMESTAMP
             WHERE id = $1
             RETURNING id, keycloak_id, username, email, full_name, organization, department, phone, created_at, updated_at, account_status, email_verified, email_verification_token, email_verification_expires_at, approved_by, approved_at, suspended_at, suspended_reason, deleted_at"
        )
        .bind(update_user.id)
        .bind(&update_user.email)
        .bind(&update_user.full_name)
        .bind(&update_user.organization)
        .bind(&update_user.department)
        .bind(&update_user.phone)
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

    fn pool(&self) -> &PgPool {
        &self.pool
    }
}
