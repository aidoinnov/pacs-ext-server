use async_trait::async_trait;
use crate::domain::ServiceError;


/// GC 작업 결과
#[derive(Debug, Clone)]
pub struct GcResult{
    pub annotation_id: i32,
    pub snapshot_image_key: String,
    pub success: bool,
    pub error_message: Option<String>,
}

#[async_trait]
pub trait GcService: Send + Sync {
    /// Job A: PENDING 상태 타임아웃 처리
    async fn timeout_pending_snapshots(
        &self,
        grace_days: i32,
        batch_size: i32,
        dry_run: bool,
    ) -> Result<Vec<GcResult>, ServiceError>;

    /// Job B: FAILED 상태 스냅샷 정리
    async fn cleanup_failed_snapshots(
        &self,
        grace_days: i32,
        batch_size: i32,
        dry_run: bool,
    ) -> Result<Vec<GcResult>, ServiceError>;
}