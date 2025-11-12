use actix_web::{test, web, App};
use pacs_server::application::use_cases::role_capability_matrix_use_case::RoleCapabilityMatrixUseCase;
use pacs_server::application::use_cases::permission_use_case::PermissionUseCase;
use pacs_server::infrastructure::services::CapabilityServiceImpl;
use pacs_server::domain::services::PermissionServiceImpl;
use pacs_server::infrastructure::repositories::{CapabilityRepositoryImpl, PermissionRepositoryImpl, RoleRepositoryImpl};
use pacs_server::presentation::controllers::role_controller;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;

/// 테스트 앱 설정
async fn setup_test_app() -> sqlx::PgPool {
    // 데이터베이스 연결
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://pacs_extension_admin:PacsExtension2024@localhost:5456/pacs_extension".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    pool
}

/// Use Case 생성 헬퍼
fn create_use_cases(
    pool: sqlx::PgPool,
) -> (
    Arc<RoleCapabilityMatrixUseCase>,
    Arc<PermissionUseCase<PermissionServiceImpl<PermissionRepositoryImpl, RoleRepositoryImpl>>>,
) {
    // Repository 생성
    let capability_repository = Arc::new(CapabilityRepositoryImpl::new(pool.clone()));
    let permission_repository = PermissionRepositoryImpl::new(pool.clone());
    let role_repository = RoleRepositoryImpl::new(pool);

    // Service 생성
    let capability_service = Arc::new(CapabilityServiceImpl::new(capability_repository));
    let permission_service = PermissionServiceImpl::new(
        permission_repository,
        role_repository,
    );

    // Use Case 생성
    let role_capability_matrix_use_case =
        Arc::new(RoleCapabilityMatrixUseCase::new(capability_service));
    let permission_use_case = Arc::new(PermissionUseCase::new(permission_service));

    (role_capability_matrix_use_case, permission_use_case)
}

/// 통합 테스트 1: scope 파라미터 없이 전체 역할 조회
#[actix_web::test]
async fn test_get_global_matrix_without_scope_filter() {
    let pool = setup_test_app().await;
    let (role_capability_matrix_use_case, permission_use_case) = create_use_cases(pool.clone());

    let app = test::init_service(
        App::new().service(
            web::scope("/api").configure(|cfg| {
                role_controller::configure_routes(
                    cfg,
                    permission_use_case.clone(),
                    role_capability_matrix_use_case.clone(),
                )
            }),
        ),
    )
    .await;

    // When: scope 파라미터 없이 요청
    let req = test::TestRequest::get()
        .uri("/api/roles/global/capabilities/matrix/all")
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Then: 성공 응답
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;

    // 응답 구조 검증
    assert!(body.get("roles").is_some());
    assert!(body.get("capabilities_by_category").is_some());
    assert!(body.get("assignments").is_some());

    let roles = body["roles"].as_array().unwrap();

    // GLOBAL과 PROJECT scope 역할이 모두 포함되어야 함
    let has_global = roles.iter().any(|r| r["scope"].as_str() == Some("GLOBAL"));
    let has_project = roles.iter().any(|r| r["scope"].as_str() == Some("PROJECT"));

    assert!(has_global, "GLOBAL scope 역할이 포함되어야 함");
    assert!(has_project, "PROJECT scope 역할이 포함되어야 함");

    println!("✅ 전체 역할 조회 성공: {} roles", roles.len());
}

