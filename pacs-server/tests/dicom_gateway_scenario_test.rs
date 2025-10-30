/// DICOM Gateway 시나리오 통합 테스트
/// 
/// 이 테스트는 다음과 같은 실제 시나리오를 검증합니다:
/// 1. 프로젝트별 필터링: 특정 프로젝트에 포함된 Study/Series/Instance만 반환
/// 2. 규칙 기반 필터링: AccessCondition에 정의된 규칙(Modality, PatientID 등)에 따라 필터링
/// 3. RBAC 필터링: Evaluator를 통해 사용자 접근 권한이 있는 데이터만 반환
/// 4. 계층적 필터링: Study → Series → Instance 순서로 상속되는 필터링
use actix_web::{test, web, App, HttpResponse, HttpServer};
use actix_web::test::TestRequest;
use serde_json::json;
use sqlx::PgPool;
use std::net::TcpListener;
use uuid::Uuid;

use pacs_server::infrastructure::services::DicomRbacEvaluatorImpl;
use pacs_server::infrastructure::external::Dcm4cheeQidoClient;
use pacs_server::infrastructure::auth::JwtService;
use pacs_server::infrastructure::repositories::AccessConditionRepositoryImpl;
use pacs_server::presentation::controllers::dicom_gateway_controller;

/// QIDO-RS JSON에서 StudyInstanceUID 추출 (테스트용)
fn extract_study_uid_test(item: &serde_json::Value) -> Option<String> {
    item.get("0020000D")
        .and_then(|v| v.get("Value"))
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.get(0))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

