use crate::application::dto::project_data_access_dto::*;
use crate::domain::entities::project_data::{
    DataAccessStatus, NewProjectData, ProjectDataInstance, ProjectDataSeries, ProjectDataStudy,
    UpdateProjectDataAccess,
};
use crate::domain::entities::subject::CreateSubject;
use crate::domain::services::{ProjectDataService, ProjectService, SubjectService};
use crate::domain::ServiceError;
use std::str::FromStr;
use std::sync::Arc;

pub struct ProjectDataAccessUseCase {
    project_data_service: Arc<dyn ProjectDataService>,
    project_service: Arc<dyn ProjectService>,
    subject_service: Arc<dyn SubjectService>,
}

impl ProjectDataAccessUseCase {
    pub fn new(
        project_data_service: Arc<dyn ProjectDataService>,
        project_service: Arc<dyn ProjectService>,
        subject_service: Arc<dyn SubjectService>,
    ) -> Self {
        Self {
            project_data_service,
            project_service,
            subject_service,
        }
    }

    /// 프로젝트 데이터 접근 매트릭스 조회
    pub async fn get_project_data_access_matrix(
        &self,
        project_id: i32,
        page: i32,
        page_size: i32,
        search: Option<String>,
        status: Option<String>,
        user_id: Option<i32>,
    ) -> Result<ProjectDataAccessMatrixResponse, ServiceError> {
        // Get project data list
        let project_data_list = if let Some(search_term) = search {
            self.project_data_service
                .search_project_data(project_id, &search_term, page, page_size)
                .await?
        } else {
            self.project_data_service
                .get_project_data_list(project_id, page, page_size)
                .await?
        };

        // Get access matrix
        let (_, access_list) = self
            .project_data_service
            .get_project_data_access_matrix(project_id, page, page_size)
            .await?;

        // Convert to DTOs
        let data_list: Vec<ProjectDataInfo> = project_data_list
            .into_iter()
            .map(|data| ProjectDataInfo {
                id: data.id,
                study_uid: data.study_uid,
                study_description: data.study_description,
                patient_id: data.patient_id,
                patient_name: data.patient_name,
                study_date: data.study_date.map(|d| d.to_string()),
                modality: data.modality,
            })
            .collect();

        // Get project members (users who belong to the project)
        println!("🔍 Fetching project members for project_id: {}", project_id);
        let project_members = match self
            .project_service
            .get_project_members(project_id)
            .await
        {
            Ok(members) => {
                println!("✅ Found {} project members", members.len());
                members
            }
            Err(e) => {
                println!("❌ Error fetching project members: {:?}", e);
                return Err(e);
            }
        };

        // Convert to UserInfo DTOs
        let users: Vec<UserInfo> = project_members
            .into_iter()
            .map(|user| UserInfo {
                id: user.id,
                username: user.username,
                email: user.email,
                full_name: None, // User entity doesn't have full_name field
                organization: None, // User entity doesn't have organization field
            })
            .collect();

        println!("📋 Converted {} users to UserInfo DTOs", users.len());

        // Convert access list to DTOs
        let access_matrix: Vec<DataAccessInfo> = access_list
            .into_iter()
            .map(|access| DataAccessInfo {
                project_data_id: access.project_data_id,
                user_id: access.user_id,
                status: access.status.to_string(),
                reviewed_at: access.reviewed_at.map(|t| t.to_rfc3339()),
                reviewed_by: access.reviewed_by,
            })
            .collect();

        // Calculate pagination
        let total_items = self
            .project_data_service
            .get_project_data_list(project_id, 1, 1000) // Get total count
            .await?
            .len() as i64;

        let total_pages = (total_items + page_size as i64 - 1) / page_size as i64;

        let pagination = PaginationInfo {
            page,
            page_size,
            total_items,
            total_pages,
        };

        Ok(ProjectDataAccessMatrixResponse {
            data_list,
            users,
            access_matrix,
            pagination,
        })
    }

