use crate::domain::reporting::entities::series_user_report::{
    NewSeriesUserReport, SeriesUserReport, UpdateSeriesUserReport,
};
use crate::domain::reporting::repositories::SeriesUserReportRepository;
use async_trait::async_trait;
use sqlx::{PgPool, Result};

#[derive(Clone)]
pub struct SeriesUserReportRepositoryImpl {
    pool: PgPool,
}

impl SeriesUserReportRepositoryImpl {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SeriesUserReportRepository for SeriesUserReportRepositoryImpl {
    async fn create_or_update(
        &self,
        series_id: i32,
        user_id: i32,
        project_id: Option<i32>,
        new_report: &NewSeriesUserReport,
    ) -> Result<SeriesUserReport> {
        sqlx::query_as::<_, SeriesUserReport>(
            r#"
            INSERT INTO series_user_report (
                series_id, user_id, project_id, status,
                dictate_file_path, dictate_file_size, dictate_mime_type,
                description, conclusion, bodypart
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (series_id, user_id, project_id)
            DO UPDATE SET
                status = EXCLUDED.status,
                dictate_file_path = EXCLUDED.dictate_file_path,
                dictate_file_size = EXCLUDED.dictate_file_size,
                dictate_mime_type = EXCLUDED.dictate_mime_type,
                description = EXCLUDED.description,
                conclusion = EXCLUDED.conclusion,
                bodypart = EXCLUDED.bodypart,
                updated_at = CURRENT_TIMESTAMP
            RETURNING id, series_id, user_id, project_id, status,
                     template_id, custom_template_id,
                     dictate_file_path, dictate_file_size, dictate_mime_type,
                     description, conclusion, bodypart, created_at, updated_at
            "#,
        )
        .bind(series_id)
        .bind(user_id)
        .bind(project_id)
        .bind(&new_report.status)
        .bind(&new_report.dictate_file_path)
        .bind(&new_report.dictate_file_size)
        .bind(&new_report.dictate_mime_type)
        .bind(&new_report.description)
        .bind(&new_report.conclusion)
        .bind(&new_report.bodypart)
        .fetch_one(&self.pool)
        .await
    }

    async fn find_by_id(&self, report_id: i32) -> Result<Option<SeriesUserReport>> {
        sqlx::query_as::<_, SeriesUserReport>(
            r#"
            SELECT id, series_id, user_id, project_id, status,
                   template_id, custom_template_id,
                   dictate_file_path, dictate_file_size, dictate_mime_type,
                   description, conclusion, bodypart, created_at, updated_at
            FROM series_user_report
            WHERE id = $1
            "#,
        )
        .bind(report_id)
        .fetch_optional(&self.pool)
        .await
    }

    async fn find_by_series_user_project(
        &self,
        series_id: i32,
        user_id: i32,
        project_id: Option<i32>,
    ) -> Result<Option<SeriesUserReport>> {
        sqlx::query_as::<_, SeriesUserReport>(
            r#"
            SELECT id, series_id, user_id, project_id, status,
                   template_id, custom_template_id,
                   dictate_file_path, dictate_file_size, dictate_mime_type,
                   description, conclusion, bodypart, created_at, updated_at
            FROM series_user_report
            WHERE series_id = $1 AND user_id = $2 
              AND (project_id = $3 OR (project_id IS NULL AND $3 IS NULL))
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
    ) -> Result<Vec<SeriesUserReport>> {
        let query = if let Some(pid) = project_id {
            sqlx::query_as::<_, SeriesUserReport>(
                r#"
                SELECT id, series_id, user_id, project_id, status,
                       template_id, custom_template_id,
                       dictate_file_path, dictate_file_size, dictate_mime_type,
                       description, conclusion, bodypart, created_at, updated_at
                FROM series_user_report
                WHERE series_id = $1 AND project_id = $2
                ORDER BY created_at DESC
                "#,
            )
            .bind(series_id)
            .bind(pid)
        } else {
            sqlx::query_as::<_, SeriesUserReport>(
                r#"
                SELECT id, series_id, user_id, project_id, status,
                       template_id, custom_template_id,
                       dictate_file_path, dictate_file_size, dictate_mime_type,
                       description, conclusion, bodypart, created_at, updated_at
                FROM series_user_report
                WHERE series_id = $1
                ORDER BY created_at DESC
                "#,
            )
            .bind(series_id)
        };

        query.fetch_all(&self.pool).await
    }

    async fn update(
        &self,
        series_id: i32,
        user_id: i32,
        project_id: Option<i32>,
        update: &UpdateSeriesUserReport,
    ) -> Result<SeriesUserReport> {
        // 동적 쿼리 구성
        let mut query = String::from(
            r#"
            UPDATE series_user_report
            SET updated_at = CURRENT_TIMESTAMP
            "#,
        );
        let mut params: Vec<String> = vec![];
        let mut param_count = 1;

        if update.status.is_some() {
            query.push_str(&format!(", status = ${}", param_count));
            params.push("status".to_string());
            param_count += 1;
        }
        if update.dictate_file_path.is_some() {
            query.push_str(&format!(", dictate_file_path = ${}", param_count));
            params.push("dictate_file_path".to_string());
            param_count += 1;
        }
        if update.dictate_file_size.is_some() {
            query.push_str(&format!(", dictate_file_size = ${}", param_count));
            params.push("dictate_file_size".to_string());
            param_count += 1;
        }
        if update.dictate_mime_type.is_some() {
            query.push_str(&format!(", dictate_mime_type = ${}", param_count));
            params.push("dictate_mime_type".to_string());
            param_count += 1;
        }
        if update.description.is_some() {
            query.push_str(&format!(", description = ${}", param_count));
            params.push("description".to_string());
            param_count += 1;
        }
        if update.conclusion.is_some() {
            query.push_str(&format!(", conclusion = ${}", param_count));
            params.push("conclusion".to_string());
            param_count += 1;
        }
        if update.bodypart.is_some() {
            query.push_str(&format!(", bodypart = ${}", param_count));
            params.push("bodypart".to_string());
            param_count += 1;
        }

        query.push_str(
            r#"
            WHERE series_id = $1 AND user_id = $2 
              AND (project_id = $3 OR (project_id IS NULL AND $3 IS NULL))
            RETURNING id, series_id, user_id, project_id, status,
                     dictate_file_path, dictate_file_size, dictate_mime_type,
                     description, conclusion, bodypart, created_at, updated_at
            "#,
        );

        // 간단한 구현: 모든 필드를 업데이트 (NULL 허용)
        sqlx::query_as::<_, SeriesUserReport>(
            r#"
            UPDATE series_user_report
            SET
                status = COALESCE($4, status),
                template_id = COALESCE($5, template_id),
                custom_template_id = COALESCE($6, custom_template_id),
                dictate_file_path = COALESCE($7, dictate_file_path),
                dictate_file_size = COALESCE($8, dictate_file_size),
                dictate_mime_type = COALESCE($9, dictate_mime_type),
                description = COALESCE($10, description),
                conclusion = COALESCE($11, conclusion),
                bodypart = COALESCE($12, bodypart),
                updated_at = CURRENT_TIMESTAMP
            WHERE series_id = $1 AND user_id = $2 
              AND (project_id = $3 OR (project_id IS NULL AND $3 IS NULL))
            RETURNING id, series_id, user_id, project_id, status,
                     template_id, custom_template_id,
                     dictate_file_path, dictate_file_size, dictate_mime_type,
                     description, conclusion, bodypart, created_at, updated_at
            "#,
        )
        .bind(series_id)
        .bind(user_id)
        .bind(project_id)
        .bind(&update.status)
        .bind(&update.template_id)
        .bind(&update.custom_template_id)
        .bind(&update.dictate_file_path)
        .bind(&update.dictate_file_size)
        .bind(&update.dictate_mime_type)
        .bind(&update.description)
        .bind(&update.conclusion)
        .bind(&update.bodypart)
        .fetch_one(&self.pool)
        .await
    }

    async fn update_report_template(
        &self,
        report_id: i32,
        template_id: Option<i32>,
        custom_template_id: Option<i32>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE series_user_report
            SET template_id = $2, custom_template_id = $3, updated_at = CURRENT_TIMESTAMP
            WHERE id = $1
            "#,
        )
        .bind(report_id)
        .bind(template_id)
        .bind(custom_template_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete(
        &self,
        series_id: i32,
        user_id: i32,
        project_id: Option<i32>,
    ) -> Result<bool> {
        let result = sqlx::query(
            r#"
            DELETE FROM series_user_report
            WHERE series_id = $1 AND user_id = $2 
              AND (project_id = $3 OR (project_id IS NULL AND $3 IS NULL))
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

