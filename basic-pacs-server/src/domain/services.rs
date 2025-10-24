//! # Domain Services
//! 
//! 도메인 계층의 서비스 인터페이스들을 정의합니다.

use crate::domain::entities::{ServerStatus, ServerInfo};
use crate::domain::errors::DomainError;

/// 헬스체크 서비스 인터페이스
pub trait HealthCheckService {
    /// 서버 상태 확인
    async fn check_health(&self) -> Result<ServerStatus, DomainError>;
    
    /// 서버 정보 조회
    async fn get_server_info(&self) -> Result<ServerInfo, DomainError>;
}

/// 헬스체크 서비스 구현체
pub struct HealthCheckServiceImpl;

impl HealthCheckServiceImpl {
    /// 새로운 헬스체크 서비스 생성
    pub fn new() -> Self {
        Self
    }
}

impl HealthCheckService for HealthCheckServiceImpl {
    async fn check_health(&self) -> Result<ServerStatus, DomainError> {
        // 간단한 헬스체크 로직
        // 실제 환경에서는 데이터베이스 연결, 외부 서비스 상태 등을 확인
        Ok(ServerStatus::new("healthy", "basic-pacs-server", env!("CARGO_PKG_VERSION")))
    }
    
    async fn get_server_info(&self) -> Result<ServerInfo, DomainError> {
        Ok(ServerInfo::new(
            "Basic PACS Server",
            env!("CARGO_PKG_VERSION"),
            "Health check만 포함하는 기초 백엔드 서버"
        ))
    }
}
