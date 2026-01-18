use crate::domain::entities::{
    AssignStudies, AssignmentResult, CreateTimePoint, StudyInfo, TimePoint, TimePointStudies,
    UnassignStudies, UpdateTimePoint, VisitType,
};
use crate::domain::repositories::{
    ProjectDataRepository, SubjectRepository, TimePointRepository, TimePointStudyRepository,
};
use crate::domain::ServiceError;
use async_trait::async_trait;
use std::sync::Arc;

/// TimePoint 관리 도메인 서비스
///
/// 이 트레이트는 TimePoint와 관련된 비즈니스 로직을 정의합니다.
#[async_trait]
pub trait TimePointService: Send + Sync {
    /// TimePoint를 생성합니다.
    ///
    /// # 비즈니스 규칙
    /// - Subject가 존재해야 함
    /// - Baseline TimePoint는 Subject당 하나만 존재해야 함
    /// - TimePoint 이름은 Subject 내에서 유일해야 함
    ///
    /// # 매개변수
    /// - `new_timepoint`: 생성할 TimePoint 정보
    ///
    /// # 반환값
    /// - `Ok(TimePoint)`: 생성된 TimePoint
    /// - `Err(ServiceError)`: 비즈니스 규칙 위반 또는 데이터베이스 오류
    async fn create_timepoint(
        &self,
        new_timepoint: CreateTimePoint,
    ) -> Result<TimePoint, ServiceError>;

    /// TimePoint를 조회합니다.
    ///
    /// # 매개변수
    /// - `id`: TimePoint ID
    ///
    /// # 반환값
    /// - `Ok(TimePoint)`: 조회된 TimePoint
    /// - `Err(ServiceError::NotFound)`: TimePoint가 존재하지 않음
    async fn get_timepoint(&self, id: i32) -> Result<TimePoint, ServiceError>;

    /// Subject의 모든 TimePoint를 조회합니다.
    ///
    /// # 매개변수
    /// - `subject_id`: Subject ID
    ///
    /// # 반환값
    /// - `Ok(Vec<TimePoint>)`: TimePoint 목록 (order_index 순)
    async fn get_timepoints_by_subject(
        &self,
        subject_id: i32,
    ) -> Result<Vec<TimePoint>, ServiceError>;

    /// TimePoint를 수정합니다.
    ///
    /// # 비즈니스 규칙
    /// - TimePoint가 존재해야 함
    /// - Baseline으로 변경 시 Subject에 다른 Baseline이 없어야 함
    /// - TimePoint 이름 변경 시 Subject 내에서 유일해야 함
    ///
    /// # 매개변수
    /// - `id`: TimePoint ID
    /// - `update_timepoint`: 수정할 TimePoint 정보
    ///
    /// # 반환값
    /// - `Ok(TimePoint)`: 수정된 TimePoint
    /// - `Err(ServiceError)`: 비즈니스 규칙 위반 또는 데이터베이스 오류
    async fn update_timepoint(
        &self,
        id: i32,
        update_timepoint: UpdateTimePoint,
    ) -> Result<TimePoint, ServiceError>;

    /// TimePoint를 삭제합니다.
    ///
    /// # 비즈니스 규칙
    /// - TimePoint가 존재해야 함
    /// - TimePoint에 할당된 Study가 있으면 삭제 불가
    ///
    /// # 매개변수
    /// - `id`: TimePoint ID
    ///
    /// # 반환값
    /// - `Ok(())`: 삭제 성공
    /// - `Err(ServiceError)`: 비즈니스 규칙 위반 또는 데이터베이스 오류
    async fn delete_timepoint(&self, id: i32) -> Result<(), ServiceError>;

    /// TimePoint에 Study를 할당합니다.
    ///
    /// # 비즈니스 규칙
    /// - TimePoint가 존재해야 함
    /// - Study는 같은 프로젝트에 속해야 함
    /// - Study는 다른 TimePoint에 할당되어 있으면 이동됨 (MOVE 시맨틱)
    ///
    /// # 매개변수
    /// - `timepoint_id`: TimePoint ID
    /// - `assign_studies`: 할당할 Study 정보
    /// - `user_id`: 할당하는 사용자 ID
    ///
    /// # 반환값
    /// - `Ok(AssignmentResult)`: 할당 결과
    async fn assign_studies(
        &self,
        timepoint_id: i32,
        assign_studies: AssignStudies,
        user_id: i32,
    ) -> Result<AssignmentResult, ServiceError>;

    /// TimePoint에서 Study를 해제합니다.
    ///
    /// # 매개변수
    /// - `timepoint_id`: TimePoint ID
    /// - `unassign_studies`: 해제할 Study 정보
    ///
    /// # 반환값
    /// - `Ok(i64)`: 해제된 Study 개수
    async fn unassign_studies(
        &self,
        timepoint_id: i32,
        unassign_studies: UnassignStudies,
    ) -> Result<i64, ServiceError>;

