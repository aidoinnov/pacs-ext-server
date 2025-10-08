use async_trait::async_trait;
use sqlx::PgPool;
use crate::domain::entities::{Annotation, AnnotationHistory, NewAnnotation};
use crate::domain::repositories::AnnotationRepository;

#[derive(Clone)]
pub struct AnnotationRepositoryImpl {
    pool: PgPool,
}

impl AnnotationRepositoryImpl {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AnnotationRepository for AnnotationRepositoryImpl {
    async fn find_by_id(&self, id: i32) -> Result<Option<Annotation>, sqlx::Error> {
        sqlx::query_as::<_, Annotation>(
            "SELECT id, project_id, user_id, study_uid, series_uid, instance_uid, 
                    tool_name, tool_version, data, is_shared, created_at, updated_at
             FROM annotation_annotation
             WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    async fn find_by_project_id(&self, project_id: i32) -> Result<Vec<Annotation>, sqlx::Error> {
        sqlx::query_as::<_, Annotation>(
            "SELECT id, project_id, user_id, study_uid, series_uid, instance_uid, 
                    tool_name, tool_version, data, is_shared, created_at, updated_at
             FROM annotation_annotation
             WHERE project_id = $1
             ORDER BY created_at DESC"
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await
    }

    async fn find_by_user_id(&self, user_id: i32) -> Result<Vec<Annotation>, sqlx::Error> {
        sqlx::query_as::<_, Annotation>(
            "SELECT id, project_id, user_id, study_uid, series_uid, instance_uid, 
                    tool_name, tool_version, data, is_shared, created_at, updated_at
             FROM annotation_annotation
             WHERE user_id = $1
             ORDER BY created_at DESC"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
    }

    async fn find_by_study_uid(&self, study_uid: &str) -> Result<Vec<Annotation>, sqlx::Error> {
        sqlx::query_as::<_, Annotation>(
            "SELECT id, project_id, user_id, study_uid, series_uid, instance_uid, 
                    tool_name, tool_version, data, is_shared, created_at, updated_at
             FROM annotation_annotation
             WHERE study_uid = $1
             ORDER BY created_at DESC"
        )
        .bind(study_uid)
        .fetch_all(&self.pool)
        .await
    }

    async fn find_by_series_uid(&self, series_uid: &str) -> Result<Vec<Annotation>, sqlx::Error> {
        sqlx::query_as::<_, Annotation>(
            "SELECT id, project_id, user_id, study_uid, series_uid, instance_uid, 
                    tool_name, tool_version, data, is_shared, created_at, updated_at
             FROM annotation_annotation
             WHERE series_uid = $1
             ORDER BY created_at DESC"
        )
        .bind(series_uid)
        .fetch_all(&self.pool)
        .await
    }

    async fn find_by_instance_uid(&self, instance_uid: &str) -> Result<Vec<Annotation>, sqlx::Error> {
        sqlx::query_as::<_, Annotation>(
            "SELECT id, project_id, user_id, study_uid, series_uid, instance_uid, 
                    tool_name, tool_version, data, is_shared, created_at, updated_at
             FROM annotation_annotation
             WHERE instance_uid = $1
             ORDER BY created_at DESC"
        )
        .bind(instance_uid)
        .fetch_all(&self.pool)
        .await
    }

    async fn find_by_project_and_study(&self, project_id: i32, study_uid: &str) -> Result<Vec<Annotation>, sqlx::Error> {
        sqlx::query_as::<_, Annotation>(
            "SELECT id, project_id, user_id, study_uid, series_uid, instance_uid, 
                    tool_name, tool_version, data, is_shared, created_at, updated_at
             FROM annotation_annotation
             WHERE project_id = $1 AND study_uid = $2
             ORDER BY created_at DESC"
        )
        .bind(project_id)
        .bind(study_uid)
        .fetch_all(&self.pool)
        .await
    }

    async fn find_shared_annotations(&self, project_id: i32) -> Result<Vec<Annotation>, sqlx::Error> {
        sqlx::query_as::<_, Annotation>(
            "SELECT id, project_id, user_id, study_uid, series_uid, instance_uid, 
                    tool_name, tool_version, data, is_shared, created_at, updated_at
             FROM annotation_annotation
             WHERE project_id = $1 AND is_shared = true
             ORDER BY created_at DESC"
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await
    }

    async fn create(&self, new_annotation: NewAnnotation) -> Result<Annotation, sqlx::Error> {
        sqlx::query_as::<_, Annotation>(
            "INSERT INTO annotation_annotation (project_id, user_id, study_uid, series_uid, instance_uid, 
                                               tool_name, tool_version, data, is_shared)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
             RETURNING id, project_id, user_id, study_uid, series_uid, instance_uid, 
                       tool_name, tool_version, data, is_shared, created_at, updated_at"
        )
        .bind(new_annotation.project_id)
        .bind(new_annotation.user_id)
        .bind(new_annotation.study_uid)
        .bind(new_annotation.series_uid)
        .bind(new_annotation.instance_uid)
        .bind(new_annotation.tool_name)
        .bind(new_annotation.tool_version)
        .bind(new_annotation.data)
        .bind(new_annotation.is_shared)
        .fetch_one(&self.pool)
        .await
    }

    async fn update(&self, id: i32, data: serde_json::Value, is_shared: bool) -> Result<Option<Annotation>, sqlx::Error> {
        sqlx::query_as::<_, Annotation>(
            "UPDATE annotation_annotation 
             SET data = $2, is_shared = $3, updated_at = CURRENT_TIMESTAMP
             WHERE id = $1
             RETURNING id, project_id, user_id, study_uid, series_uid, instance_uid, 
                       tool_name, tool_version, data, is_shared, created_at, updated_at"
        )
        .bind(id)
        .bind(data)
        .bind(is_shared)
        .fetch_optional(&self.pool)
        .await
    }

    async fn delete(&self, id: i32) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM annotation_annotation WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    async fn create_history(&self, annotation_id: i32, user_id: i32, action: &str, data_before: Option<serde_json::Value>, data_after: Option<serde_json::Value>) -> Result<AnnotationHistory, sqlx::Error> {
        sqlx::query_as::<_, AnnotationHistory>(
            "INSERT INTO annotation_annotation_history (annotation_id, user_id, action, data_before, data_after)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING id, annotation_id, user_id, action, data_before, data_after, action_at"
        )
        .bind(annotation_id)
        .bind(user_id)
        .bind(action)
        .bind(data_before)
        .bind(data_after)
        .fetch_one(&self.pool)
        .await
    }

    async fn get_history(&self, annotation_id: i32) -> Result<Vec<AnnotationHistory>, sqlx::Error> {
        sqlx::query_as::<_, AnnotationHistory>(
            "SELECT id, annotation_id, user_id, action, data_before, data_after, action_at
             FROM annotation_annotation_history
             WHERE annotation_id = $1
             ORDER BY action_at DESC"
        )
        .bind(annotation_id)
        .fetch_all(&self.pool)
        .await
    }

    fn pool(&self) -> &PgPool {
        &self.pool
    }
}

