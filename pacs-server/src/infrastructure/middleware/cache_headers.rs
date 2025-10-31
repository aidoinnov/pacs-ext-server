use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures::future::{ok, LocalBoxFuture, Ready};

/// HTTP Cache Headers Middleware
/// Adds Cache-Control headers based on configuration
#[derive(Clone)]
pub struct CacheHeaders {
    enabled: bool,
    max_age: u64,
}

impl CacheHeaders {
    pub fn new(enabled: bool, max_age: u64) -> Self {
        Self { enabled, max_age }
    }
}

impl<S, B> Transform<S, ServiceRequest> for CacheHeaders
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = CacheHeadersMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(CacheHeadersMiddleware {
            service,
            enabled: self.enabled,
            max_age: self.max_age,
        })
    }
}

pub struct CacheHeadersMiddleware<S> {
    service: S,
    enabled: bool,
    max_age: u64,
}

impl<S, B> Service<ServiceRequest> for CacheHeadersMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let enabled = self.enabled;
        let max_age = self.max_age;
        let method = req.method().clone();

        let fut = self.service.call(req);

        Box::pin(async move {
            let mut res = fut.await?;

            let headers = res.headers_mut();

            // Only add cache headers for GET requests and if enabled
            if enabled && method == actix_web::http::Method::GET {
                // Add Cache-Control header for GET requests
                headers.insert(
                    actix_web::http::header::CACHE_CONTROL,
                    actix_web::http::header::HeaderValue::from_str(&format!(
                        "public, max-age={}",
                        max_age
                    ))
                    .unwrap(),
                );
            } else {
                // Disable caching for non-GET requests or when caching is disabled
                headers.insert(
                    actix_web::http::header::CACHE_CONTROL,
                    actix_web::http::header::HeaderValue::from_static(
                        "no-cache, no-store, must-revalidate",
                    ),
                );
            }

            Ok(res)
        })
    }
}
