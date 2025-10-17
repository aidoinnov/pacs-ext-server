//! # Settings Configuration
//! 
//! 애플리케이션의 모든 설정을 관리하는 구조체와 로직을 정의합니다.

use serde::Deserialize;
use std::env;

/// 서버 관련 설정
#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    /// 서버 바인딩 호스트
    pub host: String,
    /// 서버 포트
    pub port: u16,
    /// 워커 스레드 수
    pub workers: usize,
}

/// 로깅 관련 설정
#[derive(Debug, Deserialize, Clone)]
pub struct LoggingConfig {
    /// 로그 레벨 (debug, info, warn, error)
    pub level: String,
    /// 로그 포맷 (json, pretty)
    pub format: String,
}

/// CORS 관련 설정
#[derive(Debug, Deserialize, Clone)]
pub struct CorsConfig {
    /// CORS 활성화 여부
    pub enabled: bool,
    /// 허용된 오리진 목록
    pub allowed_origins: Vec<String>,
    /// 허용된 HTTP 메서드 목록
    pub allowed_methods: Vec<String>,
    /// 허용된 헤더 목록
    pub allowed_headers: Vec<String>,
    /// 노출할 헤더 목록
    pub expose_headers: Vec<String>,
    /// preflight 요청 캐시 시간 (초)
    pub max_age: u64,
}

/// 헬스체크 관련 설정
#[derive(Debug, Deserialize, Clone)]
pub struct HealthCheckConfig {
    /// 헬스체크 활성화 여부
    pub enabled: bool,
    /// 헬스체크 엔드포인트 경로
    pub endpoint: String,
    /// 응답 포맷 (json, text)
    pub response_format: String,
}

/// 애플리케이션 전체 설정
#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    /// 서버 설정
    pub server: ServerConfig,
    /// 로깅 설정
    pub logging: LoggingConfig,
    /// CORS 설정
    pub cors: CorsConfig,
    /// 헬스체크 설정
    pub health_check: HealthCheckConfig,
}

impl Settings {
    /// 환경 변수에서 설정을 로드합니다.
    /// 
    /// # 반환값
    /// - `Ok(Settings)`: 설정 로드 성공
    /// - `Err(String)`: 설정 로드 실패
    pub fn new() -> Result<Self, String> {
        // 간단한 환경 변수 기반 설정 로드
        Ok(Self::with_env_defaults())
    }

    /// 환경 변수 기본값으로 설정을 생성합니다.
    /// 
    /// 설정 파일이 없거나 로드에 실패한 경우 사용됩니다.
    /// 
    /// # 반환값
    /// - `Settings`: 환경 변수 기본값으로 생성된 설정
    pub fn with_env_defaults() -> Self {
        Self {
            server: ServerConfig {
                host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: env::var("PORT")
                    .unwrap_or_else(|_| "8080".to_string())
                    .parse()
                    .unwrap_or(8080),
                workers: env::var("WORKERS")
                    .unwrap_or_else(|_| "4".to_string())
                    .parse()
                    .unwrap_or(4),
            },
            logging: LoggingConfig {
                level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
                format: env::var("LOG_FORMAT").unwrap_or_else(|_| "json".to_string()),
            },
            cors: CorsConfig {
                enabled: env::var("CORS_ENABLED")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                allowed_origins: vec![
                    "http://localhost:3000".to_string(),
                    "http://localhost:8080".to_string(),
                ],
                allowed_methods: vec![
                    "GET".to_string(),
                    "POST".to_string(),
                    "PUT".to_string(),
                    "DELETE".to_string(),
                    "OPTIONS".to_string(),
                ],
                allowed_headers: vec![
                    "Content-Type".to_string(),
                    "Authorization".to_string(),
                    "X-Requested-With".to_string(),
                ],
                expose_headers: vec!["Content-Length".to_string()],
                max_age: 3600,
            },
            health_check: HealthCheckConfig {
                enabled: true,
                endpoint: "/health".to_string(),
                response_format: "json".to_string(),
            },
        }
    }
}
