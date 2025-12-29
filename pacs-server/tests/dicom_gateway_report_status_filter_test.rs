//! # DICOM Gateway Report Status 필터링 통합 테스트
//!
//! 이 테스트는 Series Report Status 필터링 기능을 검증합니다:
//! 1. 배치 Series ID 조회
//! 2. 배치 Report Status 조회
//! 3. Report Status 필터링
//! 4. 엔드포인트 통합 테스트

use actix_web::test::TestRequest;
use actix_web::{test, web, App};
use serde_json::json;
use sqlx::PgPool;

use pacs_server::presentation::controllers::dicom_gateway_controller::{
    get_report_statuses_batch, get_series_ids_by_uids_batch,
    filter_series_by_report_status_batch, parse_report_status_filter,
};
use std::collections::HashMap;

use pacs_server::infrastructure::auth::JwtService;
use pacs_server::infrastructure::external::Dcm4cheeQidoClient;
use pacs_server::infrastructure::repositories::{
    AccessConditionRepositoryImpl, ProjectDataRepositoryImpl, UserRepositoryImpl,
};
use pacs_server::infrastructure::services::DicomRbacEvaluatorImpl;
use pacs_server::presentation::controllers::dicom_gateway_controller;

async fn get_test_pool() -> PgPool {
    let database_url = std::env::var("APP_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .unwrap_or_else(|_| {
            "postgres://pacs_extension_admin:PacsExtension2024@localhost:5456/pacs_extension"
                .to_string()
        });

    PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

async fn cleanup_test_data(pool: &PgPool) {
    sqlx::query("SET session_replication_role = replica")
        .execute(pool)
        .await
        .ok();

    sqlx::query("DELETE FROM series_user_report")
        .execute(pool)
        .await
        .ok();

    sqlx::query("DELETE FROM project_data_series")
        .execute(pool)
        .await
        .ok();

    sqlx::query("DELETE FROM project_data_study")
        .execute(pool)
        .await
        .ok();

    sqlx::query("DELETE FROM project_data")
        .execute(pool)
        .await
        .ok();

    sqlx::query("DELETE FROM security_user_project")
        .execute(pool)
        .await
        .ok();

    sqlx::query("DELETE FROM security_project")
        .execute(pool)
        .await
        .ok();

    sqlx::query("DELETE FROM security_user WHERE username LIKE 'test_%'")
        .execute(pool)
        .await
        .ok();
}

async fn create_test_user(pool: &PgPool, username: &str) -> i32 {
    let user_id: i32 = sqlx::query_scalar(
        "INSERT INTO security_user (keycloak_id, username, email, full_name, account_status)
         VALUES (gen_random_uuid(), $1, $2, $3, 'ACTIVE')
         RETURNING id"
    )
    .bind(username)
    .bind(format!("{}@test.com", username))
    .bind(format!("Test User {}", username))
    .fetch_one(pool)
    .await
    .expect("Failed to create test user");

    user_id
}

async fn create_test_project(pool: &PgPool, name: &str) -> i32 {
    let project_id: i32 = sqlx::query_scalar(
        "INSERT INTO security_project (name, description, status, sponsor, start_date)
         VALUES ($1, $2, 'ACTIVE', 'Test Sponsor', CURRENT_DATE)
         RETURNING id"
    )
    .bind(name)
    .bind(format!("Test project: {}", name))
    .fetch_one(pool)
    .await
    .expect("Failed to create test project");

    project_id
}

async fn create_test_study(pool: &PgPool, study_uid: &str, project_id: i32) -> i32 {
    // project_data_study 생성
    let study_id: i32 = sqlx::query_scalar(
        "INSERT INTO project_data_study (study_uid, study_description, patient_id)
         VALUES ($1, $2, $3)
         ON CONFLICT (study_uid) DO UPDATE SET study_uid = EXCLUDED.study_uid
         RETURNING id"
    )
    .bind(study_uid)
    .bind("Test Study")
    .bind("PAT001")
    .fetch_one(pool)
    .await
    .expect("Failed to create test study");

    // project_data에 할당
    sqlx::query(
        "INSERT INTO project_data (project_id, study_id, resource_level)
         VALUES ($1, $2, 'STUDY')
         ON CONFLICT DO NOTHING"
    )
    .bind(project_id)
    .bind(study_id)
    .execute(pool)
    .await
    .expect("Failed to assign study to project");

    study_id
}

