use crate::domain::entities::{CreateSubject, Subject, SubjectDetail, UpdateSubject};
use crate::domain::repositories::{ProjectRepository, SubjectRepository};
use crate::domain::ServiceError;
use async_trait::async_trait;

/// Subject 관리 도메인 서비스
///
/// 이 트레이트는 Subject와 관련된 비즈니스 로직을 정의합니다.
#[async_trait]
pub trait SubjectService: Send + Sync {
    /// Subject를 생성합니다.
    ///
    /// # 비즈니스 규칙
    /// - 프로젝트가 존재해야 함
    /// - Subject 코드는 프로젝트 내에서 유일해야 함
    /// - Patient ID는 프로젝트 내에서 유일해야 함 (제공된 경우)
    /// - Subject 코드는 유효한 형식이어야 함 (1-50자, 영문자/숫자/하이픈/언더스코어)
    ///
    /// # 매개변수
    /// - `new_subject`: 생성할 Subject 정보
    ///
    /// # 반환값
    /// - `Ok(Subject)`: 생성된 Subject
    /// - `Err(ServiceError)`: 비즈니스 규칙 위반 또는 데이터베이스 오류
    async fn create_subject(&self, new_subject: CreateSubject) -> Result<Subject, ServiceError>;

    /// Subject를 조회합니다.
    ///
    /// # 매개변수
    /// - `id`: Subject ID
    ///
    /// # 반환값
    /// - `Ok(Subject)`: 조회된 Subject
    /// - `Err(ServiceError::NotFound)`: Subject가 존재하지 않음
    async fn get_subject(&self, id: i32) -> Result<Subject, ServiceError>;

    /// Subject 상세 정보를 조회합니다 (통계 포함).
    ///
    /// # 매개변수
    /// - `id`: Subject ID
    ///
    /// # 반환값
    /// - `Ok(SubjectDetail)`: Subject 상세 정보
    /// - `Err(ServiceError::NotFound)`: Subject가 존재하지 않음
    async fn get_subject_detail(&self, id: i32) -> Result<SubjectDetail, ServiceError>;

    /// 프로젝트의 모든 Subject를 조회합니다.
    ///
    /// # 매개변수
    /// - `project_id`: 프로젝트 ID
    ///
    /// # 반환값
    /// - `Ok(Vec<Subject>)`: Subject 목록
    async fn get_subjects_by_project(&self, project_id: i32) -> Result<Vec<Subject>, ServiceError>;

    /// Subject를 수정합니다.
    ///
    /// # 비즈니스 규칙
    /// - Subject가 존재해야 함
    /// - Subject 코드 변경 시 프로젝트 내에서 유일해야 함
    /// - Patient ID 변경 시 프로젝트 내에서 유일해야 함
    ///
    /// # 매개변수
    /// - `id`: Subject ID
    /// - `update_subject`: 수정할 Subject 정보
    ///
    /// # 반환값
    /// - `Ok(Subject)`: 수정된 Subject
    /// - `Err(ServiceError)`: 비즈니스 규칙 위반 또는 데이터베이스 오류
    async fn update_subject(
        &self,
        id: i32,
        update_subject: UpdateSubject,
    ) -> Result<Subject, ServiceError>;

    /// Subject를 삭제합니다.
    ///
    /// # 비즈니스 규칙
    /// - Subject가 존재해야 함
    /// - Subject에 연결된 TimePoint가 있으면 삭제 불가 (CASCADE 방지)
    ///
    /// # 매개변수
    /// - `id`: Subject ID
    ///
    /// # 반환값
    /// - `Ok(())`: 삭제 성공
    /// - `Err(ServiceError)`: 비즈니스 규칙 위반 또는 데이터베이스 오류
    async fn delete_subject(&self, id: i32) -> Result<(), ServiceError>;

    /// 프로젝트의 Subject 목록 최종 수정 시간 조회 (ETag 캐싱용)
    ///
    /// # 매개변수
    /// - `project_id`: 프로젝트 ID
    ///
    /// # 반환값
    /// - `Ok(chrono::NaiveDateTime)`: 최종 수정 시간
    async fn get_subjects_updated_at(&self, project_id: i32) -> Result<chrono::NaiveDateTime, ServiceError>;
}

/// Subject 서비스 구현체
#[derive(Clone)]
pub struct SubjectServiceImpl<S, P>
where
    S: SubjectRepository,
    P: ProjectRepository,
{
    subject_repository: S,
    project_repository: P,
}

impl<S, P> SubjectServiceImpl<S, P>
where
    S: SubjectRepository,
    P: ProjectRepository,
{
    pub fn new(subject_repository: S, project_repository: P) -> Self {
        Self {
            subject_repository,
            project_repository,
        }
    }
}

