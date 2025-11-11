use pacs_server::application::dto::project_data_access_dto::{
    AssignSeriesToProjectRequest, AssignStudyToProjectRequest,
};
use pacs_server::application::use_cases::project_data_access_use_case::ProjectDataAccessUseCase;
use pacs_server::domain::entities::project::ProjectStatus;
use pacs_server::domain::services::ProjectServiceImpl;
use pacs_server::domain::ServiceError;
use pacs_server::infrastructure::repositories::{
    ProjectDataAccessRepositoryImpl, ProjectDataRepositoryImpl, ProjectRepositoryImpl,
    RoleRepositoryImpl, UserRepositoryImpl,
};
use pacs_server::infrastructure::services::ProjectDataServiceImpl;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use uuid::Uuid;

// ========================================
// 단위 테스트: Use Case 로직 검증
// ========================================

/// 단위 테스트 1: Series 할당 시 부모 Study가 없으면 자동 생성
#[tokio::test]
#[ignore] // 실제 DB 필요
async fn test_assign_series_creates_study_if_not_exists() {
    // Given: Use Case 및 프로젝트 설정
    let (use_case, pool) = setup_use_case().await;
    let project_id = create_test_project(&pool).await;

    // When: Study가 없는 상태에서 Series 할당
    let study_uid = format!("1.2.840.113619.2.1.1.{}", Uuid::new_v4().as_u128());
    let series_uid = format!("1.2.840.113619.2.1.2.{}.1", Uuid::new_v4().as_u128());

    let request = AssignSeriesToProjectRequest {
        study_uid: study_uid.clone(),
        series_uid: series_uid.clone(),
        series_description: Some("Axial CT 5mm".to_string()),
        modality: Some("CT".to_string()),
        series_number: Some(1),
    };

    let result = use_case
        .assign_series_to_project(project_id, request)
        .await;

    // Then: 성공 및 Study가 자동 생성되었는지 확인
    assert!(result.is_ok(), "Series assignment should succeed");

    // Study가 데이터베이스에 생성되었는지 확인
    let study_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM project_data_study WHERE study_uid = $1)",
    )
    .bind(&study_uid)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert!(study_exists, "Study should be auto-created");

    // Study가 프로젝트에 매핑되었는지 확인
    let study_mapping_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(
            SELECT 1 FROM project_data pd
            INNER JOIN project_data_study pds ON pd.study_id = pds.id
            WHERE pd.project_id = $1 AND pds.study_uid = $2 AND pd.resource_level = 'STUDY'
        )",
    )
    .bind(project_id)
    .bind(&study_uid)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert!(study_mapping_exists, "Study should be mapped to project");

    // Cleanup
    cleanup_test_data(&pool, project_id).await;
}

/// 단위 테스트 2: 존재하지 않는 프로젝트에 할당 시 NotFound 에러
#[tokio::test]
#[ignore]
async fn test_assign_series_to_nonexistent_project_returns_error() {
    // Given: Use Case 설정
    let (use_case, _pool) = setup_use_case().await;
    let nonexistent_project_id = 999999;

    // When: 존재하지 않는 프로젝트에 Series 할당 시도
    let request = AssignSeriesToProjectRequest {
        study_uid: format!("1.2.840.113619.2.1.1.{}", Uuid::new_v4().as_u128()),
        series_uid: format!("1.2.840.113619.2.1.2.{}.1", Uuid::new_v4().as_u128()),
        series_description: Some("Axial CT 5mm".to_string()),
        modality: Some("CT".to_string()),
        series_number: Some(1),
    };

    let result = use_case
        .assign_series_to_project(nonexistent_project_id, request)
        .await;

    // Then: NotFound 에러 반환
    assert!(result.is_err(), "Should return error");
    match result.unwrap_err() {
        ServiceError::NotFound(_) => {
            // 성공
        }
        other => panic!("Expected NotFound error, got: {:?}", other),
    }
}