async fn create_test_series(
    pool: &PgPool,
    study_id: i32,
    series_uid: &str,
) -> i32 {
    let series_id: i32 = sqlx::query_scalar(
        "INSERT INTO project_data_series (study_id, series_uid, series_description, modality)
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (study_id, series_uid) DO UPDATE SET series_uid = EXCLUDED.series_uid
         RETURNING id"
    )
    .bind(study_id)
    .bind(series_uid)
    .bind("Test Series")
    .bind("CT")
    .fetch_one(pool)
    .await
    .expect("Failed to create test series");

    series_id
}

async fn create_test_report(
    pool: &PgPool,
    series_id: i32,
    user_id: i32,
    project_id: Option<i32>,
    status: &str,
) {
    sqlx::query(
        "INSERT INTO series_user_report (series_id, user_id, project_id, status, description, conclusion)
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (series_id, user_id, project_id) DO UPDATE
         SET status = EXCLUDED.status,
             description = EXCLUDED.description,
             conclusion = EXCLUDED.conclusion,
             updated_at = CURRENT_TIMESTAMP"
    )
    .bind(series_id)
    .bind(user_id)
    .bind(project_id)
    .bind(status)
    .bind("Test description")
    .bind("Test conclusion")
    .execute(pool)
    .await
    .expect("Failed to create test report");
}

#[tokio::test]
#[ignore] // 통합 테스트는 명시적으로 실행
async fn test_get_series_ids_by_uids_batch_with_project() {
    let pool = get_test_pool().await;
    cleanup_test_data(&pool).await;

    let project_id = create_test_project(&pool, "test_project_1").await;
    let study_id = create_test_study(&pool, "1.2.3.4.5", project_id).await;
    let series_id_1 = create_test_series(&pool, study_id, "1.2.3.4.5.1").await;
    let series_id_2 = create_test_series(&pool, study_id, "1.2.3.4.5.2").await;
    let series_id_3 = create_test_series(&pool, study_id, "1.2.3.4.5.3").await;

    let series_uids = vec![
        "1.2.3.4.5.1".to_string(),
        "1.2.3.4.5.2".to_string(),
        "1.2.3.4.5.3".to_string(),
    ];

    // project_id가 있는 경우
    let result = dicom_gateway_controller::get_series_ids_by_uids_batch(
        &series_uids,
        Some(project_id),
        &pool,
    )
    .await
    .expect("Failed to get series IDs");

    assert_eq!(result.len(), 3);
    assert_eq!(result.get("1.2.3.4.5.1"), Some(&series_id_1));
    assert_eq!(result.get("1.2.3.4.5.2"), Some(&series_id_2));
    assert_eq!(result.get("1.2.3.4.5.3"), Some(&series_id_3));

    // 다른 프로젝트의 Series는 조회되지 않음
    let other_project_id = create_test_project(&pool, "test_project_2").await;
    let other_study_id = create_test_study(&pool, "2.3.4.5.6", other_project_id).await;
    let _other_series_id = create_test_series(&pool, other_study_id, "2.3.4.5.6.1").await;

    let result2 = get_series_ids_by_uids_batch(
        &vec!["2.3.4.5.6.1".to_string()],
        Some(project_id), // 첫 번째 프로젝트로 조회
        &pool,
    )
    .await
    .expect("Failed to get series IDs");

    assert_eq!(result2.len(), 0); // 다른 프로젝트의 Series는 조회되지 않음

    cleanup_test_data(&pool).await;
}

#[tokio::test]
#[ignore]
async fn test_get_series_ids_by_uids_batch_without_project() {
    let pool = get_test_pool().await;
    cleanup_test_data(&pool).await;

    let project_id = create_test_project(&pool, "test_project_1").await;
    let study_id = create_test_study(&pool, "1.2.3.4.5", project_id).await;
    let series_id_1 = create_test_series(&pool, study_id, "1.2.3.4.5.1").await;
    let series_id_2 = create_test_series(&pool, study_id, "1.2.3.4.5.2").await;

    let series_uids = vec![
        "1.2.3.4.5.1".to_string(),
        "1.2.3.4.5.2".to_string(),
    ];

    // project_id가 없는 경우 (전체 조회)
    let result = get_series_ids_by_uids_batch(
        &series_uids,
        None,
        &pool,
    )
    .await
    .expect("Failed to get series IDs");

    assert_eq!(result.len(), 2);
    assert_eq!(result.get("1.2.3.4.5.1"), Some(&series_id_1));
    assert_eq!(result.get("1.2.3.4.5.2"), Some(&series_id_2));

    cleanup_test_data(&pool).await;
}

