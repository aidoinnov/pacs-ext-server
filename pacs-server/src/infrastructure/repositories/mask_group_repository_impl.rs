use async_trait::async_trait;
use sqlx::{PgPool, Row};
use crate::domain::entities::{MaskGroup, NewMaskGroup, UpdateMaskGroup, MaskGroupStats, Mask};
use crate::domain::repositories::MaskGroupRepository;
use crate::domain::ServiceError;
use std::collections::HashMap;

#[derive(Clone)]
pub struct MaskGroupRepositoryImpl {
    pool: PgPool,
}

impl MaskGroupRepositoryImpl {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MaskGroupRepository for MaskGroupRepositoryImpl {
    async fn create(&self, mask_group: &NewMaskGroup) -> Result<MaskGroup, ServiceError> {
        let result = sqlx::query_as::<_, MaskGroup>(
            "INSERT INTO annotation_mask_group 
             (annotation_id, group_name, model_name, version, modality, slice_count, mask_type, description, created_by)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
             RETURNING id, annotation_id, group_name, model_name, version, modality, slice_count, mask_type, description, created_by, created_at, updated_at"
        )
        .bind(mask_group.annotation_id)
        .bind(&mask_group.group_name)
        .bind(&mask_group.model_name)
        .bind(&mask_group.version)
        .bind(&mask_group.modality)
        .bind(mask_group.slice_count)
        .bind(&mask_group.mask_type)
        .bind(&mask_group.description)
        .bind(mask_group.created_by)
        .fetch_one(&self.pool)
        .await;

        match result {
            Ok(mask_group) => Ok(mask_group),
            Err(e) => {
                eprintln!("Failed to create mask group: {}", e);
                Err(ServiceError::DatabaseError(format!("Failed to create mask group: {}", e)))
            }
        }
    }

