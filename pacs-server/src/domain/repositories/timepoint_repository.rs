use crate::domain::entities::{CreateTimePoint, TimePoint, UpdateTimePoint, VisitType};
use async_trait::async_trait;
use sqlx::PgPool;

/// TimePoint Repository Trait
///
/// TimePoint 엔티티에 대한 데이터 접근 인터페이스를 정의합니다.
/// 이 트레이트는 도메인 계층에서 정의되며, 인프라 계층에서 구현됩니다.
#[async_trait]
pub trait TimePointRepository: Send + Sync {
    /// ID로 TimePoint 조회
    ///
    /// # Arguments
    /// * `id` - TimePoint ID
    ///
    /// # Returns
    /// * `Ok(Some(TimePoint))` - TimePoint가 존재하는 경우
    /// * `Ok(None)` - TimePoint가 존재하지 않는 경우
    /// * `Err(sqlx::Error)` - 데이터베이스 오류
    async fn find_by_id(&self, id: i32) -> Result<Option<TimePoint>, sqlx::Error>;

    /// Subject의 모든 TimePoint 조회 (order_index 순)
    ///
    /// # Arguments
    /// * `subject_id` - Subject ID
    ///
    /// # Returns
    /// * `Ok(Vec<TimePoint>)` - TimePoint 목록 (order_index 오름차순)
    /// * `Err(sqlx::Error)` - 데이터베이스 오류
    async fn find_by_subject(&self, subject_id: i32) -> Result<Vec<TimePoint>, sqlx::Error>;

    /// Subject의 Baseline TimePoint 조회
    ///
    /// # Arguments
    /// * `subject_id` - Subject ID
    ///
    /// # Returns
    /// * `Ok(Some(TimePoint))` - Baseline TimePoint가 존재하는 경우
    /// * `Ok(None)` - Baseline TimePoint가 존재하지 않는 경우
    /// * `Err(sqlx::Error)` - 데이터베이스 오류
    async fn find_baseline_by_subject(
        &self,
        subject_id: i32,
    ) -> Result<Option<TimePoint>, sqlx::Error>;

    /// TimePoint 이름으로 조회
    ///
    /// # Arguments
    /// * `subject_id` - Subject ID
    /// * `name` - TimePoint 이름
    ///
    /// # Returns
    /// * `Ok(Some(TimePoint))` - TimePoint가 존재하는 경우
    /// * `Ok(None)` - TimePoint가 존재하지 않는 경우
    /// * `Err(sqlx::Error)` - 데이터베이스 오류
    async fn find_by_name(
        &self,
        subject_id: i32,
        name: &str,
    ) -> Result<Option<TimePoint>, sqlx::Error>;

    /// TimePoint 생성
    ///
    /// # Arguments
    /// * `new_timepoint` - 생성할 TimePoint 정보
    ///
    /// # Returns
    /// * `Ok(TimePoint)` - 생성된 TimePoint
    /// * `Err(sqlx::Error)` - 데이터베이스 오류 (Baseline 중복, 이름 중복 등)
    async fn create(&self, new_timepoint: CreateTimePoint) -> Result<TimePoint, sqlx::Error>;

    /// TimePoint 수정
    ///
    /// # Arguments
    /// * `id` - TimePoint ID
    /// * `update_timepoint` - 수정할 TimePoint 정보
    ///
    /// # Returns
    /// * `Ok(Some(TimePoint))` - 수정된 TimePoint
    /// * `Ok(None)` - TimePoint가 존재하지 않는 경우
    /// * `Err(sqlx::Error)` - 데이터베이스 오류
    async fn update(
        &self,
        id: i32,
        update_timepoint: UpdateTimePoint,
    ) -> Result<Option<TimePoint>, sqlx::Error>;

    /// TimePoint 삭제
    ///
    /// # Arguments
    /// * `id` - TimePoint ID
    ///
    /// # Returns
    /// * `Ok(true)` - 삭제 성공
    /// * `Ok(false)` - TimePoint가 존재하지 않는 경우
    /// * `Err(sqlx::Error)` - 데이터베이스 오류
    ///
    /// # Note
    /// TimePoint 삭제 시 매핑된 모든 Study는 Unassigned 상태로 변경됨 (CASCADE DELETE)
    async fn delete(&self, id: i32) -> Result<bool, sqlx::Error>;

    /// External Key로 조회 (CTIMS 연동용)
    ///
    /// # Arguments
    /// * `external_key` - CTIMS TimePoint Key
    ///
    /// # Returns
    /// * `Ok(Some(TimePoint))` - TimePoint가 존재하는 경우
    /// * `Ok(None)` - TimePoint가 존재하지 않는 경우
    /// * `Err(sqlx::Error)` - 데이터베이스 오류
    async fn find_by_external_key(
        &self,
        external_key: &str,
    ) -> Result<Option<TimePoint>, sqlx::Error>;

    /// Visit Type으로 조회
    ///
    /// # Arguments
    /// * `subject_id` - Subject ID
    /// * `visit_type` - Visit Type
    ///
    /// # Returns
    /// * `Ok(Vec<TimePoint>)` - TimePoint 목록
    /// * `Err(sqlx::Error)` - 데이터베이스 오류
    async fn find_by_visit_type(
        &self,
        subject_id: i32,
        visit_type: VisitType,
    ) -> Result<Vec<TimePoint>, sqlx::Error>;

    /// 데이터베이스 연결 풀 반환
    fn pool(&self) -> &PgPool;
}

