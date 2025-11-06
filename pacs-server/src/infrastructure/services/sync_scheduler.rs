use chrono::Utc;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration, Instant};

use crate::domain::services::SyncService;
use crate::infrastructure::services::sync_state::SyncState;

pub async fn run_scheduler(state: Arc<RwLock<SyncState>>, sync_service: Arc<dyn SyncService>) {
    loop {
        let interval = { state.read().await.interval_sec };
        {
            let mut s = state.write().await;
            s.next_run = Some(Utc::now() + chrono::Duration::seconds(interval as i64));
        }

        sleep(Duration::from_secs(interval)).await;

        let paused = { state.read().await.paused };
        if paused {
            continue;
        }

        {
            let mut s = state.write().await;
            s.is_running = true;
        }
        let started = Instant::now();
        let result = sync_service.run_once().await;
        let duration_ms = started.elapsed().as_millis();

        {
            let mut s = state.write().await;
            s.last_run = Some(Utc::now());
            s.is_running = false;
        }

        // TODO: persist result to history if needed
        let _ = (result, duration_ms);
    }
}
