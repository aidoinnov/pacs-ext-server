use crate::domain::entities::SeriesUserNote;
use crate::domain::repositories::SeriesUserNoteRepository;
use async_trait::async_trait;
use sqlx::PgPool;

#[derive(Clone)]
pub struct SeriesUserNoteRepositoryImpl {
    pool: PgPool,
}

impl SeriesUserNoteRepositoryImpl {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SeriesUserNoteRepository for SeriesUserNoteRepositoryImpl {
    async fn create_or_update(
        &self,
        series_id: i32,
        user_id: i32,
        project_id: Option<i32>,
        note: String,
    ) -> Result<SeriesUserNote, sqlx::Error> {
        sqlx::query_as::<_, SeriesUserNote>(
            r#"
            INSERT INTO series_user_note (series_id, user_id, project_id, note)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (series_id, user_id, project_id)
            DO UPDATE SET
                note = EXCLUDED.note,
                updated_at = CURRENT_TIMESTAMP
            RETURNING id, series_id, user_id, project_id, note, created_at, updated_at
            "#,
        )
        .bind(series_id)
        .bind(user_id)
        .bind(project_id)
        .bind(note)
        .fetch_one(&self.pool)
        .await
    }

    async fn find_by_series_user_project(
        &self,
        series_id: i32,
        user_id: i32,
        project_id: Option<i32>,
    ) -> Result<Option<SeriesUserNote>, sqlx::Error> {
        sqlx::query_as::<_, SeriesUserNote>(
            r#"
            SELECT id, series_id, user_id, project_id, note, created_at, updated_at
            FROM series_user_note
            WHERE series_id = $1 AND user_id = $2 AND (project_id = $3 OR (project_id IS NULL AND $3 IS NULL))
            "#,
        )
        .bind(series_id)
        .bind(user_id)
        .bind(project_id)
        .fetch_optional(&self.pool)
        .await
    }

    async fn find_by_series(
        &self,
        series_id: i32,
        project_id: Option<i32>,
    ) -> Result<Vec<SeriesUserNote>, sqlx::Error> {
        let query = if let Some(pid) = project_id {
            sqlx::query_as::<_, SeriesUserNote>(
                r#"
                SELECT id, series_id, user_id, project_id, note, created_at, updated_at
                FROM series_user_note
                WHERE series_id = $1 AND project_id = $2
                ORDER BY created_at DESC
                "#,
            )
            .bind(series_id)
            .bind(pid)
        } else {
            sqlx::query_as::<_, SeriesUserNote>(
                r#"
                SELECT id, series_id, user_id, project_id, note, created_at, updated_at
                FROM series_user_note
                WHERE series_id = $1
                ORDER BY created_at DESC
                "#,
            )
            .bind(series_id)
        };

        query.fetch_all(&self.pool).await
    }

    async fn delete(
        &self,
        series_id: i32,
        user_id: i32,
        project_id: Option<i32>,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM series_user_note
            WHERE series_id = $1 AND user_id = $2 AND (project_id = $3 OR (project_id IS NULL AND $3 IS NULL))
            "#,
        )
        .bind(series_id)
        .bind(user_id)
        .bind(project_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    fn pool(&self) -> &PgPool {
        &self.pool
    }
}

