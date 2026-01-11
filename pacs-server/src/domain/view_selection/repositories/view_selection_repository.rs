use async_trait::async_trait;
use crate::domain::view_selection::ViewSelection;

/// ViewSelection 데이터 접근을 위한 Repository 트레이트
///
/// 이 트레이트는 ViewSelection의 데이터 접근 로직을 추상화합니다.
/// 구체적인 구현은 Infrastructure 계층에서 제공됩니다 (Redis 기반).
#[async_trait]
pub trait ViewSelectionRepository: Send + Sync {
    /// ViewSelection을 저장합니다.
    ///
    /// # 매개변수
    /// - `selection`: 저장할 ViewSelection
    ///
    /// # 반환값
    /// - `Ok(())`: 저장 성공
    /// - `Err(String)`: 저장 실패 (에러 메시지)
    async fn save(&self, selection: &ViewSelection) -> Result<(), String>;

    /// Selection ID로 ViewSelection을 조회합니다.
    ///
    /// # 매개변수
    /// - `selection_id`: 조회할 Selection ID
    ///
    /// # 반환값
    /// - `Ok(Some(ViewSelection))`: Selection이 존재하는 경우
    /// - `Ok(None)`: Selection이 존재하지 않는 경우
    /// - `Err(String)`: 조회 실패 (에러 메시지)
    async fn find_by_id(&self, selection_id: &str) -> Result<Option<ViewSelection>, String>;

    /// Selection의 TTL을 연장합니다 (touch).
    ///
    /// # 매개변수
    /// - `selection_id`: TTL을 연장할 Selection ID
    /// - `ttl_sec`: 새로운 TTL (초 단위)
    ///
    /// # 반환값
    /// - `Ok(())`: TTL 연장 성공
    /// - `Err(String)`: TTL 연장 실패 (에러 메시지)
    async fn extend_ttl(&self, selection_id: &str, ttl_sec: u64) -> Result<(), String>;

    /// Selection을 삭제합니다.
    ///
    /// # 매개변수
    /// - `selection_id`: 삭제할 Selection ID
    ///
    /// # 반환값
    /// - `Ok(())`: 삭제 성공
    /// - `Err(String)`: 삭제 실패 (에러 메시지)
    async fn delete(&self, selection_id: &str) -> Result<(), String>;
}



