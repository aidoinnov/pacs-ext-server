use crate::domain::entities::{StudyInfo, TimePointStudy};
use crate::domain::repositories::TimePointStudyRepository;
use async_trait::async_trait;
use sqlx::PgPool;

#[derive(Clone)]
pub struct TimePointStudyRepositoryImpl {
    pool: PgPool,
}

impl TimePointStudyRepositoryImpl {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TimePointStudyRepository for TimePointStudyRepositoryImpl {
    async fn find_studies_by_timepoint(
        &self,
        timepoint_id: i32,
    ) -> Result<Vec<StudyInfo>, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct StudyInfoRow {
            study_id: i32,
            study_uid: String,
            study_description: Option<String>,
            study_date: Option<String>,
            patient_id: Option<String>,
            modality: Option<String>,
            assigned_at: chrono::DateTime<chrono::Utc>,
            assigned_by: i32,
        }

        let rows = sqlx::query_as::<_, StudyInfoRow>(
            r#"
            SELECT
                pds.id as study_id,
                pds.study_uid,
                pds.study_description,
                pds.study_date::text as study_date,
                pds.patient_id,
                NULL::text as modality,
                tps.assigned_at,
                tps.assigned_by
            FROM subject_timepoint_study_map tps
            JOIN project_data_study pds ON tps.study_id = pds.id
            WHERE tps.timepoint_id = $1
            ORDER BY pds.study_date DESC
            "#
        )
        .bind(timepoint_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| StudyInfo {
                study_id: r.study_id,
                study_uid: r.study_uid,
                study_description: r.study_description,
                study_date: r.study_date,
                patient_id: r.patient_id,
                modality: r.modality,
                assigned_at: Some(r.assigned_at),
                assigned_by: Some(r.assigned_by),
            })
            .collect())
    }

    async fn find_unassigned_studies_by_subject(
        &self,
        subject_id: i32,
    ) -> Result<Vec<StudyInfo>, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct UnassignedStudyRow {
            study_id: i32,
            study_uid: String,
            study_description: Option<String>,
            study_date: Option<String>,
            patient_id: Option<String>,
            modality: Option<String>,
        }

        let rows = sqlx::query_as::<_, UnassignedStudyRow>(
            r#"
            SELECT
                pds.id as study_id,
                pds.study_uid,
                pds.study_description,
                pds.study_date::text as study_date,
                pds.patient_id,
                NULL::text as modality
            FROM project_data_study pds
            JOIN project_subject s ON pds.patient_id = s.patient_id AND s.id = $1
            WHERE NOT EXISTS (
                SELECT 1 FROM subject_timepoint_study_map tps
                JOIN subject_timepoint tp ON tps.timepoint_id = tp.id
                WHERE tps.study_id = pds.id AND tp.subject_id = $1
            )
            ORDER BY pds.study_date DESC
            "#
        )
        .bind(subject_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| StudyInfo {
                study_id: r.study_id,
                study_uid: r.study_uid,
                study_description: r.study_description,
                study_date: r.study_date,
                patient_id: r.patient_id,
                modality: r.modality,
                assigned_at: None,
                assigned_by: None,
            })
            .collect())
    }

    async fn find_timepoint_by_study(
        &self,
        study_id: i32,
    ) -> Result<Option<i32>, sqlx::Error> {
        sqlx::query_scalar::<_, i32>(
            "SELECT timepoint_id FROM subject_timepoint_study_map WHERE study_id = $1"
        )
        .bind(study_id)
        .fetch_optional(&self.pool)
        .await
    }

    async fn assign_studies(
        &self,
        timepoint_id: i32,
        study_ids: &[i32],
        user_id: i32,
    ) -> Result<i32, sqlx::Error> {
        // Delete existing assignments for these studies (MOVE semantics)
        sqlx::query("DELETE FROM subject_timepoint_study_map WHERE study_id = ANY($1)")
            .bind(study_ids)
            .execute(&self.pool)
            .await?;

        // Insert new assignments
        let mut count = 0;
        for study_id in study_ids {
            sqlx::query(
                "INSERT INTO subject_timepoint_study_map (timepoint_id, study_id, assigned_by)
                 VALUES ($1, $2, $3)
                 ON CONFLICT (study_id) DO UPDATE SET timepoint_id = $1, assigned_by = $3, assigned_at = NOW()"
            )
            .bind(timepoint_id)
            .bind(study_id)
            .bind(user_id)
            .execute(&self.pool)
            .await?;
            count += 1;
        }

        Ok(count)
    }

    async fn unassign_studies(&self, study_ids: &[i32]) -> Result<i32, sqlx::Error> {
        let result = sqlx::query("DELETE FROM subject_timepoint_study_map WHERE study_id = ANY($1)")
            .bind(study_ids)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() as i32)
    }

    async fn unassign_all_studies_from_timepoint(
        &self,
        timepoint_id: i32,
    ) -> Result<i32, sqlx::Error> {
        let result = sqlx::query("DELETE FROM subject_timepoint_study_map WHERE timepoint_id = $1")
            .bind(timepoint_id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() as i32)
    }

    async fn get_subject_board_data(
        &self,
        subject_id: i32,
    ) -> Result<serde_json::Value, sqlx::Error> {
        // This is a complex query that returns JSON
        // For now, return a placeholder
        // TODO: Implement proper board data aggregation
        Ok(serde_json::json!({
            "subject_id": subject_id,
            "unassigned": [],
            "timepoints": []
        }))
    }

    async fn count_studies_by_timepoint(
        &self,
        timepoint_id: i32,
    ) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM subject_timepoint_study_map WHERE timepoint_id = $1"
        )
        .bind(timepoint_id)
        .fetch_one(&self.pool)
        .await
    }

    async fn count_unassigned_studies_by_subject(
        &self,
        subject_id: i32,
    ) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)
            FROM project_data_study pds
            JOIN project_subject s ON pds.patient_id = s.patient_id AND s.id = $1
            WHERE NOT EXISTS (
                SELECT 1 FROM subject_timepoint_study_map tps
                JOIN subject_timepoint tp ON tps.timepoint_id = tp.id
                WHERE tps.study_id = pds.id AND tp.subject_id = $1
            )
            "#
        )
        .bind(subject_id)
        .fetch_one(&self.pool)
        .await
    }

    fn pool(&self) -> &PgPool {
        &self.pool
    }
}
