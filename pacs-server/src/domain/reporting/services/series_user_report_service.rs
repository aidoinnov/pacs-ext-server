//! # Series User Report Service 트레이트
//!
//! 이 모듈은 Series User Report 비즈니스 로직을 위한 Service 트레이트를 정의합니다.

use crate::domain::reporting::entities::series_user_report::{
    NewSeriesUserReport, SeriesUserReport, UpdateSeriesUserReport,
};
use crate::domain::reporting::repositories::SeriesUserReportRepository;
use crate::domain::repositories::{
    ProjectDataRepository, ProjectRepository, UserRepository,
};
use crate::domain::ServiceError;
use async_trait::async_trait;
use std::sync::Arc;

#[async_trait]
pub trait SeriesUserReportService: Send + Sync + 'static {
    /// Report 생성 또는 업데이트
    async fn create_or_update_report(
        &self,
        series_id: i32,
        user_id: i32,
        project_id: Option<i32>,
        new_report: NewSeriesUserReport,
    ) -> Result<SeriesUserReport, ServiceError>;

    /// Report 조회
    async fn get_report(
        &self,
        series_id: i32,
        user_id: i32,
        project_id: Option<i32>,
    ) -> Result<Option<SeriesUserReport>, ServiceError>;

    /// 시리즈의 모든 Report 조회
    async fn get_reports_by_series(
        &self,
        series_id: i32,
        project_id: Option<i32>,
    ) -> Result<Vec<SeriesUserReport>, ServiceError>;

    /// Report 업데이트
    async fn update_report(
        &self,
        series_id: i32,
        user_id: i32,
        project_id: Option<i32>,
        update: UpdateSeriesUserReport,
    ) -> Result<SeriesUserReport, ServiceError>;

    /// Report 삭제
    async fn delete_report(
        &self,
        series_id: i32,
        user_id: i32,
        project_id: Option<i32>,
    ) -> Result<(), ServiceError>;
}

/// Series User Report Service 구현체
#[derive(Clone)]
pub struct SeriesUserReportServiceImpl<R, U, P, PD>
where
    R: SeriesUserReportRepository + 'static,
    U: UserRepository + 'static,
    P: ProjectRepository + 'static,
    PD: ProjectDataRepository + 'static,
{
    report_repository: Arc<R>,
    user_repository: Arc<U>,
    project_repository: Arc<P>,
    project_data_repository: Arc<PD>,
}

impl<R, U, P, PD> SeriesUserReportServiceImpl<R, U, P, PD>
where
    R: SeriesUserReportRepository + 'static,
    U: UserRepository + 'static,
    P: ProjectRepository + 'static,
    PD: ProjectDataRepository + 'static,
{
    pub fn new(
        report_repository: R,
        user_repository: U,
        project_repository: P,
        project_data_repository: Arc<PD>,
    ) -> Self {
        Self {
            report_repository: Arc::new(report_repository),
            user_repository: Arc::new(user_repository),
            project_repository: Arc::new(project_repository),
            project_data_repository,
        }
    }
}

#[async_trait]
impl<R, U, P, PD> SeriesUserReportService for SeriesUserReportServiceImpl<R, U, P, PD>
where
    R: SeriesUserReportRepository + 'static,
    U: UserRepository + 'static,
    P: ProjectRepository + 'static,
    PD: ProjectDataRepository + 'static,
{
    async fn create_or_update_report(
        &self,
        series_id: i32,
        user_id: i32,
        project_id: Option<i32>,
        new_report: NewSeriesUserReport,
    ) -> Result<SeriesUserReport, ServiceError> {
        // 사용자 존재 확인
        if self
            .user_repository
            .as_ref()
            .find_by_id(user_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
            .is_none()
        {
            return Err(ServiceError::NotFound("User not found".into()));
        }

        // Series 존재 확인
        if self
            .project_data_repository
            .as_ref()
            .find_series_by_id(series_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
            .is_none()
        {
            return Err(ServiceError::NotFound("Series not found".into()));
        }

        // project_id가 있는 경우, 프로젝트 멤버십 확인
        if let Some(pid) = project_id {
            // 프로젝트 존재 확인
            if self
                .project_repository
                .as_ref()
                .find_by_id(pid)
                .await
                .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
                .is_none()
            {
                return Err(ServiceError::NotFound("Project not found".into()));
            }

            // 사용자가 프로젝트 멤버인지 확인
            let is_member = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM security_user_project WHERE user_id = $1 AND project_id = $2",
            )
            .bind(user_id)
            .bind(pid)
            .fetch_one(self.report_repository.as_ref().pool())
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

            if is_member == 0 {
                return Err(ServiceError::Unauthorized(
                    "User is not a member of this project".into(),
                ));
            }
        }

        // Report 생성 또는 업데이트
        self.report_repository
            .as_ref()
            .create_or_update(series_id, user_id, project_id, &new_report)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    async fn get_report(
        &self,
        series_id: i32,
        user_id: i32,
        project_id: Option<i32>,
    ) -> Result<Option<SeriesUserReport>, ServiceError> {
        self.report_repository
            .as_ref()
            .find_by_series_user_project(series_id, user_id, project_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    async fn get_reports_by_series(
        &self,
        series_id: i32,
        project_id: Option<i32>,
    ) -> Result<Vec<SeriesUserReport>, ServiceError> {
        self.report_repository
            .as_ref()
            .find_by_series(series_id, project_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    async fn update_report(
        &self,
        series_id: i32,
        user_id: i32,
        project_id: Option<i32>,
        update: UpdateSeriesUserReport,
    ) -> Result<SeriesUserReport, ServiceError> {
        // Report 존재 확인
        let existing = self
            .get_report(series_id, user_id, project_id)
            .await?;

        if existing.is_none() {
            return Err(ServiceError::NotFound("Report not found".into()));
        }

        self.report_repository
            .as_ref()
            .update(series_id, user_id, project_id, &update)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    async fn delete_report(
        &self,
        series_id: i32,
        user_id: i32,
        project_id: Option<i32>,
    ) -> Result<(), ServiceError> {
        let deleted = self
            .report_repository
            .as_ref()
            .delete(series_id, user_id, project_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        if !deleted {
            return Err(ServiceError::NotFound("Report not found".into()));
        }

        Ok(())
    }
}
