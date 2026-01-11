//! # Study List View DTO
//!
//! Study List View API 요청/응답 DTO

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// ============================================================================
// View 관련 DTO
// ============================================================================

/// View 목록 조회 쿼리
#[derive(Debug, Deserialize, ToSchema)]
pub struct ViewListQuery {
    /// 범위 타입 필터 (project, user)
    #[serde(rename = "scopeType")]
    pub scope_type: Option<String>,
    /// 범위 ID 필터
    #[serde(rename = "scopeId")]
    pub scope_id: Option<String>,
}

/// View 생성 요청
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateViewRequest {
    /// View ID
    #[schema(example = "my-custom-view")]
    #[serde(rename = "viewId")]
    pub view_id: String,
    /// View 이름
    #[schema(example = "My Custom View")]
    #[serde(rename = "viewName")]
    pub view_name: String,
    /// 범위 타입
    #[serde(rename = "scopeType")]
    pub scope_type: Option<String>,
    /// 범위 ID
    #[serde(rename = "scopeId")]
    pub scope_id: Option<String>,
    /// 설명
    pub description: Option<String>,
    /// 필드 구성
    pub fields: Option<Vec<ViewFieldInput>>,
}

/// View 수정 요청
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateViewRequest {
    /// View 이름
    #[serde(rename = "viewName")]
    pub view_name: Option<String>,
    /// 설명
    pub description: Option<String>,
    /// 필드 구성 (전체 교체)
    pub fields: Option<Vec<ViewFieldInput>>,
}

/// View 필드 입력
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ViewFieldInput {
    /// 필드 소스 (dicom, extension)
    #[schema(example = "dicom")]
    pub source: String,
    /// 필드 키
    #[schema(example = "PatientName")]
    pub key: String,
    /// 표시 순서
    #[schema(example = 0)]
    #[serde(rename = "displayOrder")]
    pub display_order: i32,
    /// 표시 여부
    #[schema(example = true)]
    #[serde(default = "default_true")]
    pub visible: bool,
    /// 고정 여부
    #[serde(default)]
    pub pinned: bool,
    /// 너비
    pub width: Option<i32>,
    /// 사용자 정의 라벨 (없으면 원본 label 사용)
    #[serde(rename = "displayLabel")]
    pub display_label: Option<String>,
}

fn default_true() -> bool {
    true
}

/// View 목록 응답
#[derive(Debug, Serialize, ToSchema)]
pub struct ViewListResponse {
    /// View 목록
    pub items: Vec<ViewResponse>,
    /// 총 개수
    pub total: i64,
}

/// View 응답
#[derive(Debug, Serialize, ToSchema)]
pub struct ViewResponse {
    /// View ID
    #[serde(rename = "viewId")]
    pub view_id: String,
    /// View 이름
    #[serde(rename = "viewName")]
    pub view_name: String,
    /// 시스템 View 여부
    #[serde(rename = "isSystem")]
    pub is_system: bool,
    /// 소유자 ID
    #[serde(rename = "ownerUserId")]
    pub owner_user_id: Option<String>,
    /// 범위 타입
    #[serde(rename = "scopeType")]
    pub scope_type: Option<String>,
    /// 범위 ID
    #[serde(rename = "scopeId")]
    pub scope_id: Option<String>,
    /// 설명
    pub description: Option<String>,
    /// 생성일
    #[serde(rename = "createdAt")]
    #[schema(value_type = String)]
    pub created_at: DateTime<Utc>,
}

/// View 상세 응답 (필드 포함)
#[derive(Debug, Serialize, ToSchema)]
pub struct ViewDetailResponse {
    /// View ID
    #[serde(rename = "viewId")]
    pub view_id: String,
    /// View 이름
    #[serde(rename = "viewName")]
    pub view_name: String,
    /// 시스템 View 여부
    #[serde(rename = "isSystem")]
    pub is_system: bool,
    /// 소유자 ID
    #[serde(rename = "ownerUserId")]
    pub owner_user_id: Option<String>,
    /// 범위 타입
    #[serde(rename = "scopeType")]
    pub scope_type: Option<String>,
    /// 범위 ID
    #[serde(rename = "scopeId")]
    pub scope_id: Option<String>,
    /// 설명
    pub description: Option<String>,
    /// 생성일
    #[serde(rename = "createdAt")]
    #[schema(value_type = String)]
    pub created_at: DateTime<Utc>,
    /// 수정일
    #[serde(rename = "updatedAt")]
    #[schema(value_type = String)]
    pub updated_at: DateTime<Utc>,
    /// 필드 목록
    pub fields: Vec<ViewFieldResponse>,
}

