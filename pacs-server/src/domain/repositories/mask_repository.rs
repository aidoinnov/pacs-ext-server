use async_trait::async_trait;
use crate::domain::entities::mask::{Mask, NewMask, UpdateMask, MaskStats};
use crate::domain::services::ServiceError;

#[async_trait]
pub trait MaskRepository: Send + Sync {
    /// 마스크 생성
    async fn create(&self, mask: &NewMask) -> Result<Mask, ServiceError>;
    
    /// ID로 마스크 조회
    async fn get_by_id(&self, id: i32) -> Result<Option<Mask>, ServiceError>;
    
    /// 마스크 업데이트
    async fn update(&self, id: i32, update_mask: &UpdateMask) -> Result<Mask, ServiceError>;
    
    /// 마스크 삭제
    async fn delete(&self, id: i32) -> Result<(), ServiceError>;
    
    /// 마스크 목록 조회
    async fn list(
        &self,
        mask_group_id: Option<i32>,
        sop_instance_uid: Option<String>,
        label_name: Option<String>,
        mime_type: Option<String>,
        offset: Option<i64>,
        limit: Option<i64>,
    ) -> Result<Vec<Mask>, ServiceError>;
    
    /// 마스크 통계 조회
    async fn get_stats(&self, mask_group_id: Option<i32>) -> Result<MaskStats, ServiceError>;
    
    /// 마스크 개수 조회
    async fn count(
        &self,
        mask_group_id: Option<i32>,
        sop_instance_uid: Option<String>,
        label_name: Option<String>,
        mime_type: Option<String>,
    ) -> Result<i64, ServiceError>;
}