#[tokio::test]
#[ignore]
async fn test_get_report_statuses_batch_project_dependent_priority() {
    let pool = get_test_pool().await;
    cleanup_test_data(&pool).await;

    let user_id = create_test_user(&pool, "test_user_1").await;
    let project_id = create_test_project(&pool, "test_project_1").await;
    let study_id = create_test_study(&pool, "1.2.3.4.5", project_id).await;
    let series_id = create_test_series(&pool, study_id, "1.2.3.4.5.1").await;

    // Global report 생성
    create_test_report(&pool, series_id, user_id, None, "unread").await;

    // Project-dependent report 생성 (우선순위가 높아야 함)
    create_test_report(&pool, series_id, user_id, Some(project_id), "approved").await;

    let series_ids = vec![series_id];

    // project_id가 있는 경우 - project-dependent가 우선
    let result = get_report_statuses_batch(
        &series_ids,
        user_id,
        Some(project_id),
        &pool,
    )
    .await
    .expect("Failed to get report statuses");

    assert_eq!(result.len(), 1);
    assert_eq!(result.get(&series_id), Some(&"approved".to_string())); // project-dependent 우선

    cleanup_test_data(&pool).await;
}

#[tokio::test]
#[ignore]
async fn test_get_report_statuses_batch_global_only() {
    let pool = get_test_pool().await;
    cleanup_test_data(&pool).await;

    let user_id = create_test_user(&pool, "test_user_1").await;
    let project_id = create_test_project(&pool, "test_project_1").await;
    let study_id = create_test_study(&pool, "1.2.3.4.5", project_id).await;
    let series_id = create_test_series(&pool, study_id, "1.2.3.4.5.1").await;

    // Global report만 생성
    create_test_report(&pool, series_id, user_id, None, "unread").await;

    let series_ids = vec![series_id];

    // project_id가 없는 경우 - global만 조회
    let result = get_report_statuses_batch(
        &series_ids,
        user_id,
        None,
        &pool,
    )
    .await
    .expect("Failed to get report statuses");

    assert_eq!(result.len(), 1);
    assert_eq!(result.get(&series_id), Some(&"unread".to_string()));

    cleanup_test_data(&pool).await;
}

#[tokio::test]
#[ignore]
async fn test_get_report_statuses_batch_multiple_series() {
    let pool = get_test_pool().await;
    cleanup_test_data(&pool).await;

    let user_id = create_test_user(&pool, "test_user_1").await;
    let project_id = create_test_project(&pool, "test_project_1").await;
    let study_id = create_test_study(&pool, "1.2.3.4.5", project_id).await;
    let series_id_1 = create_test_series(&pool, study_id, "1.2.3.4.5.1").await;
    let series_id_2 = create_test_series(&pool, study_id, "1.2.3.4.5.2").await;
    let series_id_3 = create_test_series(&pool, study_id, "1.2.3.4.5.3").await;

    create_test_report(&pool, series_id_1, user_id, Some(project_id), "approved").await;
    create_test_report(&pool, series_id_2, user_id, Some(project_id), "unread").await;
    // series_id_3는 report 없음

    let series_ids = vec![series_id_1, series_id_2, series_id_3];

    let result = get_report_statuses_batch(
        &series_ids,
        user_id,
        Some(project_id),
        &pool,
    )
    .await
    .expect("Failed to get report statuses");

    assert_eq!(result.len(), 2); // report가 있는 것만
    assert_eq!(result.get(&series_id_1), Some(&"approved".to_string()));
    assert_eq!(result.get(&series_id_2), Some(&"unread".to_string()));
    assert_eq!(result.get(&series_id_3), None);

    cleanup_test_data(&pool).await;
}