    /// 프로젝트 데이터 생성
    pub async fn create_project_data(
        &self,
        project_id: i32,
        request: CreateProjectDataRequest,
    ) -> Result<CreateProjectDataResponse, ServiceError> {
        let mut new_data = NewProjectData::new(project_id, request.study_uid)
            .with_description(request.study_description.unwrap_or_default())
            .with_patient_info(
                request.patient_id.unwrap_or_default(),
                request.patient_name.unwrap_or_default(),
            )
            .with_modality(request.modality.unwrap_or_default());

        if let Some(study_date_str) = request.study_date {
            if let Ok(study_date) = chrono::NaiveDate::parse_from_str(&study_date_str, "%Y-%m-%d") {
                new_data = new_data.with_study_date(study_date);
            }
        }

        let project_data = self
            .project_data_service
            .create_project_data(new_data)
            .await?;

        Ok(CreateProjectDataResponse {
            success: true,
            message: "Project data created successfully".to_string(),
            data_id: Some(project_data.id),
        })
    }

    /// 개별 접근 권한 수정
    pub async fn update_data_access(
        &self,
        project_data_id: i32,
        user_id: i32,
        request: UpdateDataAccessRequest,
    ) -> Result<UpdateDataAccessResponse, ServiceError> {
        let status = DataAccessStatus::from_str(&request.status)
            .map_err(|e| ServiceError::ValidationError(e))?;

        let update_access = UpdateProjectDataAccess {
            status: Some(status),
            review_note: request.review_note,
            reviewed_at: Some(chrono::Utc::now()),
            reviewed_by: Some(1), // TODO: Get from current user context
            ..Default::default()
        };

        self.project_data_service
            .update_data_access(project_data_id, user_id, update_access)
            .await?;

        Ok(UpdateDataAccessResponse {
            success: true,
            message: "Data access updated successfully".to_string(),
        })
    }

    /// 일괄 접근 권한 수정
    pub async fn batch_update_data_access(
        &self,
        project_data_id: i32,
        request: BatchUpdateDataAccessRequest,
    ) -> Result<BatchUpdateDataAccessResponse, ServiceError> {
        let status = DataAccessStatus::from_str(&request.status)
            .map_err(|e| ServiceError::ValidationError(e))?;

        let update_access = UpdateProjectDataAccess {
            status: Some(status),
            review_note: request.review_note,
            reviewed_at: Some(chrono::Utc::now()),
            reviewed_by: Some(1), // TODO: Get from current user context
            ..Default::default()
        };

        let results = self
            .project_data_service
            .batch_update_data_access(project_data_id, request.user_ids, update_access)
            .await?;

        Ok(BatchUpdateDataAccessResponse {
            success: true,
            message: "Data access updated successfully".to_string(),
            updated_count: results.len() as i32,
        })
    }

    /// 접근 요청
    pub async fn request_data_access(
        &self,
        project_data_id: i32,
        user_id: i32,
    ) -> Result<RequestDataAccessResponse, ServiceError> {
        self.project_data_service
            .request_data_access(project_data_id, user_id, user_id) // TODO: Get from current user context
            .await?;

        Ok(RequestDataAccessResponse {
            success: true,
            message: "Data access requested successfully".to_string(),
        })
    }

    /// 프로젝트 참가 시 기본 접근 권한 부여
    pub async fn grant_default_access_to_user(
        &self,
        project_id: i32,
        user_id: i32,
    ) -> Result<(), ServiceError> {
        self.project_data_service
            .grant_default_access_to_user(project_id, user_id)
            .await?;

        Ok(())
    }

    /// 상태별 접근 권한 조회
    pub async fn get_access_by_status(
        &self,
        status: String,
        page: i32,
        page_size: i32,
    ) -> Result<Vec<DataAccessInfo>, ServiceError> {
        let data_access_status =
            DataAccessStatus::from_str(&status).map_err(|e| ServiceError::ValidationError(e))?;

        let access_list = self
            .project_data_service
            .get_access_by_status(data_access_status, page, page_size)
            .await?;

        let access_matrix: Vec<DataAccessInfo> = access_list
            .into_iter()
            .map(|access| DataAccessInfo {
                project_data_id: access.project_data_id,
                user_id: access.user_id,
                status: access.status.to_string(),
                reviewed_at: access.reviewed_at.map(|t| t.to_rfc3339()),
                reviewed_by: access.reviewed_by,
            })
            .collect();

        Ok(access_matrix)
    }

    /// 사용자별 접근 권한 조회
    pub async fn get_user_access_list(
        &self,
        user_id: i32,
        page: i32,
        page_size: i32,
    ) -> Result<Vec<DataAccessInfo>, ServiceError> {
        let access_list = self
            .project_data_service
            .get_user_access_list(user_id, page, page_size)
            .await?;

        let access_matrix: Vec<DataAccessInfo> = access_list
            .into_iter()
            .map(|access| DataAccessInfo {
                project_data_id: access.project_data_id,
                user_id: access.user_id,
                status: access.status.to_string(),
                reviewed_at: access.reviewed_at.map(|t| t.to_rfc3339()),
                reviewed_by: access.reviewed_by,
            })
            .collect();

        Ok(access_matrix)
    }

