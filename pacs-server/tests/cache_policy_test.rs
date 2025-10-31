use actix_web::http::{header, StatusCode};
use actix_web::{test, web, App, HttpResponse};
use pacs_server::infrastructure::middleware::{CacheMiddleware, CachePolicy};

#[actix_web::test]
async fn test_cache_policy_no_cache() {
    let app = test::init_service(
        App::new()
            .wrap(CacheMiddleware::new(CachePolicy::NoCache))
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
    assert_eq!(
        cache_value,
        "no-cache, no-store, must-revalidate, private, max-age=0"
    );
}

#[actix_web::test]
async fn test_cache_policy_public() {
    let app = test::init_service(
        App::new()
            .wrap(CacheMiddleware::new(CachePolicy::Public { max_age: 3600 }))
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

#[actix_web::test]
async fn test_cache_policy_private() {
    let app = test::init_service(
        App::new()
            .wrap(CacheMiddleware::new(CachePolicy::Private { max_age: 300 }))
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
    assert_eq!(cache_value, "private, max-age=300");
}

#[actix_web::test]
async fn test_cache_policy_immutable() {
    let app = test::init_service(
        App::new()
            .wrap(CacheMiddleware::new(CachePolicy::Immutable {
                max_age: 31536000,
            }))
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
    assert_eq!(cache_value, "public, max-age=31536000, immutable");
}

#[actix_web::test]
async fn test_cache_policy_with_etag() {
    let app = test::init_service(
        App::new()
            .wrap(CacheMiddleware::new(CachePolicy::Public { max_age: 3600 }).with_etag())
            .route(
                "/test",
                web::get().to(|| async { HttpResponse::Ok().body("OK") }),
            ),
    )
    .await;

    let req = test::TestRequest::get().uri("/test").to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);

    // Cache-Control should be present
    let cache_control = resp.headers().get(header::CACHE_CONTROL);
    assert!(
        cache_control.is_some(),
        "Cache-Control header should be present"
    );

    // ETag should be present when enabled
    let etag = resp.headers().get(header::ETAG);
    assert!(etag.is_some(), "ETag header should be present when enabled");

    let etag_value = etag.unwrap().to_str().unwrap();
    assert!(etag_value.starts_with("\""), "ETag should be quoted");
    assert!(etag_value.ends_with("\""), "ETag should be quoted");
}

#[actix_web::test]
async fn test_cache_policy_default() {
    // Default should be NoCache
    let app = test::init_service(App::new().wrap(CacheMiddleware::default()).route(
        "/test",
        web::get().to(|| async { HttpResponse::Ok().body("OK") }),
    ))
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
    assert_eq!(
        cache_value,
        "no-cache, no-store, must-revalidate, private, max-age=0"
    );
}
