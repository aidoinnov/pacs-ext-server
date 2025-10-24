use crate::domain::entities::project_data::{ProjectData, NewProjectData, UpdateProjectData};
use sqlx::PgPool;

#[async_trait::async_trait]
pub trait ProjectDataRepository: Send + Sync {
    /// 프로젝트 데이터 생성
    async fn create(&self, new_data: &NewProjectData) -> Result<ProjectData, sqlx::Error>;
    
    /// ID로 프로젝트 데이터 조회
    async fn find_by_id(&self, id: i32) -> Result<Option<ProjectData>, sqlx::Error>;
    
    /// 프로젝트별 데이터 목록 조회 (페이지네이션)
    async fn find_by_project_id(
        &self, 
        project_id: i32, 
        page: i32, 
        page_size: i32
    ) -> Result<Vec<ProjectData>, sqlx::Error>;
    
    /// 프로젝트별 데이터 총 개수 조회
    async fn count_by_project_id(&self, project_id: i32) -> Result<i64, sqlx::Error>;
    
    /// Study UID로 프로젝트 데이터 조회
    async fn find_by_study_uid(
        &self, 
        project_id: i32, 
        study_uid: &str
    ) -> Result<Option<ProjectData>, sqlx::Error>;
    
    /// 프로젝트 데이터 검색 (Study UID, Patient ID, Patient Name)
    async fn search_by_project_id(
        &self,
        project_id: i32,
        search_term: &str,
        page: i32,
        page_size: i32
    ) -> Result<Vec<ProjectData>, sqlx::Error>;
    
    /// 검색 결과 총 개수
    async fn count_search_results(
        &self,
        project_id: i32,
        search_term: &str
    ) -> Result<i64, sqlx::Error>;
    
    /// 프로젝트 데이터 업데이트
    async fn update(&self, id: i32, update_data: &UpdateProjectData) -> Result<Option<ProjectData>, sqlx::Error>;
    
    /// 프로젝트 데이터 삭제
    async fn delete(&self, id: i32) -> Result<bool, sqlx::Error>;
    
    /// 데이터베이스 연결 풀 반환
    fn pool(&self) -> &PgPool;
}
