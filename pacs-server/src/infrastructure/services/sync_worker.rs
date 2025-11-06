use crate::domain::services::{SyncResult, SyncService, SyncStatus};
use crate::infrastructure::config::{Dcm4cheeDbConfig, Settings};
use crate::infrastructure::services::sync_state::SyncState;
use async_trait::async_trait;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    PgPool, Row,
};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct SyncServiceImpl {
    pub rbac_pool: PgPool,
    pub dcm4chee_pool: PgPool,
    pub state: Arc<RwLock<SyncState>>,
    pub default_project_id: i32,
}

impl SyncServiceImpl {
    pub async fn new(
        settings: &Settings,
        rbac_pool: PgPool,
        state: Arc<RwLock<SyncState>>,
    ) -> Result<Self, String> {
        let db_cfg: &Dcm4cheeDbConfig = settings
            .dcm4chee
            .db
            .as_ref()
            .ok_or_else(|| "DCM4CHEE DB config missing".to_string())?;
        let mut opts = PgConnectOptions::new();
        opts = opts.host(&db_cfg.host);
        opts = opts.port(db_cfg.port);
        opts = opts.username(&db_cfg.username);
        opts = opts.password(&db_cfg.password);
        opts = opts.database(&db_cfg.database);
        let dcm_pool = PgPoolOptions::new()
            .max_connections(3)
            .connect_with(opts)
            .await
            .map_err(|e| format!("Failed to connect to DCM4CHEE DB: {}", e))?;
        let default_project_id = settings
            .sync
            .as_ref()
            .and_then(|s| s.default_project_id)
            .unwrap_or(1);
        Ok(Self {
            rbac_pool: rbac_pool,
            dcm4chee_pool: dcm_pool,
            state,
            default_project_id,
        })
    }