    async fn get_by_id(&self, id: i32) -> Result<Option<MaskGroup>, ServiceError> {
        let result = sqlx::query_as::<_, MaskGroup>(
            "SELECT id, annotation_id, group_name, model_name, version, modality, slice_count, mask_type, description, created_by, created_at, updated_at
             FROM annotation_mask_group
             WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await;

        match result {
            Ok(mask_group) => Ok(mask_group),
            Err(e) => {
                eprintln!("Failed to get mask group by id {}: {}", id, e);
                Err(ServiceError::DatabaseError(format!("Failed to get mask group: {}", e)))
            }
        }
    }

    async fn update(&self, id: i32, update_mask_group: &UpdateMaskGroup) -> Result<MaskGroup, ServiceError> {
        // For now, implement a simple update that only updates non-None fields
        // This is a simplified version - in production, you'd want to build dynamic queries
        
        let result = sqlx::query_as::<_, MaskGroup>(
            "UPDATE annotation_mask_group 
             SET group_name = COALESCE($2, group_name),
                 model_name = COALESCE($3, model_name),
                 version = COALESCE($4, version),
                 modality = COALESCE($5, modality),
                 slice_count = COALESCE($6, slice_count),
                 mask_type = COALESCE($7, mask_type),
                 description = COALESCE($8, description),
                 updated_at = CURRENT_TIMESTAMP
             WHERE id = $1
             RETURNING id, annotation_id, group_name, model_name, version, modality, slice_count, mask_type, description, created_by, created_at, updated_at"
        )
        .bind(id)
        .bind(&update_mask_group.group_name)
        .bind(&update_mask_group.model_name)
        .bind(&update_mask_group.version)
        .bind(&update_mask_group.modality)
        .bind(update_mask_group.slice_count)
        .bind(&update_mask_group.mask_type)
        .bind(&update_mask_group.description)
        .fetch_one(&self.pool)
        .await;

        match result {
            Ok(mask_group) => Ok(mask_group),
            Err(e) => {
                eprintln!("Failed to update mask group {}: {}", id, e);
                Err(ServiceError::DatabaseError(format!("Failed to update mask group: {}", e)))
            }
        }
    }

    async fn delete(&self, id: i32) -> Result<(), ServiceError> {
        let result = sqlx::query("DELETE FROM annotation_mask_group WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await;

        match result {
            Ok(result) => {
                if result.rows_affected() == 0 {
                    Err(ServiceError::NotFound(format!("Mask group with ID {} not found", id)))
                } else {
                    Ok(())
                }
            }
            Err(e) => {
                eprintln!("Failed to delete mask group {}: {}", id, e);
                Err(ServiceError::DatabaseError(format!("Failed to delete mask group: {}", e)))
            }
        }
    }

    async fn list(
        &self,
        annotation_id: Option<i32>,
        created_by: Option<i32>,
        modality: Option<String>,
        mask_type: Option<String>,
        offset: Option<i64>,
        limit: Option<i64>,
    ) -> Result<Vec<MaskGroup>, ServiceError> {
        let query = sqlx::query_as::<_, MaskGroup>(
            "SELECT id, annotation_id, group_name, model_name, version, modality, slice_count, mask_type, description, created_by, created_at, updated_at 
             FROM annotation_mask_group 
             WHERE ($1::int IS NULL OR annotation_id = $1)
               AND ($2::int IS NULL OR created_by = $2)
               AND ($3::text IS NULL OR modality = $3)
               AND ($4::text IS NULL OR mask_type = $4)
             ORDER BY created_at DESC
             LIMIT COALESCE($5, 50) OFFSET COALESCE($6, 0)"
        )
        .bind(annotation_id)
        .bind(created_by)
        .bind(&modality)
        .bind(&mask_type)
        .bind(limit)
        .bind(offset);

        let result = query.fetch_all(&self.pool).await;

        match result {
            Ok(mask_groups) => Ok(mask_groups),
            Err(e) => {
                eprintln!("Failed to list mask groups: {}", e);
                Err(ServiceError::DatabaseError(format!("Failed to list mask groups: {}", e)))
            }
        }
    }

    async fn get_masks_in_group(&self, _mask_group_id: i32) -> Result<Vec<Mask>, ServiceError> {
        // For now, return an empty vector since we don't have the Mask entity properly set up
        // This should be implemented when the Mask entity is properly configured
        Ok(Vec::new())
    }

    async fn get_stats(&self, annotation_id: Option<i32>) -> Result<MaskGroupStats, ServiceError> {
        let query = if annotation_id.is_some() {
            "SELECT 
                COUNT(*) as total_groups,
                modality,
                mask_type
             FROM annotation_mask_group 
             WHERE annotation_id = $1
             GROUP BY modality, mask_type"
        } else {
            "SELECT 
                COUNT(*) as total_groups,
                modality,
                mask_type
             FROM annotation_mask_group 
             GROUP BY modality, mask_type"
        };

        let result = if annotation_id.is_some() {
            sqlx::query(query)
                .bind(annotation_id.unwrap())
                .fetch_all(&self.pool)
                .await
        } else {
            sqlx::query(query)
                .fetch_all(&self.pool)
                .await
        };

        match result {
            Ok(rows) => {
                let mut stats = MaskGroupStats::new();
                let mut modalities = HashMap::new();
                let mut mask_types = HashMap::new();

                for row in rows {
                    let total_groups: i64 = row.get("total_groups");
                    let modality: Option<String> = row.get("modality");
                    let mask_type: Option<String> = row.get("mask_type");

                    stats.total_groups += total_groups;

                    if let Some(modality) = modality {
                        *modalities.entry(modality).or_insert(0) += total_groups;
                    }

                    if let Some(mask_type) = mask_type {
                        *mask_types.entry(mask_type).or_insert(0) += total_groups;
                    }
                }

                stats.modalities = modalities;
                stats.mask_types = mask_types;
                Ok(stats)
            }
            Err(e) => {
                eprintln!("Failed to get mask group stats: {}", e);
                Err(ServiceError::DatabaseError(format!("Failed to get mask group stats: {}", e)))
            }
        }
    }

    async fn count(
        &self,
        annotation_id: Option<i32>,
        created_by: Option<i32>,
        modality: Option<String>,
        mask_type: Option<String>,
    ) -> Result<i64, ServiceError> {
        let result = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) 
             FROM annotation_mask_group 
             WHERE ($1::int IS NULL OR annotation_id = $1)
               AND ($2::int IS NULL OR created_by = $2)
               AND ($3::text IS NULL OR modality = $3)
               AND ($4::text IS NULL OR mask_type = $4)"
        )
        .bind(annotation_id)
        .bind(created_by)
        .bind(&modality)
        .bind(&mask_type)
        .fetch_one(&self.pool)
        .await;

        match result {
            Ok(count) => Ok(count),
            Err(e) => {
                eprintln!("Failed to count mask groups: {}", e);
                Err(ServiceError::DatabaseError(format!("Failed to count mask groups: {}", e)))
            }
        }
    }
}