/// View 필드 응답
#[derive(Debug, Serialize, ToSchema)]
pub struct ViewFieldResponse {
    /// 필드 소스
    pub source: String,
    /// 필드 키
    pub key: String,
    /// 표시 라벨 (displayLabel이 있으면 displayLabel, 없으면 원본 label)
    pub label: String,
    /// 사용자 정의 라벨 (None이면 원본 label 사용)
    #[serde(rename = "displayLabel", skip_serializing_if = "Option::is_none")]
    pub display_label: Option<String>,
    /// 표시 순서
    #[serde(rename = "displayOrder")]
    pub display_order: i32,
    /// 표시 여부
    pub visible: bool,
    /// 고정 여부
    pub pinned: bool,
    /// 너비
    pub width: Option<i32>,
}

// ============================================================================
// Field 정의 DTO
// ============================================================================

/// Field 정의 목록 조회 쿼리
#[derive(Debug, Deserialize, ToSchema)]
pub struct FieldDefListQuery {
    /// 소스 필터 (dicom, extension)
    pub source: Option<String>,
    /// 레벨 필터 (study, series, instance)
    pub level: Option<String>,
    /// 정렬 가능 여부 필터
    pub sortable: Option<bool>,
    /// 필터 가능 여부 필터
    pub filterable: Option<bool>,
}

/// Field 정의 목록 응답
#[derive(Debug, Serialize, ToSchema)]
pub struct FieldDefListResponse {
    /// 필드 정의 목록
    pub items: Vec<FieldDefResponse>,
    /// 총 개수
    pub total: i64,
}

/// Field 정의 응답
#[derive(Debug, Serialize, ToSchema)]
pub struct FieldDefResponse {
    /// 소스 (dicom, extension)
    pub source: String,
    /// 필드 키
    pub key: String,
    /// DICOM 태그 (dicom인 경우만)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    /// VR (dicom인 경우만)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vr: Option<String>,
    /// 표시 라벨
    pub label: String,
    /// 레벨
    pub level: String,
    /// 값 타입
    #[serde(rename = "valueType")]
    pub value_type: String,
    /// 설명
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// 소스 시스템 (extension인 경우만)
    #[serde(rename = "sourceSystem", skip_serializing_if = "Option::is_none")]
    pub source_system: Option<String>,
    /// 정렬 가능 여부
    pub sortable: bool,
    /// 필터 가능 여부
    pub filterable: bool,
    /// 기본 표시 여부
    #[serde(rename = "defaultVisible")]
    pub default_visible: bool,
    /// 기본 순서
    #[serde(rename = "defaultOrder", skip_serializing_if = "Option::is_none")]
    pub default_order: Option<i32>,
}

// ============================================================================
// 변환 구현
// ============================================================================

impl From<crate::domain::entities::StudyListView> for ViewResponse {
    fn from(v: crate::domain::entities::StudyListView) -> Self {
        Self {
            view_id: v.view_id,
            view_name: v.view_name,
            is_system: v.is_system,
            owner_user_id: v.owner_user_id,
            scope_type: v.scope_type,
            scope_id: v.scope_id,
            description: v.description,
            created_at: v.created_at,
        }
    }
}

impl From<crate::domain::entities::FieldDef> for FieldDefResponse {
    fn from(f: crate::domain::entities::FieldDef) -> Self {
        Self {
            source: f.source,
            key: f.key,
            tag: f.tag,
            vr: f.vr,
            label: f.label,
            level: f.level,
            value_type: f.value_type,
            description: f.description,
            source_system: f.source_system,
            sortable: f.sortable,
            filterable: f.filterable,
            default_visible: f.default_visible,
            default_order: f.default_order,
        }
    }
}