/// 통합 테스트 2: scope=GLOBAL로 GLOBAL 역할만 조회
#[actix_web::test]
async fn test_get_global_matrix_with_global_scope_filter() {
    let pool = setup_test_app().await;
    let (role_capability_matrix_use_case, permission_use_case) = create_use_cases(pool);

    let app = test::init_service(
        App::new().service(
            web::scope("/api").configure(|cfg| {
                role_controller::configure_routes(
                    cfg,
                    permission_use_case.clone(),
                    role_capability_matrix_use_case.clone(),
                )
            }),
        ),
    )
    .await;

    // When: scope=GLOBAL로 요청
    let req = test::TestRequest::get()
        .uri("/api/roles/global/capabilities/matrix/all?scope=GLOBAL")
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Then: 성공 응답
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;

    let roles = body["roles"].as_array().unwrap();
    
    // 모든 역할이 GLOBAL scope여야 함
    for role in roles {
        assert_eq!(
            role["scope"].as_str().unwrap(),
            "GLOBAL",
            "모든 역할이 GLOBAL scope여야 함"
        );
    }

    // GLOBAL scope 역할이 최소 1개 이상 있어야 함
    assert!(roles.len() > 0, "GLOBAL scope 역할이 최소 1개 이상 있어야 함");

    println!("✅ GLOBAL scope 역할만 조회 성공: {} roles", roles.len());
}

/// 통합 테스트 3: scope=PROJECT로 PROJECT 역할만 조회
#[actix_web::test]
async fn test_get_global_matrix_with_project_scope_filter() {
    let pool = setup_test_app().await;
    let (role_capability_matrix_use_case, permission_use_case) = create_use_cases(pool);

    let app = test::init_service(
        App::new().service(
            web::scope("/api").configure(|cfg| {
                role_controller::configure_routes(
                    cfg,
                    permission_use_case.clone(),
                    role_capability_matrix_use_case.clone(),
                )
            }),
        ),
    )
    .await;

    // When: scope=PROJECT로 요청
    let req = test::TestRequest::get()
        .uri("/api/roles/global/capabilities/matrix/all?scope=PROJECT")
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Then: 성공 응답
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;

    let roles = body["roles"].as_array().unwrap();
    
    // 모든 역할이 PROJECT scope여야 함
    for role in roles {
        assert_eq!(
            role["scope"].as_str().unwrap(),
            "PROJECT",
            "모든 역할이 PROJECT scope여야 함"
        );
    }

    // PROJECT scope 역할이 최소 1개 이상 있어야 함
    assert!(roles.len() > 0, "PROJECT scope 역할이 최소 1개 이상 있어야 함");

    println!("✅ PROJECT scope 역할만 조회 성공: {} roles", roles.len());
}

/// 통합 테스트 4: 페이지네이션 버전에서 scope 필터링
#[actix_web::test]
async fn test_get_global_matrix_paginated_with_scope_filter() {
    let pool = setup_test_app().await;
    let (role_capability_matrix_use_case, permission_use_case) = create_use_cases(pool);

    let app = test::init_service(
        App::new().service(
            web::scope("/api").configure(|cfg| {
                role_controller::configure_routes(
                    cfg,
                    permission_use_case.clone(),
                    role_capability_matrix_use_case.clone(),
                )
            }),
        ),
    )
    .await;

    // When: 페이지네이션 + scope=GLOBAL
    let req = test::TestRequest::get()
        .uri("/api/roles/global/capabilities/matrix?page=1&size=10&scope=GLOBAL")
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Then: 성공 응답
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;

    // 페이지네이션 정보 검증
    assert!(body.get("roles").is_some());
    assert!(body.get("capabilities_by_category").is_some());
    assert!(body.get("assignments").is_some());
    assert!(body.get("pagination").is_some());

    let pagination = &body["pagination"];
    assert_eq!(pagination["current_page"].as_i64().unwrap(), 1);
    assert_eq!(pagination["page_size"].as_i64().unwrap(), 10);

    let roles = body["roles"].as_array().unwrap();

    // 모든 역할이 GLOBAL scope여야 함
    for role in roles {
        assert_eq!(
            role["scope"].as_str().unwrap(),
            "GLOBAL",
            "페이지네이션에서도 scope 필터가 적용되어야 함"
        );
    }

    println!("✅ 페이지네이션 + scope 필터 성공: {} roles", roles.len());
}