async fn get_test_pool() -> PgPool {
    let database_url = std::env::var("APP_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .unwrap_or_else(|_| "postgres://pacs_extension_admin:PacsExtension2024@localhost:5456/pacs_extension".to_string());

    PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

async fn cleanup_test_data(pool: &PgPool) {
    sqlx::query("SET session_replication_role = replica")
        .execute(pool)
        .await
        .ok();
    
    sqlx::query("DELETE FROM project_data_access").execute(pool).await.ok();
    sqlx::query("DELETE FROM project_data_instance").execute(pool).await.ok();
    sqlx::query("DELETE FROM project_data_series").execute(pool).await.ok();
    sqlx::query("DELETE FROM project_data_study").execute(pool).await.ok();
    sqlx::query("DELETE FROM project_data").execute(pool).await.ok();
    sqlx::query("DELETE FROM security_project_dicom_condition").execute(pool).await.ok();
    sqlx::query("DELETE FROM security_role_dicom_condition").execute(pool).await.ok();
    sqlx::query("DELETE FROM security_access_condition").execute(pool).await.ok();
    sqlx::query("DELETE FROM security_institution_data_access").execute(pool).await.ok();
    sqlx::query("DELETE FROM security_user_project").execute(pool).await.ok();
    sqlx::query("DELETE FROM security_user").execute(pool).await.ok();
    sqlx::query("DELETE FROM security_project").execute(pool).await.ok();
    sqlx::query("DELETE FROM security_institution").execute(pool).await.ok();
    sqlx::query("DELETE FROM project_data_institution").execute(pool).await.ok();
    
    sqlx::query("SET session_replication_role = DEFAULT")
        .execute(pool)
        .await
        .ok();
}

/// 테스트 데이터 설정
/// Returns: (user_id, project_a_id, project_b_id, study_a1_uid, study_a2_uid, study_b1_uid, series_a1_uid, series_a2_uid, series_b1_uid)
async fn setup_scenario_data(pool: &PgPool) -> (i32, i32, i32, String, String, String, String, String, String) {
    let test_id = Uuid::new_v4().as_simple().to_string()[..8].to_string();
    
    // 1. 기관 생성
    let inst_id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO security_institution (institution_code, institution_name, is_active)
         VALUES ($1, 'Test Institution', true)
         RETURNING id"
    )
    .bind(format!("INST{}", test_id))
    .fetch_one(pool)
    .await
    .unwrap();

    let data_inst_id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO project_data_institution (institution_code, institution_name, is_active)
         VALUES ($1, 'Test Data Institution', true)
         RETURNING id"
    )
    .bind(format!("DATA{}", test_id))
    .fetch_one(pool)
    .await
    .unwrap();

    // 2. 사용자 생성
    let user_id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO security_user (keycloak_id, username, email, account_status, institution_id)
         VALUES ($1, $2, $3, 'ACTIVE', $4)
         RETURNING id"
    )
    .bind(Uuid::new_v4())
    .bind(format!("user_{}", test_id))
    .bind(format!("user_{}@test.com", test_id))
    .bind(inst_id)
    .fetch_one(pool)
    .await
    .unwrap();

    // 3. 프로젝트 A, B 생성
    let project_a_id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO security_project (name, description, status)
         VALUES ($1, 'Project A', 'ACTIVE')
         RETURNING id"
    )
    .bind(format!("ProjectA_{}", test_id))
    .fetch_one(pool)
    .await
    .unwrap();

    let project_b_id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO security_project (name, description, status)
         VALUES ($1, 'Project B', 'ACTIVE')
         RETURNING id"
    )
    .bind(format!("ProjectB_{}", test_id))
    .fetch_one(pool)
    .await
    .unwrap();

    // 4. 사용자를 프로젝트 A에만 추가
    sqlx::query(
        "INSERT INTO security_user_project (user_id, project_id)
         VALUES ($1, $2)"
    )
    .bind(user_id)
    .bind(project_a_id)
    .execute(pool)
    .await
    .unwrap();

    // 5. Study 생성 (Project A: Study A1, A2 / Project B: Study B1)
    let study_a1_uid = format!("1.2.3.4.5.A1.{}", test_id);
    let study_a2_uid = format!("1.2.3.4.5.A2.{}", test_id);
    let study_b1_uid = format!("1.2.3.4.5.B1.{}", test_id);

    // project_data 레코드 생성
    let pd_a1_id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO project_data (project_id, study_uid) VALUES ($1, $2) RETURNING id"
    )
    .bind(project_a_id)
    .bind(&study_a1_uid)
    .fetch_one(pool)
    .await
    .unwrap();

    let _pd_a2_id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO project_data (project_id, study_uid) VALUES ($1, $2) RETURNING id"
    )
    .bind(project_a_id)
    .bind(&study_a2_uid)
    .fetch_one(pool)
    .await
    .unwrap();

    let _pd_b1_id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO project_data (project_id, study_uid) VALUES ($1, $2) RETURNING id"
    )
    .bind(project_b_id)
    .bind(&study_b1_uid)
    .fetch_one(pool)
    .await
    .unwrap();

    // project_data_study 생성
    let study_a1_id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO project_data_study (project_id, study_uid, data_institution_id, modality)
         VALUES ($1, $2, $3, 'CT')
         RETURNING id"
    )
    .bind(project_a_id)
    .bind(&study_a1_uid)
    .bind(data_inst_id)
    .fetch_one(pool)
    .await
    .unwrap();

    let study_a2_id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO project_data_study (project_id, study_uid, data_institution_id, modality)
         VALUES ($1, $2, $3, 'MR')
         RETURNING id"
    )
    .bind(project_a_id)
    .bind(&study_a2_uid)
    .bind(data_inst_id)
    .fetch_one(pool)
    .await
    .unwrap();

    let _study_b1_id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO project_data_study (project_id, study_uid, data_institution_id, modality)
         VALUES ($1, $2, $3, 'CT')
         RETURNING id"
    )
    .bind(project_b_id)
    .bind(&study_b1_uid)
    .bind(data_inst_id)
    .fetch_one(pool)
    .await
    .unwrap();

    // 6. Series 생성
    let series_a1_uid = format!("1.2.3.4.5.A1.S1.{}", test_id);
    let series_a2_uid = format!("1.2.3.4.5.A2.S1.{}", test_id);
    let series_b1_uid = format!("1.2.3.4.5.B1.S1.{}", test_id);

    let series_a1_id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO project_data_series (study_id, series_uid) VALUES ($1, $2) RETURNING id"
    )
    .bind(study_a1_id)
    .bind(&series_a1_uid)
    .fetch_one(pool)
    .await
    .unwrap();

    let series_a2_id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO project_data_series (study_id, series_uid) VALUES ($1, $2) RETURNING id"
    )
    .bind(study_a2_id)
    .bind(&series_a2_uid)
    .fetch_one(pool)
    .await
    .unwrap();

    // 7. 사용자에게 Study A1에 대한 명시적 접근 권한 부여
    sqlx::query(
        "INSERT INTO project_data_access (user_id, project_id, resource_level, study_id, status, project_data_id)
         VALUES ($1, $2, 'STUDY', $3, 'APPROVED', $4)"
    )
    .bind(user_id)
    .bind(project_a_id)
    .bind(study_a1_id)
    .bind(pd_a1_id)
    .execute(pool)
    .await
    .unwrap();

    (user_id, project_a_id, project_b_id, study_a1_uid, study_a2_uid, study_b1_uid, series_a1_uid, series_a2_uid, series_b1_uid)
}

