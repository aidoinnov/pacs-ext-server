use std::sync::Arc;

use crate::domain::repositories::ProjectRepository;
use crate::domain::services::dicom_rbac_evaluator::DicomRbacEvaluator;
use crate::infrastructure::repositories::{ProjectDataRepositoryImpl, ProjectRepositoryImpl};

/// 프로젝트별 접근 정보
#[derive(Debug, Clone)]
pub struct ProjectAccessResult {
    pub project_id: i32,
    pub project_name: String,
    pub access_level: String, // "STUDY", "SERIES"
    pub reason: String,       // "approved", "member", "denied"
}

/// 데이터 접근 확인 결과
#[derive(Debug)]
pub struct DataAccessCheckResult {
    pub projects: Vec<ProjectAccessResult>,
}

/// 데이터 접근 권한 확인 UseCase
pub struct DataAccessCheckUseCase {
    project_repository: ProjectRepositoryImpl,
    rbac_evaluator: Arc<dyn DicomRbacEvaluator>,
    project_data_repository: Arc<ProjectDataRepositoryImpl>,
}

impl DataAccessCheckUseCase {
    pub fn new(
        project_repository: ProjectRepositoryImpl,
        rbac_evaluator: Arc<dyn DicomRbacEvaluator>,
        project_data_repository: Arc<ProjectDataRepositoryImpl>,
    ) -> Self {
        Self {
            project_repository,
            rbac_evaluator,
            project_data_repository,
        }
    }

    /// 사용자가 특정 Study/Series에 접근 가능한지 확인
    ///
    /// # Arguments
    /// * `user_id` - 사용자 ID
    /// * `study_uid` - Study UID
    /// * `series_uid` - Series UID (선택)
    /// * `project_id` - Project ID (선택) - 특정 프로젝트에 대한 접근 권한만 확인
    ///
    /// # Returns
    /// 접근 가능한 프로젝트 목록
    pub async fn check_access(
        &self,
        user_id: i32,
        study_uid: &str,
        series_uid: Option<&str>,
        project_id: Option<i32>,
    ) -> Result<DataAccessCheckResult, Box<dyn std::error::Error>> {
        tracing::info!(
            "Checking data access for user {} on study {} (series: {:?}, project: {:?})",
            user_id,
            study_uid,
            series_uid,
            project_id
        );

        // 1. 사용자가 속한 프로젝트 조회
        let user_projects: Vec<i32> = if let Some(pid) = project_id {
            // 특정 프로젝트만 확인
            let is_member: Option<i32> = sqlx::query_scalar(
                "SELECT project_id FROM security_user_project WHERE user_id = $1 AND project_id = $2"
            )
            .bind(user_id)
            .bind(pid)
            .fetch_optional(self.project_repository.pool())
            .await
            .map_err(|e| format!("Failed to check project membership: {}", e))?;

            if let Some(p) = is_member {
                vec![p]
            } else {
                tracing::warn!("User {} is not a member of project {}", user_id, pid);
                return Ok(DataAccessCheckResult {
                    projects: Vec::new(),
                });
            }
        } else {
            // 모든 프로젝트 조회
            sqlx::query_scalar(
                "SELECT project_id FROM security_user_project WHERE user_id = $1"
            )
            .bind(user_id)
            .fetch_all(self.project_repository.pool())
            .await
            .map_err(|e| format!("Failed to get user projects: {}", e))?
        };

        if user_projects.is_empty() {
            tracing::warn!("User {} has no projects", user_id);
            return Ok(DataAccessCheckResult {
                projects: Vec::new(),
            });
        }

        tracing::debug!(
            "User {} belongs to {} projects",
            user_id,
            user_projects.len()
        );

        // 2. 각 프로젝트에서 접근 권한 확인
        let mut accessible_projects = Vec::new();

        for project_id in user_projects {
            // 2.1 프로젝트 정보 조회
            let project = match self.project_repository.find_by_id(project_id).await {
                Ok(Some(p)) => p,
                Ok(None) => {
                    tracing::warn!("Project {} not found", project_id);
                    continue;
                }
                Err(e) => {
                    tracing::error!("Error fetching project {}: {:?}", project_id, e);
                    continue;
                }
            };

            // 2.2 프로젝트가 비활성화되어 있으면 스킵
            if !project.is_active {
                tracing::debug!("Project {} is inactive, skipping", project_id);
                continue;
            }

            // 2.3 Study 접근 권한 확인
            let study_access = self.check_study_access(user_id, project_id, study_uid).await;

            if !study_access {
                tracing::debug!(
                    "User {} does not have access to study {} in project {}",
                    user_id,
                    study_uid,
                    project_id
                );
                continue;
            }

            // 2.4 Series 접근 권한 확인 (series_uid가 있는 경우)
            let access_level = if let Some(series) = series_uid {
                let series_access = self
                    .check_series_access(user_id, project_id, study_uid, series)
                    .await;

                if series_access {
                    "SERIES".to_string()
                } else {
                    tracing::debug!(
                        "User {} does not have access to series {} in project {}",
                        user_id,
                        series,
                        project_id
                    );
                    continue;
                }
            } else {
                "STUDY".to_string()
            };

            // 2.5 접근 가능한 프로젝트 추가
            accessible_projects.push(ProjectAccessResult {
                project_id,
                project_name: project.name,
                access_level,
                reason: "approved".to_string(),
            });
        }

        tracing::info!(
            "User {} has access to study {} in {} projects",
            user_id,
            study_uid,
            accessible_projects.len()
        );

        Ok(DataAccessCheckResult {
            projects: accessible_projects,
        })
    }

