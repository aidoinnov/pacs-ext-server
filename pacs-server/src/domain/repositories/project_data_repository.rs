use crate::domain::entities::project_data::{
    NewProjectData, ProjectData, ProjectDataInstance, ProjectDataPatient, ProjectDataSeries,
    ProjectDataStudy, UpdateProjectData,
};
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
        page_size: i32,
    ) -> Result<Vec<ProjectData>, sqlx::Error>;

    /// 프로젝트별 데이터 총 개수 조회
    async fn count_by_project_id(&self, project_id: i32) -> Result<i64, sqlx::Error>;

    /// Study UID로 프로젝트 데이터 조회
    async fn find_by_study_uid(
        &self,
        project_id: i32,
        study_uid: &str,
    ) -> Result<Option<ProjectData>, sqlx::Error>;

    /// 프로젝트 데이터 검색 (Study UID, Patient ID, Patient Name)
    async fn search_by_project_id(
        &self,
        project_id: i32,
        search_term: &str,
        page: i32,
        page_size: i32,
    ) -> Result<Vec<ProjectData>, sqlx::Error>;

    /// 검색 결과 총 개수
    async fn count_search_results(
        &self,
        project_id: i32,
        search_term: &str,
    ) -> Result<i64, sqlx::Error>;

    /// 프로젝트 데이터 업데이트
    async fn update(
        &self,
        id: i32,
        update_data: &UpdateProjectData,
    ) -> Result<Option<ProjectData>, sqlx::Error>;

    /// 프로젝트 데이터 삭제
    async fn delete(&self, id: i32) -> Result<bool, sqlx::Error>;

    /// 데이터베이스 연결 풀 반환
    fn pool(&self) -> &PgPool;

    // ========== 새로운 계층 구조 메서드 ==========

    /// Study 조회 (by ID)
    async fn find_study_by_id(&self, id: i32) -> Result<Option<ProjectDataStudy>, sqlx::Error>;

    /// Study 조회 (by project_id and study_uid)
    async fn find_study_by_uid(
        &self,
        project_id: i32,
        study_uid: &str,
    ) -> Result<Option<ProjectDataStudy>, sqlx::Error>;

    /// 프로젝트별 Study 목록 조회 (페이지네이션)
    async fn find_studies_by_project_id(
        &self,
        project_id: i32,
        page: i32,
        page_size: i32,
    ) -> Result<Vec<ProjectDataStudy>, sqlx::Error>;

    /// 프로젝트별 Study 총 개수
    async fn count_studies_by_project_id(&self, project_id: i32) -> Result<i64, sqlx::Error>;

    /// 프로젝트별 Study 목록 최종 수정 시간 조회 (ETag 캐싱용)
    async fn get_studies_updated_at(&self, project_id: i32) -> Result<chrono::DateTime<chrono::Utc>, sqlx::Error>;

    /// Series 조회 (by ID)
    async fn find_series_by_id(&self, id: i32) -> Result<Option<ProjectDataSeries>, sqlx::Error>;

    /// Study별 Series 목록 조회
    async fn find_series_by_study_id(
        &self,
        study_id: i32,
    ) -> Result<Vec<ProjectDataSeries>, sqlx::Error>;

    /// Study별 Series 총 개수
    async fn count_series_by_study_id(&self, study_id: i32) -> Result<i64, sqlx::Error>;

    /// 프로젝트에 할당된 Series 목록 조회 (Study별)
    async fn find_series_by_project_and_study_id(
        &self,
        project_id: i32,
        study_id: i32,
    ) -> Result<Vec<ProjectDataSeries>, sqlx::Error>;

    /// Instance 조회 (by ID)
    async fn find_instance_by_id(
        &self,
        id: i32,
    ) -> Result<Option<ProjectDataInstance>, sqlx::Error>;

    /// Series별 Instance 목록 조회
    async fn find_instances_by_series_id(
        &self,
        series_id: i32,
    ) -> Result<Vec<ProjectDataInstance>, sqlx::Error>;

    /// Series별 Instance 총 개수
    async fn count_instances_by_series_id(&self, series_id: i32) -> Result<i64, sqlx::Error>;

    /// 프로젝트에 할당된 Patient 목록 조회 (필터링, 페이지네이션)
    async fn find_patients_by_project(
        &self,
        project_id: i32,
        patient_id_filter: Option<&str>,
        patient_name_filter: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ProjectDataPatient>, sqlx::Error>;

    /// 프로젝트에 할당된 Patient 총 개수
    async fn count_patients_by_project(
        &self,
        project_id: i32,
        patient_id_filter: Option<&str>,
        patient_name_filter: Option<&str>,
    ) -> Result<i64, sqlx::Error>;
}

// Arc<T>가 ProjectDataRepository를 구현하도록 blanket implementation 추가
#[async_trait::async_trait]
impl<T: ProjectDataRepository + ?Sized> ProjectDataRepository for std::sync::Arc<T> {
    async fn create(&self, new_data: &NewProjectData) -> Result<ProjectData, sqlx::Error> {
        (**self).create(new_data).await
    }