    // ========== 새로운 계층 구조 메서드 ==========

    /// Study 조회 (by ID)
    pub async fn get_study(&self, study_id: i32) -> Result<ProjectDataStudy, ServiceError> {
        self.project_data_service.get_study_by_id(study_id).await
    }

    /// Study 조회 (by project_id and study_uid)
    pub async fn get_study_by_uid(
        &self,
        project_id: i32,
        study_uid: String,
    ) -> Result<ProjectDataStudy, ServiceError> {
        self.project_data_service
            .get_study_by_uid(project_id, &study_uid)
            .await
    }

    /// 프로젝트별 Study 목록 조회 (페이지네이션)
    pub async fn get_studies(
        &self,
        project_id: i32,
        page: i32,
        page_size: i32,
    ) -> Result<(Vec<ProjectDataStudy>, i64), ServiceError> {
        self.project_data_service
            .get_studies_by_project(project_id, page, page_size)
            .await
    }

    /// Series 조회 (by ID)
    pub async fn get_series(&self, series_id: i32) -> Result<ProjectDataSeries, ServiceError> {
        self.project_data_service.get_series_by_id(series_id).await
    }

    /// Study별 Series 목록 조회
    pub async fn get_series_by_study(
        &self,
        study_id: i32,
    ) -> Result<Vec<ProjectDataSeries>, ServiceError> {
        self.project_data_service
            .get_series_by_study(study_id)
            .await
    }

    /// 프로젝트에 할당된 Series 목록 조회 (Study별)
    pub async fn get_series_by_project_and_study(
        &self,
        project_id: i32,
        study_id: i32,
    ) -> Result<Vec<ProjectDataSeries>, ServiceError> {
        self.project_data_service
            .get_series_by_project_and_study(project_id, study_id)
            .await
    }

    /// Instance 조회 (by ID)
    pub async fn get_instance(
        &self,
        instance_id: i32,
    ) -> Result<ProjectDataInstance, ServiceError> {
        self.project_data_service
            .get_instance_by_id(instance_id)
            .await
    }

    /// Series별 Instance 목록 조회
    pub async fn get_instances_by_series(
        &self,
        series_id: i32,
    ) -> Result<Vec<ProjectDataInstance>, ServiceError> {
        self.project_data_service
            .get_instances_by_series(series_id)
            .await
    }

    // ========== Series/Study 할당 API ==========

