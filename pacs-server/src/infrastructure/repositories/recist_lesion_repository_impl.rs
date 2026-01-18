use crate::domain::entities::{
    CreateRecistLesion, CreateRecistLesionAnnotationMap, RecistLesion, RecistLesionAnnotationInfo,
    RecistLesionAnnotationMap, RecistLesionDetail, RecistLesionType, UpdateRecistLesion,
};
use crate::domain::repositories::RecistLesionRepository;
use async_trait::async_trait;
use sqlx::PgPool;

#[derive(Clone)]
pub struct RecistLesionRepositoryImpl {
    pool: PgPool,
}

impl RecistLesionRepositoryImpl {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RecistLesionRepository for RecistLesionRepositoryImpl {
    async fn find_by_id(&self, id: i32) -> Result<Option<RecistLesion>, sqlx::Error> {
        sqlx::query_as::<_, RecistLesion>(
            "SELECT id, project_id, subject_id, lesion_type, lesion_number, 
                    baseline_timepoint_id, organ_site, description, created_at, updated_at
             FROM recist_lesion
             WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    async fn find_by_subject(
        &self,
        subject_id: i32,
        lesion_type: Option<RecistLesionType>,
    ) -> Result<Vec<RecistLesion>, sqlx::Error> {
        match lesion_type {
            Some(lt) => {
                sqlx::query_as::<_, RecistLesion>(
                    "SELECT id, project_id, subject_id, lesion_type, lesion_number, 
                            baseline_timepoint_id, organ_site, description, created_at, updated_at
                     FROM recist_lesion
                     WHERE subject_id = $1 AND lesion_type = $2
                     ORDER BY lesion_number",
                )
                .bind(subject_id)
                .bind(lt)
                .fetch_all(&self.pool)
                .await
            }
            None => {
                sqlx::query_as::<_, RecistLesion>(
                    "SELECT id, project_id, subject_id, lesion_type, lesion_number, 
                            baseline_timepoint_id, organ_site, description, created_at, updated_at
                     FROM recist_lesion
                     WHERE subject_id = $1
                     ORDER BY lesion_number",
                )
                .bind(subject_id)
                .fetch_all(&self.pool)
                .await
            }
        }
    }

    async fn find_detail_by_id(&self, id: i32) -> Result<Option<RecistLesionDetail>, sqlx::Error> {
        // 최적화: 단일 쿼리로 Lesion과 Annotation을 함께 조회
        let lesion = self.find_by_id(id).await?;

        if let Some(lesion) = lesion {
            // Annotation 목록 조회 (별도 쿼리 - 향후 LEFT JOIN으로 최적화 가능)
            let annotations = self.find_annotations_by_lesion(id).await?;

            Ok(Some(RecistLesionDetail {
                lesion,
                annotations,
            }))
        } else {
            Ok(None)
        }
    }

    async fn create(&self, new_lesion: CreateRecistLesion) -> Result<RecistLesion, sqlx::Error> {
        // lesion_number 자동 생성
        let lesion_number = self.get_next_lesion_number(new_lesion.subject_id).await?;

        sqlx::query_as::<_, RecistLesion>(
            "INSERT INTO recist_lesion 
             (project_id, subject_id, lesion_type, lesion_number, baseline_timepoint_id, organ_site, description)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             RETURNING id, project_id, subject_id, lesion_type, lesion_number, 
                       baseline_timepoint_id, organ_site, description, created_at, updated_at",
        )
        .bind(new_lesion.project_id)
        .bind(new_lesion.subject_id)
        .bind(new_lesion.lesion_type)
        .bind(lesion_number)
        .bind(new_lesion.baseline_timepoint_id)
        .bind(new_lesion.organ_site)
        .bind(new_lesion.description)
        .fetch_one(&self.pool)
        .await
    }

    async fn update(
        &self,
        id: i32,
        update_data: UpdateRecistLesion,
    ) -> Result<RecistLesion, sqlx::Error> {
        sqlx::query_as::<_, RecistLesion>(
            "UPDATE recist_lesion
             SET organ_site = COALESCE($2, organ_site),
                 description = COALESCE($3, description)
             WHERE id = $1
             RETURNING id, project_id, subject_id, lesion_type, lesion_number, 
                       baseline_timepoint_id, organ_site, description, created_at, updated_at",
        )
        .bind(id)
        .bind(update_data.organ_site)
        .bind(update_data.description)
        .fetch_one(&self.pool)
        .await
    }

    async fn delete(&self, id: i32) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM recist_lesion WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn get_next_lesion_number(&self, subject_id: i32) -> Result<i32, sqlx::Error> {
        let result: Option<(Option<i32>,)> = sqlx::query_as(
            "SELECT MAX(lesion_number) FROM recist_lesion WHERE subject_id = $1",
        )
        .bind(subject_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.and_then(|(max,)| max).unwrap_or(0) + 1)
    }

    async fn create_annotation_mapping(
        &self,
        mapping: CreateRecistLesionAnnotationMap,
    ) -> Result<RecistLesionAnnotationMap, sqlx::Error> {
        sqlx::query_as::<_, RecistLesionAnnotationMap>(
            "INSERT INTO recist_lesion_annotation_map 
             (lesion_id, annotation_id, timepoint_id, measured_length_mm)
             VALUES ($1, $2, $3, $4)
             RETURNING id, lesion_id, annotation_id, timepoint_id, measured_length_mm, measured_at, created_at",
        )
        .bind(mapping.lesion_id)
        .bind(mapping.annotation_id)
        .bind(mapping.timepoint_id)
        .bind(mapping.measured_length_mm)
        .fetch_one(&self.pool)
        .await
    }

    async fn find_annotations_by_lesion(
        &self,
        lesion_id: i32,
    ) -> Result<Vec<RecistLesionAnnotationInfo>, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct AnnotationRow {
            timepoint_id: i32,
            timepoint_name: String,
            annotation_id: i32,
            measured_length_mm: Option<f64>,
            measured_at: chrono::DateTime<chrono::Utc>,
        }

        let rows = sqlx::query_as::<_, AnnotationRow>(
            r#"
            SELECT
                m.timepoint_id,
                t.name as timepoint_name,
                m.annotation_id,
                m.measured_length_mm,
                m.measured_at
            FROM recist_lesion_annotation_map m
            INNER JOIN subject_timepoint t ON t.id = m.timepoint_id
            WHERE m.lesion_id = $1
            ORDER BY t.order_index
            "#,
        )
        .bind(lesion_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| RecistLesionAnnotationInfo {
                timepoint_id: row.timepoint_id,
                timepoint_name: row.timepoint_name,
                annotation_id: row.annotation_id,
                measured_length_mm: row.measured_length_mm,
                measured_at: row.measured_at,
            })
            .collect())
    }

    async fn find_by_annotation_id(
        &self,
        annotation_id: i32,
    ) -> Result<Option<RecistLesion>, sqlx::Error> {
        sqlx::query_as::<_, RecistLesion>(
            r#"
            SELECT l.id, l.project_id, l.subject_id, l.lesion_type, l.lesion_number,
                   l.baseline_timepoint_id, l.organ_site, l.description, l.created_at, l.updated_at
            FROM recist_lesion l
            INNER JOIN recist_lesion_annotation_map m ON m.lesion_id = l.id
            WHERE m.annotation_id = $1
            "#,
        )
        .bind(annotation_id)
        .fetch_optional(&self.pool)
        .await
    }

    async fn delete_annotation_mapping(&self, annotation_id: i32) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM recist_lesion_annotation_map WHERE annotation_id = $1")
            .bind(annotation_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
