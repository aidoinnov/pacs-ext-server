//! # CORS Middleware
//! 
//! Cross-Origin Resource Sharing (CORS) 미들웨어를 설정합니다.
//! 웹 브라우저에서 다른 도메인의 리소스에 접근할 수 있도록 허용합니다.

use actix_web::middleware::DefaultHeaders;
use crate::infrastructure::config::CorsConfig;

/// CORS 설정을 기반으로 미들웨어를 구성합니다.
/// 
/// # 매개변수
/// - `cors_config`: CORS 설정 구조체
/// 
/// # 반환값
/// - `DefaultHeaders`: 기본 헤더 미들웨어
pub fn configure_cors(cors_config: &CorsConfig) -> DefaultHeaders {
    // 간단한 CORS 헤더 설정
    if cors_config.enabled {
        DefaultHeaders::new()
            .add(("Access-Control-Allow-Origin", "*"))
            .add(("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS"))
            .add(("Access-Control-Allow-Headers", "Content-Type, Authorization"))
    } else {
        DefaultHeaders::new()
    }
}
