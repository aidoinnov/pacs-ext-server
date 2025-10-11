use async_trait::async_trait;
use std::sync::Arc;
use crate::domain::entities::mask::{Mask, NewMask, UpdateMask, MaskStats};
use crate::domain::repositories::{MaskRepository, MaskGroupRepository, UserRepository};
use crate::domain::services::ServiceError;

/// 마스크 서비스 trait
/// 마스크 관련 비즈니스 로직을 정의합니다.
#[async_trait]
pub trait MaskService: Send + Sync {
    /// 새로운 마스크를 생성합니다.
    async fn create_mask(&self, new_mask: &NewMask) -> Result<Mask, ServiceError>;
    
    /// ID로 마스크를 조회합니다.
    async fn get_mask_by_id(&self, id: i32) -> Result<Option<Mask>, ServiceError>;
    
    /// 마스크를 업데이트합니다.
    async fn update_mask(&self, id: i32, update_mask: &UpdateMask) -> Result<Mask, ServiceError>;
    
    /// 마스크를 삭제합니다.
    async fn delete_mask(&self, id: i32) -> Result<(), ServiceError>;
    
    /// 마스크 목록을 조회합니다.
    async fn list_masks(
        &self,
        mask_group_id: Option<i32>,
        sop_instance_uid: Option<String>,
        label_name: Option<String>,
        mime_type: Option<String>,
        offset: Option<i64>,
        limit: Option<i64>,
    ) -> Result<Vec<Mask>, ServiceError>;
    
    /// 마스크 통계를 조회합니다.
    async fn get_mask_stats(&self, mask_group_id: Option<i32>) -> Result<MaskStats, ServiceError>;
    
    /// 마스크 개수를 조회합니다.
    async fn count_masks(
        &self,
        mask_group_id: Option<i32>,
        sop_instance_uid: Option<String>,
        label_name: Option<String>,
        mime_type: Option<String>,
    ) -> Result<i64, ServiceError>;
    
    /// 사용자가 마스크에 접근할 수 있는지 확인합니다.
    async fn can_access_mask(&self, user_id: i32, mask_id: i32) -> Result<bool, ServiceError>;
    
    /// 마스크 그룹에 마스크를 생성할 수 있는지 확인합니다.
    async fn can_create_mask(&self, user_id: i32, mask_group_id: i32) -> Result<bool, ServiceError>;
}

/// 마스크 서비스 구현체
#[derive(Debug, Clone)]
pub struct MaskServiceImpl<MR, MGR, UR> 
where
    MR: MaskRepository + Send + Sync,
    MGR: MaskGroupRepository + Send + Sync,
    UR: UserRepository + Send + Sync,
{
    mask_repository: Arc<MR>,
    mask_group_repository: Arc<MGR>,
    user_repository: Arc<UR>,
}

impl<MR, MGR, UR> MaskServiceImpl<MR, MGR, UR>
where
    MR: MaskRepository + Send + Sync,
    MGR: MaskGroupRepository + Send + Sync,
    UR: UserRepository + Send + Sync,
{
    /// 새로운 마스크 서비스 인스턴스를 생성합니다.
    pub fn new(
        mask_repository: Arc<MR>,
        mask_group_repository: Arc<MGR>,
        user_repository: Arc<UR>,
    ) -> Self {
        Self {
            mask_repository,
            mask_group_repository,
            user_repository,
        }
    }
}

