use pacs_server::domain::entities::SeriesUserNote;
use pacs_server::domain::repositories::{
    ProjectDataRepository, ProjectRepository, SeriesUserNoteRepository, UserRepository,
};
use pacs_server::domain::services::{SeriesUserNoteService, SeriesUserNoteServiceImpl};
use pacs_server::domain::ServiceError;
use pacs_server::infrastructure::repositories::{
    ProjectDataRepositoryImpl, ProjectRepositoryImpl, SeriesUserNoteRepositoryImpl,
    UserRepositoryImpl,
};
use sqlx::PgPool;
use std::sync::Arc;

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

async fn setup_test_data(pool: &PgPool) -> (i32, i32, i32) {
    // 1. 사용자 생성
    let user_id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO security_user (username, email, password_hash, is_active)
         VALUES ($1, $2, 'hashed_password', true)
         RETURNING id",
    )
    .bind(format!("test_user_{}", uuid::Uuid::new_v4()))
    .bind(format!("test_{}@example.com", uuid::Uuid::new_v4()))
    .fetch_one(pool)
    .await
    .expect("Failed to create test user");

    // 2. 프로젝트 생성
    let project_id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO security_project (project_name, description, status, is_active)
         VALUES ($1, 'Test Project', 'ACTIVE', true)
         RETURNING id",
    )
    .bind(format!("test_project_{}", uuid::Uuid::new_v4()))
    .fetch_one(pool)
    .await
    .expect("Failed to create test project");

    // 3. 사용자를 프로젝트 멤버로 추가
    sqlx::query(
        "INSERT INTO security_user_project (user_id, project_id)
         VALUES ($1, $2)",
    )
    .bind(user_id)
    .bind(project_id)
    .execute(pool)
    .await
    .expect("Failed to add user to project");

    // 4. Study 생성
    let study_id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO project_data_study (study_uid, study_description)
         VALUES ($1, 'Test Study')
         RETURNING id",
    )
    .bind(format!("1.2.840.113619.2.1.1.{}", uuid::Uuid::new_v4()))
    .fetch_one(pool)
    .await
    .expect("Failed to create test study");

    // 5. Series 생성
    let series_id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO project_data_series (study_id, series_uid, series_description, modality)
         VALUES ($1, $2, 'Test Series', 'CT')
         RETURNING id",
    )
    .bind(study_id)
    .bind(format!("1.2.840.113619.2.1.2.{}", uuid::Uuid::new_v4()))
    .fetch_one(pool)
    .await
    .expect("Failed to create test series");

    (user_id, project_id, series_id)
}

async fn cleanup_test_data(pool: &PgPool, user_id: i32, project_id: i32, series_id: i32) {
    sqlx::query("DELETE FROM series_user_note WHERE series_id = $1")
        .bind(series_id)
        .execute(pool)
        .await
        .ok();
    sqlx::query("DELETE FROM project_data_series WHERE id = $1")
        .bind(series_id)
        .execute(pool)
        .await
        .ok();
    sqlx::query("DELETE FROM project_data_study WHERE id IN (SELECT study_id FROM project_data_series WHERE id = $1)")
        .bind(series_id)
        .execute(pool)
        .await
        .ok();
    sqlx::query("DELETE FROM security_user_project WHERE user_id = $1 AND project_id = $2")
        .bind(user_id)
        .bind(project_id)
        .execute(pool)
        .await
        .ok();
    sqlx::query("DELETE FROM security_user WHERE id = $1")
        .bind(user_id)
        .execute(pool)
        .await
        .ok();
    sqlx::query("DELETE FROM security_project WHERE id = $1")
        .bind(project_id)
        .execute(pool)
        .await
        .ok();
}

fn setup_service(pool: PgPool) -> Arc<SeriesUserNoteServiceImpl<SeriesUserNoteRepositoryImpl, UserRepositoryImpl, ProjectRepositoryImpl, ProjectDataRepositoryImpl>> {
    let note_repo = SeriesUserNoteRepositoryImpl::new(pool.clone());
    let user_repo = UserRepositoryImpl::new(pool.clone());
    let project_repo = ProjectRepositoryImpl::new(pool.clone());
    let project_data_repo = Arc::new(ProjectDataRepositoryImpl::new(pool.clone()));

    Arc::new(SeriesUserNoteServiceImpl::new(
        note_repo,
        user_repo,
        project_repo,
        project_data_repo,
    ))
}

/// Service 테스트 1: Note 생성 성공
#[tokio::test]
#[ignore]
async fn test_create_note_success() {
    let pool = get_test_pool().await;
    let (user_id, project_id, series_id) = setup_test_data(&pool).await;
    let service = setup_service(pool.clone());

    // When: Note 생성
    let result = service
        .create_or_update_note(series_id, user_id, Some(project_id), "테스트 메모".to_string())
        .await;

    // Then: 성공 확인
    assert!(result.is_ok(), "Note creation should succeed");
    let note = result.unwrap();
    assert_eq!(note.note, "테스트 메모");

    cleanup_test_data(&pool, user_id, project_id, series_id).await;
}

