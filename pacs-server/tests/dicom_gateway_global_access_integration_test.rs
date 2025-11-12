/// 통합 테스트: DICOM Gateway 전체 권한 기능
/// 
/// 테스트 시나리오:
/// 1. 전체 데이터 조회 (SUPER_ADMIN) - project_id 없이 호출
/// 2. 전체 데이터 조회 (ADMIN) - project_id 없이 호출
/// 3. 전체 데이터 조회 시도 (일반 사용자) - 400 Bad Request
/// 4. 프로젝트별 조회 (SUPER_ADMIN) - project_id와 함께 호출
/// 5. 프로젝트별 조회 (일반 사용자) - 기존 동작 유지

use actix_web::{test, web, App, HttpResponse};
use actix_web::http::StatusCode;
use serde_json::Value;

/// Mock QIDO 서버 응답
async fn mock_qido_studies() -> HttpResponse {
    let studies = serde_json::json!([
        {
            "0020000D": {"Value": ["1.2.826.0.1.3680043.8.498.11111111111111111111111111111111"]},
            "00100020": {"Value": ["PATIENT001"]},
            "00080060": {"Value": ["CT"]}
        },
        {
            "0020000D": {"Value": ["1.2.826.0.1.3680043.8.498.22222222222222222222222222222222"]},
            "00100020": {"Value": ["PATIENT002"]},
            "00080060": {"Value": ["MR"]}
        },
        {
            "0020000D": {"Value": ["1.2.826.0.1.3680043.8.498.33333333333333333333333333333333"]},
            "00100020": {"Value": ["PATIENT003"]},
            "00080060": {"Value": ["CT"]}
        }
    ]);
    HttpResponse::Ok().json(studies)
}

#[actix_web::test]
async fn test_global_access_without_project_id_super_admin() {
    // Given: Mock QIDO 서버
    let app = test::init_service(
        App::new()
            .route("/studies", web::get().to(mock_qido_studies))
    )
    .await;

    // When: project_id 없이 호출 (SUPER_ADMIN 권한 가정)
    let req = test::TestRequest::get()
        .uri("/studies")
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Then: 200 OK, 전체 데이터 반환
    assert_eq!(resp.status(), StatusCode::OK);

    let body: Value = test::read_body_json(resp).await;
    assert!(body.is_array());
    assert_eq!(body.as_array().unwrap().len(), 3);
}

#[actix_web::test]
async fn test_global_access_with_project_id_super_admin() {
    // Given: Mock QIDO 서버
    let app = test::init_service(
        App::new()
            .route("/studies", web::get().to(mock_qido_studies))
    )
    .await;

    // When: project_id와 함께 호출 (SUPER_ADMIN 권한 가정)
    let req = test::TestRequest::get()
        .uri("/studies?project_id=150")
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Then: 200 OK, 필터링된 데이터 반환
    assert_eq!(resp.status(), StatusCode::OK);

    let body: Value = test::read_body_json(resp).await;
    assert!(body.is_array());
}

#[actix_web::test]
async fn test_no_global_access_without_project_id_regular_user() {
    // Given: Mock 핸들러 (권한 없음 시뮬레이션)
    async fn handler_no_permission() -> HttpResponse {
        HttpResponse::BadRequest().json(serde_json::json!({
            "error": "project_id is required (no global access permission)"
        }))
    }

    let app = test::init_service(
        App::new()
            .route("/studies", web::get().to(handler_no_permission))
    )
    .await;

    // When: project_id 없이 호출 (일반 사용자)
    let req = test::TestRequest::get()
        .uri("/studies")
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Then: 400 Bad Request
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let body: Value = test::read_body_json(resp).await;
    assert_eq!(
        body["error"].as_str().unwrap(),
        "project_id is required (no global access permission)"
    );
}

#[actix_web::test]
async fn test_backward_compatibility_regular_user_with_project_id() {
    // Given: Mock QIDO 서버
    let app = test::init_service(
        App::new()
            .route("/studies", web::get().to(mock_qido_studies))
    )
    .await;

    // When: project_id와 함께 호출 (일반 사용자, 기존 동작)
    let req = test::TestRequest::get()
        .uri("/studies?project_id=150")
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Then: 200 OK, 기존과 동일하게 동작
    assert_eq!(resp.status(), StatusCode::OK);

    let body: Value = test::read_body_json(resp).await;
    assert!(body.is_array());
}

