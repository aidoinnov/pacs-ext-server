//! # Study List View Repository Implementation
//!
//! SQLx 기반 Study List View Repository 구현

use crate::domain::entities::{
    DicomFieldDef, ExtFieldDef, NewStudyListView, NewStudyListViewField, StudyListView,
    StudyListViewField, UpdateStudyListView,
};
use crate::domain::repositories::{FieldDefFilter, StudyListViewRepository, ViewListFilter};
use async_trait::async_trait;
use sqlx::PgPool;

#[derive(Clone)]
pub struct StudyListViewRepositoryImpl {
    pool: PgPool,
}

impl StudyListViewRepositoryImpl {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl StudyListViewRepository for StudyListViewRepositoryImpl {
    // ========================================================================
    // View CRUD
    // ========================================================================

    async fn find_views(&self, filter: &ViewListFilter) -> Result<Vec<StudyListView>, sqlx::Error> {
        let mut query = String::from(
            r#"
            SELECT view_id, view_name, is_system, owner_user_id, scope_type, scope_id,
                   description, created_at, updated_at
            FROM study_list_view
            WHERE 1=1
            "#,
        );

        if !filter.include_system {
            query.push_str(" AND is_system = false");
        }

        if filter.scope_type.is_some() {
            query.push_str(" AND scope_type = $1");
        }
        if filter.scope_id.is_some() {
            query.push_str(" AND scope_id = $2");
        }
        if filter.owner_user_id.is_some() {
            query.push_str(" AND owner_user_id = $3");
        }

        query.push_str(" ORDER BY is_system DESC, created_at DESC");

        // Dynamic query building with sqlx
        sqlx::query_as::<_, StudyListView>(&query)
            .bind(&filter.scope_type)
            .bind(&filter.scope_id)
            .bind(&filter.owner_user_id)
            .fetch_all(&self.pool)
            .await
    }

    async fn find_view_by_id(&self, view_id: &str) -> Result<Option<StudyListView>, sqlx::Error> {
        sqlx::query_as::<_, StudyListView>(
            r#"
            SELECT view_id, view_name, is_system, owner_user_id, scope_type, scope_id,
                   description, created_at, updated_at
            FROM study_list_view
            WHERE view_id = $1
            "#,
        )
        .bind(view_id)
        .fetch_optional(&self.pool)
        .await
    }

    async fn create_view(&self, new_view: &NewStudyListView) -> Result<StudyListView, sqlx::Error> {
        sqlx::query_as::<_, StudyListView>(
            r#"
            INSERT INTO study_list_view (view_id, view_name, owner_user_id, scope_type, scope_id, description)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING view_id, view_name, is_system, owner_user_id, scope_type, scope_id,
                      description, created_at, updated_at
            "#,
        )
        .bind(&new_view.view_id)
        .bind(&new_view.view_name)
        .bind(&new_view.owner_user_id)
        .bind(&new_view.scope_type)
        .bind(&new_view.scope_id)
        .bind(&new_view.description)
        .fetch_one(&self.pool)
        .await
    }

    async fn update_view(
        &self,
        view_id: &str,
        update: &UpdateStudyListView,
    ) -> Result<Option<StudyListView>, sqlx::Error> {
        sqlx::query_as::<_, StudyListView>(
            r#"
            UPDATE study_list_view
            SET view_name = COALESCE($2, view_name),
                description = COALESCE($3, description),
                updated_at = NOW()
            WHERE view_id = $1 AND is_system = false
            RETURNING view_id, view_name, is_system, owner_user_id, scope_type, scope_id,
                      description, created_at, updated_at
            "#,
        )
        .bind(view_id)
        .bind(&update.view_name)
        .bind(&update.description)
        .fetch_optional(&self.pool)
        .await
    }

