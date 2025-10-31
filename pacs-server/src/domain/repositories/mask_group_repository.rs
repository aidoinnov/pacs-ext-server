use crate::domain::entities::mask::Mask;
use crate::domain::entities::mask_group::{
    MaskGroup, MaskGroupStats, NewMaskGroup, UpdateMaskGroup,
};
use crate::domain::ServiceError;
use async_trait::async_trait;

#[async_trait]
pub trait MaskGroupRepository: Send + Sync {
    /// 마스크 그룹 생성
    async fn create(&self, mask_group: &NewMaskGroup) -> Result<MaskGroup, ServiceError>;

    /// ID로 마스크 그룹 조회
    async fn get_by_id(&self, id: i32) -> Result<Option<MaskGroup>, ServiceError>;

    /// 마스크 그룹 업데이트
    async fn update(
        &self,
        id: i32,
        update_mask_group: &UpdateMaskGroup,
    ) -> Result<MaskGroup, ServiceError>;

    /// 마스크 그룹 삭제
    async fn delete(&self, id: i32) -> Result<(), ServiceError>;

    /// 마스크 그룹 목록 조회
    async fn list(
        &self,
        annotation_id: Option<i32>,
        created_by: Option<i32>,
        modality: Option<String>,
        mask_type: Option<String>,
        offset: Option<i64>,
        limit: Option<i64>,
    ) -> Result<Vec<MaskGroup>, ServiceError>;

    /// 마스크 그룹 내의 마스크들 조회
    async fn get_masks_in_group(&self, mask_group_id: i32) -> Result<Vec<Mask>, ServiceError>;

    /// 마스크 그룹 통계 조회
    async fn get_stats(&self, annotation_id: Option<i32>) -> Result<MaskGroupStats, ServiceError>;

    /// 마스크 그룹 개수 조회
    async fn count(
        &self,
        annotation_id: Option<i32>,
        created_by: Option<i32>,
        modality: Option<String>,
        mask_type: Option<String>,
    ) -> Result<i64, ServiceError>;
}