// ========================================
// 시나리오 1: 프로젝트별 필터링
// ========================================

#[tokio::test]
#[ignore]
#[ignore]
async fn test_scenario_project_filtering() {
    let pool = get_test_pool().await;
    cleanup_test_data(&pool).await;

    let (user_id, project_a_id, project_b_id, study_a1_uid, study_a2_uid, study_b1_uid, _, _, _) = setup_scenario_data(&pool).await;

    // Mock QIDO server: 모든 Study 반환
    let study_a1_clone = study_a1_uid.clone();
    let study_a2_clone = study_a2_uid.clone();
    let study_b1_clone = study_b1_uid.clone();
    
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind");
    let port = listener.local_addr().unwrap().port();
    let server = HttpServer::new(move || {
        let s1 = study_a1_clone.clone();
        let s2 = study_a2_clone.clone();
        let s3 = study_b1_clone.clone();
        App::new().route("/rs/studies", web::get().to(move || {
            let s1 = s1.clone();
            let s2 = s2.clone();
            let s3 = s3.clone();
            async move {
                HttpResponse::Ok().json(json!([
                    {"0020000D": {"Value": [s1], "vr": "UI"}, "00080060": {"Value": ["CT"], "vr": "CS"}},
                    {"0020000D": {"Value": [s2], "vr": "UI"}, "00080060": {"Value": ["MR"], "vr": "CS"}},
                    {"0020000D": {"Value": [s3], "vr": "UI"}, "00080060": {"Value": ["CT"], "vr": "CS"}},
                ]))
            }
        }))
    })
    .listen(listener)
    .expect("Failed to start mock server")
    .run();
    
    // 별도 스레드에서 서버 실행
    std::thread::spawn(move || {
        let rt = actix_rt::Runtime::new().unwrap();
        rt.block_on(server);
    });

    let cfg = pacs_server::infrastructure::config::Dcm4cheeConfig {
        base_url: format!("http://127.0.0.1:{}", port),
        qido_path: "/rs".to_string(),
        wado_path: "/wado".to_string(),
        aet: "TEST".to_string(),
        username: None,
        password: None,
        timeout_ms: 5000,
            db: None,
    };
    let qido = Dcm4cheeQidoClient::new(cfg);
    let evaluator = std::sync::Arc::new(DicomRbacEvaluatorImpl::new(pool.clone()));
    let jwt_service = std::sync::Arc::new(JwtService::new(&pacs_server::infrastructure::config::JwtConfig {
        secret: "test_secret_key_for_jwt_service_integration_tests".to_string(),
        expiration_hours: 24,
    }));
    let ac_repo = std::sync::Arc::new(AccessConditionRepositoryImpl { pool: pool.clone() });

    // Gateway 설정 (evaluator 필터링 포함)
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(qido))
            .app_data(web::Data::new(evaluator))
            .app_data(web::Data::new(jwt_service))
            .app_data(web::Data::new(ac_repo))
            .service(
                web::scope("/api/dicom")
                    .route("/studies", web::get().to(dicom_gateway_controller::get_studies)),
            ),
    )
    .await;

    // 잠시 대기 (서버 시작 대기)
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // 프로젝트 A로 요청
    let token = format!("Bearer test_token_user_{}", user_id);
    let req = test::TestRequest::get()
        .uri(&format!("/api/dicom/studies?project_id={}", project_a_id))
        .insert_header(("Authorization", token))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // Evaluator 필터링이 동작하면 Project A의 Study만 반환되어야 함
    // 사용자는 Study A1에만 명시적 접근 권한이 있음
    assert!(resp.status().is_success(), "Request should succeed");
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    println!("Project filtering response: {}", serde_json::to_string_pretty(&body).unwrap());
    
    if let Some(array) = body.as_array() {
        // Project B의 Study는 필터링되어야 함
        // 사용자는 Study A1에만 접근 권한이 있으므로 최소 1개는 반환되어야 함
        assert!(array.len() >= 1, "Should return at least Study A1");
        
        // 모든 반환된 Study가 Project A에 속하는지 확인
        for item in array.iter() {
            if let Some(uid) = extract_study_uid_test(item) {
                assert!(
                    uid == study_a1_uid || uid == study_a2_uid,
                    "Should only return Study from Project A"
                );
            }
        }
    }
    
    cleanup_test_data(&pool).await;
}