    async fn delete_view(&self, view_id: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM study_list_view
            WHERE view_id = $1 AND is_system = false
            "#,
        )
        .bind(view_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    async fn exists_view(&self, view_id: &str) -> Result<bool, sqlx::Error> {
        let result: (bool,) = sqlx::query_as(
            r#"SELECT EXISTS(SELECT 1 FROM study_list_view WHERE view_id = $1)"#,
        )
        .bind(view_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(result.0)
    }

    // ========================================================================
    // View Field CRUD
    // ========================================================================

    async fn find_view_fields(
        &self,
        view_id: &str,
    ) -> Result<Vec<StudyListViewField>, sqlx::Error> {
        sqlx::query_as::<_, StudyListViewField>(
            r#"
            SELECT view_id, field_source, field_key, display_order, visible, pinned, width, display_label, created_at
            FROM study_list_view_field
            WHERE view_id = $1
            ORDER BY display_order
            "#,
        )
        .bind(view_id)
        .fetch_all(&self.pool)
        .await
    }

    async fn create_view_fields(
        &self,
        fields: &[NewStudyListViewField],
    ) -> Result<(), sqlx::Error> {
        for field in fields {
            sqlx::query(
                r#"
                INSERT INTO study_list_view_field (view_id, field_source, field_key, display_order, visible, pinned, width, display_label)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                ON CONFLICT (view_id, field_source, field_key) DO UPDATE
                SET display_order = EXCLUDED.display_order,
                    visible = EXCLUDED.visible,
                    pinned = EXCLUDED.pinned,
                    width = EXCLUDED.width,
                    display_label = EXCLUDED.display_label
                "#,
            )
            .bind(&field.view_id)
            .bind(&field.field_source)
            .bind(&field.field_key)
            .bind(field.display_order)
            .bind(field.visible)
            .bind(field.pinned)
            .bind(field.width)
            .bind(&field.display_label)
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    async fn replace_view_fields(
        &self,
        view_id: &str,
        fields: &[NewStudyListViewField],
    ) -> Result<(), sqlx::Error> {
        // Transaction으로 처리
        let mut tx = self.pool.begin().await?;

        // 기존 필드 삭제
        sqlx::query("DELETE FROM study_list_view_field WHERE view_id = $1")
            .bind(view_id)
            .execute(&mut *tx)
            .await?;

        // 새 필드 추가
        for field in fields {
            sqlx::query(
                r#"
                INSERT INTO study_list_view_field (view_id, field_source, field_key, display_order, visible, pinned, width, display_label)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                "#,
            )
            .bind(&field.view_id)
            .bind(&field.field_source)
            .bind(&field.field_key)
            .bind(field.display_order)
            .bind(field.visible)
            .bind(field.pinned)
            .bind(field.width)
            .bind(&field.display_label)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    async fn delete_view_fields(&self, view_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM study_list_view_field WHERE view_id = $1")
            .bind(view_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // ========================================================================
    // Field Definitions
    // ========================================================================

    async fn find_dicom_field_defs(
        &self,
        filter: &FieldDefFilter,
    ) -> Result<Vec<DicomFieldDef>, sqlx::Error> {
        let mut query = String::from(
            r#"
            SELECT field_key, tag, vr, label, level, value_type, description,
                   sortable, filterable, default_visible, default_order, created_at
            FROM dicom_field_def
            WHERE 1=1
            "#,
        );

        if filter.level.is_some() {
            query.push_str(" AND level = $1");
        }
        if filter.sortable.is_some() {
            query.push_str(" AND sortable = $2");
        }
        if filter.filterable.is_some() {
            query.push_str(" AND filterable = $3");
        }

        query.push_str(" ORDER BY default_order NULLS LAST, field_key");

        sqlx::query_as::<_, DicomFieldDef>(&query)
            .bind(&filter.level)
            .bind(filter.sortable)
            .bind(filter.filterable)
            .fetch_all(&self.pool)
            .await
    }

    async fn find_ext_field_defs(
        &self,
        filter: &FieldDefFilter,
    ) -> Result<Vec<ExtFieldDef>, sqlx::Error> {
        let mut query = String::from(
            r#"
            SELECT field_key, label, level, value_type, description, source_system,
                   source_config, sortable, filterable, default_visible, default_order, created_at
            FROM ext_field_def
            WHERE 1=1
            "#,
        );

        if filter.level.is_some() {
            query.push_str(" AND level = $1");
        }
        if filter.sortable.is_some() {
            query.push_str(" AND sortable = $2");
        }
        if filter.filterable.is_some() {
            query.push_str(" AND filterable = $3");
        }

        query.push_str(" ORDER BY default_order NULLS LAST, field_key");

        sqlx::query_as::<_, ExtFieldDef>(&query)
            .bind(&filter.level)
            .bind(filter.sortable)
            .bind(filter.filterable)
            .fetch_all(&self.pool)
            .await
    }

    async fn find_dicom_field_def(
        &self,
        field_key: &str,
    ) -> Result<Option<DicomFieldDef>, sqlx::Error> {
        sqlx::query_as::<_, DicomFieldDef>(
            r#"
            SELECT field_key, tag, vr, label, level, value_type, description,
                   sortable, filterable, default_visible, default_order, created_at
            FROM dicom_field_def
            WHERE field_key = $1
            "#,
        )
        .bind(field_key)
        .fetch_optional(&self.pool)
        .await
    }

    async fn find_ext_field_def(
        &self,
        field_key: &str,
    ) -> Result<Option<ExtFieldDef>, sqlx::Error> {
        sqlx::query_as::<_, ExtFieldDef>(
            r#"
            SELECT field_key, label, level, value_type, description, source_system,
                   source_config, sortable, filterable, default_visible, default_order, created_at
            FROM ext_field_def
            WHERE field_key = $1
            "#,
        )
        .bind(field_key)
        .fetch_optional(&self.pool)
        .await
    }

    // ========================================================================
    // Count
    // ========================================================================

    async fn count_views(&self, filter: &ViewListFilter) -> Result<i64, sqlx::Error> {
        let mut query = String::from("SELECT COUNT(*) FROM study_list_view WHERE 1=1");

        if !filter.include_system {
            query.push_str(" AND is_system = false");
        }
        if filter.scope_type.is_some() {
            query.push_str(" AND scope_type = $1");
        }
        if filter.scope_id.is_some() {
            query.push_str(" AND scope_id = $2");
        }
        if filter.owner_user_id.is_some() {
            query.push_str(" AND owner_user_id = $3");
        }

        let result: (i64,) = sqlx::query_as(&query)
            .bind(&filter.scope_type)
            .bind(&filter.scope_id)
            .bind(&filter.owner_user_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(result.0)
    }

    async fn get_views_updated_at(&self, filter: &ViewListFilter) -> Result<chrono::DateTime<chrono::Utc>, sqlx::Error> {
        // View 목록 중 가장 최근 updated_at 조회
        let mut query = String::from(
            "SELECT COALESCE(MAX(updated_at), '1970-01-01'::timestamptz) FROM study_list_view WHERE 1=1"
        );

        if !filter.include_system {
            query.push_str(" AND is_system = false");
        }
        if filter.scope_type.is_some() {
            query.push_str(" AND scope_type = $1");
        }
        if filter.scope_id.is_some() {
            query.push_str(" AND scope_id = $2");
        }
        if filter.owner_user_id.is_some() {
            query.push_str(" AND owner_user_id = $3");
        }

        let updated_at = sqlx::query_scalar::<_, chrono::DateTime<chrono::Utc>>(&query)
            .bind(&filter.scope_type)
            .bind(&filter.scope_id)
            .bind(&filter.owner_user_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(updated_at)
    }
}

