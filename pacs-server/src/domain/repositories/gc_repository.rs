use async_trait::async_trait;
use crate::domain::entities::annotation::{Annotation, SnapshotUploadStatus};
#[async_trait]
pub trait GcRepository: Send + Sync {
    /// PENDING 상태의 스냅샷 조회 (grace period 이상 경과)
    async fn find_pending_snapshots(
        &self,
        grace_days: i32,
        batch_size: i32
    ) -> Result<Vec<Annotation>, sqlx::Error>;

    /// FAILED 상태의 스냅샷 조회 (grace period 이상 경과)
    async fn find_failed_snapshots(
        &self,
        grace_days: i32,
        batch_size: i32
    ) -> Result<Vec<Annotation>, sqlx::Error>;

    /// 스냅샷 상태 업데이트
    async fn update_snapshot_status(
        &self,
        annotation_id: i32,
        status: SnapshotUploadStatus
    ) -> Result<(), sqlx::Error>;

    /// 스냅샷 이미지 키를 NULL로 설정 (S3 삭제 후)
    async fn clear_snapshot_image_key(
        &self,
        annotation_id: i32
    ) -> Result<(), sqlx::Error>;
}