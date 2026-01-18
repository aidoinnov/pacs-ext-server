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
        // patient_fk 통해 patient_id, patient_name 조인 (dcm4chee 정규화 구조)
        let rows = if let Some(ts) = last_run {
            sqlx::query(
                r#"SELECT
                       st.study_iuid,
                       st.study_desc,
                       pid.pat_id AS patient_id,
                       pn.alphabetic_name AS patient_name,
                       st.study_date,
                       st.updated_time
                   FROM study st
                   LEFT JOIN patient pt ON st.patient_fk = pt.pk
                   LEFT JOIN patient_id pid ON pt.patient_id_fk = pid.pk
                   LEFT JOIN person_name pn ON pt.pat_name_fk = pn.pk
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
                r#"SELECT
                       st.study_iuid,
                       st.study_desc,
                       pid.pat_id AS patient_id,
                       pn.alphabetic_name AS patient_name,
                       st.study_date,
                       st.updated_time
                   FROM study st
                   LEFT JOIN patient pt ON st.patient_fk = pt.pk
                   LEFT JOIN patient_id pid ON pt.patient_id_fk = pid.pk
                   LEFT JOIN person_name pn ON pt.pat_name_fk = pn.pk
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
            let pname: Option<String> = r.try_get("patient_name").ok();
            let sdate_raw: Option<String> = r.try_get("study_date").ok();

            // upsert into project_data_study (patient_id, patient_name 포함)
            let _study_id: i32 = sqlx::query_scalar(
                r#"INSERT INTO project_data_study (study_uid, study_description, patient_id, patient_name, study_date)
                    VALUES ($1, $2, $3, $4, to_date($5, 'YYYYMMDD'))
                    ON CONFLICT (study_uid)
                    DO UPDATE SET study_description = COALESCE(EXCLUDED.study_description, project_data_study.study_description),
                                  patient_id = COALESCE(EXCLUDED.patient_id, project_data_study.patient_id),
                                  patient_name = COALESCE(EXCLUDED.patient_name, project_data_study.patient_name),
                                  study_date = COALESCE(EXCLUDED.study_date, project_data_study.study_date),
                                  updated_at = CURRENT_TIMESTAMP
                    RETURNING id"#,
            )
            .bind(&uid)
            .bind(&desc)
            .bind(&pid)
            .bind(&pname)
            .bind(sdate_raw.unwrap_or_default())
            .fetch_one(&self.rbac_pool)
            .await
            .map_err(|e| format!("rbac upsert study failed: {}", e))?;

            // ❌ 프로젝트 할당 제거: 사용자가 수동으로 할당해야 함
            // ❌ Subject 자동 생성 제거: assign_study_to_project API에서만 생성

            processed += 1;
        }
        Ok(processed)
    }

    async fn cleanup_missing_studies(&self) -> Result<usize, String> {
        // PACS에 있는 모든 study_uid 조회
        let pacs_study_uids: Vec<String> = sqlx::query_scalar::<_, String>(
            r#"SELECT DISTINCT study_iuid FROM study"#
        )
        .fetch_all(&self.dcm4chee_pool)
        .await
        .map_err(|e| format!("dcm4chee select all study uids failed: {}", e))?;

        if pacs_study_uids.is_empty() {
            // PACS에 Study가 없으면 모든 Study 삭제하지 않음 (안전을 위해)
            eprintln!("⚠️  [Sync] PACS에 Study가 없어 삭제 작업을 건너뜁니다");
            return Ok(0);
        }

        // 우리 DB에 있지만 PACS에 없는 Study 삭제
        // CASCADE DELETE로 인해 Series, Instance, project_data도 자동 삭제됨
        // PostgreSQL 배열을 사용하여 NOT IN 처리
        let deleted = sqlx::query(
            r#"DELETE FROM project_data_study 
               WHERE study_uid NOT IN (SELECT unnest($1::text[]))"#
        )
        .bind(&pacs_study_uids)
        .execute(&self.rbac_pool)
        .await
        .map_err(|e| format!("rbac delete missing studies failed: {}", e))?;

        Ok(deleted.rows_affected() as usize)
    }

    async fn cleanup_missing_series(&self) -> Result<usize, String> {
        // PACS에 있는 모든 series_uid 조회
        let pacs_series_uids: Vec<String> = sqlx::query_scalar::<_, String>(
            r#"SELECT DISTINCT series_iuid FROM series"#
        )
        .fetch_all(&self.dcm4chee_pool)
        .await
        .map_err(|e| format!("dcm4chee select all series uids failed: {}", e))?;

        if pacs_series_uids.is_empty() {
            eprintln!("⚠️  [Sync] PACS에 Series가 없어 삭제 작업을 건너뜁니다");
            return Ok(0);
        }

        // 우리 DB에 있지만 PACS에 없는 Series 삭제
        // CASCADE DELETE로 인해 Instance, project_data도 자동 삭제됨
        let deleted = sqlx::query(
            r#"DELETE FROM project_data_series 
               WHERE series_uid NOT IN (SELECT unnest($1::text[]))"#
        )
        .bind(&pacs_series_uids)
        .execute(&self.rbac_pool)
        .await
        .map_err(|e| format!("rbac delete missing series failed: {}", e))?;

        Ok(deleted.rows_affected() as usize)
    }

    async fn cleanup_missing_instances(&self) -> Result<usize, String> {
        // PACS에 있는 모든 instance_uid (sop_iuid) 조회
        let pacs_instance_uids: Vec<String> = sqlx::query_scalar::<_, String>(
            r#"SELECT DISTINCT sop_iuid FROM instance"#
        )
        .fetch_all(&self.dcm4chee_pool)
        .await
        .map_err(|e| format!("dcm4chee select all instance uids failed: {}", e))?;

        if pacs_instance_uids.is_empty() {
            eprintln!("⚠️  [Sync] PACS에 Instance가 없어 삭제 작업을 건너뜁니다");
            return Ok(0);
        }

        // 우리 DB에 있지만 PACS에 없는 Instance 삭제
        // CASCADE DELETE로 인해 project_data도 자동 삭제됨
        let deleted = sqlx::query(
            r#"DELETE FROM project_data_instance 
               WHERE instance_uid NOT IN (SELECT unnest($1::text[]))"#
        )
        .bind(&pacs_instance_uids)
        .execute(&self.rbac_pool)
        .await
        .map_err(|e| format!("rbac delete missing instances failed: {}", e))?;

        Ok(deleted.rows_affected() as usize)
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

            // find study id (no project_id in project_data_study)
            let study_id = sqlx::query_scalar::<_, i32>(
                r#"SELECT id FROM project_data_study WHERE study_uid = $1"#,
            )
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

            // find series id (no project_id in project_data_study)
            let series_id = sqlx::query_scalar::<_, i32>(
                r#"SELECT id FROM project_data_series WHERE series_uid = $1"#,
            )
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
        let start_time = std::time::Instant::now();
        eprintln!("🔄 [Sync] run_once() called");

        // 간단한 델타 동기화: last_run 기준으로 변경분 조회 후 upsert
        let last_run_opt = { self.state.read().await.last_run };
        let mut total_processed = 0usize;
        eprintln!("🔄 [Sync] Starting sync_studies...");

        match self.sync_studies(last_run_opt).await {
            Ok(n) => {
                eprintln!("🔄 [Sync] sync_studies completed: {} studies", n);
                total_processed += n
            },
            Err(e) => {
                eprintln!("❌ [Sync] sync_studies failed: {}", e);
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

        // PACS에 없는 데이터 삭제 (정리 작업)
        eprintln!("🔄 [Sync] Starting cleanup of missing data...");
        let mut deleted_count = 0usize;

        // Instance부터 삭제 (가장 하위 레벨)
        match self.cleanup_missing_instances().await {
            Ok(n) => {
                eprintln!("🔄 [Sync] Deleted {} missing instances", n);
                deleted_count += n;
            },
            Err(e) => {
                eprintln!("⚠️  [Sync] cleanup_missing_instances failed: {}", e);
            }
        }

        // Series 삭제
        match self.cleanup_missing_series().await {
            Ok(n) => {
                eprintln!("🔄 [Sync] Deleted {} missing series", n);
                deleted_count += n;
            },
            Err(e) => {
                eprintln!("⚠️  [Sync] cleanup_missing_series failed: {}", e);
            }
        }

        // Study 삭제 (가장 상위 레벨, CASCADE로 Series/Instance도 함께 삭제됨)
        match self.cleanup_missing_studies().await {
            Ok(n) => {
                eprintln!("🔄 [Sync] Deleted {} missing studies", n);
                deleted_count += n;
            },
            Err(e) => {
                eprintln!("⚠️  [Sync] cleanup_missing_studies failed: {}", e);
            }
        }

        eprintln!("🔄 [Sync] Cleanup completed: {} items deleted", deleted_count);

        let duration_ms = start_time.elapsed().as_millis();

        SyncResult {
            success: true,
            processed: total_processed,
            duration_ms,
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
