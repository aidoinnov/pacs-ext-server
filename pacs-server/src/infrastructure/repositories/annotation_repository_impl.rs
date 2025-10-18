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
                    tool_name, tool_version, data, is_shared, created_at, updated_at,
                    viewer_software, description, measurement_values
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
                    tool_name, tool_version, data, is_shared, created_at, updated_at,
                    viewer_software, description, measurement_values
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
                    tool_name, tool_version, data, is_shared, created_at, updated_at,
                    viewer_software, description, measurement_values
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
                    tool_name, tool_version, data, is_shared, created_at, updated_at,
                    viewer_software, description, measurement_values
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
                    tool_name, tool_version, data, is_shared, created_at, updated_at,
                    viewer_software, description, measurement_values
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
                    tool_name, tool_version, data, is_shared, created_at, updated_at,
                    viewer_software, description, measurement_values
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
                    tool_name, tool_version, data, is_shared, created_at, updated_at,
                    viewer_software, description, measurement_values
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
                    tool_name, tool_version, data, is_shared, created_at, updated_at,
                    viewer_software, description
             FROM annotation_annotation
             WHERE project_id = $1 AND is_shared = true
             ORDER BY created_at DESC"
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await
    }

    async fn create(&self, new_annotation: NewAnnotation) -> Result<Annotation, sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        // annotation 생성
        let annotation = sqlx::query_as::<_, Annotation>(
            "INSERT INTO annotation_annotation (project_id, user_id, study_uid, series_uid, instance_uid, 
                                               tool_name, tool_version, data, is_shared, viewer_software, description, measurement_values)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
             RETURNING id, project_id, user_id, study_uid, series_uid, instance_uid, 
                       tool_name, tool_version, data, is_shared, created_at, updated_at,
                       viewer_software, description, measurement_values"
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
        .bind(new_annotation.viewer_software)
        .bind(new_annotation.description)
        .bind(new_annotation.measurement_values)
        .fetch_one(&mut *tx)
        .await?;

        // history 생성 (같은 트랜잭션 내에서)
        let _ = sqlx::query_as::<_, AnnotationHistory>(
            "INSERT INTO annotation_annotation_history (annotation_id, user_id, action, data_before, data_after)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING id, annotation_id, user_id, action, data_before, data_after, action_at"
        )
        .bind(annotation.id)
        .bind(annotation.user_id)
        .bind("create")
        .bind(None::<serde_json::Value>)
        .bind(Some(annotation.data.clone()))
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(annotation)
    }

    async fn update(&self, id: i32, data: serde_json::Value, is_shared: bool) -> Result<Option<Annotation>, sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        // 기존 annotation 데이터를 가져와서 history에 저장
        let old_annotation = sqlx::query_as::<_, Annotation>(
            "SELECT id, project_id, user_id, study_uid, series_uid, instance_uid, 
                    tool_name, tool_version, data, is_shared, created_at, updated_at,
                    viewer_software, description, measurement_values
             FROM annotation_annotation WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?;

        let old_data = old_annotation.as_ref().map(|a| a.data.clone());
        let user_id = old_annotation.as_ref().map(|a| a.user_id).unwrap_or(0);

        // annotation 업데이트
        let updated_annotation = sqlx::query_as::<_, Annotation>(
            "UPDATE annotation_annotation 
             SET data = $2, is_shared = $3, updated_at = CURRENT_TIMESTAMP
             WHERE id = $1
             RETURNING id, project_id, user_id, study_uid, series_uid, instance_uid, 
                       tool_name, tool_version, data, is_shared, created_at, updated_at,
                       viewer_software, description, measurement_values"
        )
        .bind(id)
        .bind(data)
        .bind(is_shared)
        .fetch_optional(&mut *tx)
        .await?;

        // history 생성 (같은 트랜잭션 내에서)
        if let Some(annotation) = &updated_annotation {
            let _ = sqlx::query_as::<_, AnnotationHistory>(
                "INSERT INTO annotation_annotation_history (annotation_id, user_id, action, data_before, data_after)
                 VALUES ($1, $2, $3, $4, $5)
                 RETURNING id, annotation_id, user_id, action, data_before, data_after, action_at"
            )
            .bind(annotation.id)
            .bind(user_id)
            .bind("update")
            .bind(old_data)
            .bind(Some(annotation.data.clone()))
            .fetch_one(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(updated_annotation)
    }

    async fn update_with_measurements(&self, id: i32, data: serde_json::Value, is_shared: bool, measurement_values: Option<serde_json::Value>) -> Result<Option<Annotation>, sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        // 기존 annotation 데이터를 가져와서 history에 저장
        let old_annotation = sqlx::query_as::<_, Annotation>(
            "SELECT id, project_id, user_id, study_uid, series_uid, instance_uid, 
                    tool_name, tool_version, data, is_shared, created_at, updated_at,
                    viewer_software, description, measurement_values
             FROM annotation_annotation WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?;

        let old_data = old_annotation.as_ref().map(|a| a.data.clone());
        let user_id = old_annotation.as_ref().map(|a| a.user_id).unwrap_or(0);

        // annotation 업데이트 (measurement_values 포함)
        let updated_annotation = sqlx::query_as::<_, Annotation>(
            "UPDATE annotation_annotation 
             SET data = $2, is_shared = $3, measurement_values = $4, updated_at = CURRENT_TIMESTAMP
             WHERE id = $1
             RETURNING id, project_id, user_id, study_uid, series_uid, instance_uid, 
                       tool_name, tool_version, data, is_shared, created_at, updated_at,
                       viewer_software, description, measurement_values"
        )
        .bind(id)
        .bind(data)
        .bind(is_shared)
        .bind(measurement_values)
        .fetch_optional(&mut *tx)
        .await?;

        // history 생성 (같은 트랜잭션 내에서)
        if let Some(annotation) = &updated_annotation {
            let _ = sqlx::query_as::<_, AnnotationHistory>(
                "INSERT INTO annotation_annotation_history (annotation_id, user_id, action, data_before, data_after)
                 VALUES ($1, $2, $3, $4, $5)
                 RETURNING id, annotation_id, user_id, action, data_before, data_after, action_at"
            )
            .bind(annotation.id)
            .bind(user_id)
            .bind("UPDATE")
            .bind(old_data)
            .bind(Some(annotation.data.clone()))
            .fetch_one(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(updated_annotation)
    }

    async fn delete(&self, id: i32) -> Result<bool, sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        // 기존 annotation 데이터를 가져와서 history에 저장
        let old_annotation = sqlx::query_as::<_, Annotation>(
            "SELECT id, project_id, user_id, study_uid, series_uid, instance_uid, 
                    tool_name, tool_version, data, is_shared, created_at, updated_at,
                    viewer_software, description
             FROM annotation_annotation WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?;

        if let Some(annotation) = old_annotation {
            // history 생성 (같은 트랜잭션 내에서)
            let _ = sqlx::query_as::<_, AnnotationHistory>(
                "INSERT INTO annotation_annotation_history (annotation_id, user_id, action, data_before, data_after)
                 VALUES ($1, $2, $3, $4, $5)
                 RETURNING id, annotation_id, user_id, action, data_before, data_after, action_at"
            )
            .bind(annotation.id)
            .bind(annotation.user_id)
            .bind("delete")
            .bind(Some(annotation.data.clone()))
            .bind(None::<serde_json::Value>)
            .fetch_one(&mut *tx)
            .await?;

            // annotation 삭제
            let result = sqlx::query("DELETE FROM annotation_annotation WHERE id = $1")
                .bind(id)
                .execute(&mut *tx)
                .await?;

            tx.commit().await?;
            Ok(result.rows_affected() > 0)
        } else {
            tx.commit().await?;
            Ok(false)
        }
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

    // viewer_software 필터링 메서드들
    async fn find_by_user_id_with_viewer(&self, user_id: i32, viewer_software: Option<&str>) -> Result<Vec<Annotation>, sqlx::Error> {
        match viewer_software {
            Some(viewer) => {
                sqlx::query_as::<_, Annotation>(
                    "SELECT id, project_id, user_id, study_uid, series_uid, instance_uid, 
                            tool_name, tool_version, data, is_shared, created_at, updated_at,
                            viewer_software, description, measurement_values
                     FROM annotation_annotation
                     WHERE user_id = $1 AND viewer_software = $2
                     ORDER BY created_at DESC"
                )
                .bind(user_id)
                .bind(viewer)
                .fetch_all(&self.pool)
                .await
            },
            None => {
                sqlx::query_as::<_, Annotation>(
                    "SELECT id, project_id, user_id, study_uid, series_uid, instance_uid, 
                            tool_name, tool_version, data, is_shared, created_at, updated_at,
                            viewer_software, description, measurement_values
                     FROM annotation_annotation
                     WHERE user_id = $1
                     ORDER BY created_at DESC"
                )
                .bind(user_id)
                .fetch_all(&self.pool)
                .await
            }
        }
    }

    async fn find_by_project_id_with_viewer(&self, project_id: i32, viewer_software: Option<&str>) -> Result<Vec<Annotation>, sqlx::Error> {
        match viewer_software {
            Some(viewer) => {
                sqlx::query_as::<_, Annotation>(
                    "SELECT id, project_id, user_id, study_uid, series_uid, instance_uid, 
                            tool_name, tool_version, data, is_shared, created_at, updated_at,
                            viewer_software, description, measurement_values
                     FROM annotation_annotation
                     WHERE project_id = $1 AND viewer_software = $2
                     ORDER BY created_at DESC"
                )
                .bind(project_id)
                .bind(viewer)
                .fetch_all(&self.pool)
                .await
            },
            None => {
                sqlx::query_as::<_, Annotation>(
                    "SELECT id, project_id, user_id, study_uid, series_uid, instance_uid, 
                            tool_name, tool_version, data, is_shared, created_at, updated_at,
                            viewer_software, description, measurement_values
                     FROM annotation_annotation
                     WHERE project_id = $1
                     ORDER BY created_at DESC"
                )
                .bind(project_id)
                .fetch_all(&self.pool)
                .await
            }
        }
    }

    async fn find_by_study_uid_with_viewer(&self, study_uid: &str, viewer_software: Option<&str>) -> Result<Vec<Annotation>, sqlx::Error> {
        match viewer_software {
            Some(viewer) => {
                sqlx::query_as::<_, Annotation>(
                    "SELECT id, project_id, user_id, study_uid, series_uid, instance_uid, 
                            tool_name, tool_version, data, is_shared, created_at, updated_at,
                            viewer_software, description, measurement_values
                     FROM annotation_annotation
                     WHERE study_uid = $1 AND viewer_software = $2
                     ORDER BY created_at DESC"
                )
                .bind(study_uid)
                .bind(viewer)
                .fetch_all(&self.pool)
                .await
            },
            None => {
                sqlx::query_as::<_, Annotation>(
                    "SELECT id, project_id, user_id, study_uid, series_uid, instance_uid, 
                            tool_name, tool_version, data, is_shared, created_at, updated_at,
                            viewer_software, description, measurement_values
                     FROM annotation_annotation
                     WHERE study_uid = $1
                     ORDER BY created_at DESC"
                )
                .bind(study_uid)
                .fetch_all(&self.pool)
                .await
            }
        }
    }

    fn pool(&self) -> &PgPool {
        &self.pool
    }
}

