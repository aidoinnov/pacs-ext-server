use async_trait::async_trait;
use crate::domain::entities::gc_deletion_log::{GcDeletionLog, NewGcDeletionLog};

#[async_trait]
pub trait GcLogRepository: Send + Sync{
    /// GC 삭제 로그 삽입
    async fn insert(&self, log: NewGcDeletionLog) -> Result<GcDeletionLog, sqlx::Error>;

    /// 특정 기간의 로그 조회 (모니터링)
    async fn find_by_date_range(
        &self,
        start_date: chrono::DateTime<chrono::Utc>,
        end_date: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<GcDeletionLog>, sqlx::Error>;
}