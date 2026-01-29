use crate::application::reporting::dto::series_user_report_dto::*;
use crate::domain::reporting::entities::series_user_report::{
    NewSeriesUserReport, SeriesUserReport, UpdateSeriesUserReport,
};
use crate::domain::reporting::services::SeriesUserReportService;
use crate::domain::repositories::UserRepository;
use crate::domain::ServiceError;
use std::sync::Arc;

pub struct SeriesUserReportUseCase<S, U>
where
    S: SeriesUserReportService,
    U: UserRepository,
{
    report_service: Arc<S>,
    user_repository: Arc<U>,
}

impl<S, U> SeriesUserReportUseCase<S, U>
where
    S: SeriesUserReportService,
    U: UserRepository,
{
    pub fn new(report_service: Arc<S>, user_repository: Arc<U>) -> Self {
        Self {
            report_service,
            user_repository,
        }
    }

    pub async fn create_or_update_report(
        &self,
        series_id: i32,
        user_id: i32,
        project_id: Option<i32>,
        request: CreateOrUpdateSeriesReportRequest,
    ) -> Result<SeriesReportResponse, ServiceError> {
        // status 값 검증
        let status = if let Some(s) = request.status {
            let normalized = s.to_lowercase();
            if !["unread", "approval", "unapproval"].contains(&normalized.as_str()) {
                return Err(ServiceError::ValidationError(
                    format!("Invalid status '{}'. Must be one of: unread, approval, unapproval", s)
                ));
            }
            normalized
        } else {
            "unread".to_string()
        };

        let new_report = NewSeriesUserReport {
            series_id,
            user_id,
            project_id,
            status,
            dictate_file_path: None,
            dictate_file_size: None,
            dictate_mime_type: None,
            description: request.description,
            conclusion: request.conclusion,
            bodypart: request.bodypart,
        };

        let report = self
            .report_service
            .as_ref()
            .create_or_update_report(series_id, user_id, project_id, new_report)
            .await?;

        Ok(self.to_response(report))
    }

    pub async fn get_report(
        &self,
        series_id: i32,
        user_id: i32,
        project_id: Option<i32>,
    ) -> Result<Option<SeriesReportResponse>, ServiceError> {
        let report = self
            .report_service
            .as_ref()
            .get_report(series_id, user_id, project_id)
            .await?;

        Ok(report.map(|r| self.to_response(r)))
    }

    pub async fn get_reports_by_series(
        &self,
        series_id: i32,
        project_id: Option<i32>,
    ) -> Result<SeriesReportListResponse, ServiceError> {
        let reports = self
            .report_service
            .as_ref()
            .get_reports_by_series(series_id, project_id)
            .await?;

        // 사용자 정보 조회 및 DTO 변환
        let mut reports_with_user = Vec::new();
        for report in reports {
            if let Some(user) = self
                .user_repository
                .as_ref()
                .find_by_id(report.user_id)
                .await
                .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
            {
                reports_with_user.push(self.to_response_with_user(report, &user));
            }
        }

        Ok(SeriesReportListResponse {
            success: true,
            reports: reports_with_user,
        })
    }

    pub async fn update_report(
        &self,
        series_id: i32,
        user_id: i32,
        project_id: Option<i32>,
        request: CreateOrUpdateSeriesReportRequest,
    ) -> Result<SeriesReportResponse, ServiceError> {
        // status 값 검증
        let validated_status = if let Some(s) = request.status {
            let normalized = s.to_lowercase();
            if !["unread", "approval", "unapproval"].contains(&normalized.as_str()) {
                return Err(ServiceError::ValidationError(
                    format!("Invalid status '{}'. Must be one of: unread, approval, unapproval", s)
                ));
            }
            Some(normalized)
        } else {
            None
        };

        let update = UpdateSeriesUserReport {
            status: validated_status,
            dictate_file_path: None,
            dictate_file_size: None,
            dictate_mime_type: None,
            description: Some(request.description),
            conclusion: Some(request.conclusion),
            bodypart: request.bodypart,
        };

        let report = self
            .report_service
            .as_ref()
            .update_report(series_id, user_id, project_id, update)
            .await?;

        Ok(self.to_response(report))
    }

    pub async fn delete_report(
        &self,
        series_id: i32,
        user_id: i32,
        project_id: Option<i32>,
    ) -> Result<(), ServiceError> {
        self.report_service
            .as_ref()
            .delete_report(series_id, user_id, project_id)
            .await
    }

    fn to_response(&self, report: SeriesUserReport) -> SeriesReportResponse {
        SeriesReportResponse {
            id: report.id,
            series_id: report.series_id,
            user_id: report.user_id,
            project_id: report.project_id,
            status: report.status,
            dictate_file_path: report.dictate_file_path,
            dictate_file_size: report.dictate_file_size,
            dictate_mime_type: report.dictate_mime_type,
            description: report.description,
            conclusion: report.conclusion,
            bodypart: report.bodypart,
            guides: None, // Controller에서 필요시 Guide Image 조회하여 포함
            created_at: report.created_at,
            updated_at: report.updated_at,
        }
    }

    fn to_response_with_user(
        &self,
        report: SeriesUserReport,
        user: &crate::domain::entities::User,
    ) -> SeriesReportWithUserResponse {
        SeriesReportWithUserResponse {
            id: report.id,
            series_id: report.series_id,
            user: SeriesReportUserInfo {
                id: user.id,
                username: user.username.clone(),
                email: user.email.clone(),
                full_name: user.full_name.clone(),
            },
            project_id: report.project_id,
            status: report.status,
            dictate_file_path: report.dictate_file_path,
            dictate_file_size: report.dictate_file_size,
            dictate_mime_type: report.dictate_mime_type,
            description: report.description,
            conclusion: report.conclusion,
            bodypart: report.bodypart,
            created_at: report.created_at,
            updated_at: report.updated_at,
        }
    }
}