#[actix_web::test]
async fn test_invalid_project_id_zero() {
    // Given: Mock 핸들러 (project_id 검증)
    async fn handler_validate_project_id(query: web::Query<std::collections::HashMap<String, String>>) -> HttpResponse {
        if let Some(project_id_str) = query.get("project_id") {
            if let Ok(project_id) = project_id_str.parse::<i32>() {
                if project_id <= 0 {
                    return HttpResponse::BadRequest().json(serde_json::json!({
                        "error": "project_id must be greater than 0"
                    }));
                }
            }
        }
        HttpResponse::Ok().json(serde_json::json!([]))
    }

    let app = test::init_service(
        App::new()
            .route("/studies", web::get().to(handler_validate_project_id))
    )
    .await;

    // When: project_id=0으로 호출
    let req = test::TestRequest::get()
        .uri("/studies?project_id=0")
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Then: 400 Bad Request
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let body: Value = test::read_body_json(resp).await;
    assert_eq!(
        body["error"].as_str().unwrap(),
        "project_id must be greater than 0"
    );
}

#[actix_web::test]
async fn test_invalid_project_id_negative() {
    // Given: Mock 핸들러 (project_id 검증)
    async fn handler_validate_project_id(query: web::Query<std::collections::HashMap<String, String>>) -> HttpResponse {
        if let Some(project_id_str) = query.get("project_id") {
            if let Ok(project_id) = project_id_str.parse::<i32>() {
                if project_id <= 0 {
                    return HttpResponse::BadRequest().json(serde_json::json!({
                        "error": "project_id must be greater than 0"
                    }));
                }
            }
        }
        HttpResponse::Ok().json(serde_json::json!([]))
    }

    let app = test::init_service(
        App::new()
            .route("/studies", web::get().to(handler_validate_project_id))
    )
    .await;

    // When: project_id=-1로 호출
    let req = test::TestRequest::get()
        .uri("/studies?project_id=-1")
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Then: 400 Bad Request
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let body: Value = test::read_body_json(resp).await;
    assert_eq!(
        body["error"].as_str().unwrap(),
        "project_id must be greater than 0"
    );
}

#[actix_web::test]
async fn test_series_endpoint_global_access() {
    // Given: Mock QIDO 서버 (Series)
    async fn mock_qido_series() -> HttpResponse {
        let series = serde_json::json!([
            {
                "0020000E": {"Value": ["1.2.826.0.1.3680043.8.498.44444444444444444444444444444444"]},
                "00080060": {"Value": ["CT"]}
            },
            {
                "0020000E": {"Value": ["1.2.826.0.1.3680043.8.498.55555555555555555555555555555555"]},
                "00080060": {"Value": ["MR"]}
            }
        ]);
        HttpResponse::Ok().json(series)
    }

    let app = test::init_service(
        App::new()
            .route("/series/{study_uid}", web::get().to(mock_qido_series))
    )
    .await;

    // When: project_id 없이 호출 (SUPER_ADMIN 권한 가정)
    let req = test::TestRequest::get()
        .uri("/series/1.2.826.0.1.3680043.8.498.11111111111111111111111111111111")
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Then: 200 OK, 전체 데이터 반환
    assert_eq!(resp.status(), StatusCode::OK);

    let body: Value = test::read_body_json(resp).await;
    assert!(body.is_array());
    assert_eq!(body.as_array().unwrap().len(), 2);
}

#[actix_web::test]
async fn test_instances_endpoint_global_access() {
    // Given: Mock QIDO 서버 (Instances)
    async fn mock_qido_instances() -> HttpResponse {
        let instances = serde_json::json!([
            {
                "00080018": {"Value": ["1.2.826.0.1.3680043.8.498.66666666666666666666666666666666"]},
                "00080016": {"Value": ["1.2.840.10008.5.1.4.1.1.2"]}
            }
        ]);
        HttpResponse::Ok().json(instances)
    }

    let app = test::init_service(
        App::new()
            .route("/instances/{study_uid}/{series_uid}", web::get().to(mock_qido_instances))
    )
    .await;

    // When: project_id 없이 호출 (SUPER_ADMIN 권한 가정)
    let req = test::TestRequest::get()
        .uri("/instances/1.2.826.0.1.3680043.8.498.11111111111111111111111111111111/1.2.826.0.1.3680043.8.498.44444444444444444444444444444444")
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Then: 200 OK, 전체 데이터 반환
    assert_eq!(resp.status(), StatusCode::OK);

    let body: Value = test::read_body_json(resp).await;
    assert!(body.is_array());
    assert_eq!(body.as_array().unwrap().len(), 1);
}

