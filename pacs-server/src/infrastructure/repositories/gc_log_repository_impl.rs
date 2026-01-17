
use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;
use crate::domain::entities::gc_deletion_log::{GcDeletionLog, NewGcDeletionLog};
use crate::domain::repositories::GcLogRepository;

pub struct GcLogRepositoryImpl {
    pool: Arc<PgPool>
}

impl GcLogRepositoryImpl {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}



#[async_trait]
impl GcLogRepository for GcLogRepositoryImpl {
async fn insert(&self, log: NewGcDeletionLog) -> Result<GcDeletionLog, sqlx::Error> {
        sqlx::query_as::<_, GcDeletionLog>(
            r#"
            INSERT INTO gc_deletion_log (
                annotation_id, snapshot_image_key, file_size,
                dry_run, status, error_message
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#
        )
        .bind(log.annotation_id)
        .bind(log.snapshot_image_key)
        .bind(log.file_size)
        .bind(log.dry_run)
        .bind(log.status)
        .bind(log.error_message)
        .fetch_one(self.pool.as_ref())
        .await
    }

    async fn find_by_date_range(
        &self,
        start_date: chrono::DateTime<chrono::Utc>,
        end_date: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<GcDeletionLog>, sqlx::Error> {
        sqlx::query_as::<_, GcDeletionLog>(
            r#"
            SELECT * FROM gc_deletion_log
            WHERE deleted_at BETWEEN $1 AND $2
            ORDER BY deleted_at DESC
            "#
        )
        .bind(start_date)
        .bind(end_date)
        .fetch_all(self.pool.as_ref())
        .await
    }
}