// ========================================
// 시나리오 2: 규칙 기반 필터링
// ========================================

#[tokio::test]
#[ignore]
#[ignore]
async fn test_scenario_rule_based_filtering() {
    let pool = get_test_pool().await;
    cleanup_test_data(&pool).await;

    let (user_id, project_a_id, _, study_a1_uid, study_a2_uid, _, _, _, _) = setup_scenario_data(&pool).await;

    // AccessCondition 생성: Modality=CT만 허용
    let condition_id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO security_access_condition (resource_level, resource_type, dicom_tag, operator, value, condition_type)
         VALUES ('STUDY', 'study', '00080060', 'EQ', 'CT', 'ALLOW')
         RETURNING id"
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    // 프로젝트에 조건 연결
    sqlx::query(
        "INSERT INTO security_project_dicom_condition (project_id, access_condition_id)
         VALUES ($1, $2)"
    )
    .bind(project_a_id)
    .bind(condition_id)
    .execute(&pool)
    .await
    .unwrap();

    // Mock QIDO server
    let study_a1_clone_rule = study_a1_uid.clone();
    let study_a2_clone_rule = study_a2_uid.clone();
    
    let listener_rule = TcpListener::bind("127.0.0.1:0").expect("Failed to bind");
    let port_rule = listener_rule.local_addr().unwrap().port();
    let server_rule = HttpServer::new(move || {
        let s1 = study_a1_clone_rule.clone();
        let s2 = study_a2_clone_rule.clone();
        App::new().route("/rs/studies", web::get().to(move || {
            let s1 = s1.clone();
            let s2 = s2.clone();
            async move {
                HttpResponse::Ok().json(json!([
                    {"0020000D": {"Value": [s1], "vr": "UI"}, "00080060": {"Value": ["CT"], "vr": "CS"}},
                    {"0020000D": {"Value": [s2], "vr": "UI"}, "00080060": {"Value": ["MR"], "vr": "CS"}},
                ]))
            }
        }))
    })
    .listen(listener_rule)
    .expect("Failed to start mock server")
    .run();
    
    actix_rt::spawn(server_rule);

    // 잠시 대기 (서버 시작 대기)
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let cfg_rule = pacs_server::infrastructure::config::Dcm4cheeConfig {
        base_url: format!("http://127.0.0.1:{}", port_rule),
        qido_path: "/rs".to_string(),
        wado_path: "/wado".to_string(),
        aet: "TEST".to_string(),
        username: None,
        password: None,
        timeout_ms: 5000,
            db: None,
    };
    let qido = Dcm4cheeQidoClient::new(cfg_rule);
    let evaluator = std::sync::Arc::new(DicomRbacEvaluatorImpl::new(pool.clone()));
    let jwt_service = std::sync::Arc::new(JwtService::new(&pacs_server::infrastructure::config::JwtConfig {
        secret: "test_secret_key_for_jwt_service_integration_tests".to_string(),
        expiration_hours: 24,
    }));
    let ac_repo = std::sync::Arc::new(AccessConditionRepositoryImpl { pool: pool.clone() });

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(qido))
            .app_data(web::Data::new(evaluator))
            .app_data(web::Data::new(jwt_service))
            .app_data(web::Data::new(ac_repo))
            .service(
                web::scope("/api/dicom")
                    .route("/studies", web::get().to(dicom_gateway_controller::get_studies)),
            ),
    )
    .await;

    // 잠시 대기 (서버 시작 대기)
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let token = format!("Bearer test_token_user_{}", user_id);
    let req = test::TestRequest::get()
        .uri(&format!("/api/dicom/studies?project_id={}", project_a_id))
        .insert_header(("Authorization", token))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // 규칙 기반 필터링: Modality=CT 조건이 QIDO 파라미터로 전달되어 CT만 반환되어야 함
    assert!(resp.status().is_success(), "Request should succeed");
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    println!("Rule-based filtering response: {}", serde_json::to_string_pretty(&body).unwrap());
    
    // Note: 규칙 기반 필터링은 QIDO 파라미터로 전달되므로, Dcm4chee에서 필터링된 결과가 반환됨
    // 여기서는 규칙이 제대로 적용되었는지 확인 (CT만 반환되었는지)
    if let Some(array) = body.as_array() {
        for item in array.iter() {
            if let Some(modality) = item.get("00080060")
                .and_then(|v| v.get("Value"))
                .and_then(|v| v.as_array())
                .and_then(|arr| arr.get(0))
                .and_then(|v| v.as_str())
            {
                assert_eq!(modality, "CT", "Rule-based filter should only return CT");
            }
        }
    }
    
    cleanup_test_data(&pool).await;
}

