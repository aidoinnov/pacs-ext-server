use async_trait::async_trait;
use crate::domain::view_selection::ViewSelection;
use crate::domain::view_selection::SelectedSeries;
use crate::domain::view_selection::repositories::ViewSelectionRepository;
use crate::domain::view_selection::services::ViewSelectionService;
use crate::domain::ServiceError;
use std::sync::Arc;

/// ViewSelectionService 구현체
pub struct ViewSelectionServiceImpl<R>
where
    R: ViewSelectionRepository,
{
    repository: Arc<R>,
    default_ttl_sec: u64,
}

impl<R> ViewSelectionServiceImpl<R>
where
    R: ViewSelectionRepository,
{
    /// 새로운 ViewSelectionServiceImpl을 생성합니다.
    ///
    /// # Arguments
    /// * `repository` - ViewSelectionRepository
    /// * `default_ttl_sec` - 기본 TTL (초 단위)
    pub fn new(repository: Arc<R>, default_ttl_sec: u64) -> Self {
        Self {
            repository,
            default_ttl_sec,
        }
    }

    /// Selection ID를 생성합니다.
    /// 형식: `sel_{random_hex}`
    fn generate_selection_id() -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let hex: String = (0..6)
            .map(|_| format!("{:x}", rng.gen_range(0..16)))
            .collect();
        format!("sel_{}", hex)
    }
}

#[async_trait]
impl<R> ViewSelectionService for ViewSelectionServiceImpl<R>
where
    R: ViewSelectionRepository + Send + Sync,
{
    async fn create_selection(
        &self,
        series: Vec<SelectedSeries>,
        user_id: i32,
        ttl_sec: u64,
    ) -> Result<ViewSelection, ServiceError> {
        // Selection ID 생성
        let selection_id = Self::generate_selection_id();

        // ViewSelection 생성
        let selection = ViewSelection::new(selection_id, series, user_id, ttl_sec);

        // 저장
        self.repository
            .save(&selection)
            .await
            .map_err(|e| ServiceError::DatabaseError(e))?;

        Ok(selection)
    }

    async fn get_selection(&self, selection_id: &str) -> Result<Option<ViewSelection>, ServiceError> {
        self.repository
            .find_by_id(selection_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e))
    }

    async fn extend_ttl(&self, selection_id: &str, ttl_sec: u64) -> Result<(), ServiceError> {
        self.repository
            .extend_ttl(selection_id, ttl_sec)
            .await
            .map_err(|e| {
                if e.contains("not found") || e.contains("expired") {
                    ServiceError::NotFound(format!("Selection {} not found or expired", selection_id))
                } else {
                    ServiceError::DatabaseError(e)
                }
            })
    }

    async fn delete_selection(&self, selection_id: &str) -> Result<(), ServiceError> {
        self.repository
            .delete(selection_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e))
    }
}


