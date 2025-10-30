#[tokio::test]
async fn scenario_db_ct_and_date_range_perfproj() {
    use std::env;

    let db_url = match env::var("APP_DATABASE_URL") {
        Ok(v) if !v.is_empty() => v,
        _ => return, // 환경변수 없으면 스킵
    };

    let pool = sqlx::PgPool::connect(&db_url).await.expect("connect DB");

    // PerfProj 프로젝트 id
    let project_id: i32 = sqlx::query_scalar("SELECT id FROM security_project WHERE name='PerfProj'")
        .fetch_one(&pool)
        .await
        .expect("PerfProj exists");

    // CT + 날짜 범위(2024-01-01 ~ 2024-12-31)에 해당하는 study 존재 확인
    let cnt: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM project_data_study \
         WHERE project_id=$1 AND modality='CT' \
           AND study_date BETWEEN '2024-01-01' AND '2024-12-31'",
    )
    .bind(project_id)
    .fetch_one(&pool)
    .await
    .expect("count studies");

    assert!(cnt >= 1, "expected at least one CT study in 2024 for PerfProj");
}


#[tokio::test]
async fn scenario_db_series_under_seeded_study_perfproj() {
    use std::env;

    let db_url = match env::var("APP_DATABASE_URL") {
        Ok(v) if !v.is_empty() => v,
        _ => return,
    };

    let pool = sqlx::PgPool::connect(&db_url).await.expect("connect DB");

    let study_id: i32 = sqlx::query_scalar(
        "SELECT id FROM project_data_study WHERE study_uid='1.2.840.10008.1.2.1.999.1' \
         AND project_id=(SELECT id FROM security_project WHERE name='PerfProj')",
    )
    .fetch_one(&pool)
    .await
    .expect("seeded study exists");

    let series_cnt: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM project_data_series WHERE study_id=$1 AND series_uid='1.2.840.10008.1.2.1.999.1.1'",
    )
    .bind(study_id)
    .fetch_one(&pool)
    .await
    .expect("count series");

    assert!(series_cnt >= 1, "expected seeded series under seeded study");
}

#[tokio::test]
async fn scenario_db_instance_under_seeded_series_perfproj() {
    use std::env;

    let db_url = match env::var("APP_DATABASE_URL") {
        Ok(v) if !v.is_empty() => v,
        _ => return,
    };

    let pool = sqlx::PgPool::connect(&db_url).await.expect("connect DB");

    let series_id: i32 = sqlx::query_scalar(
        "SELECT id FROM project_data_series WHERE series_uid='1.2.840.10008.1.2.1.999.1.1'",
    )
    .fetch_one(&pool)
    .await
    .expect("seeded series exists");

    let inst_cnt: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM project_data_instance WHERE series_id=$1 AND instance_uid='1.2.840.10008.1.2.1.999.1.1.1'",
    )
    .bind(series_id)
    .fetch_one(&pool)
    .await
    .expect("count instances");

    assert!(inst_cnt >= 1, "expected seeded instance under seeded series");
}

// ========================================
// 규칙 시뮬레이션형 검증 (DB 수준 필터 모사)
// - DENY(PatientID) 적용 시 결과가 0으로 축소되는지
// - LIMIT(StudyDate RANGE) 교집합이 비어지는 케이스 확인
// 실제 evaluator 로직은 애플리케이션 레벨에서 수행되므로,
// 여기서는 동일한 효과의 WHERE 절을 적용해 기대 동작을 검증한다.
// ========================================
#[tokio::test]
async fn scenario_db_deny_patient_id_effect() {
    use std::env;

    let db_url = match env::var("APP_DATABASE_URL") {
        Ok(v) if !v.is_empty() => v,
        _ => return,
    };

    let pool = sqlx::PgPool::connect(&db_url).await.expect("connect DB");

    let project_id: i32 = sqlx::query_scalar("SELECT id FROM security_project WHERE name='PerfProj'")
        .fetch_one(&pool)
        .await
        .expect("PerfProj exists");

    // 시드 데이터 자체는 존재해야 함
    let base_cnt: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM project_data_study \
         WHERE project_id=$1 AND patient_id='PAT-EXPLAIN-001'",
    )
    .bind(project_id)
    .fetch_one(&pool)
    .await
    .expect("base count");
    assert!(base_cnt >= 1, "seeded patient should exist");

    // DENY(PatientID='PAT-EXPLAIN-001') 효과 모사 → 결과 0 기대
    let denied_cnt: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM project_data_study \
         WHERE project_id=$1 AND patient_id <> 'PAT-EXPLAIN-001' \
           AND study_uid='1.2.840.10008.1.2.1.999.1'",
    )
    .bind(project_id)
    .fetch_one(&pool)
    .await
    .expect("deny count");
    assert_eq!(denied_cnt, 0, "deny on PatientID should exclude the seeded row");
}

