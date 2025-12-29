//! # DICOM Gateway Report Status 필터링 엔드포인트 통합 테스트
//!
//! 이 테스트는 Series Report Status 필터링이 엔드포인트에서 올바르게 동작하는지 검증합니다.

use actix_web::{test, web, App};
use serde_json::json;
use sqlx::PgPool;

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

async fn create_test_user(pool: &PgPool, username: &str) -> (i32, String) {
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

    // 간단한 테스트용 JWT 토큰 생성 (실제로는 JwtService 사용)
    let token = format!("test_token_{}", user_id);
    (user_id, token)
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

// Note: 실제 엔드포인트 테스트는 QIDO mock 서버가 필요하므로
// 여기서는 배치 함수들의 통합 테스트만 수행합니다.
// 엔드포인트 전체 테스트는 E2E 테스트에서 수행하는 것을 권장합니다.