// ========================================
// 시나리오 3: RBAC 필터링 (명시적 접근 권한)
// ========================================

#[tokio::test]
#[ignore]
#[ignore]
async fn test_scenario_rbac_explicit_filtering() {
    let pool = get_test_pool().await;
    cleanup_test_data(&pool).await;

    let (user_id, project_a_id, _, study_a1_uid, study_a2_uid, _, _, _, _) = setup_scenario_data(&pool).await;

    // 사용자는 Study A1에만 명시적 접근 권한이 있음 (setup_scenario_data에서 설정됨)
    
    // Mock QIDO server
    let study_a1_clone3 = study_a1_uid.clone();
    let study_a2_clone3 = study_a2_uid.clone();
    
    let listener3 = TcpListener::bind("127.0.0.1:0").expect("Failed to bind");
    let port3 = listener3.local_addr().unwrap().port();
    let server3 = HttpServer::new(move || {
        let s1 = study_a1_clone3.clone();
        let s2 = study_a2_clone3.clone();
        App::new().route("/rs/studies", web::get().to(move || {
            let s1 = s1.clone();
            let s2 = s2.clone();
            async move {
                HttpResponse::Ok().json(json!([
                    {"0020000D": {"Value": [s1], "vr": "UI"}, "00080060": {"Value": ["CT"], "vr": "CS"}},
                    {"0020000D": {"Value": [s2], "vr": "UI"}, "00080060": {"Value": ["MR"], "vr": "CS"}},
                ]))
            }
        }))
    })
    .listen(listener3)
    .expect("Failed to start mock server")
    .run();
    
    actix_rt::spawn(server3);

    let cfg3 = pacs_server::infrastructure::config::Dcm4cheeConfig {
        base_url: format!("http://127.0.0.1:{}", port3),
        qido_path: "/rs".to_string(),
        wado_path: "/wado".to_string(),
        aet: "TEST".to_string(),
        username: None,
        password: None,
        timeout_ms: 5000,
            db: None,
    };
    let qido = Dcm4cheeQidoClient::new(cfg3);
    let evaluator = std::sync::Arc::new(DicomRbacEvaluatorImpl::new(pool.clone()));
    let jwt_service = std::sync::Arc::new(JwtService::new(&pacs_server::infrastructure::config::JwtConfig {
        secret: "test_secret_key_for_jwt_service_integration_tests".to_string(),
        expiration_hours: 24,
    }));
    let ac_repo = std::sync::Arc::new(AccessConditionRepositoryImpl { pool: pool.clone() });

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(qido))
            .app_data(web::Data::new(evaluator))
            .app_data(web::Data::new(jwt_service))
            .app_data(web::Data::new(ac_repo))
            .service(
                web::scope("/api/dicom")
                    .route("/studies", web::get().to(dicom_gateway_controller::get_studies)),
            ),
    )
    .await;

    // 잠시 대기 (서버 시작 대기)
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let token = format!("Bearer test_token_user_{}", user_id);
    let req = test::TestRequest::get()
        .uri(&format!("/api/dicom/studies?project_id={}", project_a_id))
        .insert_header(("Authorization", token))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // RBAC 필터링이 동작하면 Study A1만 반환되어야 함 (Study A2는 권한 없음)
    assert!(resp.status().is_success(), "Request should succeed");
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    println!("RBAC filtering response: {}", serde_json::to_string_pretty(&body).unwrap());
    
    if let Some(array) = body.as_array() {
        // 사용자는 Study A1에만 접근 권한이 있으므로 1개만 반환되어야 함
        assert_eq!(array.len(), 1, "Should return only Study A1 (user has explicit access)");
        if let Some(item) = array.get(0) {
            if let Some(uid) = extract_study_uid_test(item) {
                assert_eq!(uid, study_a1_uid, "Should return Study A1 UID");
            }
        }
    } else {
        panic!("Response should be an array");
    }
    
    cleanup_test_data(&pool).await;
}

