//! # Middleware
//! 
//! HTTP 미들웨어들을 정의합니다.

use actix_web::{dev::ServiceRequest, dev::ServiceResponse, Error, middleware::Logger};
use actix_web::http::header::{HeaderValue, CACHE_CONTROL, EXPIRES};
use actix_web::middleware::DefaultHeaders;
use actix_cors::Cors;
use std::time::Duration;

use crate::infrastructure::config::CorsConfig;

/// CORS 미들웨어 설정
pub fn configure_cors(cors_config: &CorsConfig) -> Cors {
    if cors_config.enabled {
        let mut cors = Cors::default()
            .allow_any_header()
            .allow_any_method()
            .supports_credentials();
        
        // 허용된 오리진 설정
        if cors_config.allowed_origins.contains(&"*".to_string()) {
            cors = cors.allow_any_origin();
        } else {
            cors = cors.allowed_origins(&cors_config.allowed_origins);
        }
        
        cors
    } else {
        Cors::default()
    }
}

/// 캐시 헤더 미들웨어
pub struct CacheHeaders {
    enabled: bool,
    ttl_seconds: u64,
}

impl CacheHeaders {
    /// 새로운 캐시 헤더 미들웨어 생성
    pub fn new(enabled: bool, ttl_seconds: u64) -> Self {
        Self {
            enabled,
            ttl_seconds,
        }
    }
}

impl actix_web::dev::Transform<actix_web::dev::ServiceRequest, actix_web::dev::ServiceResponse, Error> for CacheHeaders {
    type Response = actix_web::dev::ServiceResponse;
    type Error = Error;
    type Transform = CacheHeadersMiddleware;
    type InitError = ();
    type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Transform, Self::InitError>> + Send>>;

    fn new_transform(&self, _service: actix_web::dev::ServiceRequest) -> Self::Future {
        let enabled = self.enabled;
        let ttl_seconds = self.ttl_seconds;
        
        Box::pin(async move {
            Ok(CacheHeadersMiddleware {
                enabled,
                ttl_seconds,
            })
        })
    }
}

pub struct CacheHeadersMiddleware {
    enabled: bool,
    ttl_seconds: u64,
}

impl actix_web::dev::Service<actix_web::dev::ServiceRequest, Error> for CacheHeadersMiddleware {
    type Response = actix_web::dev::ServiceResponse;
    type Error = Error;
    type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: actix_web::dev::ServiceRequest) -> Self::Future {
        let enabled = self.enabled;
        let ttl_seconds = self.ttl_seconds;
        
        Box::pin(async move {
            let mut res = req.into_parts().1;
            
            if enabled {
                let cache_control = format!("public, max-age={}", ttl_seconds);
                res.headers_mut().insert(
                    CACHE_CONTROL,
                    HeaderValue::from_str(&cache_control).unwrap(),
                );
                
                let expires = chrono::Utc::now() + chrono::Duration::seconds(ttl_seconds as i64);
                res.headers_mut().insert(
                    EXPIRES,
                    HeaderValue::from_str(&expires.to_rfc2822()).unwrap(),
                );
            }
            
            Ok(actix_web::dev::ServiceResponse::new(req, res))
        })
    }
}