    async fn sync_studies(
        &self,
        last_run: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<usize, String> {
        // 실제 스키마 반영: patient_fk 통해 patient.patient_id 조인, study_date는 varchar(YYYYMMDD)
        let rows = if let Some(ts) = last_run {
            sqlx::query(
                r#"SELECT st.study_iuid, st.study_desc, NULL::text AS patient_id, st.study_date, st.updated_time
                   FROM study st
                   LEFT JOIN patient pt ON st.patient_fk = pt.pk
                   WHERE st.updated_time > $1
                   ORDER BY st.updated_time ASC
                   LIMIT 500"#,
            )
            .bind(ts)
            .fetch_all(&self.dcm4chee_pool)
            .await
            .map_err(|e| format!("dcm4chee select study failed: {}", e))?
        } else {
            sqlx::query(
                r#"SELECT st.study_iuid, st.study_desc, NULL::text AS patient_id, st.study_date, st.updated_time
                   FROM study st
                   LEFT JOIN patient pt ON st.patient_fk = pt.pk
                   ORDER BY st.updated_time DESC
                   LIMIT 500"#
            )
            .fetch_all(&self.dcm4chee_pool)
            .await
            .map_err(|e| format!("dcm4chee select study failed: {}", e))?
        };

        let mut processed = 0usize;
        for r in rows {
            let uid: String = r.try_get("study_iuid").unwrap_or_default();
            let desc: Option<String> = r.try_get("study_desc").ok();
            let pid: Option<String> = r.try_get("patient_id").ok();
            let sdate_raw: Option<String> = r.try_get("study_date").ok();

            // upsert into project_data_study
            let _ = sqlx::query(
                r#"INSERT INTO project_data_study (project_id, study_uid, study_description, patient_id, study_date)
                    VALUES ($1, $2, $3, $4, to_date($5, 'YYYYMMDD'))
                    ON CONFLICT (project_id, study_uid)
                    DO UPDATE SET study_description = EXCLUDED.study_description,
                                  patient_id = EXCLUDED.patient_id,
                                  study_date = EXCLUDED.study_date"#,
            )
            .bind(self.default_project_id)
            .bind(uid)
            .bind(desc)
            .bind(pid)
            .bind(sdate_raw)
            .execute(&self.rbac_pool)
            .await
            .map_err(|e| format!("rbac upsert study failed: {}", e))?;
            processed += 1;
        }
        Ok(processed)
    }

    async fn sync_series(
        &self,
        last_run: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<usize, String> {
        // 실제 스키마 반영: series.study_fk → study.pk, series.modality varchar
        let rows = if let Some(ts) = last_run {
            sqlx::query(
                r#"SELECT se.series_iuid, se.series_desc, se.modality, st.study_iuid, se.updated_time
                   FROM series se
                   JOIN study st ON se.study_fk = st.pk
                   WHERE se.updated_time > $1
                   ORDER BY se.updated_time ASC
                   LIMIT 1000"#,
            )
            .bind(ts)
            .fetch_all(&self.dcm4chee_pool)
            .await
            .map_err(|e| format!("dcm4chee select series failed: {}", e))?
        } else {
            sqlx::query(
                r#"SELECT se.series_iuid, se.series_desc, se.modality, st.study_iuid, se.updated_time
                   FROM series se
                   JOIN study st ON se.study_fk = st.pk
                   ORDER BY se.updated_time DESC
                   LIMIT 1000"#
            )
            .fetch_all(&self.dcm4chee_pool)
            .await
            .map_err(|e| format!("dcm4chee select series failed: {}", e))?
        };

        let mut processed = 0usize;
        for r in rows {
            let study_uid: String = r.try_get("study_iuid").unwrap_or_default();
            let series_uid: String = r.try_get("series_iuid").unwrap_or_default();
            let series_desc: Option<String> = r.try_get("series_desc").ok();
            let modality: Option<String> = r.try_get("modality").ok();

            // find study id
            let study_id = sqlx::query_scalar::<_, i32>(
                r#"SELECT id FROM project_data_study WHERE project_id = $1 AND study_uid = $2"#,
            )
            .bind(self.default_project_id)
            .bind(study_uid)
            .fetch_optional(&self.rbac_pool)
            .await
            .map_err(|e| format!("rbac select study id failed: {}", e))?;

            if let Some(sid) = study_id {
                let _ = sqlx::query(
                    r#"INSERT INTO project_data_series (study_id, series_uid, series_description, modality)
                        VALUES ($1, $2, $3, $4)
                        ON CONFLICT (study_id, series_uid)
                        DO UPDATE SET series_description = EXCLUDED.series_description,
                                      modality = EXCLUDED.modality"#,
                )
                .bind(sid)
                .bind(series_uid)
                .bind(series_desc)
                .bind(modality)
                .execute(&self.rbac_pool)
                .await
                .map_err(|e| format!("rbac upsert series failed: {}", e))?;
                processed += 1;
            }
        }
        Ok(processed)
    }

    async fn sync_instances(
        &self,
        last_run: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<usize, String> {
        // 실제 스키마 반영: instance(series_fk→series.pk), content_date/time varchar
        let rows = if let Some(ts) = last_run {
            sqlx::query(
                r#"SELECT i.sop_iuid, i.sop_cuid, i.inst_no, i.content_date, i.content_time, se.series_iuid, i.updated_time
                   FROM instance i
                   JOIN series se ON i.series_fk = se.pk
                   WHERE i.updated_time > $1
                   ORDER BY i.updated_time ASC
                   LIMIT 2000"#,
            )
            .bind(ts)
            .fetch_all(&self.dcm4chee_pool)
            .await
            .map_err(|e| format!("dcm4chee select instance failed: {}", e))?
        } else {
            sqlx::query(
                r#"SELECT i.sop_iuid, i.sop_cuid, i.inst_no, i.content_date, i.content_time, se.series_iuid, i.updated_time
                   FROM instance i
                   JOIN series se ON i.series_fk = se.pk
                   ORDER BY i.updated_time DESC
                   LIMIT 2000"#
            )
            .fetch_all(&self.dcm4chee_pool)
            .await
            .map_err(|e| format!("dcm4chee select instance failed: {}", e))?
        };

        let mut processed = 0usize;
        for r in rows {
            let series_uid: String = r.try_get("series_iuid").unwrap_or_default();
            let instance_uid: String = r.try_get("sop_iuid").unwrap_or_default();
            let sop_class_uid: Option<String> = r.try_get("sop_cuid").ok();
            let instance_number: Option<i32> = r.try_get("inst_no").ok();
            let content_date: Option<String> = r.try_get("content_date").ok();
            let content_time: Option<String> = r.try_get("content_time").ok();

            // find series id
            let series_id = sqlx::query_scalar::<_, i32>(
                r#"SELECT pds.id
                    FROM project_data_series pds
                    JOIN project_data_study pdt ON pds.study_id = pdt.id
                   WHERE pdt.project_id = $1 AND pds.series_uid = $2"#,
            )
            .bind(self.default_project_id)
            .bind(series_uid)
            .fetch_optional(&self.rbac_pool)
            .await
            .map_err(|e| format!("rbac select series id failed: {}", e))?;

            if let Some(sid) = series_id {
                let _ = sqlx::query(
                    r#"INSERT INTO project_data_instance (series_id, instance_uid, sop_class_uid, instance_number, content_date, content_time)
                        VALUES ($1, $2, $3, $4, $5, $6)
                        ON CONFLICT (series_id, instance_uid)
                        DO UPDATE SET sop_class_uid = EXCLUDED.sop_class_uid,
                                      instance_number = EXCLUDED.instance_number,
                                      content_date = EXCLUDED.content_date,
                                      content_time = EXCLUDED.content_time"#,
                )
                .bind(sid)
                .bind(instance_uid)
                .bind(sop_class_uid)
                .bind(instance_number)
                .bind(content_date)
                .bind(content_time)
                .execute(&self.rbac_pool)
                .await
                .map_err(|e| format!("rbac upsert instance failed: {}", e))?;
                processed += 1;
            }
        }
        Ok(processed)
    }
}

#[async_trait]
impl SyncService for SyncServiceImpl {
    async fn run_once(&self) -> SyncResult {
        // 간단한 델타 동기화: last_run 기준으로 변경분 조회 후 upsert
        let last_run_opt = { self.state.read().await.last_run };
        let mut total_processed = 0usize;

        match self.sync_studies(last_run_opt).await {
            Ok(n) => total_processed += n,
            Err(e) => {
                return SyncResult {
                    success: false,
                    processed: total_processed,
                    duration_ms: 0,
                    error: Some(format!("studies sync failed: {}", e)),
                }
            }
        }
        match self.sync_series(last_run_opt).await {
            Ok(n) => total_processed += n,
            Err(e) => {
                return SyncResult {
                    success: false,
                    processed: total_processed,
                    duration_ms: 0,
                    error: Some(format!("series sync failed: {}", e)),
                }
            }
        }
        match self.sync_instances(last_run_opt).await {
            Ok(n) => total_processed += n,
            Err(e) => {
                return SyncResult {
                    success: false,
                    processed: total_processed,
                    duration_ms: 0,
                    error: Some(format!("instances sync failed: {}", e)),
                }
            }
        }

        SyncResult {
            success: true,
            processed: total_processed,
            duration_ms: 0,
            error: None,
        }
    }

    async fn get_status(&self) -> SyncStatus {
        let s = self.state.read().await;
        SyncStatus {
            is_running: s.is_running,
            last_run: s.last_run,
            next_run: s.next_run,
            interval_sec: s.interval_sec,
        }
    }

    async fn pause(&self) {
        let mut s = self.state.write().await;
        s.paused = true;
    }

    async fn resume(&self) {
        let mut s = self.state.write().await;
        s.paused = false;
    }

    async fn update_interval(&self, interval_sec: u64) {
        let mut s = self.state.write().await;
        s.interval_sec = interval_sec;
    }
}
