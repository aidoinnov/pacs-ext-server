//! # Study List View 엔티티 모듈
//!
//! Study List의 컬럼 구성(View)을 정의하는 엔티티들입니다.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// ============================================================================
// Study List View
// ============================================================================

/// Study List View (컬럼 프리셋)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct StudyListView {
    pub view_id: String,
    pub view_name: String,
    pub is_system: bool,
    pub owner_user_id: Option<String>,
    pub scope_type: Option<String>,
    pub scope_id: Option<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 새로운 View 생성을 위한 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewStudyListView {
    pub view_id: String,
    pub view_name: String,
    pub owner_user_id: Option<String>,
    pub scope_type: Option<String>,
    pub scope_id: Option<String>,
    pub description: Option<String>,
}

/// View 업데이트를 위한 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateStudyListView {
    pub view_name: Option<String>,
    pub description: Option<String>,
}

// ============================================================================
// Study List View Field
// ============================================================================

/// View에 포함되는 필드 구성
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct StudyListViewField {
    pub view_id: String,
    pub field_source: String,  // 'dicom' | 'extension'
    pub field_key: String,
    pub display_order: i32,
    pub visible: bool,
    pub pinned: bool,
    pub width: Option<i32>,
    pub display_label: Option<String>,  // 사용자 정의 라벨 (None이면 원본 label 사용)
    pub created_at: DateTime<Utc>,
}

/// 새로운 View Field 생성을 위한 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewStudyListViewField {
    pub view_id: String,
    pub field_source: String,
    pub field_key: String,
    pub display_order: i32,
    pub visible: bool,
    pub pinned: bool,
    pub width: Option<i32>,
    pub display_label: Option<String>,  // 사용자 정의 라벨 (None이면 원본 label 사용)
}

// ============================================================================
// DICOM Field Definition
// ============================================================================

/// DICOM 표준 필드 정의
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DicomFieldDef {
    pub field_key: String,
    pub tag: String,
    pub vr: String,
    pub label: String,
    pub level: String,
    pub value_type: String,
    pub description: Option<String>,
    pub sortable: bool,
    pub filterable: bool,
    pub default_visible: bool,
    pub default_order: Option<i32>,
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// Extension Field Definition
// ============================================================================

/// 확장 필드 정의
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ExtFieldDef {
    pub field_key: String,
    pub label: String,
    pub level: String,
    pub value_type: String,
    pub description: Option<String>,
    pub source_system: String,
    pub source_config: Option<serde_json::Value>,
    pub sortable: bool,
    pub filterable: bool,
    pub default_visible: bool,
    pub default_order: Option<i32>,
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// Unified Field Definition (for API response)
// ============================================================================

/// 통합 필드 정의 (API 응답용)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDef {
    pub source: String,  // 'dicom' | 'extension'
    pub key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vr: Option<String>,
    pub label: String,
    pub level: String,
    pub value_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_system: Option<String>,
    pub sortable: bool,
    pub filterable: bool,
    pub default_visible: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_order: Option<i32>,
}

impl From<DicomFieldDef> for FieldDef {
    fn from(d: DicomFieldDef) -> Self {
        Self {
            source: "dicom".to_string(),
            key: d.field_key,
            tag: Some(d.tag),
            vr: Some(d.vr),
            label: d.label,
            level: d.level,
            value_type: d.value_type,
            description: d.description,
            source_system: None,
            sortable: d.sortable,
            filterable: d.filterable,
            default_visible: d.default_visible,
            default_order: d.default_order,
        }
    }
}

impl From<ExtFieldDef> for FieldDef {
    fn from(e: ExtFieldDef) -> Self {
        Self {
            source: "extension".to_string(),
            key: e.field_key,
            tag: None,
            vr: None,
            label: e.label,
            level: e.level,
            value_type: e.value_type,
            description: e.description,
            source_system: Some(e.source_system),
            sortable: e.sortable,
            filterable: e.filterable,
            default_visible: e.default_visible,
            default_order: e.default_order,
        }
    }
}

