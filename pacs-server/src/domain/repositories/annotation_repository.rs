use async_trait::async_trait;
use sqlx::PgPool;
use crate::domain::entities::{Annotation, AnnotationHistory, NewAnnotation};

#[async_trait]
pub trait AnnotationRepository: Send + Sync {
    async fn find_by_id(&self, id: i32) -> Result<Option<Annotation>, sqlx::Error>;
    async fn find_by_project_id(&self, project_id: i32) -> Result<Vec<Annotation>, sqlx::Error>;
    async fn find_by_user_id(&self, user_id: i32) -> Result<Vec<Annotation>, sqlx::Error>;
    async fn find_by_study_uid(&self, study_uid: &str) -> Result<Vec<Annotation>, sqlx::Error>;
    async fn find_by_series_uid(&self, series_uid: &str) -> Result<Vec<Annotation>, sqlx::Error>;
    async fn find_by_instance_uid(&self, instance_uid: &str) -> Result<Vec<Annotation>, sqlx::Error>;
    async fn find_by_project_and_study(&self, project_id: i32, study_uid: &str) -> Result<Vec<Annotation>, sqlx::Error>;
    async fn find_shared_annotations(&self, project_id: i32) -> Result<Vec<Annotation>, sqlx::Error>;
    async fn create(&self, new_annotation: NewAnnotation) -> Result<Annotation, sqlx::Error>;
    async fn update(&self, id: i32, data: serde_json::Value, is_shared: bool) -> Result<Option<Annotation>, sqlx::Error>;
    async fn delete(&self, id: i32) -> Result<bool, sqlx::Error>;
    async fn create_history(&self, annotation_id: i32, user_id: i32, action: &str, data_before: Option<serde_json::Value>, data_after: Option<serde_json::Value>) -> Result<AnnotationHistory, sqlx::Error>;
    async fn get_history(&self, annotation_id: i32) -> Result<Vec<AnnotationHistory>, sqlx::Error>;
    fn pool(&self) -> &PgPool;
}
