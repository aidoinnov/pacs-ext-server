use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// ViewSelection 생성 요청 DTO
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateViewSelectionRequest {
    /// 선택된 Series 목록
    #[schema(example = json!([{"study_uid": "1.2.3", "series_uid": "1.2.3.4"}]))]
    pub series: Vec<SelectedSeriesDto>,

    /// Viewport 레이아웃 (선택적)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = json!({"rows": 2, "cols": 2}))]
    pub layout: Option<ViewportLayoutDto>,

    /// 초기 Viewport 설정 목록 (선택적)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_views: Option<Vec<InitialViewportDto>>,
}

/// 선택된 Series 정보 DTO
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub struct SelectedSeriesDto {
    /// Study UID
    #[schema(example = "1.2.840.113619.2.55.3.604641477.123.1234567890.123")]
    pub study_uid: String,

    /// Series UID
    #[schema(example = "1.2.840.113619.2.55.3.604641477.123.1234567890.124")]
    pub series_uid: String,
}

/// Viewport 레이아웃 DTO
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub struct ViewportLayoutDto {
    /// 행 개수
    #[schema(example = 2)]
    pub rows: u32,

    /// 열 개수
    #[schema(example = 2)]
    pub cols: u32,
}

/// 초기 Viewport 설정 DTO
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub struct InitialViewportDto {
    /// Viewport 행 위치 (0부터 시작)
    #[schema(example = 0)]
    pub row: u32,

    /// Viewport 열 위치 (0부터 시작)
    #[schema(example = 0)]
    pub col: u32,

    /// Study UID
    #[schema(example = "1.2.840.113619.2.55.3.604641477.123")]
    pub study_uid: String,

    /// Series UID
    #[schema(example = "1.2.840.113619.2.55.3.604641477.124")]
    pub series_uid: String,

    /// SOP Instance UID (선택적)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "1.2.840.113619.2.55.3.604641477.125")]
    pub sop_uid: Option<String>,

    /// Frame Index (선택적, multi-frame 이미지용)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = 5)]
    pub frame_index: Option<u32>,
}

/// ViewSelection 생성 응답 DTO
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateViewSelectionResponse {
    /// 생성된 Selection ID
    #[schema(example = "sel_8f23ab")]
    pub selection_id: String,
}

/// ViewSelection 조회 응답 DTO
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ViewSelectionResponse {
    /// Selection ID
    #[schema(example = "sel_8f23ab")]
    pub selection_id: String,

    /// 선택된 Series 목록
    pub series: Vec<SelectedSeriesDto>,

    /// Viewport 레이아웃 (선택적)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout: Option<ViewportLayoutDto>,

    /// 초기 Viewport 설정 목록 (선택적)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_views: Option<Vec<InitialViewportDto>>,

    /// 생성 시각
    #[schema(value_type = String, example = "2025-01-15T10:00:00Z")]
    pub created_at: DateTime<Utc>,

    /// 만료 시각
    #[schema(value_type = String, example = "2025-01-15T10:30:00Z")]
    pub expires_at: DateTime<Utc>,

    /// 생성한 사용자 ID
    pub user_id: i32,
}

impl From<crate::domain::view_selection::SelectedSeries> for SelectedSeriesDto {
    fn from(series: crate::domain::view_selection::SelectedSeries) -> Self {
        Self {
            study_uid: series.study_uid,
            series_uid: series.series_uid,
        }
    }
}

impl From<SelectedSeriesDto> for crate::domain::view_selection::SelectedSeries {
    fn from(dto: SelectedSeriesDto) -> Self {
        Self {
            study_uid: dto.study_uid,
            series_uid: dto.series_uid,
        }
    }
}

// ViewportLayout 변환
impl From<crate::domain::view_selection::ViewportLayout> for ViewportLayoutDto {
    fn from(layout: crate::domain::view_selection::ViewportLayout) -> Self {
        Self {
            rows: layout.rows,
            cols: layout.cols,
        }
    }
}

impl From<ViewportLayoutDto> for crate::domain::view_selection::ViewportLayout {
    fn from(dto: ViewportLayoutDto) -> Self {
        Self {
            rows: dto.rows,
            cols: dto.cols,
        }
    }
}

// InitialViewport 변환
impl From<crate::domain::view_selection::InitialViewport> for InitialViewportDto {
    fn from(viewport: crate::domain::view_selection::InitialViewport) -> Self {
        Self {
            row: viewport.row,
            col: viewport.col,
            study_uid: viewport.study_uid,
            series_uid: viewport.series_uid,
            sop_uid: viewport.sop_uid,
            frame_index: viewport.frame_index,
        }
    }
}

impl From<InitialViewportDto> for crate::domain::view_selection::InitialViewport {
    fn from(dto: InitialViewportDto) -> Self {
        Self {
            row: dto.row,
            col: dto.col,
            study_uid: dto.study_uid,
            series_uid: dto.series_uid,
            sop_uid: dto.sop_uid,
            frame_index: dto.frame_index,
        }
    }
}

impl From<crate::domain::view_selection::ViewSelection> for ViewSelectionResponse {
    fn from(selection: crate::domain::view_selection::ViewSelection) -> Self {
        Self {
            selection_id: selection.selection_id,
            series: selection.series.into_iter().map(SelectedSeriesDto::from).collect(),
            layout: selection.layout.map(ViewportLayoutDto::from),
            initial_views: selection.initial_views.map(|views| {
                views.into_iter().map(InitialViewportDto::from).collect()
            }),
            created_at: selection.created_at,
            expires_at: selection.expires_at,
            user_id: selection.user_id,
        }
    }
}


