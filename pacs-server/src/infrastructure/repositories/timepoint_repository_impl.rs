use crate::domain::entities::{CreateTimePoint, TimePoint, UpdateTimePoint, VisitType};
use crate::domain::repositories::TimePointRepository;
use async_trait::async_trait;
use sqlx::PgPool;

#[derive(Clone)]
pub struct TimePointRepositoryImpl {
    pool: PgPool,
}

impl TimePointRepositoryImpl {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TimePointRepository for TimePointRepositoryImpl {
    async fn find_by_id(&self, id: i32) -> Result<Option<TimePoint>, sqlx::Error> {
        sqlx::query_as::<_, TimePoint>(
            "SELECT id, project_id, subject_id, name, visit_type, visit_no, order_index, external_key, created_at, updated_at
             FROM subject_timepoint
             WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    async fn find_by_subject(&self, subject_id: i32) -> Result<Vec<TimePoint>, sqlx::Error> {
        sqlx::query_as::<_, TimePoint>(
            "SELECT id, project_id, subject_id, name, visit_type, visit_no, order_index, external_key, created_at, updated_at
             FROM subject_timepoint
             WHERE subject_id = $1
             ORDER BY order_index ASC",
        )
        .bind(subject_id)
        .fetch_all(&self.pool)
        .await
    }

    async fn find_baseline_by_subject(
        &self,
        subject_id: i32,
    ) -> Result<Option<TimePoint>, sqlx::Error> {
        sqlx::query_as::<_, TimePoint>(
            "SELECT id, project_id, subject_id, name, visit_type, visit_no, order_index, external_key, created_at, updated_at
             FROM subject_timepoint
             WHERE subject_id = $1 AND visit_type = 'Baseline'",
        )
        .bind(subject_id)
        .fetch_optional(&self.pool)
        .await
    }

    async fn find_by_name(
        &self,
        subject_id: i32,
        name: &str,
    ) -> Result<Option<TimePoint>, sqlx::Error> {
        sqlx::query_as::<_, TimePoint>(
            "SELECT id, project_id, subject_id, name, visit_type, visit_no, order_index, external_key, created_at, updated_at
             FROM subject_timepoint
             WHERE subject_id = $1 AND name = $2",
        )
        .bind(subject_id)
        .bind(name)
        .fetch_optional(&self.pool)
        .await
    }

    async fn create(&self, new_timepoint: CreateTimePoint) -> Result<TimePoint, sqlx::Error> {
        // Get project_id from subject
        let project_id = sqlx::query_scalar::<_, i32>(
            "SELECT project_id FROM project_subject WHERE id = $1"
        )
        .bind(new_timepoint.subject_id)
        .fetch_one(&self.pool)
        .await?;

        sqlx::query_as::<_, TimePoint>(
            "INSERT INTO subject_timepoint (project_id, subject_id, name, visit_type, visit_no, order_index)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, project_id, subject_id, name, visit_type, visit_no, order_index, external_key, created_at, updated_at",
        )
        .bind(project_id)
        .bind(new_timepoint.subject_id)
        .bind(new_timepoint.name)
        .bind(new_timepoint.visit_type)
        .bind(new_timepoint.visit_no)
        .bind(new_timepoint.order_index)
        .fetch_one(&self.pool)
        .await
    }

    async fn update(
        &self,
        id: i32,
        update_timepoint: UpdateTimePoint,
    ) -> Result<Option<TimePoint>, sqlx::Error> {
        sqlx::query_as::<_, TimePoint>(
            "UPDATE subject_timepoint
             SET name = COALESCE($2, name),
                 visit_type = COALESCE($3, visit_type),
                 order_index = COALESCE($4, order_index),
                 updated_at = NOW()
             WHERE id = $1
             RETURNING id, project_id, subject_id, name, visit_type, visit_no, order_index, external_key, created_at, updated_at",
        )
        .bind(id)
        .bind(update_timepoint.name)
        .bind(update_timepoint.visit_type)
        .bind(update_timepoint.order_index)
        .fetch_optional(&self.pool)
        .await
    }

    async fn delete(&self, id: i32) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM subject_timepoint WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    async fn find_by_external_key(
        &self,
        external_key: &str,
    ) -> Result<Option<TimePoint>, sqlx::Error> {
        sqlx::query_as::<_, TimePoint>(
            "SELECT id, project_id, subject_id, name, visit_type, visit_no, order_index, external_key, created_at, updated_at
             FROM subject_timepoint
             WHERE external_key = $1",
        )
        .bind(external_key)
        .fetch_optional(&self.pool)
        .await
    }

    async fn find_by_visit_type(
        &self,
        subject_id: i32,
        visit_type: VisitType,
    ) -> Result<Vec<TimePoint>, sqlx::Error> {
        sqlx::query_as::<_, TimePoint>(
            "SELECT id, project_id, subject_id, name, visit_type, visit_no, order_index, external_key, created_at, updated_at
             FROM subject_timepoint
             WHERE subject_id = $1 AND visit_type = $2
             ORDER BY order_index ASC",
        )
        .bind(subject_id)
        .bind(visit_type)
        .fetch_all(&self.pool)
        .await
    }

    fn pool(&self) -> &PgPool {
        &self.pool
    }
}