/// Service 테스트 2: 존재하지 않는 사용자로 Note 생성 시 에러
#[tokio::test]
#[ignore]
async fn test_create_note_with_nonexistent_user() {
    let pool = get_test_pool().await;
    let (_user_id, project_id, series_id) = setup_test_data(&pool).await;
    let service = setup_service(pool.clone());
    let nonexistent_user_id = 999999;

    // When: 존재하지 않는 사용자로 Note 생성 시도
    let result = service
        .create_or_update_note(series_id, nonexistent_user_id, Some(project_id), "메모".to_string())
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

/// Service 테스트 3: 존재하지 않는 Series로 Note 생성 시 에러
#[tokio::test]
#[ignore]
async fn test_create_note_with_nonexistent_series() {
    let pool = get_test_pool().await;
    let (user_id, project_id, _series_id) = setup_test_data(&pool).await;
    let service = setup_service(pool.clone());
    let nonexistent_series_id = 999999;

    // When: 존재하지 않는 Series로 Note 생성 시도
    let result = service
        .create_or_update_note(nonexistent_series_id, user_id, Some(project_id), "메모".to_string())
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

/// Service 테스트 4: 프로젝트 멤버가 아닌 사용자로 Note 생성 시 에러
#[tokio::test]
#[ignore]
async fn test_create_note_with_non_member_user() {
    let pool = get_test_pool().await;
    let (user_id, project_id, series_id) = setup_test_data(&pool).await;
    let service = setup_service(pool.clone());

    // Given: 프로젝트 멤버가 아닌 사용자 생성
    let non_member_user_id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO security_user (username, email, password_hash, is_active)
         VALUES ($1, $2, 'hashed_password', true)
         RETURNING id",
    )
    .bind(format!("non_member_{}", uuid::Uuid::new_v4()))
    .bind(format!("non_member_{}@example.com", uuid::Uuid::new_v4()))
    .fetch_one(&pool)
    .await
    .unwrap();

    // When: 프로젝트 멤버가 아닌 사용자로 Note 생성 시도
    let result = service
        .create_or_update_note(series_id, non_member_user_id, Some(project_id), "메모".to_string())
        .await;

    // Then: Unauthorized 에러 반환
    assert!(result.is_err(), "Should return error");
    match result.unwrap_err() {
        ServiceError::Unauthorized(_) => {
            // 성공
        }
        other => panic!("Expected Unauthorized error, got: {:?}", other),
    }

    cleanup_test_data(&pool, user_id, project_id, series_id).await;
    sqlx::query("DELETE FROM security_user WHERE id = $1")
        .bind(non_member_user_id)
        .execute(&pool)
        .await
        .ok();
}

/// Service 테스트 5: 전역 Note 생성 (프로젝트 멤버십 검증 없음)
#[tokio::test]
#[ignore]
async fn test_create_global_note() {
    let pool = get_test_pool().await;
    let (user_id, _project_id, series_id) = setup_test_data(&pool).await;
    let service = setup_service(pool.clone());

    // When: 전역 Note 생성 (project_id = None)
    let result = service
        .create_or_update_note(series_id, user_id, None, "전역 메모".to_string())
        .await;

    // Then: 성공 확인 (프로젝트 멤버십 검증 없음)
    assert!(result.is_ok(), "Global note creation should succeed");
    let note = result.unwrap();
    assert_eq!(note.project_id, None);
    assert_eq!(note.note, "전역 메모");

    cleanup_test_data(&pool, user_id, 0, series_id).await;
}

/// Service 테스트 6: Note 조회
#[tokio::test]
#[ignore]
async fn test_get_note() {
    let pool = get_test_pool().await;
    let (user_id, project_id, series_id) = setup_test_data(&pool).await;
    let service = setup_service(pool.clone());

    // Given: Note 생성
    service
        .create_or_update_note(series_id, user_id, Some(project_id), "조회 테스트 메모".to_string())
        .await
        .unwrap();

    // When: Note 조회
    let result = service.get_note(series_id, user_id, Some(project_id)).await;

    // Then: 조회 성공 확인
    assert!(result.is_ok(), "Note retrieval should succeed");
    let note = result.unwrap();
    assert!(note.is_some(), "Note should be found");
    assert_eq!(note.unwrap().note, "조회 테스트 메모");

    cleanup_test_data(&pool, user_id, project_id, series_id).await;
}

/// Service 테스트 7: Note 삭제
#[tokio::test]
#[ignore]
async fn test_delete_note() {
    let pool = get_test_pool().await;
    let (user_id, project_id, series_id) = setup_test_data(&pool).await;
    let service = setup_service(pool.clone());

    // Given: Note 생성
    service
        .create_or_update_note(series_id, user_id, Some(project_id), "삭제될 메모".to_string())
        .await
        .unwrap();

    // When: Note 삭제
    let result = service.delete_note(series_id, user_id, Some(project_id)).await;

    // Then: 삭제 성공 확인
    assert!(result.is_ok(), "Note deletion should succeed");

    // When: 삭제 후 조회
    let found = service.get_note(series_id, user_id, Some(project_id)).await.unwrap();

    // Then: Note가 없어야 함
    assert!(found.is_none(), "Note should be deleted");

    cleanup_test_data(&pool, user_id, project_id, series_id).await;
}

/// Service 테스트 8: 존재하지 않는 Note 삭제 시 에러
#[tokio::test]
#[ignore]
async fn test_delete_nonexistent_note() {
    let pool = get_test_pool().await;
    let (user_id, project_id, series_id) = setup_test_data(&pool).await;
    let service = setup_service(pool.clone());

    // When: 존재하지 않는 Note 삭제 시도
    let result = service.delete_note(series_id, user_id, Some(project_id)).await;

    // Then: NotFound 에러 반환
    assert!(result.is_err(), "Should return error");
    match result.unwrap_err() {
        ServiceError::NotFound(_) => {
            // 성공
        }
        other => panic!("Expected NotFound error, got: {:?}", other),
    }

    cleanup_test_data(&pool, user_id, project_id, series_id).await;
}