    /// 프로젝트에 Series 할당
    pub async fn assign_series_to_project(
        &self,
        project_id: i32,
        request: AssignSeriesToProjectRequest,
    ) -> Result<AssignSeriesToProjectResponse, ServiceError> {
        // 1. 프로젝트 존재 확인
        self.project_service.get_project(project_id).await?;

        // 2. Study 조회 또는 생성 (전역 엔티티만 생성, project_data에는 추가하지 않음)
        let pool = self.project_data_service.pool();
        let study: ProjectDataStudy = sqlx::query_as(
            "INSERT INTO project_data_study (study_uid, study_description)
             VALUES ($1, $2)
             ON CONFLICT (study_uid) DO UPDATE SET study_uid = EXCLUDED.study_uid
             RETURNING id, study_uid, study_description, patient_id, patient_name,
                       patient_birth_date, study_date, created_at, updated_at",
        )
        .bind(&request.study_uid)
        .bind(format!("Study for series {}", request.series_uid))
        .fetch_one(pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        // 3. Series 조회 또는 생성
        let pool = self.project_data_service.pool();
        let series: ProjectDataSeries = sqlx::query_as(
            "INSERT INTO project_data_series (study_id, series_uid, series_description, modality, series_number)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (study_id, series_uid) DO UPDATE
             SET series_description = EXCLUDED.series_description,
                 modality = EXCLUDED.modality,
                 series_number = EXCLUDED.series_number
             RETURNING id, study_id, series_uid, series_description, modality, series_number, created_at",
        )
        .bind(study.id)
        .bind(&request.series_uid)
        .bind(&request.series_description)
        .bind(&request.modality)
        .bind(request.series_number)
        .fetch_one(pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        // 4. 이미 할당되어 있는지 확인
        let already_assigned: bool = sqlx::query_scalar(
            "SELECT EXISTS(
                SELECT 1 FROM project_data
                WHERE project_id = $1
                  AND resource_level = 'SERIES'::resource_level_enum
                  AND series_id = $2
            )",
        )
        .bind(project_id)
        .bind(series.id)
        .fetch_one(pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        if already_assigned {
            return Err(ServiceError::AlreadyExists(
                "Series already assigned to this project".to_string(),
            ));
        }

        // 5. project_data에 Series 매핑 추가
        sqlx::query(
            "INSERT INTO project_data (project_id, resource_level, study_id, series_id)
             VALUES ($1, 'SERIES'::resource_level_enum, $2, $3)",
        )
        .bind(project_id)
        .bind(study.id)
        .bind(series.id)
        .execute(pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        Ok(AssignSeriesToProjectResponse {
            success: true,
            message: format!("Series {} assigned to project successfully", request.series_uid),
            series_id: series.id,
        })
    }

    /// 프로젝트에 Study 할당
    pub async fn assign_study_to_project(
        &self,
        project_id: i32,
        request: AssignStudyToProjectRequest,
    ) -> Result<AssignStudyToProjectResponse, ServiceError> {
        // 1. 프로젝트 존재 확인
        self.project_service.get_project(project_id).await?;

        // 2. Study 조회 (DICOM 메타데이터는 이미 DB에 있어야 함)
        let pool = self.project_data_service.pool();
        let study: ProjectDataStudy = sqlx::query_as(
            "SELECT id, study_uid, study_description, patient_id, patient_name,
                    patient_birth_date, study_date, created_at, updated_at
             FROM project_data_study
             WHERE study_uid = $1",
        )
        .bind(&request.study_uid)
        .fetch_one(pool)
        .await
        .map_err(|e| ServiceError::NotFound(format!("Study not found: {}", e)))?;

        // 3. 이미 할당되어 있는지 확인
        let already_assigned: bool = sqlx::query_scalar(
            "SELECT EXISTS(
                SELECT 1 FROM project_data
                WHERE project_id = $1
                  AND resource_level = 'STUDY'::resource_level_enum
                  AND study_id = $2
            )",
        )
        .bind(project_id)
        .bind(study.id)
        .fetch_one(pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        if already_assigned {
            return Err(ServiceError::AlreadyExists(
                "Study already assigned to this project".to_string(),
            ));
        }

        // 4. project_data에 Study 매핑 추가
        sqlx::query(
            "INSERT INTO project_data (project_id, resource_level, study_id)
             VALUES ($1, 'STUDY'::resource_level_enum, $2)",
        )
        .bind(project_id)
        .bind(study.id)
        .execute(pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        // 5. Subject 자동 생성 (Patient ID가 있는 경우)
        if let Some(ref patient_id) = study.patient_id {
            // Patient ID로 기존 Subject 찾기
            let existing_subject: Option<(i32,)> = sqlx::query_as(
                "SELECT id FROM project_subject
                 WHERE project_id = $1 AND patient_id = $2",
            )
            .bind(project_id)
            .bind(patient_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

            // Subject가 없으면 자동 생성
            if existing_subject.is_none() {
                // Subject Code 결정: 사용자 지정 > Patient ID 기반 자동 생성
                let subject_code = if let Some(ref code) = request.subject_code {
                    // 사용자가 지정한 코드 사용 (중복 체크는 SubjectService에서 수행)
                    code.clone()
                } else {
                    // Patient ID 기반 자동 생성
                    self.generate_unique_subject_code(project_id, patient_id).await?
                };

                let new_subject = CreateSubject {
                    project_id,
                    subject_code,
                    patient_id: Some(patient_id.clone()),
                    patient_name: study.patient_name.clone(),
                    patient_birth_date: study.patient_birth_date,
                };

                // Subject 생성 (에러 무시 - 동시성 이슈로 이미 생성되었을 수 있음)
                let _ = self.subject_service.create_subject(new_subject).await;
            }
        }

        Ok(AssignStudyToProjectResponse {
            success: true,
            message: format!("Study {} assigned to project successfully", request.study_uid),
            study_id: study.id,
        })
    }

    /// 유일한 Subject Code 생성
    ///
    /// Patient ID 기반으로 Subject Code를 생성하되, 중복 시 suffix를 추가합니다.
    /// Patient ID가 너무 길거나 유효하지 않으면 순차 코드를 사용합니다 (A-001, A-002, ..., A-999, B-001, ...).
    async fn generate_unique_subject_code(
        &self,
        project_id: i32,
        patient_id: &str,
    ) -> Result<String, ServiceError> {
        let pool = self.project_data_service.pool();

        // Patient ID를 Subject Code로 사용 가능한지 검증 (1-50자, 영문자/숫자/하이픈/언더스코어)
        let sanitized_patient_id = patient_id
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
            .take(50)
            .collect::<String>();

        if !sanitized_patient_id.is_empty() {
            // Patient ID 기반 Subject Code 시도
            let mut candidate = sanitized_patient_id.clone();
            let mut suffix = 0;

            loop {
                // 중복 체크
                let exists: bool = sqlx::query_scalar(
                    "SELECT EXISTS(SELECT 1 FROM project_subject WHERE project_id = $1 AND subject_code = $2)",
                )
                .bind(project_id)
                .bind(&candidate)
                .fetch_one(pool)
                .await
                .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

                if !exists {
                    return Ok(candidate);
                }

                // 중복이면 suffix 추가
                suffix += 1;
                candidate = format!("{}_{}", sanitized_patient_id, suffix);

                // 무한 루프 방지 (최대 100번 시도)
                if suffix > 100 {
                    break;
                }
            }
        }

        // Patient ID 기반 생성 실패 시 순차 코드 사용 (A-001, A-002, ..., A-999, B-001, ...)
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM project_subject WHERE project_id = $1",
        )
        .bind(project_id)
        .fetch_one(pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        let mut offset = 0;

        // 순차 번호도 중복 체크 (동시성 이슈 대비)
        loop {
            // A-001 ~ A-999, B-001 ~ B-999, ...
            let total_num = count + offset;
            let prefix = char::from_u32(65 + (total_num / 999) as u32)
                .unwrap_or('A');
            let number = (total_num % 999) + 1;
            let candidate = format!("{}-{:03}", prefix, number);

            let exists: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM project_subject WHERE project_id = $1 AND subject_code = $2)",
            )
            .bind(project_id)
            .bind(&candidate)
            .fetch_one(pool)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

            if !exists {
                return Ok(candidate);
            }

            offset += 1;

            // 무한 루프 방지
            if offset > 10000 {
                return Err(ServiceError::DatabaseError(
                    "Failed to generate unique subject code".to_string(),
                ));
            }
        }
    }

    /// 프로젝트에서 Series 할당 해제
    pub async fn unassign_series_from_project(
        &self,
        project_id: i32,
        series_id: i32,
    ) -> Result<UnassignSeriesFromProjectResponse, ServiceError> {
        // 1. 프로젝트 존재 확인
        self.project_service.get_project(project_id).await?;

        // 2. project_data에서 Series 매핑 삭제
        let pool = self.project_data_service.pool();
        let result = sqlx::query(
            "DELETE FROM project_data
             WHERE project_id = $1
               AND resource_level = 'SERIES'::resource_level_enum
               AND series_id = $2",
        )
        .bind(project_id)
        .bind(series_id)
        .execute(pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(ServiceError::NotFound(
                "Series not assigned to this project".to_string(),
            ));
        }

        Ok(UnassignSeriesFromProjectResponse {
            success: true,
            message: format!("Series {} unassigned from project successfully", series_id),
        })
    }

    /// 프로젝트에서 Study 할당 해제
    pub async fn unassign_study_from_project(
        &self,
        project_id: i32,
        study_id: i32,
    ) -> Result<UnassignStudyFromProjectResponse, ServiceError> {
        // 1. 프로젝트 존재 확인
        self.project_service.get_project(project_id).await?;

        // 2. project_data에서 Study 매핑 삭제
        let pool = self.project_data_service.pool();
        let result = sqlx::query(
            "DELETE FROM project_data
             WHERE project_id = $1
               AND resource_level = 'STUDY'::resource_level_enum
               AND study_id = $2",
        )
        .bind(project_id)
        .bind(study_id)
        .execute(pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(ServiceError::NotFound(
                "Study not assigned to this project".to_string(),
            ));
        }

        Ok(UnassignStudyFromProjectResponse {
            success: true,
            message: format!("Study {} unassigned from project successfully", study_id),
        })
    }
}
