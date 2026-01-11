//! # Study List View Use Case 모듈
//!
//! Study List View 관련 비즈니스 로직을 처리하는 Use Case입니다.

use crate::application::dto::study_list_view_dto::*;
use crate::domain::entities::{
    FieldDef, NewStudyListView, NewStudyListViewField, StudyListView, StudyListViewField,
    UpdateStudyListView,
};
use crate::domain::repositories::{FieldDefFilter, StudyListViewRepository, ViewListFilter};
use crate::domain::ServiceError;
use std::sync::Arc;

/// Study List View Use Case
pub struct StudyListViewUseCase<R>
where
    R: StudyListViewRepository,
{
    repository: Arc<R>,
}

impl<R> StudyListViewUseCase<R>
where
    R: StudyListViewRepository,
{
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // View CRUD
    // ========================================================================

    /// View 목록 조회
    pub async fn list_views(
        &self,
        query: &ViewListQuery,
        current_user_id: Option<&str>,
    ) -> Result<ViewListResponse, ServiceError> {
        let filter = ViewListFilter {
            scope_type: query.scope_type.clone(),
            scope_id: query.scope_id.clone(),
            owner_user_id: current_user_id.map(|s| s.to_string()),
            include_system: true,
        };

        let views = self.repository.find_views(&filter).await?;
        let total = self.repository.count_views(&filter).await?;

        let items: Vec<ViewResponse> = views.into_iter().map(ViewResponse::from).collect();

        Ok(ViewListResponse { items, total })
    }

    /// View 상세 조회
    pub async fn get_view(&self, view_id: &str) -> Result<ViewDetailResponse, ServiceError> {
        let view = self
            .repository
            .find_view_by_id(view_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound(format!("View not found: {}", view_id)))?;

        let fields = self.repository.find_view_fields(view_id).await?;

        // 필드 정보에 라벨 추가
        let field_responses = self.enrich_fields_with_labels(&fields).await?;

        Ok(ViewDetailResponse {
            view_id: view.view_id,
            view_name: view.view_name,
            is_system: view.is_system,
            owner_user_id: view.owner_user_id,
            scope_type: view.scope_type,
            scope_id: view.scope_id,
            description: view.description,
            created_at: view.created_at,
            updated_at: view.updated_at,
            fields: field_responses,
        })
    }

    /// View 생성
    pub async fn create_view(
        &self,
        request: &CreateViewRequest,
        current_user_id: &str,
    ) -> Result<ViewResponse, ServiceError> {
        // 중복 체크
        if self.repository.exists_view(&request.view_id).await? {
            return Err(ServiceError::ValidationFailed(format!(
                "View already exists: {}",
                request.view_id
            )));
        }

        let new_view = NewStudyListView {
            view_id: request.view_id.clone(),
            view_name: request.view_name.clone(),
            owner_user_id: Some(current_user_id.to_string()),
            scope_type: request.scope_type.clone(),
            scope_id: request.scope_id.clone(),
            description: request.description.clone(),
        };

        let view = self.repository.create_view(&new_view).await?;

        // 필드가 있으면 추가
        if let Some(fields) = &request.fields {
            let new_fields: Vec<NewStudyListViewField> = fields
                .iter()
                .map(|f| NewStudyListViewField {
                    view_id: view.view_id.clone(),
                    field_source: f.source.clone(),
                    field_key: f.key.clone(),
                    display_order: f.display_order,
                    visible: f.visible,
                    pinned: f.pinned,
                    width: f.width,
                    display_label: f.display_label.clone(),
                })
                .collect();
            self.repository.create_view_fields(&new_fields).await?;
        }

        Ok(ViewResponse::from(view))
    }

    /// View 수정
    pub async fn update_view(
        &self,
        view_id: &str,
        request: &UpdateViewRequest,
    ) -> Result<ViewResponse, ServiceError> {
        // View 존재 여부 체크
        let existing = self
            .repository
            .find_view_by_id(view_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound(format!("View not found: {}", view_id)))?;

        // 시스템 View는 필드만 수정 가능 (이름/설명 변경 불가)
        let update = if existing.is_system {
            UpdateStudyListView {
                view_name: None, // 시스템 View 이름 변경 불가
                description: None,
            }
        } else {
            UpdateStudyListView {
                view_name: request.view_name.clone(),
                description: request.description.clone(),
            }
        };

        // 시스템 View가 아닌 경우에만 메타데이터 업데이트
        let view = if !existing.is_system {
            self.repository
                .update_view(view_id, &update)
                .await?
                .ok_or_else(|| ServiceError::NotFound(format!("View not found: {}", view_id)))?
        } else {
            existing
        };

        // 필드 교체
        if let Some(fields) = &request.fields {
            let new_fields: Vec<NewStudyListViewField> = fields
                .iter()
                .map(|f| NewStudyListViewField {
                    view_id: view_id.to_string(),
                    field_source: f.source.clone(),
                    field_key: f.key.clone(),
                    display_order: f.display_order,
                    visible: f.visible,
                    pinned: f.pinned,
                    width: f.width,
                    display_label: f.display_label.clone(),
                })
                .collect();
            self.repository.replace_view_fields(view_id, &new_fields).await?;
        }

        Ok(ViewResponse::from(view))
    }

    /// View 삭제
    pub async fn delete_view(&self, view_id: &str) -> Result<(), ServiceError> {
        // 시스템 View 체크
        let existing = self
            .repository
            .find_view_by_id(view_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound(format!("View not found: {}", view_id)))?;

        if existing.is_system {
            return Err(ServiceError::Forbidden(
                "Cannot delete system view".to_string(),
            ));
        }

        self.repository.delete_view(view_id).await?;
        Ok(())
    }

    // ========================================================================
    // Field Definitions
    // ========================================================================

    /// 필드 정의 목록 조회
    pub async fn list_field_defs(
        &self,
        query: &FieldDefListQuery,
    ) -> Result<FieldDefListResponse, ServiceError> {
        let filter = FieldDefFilter {
            source: query.source.clone(),
            level: query.level.clone(),
            sortable: query.sortable,
            filterable: query.filterable,
        };

        let mut items: Vec<FieldDefResponse> = Vec::new();

        // source 필터에 따라 조회
        if query.source.is_none() || query.source.as_deref() == Some("dicom") {
            let dicom_defs = self.repository.find_dicom_field_defs(&filter).await?;
            items.extend(dicom_defs.into_iter().map(|d| FieldDefResponse::from(FieldDef::from(d))));
        }

        if query.source.is_none() || query.source.as_deref() == Some("extension") {
            let ext_defs = self.repository.find_ext_field_defs(&filter).await?;
            items.extend(ext_defs.into_iter().map(|e| FieldDefResponse::from(FieldDef::from(e))));
        }

        // default_order로 정렬
        items.sort_by(|a, b| {
            a.default_order.unwrap_or(999).cmp(&b.default_order.unwrap_or(999))
        });

        let total = items.len() as i64;

        Ok(FieldDefListResponse { items, total })
    }

    // ========================================================================
    // Helper Methods
    // ========================================================================

    /// 필드에 라벨 정보 추가
    async fn enrich_fields_with_labels(
        &self,
        fields: &[StudyListViewField],
    ) -> Result<Vec<ViewFieldResponse>, ServiceError> {
        let mut responses = Vec::new();

        for field in fields {
            // 원본 label 조회
            let original_label = if field.field_source == "dicom" {
                self.repository
                    .find_dicom_field_def(&field.field_key)
                    .await?
                    .map(|d| d.label)
                    .unwrap_or_else(|| field.field_key.clone())
            } else {
                self.repository
                    .find_ext_field_def(&field.field_key)
                    .await?
                    .map(|e| e.label)
                    .unwrap_or_else(|| field.field_key.clone())
            };

            // display_label이 있으면 그걸 label로, 없으면 원본 label 사용
            let label = field.display_label.clone().unwrap_or(original_label);

            responses.push(ViewFieldResponse {
                source: field.field_source.clone(),
                key: field.field_key.clone(),
                label,
                display_label: field.display_label.clone(),
                display_order: field.display_order,
                visible: field.visible,
                pinned: field.pinned,
                width: field.width,
            });
        }

        Ok(responses)
    }
}
