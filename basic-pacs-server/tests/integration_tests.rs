//! # Integration Tests
//! 
//! API 엔드포인트에 대한 통합 테스트를 정의합니다.

use actix_web::{test, web, App};
use basic_pacs_server::presentation::controllers::health_controller;
use basic_pacs_server::application::use_cases::HealthCheckUseCase;
use basic_pacs_server::domain::services::HealthCheckServiceImpl;
use std::sync::Arc;

/// 테스트용 앱 설정
fn create_test_app() -> impl actix_web::dev::Service<actix_web::dev::ServiceRequest, actix_web::Error> {
    let health_check_service = Box::new(HealthCheckServiceImpl::new());
    let health_check_use_case = HealthCheckUseCase::new(health_check_service);
    let health_controller = Arc::new(health_controller::HealthController::new(health_check_use_case));

    App::new()
        .app_data(web::Data::from(health_controller.clone()))
        .service(
            web::scope("/api")
                .configure(health_controller::configure_routes)
        )
}

#[actix_web::test]
async fn test_health_check_endpoint() {
    let app = test::init_service(create_test_app()).await;
    
    let req = test::TestRequest::get()
        .uri("/api/health")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    assert!(resp.status().is_success());
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status"], "healthy");
    assert_eq!(body["service"], "basic-pacs-server");
}

#[actix_web::test]
async fn test_server_info_endpoint() {
    let app = test::init_service(create_test_app()).await;
    
    let req = test::TestRequest::get()
        .uri("/api/info")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    assert!(resp.status().is_success());
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["name"], "Basic PACS Server");
    assert_eq!(body["architecture"], "Clean Architecture");
    assert_eq!(body["framework"], "Actix Web");
    assert_eq!(body["language"], "Rust");
}

#[actix_web::test]
async fn test_basic_response_endpoint() {
    let app = test::init_service(create_test_app()).await;
    
    let req = test::TestRequest::get()
        .uri("/api/")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    assert!(resp.status().is_success());
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["message"], "Welcome to Basic PACS Server API");
    assert!(body["endpoints"].is_object());
}
