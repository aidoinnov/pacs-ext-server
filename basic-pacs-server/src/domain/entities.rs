//! # Domain Entities
//! 
//! 도메인 계층의 핵심 엔티티들을 정의합니다.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// 서버 상태를 나타내는 엔티티
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStatus {
    /// 서버 상태 (healthy, unhealthy)
    pub status: String,
    /// 서비스 이름
    pub service: String,
    /// 버전
    pub version: String,
    /// 타임스탬프
    pub timestamp: DateTime<Utc>,
    /// 추가 메타데이터
    pub metadata: Option<serde_json::Value>,
}

impl ServerStatus {
    /// 새로운 서버 상태 생성
    pub fn new(status: impl Into<String>, service: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            status: status.into(),
            service: service.into(),
            version: version.into(),
            timestamp: Utc::now(),
            metadata: None,
        }
    }
    
    /// 메타데이터와 함께 서버 상태 생성
    pub fn with_metadata(
        status: impl Into<String>, 
        service: impl Into<String>, 
        version: impl Into<String>,
        metadata: serde_json::Value
    ) -> Self {
        Self {
            status: status.into(),
            service: service.into(),
            version: version.into(),
            timestamp: Utc::now(),
            metadata: Some(metadata),
        }
    }
    
    /// 서버가 정상 상태인지 확인
    pub fn is_healthy(&self) -> bool {
        self.status == "healthy"
    }
}

/// 서버 정보를 나타내는 엔티티
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    /// 서버 이름
    pub name: String,
    /// 버전
    pub version: String,
    /// 설명
    pub description: String,
    /// 아키텍처
    pub architecture: String,
    /// 프레임워크
    pub framework: String,
    /// 언어
    pub language: String,
    /// 타임스탬프
    pub timestamp: DateTime<Utc>,
}

impl ServerInfo {
    /// 새로운 서버 정보 생성
    pub fn new(
        name: impl Into<String>,
        version: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            description: description.into(),
            architecture: "Clean Architecture".to_string(),
            framework: "Actix Web".to_string(),
            language: "Rust".to_string(),
            timestamp: Utc::now(),
        }
    }
}
