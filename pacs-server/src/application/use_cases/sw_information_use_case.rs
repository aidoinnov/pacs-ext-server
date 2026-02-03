//! # SW Information Use Case
//!
//! SW Information 조회 비즈니스 로직

use crate::application::dto::sw_information_dto::{SwInformationListResponse, SwInformationResponse};
use crate::domain::sw_information::SwInformationRepository;
use crate::domain::ServiceError;
use std::sync::Arc;

/// SW Information Use Case
pub struct SwInformationUseCase<R>
where
    R: SwInformationRepository,
{
    repository: Arc<R>,
}

impl<R> SwInformationUseCase<R>
where
    R: SwInformationRepository,
{
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    /// 목록 조회
    pub async fn list(&self) -> Result<SwInformationListResponse, ServiceError> {
        let items = self
            .repository
            .find_all()
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        let total_count = items.len() as i64;
        let items: Vec<SwInformationResponse> = items.into_iter().map(SwInformationResponse::from).collect();

        Ok(SwInformationListResponse {
            success: true,
            items,
            total_count,
        })
    }

    /// ID로 상세 조회
    pub async fn get_by_id(&self, id: i32) -> Result<Option<SwInformationResponse>, ServiceError> {
        let opt = self
            .repository
            .find_by_id(id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        Ok(opt.map(SwInformationResponse::from))
    }
}
