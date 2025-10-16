use async_trait::async_trait;
use std::sync::Arc;
use crate::domain::entities::mask_group::{MaskGroup, NewMaskGroup, UpdateMaskGroup, MaskGroupStats};
use crate::domain::entities::mask::Mask;
use crate::domain::repositories::{MaskGroupRepository, AnnotationRepository, UserRepository};
use crate::domain::ServiceError;

/// 마스크 그룹 서비스 trait
/// 마스크 그룹 관련 비즈니스 로직을 정의합니다.
#[async_trait]
pub trait MaskGroupService: Send + Sync {
    /// 새로운 마스크 그룹을 생성합니다.
    async fn create_mask_group(&self, new_mask_group: &NewMaskGroup) -> Result<MaskGroup, ServiceError>;
    
    /// ID로 마스크 그룹을 조회합니다.
    async fn get_mask_group_by_id(&self, id: i32) -> Result<Option<MaskGroup>, ServiceError>;
    
    /// 마스크 그룹을 업데이트합니다.
    async fn update_mask_group(&self, id: i32, update_mask_group: &UpdateMaskGroup) -> Result<MaskGroup, ServiceError>;
    
    /// 마스크 그룹을 삭제합니다.
    async fn delete_mask_group(&self, id: i32) -> Result<(), ServiceError>;
    
    /// 어노테이션의 마스크 그룹 목록을 조회합니다.
    async fn list_mask_groups(
        &self,
        annotation_id: Option<i32>,
        created_by: Option<i32>,
        modality: Option<String>,
        mask_type: Option<String>,
        offset: Option<i64>,
        limit: Option<i64>,
    ) -> Result<Vec<MaskGroup>, ServiceError>;
    
    /// 마스크 그룹의 마스크 목록을 조회합니다.
    async fn get_masks_in_group(&self, mask_group_id: i32) -> Result<Vec<Mask>, ServiceError>;
    
    /// 마스크 그룹 통계를 조회합니다.
    async fn get_mask_group_stats(&self, annotation_id: Option<i32>) -> Result<MaskGroupStats, ServiceError>;
    
    /// 마스크 그룹 개수를 조회합니다.
    async fn count_mask_groups(
        &self,
        annotation_id: Option<i32>,
        created_by: Option<i32>,
        modality: Option<String>,
        mask_type: Option<String>,
    ) -> Result<i64, ServiceError>;
    
    /// 사용자가 마스크 그룹에 접근할 수 있는지 확인합니다.
    async fn can_access_mask_group(&self, user_id: i32, mask_group_id: i32) -> Result<bool, ServiceError>;
    
    /// 어노테이션에 마스크 그룹을 생성할 수 있는지 확인합니다.
    async fn can_create_mask_group(&self, user_id: i32, annotation_id: i32) -> Result<bool, ServiceError>;
}

/// 마스크 그룹 서비스 구현체
#[derive(Debug, Clone)]
pub struct MaskGroupServiceImpl<MGR, AR, UR> 
where
    MGR: MaskGroupRepository + Send + Sync,
    AR: AnnotationRepository + Send + Sync,
    UR: UserRepository + Send + Sync,
{
    mask_group_repository: Arc<MGR>,
    annotation_repository: Arc<AR>,
    user_repository: Arc<UR>,
}

impl<MGR, AR, UR> MaskGroupServiceImpl<MGR, AR, UR>
where
    MGR: MaskGroupRepository + Send + Sync,
    AR: AnnotationRepository + Send + Sync,
    UR: UserRepository + Send + Sync,
{
    /// 새로운 마스크 그룹 서비스 인스턴스를 생성합니다.
    pub fn new(
        mask_group_repository: Arc<MGR>,
        annotation_repository: Arc<AR>,
        user_repository: Arc<UR>,
    ) -> Self {
        Self {
            mask_group_repository,
            annotation_repository,
            user_repository,
        }
    }
}