/// 단위 테스트 3: 중복 Series 할당 시 AlreadyExists 에러
#[tokio::test]
#[ignore]
async fn test_assign_duplicate_series_returns_error() {
    // Given: 프로젝트 및 이미 할당된 Series
    let (use_case, pool) = setup_use_case().await;
    let project_id = create_test_project(&pool).await;

    let study_uid = format!("1.2.840.113619.2.1.1.{}", Uuid::new_v4().as_u128());
    let series_uid = format!("1.2.840.113619.2.1.2.{}.1", Uuid::new_v4().as_u128());

    let request = AssignSeriesToProjectRequest {
        study_uid: study_uid.clone(),
        series_uid: series_uid.clone(),
        series_description: Some("Axial CT 5mm".to_string()),
        modality: Some("CT".to_string()),
        series_number: Some(1),
    };

    // 첫 번째 할당 (성공)
    let result1 = use_case
        .assign_series_to_project(project_id, request.clone())
        .await;
    assert!(result1.is_ok(), "First assignment should succeed");

    // When: 동일한 Series를 다시 할당 시도
    let result2 = use_case
        .assign_series_to_project(project_id, request)
        .await;

    // Then: AlreadyExists 에러 반환
    assert!(result2.is_err(), "Duplicate assignment should fail");
    match result2.unwrap_err() {
        ServiceError::AlreadyExists(_) => {
            // 성공
        }
        other => panic!("Expected AlreadyExists error, got: {:?}", other),
    }

    // Cleanup
    cleanup_test_data(&pool, project_id).await;
}

/// 단위 테스트 4: Study 할당 시 올바른 메타데이터 저장
#[tokio::test]
#[ignore]
async fn test_assign_study_saves_metadata_correctly() {
    // Given: Use Case 및 프로젝트 설정
    let (use_case, pool) = setup_use_case().await;
    let project_id = create_test_project(&pool).await;

    // When: Study 할당 (메타데이터 포함)
    let study_uid = format!("1.2.840.113619.2.1.1.{}", Uuid::new_v4().as_u128());

    let request = AssignStudyToProjectRequest {
        study_uid: study_uid.clone(),
        study_description: Some("CT Chest with Contrast".to_string()),
        patient_id: Some("P12345".to_string()),
        patient_name: Some("John Doe".to_string()),
        study_date: Some("2024-01-15".to_string()),
        modality: Some("CT".to_string()),
    };

    let result = use_case.assign_study_to_project(project_id, request).await;

    // Then: 성공 및 메타데이터 확인
    assert!(result.is_ok(), "Study assignment should succeed");

    // 데이터베이스에서 메타데이터 확인
    let study: (String, Option<String>, Option<String>, Option<String>) = sqlx::query_as(
        "SELECT study_uid, study_description, patient_id, patient_name
         FROM project_data_study
         WHERE study_uid = $1",
    )
    .bind(&study_uid)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(study.0, study_uid);
    assert_eq!(
        study.1,
        Some("CT Chest with Contrast".to_string()),
        "Study description should match"
    );
    assert_eq!(study.2, Some("P12345".to_string()), "Patient ID should match");
    assert_eq!(
        study.3,
        Some("John Doe".to_string()),
        "Patient name should match"
    );

    // Cleanup
    cleanup_test_data(&pool, project_id).await;
}

/// 단위 테스트 5: Series 할당 시 Series 메타데이터 저장
#[tokio::test]
#[ignore]
async fn test_assign_series_saves_metadata_correctly() {
    // Given: Use Case 및 프로젝트 설정
    let (use_case, pool) = setup_use_case().await;
    let project_id = create_test_project(&pool).await;

    // When: Series 할당 (메타데이터 포함)
    let study_uid = format!("1.2.840.113619.2.1.1.{}", Uuid::new_v4().as_u128());
    let series_uid = format!("1.2.840.113619.2.1.2.{}.1", Uuid::new_v4().as_u128());

    let request = AssignSeriesToProjectRequest {
        study_uid: study_uid.clone(),
        series_uid: series_uid.clone(),
        series_description: Some("Axial CT 5mm".to_string()),
        modality: Some("CT".to_string()),
        series_number: Some(1),
    };

    let result = use_case
        .assign_series_to_project(project_id, request)
        .await;

    // Then: 성공 및 메타데이터 확인
    assert!(result.is_ok(), "Series assignment should succeed");

    // 데이터베이스에서 메타데이터 확인
    let series: (String, Option<String>, Option<String>, Option<i32>) = sqlx::query_as(
        "SELECT series_uid, series_description, modality, series_number
         FROM project_data_series
         WHERE series_uid = $1",
    )
    .bind(&series_uid)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(series.0, series_uid);
    assert_eq!(
        series.1,
        Some("Axial CT 5mm".to_string()),
        "Series description should match"
    );
    assert_eq!(series.2, Some("CT".to_string()), "Modality should match");
    assert_eq!(series.3, Some(1), "Series number should match");

    // Cleanup
    cleanup_test_data(&pool, project_id).await;
}

