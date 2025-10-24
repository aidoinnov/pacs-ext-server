use std::sync::Arc;
use std::str::FromStr;
use crate::domain::services::ProjectDataService;
use crate::domain::ServiceError;
use crate::application::dto::project_data_access_dto::*;
use crate::domain::entities::project_data::{NewProjectData, UpdateProjectData, UpdateProjectDataAccess, DataAccessStatus};

pub struct ProjectDataAccessUseCase {
    project_data_service: Arc<dyn ProjectDataService>,
}

impl ProjectDataAccessUseCase {
    pub fn new(project_data_service: Arc<dyn ProjectDataService>) -> Self {
        Self { project_data_service }
    }

    /// 프로젝트 데이터 접근 매트릭스 조회
    pub async fn get_project_data_access_matrix(
        &self,
        project_id: i32,
        page: i32,
        page_size: i32,
        search: Option<String>,
        status: Option<String>,
        user_id: Option<i32>,
    ) -> Result<ProjectDataAccessMatrixResponse, ServiceError> {
        // Get project data list
        let project_data_list = if let Some(search_term) = search {
            self.project_data_service
                .search_project_data(project_id, &search_term, page, page_size)
                .await?
        } else {
            self.project_data_service
                .get_project_data_list(project_id, page, page_size)
                .await?
        };

        // Get access matrix
        let (_, access_list) = self.project_data_service
            .get_project_data_access_matrix(project_id, page, page_size)
            .await?;

        // Convert to DTOs
        let data_list: Vec<ProjectDataInfo> = project_data_list
            .into_iter()
            .map(|data| ProjectDataInfo {
                id: data.id,
                study_uid: data.study_uid,
                study_description: data.study_description,
                patient_id: data.patient_id,
                patient_name: data.patient_name,
                study_date: data.study_date.map(|d| d.to_string()),
                modality: data.modality,
            })
            .collect();

        // Get users from access list
        let user_ids: std::collections::HashSet<i32> = access_list
            .iter()
            .map(|access| access.user_id)
            .collect();

        // TODO: Get user information from UserService
        // For now, create mock user info
        let users: Vec<UserInfo> = user_ids
            .into_iter()
            .map(|id| UserInfo {
                id,
                username: format!("user{}", id),
                email: format!("user{}@example.com", id),
                full_name: Some(format!("사용자 {}", id)),
                organization: Some("서울대학교병원".to_string()),
            })
            .collect();

        // Convert access list to DTOs
        let access_matrix: Vec<DataAccessInfo> = access_list
            .into_iter()
            .map(|access| DataAccessInfo {
                project_data_id: access.project_data_id,
                user_id: access.user_id,
                status: access.status.to_string(),
                reviewed_at: access.reviewed_at.map(|t| t.to_rfc3339()),
                reviewed_by: access.reviewed_by,
            })
            .collect();

        // Calculate pagination
        let total_items = self.project_data_service
            .get_project_data_list(project_id, 1, 1000) // Get total count
            .await?
            .len() as i64;

        let total_pages = (total_items + page_size as i64 - 1) / page_size as i64;

        let pagination = PaginationInfo {
            page,
            page_size,
            total_items,
            total_pages,
        };

        Ok(ProjectDataAccessMatrixResponse {
            data_list,
            users,
            access_matrix,
            pagination,
        })
    }

    /// 프로젝트 데이터 생성
    pub async fn create_project_data(
        &self,
        project_id: i32,
        request: CreateProjectDataRequest,
    ) -> Result<CreateProjectDataResponse, ServiceError> {
        let mut new_data = NewProjectData::new(project_id, request.study_uid)
            .with_description(request.study_description.unwrap_or_default())
            .with_patient_info(
                request.patient_id.unwrap_or_default(),
                request.patient_name.unwrap_or_default(),
            )
            .with_modality(request.modality.unwrap_or_default());

        if let Some(study_date_str) = request.study_date {
            if let Ok(study_date) = chrono::NaiveDate::parse_from_str(&study_date_str, "%Y-%m-%d") {
                new_data = new_data.with_study_date(study_date);
            }
        }

        let project_data = self.project_data_service.create_project_data(new_data).await?;

        Ok(CreateProjectDataResponse {
            success: true,
            message: "Project data created successfully".to_string(),
            data_id: Some(project_data.id),
        })
    }