#[tokio::test]
async fn scenario_db_limit_date_range_intersection_empty() {
    use std::env;

    let db_url = match env::var("APP_DATABASE_URL") {
        Ok(v) if !v.is_empty() => v,
        _ => return,
    };

    let pool = sqlx::PgPool::connect(&db_url).await.expect("connect DB");

    let project_id: i32 = sqlx::query_scalar("SELECT id FROM security_project WHERE name='PerfProj'")
        .fetch_one(&pool)
        .await
        .expect("PerfProj exists");

    // 시드 스터디는 2024-06-15 → 2023 범위를 LIMIT 하면 교집합은 비어야 함
    let cnt: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM project_data_study \
         WHERE project_id=$1 AND study_date BETWEEN '2023-01-01' AND '2023-12-31' \
           AND study_uid='1.2.840.10008.1.2.1.999.1'",
    )
    .bind(project_id)
    .fetch_one(&pool)
    .await
    .expect("limit date range count");
    assert_eq!(cnt, 0, "limit with non-overlapping date range should yield empty set");
}

#[tokio::test]
async fn scenario_db_deny_over_allow_simulation() {
    use std::env;

    let db_url = match env::var("APP_DATABASE_URL") {
        Ok(v) if !v.is_empty() => v,
        _ => return,
    };

    let pool = sqlx::PgPool::connect(&db_url).await.expect("connect DB");

    let project_id: i32 = sqlx::query_scalar("SELECT id FROM security_project WHERE name='PerfProj'")
        .fetch_one(&pool)
        .await
        .expect("PerfProj exists");

    // 시드: Modality='CT', PatientID='PAT-EXPLAIN-001', StudyDate='2024-06-15'
    // ALLOW(Modality=CT)와 동시에 DENY(PatientID=PAT-EXPLAIN-001)가 있으면 최종 제외되어야 함
    // SQL로는 ALLOW 후보셋에서 DENY를 제외한 결과가 0이 되는지 확인

    let allow_ct_cnt: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM project_data_study \
         WHERE project_id=$1 AND modality='CT' AND study_uid='1.2.840.10008.1.2.1.999.1'",
    )
    .bind(project_id)
    .fetch_one(&pool)
    .await
    .expect("allow set count");
    assert_eq!(allow_ct_cnt, 1, "seed should be CT");

    let final_cnt: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM project_data_study \
         WHERE project_id=$1 AND modality='CT' AND patient_id <> 'PAT-EXPLAIN-001' \
           AND study_uid='1.2.840.10008.1.2.1.999.1'",
    )
    .bind(project_id)
    .fetch_one(&pool)
    .await
    .expect("final count after deny");
    assert_eq!(final_cnt, 0, "DENY must override ALLOW resulting in exclusion");
}

#[tokio::test]
async fn scenario_db_allow_ct_and_limit_june_positive() {
    use std::env;

    let db_url = match env::var("APP_DATABASE_URL") {
        Ok(v) if !v.is_empty() => v,
        _ => return,
    };

    let pool = sqlx::PgPool::connect(&db_url).await.expect("connect DB");

    let project_id: i32 = sqlx::query_scalar("SELECT id FROM security_project WHERE name='PerfProj'")
        .fetch_one(&pool)
        .await
        .expect("PerfProj exists");

    // ALLOW(Modality=CT) ∩ LIMIT(2024-06-01..2024-06-30) → 시드 1건 유지 기대
    let cnt: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM project_data_study \
         WHERE project_id=$1 AND modality='CT' \
           AND study_date BETWEEN '2024-06-01' AND '2024-06-30' \
           AND study_uid='1.2.840.10008.1.2.1.999.1'",
    )
    .bind(project_id)
    .fetch_one(&pool)
    .await
    .expect("allow+limit positive count");
    assert_eq!(cnt, 1, "ALLOW CT with LIMIT June should retain seeded row");
}