#[async_trait]
impl<MGR, AR, UR> MaskGroupService for MaskGroupServiceImpl<MGR, AR, UR>
where
    MGR: MaskGroupRepository + Send + Sync,
    AR: AnnotationRepository + Send + Sync,
    UR: UserRepository + Send + Sync,
{
    async fn create_mask_group(&self, new_mask_group: &NewMaskGroup) -> Result<MaskGroup, ServiceError> {
        // 트랜잭션을 사용하여 원자적 처리
        let mut tx = self.annotation_repository.pool().begin().await
            .map_err(|e| ServiceError::DatabaseError(format!("Failed to begin transaction: {}", e)))?;

        // 어노테이션이 존재하는지 확인 (트랜잭션 내에서)
        println!("🔍 [MaskGroupService] 어노테이션 존재 확인: annotation_id = {}", new_mask_group.annotation_id);
        let annotation = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM annotation_annotation WHERE id = $1)"
        )
        .bind(new_mask_group.annotation_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| ServiceError::DatabaseError(format!("Failed to check annotation existence: {}", e)))?;

        println!("🔍 [MaskGroupService] 어노테이션 존재 여부: {}", annotation);
        if !annotation {
            tx.rollback().await.ok();
            return Err(ServiceError::NotFound(format!("Annotation with id {} not found", new_mask_group.annotation_id)));
        }

        // 사용자가 존재하는지 확인 (created_by가 있는 경우)
        if let Some(created_by) = new_mask_group.created_by {
            let user_exists = sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS(SELECT 1 FROM security_user WHERE id = $1)"
            )
            .bind(created_by)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| ServiceError::DatabaseError(format!("Failed to check user existence: {}", e)))?;

            if !user_exists {
                tx.rollback().await.ok();
                return Err(ServiceError::NotFound(format!("User with id {} not found", created_by)));
            }
        }

        // 마스크 그룹 생성 (트랜잭션 내에서)
        let mask_group = sqlx::query_as::<_, MaskGroup>(
            "INSERT INTO annotation_mask_group (annotation_id, group_name, model_name, version, modality, slice_count, mask_type, description, created_by)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
             RETURNING id, annotation_id, group_name, model_name, version, modality, slice_count, mask_type, description, created_by, created_at, updated_at"
        )
        .bind(new_mask_group.annotation_id)
        .bind(&new_mask_group.group_name)
        .bind(&new_mask_group.model_name)
        .bind(&new_mask_group.version)
        .bind(&new_mask_group.modality)
        .bind(new_mask_group.slice_count)
        .bind(&new_mask_group.mask_type)
        .bind(&new_mask_group.description)
        .bind(new_mask_group.created_by)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| ServiceError::DatabaseError(format!("Failed to create mask group: {}", e)))?;

        tx.commit().await
            .map_err(|e| ServiceError::DatabaseError(format!("Failed to commit transaction: {}", e)))?;

        Ok(mask_group)
    }

    async fn get_mask_group_by_id(&self, id: i32) -> Result<Option<MaskGroup>, ServiceError> {
        self.mask_group_repository
            .get_by_id(id)
            .await
    }

    async fn update_mask_group(&self, id: i32, update_mask_group: &UpdateMaskGroup) -> Result<MaskGroup, ServiceError> {
        // 마스크 그룹이 존재하는지 확인
        let existing_mask_group = self.mask_group_repository
            .get_by_id(id)
            .await?
            .ok_or_else(|| ServiceError::NotFound(format!("Mask group with id {} not found", id)))?;

        // 업데이트 실행
        self.mask_group_repository
            .update(id, update_mask_group)
            .await
    }

    async fn delete_mask_group(&self, id: i32) -> Result<(), ServiceError> {
        // 마스크 그룹이 존재하는지 확인
        let existing_mask_group = self.mask_group_repository
            .get_by_id(id)
            .await?
            .ok_or_else(|| ServiceError::NotFound(format!("Mask group with id {} not found", id)))?;

        // 삭제 실행
        self.mask_group_repository
            .delete(id)
            .await
    }

    async fn list_mask_groups(
        &self,
        annotation_id: Option<i32>,
        created_by: Option<i32>,
        modality: Option<String>,
        mask_type: Option<String>,
        offset: Option<i64>,
        limit: Option<i64>,
    ) -> Result<Vec<MaskGroup>, ServiceError> {
        // 어노테이션이 존재하는지 확인 (annotation_id가 있는 경우)
        if let Some(ann_id) = annotation_id {
            let annotation = self.annotation_repository
                .find_by_id(ann_id)
                .await
                .map_err(|e| ServiceError::DatabaseError(format!("Failed to find annotation: {}", e)))?
                .ok_or_else(|| ServiceError::NotFound(format!("Annotation with id {} not found", ann_id)))?;
        }

        // 사용자가 존재하는지 확인 (created_by가 있는 경우)
        if let Some(user_id) = created_by {
            let user = self.user_repository
                .find_by_id(user_id)
                .await
                .map_err(|e| ServiceError::DatabaseError(format!("Failed to find user: {}", e)))?
                .ok_or_else(|| ServiceError::NotFound(format!("User with id {} not found", user_id)))?;
        }

        self.mask_group_repository
            .list(annotation_id, created_by, modality, mask_type, offset, limit)
            .await
    }

    async fn get_masks_in_group(&self, mask_group_id: i32) -> Result<Vec<Mask>, ServiceError> {
        // 마스크 그룹이 존재하는지 확인
        let mask_group = self.mask_group_repository
            .get_by_id(mask_group_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound(format!("Mask group with id {} not found", mask_group_id)))?;

        self.mask_group_repository
            .get_masks_in_group(mask_group_id)
            .await
    }

    async fn get_mask_group_stats(&self, annotation_id: Option<i32>) -> Result<MaskGroupStats, ServiceError> {
        // 어노테이션이 존재하는지 확인 (annotation_id가 있는 경우)
        if let Some(ann_id) = annotation_id {
            let annotation = self.annotation_repository
                .find_by_id(ann_id)
                .await
                .map_err(|e| ServiceError::DatabaseError(format!("Failed to find annotation: {}", e)))?
                .ok_or_else(|| ServiceError::NotFound(format!("Annotation with id {} not found", ann_id)))?;
        }

        self.mask_group_repository
            .get_stats(annotation_id)
            .await
    }

    async fn count_mask_groups(
        &self,
        annotation_id: Option<i32>,
        created_by: Option<i32>,
        modality: Option<String>,
        mask_type: Option<String>,
    ) -> Result<i64, ServiceError> {
        // 어노테이션이 존재하는지 확인 (annotation_id가 있는 경우)
        if let Some(ann_id) = annotation_id {
            let annotation = self.annotation_repository
                .find_by_id(ann_id)
                .await
                .map_err(|e| ServiceError::DatabaseError(format!("Failed to find annotation: {}", e)))?
                .ok_or_else(|| ServiceError::NotFound(format!("Annotation with id {} not found", ann_id)))?;
        }

        // 사용자가 존재하는지 확인 (created_by가 있는 경우)
        if let Some(user_id) = created_by {
            let user = self.user_repository
                .find_by_id(user_id)
                .await
                .map_err(|e| ServiceError::DatabaseError(format!("Failed to find user: {}", e)))?
                .ok_or_else(|| ServiceError::NotFound(format!("User with id {} not found", user_id)))?;
        }

        self.mask_group_repository
            .count(annotation_id, created_by, modality, mask_type)
            .await
    }

    async fn can_access_mask_group(&self, user_id: i32, mask_group_id: i32) -> Result<bool, ServiceError> {
        // 마스크 그룹 조회
        let mask_group = self.mask_group_repository
            .get_by_id(mask_group_id)
            .await?;

        let mask_group = match mask_group {
            Some(mg) => mg,
            None => return Ok(false),
        };

        // 어노테이션 조회
        let annotation = self.annotation_repository
            .find_by_id(mask_group.annotation_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(format!("Failed to find annotation: {}", e)))?;

        let annotation = match annotation {
            Some(ann) => ann,
            None => return Ok(false),
        };

        // 사용자가 어노테이션에 접근할 수 있는지 확인
        // 어노테이션 소유자이거나 마스크 그룹 생성자인 경우 접근 허용
        let annotation_owner = annotation.user_id == user_id;
        let group_creator = mask_group.created_by == Some(user_id);
        println!("DEBUG: can_access_mask_group - user_id={}, annotation.user_id={}, mask_group.created_by={:?}, annotation_owner={}, group_creator={}", 
                 user_id, annotation.user_id, mask_group.created_by, annotation_owner, group_creator);
        
        if annotation_owner || group_creator {
            Ok(true)
        } else {
            Err(ServiceError::Unauthorized("Access denied to mask group".to_string()))
        }
    }

    async fn can_create_mask_group(&self, user_id: i32, annotation_id: i32) -> Result<bool, ServiceError> {
        // 어노테이션 조회
        let annotation = self.annotation_repository
            .find_by_id(annotation_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(format!("Failed to find annotation: {}", e)))?
            .ok_or_else(|| ServiceError::NotFound(format!("Annotation with id {} not found", annotation_id)))?;

        // 사용자가 어노테이션에 접근할 수 있는지 확인
        // 여기서는 간단히 user_id로 확인 (실제로는 프로젝트 권한 확인 필요)
        Ok(annotation.user_id == user_id)
    }
}