#[async_trait]
impl<S, P> SubjectService for SubjectServiceImpl<S, P>
where
    S: SubjectRepository,
    P: ProjectRepository,
{
    async fn create_subject(&self, new_subject: CreateSubject) -> Result<Subject, ServiceError> {
        // 1. 프로젝트 존재 확인
        self.project_repository
            .find_by_id(new_subject.project_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Project not found".into()))?;

        // 2. Subject 코드 유효성 검증
        if !Subject::validate_subject_code(&new_subject.subject_code) {
            return Err(ServiceError::ValidationError(
                "Invalid subject code format. Must be 1-50 characters, alphanumeric, hyphen, or underscore only".into(),
            ));
        }

        // 3. Subject 코드 중복 체크
        if let Some(_) = self
            .subject_repository
            .find_by_code(new_subject.project_id, &new_subject.subject_code)
            .await?
        {
            return Err(ServiceError::AlreadyExists(format!(
                "Subject code '{}' already exists in this project",
                new_subject.subject_code
            )));
        }

        // 4. Patient ID 중복 체크 (제공된 경우)
        if let Some(ref patient_id) = new_subject.patient_id {
            if let Some(_) = self
                .subject_repository
                .find_by_patient_id(new_subject.project_id, patient_id)
                .await?
            {
                return Err(ServiceError::AlreadyExists(format!(
                    "Patient ID '{}' already exists in this project",
                    patient_id
                )));
            }
        }

        // 5. Subject 생성
        Ok(self.subject_repository.create(new_subject).await?)
    }

    async fn get_subject(&self, id: i32) -> Result<Subject, ServiceError> {
        self.subject_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Subject not found".into()))
    }

    async fn get_subject_detail(&self, id: i32) -> Result<SubjectDetail, ServiceError> {
        self.subject_repository
            .find_detail_by_id(id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Subject not found".into()))
    }

    async fn get_subjects_by_project(&self, project_id: i32) -> Result<Vec<Subject>, ServiceError> {
        // 프로젝트 존재 확인
        self.project_repository
            .find_by_id(project_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Project not found".into()))?;

        Ok(self.subject_repository.find_by_project(project_id).await?)
    }

    async fn update_subject(
        &self,
        id: i32,
        update_subject: UpdateSubject,
    ) -> Result<Subject, ServiceError> {
        // 1. Subject 존재 확인
        let existing = self
            .subject_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Subject not found".into()))?;

        // 2. Subject 코드 변경 시 유효성 검증
        if let Some(ref new_code) = update_subject.subject_code {
            if !Subject::validate_subject_code(new_code) {
                return Err(ServiceError::ValidationError(
                    "Invalid subject code format".into(),
                ));
            }

            // 3. Subject 코드 중복 체크 (다른 Subject와 중복되는지)
            if new_code != &existing.subject_code {
                if let Some(_) = self
                    .subject_repository
                    .find_by_code(existing.project_id, new_code)
                    .await?
                {
                    return Err(ServiceError::AlreadyExists(format!(
                        "Subject code '{}' already exists in this project",
                        new_code
                    )));
                }
            }
        }

        // 4. Patient ID 변경 시 중복 체크
        if let Some(ref new_patient_id) = update_subject.patient_id {
            if Some(new_patient_id) != existing.patient_id.as_ref() {
                if let Some(_) = self
                    .subject_repository
                    .find_by_patient_id(existing.project_id, new_patient_id)
                    .await?
                {
                    return Err(ServiceError::AlreadyExists(format!(
                        "Patient ID '{}' already exists in this project",
                        new_patient_id
                    )));
                }
            }
        }

        // 5. Subject 수정
        self.subject_repository
            .update(id, update_subject)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Subject not found".into()))
    }

    async fn delete_subject(&self, id: i32) -> Result<(), ServiceError> {
        // Subject 존재 확인
        let subject = self
            .subject_repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Subject not found".into()))?;

        // TimePoint 존재 여부 확인 (CASCADE 방지)
        let detail = self.subject_repository.find_detail_by_id(id).await?;
        if let Some(detail) = detail {
            if detail.timepoint_count > 0 {
                return Err(ServiceError::ValidationError(format!(
                    "Cannot delete subject with {} timepoint(s). Delete timepoints first.",
                    detail.timepoint_count
                )));
            }
        }

        // Subject 삭제
        let deleted = self.subject_repository.delete(id).await?;
        if !deleted {
            return Err(ServiceError::NotFound("Subject not found".into()));
        }

        Ok(())
    }

    async fn get_subjects_updated_at(&self, project_id: i32) -> Result<chrono::NaiveDateTime, ServiceError> {
        Ok(self.subject_repository.get_subjects_updated_at(project_id).await?)
    }
}
