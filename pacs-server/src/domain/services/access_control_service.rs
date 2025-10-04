use async_trait::async_trait;
use crate::domain::entities::{AccessLog, NewAccessLog};
use crate::domain::repositories::{AccessLogRepository, UserRepository, ProjectRepository};
use super::user_service::ServiceError;

/// 접근 제어 및 로깅 도메인 서비스
/// DICOM 리소스 접근에 대한 로그를 관리합니다
#[async_trait]
pub trait AccessControlService: Send + Sync {
    /// DICOM 리소스 접근 로그 기록
    async fn log_dicom_access(
        &self,
        user_id: i32,
        project_id: Option<i32>,
        resource_type: String,
        study_uid: Option<String>,
        series_uid: Option<String>,
        instance_uid: Option<String>,
        action: String,
        result: String,
        ip_address: Option<String>,
        ae_title: Option<String>,
    ) -> Result<AccessLog, ServiceError>;

    /// 특정 사용자의 접근 로그 조회 (최근 N개)
    async fn get_user_access_logs(&self, user_id: i32, limit: i64) -> Result<Vec<AccessLog>, ServiceError>;

    /// 특정 프로젝트의 접근 로그 조회 (최근 N개)
    async fn get_project_access_logs(&self, project_id: i32, limit: i64) -> Result<Vec<AccessLog>, ServiceError>;

    /// 특정 Study의 접근 로그 조회
    async fn get_study_access_logs(&self, study_uid: &str, limit: i64) -> Result<Vec<AccessLog>, ServiceError>;

    /// 사용자의 접근 로그 개수 조회
    async fn count_user_access(&self, user_id: i32) -> Result<i64, ServiceError>;

    /// 사용자가 프로젝트에 접근 가능한지 확인
    async fn can_access_project(&self, user_id: i32, project_id: i32) -> Result<bool, ServiceError>;
}

pub struct AccessControlServiceImpl<A, U, P>
where
    A: AccessLogRepository,
    U: UserRepository,
    P: ProjectRepository,
{
    access_log_repository: A,
    user_repository: U,
    project_repository: P,
}

impl<A, U, P> AccessControlServiceImpl<A, U, P>
where
    A: AccessLogRepository,
    U: UserRepository,
    P: ProjectRepository,
{
    pub fn new(access_log_repository: A, user_repository: U, project_repository: P) -> Self {
        Self {
            access_log_repository,
            user_repository,
            project_repository,
        }
    }
}

#[async_trait]
impl<A, U, P> AccessControlService for AccessControlServiceImpl<A, U, P>
where
    A: AccessLogRepository,
    U: UserRepository,
    P: ProjectRepository,
{
    async fn log_dicom_access(
        &self,
        user_id: i32,
        project_id: Option<i32>,
        resource_type: String,
        study_uid: Option<String>,
        series_uid: Option<String>,
        instance_uid: Option<String>,
        action: String,
        result: String,
        ip_address: Option<String>,
        ae_title: Option<String>,
    ) -> Result<AccessLog, ServiceError> {
        // 사용자 존재 확인
        if self.user_repository.find_by_id(user_id).await?.is_none() {
            return Err(ServiceError::NotFound("User not found".into()));
        }

        // 프로젝트가 지정된 경우 존재 확인
        if let Some(pid) = project_id {
            if self.project_repository.find_by_id(pid).await?.is_none() {
                return Err(ServiceError::NotFound("Project not found".into()));
            }
        }

        // 리소스 타입 검증
        if resource_type.trim().is_empty() {
            return Err(ServiceError::ValidationError("Resource type cannot be empty".into()));
        }

        let new_log = NewAccessLog {
            user_id,
            project_id,
            resource_type,
            study_uid,
            series_uid,
            instance_uid,
            action,
            result,
            dicom_tag_check: None,
            ae_title,
            ip_address,
            session_id: None,
            via_group_id: None,
        };

        Ok(self.access_log_repository.create(new_log).await?)
    }

    async fn get_user_access_logs(&self, user_id: i32, limit: i64) -> Result<Vec<AccessLog>, ServiceError> {
        // 사용자 존재 확인
        if self.user_repository.find_by_id(user_id).await?.is_none() {
            return Err(ServiceError::NotFound("User not found".into()));
        }

        Ok(self.access_log_repository.find_by_user_id(user_id, limit).await?)
    }

    async fn get_project_access_logs(&self, project_id: i32, limit: i64) -> Result<Vec<AccessLog>, ServiceError> {
        // 프로젝트 존재 확인
        if self.project_repository.find_by_id(project_id).await?.is_none() {
            return Err(ServiceError::NotFound("Project not found".into()));
        }

        Ok(self.access_log_repository.find_by_project_id(project_id, limit).await?)
    }

    async fn get_study_access_logs(&self, study_uid: &str, limit: i64) -> Result<Vec<AccessLog>, ServiceError> {
        Ok(self.access_log_repository.find_by_study_uid(study_uid, limit).await?)
    }

    async fn count_user_access(&self, user_id: i32) -> Result<i64, ServiceError> {
        Ok(self.access_log_repository.count_by_user_id(user_id).await?)
    }

    async fn can_access_project(&self, user_id: i32, project_id: i32) -> Result<bool, ServiceError> {
        // 사용자 존재 확인
        if self.user_repository.find_by_id(user_id).await?.is_none() {
            return Err(ServiceError::NotFound("User not found".into()));
        }

        // 프로젝트 존재 확인
        let project = self.project_repository
            .find_by_id(project_id)
            .await?
            .ok_or(ServiceError::NotFound("Project not found".into()))?;

        // 프로젝트가 비활성화되어 있으면 접근 불가
        if !project.is_active {
            return Ok(false);
        }

        // TODO: 실제로는 security_user_project 테이블을 확인하여
        // 사용자가 프로젝트의 멤버인지 확인해야 함
        // 현재는 프로젝트가 활성화되어 있으면 접근 가능한 것으로 단순화

        Ok(true)
    }
}
