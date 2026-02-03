//! # SW Information Repository 트레이트
//!
//! SW Information 데이터 접근을 위한 Repository 인터페이스

use crate::domain::sw_information::entities::SwInformation;
use async_trait::async_trait;

#[async_trait]
pub trait SwInformationRepository: Send + Sync {
    /// 전체 목록 조회
    async fn find_all(&self) -> Result<Vec<SwInformation>, sqlx::Error>;

    /// ID로 상세 조회
    async fn find_by_id(&self, id: i32) -> Result<Option<SwInformation>, sqlx::Error>;
}
