//! # Study List View Repository
//!
//! Study List View 관련 데이터 접근 인터페이스를 정의합니다.

use crate::domain::entities::{
    DicomFieldDef, ExtFieldDef, NewStudyListView, NewStudyListViewField, StudyListView,
    StudyListViewField, UpdateStudyListView,
};
use async_trait::async_trait;

/// View 목록 조회 필터
#[derive(Debug, Clone, Default)]
pub struct ViewListFilter {
    pub scope_type: Option<String>,
    pub scope_id: Option<String>,
    pub owner_user_id: Option<String>,
    pub include_system: bool,
}

/// 필드 정의 조회 필터
#[derive(Debug, Clone, Default)]
pub struct FieldDefFilter {
    pub source: Option<String>,  // 'dicom' | 'extension'
    pub level: Option<String>,   // 'study' | 'series' | 'instance'
    pub sortable: Option<bool>,
    pub filterable: Option<bool>,
}

#[async_trait]
pub trait StudyListViewRepository: Send + Sync {
    // ========================================================================
    // View CRUD
    // ========================================================================
    
    /// View 목록 조회
    async fn find_views(&self, filter: &ViewListFilter) -> Result<Vec<StudyListView>, sqlx::Error>;
    
    /// View 상세 조회
    async fn find_view_by_id(&self, view_id: &str) -> Result<Option<StudyListView>, sqlx::Error>;
    
    /// View 생성
    async fn create_view(&self, new_view: &NewStudyListView) -> Result<StudyListView, sqlx::Error>;
    
    /// View 수정
    async fn update_view(
        &self,
        view_id: &str,
        update: &UpdateStudyListView,
    ) -> Result<Option<StudyListView>, sqlx::Error>;
    
    /// View 삭제
    async fn delete_view(&self, view_id: &str) -> Result<bool, sqlx::Error>;
    
    /// View 존재 여부 확인
    async fn exists_view(&self, view_id: &str) -> Result<bool, sqlx::Error>;

    // ========================================================================
    // View Field CRUD
    // ========================================================================
    
    /// View의 필드 목록 조회
    async fn find_view_fields(
        &self,
        view_id: &str,
    ) -> Result<Vec<StudyListViewField>, sqlx::Error>;
    
    /// View에 필드 추가 (배치)
    async fn create_view_fields(
        &self,
        fields: &[NewStudyListViewField],
    ) -> Result<(), sqlx::Error>;
    
    /// View의 필드 전체 교체
    async fn replace_view_fields(
        &self,
        view_id: &str,
        fields: &[NewStudyListViewField],
    ) -> Result<(), sqlx::Error>;
    
    /// View의 필드 전체 삭제
    async fn delete_view_fields(&self, view_id: &str) -> Result<(), sqlx::Error>;

    // ========================================================================
    // Field Definitions
    // ========================================================================
    
    /// DICOM 필드 정의 목록 조회
    async fn find_dicom_field_defs(
        &self,
        filter: &FieldDefFilter,
    ) -> Result<Vec<DicomFieldDef>, sqlx::Error>;
    
    /// Extension 필드 정의 목록 조회
    async fn find_ext_field_defs(
        &self,
        filter: &FieldDefFilter,
    ) -> Result<Vec<ExtFieldDef>, sqlx::Error>;
    
    /// DICOM 필드 정의 조회
    async fn find_dicom_field_def(
        &self,
        field_key: &str,
    ) -> Result<Option<DicomFieldDef>, sqlx::Error>;
    
    /// Extension 필드 정의 조회
    async fn find_ext_field_def(
        &self,
        field_key: &str,
    ) -> Result<Option<ExtFieldDef>, sqlx::Error>;

    // ========================================================================
    // Count
    // ========================================================================
    
    /// View 개수
    async fn count_views(&self, filter: &ViewListFilter) -> Result<i64, sqlx::Error>;
}

