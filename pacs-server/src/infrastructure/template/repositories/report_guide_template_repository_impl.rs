use crate::domain::template::entities::report_guide_template::*;
use crate::domain::template::repositories::ReportGuideTemplateRepository;
use async_trait::async_trait;
use sqlx::{PgPool, Result};

#[derive(Clone)]
pub struct ReportGuideTemplateRepositoryImpl {
    pool: PgPool,
}

impl ReportGuideTemplateRepositoryImpl {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ReportGuideTemplateRepository for ReportGuideTemplateRepositoryImpl {
    // ========== 원본 템플릿 ==========

    async fn create_template(
        &self,
        new_template: &NewReportGuideTemplate,
    ) -> Result<ReportGuideTemplate> {
        sqlx::query_as::<_, ReportGuideTemplate>(
            r#"
            INSERT INTO report_guide_template (name, description, conclusion, bodypart, is_shared, created_by)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, name, description, conclusion, bodypart, is_shared, is_active, created_by, created_at, updated_at
            "#,
        )
        .bind(&new_template.name)
        .bind(&new_template.description)
        .bind(&new_template.conclusion)
        .bind(&new_template.bodypart)
        .bind(new_template.is_shared)
        .bind(new_template.created_by)
        .fetch_one(&self.pool)
        .await
    }