    async fn find_by_id(&self, id: i32) -> Result<Option<ProjectData>, sqlx::Error> {
        (**self).find_by_id(id).await
    }

    async fn find_by_project_id(
        &self,
        project_id: i32,
        page: i32,
        page_size: i32,
    ) -> Result<Vec<ProjectData>, sqlx::Error> {
        (**self).find_by_project_id(project_id, page, page_size).await
    }

    async fn count_by_project_id(&self, project_id: i32) -> Result<i64, sqlx::Error> {
        (**self).count_by_project_id(project_id).await
    }

    async fn find_by_study_uid(
        &self,
        project_id: i32,
        study_uid: &str,
    ) -> Result<Option<ProjectData>, sqlx::Error> {
        (**self).find_by_study_uid(project_id, study_uid).await
    }

    async fn search_by_project_id(
        &self,
        project_id: i32,
        search_term: &str,
        page: i32,
        page_size: i32,
    ) -> Result<Vec<ProjectData>, sqlx::Error> {
        (**self).search_by_project_id(project_id, search_term, page, page_size).await
    }

    async fn count_search_results(
        &self,
        project_id: i32,
        search_term: &str,
    ) -> Result<i64, sqlx::Error> {
        (**self).count_search_results(project_id, search_term).await
    }

    async fn update(&self, id: i32, update: &UpdateProjectData) -> Result<Option<ProjectData>, sqlx::Error> {
        (**self).update(id, update).await
    }

    async fn delete(&self, id: i32) -> Result<bool, sqlx::Error> {
        (**self).delete(id).await
    }

    fn pool(&self) -> &sqlx::PgPool {
        (**self).pool()
    }

    async fn find_study_by_id(&self, id: i32) -> Result<Option<ProjectDataStudy>, sqlx::Error> {
        (**self).find_study_by_id(id).await
    }

    async fn find_study_by_uid(
        &self,
        project_id: i32,
        study_uid: &str,
    ) -> Result<Option<ProjectDataStudy>, sqlx::Error> {
        (**self).find_study_by_uid(project_id, study_uid).await
    }

    async fn find_studies_by_project_id(
        &self,
        project_id: i32,
        page: i32,
        page_size: i32,
    ) -> Result<Vec<ProjectDataStudy>, sqlx::Error> {
        (**self).find_studies_by_project_id(project_id, page, page_size).await
    }

    async fn count_studies_by_project_id(&self, project_id: i32) -> Result<i64, sqlx::Error> {
        (**self).count_studies_by_project_id(project_id).await
    }

    async fn get_studies_updated_at(&self, project_id: i32) -> Result<chrono::DateTime<chrono::Utc>, sqlx::Error> {
        (**self).get_studies_updated_at(project_id).await
    }

    async fn find_series_by_id(&self, id: i32) -> Result<Option<ProjectDataSeries>, sqlx::Error> {
        (**self).find_series_by_id(id).await
    }

    async fn find_series_by_study_id(
        &self,
        study_id: i32,
    ) -> Result<Vec<ProjectDataSeries>, sqlx::Error> {
        (**self).find_series_by_study_id(study_id).await
    }

    async fn count_series_by_study_id(&self, study_id: i32) -> Result<i64, sqlx::Error> {
        (**self).count_series_by_study_id(study_id).await
    }

    async fn find_series_by_project_and_study_id(
        &self,
        project_id: i32,
        study_id: i32,
    ) -> Result<Vec<ProjectDataSeries>, sqlx::Error> {
        (**self).find_series_by_project_and_study_id(project_id, study_id).await
    }

    async fn find_instance_by_id(
        &self,
        id: i32,
    ) -> Result<Option<ProjectDataInstance>, sqlx::Error> {
        (**self).find_instance_by_id(id).await
    }

    async fn find_instances_by_series_id(
        &self,
        series_id: i32,
    ) -> Result<Vec<ProjectDataInstance>, sqlx::Error> {
        (**self).find_instances_by_series_id(series_id).await
    }

    async fn count_instances_by_series_id(&self, series_id: i32) -> Result<i64, sqlx::Error> {
        (**self).count_instances_by_series_id(series_id).await
    }

    async fn find_patients_by_project(
        &self,
        project_id: i32,
        patient_id_filter: Option<&str>,
        patient_name_filter: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ProjectDataPatient>, sqlx::Error> {
        (**self).find_patients_by_project(project_id, patient_id_filter, patient_name_filter, limit, offset).await
    }

    async fn count_patients_by_project(
        &self,
        project_id: i32,
        patient_id_filter: Option<&str>,
        patient_name_filter: Option<&str>,
    ) -> Result<i64, sqlx::Error> {
        (**self).count_patients_by_project(project_id, patient_id_filter, patient_name_filter).await
    }
}
