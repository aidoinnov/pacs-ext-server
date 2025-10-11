use async_trait::async_trait;
use sqlx::PgPool;
use chrono::{DateTime, Utc};
use crate::domain::entities::mask_group::{MaskGroup, NewMaskGroup, UpdateMaskGroup, MaskGroupStats};
use crate::domain::entities::mask::Mask;
use crate::domain::repositories::MaskGroupRepository;
use crate::domain::ServiceError;

/// MaskGroupRepository의 PostgreSQL 구현체
#[derive(Debug, Clone)]
pub struct MaskGroupRepositoryImpl {
    pool: PgPool,
}

impl MaskGroupRepositoryImpl {
    /// 새로운 MaskGroupRepositoryImpl 생성
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MaskGroupRepository for MaskGroupRepositoryImpl {
    /// 마스크 그룹 생성
    async fn create(&self, new_mask_group: &NewMaskGroup) -> Result<MaskGroup, ServiceError> {
        let result = sqlx::query!(
            r#"
            INSERT INTO annotation_mask_group (
                annotation_id, group_name, model_name, version, modality,
                slice_count, mask_type, description, created_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, annotation_id, group_name, model_name, version, modality,
                     slice_count, mask_type, description, created_by, created_at, updated_at
            "#,
            new_mask_group.annotation_id,
            new_mask_group.group_name,
            new_mask_group.model_name,
            new_mask_group.version,
            new_mask_group.modality,
            new_mask_group.slice_count,
            new_mask_group.mask_type,
            new_mask_group.description,
            new_mask_group.created_by
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(format!("Failed to create mask group: {}", e)))?;

        Ok(MaskGroup {
            id: result.id,
            annotation_id: result.annotation_id,
            group_name: result.group_name,
            model_name: result.model_name,
            version: result.version,
            modality: result.modality,
            slice_count: result.slice_count,
            mask_type: result.mask_type,
            description: result.description,
            created_by: result.created_by,
            created_at: result.created_at.unwrap_or_default(),
            updated_at: result.updated_at.unwrap_or_default(),
        })
    }

    /// ID로 마스크 그룹 조회
    async fn get_by_id(&self, id: i32) -> Result<Option<MaskGroup>, ServiceError> {
        let result = sqlx::query!(
            r#"
            SELECT id, annotation_id, group_name, model_name, version, modality,
                   slice_count, mask_type, description, created_by, created_at, updated_at
            FROM annotation_mask_group
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(format!("Failed to get mask group by id: {}", e)))?;

        Ok(result.map(|row| MaskGroup {
            id: row.id,
            annotation_id: row.annotation_id,
            group_name: row.group_name,
            model_name: row.model_name,
            version: row.version,
            modality: row.modality,
            slice_count: row.slice_count,
            mask_type: row.mask_type,
            description: row.description,
            created_by: row.created_by,
            created_at: row.created_at.unwrap_or_default(),
            updated_at: row.updated_at.unwrap_or_default(),
        }))
    }

    /// 마스크 그룹 업데이트
    async fn update(&self, id: i32, update_mask_group: &UpdateMaskGroup) -> Result<MaskGroup, ServiceError> {
        let result = sqlx::query!(
            r#"
            UPDATE annotation_mask_group
            SET group_name = COALESCE($2, group_name),
                model_name = COALESCE($3, model_name),
                version = COALESCE($4, version),
                modality = COALESCE($5, modality),
                slice_count = COALESCE($6, slice_count),
                mask_type = COALESCE($7, mask_type),
                description = COALESCE($8, description),
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $1
            RETURNING id, annotation_id, group_name, model_name, version, modality,
                     slice_count, mask_type, description, created_by, created_at, updated_at
            "#,
            update_mask_group.id,
            update_mask_group.group_name,
            update_mask_group.model_name,
            update_mask_group.version,
            update_mask_group.modality,
            update_mask_group.slice_count,
            update_mask_group.mask_type,
            update_mask_group.description
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(format!("Failed to update mask group: {}", e)))?;

        Ok(MaskGroup {
            id: result.id,
            annotation_id: result.annotation_id,
            group_name: result.group_name,
            model_name: result.model_name,
            version: result.version,
            modality: result.modality,
            slice_count: result.slice_count,
            mask_type: result.mask_type,
            description: result.description,
            created_by: result.created_by,
            created_at: result.created_at.unwrap_or_default(),
            updated_at: result.updated_at.unwrap_or_default(),
        })
    }

    /// 마스크 그룹 삭제
    async fn delete(&self, id: i32) -> Result<(), ServiceError> {
        sqlx::query!(
            "DELETE FROM annotation_mask_group WHERE id = $1",
            id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(format!("Failed to delete mask group: {}", e)))?;

        Ok(())
    }

    /// 마스크 그룹 목록 조회
    async fn list(
        &self,
        annotation_id: Option<i32>,
        created_by: Option<i32>,
        modality: Option<String>,
        mask_type: Option<String>,
        offset: Option<i64>,
        limit: Option<i64>,
    ) -> Result<Vec<MaskGroup>, ServiceError> {
        let results = sqlx::query!(
            r#"
            SELECT id, annotation_id, group_name, model_name, version, modality,
                   slice_count, mask_type, description, created_by, created_at, updated_at
            FROM annotation_mask_group
            WHERE ($1::INTEGER IS NULL OR annotation_id = $1)
              AND ($2::INTEGER IS NULL OR created_by = $2)
              AND ($3::TEXT IS NULL OR modality = $3)
              AND ($4::TEXT IS NULL OR mask_type = $4)
            ORDER BY created_at DESC
            OFFSET COALESCE($5, 0)
            LIMIT COALESCE($6, 50)
            "#,
            annotation_id,
            created_by,
            modality,
            mask_type,
            offset.unwrap_or(0) as i32,
            limit.unwrap_or(50) as i32
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(format!("Failed to list mask groups: {}", e)))?;

        Ok(results.into_iter().map(|row| MaskGroup {
            id: row.id,
            annotation_id: row.annotation_id,
            group_name: row.group_name,
            model_name: row.model_name,
            version: row.version,
            modality: row.modality,
            slice_count: row.slice_count,
            mask_type: row.mask_type,
            description: row.description,
            created_by: row.created_by,
            created_at: row.created_at.unwrap_or_default(),
            updated_at: row.updated_at.unwrap_or_default(),
        }).collect())
    }

    /// 마스크 그룹 내의 마스크들 조회
    async fn get_masks_in_group(&self, mask_group_id: i32) -> Result<Vec<Mask>, ServiceError> {
        let results = sqlx::query!(
            r#"
            SELECT id, mask_group_id, slice_index, sop_instance_uid, label_name,
                   file_path, mime_type, file_size, checksum, width, height, created_at, updated_at
            FROM annotation_mask
            WHERE mask_group_id = $1
            ORDER BY slice_index ASC, created_at ASC
            "#,
            mask_group_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(format!("Failed to get masks in group: {}", e)))?;

        Ok(results.into_iter().map(|row| Mask {
            id: row.id,
            mask_group_id: row.mask_group_id,
            slice_index: row.slice_index,
            sop_instance_uid: row.sop_instance_uid,
            label_name: row.label_name,
            file_path: row.file_path,
            mime_type: row.mime_type,
            file_size: row.file_size,
            checksum: row.checksum,
            width: row.width,
            height: row.height,
            created_at: row.created_at.unwrap_or_default(),
            updated_at: row.updated_at.unwrap_or_default(),
        }).collect())
    }

    /// 마스크 그룹 통계 조회
    async fn get_stats(&self, annotation_id: Option<i32>) -> Result<MaskGroupStats, ServiceError> {
        let result = sqlx::query!(
            r#"
            SELECT 
                COUNT(DISTINCT amg.id) as total_groups,
                COUNT(am.id) as total_masks,
                COALESCE(SUM(am.file_size), 0) as total_size_bytes
            FROM annotation_mask_group amg
            LEFT JOIN annotation_mask am ON amg.id = am.mask_group_id
            WHERE ($1::INTEGER IS NULL OR amg.annotation_id = $1)
            "#,
            annotation_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(format!("Failed to get mask group stats: {}", e)))?;

        Ok(MaskGroupStats {
            total_groups: result.total_groups.unwrap_or(0),
            total_masks: result.total_masks.unwrap_or(0),
            total_size_bytes: result.total_size_bytes.unwrap_or_default().to_string().parse::<i64>().unwrap_or(0),
            modalities: std::collections::HashMap::new(),
            mask_types: std::collections::HashMap::new(),
        })
    }

    /// 마스크 그룹 개수 조회
    async fn count(
        &self,
        annotation_id: Option<i32>,
        created_by: Option<i32>,
        modality: Option<String>,
        mask_type: Option<String>,
    ) -> Result<i64, ServiceError> {
        let result = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*)
            FROM annotation_mask_group
            WHERE ($1::INTEGER IS NULL OR annotation_id = $1)
              AND ($2::INTEGER IS NULL OR created_by = $2)
              AND ($3::TEXT IS NULL OR modality = $3)
              AND ($4::TEXT IS NULL OR mask_type = $4)
            "#,
            annotation_id,
            created_by,
            modality,
            mask_type
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(format!("Failed to count mask groups: {}", e)))?;

        Ok(result.unwrap_or(0))
    }
}