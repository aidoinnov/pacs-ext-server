use async_trait::async_trait;
use crate::domain::entities::{AccessLog, NewAccessLog, Permission};
use crate::domain::repositories::{AccessLogRepository, UserRepository, ProjectRepository, RoleRepository, PermissionRepository};
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

    // === 권한 검증 ===

    /// 사용자가 특정 권한을 가지고 있는지 확인 (프로젝트 컨텍스트)
    async fn check_permission(
        &self,
        user_id: i32,
        project_id: i32,
        resource_type: &str,
        action: &str,
    ) -> Result<bool, ServiceError>;

    /// 사용자가 가진 모든 권한 조회 (프로젝트 컨텍스트)
    async fn get_user_permissions(
        &self,
        user_id: i32,
        project_id: i32,
    ) -> Result<Vec<Permission>, ServiceError>;

    /// 사용자가 프로젝트의 멤버인지 확인
    async fn is_project_member(&self, user_id: i32, project_id: i32) -> Result<bool, ServiceError>;
}

pub struct AccessControlServiceImpl<A, U, P, R, PE>
where
    A: AccessLogRepository,
    U: UserRepository,
    P: ProjectRepository,
    R: RoleRepository,
    PE: PermissionRepository,
{
    access_log_repository: A,
    user_repository: U,
    project_repository: P,
    role_repository: R,
    permission_repository: PE,
}

impl<A, U, P, R, PE> AccessControlServiceImpl<A, U, P, R, PE>
where
    A: AccessLogRepository,
    U: UserRepository,
    P: ProjectRepository,
    R: RoleRepository,
    PE: PermissionRepository,
{
    pub fn new(
        access_log_repository: A,
        user_repository: U,
        project_repository: P,
        role_repository: R,
        permission_repository: PE,
    ) -> Self {
        Self {
            access_log_repository,
            user_repository,
            project_repository,
            role_repository,
            permission_repository,
        }
    }
}

#[async_trait]
impl<A, U, P, R, PE> AccessControlService for AccessControlServiceImpl<A, U, P, R, PE>
where
    A: AccessLogRepository,
    U: UserRepository,
    P: ProjectRepository,
    R: RoleRepository,
    PE: PermissionRepository,
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

        // security_user_project 테이블을 확인하여 사용자가 프로젝트의 멤버인지 확인
        self.is_project_member(user_id, project_id).await
    }

    // === 권한 검증 구현 ===

    async fn check_permission(
        &self,
        user_id: i32,
        project_id: i32,
        resource_type: &str,
        action: &str,
    ) -> Result<bool, ServiceError> {
        // 단일 쿼리로 통합 - 성능 향상 및 일관성 보장
        let has_permission = sqlx::query_scalar::<_, bool>(
            "WITH permission_id AS (
                SELECT id FROM security_permission
                WHERE resource_type = $3 AND action = $4
                LIMIT 1
            ),
            user_membership AS (
                SELECT 1 FROM security_user_project
                WHERE user_id = $1 AND project_id = $2
                LIMIT 1
            )
            SELECT EXISTS(
                SELECT 1 FROM user_membership
                WHERE EXISTS(
                    -- 역할 기반 권한
                    SELECT 1
                    FROM security_role_permission rp
                    INNER JOIN security_project_role pr ON rp.role_id = pr.role_id
                    INNER JOIN security_user_project up ON pr.project_id = up.project_id
                    WHERE up.user_id = $1
                      AND up.project_id = $2
                      AND rp.permission_id = (SELECT id FROM permission_id)

                    UNION ALL

                    -- 프로젝트 직접 권한
                    SELECT 1
                    FROM security_project_permission pp
                    INNER JOIN security_user_project up ON pp.project_id = up.project_id
                    WHERE up.user_id = $1
                      AND pp.project_id = $2
                      AND pp.permission_id = (SELECT id FROM permission_id)

                    LIMIT 1
                )
            )"
        )
        .bind(user_id)
        .bind(project_id)
        .bind(resource_type)
        .bind(action)
        .fetch_one(self.user_repository.pool())
        .await?;

        Ok(has_permission)
    }

    async fn get_user_permissions(
        &self,
        user_id: i32,
        project_id: i32,
    ) -> Result<Vec<Permission>, ServiceError> {
        // 사용자 존재 확인
        if self.user_repository.find_by_id(user_id).await?.is_none() {
            return Err(ServiceError::NotFound("User not found".into()));
        }

        // 프로젝트 존재 확인
        if self.project_repository.find_by_id(project_id).await?.is_none() {
            return Err(ServiceError::NotFound("Project not found".into()));
        }

        // 사용자가 프로젝트의 멤버인지 확인
        if !self.is_project_member(user_id, project_id).await? {
            return Err(ServiceError::Unauthorized("User is not a member of this project".into()));
        }

        // 사용자가 프로젝트에서 가진 모든 권한 조회 (역할을 통한 권한 + 프로젝트 직접 권한)
        let permissions = sqlx::query_as::<_, Permission>(
            "SELECT DISTINCT p.id, p.resource_type, p.action
             FROM security_permission p
             WHERE p.id IN (
                 -- 역할을 통한 권한
                 SELECT rp.permission_id FROM security_user_project up
                 INNER JOIN security_project_role pr ON up.project_id = pr.project_id
                 INNER JOIN security_role_permission rp ON pr.role_id = rp.role_id
                 WHERE up.user_id = $1 AND up.project_id = $2
                 UNION
                 -- 프로젝트 직접 권한
                 SELECT pp.permission_id FROM security_project_permission pp
                 INNER JOIN security_user_project up ON pp.project_id = up.project_id
                 WHERE up.user_id = $1 AND pp.project_id = $2
             )
             ORDER BY p.resource_type, p.action"
        )
        .bind(user_id)
        .bind(project_id)
        .fetch_all(self.user_repository.pool())
        .await?;

        Ok(permissions)
    }

    async fn is_project_member(&self, user_id: i32, project_id: i32) -> Result<bool, ServiceError> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM security_user_project WHERE user_id = $1 AND project_id = $2"
        )
        .bind(user_id)
        .bind(project_id)
        .fetch_one(self.user_repository.pool())
        .await?;

        Ok(count > 0)
    }
}