    async fn find_template_by_id(&self, id: i32) -> Result<Option<ReportGuideTemplate>> {
        sqlx::query_as::<_, ReportGuideTemplate>(
            r#"
            SELECT id, name, description, conclusion, bodypart, is_shared, is_active, created_by, created_at, updated_at
            FROM report_guide_template
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    async fn find_templates(
        &self,
        modality: Option<&str>,
        bodypart: Option<&str>,
        is_active: Option<bool>,
    ) -> Result<Vec<ReportGuideTemplate>> {
        // 간단한 구현: 필터링은 나중에 개선
        let query = if let Some(active) = is_active {
            sqlx::query_as::<_, ReportGuideTemplate>(
                r#"
                SELECT id, name, description, conclusion, bodypart, is_shared, is_active, created_by, created_at, updated_at
                FROM report_guide_template
                WHERE is_active = $1
                ORDER BY created_at DESC
                "#,
            )
            .bind(active)
        } else {
            sqlx::query_as::<_, ReportGuideTemplate>(
                r#"
                SELECT id, name, description, conclusion, bodypart, is_shared, is_active, created_by, created_at, updated_at
                FROM report_guide_template
                ORDER BY created_at DESC
                "#,
            )
        };

        query.fetch_all(&self.pool).await
    }

    async fn update_template(
        &self,
        id: i32,
        update: &UpdateReportGuideTemplate,
    ) -> Result<ReportGuideTemplate> {
        sqlx::query_as::<_, ReportGuideTemplate>(
            r#"
            UPDATE report_guide_template
            SET
                name = COALESCE($2, name),
                description = COALESCE($3, description),
                conclusion = COALESCE($4, conclusion),
                bodypart = COALESCE($5, bodypart),
                is_shared = COALESCE($6, is_shared),
                is_active = COALESCE($7, is_active),
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $1
            RETURNING id, name, description, conclusion, bodypart, is_shared, is_active, created_by, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(&update.name)
        .bind(&update.description)
        .bind(&update.conclusion)
        .bind(&update.bodypart)
        .bind(&update.is_shared)
        .bind(&update.is_active)
        .fetch_one(&self.pool)
        .await
    }

    async fn delete_template(&self, id: i32) -> Result<bool> {
        let result = sqlx::query("DELETE FROM report_guide_template WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    // ========== 템플릿 모달리티 ==========

    async fn add_modality(
        &self,
        template_id: i32,
        modality: &str,
    ) -> Result<ReportGuideTemplateModality> {
        sqlx::query_as::<_, ReportGuideTemplateModality>(
            r#"
            INSERT INTO report_guide_template_modality (template_id, modality)
            VALUES ($1, $2)
            ON CONFLICT (template_id, modality) DO NOTHING
            RETURNING id, template_id, modality
            "#,
        )
        .bind(template_id)
        .bind(modality)
        .fetch_one(&self.pool)
        .await
    }

    async fn find_modalities_by_template(
        &self,
        template_id: i32,
    ) -> Result<Vec<ReportGuideTemplateModality>> {
        sqlx::query_as::<_, ReportGuideTemplateModality>(
            r#"
            SELECT id, template_id, modality
            FROM report_guide_template_modality
            WHERE template_id = $1
            ORDER BY modality
            "#,
        )
        .bind(template_id)
        .fetch_all(&self.pool)
        .await
    }

    async fn remove_modality(&self, template_id: i32, modality: &str) -> Result<bool> {
        let result = sqlx::query(
            "DELETE FROM report_guide_template_modality WHERE template_id = $1 AND modality = $2",
        )
        .bind(template_id)
        .bind(modality)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    // ========== 템플릿 이미지 ==========

    async fn add_template_image(
        &self,
        new_image: &NewReportGuideTemplateImage,
    ) -> Result<ReportGuideTemplateImage> {
        sqlx::query_as::<_, ReportGuideTemplateImage>(
            r#"
            INSERT INTO report_guide_template_image (
                template_id, image_path, image_url, file_size, mime_type,
                display_order, is_shared, uploaded_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, template_id, image_path, image_url, file_size, mime_type,
                     display_order, is_shared, uploaded_by, created_at
            "#,
        )
        .bind(new_image.template_id)
        .bind(&new_image.image_path)
        .bind(&new_image.image_url)
        .bind(new_image.file_size)
        .bind(&new_image.mime_type)
        .bind(new_image.display_order)
        .bind(new_image.is_shared)
        .bind(new_image.uploaded_by)
        .fetch_one(&self.pool)
        .await
    }

    async fn find_template_images(
        &self,
        template_id: i32,
    ) -> Result<Vec<ReportGuideTemplateImage>> {
        sqlx::query_as::<_, ReportGuideTemplateImage>(
            r#"
            SELECT id, template_id, image_path, image_url, file_size, mime_type,
                   display_order, is_shared, uploaded_by, created_at
            FROM report_guide_template_image
            WHERE template_id = $1
            ORDER BY display_order, created_at
            "#,
        )
        .bind(template_id)
        .fetch_all(&self.pool)
        .await
    }

    async fn find_template_image_by_id(
        &self,
        id: i32,
    ) -> Result<Option<ReportGuideTemplateImage>> {
        sqlx::query_as::<_, ReportGuideTemplateImage>(
            r#"
            SELECT id, template_id, image_path, image_url, file_size, mime_type,
                   display_order, is_shared, uploaded_by, created_at
            FROM report_guide_template_image
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    async fn update_image_share_status(
        &self,
        image_id: i32,
        is_shared: bool,
    ) -> Result<ReportGuideTemplateImage> {
        sqlx::query_as::<_, ReportGuideTemplateImage>(
            r#"
            UPDATE report_guide_template_image
            SET is_shared = $2
            WHERE id = $1
            RETURNING id, template_id, image_path, image_url, file_size, mime_type,
                     display_order, is_shared, uploaded_by, created_at
            "#,
        )
        .bind(image_id)
        .bind(is_shared)
        .fetch_one(&self.pool)
        .await
    }

    async fn delete_template_image(&self, image_id: i32) -> Result<bool> {
        let result = sqlx::query("DELETE FROM report_guide_template_image WHERE id = $1")
            .bind(image_id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    // ========== 사용자 커스텀 템플릿 ==========

    async fn create_custom_template(
        &self,
        new_template: &NewUserCustomReportTemplate,
    ) -> Result<UserCustomReportTemplate> {
        sqlx::query_as::<_, UserCustomReportTemplate>(
            r#"
            INSERT INTO user_custom_report_template (
                user_id, base_template_id, name, description, conclusion, bodypart
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, user_id, base_template_id, name, description, conclusion, bodypart,
                     is_active, created_at, updated_at
            "#,
        )
        .bind(new_template.user_id)
        .bind(new_template.base_template_id)
        .bind(&new_template.name)
        .bind(&new_template.description)
        .bind(&new_template.conclusion)
        .bind(&new_template.bodypart)
        .fetch_one(&self.pool)
        .await
    }

    async fn find_custom_template_by_id(
        &self,
        id: i32,
    ) -> Result<Option<UserCustomReportTemplate>> {
        sqlx::query_as::<_, UserCustomReportTemplate>(
            r#"
            SELECT id, user_id, base_template_id, name, description, conclusion, bodypart,
                   is_active, created_at, updated_at
            FROM user_custom_report_template
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    async fn find_custom_templates_by_user(
        &self,
        user_id: i32,
    ) -> Result<Vec<UserCustomReportTemplate>> {
        sqlx::query_as::<_, UserCustomReportTemplate>(
            r#"
            SELECT id, user_id, base_template_id, name, description, conclusion, bodypart,
                   is_active, created_at, updated_at
            FROM user_custom_report_template
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
    }

    async fn update_custom_template(
        &self,
        id: i32,
        update: &UpdateUserCustomReportTemplate,
    ) -> Result<UserCustomReportTemplate> {
        sqlx::query_as::<_, UserCustomReportTemplate>(
            r#"
            UPDATE user_custom_report_template
            SET
                name = COALESCE($2, name),
                description = COALESCE($3, description),
                conclusion = COALESCE($4, conclusion),
                bodypart = COALESCE($5, bodypart),
                is_active = COALESCE($6, is_active),
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $1
            RETURNING id, user_id, base_template_id, name, description, conclusion, bodypart,
                     is_active, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(&update.name)
        .bind(&update.description)
        .bind(&update.conclusion)
        .bind(&update.bodypart)
        .bind(&update.is_active)
        .fetch_one(&self.pool)
        .await
    }

    async fn delete_custom_template(&self, id: i32) -> Result<bool> {
        let result = sqlx::query("DELETE FROM user_custom_report_template WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    // ========== 커스텀 템플릿 모달리티 ==========

    async fn add_custom_modality(
        &self,
        custom_template_id: i32,
        modality: &str,
    ) -> Result<UserCustomTemplateModality> {
        sqlx::query_as::<_, UserCustomTemplateModality>(
            r#"
            INSERT INTO user_custom_template_modality (custom_template_id, modality)
            VALUES ($1, $2)
            ON CONFLICT (custom_template_id, modality) DO NOTHING
            RETURNING id, custom_template_id, modality
            "#,
        )
        .bind(custom_template_id)
        .bind(modality)
        .fetch_one(&self.pool)
        .await
    }

    async fn find_custom_modalities_by_template(
        &self,
        custom_template_id: i32,
    ) -> Result<Vec<UserCustomTemplateModality>> {
        sqlx::query_as::<_, UserCustomTemplateModality>(
            r#"
            SELECT id, custom_template_id, modality
            FROM user_custom_template_modality
            WHERE custom_template_id = $1
            ORDER BY modality
            "#,
        )
        .bind(custom_template_id)
        .fetch_all(&self.pool)
        .await
    }

    async fn remove_custom_modality(
        &self,
        custom_template_id: i32,
        modality: &str,
    ) -> Result<bool> {
        let result = sqlx::query(
            "DELETE FROM user_custom_template_modality WHERE custom_template_id = $1 AND modality = $2",
        )
        .bind(custom_template_id)
        .bind(modality)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    // ========== 커스텀 템플릿 이미지 ==========

    async fn add_custom_template_image(
        &self,
        new_image: &NewUserCustomTemplateImage,
    ) -> Result<UserCustomTemplateImage> {
        sqlx::query_as::<_, UserCustomTemplateImage>(
            r#"
            INSERT INTO user_custom_template_image (
                custom_template_id, image_path, image_url, file_size, mime_type,
                display_order, is_shared, uploaded_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, custom_template_id, image_path, image_url, file_size, mime_type,
                     display_order, is_shared, uploaded_by, created_at
            "#,
        )
        .bind(new_image.custom_template_id)
        .bind(&new_image.image_path)
        .bind(&new_image.image_url)
        .bind(new_image.file_size)
        .bind(&new_image.mime_type)
        .bind(new_image.display_order)
        .bind(new_image.is_shared)
        .bind(new_image.uploaded_by)
        .fetch_one(&self.pool)
        .await
    }

    async fn find_custom_template_images(
        &self,
        custom_template_id: i32,
    ) -> Result<Vec<UserCustomTemplateImage>> {
        sqlx::query_as::<_, UserCustomTemplateImage>(
            r#"
            SELECT id, custom_template_id, image_path, image_url, file_size, mime_type,
                   display_order, is_shared, uploaded_by, created_at
            FROM user_custom_template_image
            WHERE custom_template_id = $1
            ORDER BY display_order, created_at
            "#,
        )
        .bind(custom_template_id)
        .fetch_all(&self.pool)
        .await
    }

    async fn delete_custom_template_image(&self, image_id: i32) -> Result<bool> {
        let result = sqlx::query("DELETE FROM user_custom_template_image WHERE id = $1")
            .bind(image_id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    // ========== Report-가이드 매핑 ==========

    async fn add_report_guide(
        &self,
        new_guide: &NewSeriesUserReportGuide,
    ) -> Result<SeriesUserReportGuide> {
        sqlx::query_as::<_, SeriesUserReportGuide>(
            r#"
            INSERT INTO series_user_report_guide (report_id, template_id, custom_template_id, display_order)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (report_id, COALESCE(template_id, -1), COALESCE(custom_template_id, -1)) DO NOTHING
            RETURNING id, report_id, template_id, custom_template_id, display_order, created_at
            "#,
        )
        .bind(new_guide.report_id)
        .bind(new_guide.template_id)
        .bind(new_guide.custom_template_id)
        .bind(new_guide.display_order)
        .fetch_one(&self.pool)
        .await
    }

    async fn find_report_guides(
        &self,
        report_id: i32,
    ) -> Result<Vec<SeriesUserReportGuide>> {
        sqlx::query_as::<_, SeriesUserReportGuide>(
            r#"
            SELECT id, report_id, template_id, custom_template_id, display_order, created_at
            FROM series_user_report_guide
            WHERE report_id = $1
            ORDER BY display_order, created_at
            "#,
        )
        .bind(report_id)
        .fetch_all(&self.pool)
        .await
    }

    async fn delete_report_guide(&self, guide_id: i32) -> Result<bool> {
        let result = sqlx::query("DELETE FROM series_user_report_guide WHERE id = $1")
            .bind(guide_id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    async fn delete_all_report_guides(&self, report_id: i32) -> Result<usize> {
        let result = sqlx::query("DELETE FROM series_user_report_guide WHERE report_id = $1")
            .bind(report_id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() as usize)
    }

    fn pool(&self) -> &PgPool {
        &self.pool
    }
}

