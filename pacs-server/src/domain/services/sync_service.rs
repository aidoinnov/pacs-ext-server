use async_trait::async_trait;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct SyncStatus {
    pub is_running: bool,
    pub last_run: Option<DateTime<Utc>>,
    pub next_run: Option<DateTime<Utc>>,
    pub interval_sec: u64,
}

#[derive(Debug, Clone)]
pub struct SyncResult {
    pub success: bool,
    pub processed: usize,
    pub duration_ms: u128,
    pub error: Option<String>,
}

#[async_trait]
pub trait SyncService: Send + Sync {
    async fn run_once(&self) -> SyncResult;
    async fn get_status(&self) -> SyncStatus;
    async fn pause(&self);
    async fn resume(&self);
    async fn update_interval(&self, interval_sec: u64);
}
