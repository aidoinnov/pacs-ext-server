use actix_web::{web, HttpResponse};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::domain::services::{SyncService, SyncStatus};
use crate::infrastructure::services::sync_worker::SyncServiceImpl;
use crate::infrastructure::services::sync_state::SyncState;

pub fn configure_routes(cfg: &mut web::ServiceConfig, state: Arc<RwLock<SyncState>>, svc: Arc<SyncServiceImpl>) {
    cfg.app_data(web::Data::from(state))
        .app_data(web::Data::from(svc))
        .service(
            web::scope("/sync")
                .route("/status", web::get().to(get_status))
                .route("/run", web::post().to(run_once))
                .route("/pause", web::post().to(pause))
                .route("/resume", web::post().to(resume))
                .route("/schedule", web::get().to(get_schedule))
                .route("/schedule", web::put().to(update_schedule))
                .route("/deps", web::get().to(deps_check)),
        );
}

async fn get_status(state: web::Data<Arc<RwLock<SyncState>>>, svc: web::Data<SyncServiceImpl>) -> HttpResponse {
    let st = svc.get_status().await;
    HttpResponse::Ok().json(serde_json::json!({
        "is_running": st.is_running,
        "last_run": st.last_run.map(|d| d.to_rfc3339()),
        "next_run": st.next_run.map(|d| d.to_rfc3339()),
        "interval_sec": st.interval_sec,
    }))
}

async fn run_once(svc: web::Data<SyncServiceImpl>) -> HttpResponse {
    let res = svc.run_once().await;
    HttpResponse::Ok().json(serde_json::json!({
        "success": res.success,
        "processed": res.processed,
        "duration_ms": res.duration_ms,
        "error": res.error,
    }))
}

async fn pause(svc: web::Data<SyncServiceImpl>) -> HttpResponse {
    svc.pause().await;
    HttpResponse::Ok().finish()
}

async fn resume(svc: web::Data<SyncServiceImpl>) -> HttpResponse {
    svc.resume().await;
    HttpResponse::Ok().finish()
}

async fn get_schedule(svc: web::Data<SyncServiceImpl>) -> HttpResponse {
    let st = svc.get_status().await;
    HttpResponse::Ok().json(serde_json::json!({ "interval_sec": st.interval_sec }))
}

#[derive(serde::Deserialize)]
struct UpdateSchedule { interval_sec: u64 }

async fn update_schedule(svc: web::Data<SyncServiceImpl>, body: web::Json<UpdateSchedule>) -> HttpResponse {
    svc.update_interval(body.interval_sec).await;
    HttpResponse::Ok().finish()
}

async fn deps_check(req: actix_web::HttpRequest) -> HttpResponse {
    // Detect presence under multiple plausible type registrations
    let has_state_arc = req.app_data::<web::Data<Arc<RwLock<SyncState>>>>().is_some();
    let has_state_lock = req.app_data::<web::Data<RwLock<SyncState>>>().is_some();
    let has_state = has_state_arc || has_state_lock;

    let has_svc = req.app_data::<web::Data<SyncServiceImpl>>().is_some();

    HttpResponse::Ok().json(serde_json::json!({"state": has_state, "svc": has_svc}))
}