// ========================================
// 시나리오 4: 계층적 필터링 (Study → Series)
// ========================================

#[tokio::test]
#[ignore]
#[ignore]
async fn test_scenario_hierarchical_filtering() {
    let pool = get_test_pool().await;
    cleanup_test_data(&pool).await;

    let (user_id, project_a_id, _, study_a1_uid, _, _, series_a1_uid, series_a2_uid, _) = setup_scenario_data(&pool).await;

    // Mock QIDO server: Series 반환
    let study_uid_clone = study_a1_uid.clone();
    let series_a1_clone = series_a1_uid.clone();
    let series_a2_clone = series_a2_uid.clone();
    
    let listener4 = TcpListener::bind("127.0.0.1:0").expect("Failed to bind");
    let port4 = listener4.local_addr().unwrap().port();
    let server4 = HttpServer::new(move || {
        let su = study_uid_clone.clone();
        let s1 = series_a1_clone.clone();
        let s2 = series_a2_clone.clone();
        App::new().route(
            &format!("/rs/studies/{}/series", su),
            web::get().to(move || {
                let s1 = s1.clone();
                let s2 = s2.clone();
                async move {
                    HttpResponse::Ok().json(json!([
                        {"0020000E": {"Value": [s1], "vr": "UI"}, "00080060": {"Value": ["CT"], "vr": "CS"}},
                        {"0020000E": {"Value": [s2], "vr": "UI"}, "00080060": {"Value": ["CT"], "vr": "CS"}},
                    ]))
                }
            })
        )
    })
    .listen(listener4)
    .expect("Failed to start mock server")
    .run();
    
    actix_rt::spawn(server4);

    let cfg4 = pacs_server::infrastructure::config::Dcm4cheeConfig {
        base_url: format!("http://127.0.0.1:{}", port4),
        qido_path: "/rs".to_string(),
        wado_path: "/wado".to_string(),
        aet: "TEST".to_string(),
        username: None,
        password: None,
        timeout_ms: 5000,
            db: None,
    };
    let qido = Dcm4cheeQidoClient::new(cfg4);
    let evaluator = std::sync::Arc::new(DicomRbacEvaluatorImpl::new(pool.clone()));
    let jwt_service = std::sync::Arc::new(JwtService::new(&pacs_server::infrastructure::config::JwtConfig {
        secret: "test_secret_key_for_jwt_service_integration_tests".to_string(),
        expiration_hours: 24,
    }));
    let ac_repo = std::sync::Arc::new(AccessConditionRepositoryImpl { pool: pool.clone() });

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(qido))
            .app_data(web::Data::new(evaluator))
            .app_data(web::Data::new(jwt_service))
            .app_data(web::Data::new(ac_repo))
            .service(
                web::scope("/api/dicom")
                    .route("/studies/{study_uid}/series", web::get().to(dicom_gateway_controller::get_series)),
            ),
    )
    .await;

    // 잠시 대기 (서버 시작 대기)
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let token = format!("Bearer test_token_user_{}", user_id);
    let req = test::TestRequest::get()
        .uri(&format!("/api/dicom/studies/{}/series?project_id={}", study_a1_uid, project_a_id))
        .insert_header(("Authorization", token))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // 계층적 필터링: Study A1에 대한 접근 권한이 있으면 Series도 상속되어 반환되어야 함
    assert!(resp.status().is_success(), "Request should succeed");
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    println!("Hierarchical filtering response: {}", serde_json::to_string_pretty(&body).unwrap());
    
    if let Some(array) = body.as_array() {
        // Study A1의 Series는 Study 권한을 상속받으므로 반환되어야 함
        assert!(array.len() > 0, "Should return Series from Study A1 (inherited access)");
    }
    
    cleanup_test_data(&pool).await;
}

