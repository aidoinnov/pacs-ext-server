//! # Template Domain Module
//!
//! 이 모듈은 Report Guide Template 도메인을 담당하는 Bounded Context입니다.
//! 단일 책임 원칙에 따라 템플릿 관리와 관련된 모든 도메인 로직을 포함합니다.

pub mod entities;
pub mod repositories;
pub mod services;

pub use entities::*;
pub use repositories::*;
pub use services::*;