/// 단위 테스트 6: 중복 Study 할당 시 AlreadyExists 에러
#[tokio::test]
#[ignore]
async fn test_assign_duplicate_study_returns_error() {
    // Given: 프로젝트 및 이미 할당된 Study
    let (use_case, pool) = setup_use_case().await;
    let project_id = create_test_project(&pool).await;

    let study_uid = format!("1.2.840.113619.2.1.1.{}", Uuid::new_v4().as_u128());

    let request = AssignStudyToProjectRequest {
        study_uid: study_uid.clone(),
        study_description: Some("CT Chest".to_string()),
        patient_id: Some("P12345".to_string()),
        patient_name: Some("John Doe".to_string()),
        study_date: Some("2024-01-15".to_string()),
        modality: Some("CT".to_string()),
    };

    // 첫 번째 할당 (성공)
    let result1 = use_case
        .assign_study_to_project(project_id, request.clone())
        .await;
    assert!(result1.is_ok(), "First assignment should succeed");

    // When: 동일한 Study를 다시 할당 시도
    let result2 = use_case.assign_study_to_project(project_id, request).await;

    // Then: AlreadyExists 에러 반환
    assert!(result2.is_err(), "Duplicate assignment should fail");
    match result2.unwrap_err() {
        ServiceError::AlreadyExists(_) => {
            // 성공
        }
        other => panic!("Expected AlreadyExists error, got: {:?}", other),
    }

    // Cleanup
    cleanup_test_data(&pool, project_id).await;
}

// ========================================
// 헬퍼 함수들
// ========================================

/// Use Case 설정
async fn setup_use_case() -> (Arc<ProjectDataAccessUseCase>, sqlx::PgPool) {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .or_else(|_| std::env::var("APP_DATABASE_URL"))
        .unwrap_or_else(|_| {
            "postgres://pacs_extension_admin:PacsExtension2024@localhost:5456/pacs_extension"
                .to_string()
        });

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    // 서비스 설정
    let user_repo = UserRepositoryImpl::new(pool.clone());
    let project_repo = ProjectRepositoryImpl::new(pool.clone());
    let role_repo = RoleRepositoryImpl::new(pool.clone());
    let project_data_repo = Arc::new(ProjectDataRepositoryImpl::new(pool.clone()));
    let project_data_access_repo = Arc::new(ProjectDataAccessRepositoryImpl::new(pool.clone()));

    let project_service = Arc::new(ProjectServiceImpl::new(project_repo, user_repo, role_repo));
    let project_data_service = ProjectDataServiceImpl::new(
        project_data_repo.clone(),
        project_data_access_repo.clone(),
    );

    // ProjectDataService trait object로 변환
    let project_data_service_arc: Arc<dyn pacs_server::domain::services::ProjectDataService> =
        Arc::new(project_data_service);

    let use_case = Arc::new(ProjectDataAccessUseCase::new(
        project_data_service_arc.clone(),
        project_service.clone(),
    ));

    (use_case, pool)
}

/// 테스트 프로젝트 생성
async fn create_test_project(pool: &sqlx::PgPool) -> i32 {
    let project_name = format!("Test Project {}", Uuid::new_v4());
    let description = "Test project for unit tests";

    let rec: (i32,) = sqlx::query_as(
        "INSERT INTO security_project (name, description, sponsor, start_date, end_date, status)
         VALUES ($1, $2, 'Test Sponsor', CURRENT_DATE, CURRENT_DATE + INTERVAL '1 year', 'PREPARING'::project_status)
         RETURNING id",
    )
    .bind(&project_name)
    .bind(description)
    .fetch_one(pool)
    .await
    .expect("Failed to create test project");

    rec.0
}

/// 테스트 데이터 정리
async fn cleanup_test_data(pool: &sqlx::PgPool, project_id: i32) {
    // project_data 삭제
    let _ = sqlx::query("DELETE FROM project_data WHERE project_id = $1")
        .bind(project_id)
        .execute(pool)
        .await;

    // 프로젝트 삭제
    let _ = sqlx::query("DELETE FROM security_project WHERE id = $1")
        .bind(project_id)
        .execute(pool)
        .await;

    // 고아 Study/Series 정리
    let _ = sqlx::query(
        "DELETE FROM project_data_study WHERE id NOT IN (SELECT DISTINCT study_id FROM project_data WHERE study_id IS NOT NULL)"
    )
    .execute(pool)
    .await;

    let _ = sqlx::query(
        "DELETE FROM project_data_series WHERE id NOT IN (SELECT DISTINCT series_id FROM project_data WHERE series_id IS NOT NULL)"
    )
    .execute(pool)
    .await;
}

