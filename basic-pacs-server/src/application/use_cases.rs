//! # Use Cases
//! 
//! 애플리케이션 계층의 유스케이스들을 정의합니다.

use crate::domain::services::HealthCheckService;
use crate::domain::errors::DomainError;
use crate::application::dto::{HealthCheckResponse, ServerInfoResponse};

/// 헬스체크 유스케이스
pub struct HealthCheckUseCase {
    health_check_service: Box<dyn HealthCheckService + Send + Sync>,
}

impl HealthCheckUseCase {
    /// 새로운 헬스체크 유스케이스 생성
    pub fn new(health_check_service: Box<dyn HealthCheckService + Send + Sync>) -> Self {
        Self {
            health_check_service,
        }
    }
    
    /// 서버 상태 확인
    pub async fn check_health(&self) -> Result<HealthCheckResponse, DomainError> {
        let status = self.health_check_service.check_health().await?;
        Ok(HealthCheckResponse::from(status))
    }
    
    /// 서버 정보 조회
    pub async fn get_server_info(&self) -> Result<ServerInfoResponse, DomainError> {
        let info = self.health_check_service.get_server_info().await?;
        Ok(ServerInfoResponse::from(info))
    }
}
