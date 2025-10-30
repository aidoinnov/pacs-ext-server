use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::header::{HeaderValue, CACHE_CONTROL, ETAG},
    Error,
};
use std::{
    future::{ready, Future, Ready},
    pin::Pin,
    rc::Rc,
};

/// HTTP 캐싱 정책 설정
#[derive(Debug, Clone)]
pub enum CachePolicy {
    /// 캐싱 비활성화 (기본값 - 인증이 필요한 API)
    NoCache,
    /// Public 캐싱 (정적 리소스, 공개 데이터)
    Public { max_age: u32 },
    /// Private 캐싱 (사용자별 데이터)
    Private { max_age: u32 },
    /// Immutable 캐싱 (변경되지 않는 리소스)
    Immutable { max_age: u32 },
}

impl CachePolicy {
    fn to_header_value(&self) -> String {
        match self {
            CachePolicy::NoCache => {
                "no-cache, no-store, must-revalidate, private, max-age=0".to_string()
            }
            CachePolicy::Public { max_age } => {
                format!("public, max-age={}", max_age)
            }
            CachePolicy::Private { max_age } => {
                format!("private, max-age={}", max_age)
            }
            CachePolicy::Immutable { max_age } => {
                format!("public, max-age={}, immutable", max_age)
            }
        }
    }
}

/// 캐싱 미들웨어
pub struct CacheMiddleware {
    policy: CachePolicy,
    enable_etag: bool,
}

impl CacheMiddleware {
    /// 새로운 캐싱 미들웨어 생성
    pub fn new(policy: CachePolicy) -> Self {
        Self {
            policy,
            enable_etag: false,
        }
    }

    /// ETag 생성 활성화
    pub fn with_etag(mut self) -> Self {
        self.enable_etag = true;
        self
    }
}

impl Default for CacheMiddleware {
    fn default() -> Self {
        Self::new(CachePolicy::NoCache)
    }
}

impl<S, B> Transform<S, ServiceRequest> for CacheMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = CacheMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(CacheMiddlewareService {
            service: Rc::new(service),
            policy: self.policy.clone(),
            enable_etag: self.enable_etag,
        }))
    }
}

pub struct CacheMiddlewareService<S> {
    service: Rc<S>,
    policy: CachePolicy,
    enable_etag: bool,
}

impl<S, B> Service<ServiceRequest> for CacheMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let policy = self.policy.clone();
        let enable_etag = self.enable_etag;

        Box::pin(async move {
            let mut res = service.call(req).await?;

            // Cache-Control 헤더 추가
            let cache_value = HeaderValue::from_str(&policy.to_header_value())
                .unwrap_or_else(|_| HeaderValue::from_static("no-cache"));
            res.headers_mut().insert(CACHE_CONTROL, cache_value);

            // ETag 생성 (선택적)
            if enable_etag {
                // 간단한 타임스탬프 기반 ETag 생성
                let etag_value = format!("\"{}\"", generate_simple_etag());
                if let Ok(etag_header) = HeaderValue::from_str(&etag_value) {
                    res.headers_mut().insert(ETAG, etag_header);
                }
            }

            Ok(res)
        })
    }
}

/// 간단한 ETag 생성 (실제 프로덕션에서는 더 정교한 해시 사용 권장)
fn generate_simple_etag() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();

    format!("{:x}", timestamp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_policy_headers() {
        let no_cache = CachePolicy::NoCache;
        assert_eq!(
            no_cache.to_header_value(),
            "no-cache, no-store, must-revalidate, private, max-age=0"
        );

        let public = CachePolicy::Public { max_age: 3600 };
        assert_eq!(public.to_header_value(), "public, max-age=3600");

        let private = CachePolicy::Private { max_age: 300 };
        assert_eq!(private.to_header_value(), "private, max-age=300");

        let immutable = CachePolicy::Immutable { max_age: 31536000 };
        assert_eq!(
            immutable.to_header_value(),
            "public, max-age=31536000, immutable"
        );
    }
}
