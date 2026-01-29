use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 선택된 Series 정보
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SelectedSeries {
    /// Study UID
    pub study_uid: String,

    /// Series UID
    pub series_uid: String,
}

/// Viewport 레이아웃 정보
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ViewportLayout {
    /// 행 개수
    pub rows: u32,

    /// 열 개수
    pub cols: u32,
}

/// 초기 Viewport 설정
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InitialViewport {
    /// Viewport 행 위치 (0부터 시작)
    pub row: u32,

    /// Viewport 열 위치 (0부터 시작)
    pub col: u32,

    /// Study UID
    pub study_uid: String,

    /// Series UID
    pub series_uid: String,

    /// SOP Instance UID (선택적)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sop_uid: Option<String>,

    /// Frame Index (선택적, multi-frame 이미지용)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frame_index: Option<u32>,
}

/// Viewer Selection 엔티티
///
/// 여러 Study에 속한 Series를 선택하여 Viewer에서 출력하기 위한 선택 상태를 나타냅니다.
/// Selection ID를 통해 Viewer 상태를 재현할 수 있습니다.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ViewSelection {
    /// Selection ID (예: "sel_8f23ab")
    pub selection_id: String,

    /// 선택된 Series 목록
    pub series: Vec<SelectedSeries>,

    /// Viewport 레이아웃 (선택적)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout: Option<ViewportLayout>,

    /// 초기 Viewport 설정 목록 (선택적)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_views: Option<Vec<InitialViewport>>,

    /// 생성 시각
    pub created_at: DateTime<Utc>,

    /// 만료 시각 (TTL 기반)
    pub expires_at: DateTime<Utc>,

    /// 생성한 사용자 ID
    pub user_id: i32,
}

impl ViewSelection {
    /// 새로운 ViewSelection을 생성합니다.
    ///
    /// # Arguments
    /// * `selection_id` - Selection ID
    /// * `series` - 선택된 Series 목록
    /// * `layout` - Viewport 레이아웃 (선택적)
    /// * `initial_views` - 초기 Viewport 설정 목록 (선택적)
    /// * `user_id` - 생성한 사용자 ID
    /// * `ttl_sec` - TTL (초 단위)
    ///
    /// # Returns
    /// 생성된 ViewSelection
    pub fn new(
        selection_id: String,
        series: Vec<SelectedSeries>,
        layout: Option<ViewportLayout>,
        initial_views: Option<Vec<InitialViewport>>,
        user_id: i32,
        ttl_sec: u64,
    ) -> Self {
        let now = Utc::now();
        let expires_at = now + chrono::Duration::seconds(ttl_sec as i64);

        Self {
            selection_id,
            series,
            layout,
            initial_views,
            created_at: now,
            expires_at,
            user_id,
        }
    }

    /// ViewSelection의 유효성을 검증합니다.
    ///
    /// # Returns
    /// * `Ok(())` - 유효한 경우
    /// * `Err(String)` - 유효하지 않은 경우 (에러 메시지)
    pub fn validate(&self) -> Result<(), String> {
        // initial_views가 있으면 layout도 있어야 함
        if self.initial_views.is_some() && self.layout.is_none() {
            return Err("initial_views requires layout to be set".to_string());
        }

        // initial_views의 위치가 layout 범위 내에 있는지 확인
        if let (Some(ref views), Some(ref layout)) = (&self.initial_views, &self.layout) {
            for (idx, view) in views.iter().enumerate() {
                if view.row >= layout.rows {
                    return Err(format!(
                        "initial_views[{}]: row {} is out of layout bounds (rows: {})",
                        idx, view.row, layout.rows
                    ));
                }
                if view.col >= layout.cols {
                    return Err(format!(
                        "initial_views[{}]: col {} is out of layout bounds (cols: {})",
                        idx, view.col, layout.cols
                    ));
                }
            }
        }

        Ok(())
    }

    /// Selection이 만료되었는지 확인합니다.
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// TTL을 연장합니다.
    /// 
    /// # Arguments
    /// * `ttl_sec` - 새로운 TTL (초 단위)
    pub fn extend_ttl(&mut self, ttl_sec: u64) {
        self.expires_at = Utc::now() + chrono::Duration::seconds(ttl_sec as i64);
    }
}


