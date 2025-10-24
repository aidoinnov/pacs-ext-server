use crate::domain::entities::project_data::{ProjectDataAccess, NewProjectDataAccess, UpdateProjectDataAccess, DataAccessStatus};
use sqlx::PgPool;

#[async_trait::async_trait]
pub trait ProjectDataAccessRepository: Send + Sync {
    /// 프로젝트 데이터 접근 권한 생성
    async fn create(&self, new_access: &NewProjectDataAccess) -> Result<ProjectDataAccess, sqlx::Error>;
    
    /// ID로 접근 권한 조회
    async fn find_by_id(&self, id: i32) -> Result<Option<ProjectDataAccess>, sqlx::Error>;
    
    /// 프로젝트 데이터별 접근 권한 조회
    async fn find_by_project_data_id(&self, project_data_id: i32) -> Result<Vec<ProjectDataAccess>, sqlx::Error>;
    
    /// 사용자별 접근 권한 조회
    async fn find_by_user_id(&self, user_id: i32) -> Result<Vec<ProjectDataAccess>, sqlx::Error>;
    
    /// 특정 데이터에 대한 사용자 접근 권한 조회
    async fn find_by_project_data_and_user(
        &self, 
        project_data_id: i32, 
        user_id: i32
    ) -> Result<Option<ProjectDataAccess>, sqlx::Error>;
    
    /// 프로젝트별 접근 권한 매트릭스 조회 (페이지네이션)
    async fn find_matrix_by_project_id(
        &self,
        project_id: i32,
        page: i32,
        page_size: i32
    ) -> Result<Vec<ProjectDataAccess>, sqlx::Error>;
    
    /// 상태별 접근 권한 필터링
    async fn find_by_status(
        &self,
        status: DataAccessStatus,
        page: i32,
        page_size: i32
    ) -> Result<Vec<ProjectDataAccess>, sqlx::Error>;
    
    /// 사용자별 상태별 접근 권한 필터링
    async fn find_by_user_and_status(
        &self,
        user_id: i32,
        status: DataAccessStatus,
        page: i32,
        page_size: i32
    ) -> Result<Vec<ProjectDataAccess>, sqlx::Error>;
    
    /// 프로젝트별 접근 권한 총 개수
    async fn count_by_project_id(&self, project_id: i32) -> Result<i64, sqlx::Error>;
    
    /// 상태별 접근 권한 총 개수
    async fn count_by_status(&self, status: DataAccessStatus) -> Result<i64, sqlx::Error>;
    
    /// 접근 권한 업데이트
    async fn update(&self, id: i32, update_access: &UpdateProjectDataAccess) -> Result<Option<ProjectDataAccess>, sqlx::Error>;
    
    /// 특정 데이터에 대한 사용자 접근 권한 업데이트
    async fn update_by_project_data_and_user(
        &self,
        project_data_id: i32,
        user_id: i32,
        update_access: &UpdateProjectDataAccess
    ) -> Result<Option<ProjectDataAccess>, sqlx::Error>;
    
    /// 일괄 접근 권한 생성
    async fn create_batch(&self, access_list: &[NewProjectDataAccess]) -> Result<Vec<ProjectDataAccess>, sqlx::Error>;
    
    /// 일괄 접근 권한 업데이트
    async fn update_batch(
        &self,
        project_data_id: i32,
        user_ids: &[i32],
        update_access: &UpdateProjectDataAccess
    ) -> Result<Vec<ProjectDataAccess>, sqlx::Error>;
    
    /// 접근 권한 삭제
    async fn delete(&self, id: i32) -> Result<bool, sqlx::Error>;
    
    /// 특정 데이터에 대한 사용자 접근 권한 삭제
    async fn delete_by_project_data_and_user(
        &self,
        project_data_id: i32,
        user_id: i32
    ) -> Result<bool, sqlx::Error>;
    
    /// 데이터베이스 연결 풀 반환
    fn pool(&self) -> &PgPool;
}
