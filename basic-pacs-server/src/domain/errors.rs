//! # Domain Errors
//! 
//! 도메인 계층에서 사용하는 에러 타입들을 정의합니다.

use thiserror::Error;

/// 도메인 에러 타입
#[derive(Error, Debug)]
pub enum DomainError {
    /// 서버 상태 에러
    #[error("Server is not healthy: {message}")]
    UnhealthyServer { message: String },
    
    /// 유효성 검증 에러
    #[error("Validation error: {field} - {message}")]
    ValidationError { field: String, message: String },
    
    /// 일반적인 도메인 에러
    #[error("Domain error: {message}")]
    General { message: String },
}

impl DomainError {
    /// 서버 상태 에러 생성
    pub fn unhealthy_server(message: impl Into<String>) -> Self {
        Self::UnhealthyServer {
            message: message.into(),
        }
    }
    
    /// 유효성 검증 에러 생성
    pub fn validation_error(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ValidationError {
            field: field.into(),
            message: message.into(),
        }
    }
    
    /// 일반 도메인 에러 생성
    pub fn general(message: impl Into<String>) -> Self {
        Self::General {
            message: message.into(),
        }
    }
}
