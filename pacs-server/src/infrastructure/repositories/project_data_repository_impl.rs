use crate::domain::entities::project_data::{ProjectData, ProjectDataStudy, ProjectDataSeries, NewProjectData, UpdateProjectData};
use crate::domain::repositories::ProjectDataRepository;
use sqlx::{PgPool, FromRow};

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
            "SELECT id, project_id, study_uid, study_description, patient_id, patient_name, study_date, modality, created_at
             FROM project_data WHERE id = $1"
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
        page_size: i32
    ) -> Result<Vec<ProjectData>, sqlx::Error> {
        let offset = (page - 1) * page_size;
        
        let results = sqlx::query_as::<_, ProjectData>(
            "SELECT id, project_id, study_uid, study_description, patient_id, patient_name, study_date, modality, created_at
             FROM project_data 
             WHERE project_id = $1
             ORDER BY created_at DESC
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
            "SELECT COUNT(*) FROM project_data WHERE project_id = $1"
        )
        .bind(project_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }

    async fn find_by_study_uid(
        &self, 
        project_id: i32, 
        study_uid: &str
    ) -> Result<Option<ProjectData>, sqlx::Error> {
        let result = sqlx::query_as::<_, ProjectData>(
            "SELECT id, project_id, study_uid, study_description, patient_id, patient_name, study_date, modality, created_at
             FROM project_data 
             WHERE project_id = $1 AND study_uid = $2"
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
        page_size: i32
    ) -> Result<Vec<ProjectData>, sqlx::Error> {
        let offset = (page - 1) * page_size;
        let search_pattern = format!("%{}%", search_term);
        
        let results = sqlx::query_as::<_, ProjectData>(
            "SELECT id, project_id, study_uid, study_description, patient_id, patient_name, study_date, modality, created_at
             FROM project_data 
             WHERE project_id = $1 
             AND (study_uid ILIKE $2 OR patient_id ILIKE $2 OR patient_name ILIKE $2)
             ORDER BY created_at DESC
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
        search_term: &str
    ) -> Result<i64, sqlx::Error> {
        let search_pattern = format!("%{}%", search_term);
        
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM project_data 
             WHERE project_id = $1 
             AND (study_uid ILIKE $2 OR patient_id ILIKE $2 OR patient_name ILIKE $2)"
        )
        .bind(project_id)
        .bind(&search_pattern)
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }

    async fn update(&self, id: i32, update_data: &UpdateProjectData) -> Result<Option<ProjectData>, sqlx::Error> {
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
            "SELECT id, project_id, study_uid, study_description, patient_id, patient_name, patient_birth_date, study_date, created_at, updated_at
             FROM project_data_study WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(result)
    }
    
    async fn find_study_by_uid(&self, project_id: i32, study_uid: &str) -> Result<Option<ProjectDataStudy>, sqlx::Error> {
        let result = sqlx::query_as::<_, ProjectDataStudy>(
            "SELECT id, project_id, study_uid, study_description, patient_id, patient_name, patient_birth_date, study_date, created_at, updated_at
             FROM project_data_study WHERE project_id = $1 AND study_uid = $2"
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
        page_size: i32
    ) -> Result<Vec<ProjectDataStudy>, sqlx::Error> {
        let offset = (page - 1) * page_size;
        
        let results = sqlx::query_as::<_, ProjectDataStudy>(
            "SELECT id, project_id, study_uid, study_description, patient_id, patient_name, patient_birth_date, study_date, created_at, updated_at
             FROM project_data_study 
             WHERE project_id = $1
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
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM project_data_study WHERE project_id = $1"
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
    
    async fn find_series_by_study_id(&self, study_id: i32) -> Result<Vec<ProjectDataSeries>, sqlx::Error> {
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
            "SELECT COUNT(*) FROM project_data_series WHERE study_id = $1"
        )
        .bind(study_id)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(count)
    }
}