    /// 개별 접근 권한 수정
    pub async fn update_data_access(
        &self,
        project_data_id: i32,
        user_id: i32,
        request: UpdateDataAccessRequest,
    ) -> Result<UpdateDataAccessResponse, ServiceError> {
        let status = DataAccessStatus::from_str(&request.status)
            .map_err(|e| ServiceError::ValidationError(e))?;

        let update_access = UpdateProjectDataAccess {
            status: Some(status),
            review_note: request.review_note,
            reviewed_at: Some(chrono::Utc::now()),
            reviewed_by: Some(1), // TODO: Get from current user context
            ..Default::default()
        };

        self.project_data_service
            .update_data_access(project_data_id, user_id, update_access)
            .await?;

        Ok(UpdateDataAccessResponse {
            success: true,
            message: "Data access updated successfully".to_string(),
        })
    }

    /// 일괄 접근 권한 수정
    pub async fn batch_update_data_access(
        &self,
        project_data_id: i32,
        request: BatchUpdateDataAccessRequest,
    ) -> Result<BatchUpdateDataAccessResponse, ServiceError> {
        let status = DataAccessStatus::from_str(&request.status)
            .map_err(|e| ServiceError::ValidationError(e))?;

        let update_access = UpdateProjectDataAccess {
            status: Some(status),
            review_note: request.review_note,
            reviewed_at: Some(chrono::Utc::now()),
            reviewed_by: Some(1), // TODO: Get from current user context
            ..Default::default()
        };

        let results = self.project_data_service
            .batch_update_data_access(project_data_id, request.user_ids, update_access)
            .await?;

        Ok(BatchUpdateDataAccessResponse {
            success: true,
            message: "Data access updated successfully".to_string(),
            updated_count: results.len() as i32,
        })
    }

    /// 접근 요청
    pub async fn request_data_access(
        &self,
        project_data_id: i32,
        user_id: i32,
    ) -> Result<RequestDataAccessResponse, ServiceError> {
        self.project_data_service
            .request_data_access(project_data_id, user_id, user_id) // TODO: Get from current user context
            .await?;

        Ok(RequestDataAccessResponse {
            success: true,
            message: "Data access requested successfully".to_string(),
        })
    }

    /// 프로젝트 참가 시 기본 접근 권한 부여
    pub async fn grant_default_access_to_user(
        &self,
        project_id: i32,
        user_id: i32,
    ) -> Result<(), ServiceError> {
        self.project_data_service
            .grant_default_access_to_user(project_id, user_id)
            .await?;

        Ok(())
    }

    /// 상태별 접근 권한 조회
    pub async fn get_access_by_status(
        &self,
        status: String,
        page: i32,
        page_size: i32,
    ) -> Result<Vec<DataAccessInfo>, ServiceError> {
        let data_access_status = DataAccessStatus::from_str(&status)
            .map_err(|e| ServiceError::ValidationError(e))?;

        let access_list = self.project_data_service
            .get_access_by_status(data_access_status, page, page_size)
            .await?;

        let access_matrix: Vec<DataAccessInfo> = access_list
            .into_iter()
            .map(|access| DataAccessInfo {
                project_data_id: access.project_data_id,
                user_id: access.user_id,
                status: access.status.to_string(),
                reviewed_at: access.reviewed_at.map(|t| t.to_rfc3339()),
                reviewed_by: access.reviewed_by,
            })
            .collect();

        Ok(access_matrix)
    }

    /// 사용자별 접근 권한 조회
    pub async fn get_user_access_list(
        &self,
        user_id: i32,
        page: i32,
        page_size: i32,
    ) -> Result<Vec<DataAccessInfo>, ServiceError> {
        let access_list = self.project_data_service
            .get_user_access_list(user_id, page, page_size)
            .await?;

        let access_matrix: Vec<DataAccessInfo> = access_list
            .into_iter()
            .map(|access| DataAccessInfo {
                project_data_id: access.project_data_id,
                user_id: access.user_id,
                status: access.status.to_string(),
                reviewed_at: access.reviewed_at.map(|t| t.to_rfc3339()),
                reviewed_by: access.reviewed_by,
            })
            .collect();

        Ok(access_matrix)
    }
}
