use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// ViewSelection 생성 요청 DTO
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateViewSelectionRequest {
    /// 선택된 Series 목록
    #[schema(example = json!([{"study_uid": "1.2.3", "series_uid": "1.2.3.4"}]))]
    pub series: Vec<SelectedSeriesDto>,
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

impl From<crate::domain::view_selection::ViewSelection> for ViewSelectionResponse {
    fn from(selection: crate::domain::view_selection::ViewSelection) -> Self {
        Self {
            selection_id: selection.selection_id,
            series: selection.series.into_iter().map(SelectedSeriesDto::from).collect(),
            created_at: selection.created_at,
            expires_at: selection.expires_at,
            user_id: selection.user_id,
        }
    }
}


