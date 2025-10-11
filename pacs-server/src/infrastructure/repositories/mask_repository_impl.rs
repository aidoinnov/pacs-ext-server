use async_trait::async_trait;
use sqlx::PgPool;
use chrono::{DateTime, Utc};
use crate::domain::entities::mask::{Mask, NewMask, UpdateMask, MaskStats};
use crate::domain::repositories::MaskRepository;
use crate::domain::services::ServiceError;

/// MaskRepository의 PostgreSQL 구현체
#[derive(Debug, Clone)]
pub struct MaskRepositoryImpl {
    pool: PgPool,
}

impl MaskRepositoryImpl {
    /// 새로운 MaskRepositoryImpl 생성
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MaskRepository for MaskRepositoryImpl {
    /// 마스크 생성
    async fn create(&self, new_mask: &NewMask) -> Result<Mask, ServiceError> {
        let result = sqlx::query!(
            r#"
            INSERT INTO annotation_mask (
                mask_group_id, slice_index, sop_instance_uid, label_name,
                file_path, mime_type, file_size, checksum, width, height
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id, mask_group_id, slice_index, sop_instance_uid, label_name,
                     file_path, mime_type, file_size, checksum, width, height, created_at, updated_at
            "#,
            new_mask.mask_group_id,
            new_mask.slice_index,
            new_mask.sop_instance_uid,
            new_mask.label_name,
            new_mask.file_path,
            new_mask.mime_type,
            new_mask.file_size,
            new_mask.checksum,
            new_mask.width,
            new_mask.height
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(format!("Failed to create mask: {}", e)))?;

        Ok(Mask {
            id: result.id,
            mask_group_id: result.mask_group_id,
            slice_index: result.slice_index,
            sop_instance_uid: result.sop_instance_uid,
            label_name: result.label_name,
            file_path: result.file_path,
            mime_type: result.mime_type,
            file_size: result.file_size,
            checksum: result.checksum,
            width: result.width,
            height: result.height,
            created_at: DateTime::from_naive_utc_and_offset(result.created_at.unwrap_or_default(), Utc),
            updated_at: DateTime::from_naive_utc_and_offset(result.updated_at.unwrap_or_default(), Utc),
        })
    }

    /// ID로 마스크 조회
    async fn get_by_id(&self, id: i32) -> Result<Option<Mask>, ServiceError> {
        let result = sqlx::query!(
            r#"
            SELECT id, mask_group_id, slice_index, sop_instance_uid, label_name,
                   file_path, mime_type, file_size, checksum, width, height, created_at, updated_at
            FROM annotation_mask
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(format!("Failed to get mask by id: {}", e)))?;

        Ok(result.map(|row| Mask {
            id: row.id,
            mask_group_id: row.mask_group_id,
            slice_index: row.slice_index,
            sop_instance_uid: row.sop_instance_uid,
            label_name: row.label_name,
            file_path: row.file_path,
            mime_type: row.mime_type,
            checksum: row.checksum,
            width: row.width,
            height: row.height,
            file_size: row.file_size,
            created_at: DateTime::from_naive_utc_and_offset(row.created_at.unwrap_or_default(), Utc),
            updated_at: DateTime::from_naive_utc_and_offset(row.updated_at.unwrap_or_default(), Utc),
        }))
    }

    /// 마스크 업데이트
    async fn update(&self, id: i32, update_mask: &UpdateMask) -> Result<Mask, ServiceError> {
        let result = sqlx::query!(
            r#"
            UPDATE annotation_mask
            SET slice_index = COALESCE($2, slice_index),
                sop_instance_uid = COALESCE($3, sop_instance_uid),
                label_name = COALESCE($4, label_name),
                file_path = COALESCE($5, file_path),
                mime_type = COALESCE($6, mime_type),
                file_size = COALESCE($7, file_size),
                checksum = COALESCE($8, checksum),
                width = COALESCE($9, width),
                height = COALESCE($10, height),
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $1
            RETURNING id, mask_group_id, slice_index, sop_instance_uid, label_name,
                     file_path, mime_type, file_size, checksum, width, height, created_at, updated_at
            "#,
            update_mask.id,
            update_mask.slice_index,
            update_mask.sop_instance_uid,
            update_mask.label_name,
            update_mask.file_path,
            update_mask.mime_type,
            update_mask.file_size,
            update_mask.checksum,
            update_mask.width,
            update_mask.height
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(format!("Failed to update mask: {}", e)))?;

        Ok(Mask {
            id: result.id,
            mask_group_id: result.mask_group_id,
            slice_index: result.slice_index,
            sop_instance_uid: result.sop_instance_uid,
            label_name: result.label_name,
            file_path: result.file_path,
            mime_type: result.mime_type,
            file_size: result.file_size,
            checksum: result.checksum,
            width: result.width,
            height: result.height,
            created_at: DateTime::from_naive_utc_and_offset(result.created_at.unwrap_or_default(), Utc),
            updated_at: DateTime::from_naive_utc_and_offset(result.updated_at.unwrap_or_default(), Utc),
        })
    }

    /// 마스크 삭제
    async fn delete(&self, id: i32) -> Result<(), ServiceError> {
        sqlx::query!(
            "DELETE FROM annotation_mask WHERE id = $1",
            id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(format!("Failed to delete mask: {}", e)))?;

        Ok(())
    }

    /// 마스크 목록 조회
    async fn list(
        &self,
        mask_group_id: Option<i32>,
        sop_instance_uid: Option<String>,
        label_name: Option<String>,
        mime_type: Option<String>,
        offset: Option<i64>,
        limit: Option<i64>,
    ) -> Result<Vec<Mask>, ServiceError> {
        let results = sqlx::query!(
            r#"
            SELECT id, mask_group_id, slice_index, sop_instance_uid, label_name,
                   file_path, mime_type, file_size, checksum, width, height, created_at, updated_at
            FROM annotation_mask
            WHERE ($1::INTEGER IS NULL OR mask_group_id = $1)
              AND ($2::TEXT IS NULL OR sop_instance_uid = $2)
              AND ($3::TEXT IS NULL OR label_name = $3)
              AND ($4::TEXT IS NULL OR mime_type = $4)
            ORDER BY slice_index ASC, created_at ASC
            OFFSET COALESCE($5, 0)
            LIMIT COALESCE($6, 50)
            "#,
            mask_group_id,
            sop_instance_uid,
            label_name,
            mime_type,
            offset.unwrap_or(0) as i32,
            limit.unwrap_or(50) as i32
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(format!("Failed to list masks: {}", e)))?;

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
            created_at: DateTime::from_naive_utc_and_offset(row.created_at.unwrap_or_default(), Utc),
            updated_at: DateTime::from_naive_utc_and_offset(row.updated_at.unwrap_or_default(), Utc),
        }).collect())
    }

    /// 마스크 통계 조회
    async fn get_stats(&self, mask_group_id: Option<i32>) -> Result<MaskStats, ServiceError> {
        let result = sqlx::query!(
            r#"
            SELECT 
                COUNT(*) as total_masks,
                COALESCE(SUM(file_size), 0) as total_size_bytes
            FROM annotation_mask
            WHERE ($1::INTEGER IS NULL OR mask_group_id = $1)
            "#,
            mask_group_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(format!("Failed to get mask stats: {}", e)))?;

        Ok(MaskStats {
            total_masks: result.total_masks.unwrap_or(0),
            total_size_bytes: result.total_size_bytes.unwrap_or_default().to_string().parse::<i64>().unwrap_or(0),
            mime_types: std::collections::HashMap::new(),
            label_names: std::collections::HashMap::new(),
            average_file_size: 0.0,
            largest_file_size: 0,
            smallest_file_size: 0,
        })
    }

    /// 마스크 개수 조회
    async fn count(
        &self,
        mask_group_id: Option<i32>,
        sop_instance_uid: Option<String>,
        label_name: Option<String>,
        mime_type: Option<String>,
    ) -> Result<i64, ServiceError> {
        let result = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*)
            FROM annotation_mask
            WHERE ($1::INTEGER IS NULL OR mask_group_id = $1)
              AND ($2::TEXT IS NULL OR sop_instance_uid = $2)
              AND ($3::TEXT IS NULL OR label_name = $3)
              AND ($4::TEXT IS NULL OR mime_type = $4)
            "#,
            mask_group_id,
            sop_instance_uid,
            label_name,
            mime_type
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(format!("Failed to count masks: {}", e)))?;

        Ok(result.unwrap_or(0))
    }
}