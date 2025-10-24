//! # Data Transfer Objects (DTOs)
//! 
//! API 요청/응답을 위한 데이터 전송 객체들을 정의합니다.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// 헬스체크 응답 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    /// 서버 상태
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

impl From<crate::domain::entities::ServerStatus> for HealthCheckResponse {
    fn from(status: crate::domain::entities::ServerStatus) -> Self {
        Self {
            status: status.status,
            service: status.service,
            version: status.version,
            timestamp: status.timestamp,
            metadata: status.metadata,
        }
    }
}

/// 서버 정보 응답 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfoResponse {
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

impl From<crate::domain::entities::ServerInfo> for ServerInfoResponse {
    fn from(info: crate::domain::entities::ServerInfo) -> Self {
        Self {
            name: info.name,
            version: info.version,
            description: info.description,
            architecture: info.architecture,
            framework: info.framework,
            language: info.language,
            timestamp: info.timestamp,
        }
    }
}

/// 기본 응답 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicResponse {
    /// 메시지
    pub message: String,
    /// 엔드포인트 정보
    pub endpoints: EndpointInfo,
    /// 타임스탬프
    pub timestamp: DateTime<Utc>,
}

/// 엔드포인트 정보
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointInfo {
    /// 헬스체크 엔드포인트
    pub health: String,
    /// 서버 정보 엔드포인트
    pub info: String,
    /// API 엔드포인트
    pub api: String,
}
