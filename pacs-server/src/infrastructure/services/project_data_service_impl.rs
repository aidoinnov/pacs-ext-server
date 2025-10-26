use crate::domain::entities::project_data::{ProjectData, ProjectDataAccess, ProjectDataStudy, ProjectDataSeries, NewProjectData, UpdateProjectData, NewProjectDataAccess, UpdateProjectDataAccess, DataAccessStatus};
use crate::domain::repositories::{ProjectDataRepository, ProjectDataAccessRepository};
use crate::domain::services::ProjectDataService;
use crate::domain::ServiceError;
use std::sync::Arc;

pub struct ProjectDataServiceImpl<P, A> 
where
    P: ProjectDataRepository,
    A: ProjectDataAccessRepository,
{
    project_data_repository: Arc<P>,
    project_data_access_repository: Arc<A>,
}

impl<P, A> ProjectDataServiceImpl<P, A>
where
    P: ProjectDataRepository,
    A: ProjectDataAccessRepository,
{
    pub fn new(project_data_repository: Arc<P>, project_data_access_repository: Arc<A>) -> Self {
        Self {
            project_data_repository,
            project_data_access_repository,
        }
    }
}

#[async_trait::async_trait]
impl<P, A> ProjectDataService for ProjectDataServiceImpl<P, A>
where
    P: ProjectDataRepository,
    A: ProjectDataAccessRepository,
{
    async fn create_project_data(&self, new_data: NewProjectData) -> Result<ProjectData, ServiceError> {
        // Check if study already exists in project
        if let Some(_) = self.project_data_repository
            .find_by_study_uid(new_data.project_id, &new_data.study_uid)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))? {
            return Err(ServiceError::AlreadyExists("Study already exists in project".to_string()));
        }

        let project_data = self.project_data_repository
            .create(&new_data)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        // Grant access to existing project users
        self.grant_access_to_existing_users(project_data.id, new_data.project_id).await?;

        Ok(project_data)
    }

    async fn get_project_data(&self, id: i32) -> Result<ProjectData, ServiceError> {
        self.project_data_repository
            .find_by_id(id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
            .ok_or_else(|| ServiceError::NotFound("Project data not found".to_string()))
    }

    async fn get_project_data_list(
        &self,
        project_id: i32,
        page: i32,
        page_size: i32
    ) -> Result<Vec<ProjectData>, ServiceError> {
        self.project_data_repository
            .find_by_project_id(project_id, page, page_size)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    async fn search_project_data(
        &self,
        project_id: i32,
        search_term: &str,
        page: i32,
        page_size: i32
    ) -> Result<Vec<ProjectData>, ServiceError> {
        self.project_data_repository
            .search_by_project_id(project_id, search_term, page, page_size)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    async fn update_project_data(&self, id: i32, update_data: UpdateProjectData) -> Result<ProjectData, ServiceError> {
        let project_data = self.project_data_repository
            .update(id, &update_data)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
            .ok_or_else(|| ServiceError::NotFound("Project data not found".to_string()))?;

        Ok(project_data)
    }

    async fn delete_project_data(&self, id: i32) -> Result<(), ServiceError> {
        let deleted = self.project_data_repository
            .delete(id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        if !deleted {
            return Err(ServiceError::NotFound("Project data not found".to_string()));
        }

        Ok(())
    }

    async fn get_project_data_access_matrix(
        &self,
        project_id: i32,
        page: i32,
        page_size: i32
    ) -> Result<(Vec<ProjectData>, Vec<ProjectDataAccess>), ServiceError> {
        let project_data_list = self.project_data_repository
            .find_by_project_id(project_id, page, page_size)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        let access_list = self.project_data_access_repository
            .find_matrix_by_project_id(project_id, page, page_size)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        Ok((project_data_list, access_list))
    }

    async fn get_data_access(&self, project_data_id: i32, user_id: i32) -> Result<ProjectDataAccess, ServiceError> {
        self.project_data_access_repository
            .find_by_project_data_and_user(project_data_id, user_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
            .ok_or_else(|| ServiceError::NotFound("Data access not found".to_string()))
    }

    async fn update_data_access(
        &self,
        project_data_id: i32,
        user_id: i32,
        update_access: UpdateProjectDataAccess
    ) -> Result<ProjectDataAccess, ServiceError> {
        let access = self.project_data_access_repository
            .update_by_project_data_and_user(project_data_id, user_id, &update_access)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
            .ok_or_else(|| ServiceError::NotFound("Data access not found".to_string()))?;

        Ok(access)
    }

    async fn batch_update_data_access(
        &self,
        project_data_id: i32,
        user_ids: Vec<i32>,
        update_access: UpdateProjectDataAccess
    ) -> Result<Vec<ProjectDataAccess>, ServiceError> {
        let results = self.project_data_access_repository
            .update_batch(project_data_id, &user_ids, &update_access)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        Ok(results)
    }

    async fn request_data_access(
        &self,
        project_data_id: i32,
        user_id: i32,
        requested_by: i32
    ) -> Result<ProjectDataAccess, ServiceError> {
        // Check if access already exists
        if let Some(_) = self.project_data_access_repository
            .find_by_project_data_and_user(project_data_id, user_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))? {
            return Err(ServiceError::AlreadyExists("Access already exists".to_string()));
        }

        let new_access = NewProjectDataAccess::new(project_data_id, user_id, DataAccessStatus::Pending)
            .with_request_info(requested_by);

        let access = self.project_data_access_repository
            .create(&new_access)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        Ok(access)
    }

    async fn grant_default_access_to_user(
        &self,
        project_id: i32,
        user_id: i32
    ) -> Result<Vec<ProjectDataAccess>, ServiceError> {
        // Get all project data for the project
        let project_data_list = self.project_data_repository
            .find_by_project_id(project_id, 1, 1000) // Get all data (assuming max 1000)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        let mut access_list = Vec::new();
        
        for project_data in project_data_list {
            // Check if access already exists
            if let None = self.project_data_access_repository
                .find_by_project_data_and_user(project_data.id, user_id)
                .await
                .map_err(|e| ServiceError::DatabaseError(e.to_string()))? {
                
                let new_access = NewProjectDataAccess::new(project_data.id, user_id, DataAccessStatus::Approved);
                let access = self.project_data_access_repository
                    .create(&new_access)
                    .await
                    .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
                
                access_list.push(access);
            }
        }

        Ok(access_list)
    }

    async fn grant_access_to_existing_users(
        &self,
        project_data_id: i32,
        project_id: i32
    ) -> Result<Vec<ProjectDataAccess>, ServiceError> {
        // Get all users in the project
        let users = sqlx::query_as::<_, (i32,)>(
            "SELECT user_id FROM security_user_project WHERE project_id = $1"
        )
        .bind(project_id)
        .fetch_all(self.project_data_access_repository.pool())
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        let mut access_list = Vec::new();
        
        for (user_id,) in users {
            // Check if access already exists
            if let None = self.project_data_access_repository
                .find_by_project_data_and_user(project_data_id, user_id)
                .await
                .map_err(|e| ServiceError::DatabaseError(e.to_string()))? {
                
                let new_access = NewProjectDataAccess::new(project_data_id, user_id, DataAccessStatus::Approved);
                let access = self.project_data_access_repository
                    .create(&new_access)
                    .await
                    .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
                
                access_list.push(access);
            }
        }

        Ok(access_list)
    }

    async fn get_access_by_status(
        &self,
        status: DataAccessStatus,
        page: i32,
        page_size: i32
    ) -> Result<Vec<ProjectDataAccess>, ServiceError> {
        self.project_data_access_repository
            .find_by_status(status, page, page_size)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    async fn get_user_access_list(
        &self,
        user_id: i32,
        page: i32,
        page_size: i32
    ) -> Result<Vec<ProjectDataAccess>, ServiceError> {
        self.project_data_access_repository
            .find_by_user_id(user_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    async fn delete_data_access(&self, project_data_id: i32, user_id: i32) -> Result<(), ServiceError> {
        let deleted = self.project_data_access_repository
            .delete_by_project_data_and_user(project_data_id, user_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        if !deleted {
            return Err(ServiceError::NotFound("Data access not found".to_string()));
        }

        Ok(())
    }
    
    // ========== 새로운 계층 구조 메서드 구현 ==========
    
    async fn get_study_by_id(&self, id: i32) -> Result<ProjectDataStudy, ServiceError> {
        self.project_data_repository
            .find_study_by_id(id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
            .ok_or_else(|| ServiceError::NotFound("Study not found".to_string()))
    }
    
    async fn get_study_by_uid(&self, project_id: i32, study_uid: &str) -> Result<ProjectDataStudy, ServiceError> {
        self.project_data_repository
            .find_study_by_uid(project_id, study_uid)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
            .ok_or_else(|| ServiceError::NotFound("Study not found".to_string()))
    }
    
    async fn get_studies_by_project(
        &self,
        project_id: i32,
        page: i32,
        page_size: i32
    ) -> Result<(Vec<ProjectDataStudy>, i64), ServiceError> {
        let studies = self.project_data_repository
            .find_studies_by_project_id(project_id, page, page_size)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        let total = self.project_data_repository
            .count_studies_by_project_id(project_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        Ok((studies, total))
    }
    
    async fn get_series_by_id(&self, id: i32) -> Result<ProjectDataSeries, ServiceError> {
        self.project_data_repository
            .find_series_by_id(id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
            .ok_or_else(|| ServiceError::NotFound("Series not found".to_string()))
    }
    
    async fn get_series_by_study(&self, study_id: i32) -> Result<Vec<ProjectDataSeries>, ServiceError> {
        self.project_data_repository
            .find_series_by_study_id(study_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }
}
