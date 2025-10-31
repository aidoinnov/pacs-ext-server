use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct SyncState {
    pub is_running: bool,
    pub paused: bool,
    pub last_run: Option<DateTime<Utc>>,
    pub next_run: Option<DateTime<Utc>>,
    pub interval_sec: u64,
}

impl SyncState {
    pub fn new(interval_sec: u64) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self {
            is_running: false,
            paused: false,
            last_run: None,
            next_run: None,
            interval_sec,
        }))
    }
}
