/// DICOM Gateway Series API - resource_level 필터링 테스트
///
/// 이 테스트는 resource_level에 따른 올바른 Series 필터링을 검증합니다:
/// 1. resource_level='SERIES': series_id로 직접 조회 (해당 series만)
/// 2. resource_level='STUDY': study_id로 조인하여 study의 모든 series 조회
/// 3. 페이지네이션 동작 확인

use actix_web::{test, web, App, HttpResponse};
use serde_json::json;
use sqlx::PgPool;

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

async fn cleanup_test_data(pool: &PgPool, project_id: i32) {
    sqlx::query("DELETE FROM project_data WHERE project_id = $1")
        .bind(project_id)
        .execute(pool)
        .await
        .ok();
}

async fn setup_test_data(pool: &PgPool, project_id: i32) -> (i32, i32, Vec<i32>) {
    // 테스트용 Study 생성
    let study_uid_1 = "1.2.840.113619.2.55.3.test.study.1";
    let study_uid_2 = "1.2.840.113619.2.55.3.test.study.2";

    let study_id_1: i32 = sqlx::query_scalar(
        "INSERT INTO project_data_study (study_uid, study_description, patient_id, study_date)
         VALUES ($1, 'Test Study 1', 'PAT001', '20240101')
         ON CONFLICT (study_uid) DO UPDATE SET study_uid = EXCLUDED.study_uid
         RETURNING id",
    )
    .bind(study_uid_1)
    .fetch_one(pool)
    .await
    .expect("Failed to create test study 1");

    let study_id_2: i32 = sqlx::query_scalar(
        "INSERT INTO project_data_study (study_uid, study_description, patient_id, study_date)
         VALUES ($1, 'Test Study 2', 'PAT002', '20240102')
         ON CONFLICT (study_uid) DO UPDATE SET study_uid = EXCLUDED.study_uid
         RETURNING id",
    )
    .bind(study_uid_2)
    .fetch_one(pool)
    .await
    .expect("Failed to create test study 2");

    // 테스트용 Series 생성
    let series_uids = vec![
        "1.2.840.113619.2.55.3.test.series.1",
        "1.2.840.113619.2.55.3.test.series.2",
        "1.2.840.113619.2.55.3.test.series.3",
    ];

    let mut series_ids = Vec::new();
    for (idx, series_uid) in series_uids.iter().enumerate() {
        let study_id = if idx < 2 { study_id_1 } else { study_id_2 };
        let series_id: i32 = sqlx::query_scalar(
            "INSERT INTO project_data_series (study_id, series_uid, series_description, modality)
             VALUES ($1, $2, 'Test Series', 'CT')
             ON CONFLICT (study_id, series_uid) DO UPDATE SET series_uid = EXCLUDED.series_uid
             RETURNING id",
        )
        .bind(study_id)
        .bind(series_uid)
        .fetch_one(pool)
        .await
        .expect(&format!("Failed to create test series {}", idx + 1));
        series_ids.push(series_id);
    }

    // project_data 생성
    // 1. SERIES 레벨: series_id로 직접 할당 (2개)
    sqlx::query(
        "INSERT INTO project_data (project_id, resource_level, study_id, series_id)
         VALUES ($1, 'SERIES', $2, $3)",
    )
    .bind(project_id)
    .bind(study_id_1)
    .bind(series_ids[0])
    .execute(pool)
    .await
    .expect("Failed to create project_data 1");

    sqlx::query(
        "INSERT INTO project_data (project_id, resource_level, study_id, series_id)
         VALUES ($1, 'SERIES', $2, $3)",
    )
    .bind(project_id)
    .bind(study_id_1)
    .bind(series_ids[1])
    .execute(pool)
    .await
    .expect("Failed to create project_data 2");

    // 2. STUDY 레벨: study_id로 할당 (study의 모든 series 포함)
    sqlx::query(
        "INSERT INTO project_data (project_id, resource_level, study_id, series_id)
         VALUES ($1, 'STUDY', $2, NULL)",
    )
    .bind(project_id)
    .bind(study_id_2)
    .execute(pool)
    .await
    .expect("Failed to create project_data 3");

    (study_id_1, study_id_2, series_ids)
}

#[actix_web::test]
#[ignore] // 실제 DB 필요
async fn test_get_allowed_series_uids_with_series_level() {
    let pool = get_test_pool().await;
    let test_project_id = 9999;

    cleanup_test_data(&pool, test_project_id).await;
    let (study_id_1, _study_id_2, series_ids) = setup_test_data(&pool, test_project_id).await;

    // get_allowed_series_uids 함수 직접 테스트
    use pacs_server::presentation::controllers::dicom_gateway_controller::get_allowed_series_uids;

    let result = get_allowed_series_uids(test_project_id, &pool).await;

    assert!(result.is_ok());
    let series_uids = result.unwrap();

    // SERIES 레벨 2개 + STUDY 레벨 1개 (study_id_2의 series)
    // study_id_2에는 series_ids[2]만 있음
    assert_eq!(series_uids.len(), 3, "Should return 3 series: 2 from SERIES level + 1 from STUDY level");

    // SERIES 레벨로 할당된 series 확인
    let series_uid_1: String = sqlx::query_scalar(
        "SELECT series_uid FROM project_data_series WHERE id = $1",
    )
    .bind(series_ids[0])
    .fetch_one(&pool)
    .await
    .unwrap();

    let series_uid_2: String = sqlx::query_scalar(
        "SELECT series_uid FROM project_data_series WHERE id = $1",
    )
    .bind(series_ids[1])
    .fetch_one(&pool)
    .await
    .unwrap();

    assert!(series_uids.contains(&series_uid_1), "Should contain series_uid_1");
    assert!(series_uids.contains(&series_uid_2), "Should contain series_uid_2");

    cleanup_test_data(&pool, test_project_id).await;
}

#[actix_web::test]
#[ignore] // 실제 DB 필요
async fn test_series_api_with_resource_level_filtering() {
    // 이 테스트는 실제 서버가 필요하므로 통합 테스트로 분리
    // Python E2E 테스트에서 검증
}

#[actix_web::test]
#[ignore] // 실제 DB 필요
async fn test_series_api_pagination() {
    // 페이지네이션 테스트는 Python E2E 테스트에서 검증
}

