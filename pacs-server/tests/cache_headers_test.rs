use actix_web::http::{header, StatusCode};
use actix_web::{test, web, App, HttpResponse};
use pacs_server::infrastructure::middleware::CacheHeaders;

#[actix_web::test]
async fn test_cache_headers_enabled() {
    // Create test middleware with caching enabled
    let cache_enabled = true;
    let cache_ttl = 300u64;

    let app = test::init_service(
        App::new()
            .wrap(CacheHeaders::new(cache_enabled, cache_ttl))
            .route(
                "/test",
                web::get().to(|| async { HttpResponse::Ok().body("OK") }),
            ),
    )
    .await;

    let req = test::TestRequest::get().uri("/test").to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);

    // Check Cache-Control header
    let cache_control = resp.headers().get(header::CACHE_CONTROL);
    assert!(
        cache_control.is_some(),
        "Cache-Control header should be present"
    );

    let cache_value = cache_control.unwrap().to_str().unwrap();
    assert_eq!(cache_value, "public, max-age=300");
}

#[actix_web::test]
async fn test_cache_headers_disabled() {
    // Create test middleware with caching disabled
    let cache_enabled = false;
    let cache_ttl = 300u64;

    let app = test::init_service(
        App::new()
            .wrap(CacheHeaders::new(cache_enabled, cache_ttl))
            .route(
                "/test",
                web::get().to(|| async { HttpResponse::Ok().body("OK") }),
            ),
    )
    .await;

    let req = test::TestRequest::get().uri("/test").to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);

    // Check Cache-Control header for no-cache
    let cache_control = resp.headers().get(header::CACHE_CONTROL);
    assert!(
        cache_control.is_some(),
        "Cache-Control header should be present"
    );

    let cache_value = cache_control.unwrap().to_str().unwrap();
    assert_eq!(cache_value, "no-cache, no-store, must-revalidate");
}

#[actix_web::test]
async fn test_cache_headers_post_request() {
    // POST requests should not be cached
    let cache_enabled = true;
    let cache_ttl = 300u64;

    let app = test::init_service(
        App::new()
            .wrap(CacheHeaders::new(cache_enabled, cache_ttl))
            .route(
                "/test",
                web::post().to(|| async { HttpResponse::Ok().body("OK") }),
            ),
    )
    .await;

    let req = test::TestRequest::post().uri("/test").to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);

    // POST requests should have no-cache even when caching is enabled
    let cache_control = resp.headers().get(header::CACHE_CONTROL);
    assert!(
        cache_control.is_some(),
        "Cache-Control header should be present"
    );

    let cache_value = cache_control.unwrap().to_str().unwrap();
    assert_eq!(cache_value, "no-cache, no-store, must-revalidate");
}

#[actix_web::test]
async fn test_cache_headers_custom_ttl() {
    // Test with custom TTL value
    let cache_enabled = true;
    let cache_ttl = 3600u64; // 1 hour

    let app = test::init_service(
        App::new()
            .wrap(CacheHeaders::new(cache_enabled, cache_ttl))
            .route(
                "/test",
                web::get().to(|| async { HttpResponse::Ok().body("OK") }),
            ),
    )
    .await;

    let req = test::TestRequest::get().uri("/test").to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);

    let cache_control = resp.headers().get(header::CACHE_CONTROL);
    assert!(
        cache_control.is_some(),
        "Cache-Control header should be present"
    );

    let cache_value = cache_control.unwrap().to_str().unwrap();
    assert_eq!(cache_value, "public, max-age=3600");
}