    /// TimePoint에 할당된 Study 목록을 조회합니다.
    ///
    /// # 매개변수
    /// - `timepoint_id`: TimePoint ID
    ///
    /// # 반환값
    /// - `Ok(TimePointStudies)`: Study 목록
    async fn get_studies_by_timepoint(
        &self,
        timepoint_id: i32,
    ) -> Result<TimePointStudies, ServiceError>;

    /// Subject의 미할당 Study 목록을 조회합니다.
    ///
    /// # 매개변수
    /// - `subject_id`: Subject ID
    ///
    /// # 반환값
    /// - `Ok(Vec<StudyInfo>)`: 미할당 Study 목록
    async fn get_unassigned_studies_by_subject(
        &self,
        subject_id: i32,
    ) -> Result<Vec<StudyInfo>, ServiceError>;
}

/// TimePoint 서비스 구현체
#[derive(Clone)]
pub struct TimePointServiceImpl<T, TS, S, PD>
where
    T: TimePointRepository,
    TS: TimePointStudyRepository,
    S: SubjectRepository,
    PD: ProjectDataRepository,
{
    timepoint_repository: T,
    timepoint_study_repository: TS,
    subject_repository: S,
    project_data_repository: Arc<PD>,
}

impl<T, TS, S, PD> TimePointServiceImpl<T, TS, S, PD>
where
    T: TimePointRepository,
    TS: TimePointStudyRepository,
    S: SubjectRepository,
    PD: ProjectDataRepository,
{
    pub fn new(
        timepoint_repository: T,
        timepoint_study_repository: TS,
        subject_repository: S,
        project_data_repository: Arc<PD>,
    ) -> Self {
        Self {
            timepoint_repository,
            timepoint_study_repository,
            subject_repository,
            project_data_repository,
        }
    }
}