    /// Study 접근 권한 확인
    async fn check_study_access(
        &self,
        user_id: i32,
        project_id: i32,
        study_uid: &str,
    ) -> bool {
        // 1. RBAC 평가
        let rbac_result = self
            .rbac_evaluator
            .evaluate_study_uid(user_id, project_id, study_uid)
            .await;

        if !rbac_result.allowed {
            tracing::debug!(
                "RBAC denied access for user {} to study {} in project {}: {:?}",
                user_id,
                study_uid,
                project_id,
                rbac_result.reason
            );
            return false;
        }

        // 2. project_data_access 확인
        let data_access = self
            .check_project_data_access(user_id, project_id, study_uid)
            .await;

        if !data_access {
            tracing::debug!(
                "project_data_access denied for user {} to study {} in project {}",
                user_id,
                study_uid,
                project_id
            );
            return false;
        }

        true
    }

    /// Series 접근 권한 확인
    async fn check_series_access(
        &self,
        user_id: i32,
        project_id: i32,
        study_uid: &str,
        series_uid: &str,
    ) -> bool {
        // 1. Study 접근 권한 먼저 확인
        if !self.check_study_access(user_id, project_id, study_uid).await {
            return false;
        }

        // 2. Series ID 조회
        let series_id = match self.get_series_id(study_uid, series_uid, project_id).await {
            Some(id) => id,
            None => {
                tracing::warn!(
                    "Series {} not found in study {} for project {}",
                    series_uid,
                    study_uid,
                    project_id
                );
                return false;
            }
        };

        // 3. RBAC 평가 (Series 레벨)
        let rbac_result = self
            .rbac_evaluator
            .evaluate_series_access(user_id, project_id, series_id)
            .await;

        rbac_result.allowed
    }

    /// project_data_access 테이블 확인
    async fn check_project_data_access(
        &self,
        user_id: i32,
        project_id: i32,
        study_uid: &str,
    ) -> bool {
        use crate::presentation::controllers::dicom_gateway_controller::can_access_study;

        can_access_study(
            user_id,
            project_id,
            study_uid,
            self.project_data_repository.pool(),
        )
        .await
    }

    /// Series ID 조회
    async fn get_series_id(
        &self,
        study_uid: &str,
        series_uid: &str,
        project_id: i32,
    ) -> Option<i32> {
        let result: Option<i32> = sqlx::query_scalar(
            "SELECT pds.id
             FROM project_data_series pds
             INNER JOIN project_data_study pdt ON pds.study_id = pdt.id
             INNER JOIN project_data pd ON pd.study_id = pdt.id
             WHERE pdt.study_uid = $1
             AND pds.series_uid = $2
             AND pd.project_id = $3
             LIMIT 1",
        )
        .bind(study_uid)
        .bind(series_uid)
        .bind(project_id)
        .fetch_optional(self.project_data_repository.pool())
        .await
        .ok()
        .flatten();

        result
    }
}

