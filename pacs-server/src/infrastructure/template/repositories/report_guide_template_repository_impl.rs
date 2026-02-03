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
            INSERT INTO report_guide_template (description, conclusion, bodypart, is_shared, created_by)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, description, conclusion, bodypart, is_shared, is_active, created_by, created_at, updated_at
            "#,
        )
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
            SELECT id, description, conclusion, bodypart, is_shared, is_active, created_by, created_at, updated_at
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
                SELECT id, description, conclusion, bodypart, is_shared, is_active, created_by, created_at, updated_at
                FROM report_guide_template
                WHERE is_active = $1
                ORDER BY created_at DESC
                "#,
            )
            .bind(active)
        } else {
            sqlx::query_as::<_, ReportGuideTemplate>(
                r#"
                SELECT id, description, conclusion, bodypart, is_shared, is_active, created_by, created_at, updated_at
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
                description = COALESCE($2, description),
                conclusion = COALESCE($3, conclusion),
                bodypart = COALESCE($4, bodypart),
                is_shared = COALESCE($5, is_shared),
                is_active = COALESCE($6, is_active),
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $1
            RETURNING id, description, conclusion, bodypart, is_shared, is_active, created_by, created_at, updated_at
            "#,
        )
        .bind(id)
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

    async fn delete_template_modalities_by_template(
        &self,
        template_id: i32,
    ) -> Result<u64> {
        let result = sqlx::query(
            "DELETE FROM report_guide_template_modality WHERE template_id = $1",
        )
        .bind(template_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    // ========== 독립적인 가이드 이미지 ==========

    async fn create_guide_image(
        &self,
        new_image: &NewGuideImage,
    ) -> Result<GuideImage> {
        sqlx::query_as::<_, GuideImage>(
            r#"
            INSERT INTO guide_image (
                image_path, image_url, file_size, mime_type, is_shared, uploaded_by
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, image_path, image_url, file_size, mime_type, is_shared, uploaded_by, created_at
            "#,
        )
        .bind(&new_image.image_path)
        .bind(&new_image.image_url)
        .bind(new_image.file_size)
        .bind(&new_image.mime_type)
        .bind(new_image.is_shared)
        .bind(new_image.uploaded_by)
        .fetch_one(&self.pool)
        .await
    }

    async fn find_guide_image_by_id(&self, id: i32) -> Result<Option<GuideImage>> {
        sqlx::query_as::<_, GuideImage>(
            r#"
            SELECT id, image_path, image_url, file_size, mime_type, is_shared, uploaded_by, created_at
            FROM guide_image
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    async fn find_guide_images_by_user(
        &self,
        user_id: i32,
        is_shared: Option<bool>,
    ) -> Result<Vec<GuideImage>> {
        let mut query = String::from(
            r#"
            SELECT id, image_path, image_url, file_size, mime_type, is_shared, uploaded_by, created_at
            FROM guide_image
            WHERE uploaded_by = $1
            "#,
        );

        if let Some(shared) = is_shared {
            query.push_str(&format!(" AND is_shared = {}", shared));
        }

        query.push_str(" ORDER BY created_at DESC");

        sqlx::query_as::<_, GuideImage>(&query)
            .bind(user_id)
            .fetch_all(&self.pool)
            .await
    }

    async fn update_guide_image_share_status(
        &self,
        image_id: i32,
        is_shared: bool,
    ) -> Result<GuideImage> {
        sqlx::query_as::<_, GuideImage>(
            r#"
            UPDATE guide_image
            SET is_shared = $2
            WHERE id = $1
            RETURNING id, image_path, image_url, file_size, mime_type, is_shared, uploaded_by, created_at
            "#,
        )
        .bind(image_id)
        .bind(is_shared)
        .fetch_one(&self.pool)
        .await
    }

    async fn delete_guide_image(&self, image_id: i32) -> Result<bool> {
        let result = sqlx::query("DELETE FROM guide_image WHERE id = $1")
            .bind(image_id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    // ========== 템플릿-이미지 매핑 ==========

    async fn create_template_image_mapping(
        &self,
        new_mapping: &NewTemplateImageMapping,
    ) -> Result<TemplateImageMapping> {
        sqlx::query_as::<_, TemplateImageMapping>(
            r#"
            INSERT INTO report_guide_template_image_mapping (template_id, image_id, display_order)
            VALUES ($1, $2, $3)
            RETURNING id, template_id, image_id, display_order, created_at
            "#,
        )
        .bind(new_mapping.template_id)
        .bind(new_mapping.image_id)
        .bind(new_mapping.display_order)
        .fetch_one(&self.pool)
        .await
    }

    async fn delete_template_image_mappings_by_template(
        &self,
        template_id: i32,
    ) -> Result<u64> {
        let result = sqlx::query(
            "DELETE FROM report_guide_template_image_mapping WHERE template_id = $1",
        )
        .bind(template_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    async fn find_template_image_mappings(
        &self,
        template_id: i32,
    ) -> Result<Vec<TemplateImageMapping>> {
        sqlx::query_as::<_, TemplateImageMapping>(
            r#"
            SELECT id, template_id, image_id, display_order, created_at
            FROM report_guide_template_image_mapping
            WHERE template_id = $1
            ORDER BY display_order
            "#,
        )
        .bind(template_id)
        .fetch_all(&self.pool)
        .await
    }

    async fn find_guide_images_by_template(
        &self,
        template_id: i32,
    ) -> Result<Vec<GuideImage>> {
        sqlx::query_as::<_, GuideImage>(
            r#"
            SELECT gi.id, gi.image_path, gi.image_url, gi.file_size, gi.mime_type,
                   gi.is_shared, gi.uploaded_by, gi.created_at
            FROM guide_image gi
            JOIN report_guide_template_image_mapping m ON gi.id = m.image_id
            WHERE m.template_id = $1
            ORDER BY m.display_order
            "#,
        )
        .bind(template_id)
        .fetch_all(&self.pool)
        .await
    }

    // ========== 커스텀 템플릿-이미지 매핑 ==========

    async fn create_custom_template_image_mapping(
        &self,
        new_mapping: &NewCustomTemplateImageMapping,
    ) -> Result<CustomTemplateImageMapping> {
        sqlx::query_as::<_, CustomTemplateImageMapping>(
            r#"
            INSERT INTO user_custom_template_image_mapping (custom_template_id, image_id, display_order)
            VALUES ($1, $2, $3)
            RETURNING id, custom_template_id, image_id, display_order, created_at
            "#,
        )
        .bind(new_mapping.custom_template_id)
        .bind(new_mapping.image_id)
        .bind(new_mapping.display_order)
        .fetch_one(&self.pool)
        .await
    }

    async fn delete_custom_template_image_mappings_by_template(
        &self,
        custom_template_id: i32,
    ) -> Result<u64> {
        let result = sqlx::query(
            "DELETE FROM user_custom_template_image_mapping WHERE custom_template_id = $1",
        )
        .bind(custom_template_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    async fn find_custom_template_image_mappings(
        &self,
        custom_template_id: i32,
    ) -> Result<Vec<CustomTemplateImageMapping>> {
        sqlx::query_as::<_, CustomTemplateImageMapping>(
            r#"
            SELECT id, custom_template_id, image_id, display_order, created_at
            FROM user_custom_template_image_mapping
            WHERE custom_template_id = $1
            ORDER BY display_order
            "#,
        )
        .bind(custom_template_id)
        .fetch_all(&self.pool)
        .await
    }

    async fn find_guide_images_by_custom_template(
        &self,
        custom_template_id: i32,
    ) -> Result<Vec<GuideImage>> {
        sqlx::query_as::<_, GuideImage>(
            r#"
            SELECT gi.id, gi.image_path, gi.image_url, gi.file_size, gi.mime_type,
                   gi.is_shared, gi.uploaded_by, gi.created_at
            FROM guide_image gi
            JOIN user_custom_template_image_mapping m ON gi.id = m.image_id
            WHERE m.custom_template_id = $1
            ORDER BY m.display_order
            "#,
        )
        .bind(custom_template_id)
        .fetch_all(&self.pool)
        .await
    }

    // ========== 템플릿 이미지 (기존 구조 - 하위 호환성) ==========

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

    async fn find_template_images_by_user(
        &self,
        user_id: i32,
    ) -> Result<Vec<ReportGuideTemplateImage>> {
        sqlx::query_as::<_, ReportGuideTemplateImage>(
            r#"
            SELECT id, template_id, image_path, image_url, file_size, mime_type,
                   display_order, is_shared, uploaded_by, created_at
            FROM report_guide_template_image
            WHERE uploaded_by = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
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
                user_id, base_template_id, description, conclusion, bodypart
            )
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, user_id, base_template_id, description, conclusion, bodypart,
                     is_active, created_at, updated_at
            "#,
        )
        .bind(new_template.user_id)
        .bind(new_template.base_template_id)
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
            SELECT id, user_id, base_template_id, description, conclusion, bodypart,
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
            SELECT id, user_id, base_template_id, description, conclusion, bodypart,
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
                description = COALESCE($2, description),
                conclusion = COALESCE($3, conclusion),
                bodypart = COALESCE($4, bodypart),
                is_active = COALESCE($5, is_active),
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $1
            RETURNING id, user_id, base_template_id, description, conclusion, bodypart,
                     is_active, created_at, updated_at
            "#,
        )
        .bind(id)
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

    async fn delete_custom_template_modalities_by_template(
        &self,
        custom_template_id: i32,
    ) -> Result<u64> {
        let result = sqlx::query(
            "DELETE FROM user_custom_template_modality WHERE custom_template_id = $1",
        )
        .bind(custom_template_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    async fn insert_custom_modality_ignore_conflict(
        &self,
        custom_template_id: i32,
        modality: &str,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO user_custom_template_modality (custom_template_id, modality)
            VALUES ($1, $2)
            ON CONFLICT (custom_template_id, modality) DO NOTHING
            "#,
        )
        .bind(custom_template_id)
        .bind(modality)
        .execute(&self.pool)
        .await?;
        Ok(())
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

    // ========== Report 이미지 스냅샷 ==========

    async fn insert_report_images(
        &self,
        report_id: i32,
        image_entries: &[(i32, i32)],
    ) -> Result<()> {
        for (image_id, display_order) in image_entries {
            sqlx::query(
                r#"
                INSERT INTO report_image (report_id, image_id, display_order)
                VALUES ($1, $2, $3)
                ON CONFLICT (report_id, image_id) DO NOTHING
                "#,
            )
            .bind(report_id)
            .bind(image_id)
            .bind(display_order)
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    async fn delete_report_images_by_report(&self, report_id: i32) -> Result<u64> {
        let result = sqlx::query("DELETE FROM report_image WHERE report_id = $1")
            .bind(report_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected())
    }

    async fn find_guide_images_by_report(
        &self,
        report_id: i32,
    ) -> Result<Vec<(GuideImage, i32)>> {
        #[derive(sqlx::FromRow)]
        struct ReportImageRow {
            id: i32,
            image_path: String,
            image_url: String,
            file_size: Option<i64>,
            mime_type: Option<String>,
            is_shared: bool,
            uploaded_by: i32,
            created_at: chrono::DateTime<chrono::Utc>,
            display_order: i32,
        }
        let rows = sqlx::query_as::<_, ReportImageRow>(
            r#"
            SELECT gi.id, gi.image_path, gi.image_url, gi.file_size, gi.mime_type,
                   gi.is_shared, gi.uploaded_by, gi.created_at,
                   ri.display_order
            FROM report_image ri
            JOIN guide_image gi ON gi.id = ri.image_id
            WHERE ri.report_id = $1
            ORDER BY ri.display_order, ri.created_at
            "#,
        )
        .bind(report_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| {
                (
                    GuideImage {
                        id: r.id,
                        image_path: r.image_path,
                        image_url: r.image_url,
                        file_size: r.file_size,
                        mime_type: r.mime_type,
                        is_shared: r.is_shared,
                        uploaded_by: r.uploaded_by,
                        created_at: r.created_at,
                    },
                    r.display_order,
                )
            })
            .collect())
    }

    fn pool(&self) -> &PgPool {
        &self.pool
    }
}

