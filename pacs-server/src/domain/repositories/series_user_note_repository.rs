use crate::domain::entities::SeriesUserNote;
use async_trait::async_trait;
use sqlx::PgPool;

/// Series User Note 데이터 접근을 위한 Repository 트레이트
///
/// 이 트레이트는 Series User Note의 데이터 접근 로직을 추상화합니다.
/// 구체적인 구현은 Infrastructure 계층에서 제공됩니다.
#[async_trait]
pub trait SeriesUserNoteRepository: Send + Sync {
    /// Series User Note를 생성하거나 업데이트합니다 (UPSERT).
    ///
    /// 같은 (series_id, user_id, project_id) 조합이 이미 존재하면 업데이트하고,
    /// 존재하지 않으면 새로 생성합니다.
    ///
    /// # 매개변수
    /// - `series_id`: Series ID
    /// - `user_id`: 사용자 ID
    /// - `project_id`: 프로젝트 ID (None이면 전역 note, Some(id)이면 프로젝트별 note)
    /// - `note`: 메모 텍스트
    ///
    /// # 반환값
    /// - `Ok(SeriesUserNote)`: 생성 또는 업데이트된 note
    /// - `Err(sqlx::Error)`: 데이터베이스 오류
    async fn create_or_update(
        &self,
        series_id: i32,
        user_id: i32,
        project_id: Option<i32>,
        note: String,
    ) -> Result<SeriesUserNote, sqlx::Error>;

    /// 특정 Series, User, Project 조합의 note를 조회합니다.
    ///
    /// # 매개변수
    /// - `series_id`: Series ID
    /// - `user_id`: 사용자 ID
    /// - `project_id`: 프로젝트 ID (None이면 전역 note 조회, Some(id)이면 프로젝트별 note 조회)
    ///
    /// # 반환값
    /// - `Ok(Some(SeriesUserNote))`: note가 존재하는 경우
    /// - `Ok(None)`: note가 존재하지 않는 경우
    /// - `Err(sqlx::Error)`: 데이터베이스 오류
    async fn find_by_series_user_project(
        &self,
        series_id: i32,
        user_id: i32,
        project_id: Option<i32>,
    ) -> Result<Option<SeriesUserNote>, sqlx::Error>;

    /// 특정 Series의 모든 note를 조회합니다.
    ///
    /// # 매개변수
    /// - `series_id`: Series ID
    /// - `project_id`: 프로젝트 ID 필터 (None이면 모든 note, Some(id)이면 해당 프로젝트의 note만)
    ///
    /// # 반환값
    /// - `Ok(Vec<SeriesUserNote>)`: 조회된 note 목록
    /// - `Err(sqlx::Error)`: 데이터베이스 오류
    async fn find_by_series(
        &self,
        series_id: i32,
        project_id: Option<i32>,
    ) -> Result<Vec<SeriesUserNote>, sqlx::Error>;

    /// Series User Note를 삭제합니다.
    ///
    /// # 매개변수
    /// - `series_id`: Series ID
    /// - `user_id`: 사용자 ID
    /// - `project_id`: 프로젝트 ID (None이면 전역 note 삭제, Some(id)이면 프로젝트별 note 삭제)
    ///
    /// # 반환값
    /// - `Ok(true)`: 삭제 성공
    /// - `Ok(false)`: 삭제할 note가 없음
    /// - `Err(sqlx::Error)`: 데이터베이스 오류
    async fn delete(
        &self,
        series_id: i32,
        user_id: i32,
        project_id: Option<i32>,
    ) -> Result<bool, sqlx::Error>;

    /// 데이터베이스 연결 풀을 반환합니다.
    ///
    /// # 반환값
    /// 데이터베이스 연결 풀 참조
    fn pool(&self) -> &PgPool;
}

