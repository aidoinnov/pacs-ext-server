use crate::domain::entities::project_data::{
    NewProjectData, ProjectData, ProjectDataInstance, ProjectDataSeries, ProjectDataStudy,
    UpdateProjectData,
};
use crate::domain::repositories::ProjectDataRepository;
use sqlx::PgPool;

pub struct ProjectDataRepositoryImpl {
    pool: PgPool,
}

impl ProjectDataRepositoryImpl {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl ProjectDataRepository for ProjectDataRepositoryImpl {
    async fn create(&self, new_data: &NewProjectData) -> Result<ProjectData, sqlx::Error> {
        let result = sqlx::query_as::<_, ProjectData>(
            "INSERT INTO project_data (project_id, study_uid, study_description, patient_id, patient_name, study_date, modality)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             RETURNING id, project_id, study_uid, study_description, patient_id, patient_name, study_date, modality, created_at"
        )
        .bind(new_data.project_id)
        .bind(&new_data.study_uid)
        .bind(&new_data.study_description)
        .bind(&new_data.patient_id)
        .bind(&new_data.patient_name)
        .bind(new_data.study_date)
        .bind(&new_data.modality)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    async fn find_by_id(&self, id: i32) -> Result<Option<ProjectData>, sqlx::Error> {
        let result = sqlx::query_as::<_, ProjectData>(
            "SELECT
                pd.id,
                pd.project_id,
                pds.study_uid,
                pds.study_description,
                pds.patient_id,
                pds.patient_name,
                pds.study_date,
                NULL::text as modality,
                pd.created_at
             FROM project_data pd
             INNER JOIN project_data_study pds ON pd.study_id = pds.id
             WHERE pd.id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    async fn find_by_project_id(
        &self,
        project_id: i32,
        page: i32,
        page_size: i32,
    ) -> Result<Vec<ProjectData>, sqlx::Error> {
        let offset = (page - 1) * page_size;

        // JOIN with project_data_study to get study information
        let results = sqlx::query_as::<_, ProjectData>(
            "SELECT
                pd.id,
                pd.project_id,
                pds.study_uid,
                pds.study_description,
                pds.patient_id,
                pds.patient_name,
                pds.study_date,
                NULL::text as modality,
                pd.created_at
             FROM project_data pd
             INNER JOIN project_data_study pds ON pd.study_id = pds.id
             WHERE pd.project_id = $1 AND pd.resource_level = 'STUDY'
             ORDER BY pd.id ASC
             LIMIT $2 OFFSET $3"
        )
        .bind(project_id)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }

    async fn count_by_project_id(&self, project_id: i32) -> Result<i64, sqlx::Error> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*)
             FROM project_data pd
             INNER JOIN project_data_study pds ON pd.study_id = pds.id
             WHERE pd.project_id = $1 AND pd.resource_level = 'STUDY'"
        )
        .bind(project_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }

    async fn find_by_study_uid(
        &self,
        project_id: i32,
        study_uid: &str,
    ) -> Result<Option<ProjectData>, sqlx::Error> {
        let result = sqlx::query_as::<_, ProjectData>(
            "SELECT
                pd.id,
                pd.project_id,
                pds.study_uid,
                pds.study_description,
                pds.patient_id,
                pds.patient_name,
                pds.study_date,
                NULL::text as modality,
                pd.created_at
             FROM project_data pd
             INNER JOIN project_data_study pds ON pd.study_id = pds.id
             WHERE pd.project_id = $1 AND pds.study_uid = $2 AND pd.resource_level = 'STUDY'"
        )
        .bind(project_id)
        .bind(study_uid)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    async fn search_by_project_id(
        &self,
        project_id: i32,
        search_term: &str,
        page: i32,
        page_size: i32,
    ) -> Result<Vec<ProjectData>, sqlx::Error> {
        let offset = (page - 1) * page_size;
        let search_pattern = format!("%{}%", search_term);

        let results = sqlx::query_as::<_, ProjectData>(
            "SELECT
                pd.id,
                pd.project_id,
                pds.study_uid,
                pds.study_description,
                pds.patient_id,
                pds.patient_name,
                pds.study_date,
                NULL::text as modality,
                pd.created_at
             FROM project_data pd
             INNER JOIN project_data_study pds ON pd.study_id = pds.id
             WHERE pd.project_id = $1
             AND pd.resource_level = 'STUDY'
             AND (pds.study_uid ILIKE $2 OR pds.patient_id ILIKE $2 OR pds.patient_name ILIKE $2)
             ORDER BY pd.id ASC
             LIMIT $3 OFFSET $4"
        )
        .bind(project_id)
        .bind(&search_pattern)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }

    async fn count_search_results(
        &self,
        project_id: i32,
        search_term: &str,
    ) -> Result<i64, sqlx::Error> {
        let search_pattern = format!("%{}%", search_term);

        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*)
             FROM project_data pd
             INNER JOIN project_data_study pds ON pd.study_id = pds.id
             WHERE pd.project_id = $1
             AND pd.resource_level = 'STUDY'
             AND (pds.study_uid ILIKE $2 OR pds.patient_id ILIKE $2 OR pds.patient_name ILIKE $2)",
        )
        .bind(project_id)
        .bind(&search_pattern)
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }

