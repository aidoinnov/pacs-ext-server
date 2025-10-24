//! # Configuration
//! 
//! 애플리케이션 설정을 관리합니다.

use serde::{Deserialize, Serialize};
use std::env;

/// 서버 설정
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// 호스트
    pub host: String,
    /// 포트
    pub port: u16,
    /// 워커 수
    pub workers: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            workers: 2,
        }
    }
}

/// 로깅 설정
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// 로그 레벨
    pub level: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
        }
    }
}

/// CORS 설정
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    /// CORS 활성화 여부
    pub enabled: bool,
    /// 허용된 오리진 목록
    pub allowed_origins: Vec<String>,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            allowed_origins: vec!["*".to_string()],
        }
    }
}

/// 애플리케이션 설정
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// 서버 설정
    pub server: ServerConfig,
    /// 로깅 설정
    pub logging: LoggingConfig,
    /// CORS 설정
    pub cors: CorsConfig,
}

impl Settings {
    /// 설정 파일에서 설정 로드
    pub fn new() -> Result<Self, config::ConfigError> {
        let mut config = config::Config::default();
        
        // 기본 설정 파일 로드
        config.merge(config::File::with_name("config/default"))?;
        
        // 환경별 설정 파일 로드
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".to_string());
        config.merge(config::File::with_name(&format!("config/{}", run_mode)).required(false))?;
        
        // 환경 변수 오버라이드
        config.merge(config::Environment::with_prefix("APP"))?;
        
        config.try_into()
    }
    
    /// 환경 변수 기본값으로 설정 생성
    pub fn with_env_defaults() -> Self {
        Self {
            server: ServerConfig {
                host: env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
                port: env::var("PORT")
                    .unwrap_or_else(|_| "8080".to_string())
                    .parse()
                    .unwrap_or(8080),
                workers: env::var("WORKERS")
                    .unwrap_or_else(|_| "2".to_string())
                    .parse()
                    .unwrap_or(2),
            },
            logging: LoggingConfig {
                level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
            },
            cors: CorsConfig {
                enabled: env::var("CORS_ENABLED")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                allowed_origins: env::var("CORS_ORIGINS")
                    .unwrap_or_else(|_| "*".to_string())
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect(),
            },
        }
    }
}
