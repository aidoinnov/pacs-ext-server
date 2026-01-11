use async_trait::async_trait;
use crate::domain::view_selection::ViewSelection;
use crate::domain::view_selection::SelectedSeries;
use crate::domain::ServiceError;

/// ViewSelection 비즈니스 로직을 처리하는 Service 트레이트
///
/// 이 트레이트는 ViewSelection과 관련된 비즈니스 규칙을 정의합니다.
/// 구체적인 구현은 Infrastructure 계층에서 제공됩니다.
#[async_trait]
pub trait ViewSelectionService: Send + Sync {
    /// ViewSelection을 생성합니다.
    ///
    /// # 매개변수
    /// - `series` - 선택된 Series 목록
    /// - `user_id` - 생성한 사용자 ID
    /// - `ttl_sec` - TTL (초 단위)
    ///
    /// # 반환값
    /// - `Ok(ViewSelection)` - 생성된 ViewSelection
    /// - `Err(ServiceError)` - 생성 실패
    async fn create_selection(
        &self,
        series: Vec<SelectedSeries>,
        user_id: i32,
        ttl_sec: u64,
    ) -> Result<ViewSelection, ServiceError>;

    /// Selection ID로 ViewSelection을 조회합니다.
    ///
    /// # 매개변수
    /// - `selection_id` - 조회할 Selection ID
    ///
    /// # 반환값
    /// - `Ok(Some(ViewSelection))` - Selection이 존재하는 경우
    /// - `Ok(None)` - Selection이 존재하지 않거나 만료된 경우
    /// - `Err(ServiceError)` - 조회 실패
    async fn get_selection(&self, selection_id: &str) -> Result<Option<ViewSelection>, ServiceError>;

    /// Selection의 TTL을 연장합니다 (touch).
    ///
    /// # 매개변수
    /// - `selection_id` - TTL을 연장할 Selection ID
    /// - `ttl_sec` - 새로운 TTL (초 단위)
    ///
    /// # 반환값
    /// - `Ok(())` - TTL 연장 성공
    /// - `Err(ServiceError)` - TTL 연장 실패 (Selection이 존재하지 않거나 만료된 경우)
    async fn extend_ttl(&self, selection_id: &str, ttl_sec: u64) -> Result<(), ServiceError>;

    /// Selection을 삭제합니다.
    ///
    /// # 매개변수
    /// - `selection_id` - 삭제할 Selection ID
    ///
    /// # 반환값
    /// - `Ok(())` - 삭제 성공
    /// - `Err(ServiceError)` - 삭제 실패
    async fn delete_selection(&self, selection_id: &str) -> Result<(), ServiceError>;
}