    async fn update(
        &self,
        id: i32,
        update_data: &UpdateProjectData,
    ) -> Result<Option<ProjectData>, sqlx::Error> {
        let mut query = String::from("UPDATE project_data SET ");
        let mut params: Vec<Box<dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync>> = Vec::new();
        let mut param_count = 1;

        if let Some(description) = &update_data.study_description {
            query.push_str(&format!("study_description = ${}, ", param_count));
            params.push(Box::new(description.clone()));
            param_count += 1;
        }

        if let Some(patient_id) = &update_data.patient_id {
            query.push_str(&format!("patient_id = ${}, ", param_count));
            params.push(Box::new(patient_id.clone()));
            param_count += 1;
        }

        if let Some(patient_name) = &update_data.patient_name {
            query.push_str(&format!("patient_name = ${}, ", param_count));
            params.push(Box::new(patient_name.clone()));
            param_count += 1;
        }

        if let Some(study_date) = &update_data.study_date {
            query.push_str(&format!("study_date = ${}, ", param_count));
            params.push(Box::new(study_date.clone()));
            param_count += 1;
        }

        if let Some(modality) = &update_data.modality {
            query.push_str(&format!("modality = ${}, ", param_count));
            params.push(Box::new(modality.clone()));
            param_count += 1;
        }

        if param_count == 1 {
            // No fields to update
            return self.find_by_id(id).await;
        }

        // Remove trailing comma and space
        query.pop();
        query.pop();

        query.push_str(&format!(" WHERE id = ${} RETURNING id, project_id, study_uid, study_description, patient_id, patient_name, study_date, modality, created_at", param_count));
        params.push(Box::new(id));

        // Execute the query
        let result = sqlx::query_as::<_, ProjectData>(&query)
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(result)
    }

    async fn delete(&self, id: i32) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM project_data WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    fn pool(&self) -> &PgPool {
        &self.pool
    }

    // ========== 새로운 계층 구조 메서드 구현 ==========

