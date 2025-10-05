use crate::application::dto::{
    LogDicomAccessRequest, AccessLogResponse, AccessLogListResponse, CheckPermissionRequest,
    CheckPermissionResponse, UserPermissionsResponse, PermissionInfo, ProjectAccessResponse,
};
use crate::domain::services::{AccessControlService, ServiceError};

/// 접근 제어 유스케이스
pub struct AccessControlUseCase<A: AccessControlService> {
    access_control_service: A,
}

impl<A: AccessControlService> AccessControlUseCase<A> {
    pub fn new(access_control_service: A) -> Self {
        Self {
            access_control_service,
        }
    }

    /// DICOM 접근 로그 기록
    pub async fn log_dicom_access(&self, request: LogDicomAccessRequest) -> Result<AccessLogResponse, ServiceError> {
        let log = self
            .access_control_service
            .log_dicom_access(
                request.user_id,
                request.project_id,
                request.resource_type,
                request.study_uid,
                request.series_uid,
                request.instance_uid,
                request.action,
                request.result,
                request.ip_address,
                request.ae_title,
            )
            .await?;

        Ok(AccessLogResponse {
            id: log.id,
            user_id: log.user_id,
            project_id: log.project_id,
            resource_type: log.resource_type,
            study_uid: log.study_uid,
            series_uid: log.series_uid,
            instance_uid: log.instance_uid,
            action: log.action,
            result: log.result,
            logged_at: log.logged_at,
        })
    }

    /// 사용자 접근 로그 조회
    pub async fn get_user_access_logs(
        &self,
        user_id: i32,
        limit: i64,
    ) -> Result<AccessLogListResponse, ServiceError> {
        let logs = self
            .access_control_service
            .get_user_access_logs(user_id, limit)
            .await?;

        let total = logs.len();

        let log_responses = logs
            .into_iter()
            .map(|l| AccessLogResponse {
                id: l.id,
                user_id: l.user_id,
                project_id: l.project_id,
                resource_type: l.resource_type,
                study_uid: l.study_uid,
                series_uid: l.series_uid,
                instance_uid: l.instance_uid,
                action: l.action,
                result: l.result,
                logged_at: l.logged_at,
            })
            .collect();

        Ok(AccessLogListResponse {
            logs: log_responses,
            total,
        })
    }

    /// 프로젝트 접근 로그 조회
    pub async fn get_project_access_logs(
        &self,
        project_id: i32,
        limit: i64,
    ) -> Result<AccessLogListResponse, ServiceError> {
        let logs = self
            .access_control_service
            .get_project_access_logs(project_id, limit)
            .await?;

        let total = logs.len();

        let log_responses = logs
            .into_iter()
            .map(|l| AccessLogResponse {
                id: l.id,
                user_id: l.user_id,
                project_id: l.project_id,
                resource_type: l.resource_type,
                study_uid: l.study_uid,
                series_uid: l.series_uid,
                instance_uid: l.instance_uid,
                action: l.action,
                result: l.result,
                logged_at: l.logged_at,
            })
            .collect();

        Ok(AccessLogListResponse {
            logs: log_responses,
            total,
        })
    }

    /// Study 접근 로그 조회
    pub async fn get_study_access_logs(
        &self,
        study_uid: &str,
        limit: i64,
    ) -> Result<AccessLogListResponse, ServiceError> {
        let logs = self
            .access_control_service
            .get_study_access_logs(study_uid, limit)
            .await?;

        let total = logs.len();

        let log_responses = logs
            .into_iter()
            .map(|l| AccessLogResponse {
                id: l.id,
                user_id: l.user_id,
                project_id: l.project_id,
                resource_type: l.resource_type,
                study_uid: l.study_uid,
                series_uid: l.series_uid,
                instance_uid: l.instance_uid,
                action: l.action,
                result: l.result,
                logged_at: l.logged_at,
            })
            .collect();

        Ok(AccessLogListResponse {
            logs: log_responses,
            total,
        })
    }

    /// 권한 검증
    pub async fn check_permission(&self, request: CheckPermissionRequest) -> Result<CheckPermissionResponse, ServiceError> {
        let has_permission = self
            .access_control_service
            .check_permission(
                request.user_id,
                request.project_id,
                &request.resource_type,
                &request.action,
            )
            .await?;

        Ok(CheckPermissionResponse {
            user_id: request.user_id,
            project_id: request.project_id,
            resource_type: request.resource_type,
            action: request.action,
            has_permission,
        })
    }

    /// 사용자 권한 목록 조회
    pub async fn get_user_permissions(
        &self,
        user_id: i32,
        project_id: i32,
    ) -> Result<UserPermissionsResponse, ServiceError> {
        let permissions = self
            .access_control_service
            .get_user_permissions(user_id, project_id)
            .await?;

        let permission_infos = permissions
            .into_iter()
            .map(|p| PermissionInfo {
                id: p.id,
                resource_type: p.resource_type,
                action: p.action,
            })
            .collect();

        Ok(UserPermissionsResponse {
            user_id,
            project_id,
            permissions: permission_infos,
        })
    }

    /// 프로젝트 접근 가능 여부 확인
    pub async fn can_access_project(&self, user_id: i32, project_id: i32) -> Result<ProjectAccessResponse, ServiceError> {
        let can_access = self
            .access_control_service
            .can_access_project(user_id, project_id)
            .await?;

        let is_member = self
            .access_control_service
            .is_project_member(user_id, project_id)
            .await?;

        Ok(ProjectAccessResponse {
            user_id,
            project_id,
            can_access,
            is_member,
        })
    }
}
