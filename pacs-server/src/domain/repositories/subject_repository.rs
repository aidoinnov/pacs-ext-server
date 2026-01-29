use crate::domain::entities::{CreateSubject, Subject, SubjectDetail, UpdateSubject};
use async_trait::async_trait;
use sqlx::PgPool;

/// Subject Repository Trait
///
/// Subject 엔티티에 대한 데이터 접근 인터페이스를 정의합니다.
/// 이 트레이트는 도메인 계층에서 정의되며, 인프라 계층에서 구현됩니다.
#[async_trait]
pub trait SubjectRepository: Send + Sync {
    /// ID로 Subject 조회
    ///
    /// # Arguments
    /// * `id` - Subject ID
    ///
    /// # Returns
    /// * `Ok(Some(Subject))` - Subject가 존재하는 경우
    /// * `Ok(None)` - Subject가 존재하지 않는 경우
    /// * `Err(sqlx::Error)` - 데이터베이스 오류
    async fn find_by_id(&self, id: i32) -> Result<Option<Subject>, sqlx::Error>;

    /// Subject 코드로 조회
    ///
    /// # Arguments
    /// * `project_id` - 프로젝트 ID
    /// * `subject_code` - Subject 코드
    ///
    /// # Returns
    /// * `Ok(Some(Subject))` - Subject가 존재하는 경우
    /// * `Ok(None)` - Subject가 존재하지 않는 경우
    /// * `Err(sqlx::Error)` - 데이터베이스 오류
    async fn find_by_code(
        &self,
        project_id: i32,
        subject_code: &str,
    ) -> Result<Option<Subject>, sqlx::Error>;

    /// Patient ID로 조회
    ///
    /// # Arguments
    /// * `project_id` - 프로젝트 ID
    /// * `patient_id` - Patient ID
    ///
    /// # Returns
    /// * `Ok(Some(Subject))` - Subject가 존재하는 경우
    /// * `Ok(None)` - Subject가 존재하지 않는 경우
    /// * `Err(sqlx::Error)` - 데이터베이스 오류
    async fn find_by_patient_id(
        &self,
        project_id: i32,
        patient_id: &str,
    ) -> Result<Option<Subject>, sqlx::Error>;

    /// 프로젝트의 모든 Subject 조회
    ///
    /// # Arguments
    /// * `project_id` - 프로젝트 ID
    ///
    /// # Returns
    /// * `Ok(Vec<Subject>)` - Subject 목록
    /// * `Err(sqlx::Error)` - 데이터베이스 오류
    async fn find_by_project(&self, project_id: i32) -> Result<Vec<Subject>, sqlx::Error>;

    /// Subject 상세 정보 조회 (통계 포함)
    ///
    /// # Arguments
    /// * `id` - Subject ID
    ///
    /// # Returns
    /// * `Ok(Some(SubjectDetail))` - Subject 상세 정보
    /// * `Ok(None)` - Subject가 존재하지 않는 경우
    /// * `Err(sqlx::Error)` - 데이터베이스 오류
    async fn find_detail_by_id(&self, id: i32) -> Result<Option<SubjectDetail>, sqlx::Error>;

    /// Subject 생성
    ///
    /// # Arguments
    /// * `new_subject` - 생성할 Subject 정보
    ///
    /// # Returns
    /// * `Ok(Subject)` - 생성된 Subject
    /// * `Err(sqlx::Error)` - 데이터베이스 오류 (중복 코드 등)
    async fn create(&self, new_subject: CreateSubject) -> Result<Subject, sqlx::Error>;

    /// Subject 수정
    ///
    /// # Arguments
    /// * `id` - Subject ID
    /// * `update_subject` - 수정할 Subject 정보
    ///
    /// # Returns
    /// * `Ok(Some(Subject))` - 수정된 Subject
    /// * `Ok(None)` - Subject가 존재하지 않는 경우
    /// * `Err(sqlx::Error)` - 데이터베이스 오류
    async fn update(
        &self,
        id: i32,
        update_subject: UpdateSubject,
    ) -> Result<Option<Subject>, sqlx::Error>;

    /// Subject 삭제
    ///
    /// # Arguments
    /// * `id` - Subject ID
    ///
    /// # Returns
    /// * `Ok(true)` - 삭제 성공
    /// * `Ok(false)` - Subject가 존재하지 않는 경우
    /// * `Err(sqlx::Error)` - 데이터베이스 오류
    async fn delete(&self, id: i32) -> Result<bool, sqlx::Error>;

    /// External Subject Key로 조회 (CTIMS 연동용)
    ///
    /// # Arguments
    /// * `external_key` - CTIMS Subject PK
    ///
    /// # Returns
    /// * `Ok(Some(Subject))` - Subject가 존재하는 경우
    /// * `Ok(None)` - Subject가 존재하지 않는 경우
    /// * `Err(sqlx::Error)` - 데이터베이스 오류
    async fn find_by_external_key(
        &self,
        external_key: &str,
    ) -> Result<Option<Subject>, sqlx::Error>;

    /// 데이터베이스 연결 풀 반환
    fn pool(&self) -> &PgPool;

    /// 프로젝트의 Subject 목록 최종 수정 시간 조회 (ETag 캐싱용)
    ///
    /// # Arguments
    /// * `project_id` - 프로젝트 ID
    ///
    /// # Returns
    /// * `Ok(chrono::NaiveDateTime)` - 최종 수정 시간
    /// * `Err(sqlx::Error)` - 데이터베이스 오류
    async fn get_subjects_updated_at(&self, project_id: i32) -> Result<chrono::NaiveDateTime, sqlx::Error>;
}

