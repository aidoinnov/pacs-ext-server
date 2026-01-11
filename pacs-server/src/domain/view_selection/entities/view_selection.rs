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
    /// * `user_id` - 생성한 사용자 ID
    /// * `ttl_sec` - TTL (초 단위)
    /// 
    /// # Returns
    /// 생성된 ViewSelection
    pub fn new(
        selection_id: String,
        series: Vec<SelectedSeries>,
        user_id: i32,
        ttl_sec: u64,
    ) -> Self {
        let now = Utc::now();
        let expires_at = now + chrono::Duration::seconds(ttl_sec as i64);
        
        Self {
            selection_id,
            series,
            created_at: now,
            expires_at,
            user_id,
        }
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


