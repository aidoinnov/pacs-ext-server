//! # Infrastructure Layer
//! 
//! 외부 의존성을 담당하는 계층입니다.
//! 데이터베이스, 외부 API, 설정 등을 관리합니다.

pub mod config;
pub mod middleware;

// 공개 API
pub use config::*;
pub use middleware::*;
