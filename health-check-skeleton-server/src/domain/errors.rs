//! # Domain Errors
//! 
//! 도메인 계층에서 사용되는 에러 타입들을 정의합니다.

use thiserror::Error;

/// 도메인 계층에서 발생할 수 있는 에러들
#[derive(Error, Debug)]
pub enum DomainError {
    /// 서버 상태가 비정상일 때
    #[error("Server is unhealthy: {reason}")]
    UnhealthyServer { reason: String },
    
    /// 잘못된 요청일 때
    #[error("Invalid request: {reason}")]
    InvalidRequest { reason: String },
    
    /// 내부 서버 에러
    #[error("Internal server error: {reason}")]
    InternalError { reason: String },
}

impl DomainError {
    /// 서버가 비정상 상태일 때의 에러 생성
    pub fn unhealthy(reason: impl Into<String>) -> Self {
        Self::UnhealthyServer {
            reason: reason.into(),
        }
    }
    
    /// 잘못된 요청일 때의 에러 생성
    pub fn invalid_request(reason: impl Into<String>) -> Self {
        Self::InvalidRequest {
            reason: reason.into(),
        }
    }
    
    /// 내부 서버 에러 생성
    pub fn internal(reason: impl Into<String>) -> Self {
        Self::InternalError {
            reason: reason.into(),
        }
    }
}
