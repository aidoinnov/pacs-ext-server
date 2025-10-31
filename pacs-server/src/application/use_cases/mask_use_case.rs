use crate::application::dto::mask_dto::{
    CreateMaskRequest, DownloadUrlRequest, DownloadUrlResponse, MaskListResponse, MaskResponse,
    MaskStatsResponse, UpdateMaskRequest,
};
use crate::application::services::SignedUrlService;
use crate::domain::entities::{NewMask, UpdateMask};
use crate::domain::services::{MaskGroupService, MaskService};
use crate::domain::ServiceError;
use std::sync::Arc;

/// Mask 관리 유스케이스
pub struct MaskUseCase<MS, MGS, SUS>
where
    MS: MaskService + Send + Sync,
    MGS: MaskGroupService + Send + Sync,
    SUS: SignedUrlService + Send + Sync,
{
    mask_service: Arc<MS>,
    mask_group_service: Arc<MGS>,
    signed_url_service: Arc<SUS>,
}

impl<MS, MGS, SUS> MaskUseCase<MS, MGS, SUS>
where
    MS: MaskService + Send + Sync,
    MGS: MaskGroupService + Send + Sync,
    SUS: SignedUrlService + Send + Sync,
{
    pub fn new(
        mask_service: Arc<MS>,
        mask_group_service: Arc<MGS>,
        signed_url_service: Arc<SUS>,
    ) -> Self {
        Self {
            mask_service,
            mask_group_service,
            signed_url_service,
        }
    }

    /// Mask 생성
    pub async fn create_mask(
        &self,
        request: CreateMaskRequest,
        user_id: i32,
    ) -> Result<MaskResponse, ServiceError> {
        // 권한 확인 (Mask Group에 접근 가능한지 확인)
        self.mask_group_service
            .can_access_mask_group(user_id, request.mask_group_id)
            .await?;

        let new_mask = NewMask::new(
            request.mask_group_id,
            request.file_path,
            request.mime_type,
            request.slice_index,
            request.sop_instance_uid,
            request.label_name,
            request.file_size,
            request.checksum,
            request.width,
            request.height,
        );

        let mask = self.mask_service.create_mask(&new_mask).await?;

        Ok(MaskResponse {
            id: mask.id,
            mask_group_id: mask.mask_group_id,
            slice_index: mask.slice_index,
            sop_instance_uid: mask.sop_instance_uid,
            label_name: mask.label_name,
            file_path: mask.file_path,
            mime_type: mask.mime_type.unwrap_or_default(),
            file_size: mask.file_size,
            checksum: mask.checksum,
            width: mask.width,
            height: mask.height,
            created_at: mask.created_at.to_string(),
            updated_at: mask
                .updated_at
                .map(|dt| dt.to_string())
                .unwrap_or("".to_string()),
        })
    }

    /// Mask 조회
    pub async fn get_mask(&self, id: i32, user_id: i32) -> Result<MaskResponse, ServiceError> {
        // 권한 확인
        self.mask_service.can_access_mask(user_id, id).await?;

        let mask = self
            .mask_service
            .get_mask_by_id(id)
            .await?
            .ok_or_else(|| ServiceError::NotFound(format!("Mask with ID {} not found", id)))?;

        Ok(MaskResponse {
            id: mask.id,
            mask_group_id: mask.mask_group_id,
            slice_index: mask.slice_index,
            sop_instance_uid: mask.sop_instance_uid,
            label_name: mask.label_name,
            file_path: mask.file_path,
            mime_type: mask.mime_type.unwrap_or_default(),
            file_size: mask.file_size,
            checksum: mask.checksum,
            width: mask.width,
            height: mask.height,
            created_at: mask.created_at.to_string(),
            updated_at: mask
                .updated_at
                .map(|dt| dt.to_string())
                .unwrap_or("".to_string()),
        })
    }

    /// Mask 목록 조회
    pub async fn list_masks(
        &self,
        mask_group_id: Option<i32>,
        user_id: i32,
        offset: Option<i64>,
        limit: Option<i64>,
    ) -> Result<MaskListResponse, ServiceError> {
        // Mask Group이 지정된 경우 권한 확인
        if let Some(group_id) = mask_group_id {
            self.mask_group_service
                .can_access_mask_group(user_id, group_id)
                .await?;
        }

        let masks = self
            .mask_service
            .list_masks(
                mask_group_id,
                None, // sop_instance_uid 필터
                None, // label_name 필터
                None, // mime_type 필터
                offset,
                limit,
            )
            .await?;

        let total_count = self
            .mask_service
            .count_masks(mask_group_id, None, None, None)
            .await?;

        let mask_responses: Vec<MaskResponse> = masks
            .into_iter()
            .map(|mask| MaskResponse {
                id: mask.id,
                mask_group_id: mask.mask_group_id,
                slice_index: mask.slice_index,
                sop_instance_uid: mask.sop_instance_uid,
                label_name: mask.label_name,
                file_path: mask.file_path,
                mime_type: mask.mime_type.unwrap_or_default(),
                file_size: mask.file_size,
                checksum: mask.checksum,
                width: mask.width,
                height: mask.height,
                created_at: mask.created_at.to_string(),
                updated_at: mask
                    .updated_at
                    .map(|dt| dt.to_string())
                    .unwrap_or("".to_string()),
            })
            .collect();

        let page_size = limit.unwrap_or(50) as i32;
        let current_page = (offset.unwrap_or(0) / page_size as i64) as i32 + 1;
        let total_pages = ((total_count + page_size as i64 - 1) / page_size as i64) as i32;

        Ok(MaskListResponse {
            masks: mask_responses,
            total_count,
            offset: offset.unwrap_or(0),
            limit: limit.unwrap_or(50),
            current_page,
            page_size,
            total_pages,
        })
    }

    /// Mask 수정
    pub async fn update_mask(
        &self,
        id: i32,
        request: UpdateMaskRequest,
        user_id: i32,
    ) -> Result<MaskResponse, ServiceError> {
        // 권한 확인
        self.mask_service.can_access_mask(user_id, id).await?;

        let mut update_mask = UpdateMask::new(id);

        if let Some(slice_index) = request.slice_index {
            update_mask = update_mask.with_slice_index(slice_index);
        }
        if let Some(sop_instance_uid) = request.sop_instance_uid {
            update_mask = update_mask.with_sop_instance_uid(sop_instance_uid);
        }
        if let Some(label_name) = request.label_name {
            update_mask = update_mask.with_label_name(label_name);
        }
        if let Some(file_path) = request.file_path {
            update_mask = update_mask.with_file_path(file_path);
        }
        if let Some(mime_type) = request.mime_type {
            update_mask = update_mask.with_mime_type(mime_type);
        }
        if let Some(file_size) = request.file_size {
            update_mask = update_mask.with_file_size(file_size);
        }
        if let Some(checksum) = request.checksum {
            update_mask = update_mask.with_checksum(checksum);
        }

        let mask = self.mask_service.update_mask(id, &update_mask).await?;

        Ok(MaskResponse {
            id: mask.id,
            mask_group_id: mask.mask_group_id,
            slice_index: mask.slice_index,
            sop_instance_uid: mask.sop_instance_uid,
            label_name: mask.label_name,
            file_path: mask.file_path,
            mime_type: mask.mime_type.unwrap_or_default(),
            file_size: mask.file_size,
            checksum: mask.checksum,
            width: mask.width,
            height: mask.height,
            created_at: mask.created_at.to_string(),
            updated_at: mask
                .updated_at
                .map(|dt| dt.to_string())
                .unwrap_or("".to_string()),
        })
    }

    /// Mask 삭제
    pub async fn delete_mask(&self, id: i32, user_id: i32) -> Result<(), ServiceError> {
        // 권한 확인
        self.mask_service.can_access_mask(user_id, id).await?;

        self.mask_service.delete_mask(id).await?;
        Ok(())
    }

    /// 다운로드 URL 생성
    pub async fn generate_download_url(
        &self,
        request: DownloadUrlRequest,
        user_id: i32,
    ) -> Result<DownloadUrlResponse, ServiceError> {
        // 권한 확인
        self.mask_service
            .can_access_mask(user_id, request.mask_id)
            .await?;

        let signed_url = self
            .signed_url_service
            .generate_mask_download_url(request.file_path, request.expires_in)
            .await?;

        Ok(DownloadUrlResponse {
            download_url: signed_url.url,
            file_path: signed_url.file_path,
            expires_in: signed_url.ttl_seconds,
            expires_at: signed_url.expires_at.to_string(),
        })
    }

    /// Mask 통계 조회
    pub async fn get_mask_stats(
        &self,
        mask_group_id: Option<i32>,
        user_id: i32,
    ) -> Result<MaskStatsResponse, ServiceError> {
        // Mask Group이 지정된 경우 권한 확인
        if let Some(group_id) = mask_group_id {
            self.mask_group_service
                .can_access_mask_group(user_id, group_id)
                .await?;
        }

        let stats = self.mask_service.get_mask_stats(mask_group_id).await?;

        Ok(MaskStatsResponse {
            total_masks: stats.total_masks,
            total_size_bytes: stats.total_size_bytes,
            average_file_size: stats.average_file_size,
            masks_by_label: stats.label_names,
            mime_type_distribution: stats.mime_types,
        })
    }
}
