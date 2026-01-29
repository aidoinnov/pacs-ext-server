use crate::domain::entities::{CreateSubject, Subject, SubjectDetail, UpdateSubject};
use crate::domain::repositories::SubjectRepository;
use async_trait::async_trait;
use sqlx::PgPool;

#[derive(Clone)]
pub struct SubjectRepositoryImpl {
    pool: PgPool,
}

impl SubjectRepositoryImpl {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SubjectRepository for SubjectRepositoryImpl {
    async fn find_by_id(&self, id: i32) -> Result<Option<Subject>, sqlx::Error> {
        sqlx::query_as::<_, Subject>(
            "SELECT id, project_id, subject_code, external_subject_key, patient_id, patient_name, patient_birth_date, created_at, updated_at
             FROM project_subject
             WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    async fn find_by_code(
        &self,
        project_id: i32,
        subject_code: &str,
    ) -> Result<Option<Subject>, sqlx::Error> {
        sqlx::query_as::<_, Subject>(
            "SELECT id, project_id, subject_code, external_subject_key, patient_id, patient_name, patient_birth_date, created_at, updated_at
             FROM project_subject
             WHERE project_id = $1 AND subject_code = $2",
        )
        .bind(project_id)
        .bind(subject_code)
        .fetch_optional(&self.pool)
        .await
    }

    async fn find_by_patient_id(
        &self,
        project_id: i32,
        patient_id: &str,
    ) -> Result<Option<Subject>, sqlx::Error> {
        sqlx::query_as::<_, Subject>(
            "SELECT id, project_id, subject_code, external_subject_key, patient_id, patient_name, patient_birth_date, created_at, updated_at
             FROM project_subject
             WHERE project_id = $1 AND patient_id = $2",
        )
        .bind(project_id)
        .bind(patient_id)
        .fetch_optional(&self.pool)
        .await
    }

    async fn find_by_project(&self, project_id: i32) -> Result<Vec<Subject>, sqlx::Error> {
        sqlx::query_as::<_, Subject>(
            "SELECT id, project_id, subject_code, external_subject_key, patient_id, patient_name, patient_birth_date, created_at, updated_at
             FROM project_subject
             WHERE project_id = $1
             ORDER BY created_at DESC",
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await
    }

    async fn find_detail_by_id(&self, id: i32) -> Result<Option<SubjectDetail>, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct SubjectDetailRow {
            id: i32,
            project_id: i32,
            subject_code: String,
            external_subject_key: Option<String>,
            patient_id: Option<String>,
            patient_name: Option<String>,
            patient_birth_date: Option<chrono::NaiveDate>,
            created_at: chrono::DateTime<chrono::Utc>,
            updated_at: chrono::DateTime<chrono::Utc>,
            timepoint_count: Option<i64>,
            study_count: Option<i64>,
        }

        let row = sqlx::query_as::<_, SubjectDetailRow>(
            r#"
            SELECT
                s.id, s.project_id, s.subject_code, s.external_subject_key,
                s.patient_id, s.patient_name, s.patient_birth_date,
                s.created_at, s.updated_at,
                COUNT(DISTINCT tp.id) as timepoint_count,
                COUNT(DISTINCT tps.study_id) as study_count
            FROM project_subject s
            LEFT JOIN subject_timepoint tp ON s.id = tp.subject_id
            LEFT JOIN subject_timepoint_study_map tps ON tp.id = tps.timepoint_id
            WHERE s.id = $1
            GROUP BY s.id
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| SubjectDetail {
            subject: Subject {
                id: r.id,
                project_id: r.project_id,
                subject_code: r.subject_code,
                external_subject_key: r.external_subject_key,
                patient_id: r.patient_id,
                patient_name: r.patient_name,
                patient_birth_date: r.patient_birth_date,
                created_at: r.created_at,
                updated_at: r.updated_at,
            },
            timepoint_count: r.timepoint_count.unwrap_or(0),
            study_count: r.study_count.unwrap_or(0),
        }))
    }

    async fn create(&self, new_subject: CreateSubject) -> Result<Subject, sqlx::Error> {
        sqlx::query_as::<_, Subject>(
            "INSERT INTO project_subject (project_id, subject_code, patient_id, patient_name, patient_birth_date)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING id, project_id, subject_code, external_subject_key, patient_id, patient_name, patient_birth_date, created_at, updated_at",
        )
        .bind(new_subject.project_id)
        .bind(new_subject.subject_code)
        .bind(new_subject.patient_id)
        .bind(new_subject.patient_name)
        .bind(new_subject.patient_birth_date)
        .fetch_one(&self.pool)
        .await
    }

    async fn update(
        &self,
        id: i32,
        update_subject: UpdateSubject,
    ) -> Result<Option<Subject>, sqlx::Error> {
        sqlx::query_as::<_, Subject>(
            "UPDATE project_subject
             SET subject_code = COALESCE($2, subject_code),
                 external_subject_key = COALESCE($3, external_subject_key),
                 patient_id = COALESCE($4, patient_id),
                 patient_name = COALESCE($5, patient_name),
                 patient_birth_date = COALESCE($6, patient_birth_date),
                 updated_at = NOW()
             WHERE id = $1
             RETURNING id, project_id, subject_code, external_subject_key, patient_id, patient_name, patient_birth_date, created_at, updated_at",
        )
        .bind(id)
        .bind(update_subject.subject_code)
        .bind(update_subject.external_subject_key)
        .bind(update_subject.patient_id)
        .bind(update_subject.patient_name)
        .bind(update_subject.patient_birth_date)
        .fetch_optional(&self.pool)
        .await
    }

    async fn delete(&self, id: i32) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM project_subject WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    async fn find_by_external_key(
        &self,
        external_key: &str,
    ) -> Result<Option<Subject>, sqlx::Error> {
        sqlx::query_as::<_, Subject>(
            "SELECT id, project_id, subject_code, external_subject_key, patient_id, patient_name, patient_birth_date, created_at, updated_at
             FROM project_subject
             WHERE external_subject_key = $1",
        )
        .bind(external_key)
        .fetch_optional(&self.pool)
        .await
    }

    fn pool(&self) -> &PgPool {
        &self.pool
    }

    async fn get_subjects_updated_at(&self, project_id: i32) -> Result<chrono::NaiveDateTime, sqlx::Error> {
        // 프로젝트의 Subject 중 가장 최근 updated_at 조회
        // Subject가 없으면 고정된 기본값 사용 (1970-01-01)
        let updated_at = sqlx::query_scalar::<_, chrono::DateTime<chrono::Utc>>(
            "SELECT COALESCE(MAX(updated_at), '1970-01-01'::timestamptz) FROM project_subject WHERE project_id = $1"
        )
        .bind(project_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(updated_at.naive_utc())
    }
}