#[tokio::test]
#[ignore]
async fn test_filter_series_by_report_status_batch() {
    let pool = get_test_pool().await;
    cleanup_test_data(&pool).await;

    let user_id = create_test_user(&pool, "test_user_1").await;
    let project_id = create_test_project(&pool, "test_project_1").await;
    let study_id = create_test_study(&pool, "1.2.3.4.5", project_id).await;
    let series_id_1 = create_test_series(&pool, study_id, "1.2.3.4.5.1").await;
    let series_id_2 = create_test_series(&pool, study_id, "1.2.3.4.5.2").await;
    let series_id_3 = create_test_series(&pool, study_id, "1.2.3.4.5.3").await;

    create_test_report(&pool, series_id_1, user_id, Some(project_id), "approved").await;
    create_test_report(&pool, series_id_2, user_id, Some(project_id), "unread").await;
    // series_id_3는 report 없음

    let series_array = vec![
        json!({
            "0020000E": {"Value": ["1.2.3.4.5.1"], "vr": "UI"},
            "00080060": {"Value": ["CT"], "vr": "CS"}
        }),
        json!({
            "0020000E": {"Value": ["1.2.3.4.5.2"], "vr": "UI"},
            "00080060": {"Value": ["CT"], "vr": "CS"}
        }),
        json!({
            "0020000E": {"Value": ["1.2.3.4.5.3"], "vr": "UI"},
            "00080060": {"Value": ["CT"], "vr": "CS"}
        }),
    ];

    // approved만 필터링
    let status_filter = vec!["approved".to_string()];
    let result = filter_series_by_report_status_batch(
        &series_array,
        user_id,
        Some(project_id),
        &status_filter,
        &pool,
    )
    .await
    .expect("Failed to filter series");

    assert_eq!(result.len(), 1);
    assert_eq!(
        result[0].get("0020000E")
            .and_then(|v| v.get("Value"))
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.get(0))
            .and_then(|v| v.as_str()),
        Some("1.2.3.4.5.1")
    );

    // approved, unread 필터링
    let status_filter = vec!["approved".to_string(), "unread".to_string()];
    let result = filter_series_by_report_status_batch(
        &series_array,
        user_id,
        Some(project_id),
        &status_filter,
        &pool,
    )
    .await
    .expect("Failed to filter series");

    assert_eq!(result.len(), 2); // approved와 unread 모두 포함

    cleanup_test_data(&pool).await;
}

#[tokio::test]
#[ignore]
async fn test_filter_series_by_report_status_batch_no_report() {
    let pool = get_test_pool().await;
    cleanup_test_data(&pool).await;

    let user_id = create_test_user(&pool, "test_user_1").await;
    let project_id = create_test_project(&pool, "test_project_1").await;
    let study_id = create_test_study(&pool, "1.2.3.4.5", project_id).await;
    let series_id_1 = create_test_series(&pool, study_id, "1.2.3.4.5.1").await;

    // Report 없음

    let series_array = vec![json!({
        "0020000E": {"Value": ["1.2.3.4.5.1"], "vr": "UI"},
        "00080060": {"Value": ["CT"], "vr": "CS"}
    })];

    let status_filter = vec!["approved".to_string()];
    let result = filter_series_by_report_status_batch(
        &series_array,
        user_id,
        Some(project_id),
        &status_filter,
        &pool,
    )
    .await
    .expect("Failed to filter series");

    assert_eq!(result.len(), 0); // Report가 없으면 제외

    cleanup_test_data(&pool).await;
}

#[tokio::test]
#[ignore]
async fn test_filter_series_by_report_status_batch_empty_input() {
    let pool = get_test_pool().await;

    let series_array = vec![];

    let status_filter = vec!["approved".to_string()];
    let result = filter_series_by_report_status_batch(
        &series_array,
        1,
        Some(1),
        &status_filter,
        &pool,
    )
    .await
    .expect("Failed to filter series");

    assert_eq!(result.len(), 0);
}

#[tokio::test]
#[ignore]
async fn test_filter_series_by_report_status_batch_empty_filter() {
    let pool = get_test_pool().await;
    cleanup_test_data(&pool).await;

    let user_id = create_test_user(&pool, "test_user_1").await;
    let project_id = create_test_project(&pool, "test_project_1").await;
    let study_id = create_test_study(&pool, "1.2.3.4.5", project_id).await;
    let series_id_1 = create_test_series(&pool, study_id, "1.2.3.4.5.1").await;

    create_test_report(&pool, series_id_1, user_id, Some(project_id), "approved").await;

    let series_array = vec![json!({
        "0020000E": {"Value": ["1.2.3.4.5.1"], "vr": "UI"},
        "00080060": {"Value": ["CT"], "vr": "CS"}
    })];

    let status_filter = vec![]; // 빈 필터
    let result = filter_series_by_report_status_batch(
        &series_array,
        user_id,
        Some(project_id),
        &status_filter,
        &pool,
    )
    .await
    .expect("Failed to filter series");

    assert_eq!(result.len(), 1); // 빈 필터면 모두 반환

    cleanup_test_data(&pool).await;
}

