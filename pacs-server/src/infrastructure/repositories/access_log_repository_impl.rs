use crate::domain::entities::{AccessLog, NewAccessLog};
use crate::domain::repositories::AccessLogRepository;
use async_trait::async_trait;
use chrono::NaiveDateTime;
use sqlx::PgPool;

pub struct AccessLogRepositoryImpl {
    pool: PgPool,
}

impl AccessLogRepositoryImpl {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AccessLogRepository for AccessLogRepositoryImpl {
    async fn create(&self, new_log: NewAccessLog) -> Result<AccessLog, sqlx::Error> {
        sqlx::query_as::<_, AccessLog>(
            "INSERT INTO security_access_log
             (user_id, project_id, resource_type, study_uid, series_uid, instance_uid,
              action, result, dicom_tag_check, ae_title, ip_address, session_id, via_group_id)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
             RETURNING id, user_id, project_id, resource_type, study_uid, series_uid, instance_uid,
                       action, result, dicom_tag_check, ae_title, ip_address, session_id, via_group_id, logged_at"
        )
        .bind(new_log.user_id)
        .bind(new_log.project_id)
        .bind(new_log.resource_type)
        .bind(new_log.study_uid)
        .bind(new_log.series_uid)
        .bind(new_log.instance_uid)
        .bind(new_log.action)
        .bind(new_log.result)
        .bind(new_log.dicom_tag_check)
        .bind(new_log.ae_title)
        .bind(new_log.ip_address)
        .bind(new_log.session_id)
        .bind(new_log.via_group_id)
        .fetch_one(&self.pool)
        .await
    }

    async fn find_by_user_id(
        &self,
        user_id: i32,
        limit: i64,
    ) -> Result<Vec<AccessLog>, sqlx::Error> {
        sqlx::query_as::<_, AccessLog>(
            "SELECT id, user_id, project_id, resource_type, study_uid, series_uid, instance_uid,
                    action, result, dicom_tag_check, ae_title, ip_address, session_id, via_group_id, logged_at
             FROM security_access_log
             WHERE user_id = $1
             ORDER BY logged_at DESC
             LIMIT $2"
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }

    async fn find_by_project_id(
        &self,
        project_id: i32,
        limit: i64,
    ) -> Result<Vec<AccessLog>, sqlx::Error> {
        sqlx::query_as::<_, AccessLog>(
            "SELECT id, user_id, project_id, resource_type, study_uid, series_uid, instance_uid,
                    action, result, dicom_tag_check, ae_title, ip_address, session_id, via_group_id, logged_at
             FROM security_access_log
             WHERE project_id = $1
             ORDER BY logged_at DESC
             LIMIT $2"
        )
        .bind(project_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }

    async fn find_by_study_uid(
        &self,
        study_uid: &str,
        limit: i64,
    ) -> Result<Vec<AccessLog>, sqlx::Error> {
        sqlx::query_as::<_, AccessLog>(
            "SELECT id, user_id, project_id, resource_type, study_uid, series_uid, instance_uid,
                    action, result, dicom_tag_check, ae_title, ip_address, session_id, via_group_id, logged_at
             FROM security_access_log
             WHERE study_uid = $1
             ORDER BY logged_at DESC
             LIMIT $2"
        )
        .bind(study_uid)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }

    async fn find_by_time_range(
        &self,
        start: NaiveDateTime,
        end: NaiveDateTime,
    ) -> Result<Vec<AccessLog>, sqlx::Error> {
        sqlx::query_as::<_, AccessLog>(
            "SELECT id, user_id, project_id, resource_type, study_uid, series_uid, instance_uid,
                    action, result, dicom_tag_check, ae_title, ip_address, session_id, via_group_id, logged_at
             FROM security_access_log
             WHERE logged_at BETWEEN $1 AND $2
             ORDER BY logged_at DESC"
        )
        .bind(start)
        .bind(end)
        .fetch_all(&self.pool)
        .await
    }

    async fn count_by_user_id(&self, user_id: i32) -> Result<i64, sqlx::Error> {
        let row: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM security_access_log WHERE user_id = $1")
                .bind(user_id)
                .fetch_one(&self.pool)
                .await?;

        Ok(row.0)
    }
}