#[async_trait]
impl<MR, MGR, UR> MaskService for MaskServiceImpl<MR, MGR, UR>
where
    MR: MaskRepository + Send + Sync,
    MGR: MaskGroupRepository + Send + Sync,
    UR: UserRepository + Send + Sync,
{
    async fn create_mask(&self, new_mask: &NewMask) -> Result<Mask, ServiceError> {
        // 마스크 그룹이 존재하는지 확인
        let mask_group = self.mask_group_repository
            .get_by_id(new_mask.mask_group_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound(format!("Mask group with id {} not found", new_mask.mask_group_id)))?;

        // 마스크 생성
        self.mask_repository
            .create(new_mask)
            .await
    }

    async fn get_mask_by_id(&self, id: i32) -> Result<Option<Mask>, ServiceError> {
        self.mask_repository
            .get_by_id(id)
            .await
    }

    async fn update_mask(&self, id: i32, update_mask: &UpdateMask) -> Result<Mask, ServiceError> {
        // 마스크가 존재하는지 확인
        let existing_mask = self.mask_repository
            .get_by_id(id)
            .await?
            .ok_or_else(|| ServiceError::NotFound(format!("Mask with id {} not found", id)))?;

        // 업데이트 실행
        self.mask_repository
            .update(id, update_mask)
            .await
    }

    async fn delete_mask(&self, id: i32) -> Result<(), ServiceError> {
        // 마스크가 존재하는지 확인
        let existing_mask = self.mask_repository
            .get_by_id(id)
            .await?
            .ok_or_else(|| ServiceError::NotFound(format!("Mask with id {} not found", id)))?;

        // 삭제 실행
        self.mask_repository
            .delete(id)
            .await
    }

    async fn list_masks(
        &self,
        mask_group_id: Option<i32>,
        sop_instance_uid: Option<String>,
        label_name: Option<String>,
        mime_type: Option<String>,
        offset: Option<i64>,
        limit: Option<i64>,
    ) -> Result<Vec<Mask>, ServiceError> {
        // 마스크 그룹이 존재하는지 확인 (mask_group_id가 있는 경우)
        if let Some(mg_id) = mask_group_id {
            let mask_group = self.mask_group_repository
                .get_by_id(mg_id)
                .await?
                .ok_or_else(|| ServiceError::NotFound(format!("Mask group with id {} not found", mg_id)))?;
        }

        self.mask_repository
            .list(mask_group_id, sop_instance_uid, label_name, mime_type, offset, limit)
            .await
    }

    async fn get_mask_stats(&self, mask_group_id: Option<i32>) -> Result<MaskStats, ServiceError> {
        // 마스크 그룹이 존재하는지 확인 (mask_group_id가 있는 경우)
        if let Some(mg_id) = mask_group_id {
            let mask_group = self.mask_group_repository
                .get_by_id(mg_id)
                .await?
                .ok_or_else(|| ServiceError::NotFound(format!("Mask group with id {} not found", mg_id)))?;
        }

        self.mask_repository
            .get_stats(mask_group_id)
            .await
    }

    async fn count_masks(
        &self,
        mask_group_id: Option<i32>,
        sop_instance_uid: Option<String>,
        label_name: Option<String>,
        mime_type: Option<String>,
    ) -> Result<i64, ServiceError> {
        // 마스크 그룹이 존재하는지 확인 (mask_group_id가 있는 경우)
        if let Some(mg_id) = mask_group_id {
            let mask_group = self.mask_group_repository
                .get_by_id(mg_id)
                .await?
                .ok_or_else(|| ServiceError::NotFound(format!("Mask group with id {} not found", mg_id)))?;
        }

        self.mask_repository
            .count(mask_group_id, sop_instance_uid, label_name, mime_type)
            .await
    }

    async fn can_access_mask(&self, user_id: i32, mask_id: i32) -> Result<bool, ServiceError> {
        // 마스크 조회
        let mask = self.mask_repository
            .get_by_id(mask_id)
            .await?;

        let mask = match mask {
            Some(m) => m,
            None => return Ok(false),
        };

        // 마스크 그룹 조회
        let mask_group = self.mask_group_repository
            .get_by_id(mask.mask_group_id)
            .await?;

        let mask_group = match mask_group {
            Some(mg) => mg,
            None => return Ok(false),
        };

        // 마스크 그룹에 접근할 수 있는지 확인
        // 여기서는 간단히 마스크 그룹의 created_by로 확인 (실제로는 프로젝트 권한 확인 필요)
        Ok(mask_group.created_by == Some(user_id))
    }

    async fn can_create_mask(&self, user_id: i32, mask_group_id: i32) -> Result<bool, ServiceError> {
        // 마스크 그룹 조회
        let mask_group = self.mask_group_repository
            .get_by_id(mask_group_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound(format!("Mask group with id {} not found", mask_group_id)))?;

        // 마스크 그룹에 접근할 수 있는지 확인
        // 여기서는 간단히 created_by로 확인 (실제로는 프로젝트 권한 확인 필요)
        Ok(mask_group.created_by == Some(user_id))
    }
}
