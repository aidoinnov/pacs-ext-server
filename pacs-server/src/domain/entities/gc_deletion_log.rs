
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;


/// GC 삭제 로그 엔티티
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct GcDeletionLog {
    pub id: i64,
    pub annotation_id: i32,
    pub snapshot_image_key: String,
    pub file_size: Option<i64>,
    pub deleted_at: DateTime<Utc>,
    pub dry_run: bool,
    pub status: String, // "success", "failed", "skipped"
    pub error_message: Option<String>
}

/// 새로운 GC 삭제 로그 생성용 DTO요
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewGcDeletionLog {
    pub annotation_id: i32,
    pub snapshot_image_key:String,
    pub file_size: Option<i64>,
    pub dry_run: bool,
    pub status: String,
    pub error_message: Option<String>
}