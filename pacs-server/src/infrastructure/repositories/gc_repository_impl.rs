
use async_trait::async_trait;
use sqlx::PgPool;   
use std::sync::Arc;
use crate::domain::entities::annotation::{Annotation, SnapshotUploadStatus};
use crate::domain::repositories::GcRepository;

pub struct GcRepositoryImpl {
    pool: Arc<PgPool>
}

impl GcRepositoryImpl {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl GcRepository for GcRepositoryImpl {
    async fn find_pending_snapshots(
        &self,
        grace_days: i32,
        batch_size: i32
    ) -> Result<Vec<Annotation>, sqlx::Error> {
        sqlx::query_as::<_, Annotation>(
            r#"
            SELECT
                id, project_id, user_id, study_uid, series_uid, instance_uid,
                tool_name, tool_version, data, is_shared,
                snapshot_image_key, snapshot_status, snapshot_uploaded_at,
                created_at, updated_at,
                0 as version,
                viewer_software, description,
                NULL::jsonb as measurement_values,
                label
            FROM annotation_annotation
            WHERE snapshot_status = $1
              AND updated_at < NOW() - INTERVAL '1 day' * $2
            ORDER BY updated_at ASC
            LIMIT $3
            "#
        )
        .bind(SnapshotUploadStatus::Pending)
        .bind(grace_days)
        .bind(batch_size)
        .fetch_all(self.pool.as_ref())
        .await
    }

    async fn find_failed_snapshots(
        &self,
        grace_days: i32,
        batch_size: i32
    ) -> Result<Vec<Annotation>, sqlx::Error> {
        sqlx::query_as::<_, Annotation>(
            r#"
            SELECT
                id, project_id, user_id, study_uid, series_uid, instance_uid,
                tool_name, tool_version, data, is_shared,
                snapshot_image_key, snapshot_status, snapshot_uploaded_at,
                created_at, updated_at,
                0 as version,
                viewer_software, description,
                NULL::jsonb as measurement_values,
                label
            FROM annotation_annotation
            WHERE snapshot_status = $1
              AND snapshot_image_key IS NOT NULL
              AND updated_at < NOW() - INTERVAL '1 day' * $2
            ORDER BY updated_at ASC
            LIMIT $3
            "#
        )
        .bind(SnapshotUploadStatus::Failed)
        .bind(grace_days)
        .bind(batch_size)
        .fetch_all(self.pool.as_ref())
        .await
    }

    async fn update_snapshot_status(
        &self,
        annotation_id: i32,
        status: SnapshotUploadStatus
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE annotation_annotation
            SET snapshot_status = $1,
                updated_at = NOW()
            WHERE id = $2
            "#
        )
        .bind(status)
        .bind(annotation_id)
        .execute(self.pool.as_ref())
        .await?;

        Ok(())
    }

    async fn clear_snapshot_image_key(
        &self,
        annotation_id: i32
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE annotation_annotation
            SET snapshot_image_key = NULL,
                updated_at = NOW()
            WHERE id = $1
            "#
        )
        .bind(annotation_id)
        .execute(self.pool.as_ref())
        .await?;

        Ok(())
    }
}