    async fn find_study_by_id(&self, id: i32) -> Result<Option<ProjectDataStudy>, sqlx::Error> {
        let result = sqlx::query_as::<_, ProjectDataStudy>(
            "SELECT id, study_uid, study_description, patient_id, patient_name, patient_birth_date, study_date, created_at, updated_at
             FROM project_data_study WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    async fn find_study_by_uid(
        &self,
        project_id: i32,
        study_uid: &str,
    ) -> Result<Option<ProjectDataStudy>, sqlx::Error> {
        // JOIN with project_data to filter by project_id
        let result = sqlx::query_as::<_, ProjectDataStudy>(
            "SELECT pds.id, pds.study_uid, pds.study_description, pds.patient_id, pds.patient_name,
                    pds.patient_birth_date, pds.study_date, pds.created_at, pds.updated_at
             FROM project_data_study pds
             INNER JOIN project_data pd ON pd.study_id = pds.id
             WHERE pd.project_id = $1 AND pds.study_uid = $2
             LIMIT 1"
        )
        .bind(project_id)
        .bind(study_uid)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    async fn find_studies_by_project_id(
        &self,
        project_id: i32,
        page: i32,
        page_size: i32,
    ) -> Result<Vec<ProjectDataStudy>, sqlx::Error> {
        let offset = (page - 1) * page_size;

        // UNION of directly assigned studies and rule-based matched studies
        let results = sqlx::query_as::<_, ProjectDataStudy>(
            "WITH directly_assigned AS (
                -- 직접 할당된 Study
                SELECT DISTINCT pds.id, pds.study_uid, pds.study_description, pds.patient_id, pds.patient_name,
                       pds.patient_birth_date, pds.study_date, pds.created_at, pds.updated_at
                FROM project_data_study pds
                INNER JOIN project_data pd ON pd.study_id = pds.id
                WHERE pd.project_id = $1 AND pd.resource_level = 'STUDY'
            ),
            rule_based AS (
                -- 규칙 기반으로 매칭되는 Study
                SELECT DISTINCT pds.id, pds.study_uid, pds.study_description, pds.patient_id, pds.patient_name,
                       pds.patient_birth_date, pds.study_date, pds.created_at, pds.updated_at
                FROM project_data_study pds
                WHERE EXISTS (
                    SELECT 1
                    FROM security_project_dicom_condition spdc
                    INNER JOIN security_access_condition ac ON spdc.access_condition_id = ac.id
                    WHERE spdc.project_id = $1
                      AND ac.resource_level = 'STUDY'
                      AND ac.condition_type = 'ALLOW'
                      AND (
                          -- Modality 조건
                          (ac.dicom_tag IN ('00080060', 'Modality') AND ac.modality IS NOT NULL
                           AND pds.modality = ac.modality)
                          OR
                          -- PatientID 조건
                          (ac.dicom_tag IN ('00100020', 'PatientID') AND ac.patient_id IS NOT NULL
                           AND pds.patient_id = ac.patient_id)
                          OR
                          -- StudyDate 범위 조건
                          (ac.dicom_tag IN ('00080020', 'StudyDate')
                           AND ac.date_range_start IS NOT NULL
                           AND ac.date_range_end IS NOT NULL
                           AND pds.study_date BETWEEN ac.date_range_start AND ac.date_range_end)
                          OR
                          -- 조건이 없으면 모든 Study 허용 (프로젝트 전체 접근)
                          (ac.dicom_tag IS NULL AND ac.modality IS NULL AND ac.patient_id IS NULL)
                      )
                )
            )
            SELECT * FROM (
                SELECT * FROM directly_assigned
                UNION
                SELECT * FROM rule_based
            ) AS combined
            ORDER BY study_date DESC NULLS LAST, created_at DESC
            LIMIT $2 OFFSET $3"
        )
        .bind(project_id)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }

    async fn count_studies_by_project_id(&self, project_id: i32) -> Result<i64, sqlx::Error> {
        // COUNT with UNION of directly assigned and rule-based matched studies
        let count = sqlx::query_scalar::<_, i64>(
            "WITH directly_assigned AS (
                SELECT DISTINCT pds.id
                FROM project_data_study pds
                INNER JOIN project_data pd ON pd.study_id = pds.id
                WHERE pd.project_id = $1 AND pd.resource_level = 'STUDY'
            ),
            rule_based AS (
                SELECT DISTINCT pds.id
                FROM project_data_study pds
                WHERE EXISTS (
                    SELECT 1
                    FROM security_project_dicom_condition spdc
                    INNER JOIN security_access_condition ac ON spdc.access_condition_id = ac.id
                    WHERE spdc.project_id = $1
                      AND ac.resource_level = 'STUDY'
                      AND ac.condition_type = 'ALLOW'
                      AND (
                          (ac.dicom_tag IN ('00080060', 'Modality') AND ac.modality IS NOT NULL
                           AND pds.modality = ac.modality)
                          OR
                          (ac.dicom_tag IN ('00100020', 'PatientID') AND ac.patient_id IS NOT NULL
                           AND pds.patient_id = ac.patient_id)
                          OR
                          (ac.dicom_tag IN ('00080020', 'StudyDate')
                           AND ac.date_range_start IS NOT NULL
                           AND ac.date_range_end IS NOT NULL
                           AND pds.study_date BETWEEN ac.date_range_start AND ac.date_range_end)
                          OR
                          (ac.dicom_tag IS NULL AND ac.modality IS NULL AND ac.patient_id IS NULL)
                      )
                )
            )
            SELECT COUNT(*) FROM (
                SELECT id FROM directly_assigned
                UNION
                SELECT id FROM rule_based
            ) AS combined",
        )
        .bind(project_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }

    async fn find_series_by_id(&self, id: i32) -> Result<Option<ProjectDataSeries>, sqlx::Error> {
        let result = sqlx::query_as::<_, ProjectDataSeries>(
            "SELECT id, study_id, series_uid, series_description, modality, series_number, created_at
             FROM project_data_series WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    async fn find_series_by_study_id(
        &self,
        study_id: i32,
    ) -> Result<Vec<ProjectDataSeries>, sqlx::Error> {
        let results = sqlx::query_as::<_, ProjectDataSeries>(
            "SELECT id, study_id, series_uid, series_description, modality, series_number, created_at
             FROM project_data_series 
             WHERE study_id = $1
             ORDER BY series_number ASC NULLS LAST, created_at ASC"
        )
        .bind(study_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }

    async fn count_series_by_study_id(&self, study_id: i32) -> Result<i64, sqlx::Error> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM project_data_series WHERE study_id = $1",
        )
        .bind(study_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }

    async fn find_instance_by_id(
        &self,
        id: i32,
    ) -> Result<Option<ProjectDataInstance>, sqlx::Error> {
        let result = sqlx::query_as::<_, ProjectDataInstance>(
            "SELECT id, series_id, instance_uid, sop_class_uid, instance_number, created_at
             FROM project_data_instance
             WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    async fn find_instances_by_series_id(
        &self,
        series_id: i32,
    ) -> Result<Vec<ProjectDataInstance>, sqlx::Error> {
        let results = sqlx::query_as::<_, ProjectDataInstance>(
            "SELECT id, series_id, instance_uid, sop_class_uid, instance_number, created_at
             FROM project_data_instance
             WHERE series_id = $1
             ORDER BY instance_number ASC NULLS LAST, created_at ASC",
        )
        .bind(series_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }

    async fn count_instances_by_series_id(&self, series_id: i32) -> Result<i64, sqlx::Error> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM project_data_instance WHERE series_id = $1",
        )
        .bind(series_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }
}
