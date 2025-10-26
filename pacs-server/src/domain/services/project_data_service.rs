use crate::domain::entities::project_data::{ProjectData, ProjectDataAccess, ProjectDataStudy, ProjectDataSeries, NewProjectData, UpdateProjectData, NewProjectDataAccess, UpdateProjectDataAccess, DataAccessStatus};
use crate::domain::ServiceError;

#[async_trait::async_trait]
pub trait ProjectDataService: Send + Sync {
    /// 프로젝트 데이터 등록
    async fn create_project_data(&self, new_data: NewProjectData) -> Result<ProjectData, ServiceError>;
    
    /// 프로젝트 데이터 조회
    async fn get_project_data(&self, id: i32) -> Result<ProjectData, ServiceError>;
    
    /// 프로젝트별 데이터 목록 조회 (페이지네이션)
    async fn get_project_data_list(
        &self,
        project_id: i32,
        page: i32,
        page_size: i32
    ) -> Result<Vec<ProjectData>, ServiceError>;
    
    /// 프로젝트 데이터 검색
    async fn search_project_data(
        &self,
        project_id: i32,
        search_term: &str,
        page: i32,
        page_size: i32
    ) -> Result<Vec<ProjectData>, ServiceError>;
    
    /// 프로젝트 데이터 업데이트
    async fn update_project_data(&self, id: i32, update_data: UpdateProjectData) -> Result<ProjectData, ServiceError>;
    
    /// 프로젝트 데이터 삭제
    async fn delete_project_data(&self, id: i32) -> Result<(), ServiceError>;
    
    /// 프로젝트 데이터 접근 매트릭스 조회
    async fn get_project_data_access_matrix(
        &self,
        project_id: i32,
        page: i32,
        page_size: i32
    ) -> Result<(Vec<ProjectData>, Vec<ProjectDataAccess>), ServiceError>;
    
    /// 프로젝트 데이터 접근 권한 조회
    async fn get_data_access(&self, project_data_id: i32, user_id: i32) -> Result<ProjectDataAccess, ServiceError>;
    
    /// 프로젝트 데이터 접근 권한 수정
    async fn update_data_access(
        &self,
        project_data_id: i32,
        user_id: i32,
        update_access: UpdateProjectDataAccess
    ) -> Result<ProjectDataAccess, ServiceError>;
    
    /// 일괄 접근 권한 수정
    async fn batch_update_data_access(
        &self,
        project_data_id: i32,
        user_ids: Vec<i32>,
        update_access: UpdateProjectDataAccess
    ) -> Result<Vec<ProjectDataAccess>, ServiceError>;
    
    /// 접근 요청
    async fn request_data_access(
        &self,
        project_data_id: i32,
        user_id: i32,
        requested_by: i32
    ) -> Result<ProjectDataAccess, ServiceError>;
    
    /// 프로젝트 참가 시 기본 접근 권한 부여
    async fn grant_default_access_to_user(
        &self,
        project_id: i32,
        user_id: i32
    ) -> Result<Vec<ProjectDataAccess>, ServiceError>;
    
    /// 새 프로젝트 데이터 등록 시 기존 참가자들에게 접근 권한 부여
    async fn grant_access_to_existing_users(
        &self,
        project_data_id: i32,
        project_id: i32
    ) -> Result<Vec<ProjectDataAccess>, ServiceError>;
    
    /// 상태별 접근 권한 필터링
    async fn get_access_by_status(
        &self,
        status: DataAccessStatus,
        page: i32,
        page_size: i32
    ) -> Result<Vec<ProjectDataAccess>, ServiceError>;
    
    /// 사용자별 접근 권한 조회
    async fn get_user_access_list(
        &self,
        user_id: i32,
        page: i32,
        page_size: i32
    ) -> Result<Vec<ProjectDataAccess>, ServiceError>;
    
    /// 프로젝트 데이터 접근 권한 삭제
    async fn delete_data_access(&self, project_data_id: i32, user_id: i32) -> Result<(), ServiceError>;
    
    // ========== 새로운 계층 구조 메서드 ==========
    
    /// Study 조회 (by ID)
    async fn get_study_by_id(&self, id: i32) -> Result<ProjectDataStudy, ServiceError>;
    
    /// Study 조회 (by project_id and study_uid)
    async fn get_study_by_uid(&self, project_id: i32, study_uid: &str) -> Result<ProjectDataStudy, ServiceError>;
    
    /// 프로젝트별 Study 목록 조회 (페이지네이션)
    async fn get_studies_by_project(
        &self,
        project_id: i32,
        page: i32,
        page_size: i32
    ) -> Result<(Vec<ProjectDataStudy>, i64), ServiceError>;
    
    /// Series 조회 (by ID)
    async fn get_series_by_id(&self, id: i32) -> Result<ProjectDataSeries, ServiceError>;
    
    /// Study별 Series 목록 조회
    async fn get_series_by_study(&self, study_id: i32) -> Result<Vec<ProjectDataSeries>, ServiceError>;
}
