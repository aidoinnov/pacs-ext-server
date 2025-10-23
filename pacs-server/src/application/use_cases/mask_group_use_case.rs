use std::sync::Arc;
use crate::application::dto::mask_group_dto::{
    CreateMaskGroupRequest, UpdateMaskGroupRequest, MaskGroupResponse, 
    MaskGroupListResponse, MaskGroupDetailResponse, SignedUrlRequest, 
    SignedUrlResponse, CompleteUploadRequest, CompleteUploadResponse
};
use crate::domain::services::MaskGroupService;
use crate::domain::ServiceError;
use crate::application::services::SignedUrlService;
use crate::domain::entities::{NewMaskGroup, UpdateMaskGroup, MaskGroup};

/// Mask Group 관리 유스케이스
pub struct MaskGroupUseCase<MGS, SUS> 
where
    MGS: MaskGroupService + Send + Sync,
    SUS: SignedUrlService + Send + Sync,
{
    mask_group_service: Arc<MGS>,
    signed_url_service: Arc<SUS>,
}

impl<MGS, SUS> MaskGroupUseCase<MGS, SUS>
where
    MGS: MaskGroupService + Send + Sync,
    SUS: SignedUrlService + Send + Sync,
{
    pub fn new(mask_group_service: Arc<MGS>, signed_url_service: Arc<SUS>) -> Self {
        Self {
            mask_group_service,
            signed_url_service,
        }
    }

    /// Mask Group 생성
    pub async fn create_mask_group(
        &self, 
        annotation_id: i32,
        request: CreateMaskGroupRequest, 
        user_id: i32
    ) -> Result<MaskGroupResponse, ServiceError> {
        // 권한 확인
        self.mask_group_service
            .can_create_mask_group(user_id, annotation_id)
            .await?;

        let new_mask_group = NewMaskGroup::new(
            annotation_id,
            request.group_name,
            request.model_name,
            request.version,
            request.modality,
            request.slice_count,
            request.mask_type,
            request.description,
            Some(user_id),
        );

        let mask_group = self.mask_group_service.create_mask_group(&new_mask_group).await?;

        Ok(MaskGroupResponse {
            id: mask_group.id,
            annotation_id: mask_group.annotation_id,
            group_name: mask_group.group_name,
            model_name: mask_group.model_name,
            version: mask_group.version,
            modality: mask_group.modality,
            slice_count: mask_group.slice_count.unwrap_or(0),
            mask_type: mask_group.mask_type.unwrap_or_default(),
            description: mask_group.description,
            created_by: mask_group.created_by,
            created_at: mask_group.created_at.to_string(),
            updated_at: mask_group.updated_at.to_string(),
        })
    }

    /// Mask Group 조회
    pub async fn get_mask_group(&self, id: i32, user_id: i32) -> Result<MaskGroupDetailResponse, ServiceError> {
        // 권한 확인
        self.mask_group_service.can_access_mask_group(user_id, id).await?;

        let mask_group = self.mask_group_service
            .get_mask_group_by_id(id)
            .await?
            .ok_or_else(|| ServiceError::NotFound(format!("Mask group with ID {} not found", id)))?;

        // 통계 정보 조회
        let stats = self.mask_group_service
            .get_mask_group_stats(Some(mask_group.annotation_id))
            .await?;

        Ok(MaskGroupDetailResponse {
            id: mask_group.id,
            annotation_id: mask_group.annotation_id,
            group_name: mask_group.group_name,
            model_name: mask_group.model_name,
            version: mask_group.version,
            modality: mask_group.modality,
            slice_count: mask_group.slice_count,
            mask_type: mask_group.mask_type,
            description: mask_group.description,
            created_by: mask_group.created_by,
            created_at: mask_group.created_at.to_string(),
            updated_at: mask_group.updated_at.to_string(),
            stats,
        })
    }

    /// Mask Group 목록 조회
    pub async fn list_mask_groups(
        &self,
        annotation_id: Option<i32>,
        user_id: i32,
        offset: Option<i64>,
        limit: Option<i64>,
    ) -> Result<MaskGroupListResponse, ServiceError> {
        let mask_groups = self.mask_group_service
            .list_mask_groups(
                annotation_id,
                Some(user_id), // 사용자별 필터링
                None, // modality 필터
                None, // mask_type 필터
                offset,
                limit,
            )
            .await?;

        let total_count = self.mask_group_service
            .count_mask_groups(annotation_id, Some(user_id), None, None)
            .await?;

        let mask_group_responses: Vec<MaskGroupResponse> = mask_groups
            .into_iter()
            .map(|mg| MaskGroupResponse {
                id: mg.id,
                annotation_id: mg.annotation_id,
                group_name: mg.group_name,
                model_name: mg.model_name,
                version: mg.version,
                modality: mg.modality,
                slice_count: mg.slice_count.unwrap_or(0),
                mask_type: mg.mask_type.unwrap_or_default(),
                description: mg.description,
                created_by: mg.created_by,
                created_at: mg.created_at.to_string(),
                updated_at: mg.updated_at.to_string(),
            })
            .collect();

        Ok(MaskGroupListResponse {
            mask_groups: mask_group_responses,
            total_count,
            offset: offset.unwrap_or(0),
            limit: limit.unwrap_or(50),
        })
    }

    /// Mask Group 수정
    pub async fn update_mask_group(
        &self,
        id: i32,
        request: UpdateMaskGroupRequest,
        user_id: i32,
    ) -> Result<MaskGroupResponse, ServiceError> {
        // 권한 확인
        self.mask_group_service.can_access_mask_group(user_id, id).await?;

        let mut update_mask_group = UpdateMaskGroup::new(id);
        
        if let Some(group_name) = request.group_name {
            update_mask_group = update_mask_group.with_group_name(group_name);
        }
        if let Some(model_name) = request.model_name {
            if let Some(version) = request.version {
                update_mask_group = update_mask_group.with_model_info(model_name, version);
            }
        }
        if let Some(modality) = request.modality {
            update_mask_group = update_mask_group.with_modality(modality);
        }
        if let Some(slice_count) = request.slice_count {
            update_mask_group = update_mask_group.with_slice_count(slice_count);
        }
        if let Some(mask_type) = request.mask_type {
            update_mask_group = update_mask_group.with_mask_type(mask_type);
        }
        if let Some(description) = request.description {
            update_mask_group = update_mask_group.with_description(description);
        }

        let mask_group = self.mask_group_service
            .update_mask_group(id, &update_mask_group)
            .await?;

        Ok(MaskGroupResponse {
            id: mask_group.id,
            annotation_id: mask_group.annotation_id,
            group_name: mask_group.group_name,
            model_name: mask_group.model_name,
            version: mask_group.version,
            modality: mask_group.modality,
            slice_count: mask_group.slice_count.unwrap_or(0),
            mask_type: mask_group.mask_type.unwrap_or_default(),
            description: mask_group.description,
            created_by: mask_group.created_by,
            created_at: mask_group.created_at.to_string(),
            updated_at: mask_group.updated_at.to_string(),
        })
    }

    /// Mask Group 삭제
    pub async fn delete_mask_group(&self, id: i32, user_id: i32) -> Result<(), ServiceError> {
        // 권한 확인
        self.mask_group_service.can_access_mask_group(user_id, id).await?;

        self.mask_group_service.delete_mask_group(id).await?;
        Ok(())
    }

    /// Signed URL 생성 (업로드용)
    pub async fn generate_upload_url(
        &self,
        request: SignedUrlRequest,
        user_id: i32,
    ) -> Result<SignedUrlResponse, ServiceError> {
        // 권한 확인
        self.mask_group_service.can_access_mask_group(user_id, request.mask_group_id).await?;

        // 마스크 그룹에서 annotation_id 조회
        let mask_group = self.mask_group_service
            .get_mask_group_by_id(request.mask_group_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound(format!("Mask group with ID {} not found", request.mask_group_id)))?;

        let signed_url = self.signed_url_service
            .generate_mask_upload_url(
                mask_group.annotation_id, // 실제 annotation_id 사용
                request.mask_group_id,
                request.filename,
                request.mime_type,
                request.ttl_seconds,
                Some(user_id),
            )
            .await?;

        let url = signed_url.url.clone();
        Ok(SignedUrlResponse {
            upload_url: url.clone(),
            download_url: url, // 업로드와 다운로드 URL이 같을 수 있음
            file_path: signed_url.file_path,
            expires_in: signed_url.ttl_seconds,
            expires_at: signed_url.expires_at.to_string(),
        })
    }

    /// 업로드 완료 처리
    pub async fn complete_upload(
        &self,
        request: CompleteUploadRequest,
        user_id: i32,
    ) -> Result<CompleteUploadResponse, ServiceError> {
        // 권한 확인
        self.mask_group_service.can_access_mask_group(user_id, request.mask_group_id).await?;

        // 여기서는 단순히 성공 응답을 반환
        // 실제로는 업로드된 파일의 메타데이터를 검증하고 데이터베이스에 기록하는 로직이 필요
        Ok(CompleteUploadResponse {
            success: true,
            status: "success".to_string(),
            processed_masks: request.slice_count,
            uploaded_files: request.uploaded_files,
            message: "Upload completed successfully".to_string(),
        })
    }
}
