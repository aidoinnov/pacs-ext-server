use async_trait::async_trait;
use chrono::NaiveDateTime;
use crate::domain::entities::{AccessLog, NewAccessLog};

#[async_trait]
pub trait AccessLogRepository: Send + Sync {
    async fn create(&self, new_log: NewAccessLog) -> Result<AccessLog, sqlx::Error>;
    async fn find_by_user_id(&self, user_id: i32, limit: i64) -> Result<Vec<AccessLog>, sqlx::Error>;
    async fn find_by_project_id(&self, project_id: i32, limit: i64) -> Result<Vec<AccessLog>, sqlx::Error>;
    async fn find_by_study_uid(&self, study_uid: &str, limit: i64) -> Result<Vec<AccessLog>, sqlx::Error>;
    async fn find_by_time_range(&self, start: NaiveDateTime, end: NaiveDateTime) -> Result<Vec<AccessLog>, sqlx::Error>;
    async fn count_by_user_id(&self, user_id: i32) -> Result<i64, sqlx::Error>;
}
