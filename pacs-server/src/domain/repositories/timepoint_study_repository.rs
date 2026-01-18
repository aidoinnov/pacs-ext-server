use crate::domain::entities::{StudyInfo, TimePointStudy};
use async_trait::async_trait;
use sqlx::PgPool;

/// TimePointStudy Repository Trait
///
/// TimePoint-Study 매핑에 대한 데이터 접근 인터페이스를 정의합니다.
/// 이 트레이트는 도메인 계층에서 정의되며, 인프라 계층에서 구현됩니다.
#[async_trait]
pub trait TimePointStudyRepository: Send + Sync {
    /// TimePoint에 할당된 Study 목록 조회
    ///
    /// # Arguments
    /// * `timepoint_id` - TimePoint ID
    ///
    /// # Returns
    /// * `Ok(Vec<StudyInfo>)` - Study 정보 목록
    /// * `Err(sqlx::Error)` - 데이터베이스 오류
    async fn find_studies_by_timepoint(
        &self,
        timepoint_id: i32,
    ) -> Result<Vec<StudyInfo>, sqlx::Error>;

    /// Subject의 Unassigned Study 목록 조회
    ///
    /// # Arguments
    /// * `subject_id` - Subject ID
    ///
    /// # Returns
    /// * `Ok(Vec<StudyInfo>)` - Unassigned Study 정보 목록
    /// * `Err(sqlx::Error)` - 데이터베이스 오류
    async fn find_unassigned_studies_by_subject(
        &self,
        subject_id: i32,
    ) -> Result<Vec<StudyInfo>, sqlx::Error>;

    /// Study가 할당된 TimePoint 조회
    ///
    /// # Arguments
    /// * `study_id` - Study ID
    ///
    /// # Returns
    /// * `Ok(Some(i32))` - TimePoint ID
    /// * `Ok(None)` - Unassigned 상태
    /// * `Err(sqlx::Error)` - 데이터베이스 오류
    async fn find_timepoint_by_study(&self, study_id: i32)
        -> Result<Option<i32>, sqlx::Error>;

    /// Study를 TimePoint에 할당
    ///
    /// # Arguments
    /// * `timepoint_id` - TimePoint ID
    /// * `study_ids` - 할당할 Study ID 목록
    /// * `user_id` - 할당하는 사용자 ID
    ///
    /// # Returns
    /// * `Ok(i32)` - 할당된 Study 개수
    /// * `Err(sqlx::Error)` - 데이터베이스 오류
    ///
    /// # Note
    /// 이미 다른 TimePoint에 할당된 Study는 자동으로 재할당됨 (MOVE 동작)
    async fn assign_studies(
        &self,
        timepoint_id: i32,
        study_ids: &[i32],
        user_id: i32,
    ) -> Result<i32, sqlx::Error>;

    /// Study를 Unassigned로 이동
    ///
    /// # Arguments
    /// * `study_ids` - 해제할 Study ID 목록
    ///
    /// # Returns
    /// * `Ok(i32)` - 해제된 Study 개수
    /// * `Err(sqlx::Error)` - 데이터베이스 오류
    async fn unassign_studies(&self, study_ids: &[i32]) -> Result<i32, sqlx::Error>;

    /// TimePoint의 모든 Study 해제
    ///
    /// # Arguments
    /// * `timepoint_id` - TimePoint ID
    ///
    /// # Returns
    /// * `Ok(i32)` - 해제된 Study 개수
    /// * `Err(sqlx::Error)` - 데이터베이스 오류
    async fn unassign_all_studies_from_timepoint(
        &self,
        timepoint_id: i32,
    ) -> Result<i32, sqlx::Error>;

    /// Subject의 전체 보드 데이터 조회 (Unassigned + TimePoints)
    ///
    /// # Arguments
    /// * `subject_id` - Subject ID
    ///
    /// # Returns
    /// * `Ok(serde_json::Value)` - 보드 데이터 (JSON)
    /// * `Err(sqlx::Error)` - 데이터베이스 오류
    async fn get_subject_board_data(
        &self,
        subject_id: i32,
    ) -> Result<serde_json::Value, sqlx::Error>;

    /// TimePoint별 Study 개수 조회
    ///
    /// # Arguments
    /// * `timepoint_id` - TimePoint ID
    ///
    /// # Returns
    /// * `Ok(i64)` - Study 개수
    /// * `Err(sqlx::Error)` - 데이터베이스 오류
    async fn count_studies_by_timepoint(&self, timepoint_id: i32)
        -> Result<i64, sqlx::Error>;

    /// Subject별 Unassigned Study 개수 조회
    ///
    /// # Arguments
    /// * `subject_id` - Subject ID
    ///
    /// # Returns
    /// * `Ok(i64)` - Unassigned Study 개수
    /// * `Err(sqlx::Error)` - 데이터베이스 오류
    async fn count_unassigned_studies_by_subject(
        &self,
        subject_id: i32,
    ) -> Result<i64, sqlx::Error>;

    /// 데이터베이스 연결 풀 반환
    fn pool(&self) -> &PgPool;
}

