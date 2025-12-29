//! # Series User Report Repository 트레이트
//!
//! 이 모듈은 Series User Report 데이터 접근을 위한 Repository 트레이트를 정의합니다.

use crate::domain::reporting::entities::series_user_report::{
    NewSeriesUserReport, SeriesUserReport, UpdateSeriesUserReport,
};
use async_trait::async_trait;
use sqlx::PgPool;

#[async_trait]
pub trait SeriesUserReportRepository: Send + Sync {
    /// Report 생성 또는 업데이트 (UPSERT)
    async fn create_or_update(
        &self,
        series_id: i32,
        user_id: i32,
        project_id: Option<i32>,
        new_report: &NewSeriesUserReport,
    ) -> Result<SeriesUserReport, sqlx::Error>;

    /// Report 조회 (유저-시리즈-프로젝트 조합)
    async fn find_by_series_user_project(
        &self,
        series_id: i32,
        user_id: i32,
        project_id: Option<i32>,
    ) -> Result<Option<SeriesUserReport>, sqlx::Error>;

    /// 시리즈의 모든 Report 조회
    async fn find_by_series(
        &self,
        series_id: i32,
        project_id: Option<i32>,
    ) -> Result<Vec<SeriesUserReport>, sqlx::Error>;

    /// Report 업데이트
    async fn update(
        &self,
        series_id: i32,
        user_id: i32,
        project_id: Option<i32>,
        update: &UpdateSeriesUserReport,
    ) -> Result<SeriesUserReport, sqlx::Error>;

    /// Report 삭제
    async fn delete(
        &self,
        series_id: i32,
        user_id: i32,
        project_id: Option<i32>,
    ) -> Result<bool, sqlx::Error>;

    /// 데이터베이스 풀 참조
    fn pool(&self) -> &PgPool;
}