#[async_trait]
impl<T, TS, S, PD> TimePointService for TimePointServiceImpl<T, TS, S, PD>
where
    T: TimePointRepository,
    TS: TimePointStudyRepository,
    S: SubjectRepository,
    PD: ProjectDataRepository,
{
    async fn create_timepoint(
        &self,
        new_timepoint: CreateTimePoint,
    ) -> Result<TimePoint, ServiceError> {
        // 1. Subject 존재 확인
        self.subject_repository
            .find_by_id(new_timepoint.subject_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Subject not found".into()))?;

        // 2. Baseline 중복 체크
        if new_timepoint.visit_type == VisitType::Baseline {
            if let Some(_) = self
                .timepoint_repository
                .find_baseline_by_subject(new_timepoint.subject_id)
                .await?
            {
                return Err(ServiceError::AlreadyExists(
                    "Baseline timepoint already exists for this subject".into(),
                ));
            }
        }

        // 3. TimePoint 이름 중복 체크
        if let Some(_) = self
            .timepoint_repository
            .find_by_name(new_timepoint.subject_id, &new_timepoint.name)
            .await?
        {
            return Err(ServiceError::AlreadyExists(format!(
                "TimePoint name '{}' already exists for this subject",
                new_timepoint.name
            )));
        }

        // 4. TimePoint 생성
        Ok(self.timepoint_repository.create(new_timepoint).await?)
    }

    async fn get_timepoint(&self, id: i32) -> Result<TimePoint, ServiceError> {
        self.timepoint_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("TimePoint not found".into()))
    }

    async fn get_timepoints_by_subject(
        &self,
        subject_id: i32,
    ) -> Result<Vec<TimePoint>, ServiceError> {
        // Subject 존재 확인
        self.subject_repository
            .find_by_id(subject_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Subject not found".into()))?;

        Ok(self
            .timepoint_repository
            .find_by_subject(subject_id)
            .await?)
    }

    async fn update_timepoint(
        &self,
        id: i32,
        update_timepoint: UpdateTimePoint,
    ) -> Result<TimePoint, ServiceError> {
        // 1. TimePoint 존재 확인
        let existing = self
            .timepoint_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("TimePoint not found".into()))?;

        // 2. Baseline으로 변경 시 중복 체크
        if let Some(ref new_visit_type) = update_timepoint.visit_type {
            if *new_visit_type == VisitType::Baseline
                && existing.visit_type != VisitType::Baseline
            {
                if let Some(_) = self
                    .timepoint_repository
                    .find_baseline_by_subject(existing.subject_id)
                    .await?
                {
                    return Err(ServiceError::AlreadyExists(
                        "Baseline timepoint already exists for this subject".into(),
                    ));
                }
            }
        }

        // 3. TimePoint 이름 변경 시 중복 체크
        if let Some(ref new_name) = update_timepoint.name {
            if new_name != &existing.name {
                if let Some(_) = self
                    .timepoint_repository
                    .find_by_name(existing.subject_id, new_name)
                    .await?
                {
                    return Err(ServiceError::AlreadyExists(format!(
                        "TimePoint name '{}' already exists for this subject",
                        new_name
                    )));
                }
            }
        }

        // 4. TimePoint 수정
        self.timepoint_repository
            .update(id, update_timepoint)
            .await?
            .ok_or_else(|| ServiceError::NotFound("TimePoint not found".into()))
    }

    async fn delete_timepoint(&self, id: i32) -> Result<(), ServiceError> {
        // 1. TimePoint 존재 확인
        let timepoint = self
            .timepoint_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("TimePoint not found".into()))?;

        // 2. 할당된 Study 개수 확인
        let study_count = self
            .timepoint_study_repository
            .count_studies_by_timepoint(id)
            .await?;

        if study_count > 0 {
            return Err(ServiceError::ValidationError(format!(
                "Cannot delete timepoint with {} assigned study(ies). Unassign studies first.",
                study_count
            )));
        }

        // 3. TimePoint 삭제
        let deleted = self.timepoint_repository.delete(id).await?;
        if !deleted {
            return Err(ServiceError::NotFound("TimePoint not found".into()));
        }

        Ok(())
    }

    async fn assign_studies(
        &self,
        timepoint_id: i32,
        assign_studies: AssignStudies,
        user_id: i32,
    ) -> Result<AssignmentResult, ServiceError> {
        // 1. TimePoint 존재 확인 및 Subject 조회
        let timepoint = self
            .timepoint_repository
            .find_by_id(timepoint_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("TimePoint not found".into()))?;

        // 2. Subject 조회 (project_id 필요)
        let subject = self
            .subject_repository
            .find_by_id(timepoint.subject_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Subject not found".into()))?;

        // 3. study_instance_uids를 study_ids로 변환 (필요한 경우)
        let study_ids = if let Some(ids) = assign_studies.study_ids {
            // study_ids가 제공된 경우 그대로 사용
            ids
        } else if let Some(uids) = assign_studies.study_instance_uids {
            // study_instance_uids가 제공된 경우 변환
            let mut ids = Vec::new();
            for uid in &uids {
                let study = self
                    .project_data_repository
                    .as_ref()
                    .find_study_by_uid(subject.project_id, uid)
                    .await
                    .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
                    .ok_or_else(|| {
                        ServiceError::NotFound(format!("Study with UID {} not found", uid))
                    })?;
                ids.push(study.id);
            }
            ids
        } else {
            return Err(ServiceError::ValidationError(
                "Either study_ids or study_instance_uids must be provided".into(),
            ));
        };

        // 4. Study 할당 (MOVE 시맨틱)
        let assigned_count = self
            .timepoint_study_repository
            .assign_studies(timepoint_id, &study_ids, user_id)
            .await?;

        Ok(AssignmentResult {
            affected_count: assigned_count,
            study_ids,
        })
    }

    async fn unassign_studies(
        &self,
        timepoint_id: i32,
        unassign_studies: UnassignStudies,
    ) -> Result<i64, ServiceError> {
        // 1. TimePoint 존재 확인 및 Subject 조회
        let timepoint = self
            .timepoint_repository
            .find_by_id(timepoint_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("TimePoint not found".into()))?;

        // 2. Subject 조회 (project_id 필요)
        let subject = self
            .subject_repository
            .find_by_id(timepoint.subject_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Subject not found".into()))?;

        // 3. study_instance_uids를 study_ids로 변환 (필요한 경우)
        let study_ids = if let Some(ids) = unassign_studies.study_ids {
            // study_ids가 제공된 경우 그대로 사용
            ids
        } else if let Some(uids) = unassign_studies.study_instance_uids {
            // study_instance_uids가 제공된 경우 변환
            let mut ids = Vec::new();
            for uid in &uids {
                let study = self
                    .project_data_repository
                    .as_ref()
                    .find_study_by_uid(subject.project_id, uid)
                    .await
                    .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
                    .ok_or_else(|| {
                        ServiceError::NotFound(format!("Study with UID {} not found", uid))
                    })?;
                ids.push(study.id);
            }
            ids
        } else {
            return Err(ServiceError::ValidationError(
                "Either study_ids or study_instance_uids must be provided".into(),
            ));
        };

        // 4. Study 해제
        Ok(self
            .timepoint_study_repository
            .unassign_studies(&study_ids)
            .await? as i64)
    }

    async fn get_studies_by_timepoint(
        &self,
        timepoint_id: i32,
    ) -> Result<TimePointStudies, ServiceError> {
        // 1. TimePoint 존재 확인
        let timepoint = self
            .timepoint_repository
            .find_by_id(timepoint_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("TimePoint not found".into()))?;

        // 2. Study 목록 조회
        let studies = self
            .timepoint_study_repository
            .find_studies_by_timepoint(timepoint_id)
            .await?;

        Ok(TimePointStudies {
            timepoint_id,
            timepoint_name: timepoint.name,
            studies,
        })
    }

    async fn get_unassigned_studies_by_subject(
        &self,
        subject_id: i32,
    ) -> Result<Vec<StudyInfo>, ServiceError> {
        // 1. Subject 존재 확인
        self.subject_repository
            .find_by_id(subject_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Subject not found".into()))?;

        // 2. 미할당 Study 목록 조회
        Ok(self
            .timepoint_study_repository
            .find_unassigned_studies_by_subject(subject_id)
            .await?)
    }
}
