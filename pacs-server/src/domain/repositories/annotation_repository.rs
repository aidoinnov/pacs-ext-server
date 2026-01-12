use crate::domain::entities::{Annotation, AnnotationHistory, NewAnnotation, SnapshotUploadStatus};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

#[async_trait]
pub trait AnnotationRepository: Send + Sync {
    async fn find_by_id(&self, id: i32) -> Result<Option<Annotation>, sqlx::Error>;
    async fn find_by_project_id(&self, project_id: i32) -> Result<Vec<Annotation>, sqlx::Error>;
    async fn find_by_user_id(&self, user_id: i32) -> Result<Vec<Annotation>, sqlx::Error>;
    async fn find_by_study_uid(&self, study_uid: &str) -> Result<Vec<Annotation>, sqlx::Error>;
    async fn find_by_series_uid(&self, series_uid: &str) -> Result<Vec<Annotation>, sqlx::Error>;
    async fn find_by_instance_uid(
        &self,
        instance_uid: &str,
    ) -> Result<Vec<Annotation>, sqlx::Error>;
    async fn find_by_project_and_study(
        &self,
        project_id: i32,
        study_uid: &str,
    ) -> Result<Vec<Annotation>, sqlx::Error>;
    async fn find_by_project_and_series(
        &self,
        project_id: i32,
        series_uid: &str,
    ) -> Result<Vec<Annotation>, sqlx::Error>;
    async fn find_shared_annotations(
        &self,
        project_id: i32,
    ) -> Result<Vec<Annotation>, sqlx::Error>;

    // viewer_software 필터링 메서드들
    async fn find_by_user_id_with_viewer(
        &self,
        user_id: i32,
        viewer_software: Option<&str>,
    ) -> Result<Vec<Annotation>, sqlx::Error>;
    async fn find_by_project_id_with_viewer(
        &self,
        project_id: i32,
        viewer_software: Option<&str>,
    ) -> Result<Vec<Annotation>, sqlx::Error>;
    async fn find_by_study_uid_with_viewer(
        &self,
        study_uid: &str,
        viewer_software: Option<&str>,
    ) -> Result<Vec<Annotation>, sqlx::Error>;
    async fn create(&self, new_annotation: NewAnnotation) -> Result<Annotation, sqlx::Error>;
    async fn update(
        &self,
        id: i32,
        data: serde_json::Value,
        is_shared: bool,
    ) -> Result<Option<Annotation>, sqlx::Error>;
    async fn update_with_measurements(
        &self,
        id: i32,
        data: serde_json::Value,
        is_shared: bool,
        measurement_values: Option<serde_json::Value>,
        label: Option<String>,
    ) -> Result<Option<Annotation>, sqlx::Error>;
    /// 버전 검증을 포함한 업데이트 (Optimistic Locking)
    /// 클라이언트가 제공한 base_version과 현재 버전이 일치할 때만 업데이트 수행
    async fn update_with_version_check(
        &self,
        id: i32,
        base_version: i32,
        data: serde_json::Value,
        is_shared: bool,
        measurement_values: Option<serde_json::Value>,
        label: Option<String>,
    ) -> Result<Option<Annotation>, sqlx::Error>;

    async fn update_snapshot(
        &self,
        id: i32,
        snapshot_image_key: String,
        snapshot_status: SnapshotUploadStatus,
        snapshot_uploaded_at: Option<DateTime<Utc>>,
    ) -> Result<Option<Annotation>, sqlx::Error>;


    async fn delete(&self, id: i32) -> Result<bool, sqlx::Error>;
    async fn create_history(
        &self,
        annotation_id: i32,
        user_id: i32,
        action: &str,
        data_before: Option<serde_json::Value>,
        data_after: Option<serde_json::Value>,
    ) -> Result<AnnotationHistory, sqlx::Error>;
    async fn get_history(&self, annotation_id: i32) -> Result<Vec<AnnotationHistory>, sqlx::Error>;

    /// 프로젝트와 Series UID로 최신 수정 시간 조회 (리스트 버전용)
    async fn get_max_updated_at_by_project_and_series(
        &self,
        project_id: i32,
        series_uid: &str,
    ) -> Result<Option<DateTime<Utc>>, sqlx::Error>;

    /// 프로젝트와 Series UID로 페이지네이션된 어노테이션 조회
    async fn find_by_project_and_series_paginated(
        &self,
        project_id: i32,
        series_uid: &str,
        page: i32,
        limit: i32,
    ) -> Result<Vec<Annotation>, sqlx::Error>;

    fn pool(&self) -> &PgPool;
}