/// 통합 테스트 5: 잘못된 scope 값 처리
#[actix_web::test]
async fn test_get_global_matrix_with_invalid_scope() {
    let pool = setup_test_app().await;
    let (role_capability_matrix_use_case, permission_use_case) = create_use_cases(pool);

    let app = test::init_service(
        App::new().service(
            web::scope("/api").configure(|cfg| {
                role_controller::configure_routes(
                    cfg,
                    permission_use_case.clone(),
                    role_capability_matrix_use_case.clone(),
                )
            }),
        ),
    )
    .await;

    // When: 잘못된 scope 값으로 요청
    let req = test::TestRequest::get()
        .uri("/api/roles/global/capabilities/matrix/all?scope=INVALID")
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Then: 성공 응답 (빈 결과)
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;

    let roles = body["roles"].as_array().unwrap();
    
    // 잘못된 scope는 결과가 없어야 함
    assert_eq!(roles.len(), 0, "잘못된 scope는 빈 결과를 반환해야 함");

    println!("✅ 잘못된 scope 처리 성공: 빈 결과 반환");
}

/// 통합 테스트 6: 응답 구조 검증
#[actix_web::test]
async fn test_response_structure_validation() {
    let pool = setup_test_app().await;
    let (role_capability_matrix_use_case, permission_use_case) = create_use_cases(pool);

    let app = test::init_service(
        App::new().service(
            web::scope("/api").configure(|cfg| {
                role_controller::configure_routes(
                    cfg,
                    permission_use_case.clone(),
                    role_capability_matrix_use_case.clone(),
                )
            }),
        ),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/roles/global/capabilities/matrix/all?scope=GLOBAL")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;

    // roles 배열 구조 검증
    let roles = body["roles"].as_array().unwrap();
    if !roles.is_empty() {
        let first_role = &roles[0];
        assert!(first_role.get("id").is_some());
        assert!(first_role.get("name").is_some());
        assert!(first_role.get("description").is_some());
        assert!(first_role.get("scope").is_some());
        // created_at은 응답에 포함되지 않음
    }

    // capabilities_by_category 구조 검증
    let capabilities_by_category = body["capabilities_by_category"].as_object().unwrap();
    assert!(!capabilities_by_category.is_empty(), "capabilities_by_category should not be empty");

    // 첫 번째 카테고리의 capabilities 검증
    if let Some((_, capabilities)) = capabilities_by_category.iter().next() {
        let caps_array = capabilities.as_array().unwrap();
        if !caps_array.is_empty() {
            let first_cap = &caps_array[0];
            assert!(first_cap.get("id").is_some());
            assert!(first_cap.get("name").is_some());
            assert!(first_cap.get("display_name").is_some());
        }
    }

    // assignments 구조 검증 (배열)
    let assignments = body["assignments"].as_array().unwrap();
    // assignments는 비어있을 수 있음
    if !assignments.is_empty() {
        let first_assignment = &assignments[0];
        assert!(first_assignment.get("role_id").is_some());
        assert!(first_assignment.get("capability_id").is_some());
        assert!(first_assignment.get("assigned").is_some());
    }

    println!("✅ 응답 구조 검증 성공");
}

/// 통합 테스트 7: scope 대소문자 구분 테스트
#[actix_web::test]
async fn test_scope_case_sensitivity() {
    let pool = setup_test_app().await;
    let (role_capability_matrix_use_case, permission_use_case) = create_use_cases(pool);

    let app = test::init_service(
        App::new().service(
            web::scope("/api").configure(|cfg| {
                role_controller::configure_routes(
                    cfg,
                    permission_use_case.clone(),
                    role_capability_matrix_use_case.clone(),
                )
            }),
        ),
    )
    .await;

    // 소문자 scope
    let req = test::TestRequest::get()
        .uri("/api/roles/global/capabilities/matrix/all?scope=global")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    let roles = body["roles"].as_array().unwrap();
    
    // 대소문자를 구분하므로 빈 결과가 나와야 함
    assert_eq!(roles.len(), 0, "scope는 대소문자를 구분해야 함");

    println!("✅ scope 대소문자 구분 테스트 성공");
}

