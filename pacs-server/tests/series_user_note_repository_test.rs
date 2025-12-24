use pacs_server::domain::entities::SeriesUserNote;
use pacs_server::domain::repositories::SeriesUserNoteRepository;
use pacs_server::infrastructure::repositories::SeriesUserNoteRepositoryImpl;
use sqlx::PgPool;

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

    // 3. Study 생성
    let study_id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO project_data_study (study_uid, study_description)
         VALUES ($1, 'Test Study')
         RETURNING id",
    )
    .bind(format!("1.2.840.113619.2.1.1.{}", uuid::Uuid::new_v4()))
    .fetch_one(pool)
    .await
    .expect("Failed to create test study");

    // 4. Series 생성
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

/// Repository 테스트 1: Note 생성 및 조회
#[tokio::test]
#[ignore] // 실제 DB 필요
async fn test_create_and_find_note() {
    let pool = get_test_pool().await;
    let (user_id, project_id, series_id) = setup_test_data(&pool).await;
    let repo = SeriesUserNoteRepositoryImpl::new(pool.clone());

    // When: Note 생성
    let note_text = "테스트 메모입니다";
    let result = repo
        .create_or_update(series_id, user_id, Some(project_id), note_text.to_string())
        .await;

    // Then: 성공 확인
    assert!(result.is_ok(), "Note creation should succeed");
    let note = result.unwrap();
    assert_eq!(note.series_id, series_id);
    assert_eq!(note.user_id, user_id);
    assert_eq!(note.project_id, Some(project_id));
    assert_eq!(note.note, note_text);

    // When: Note 조회
    let found = repo
        .find_by_series_user_project(series_id, user_id, Some(project_id))
        .await;

    // Then: 조회 성공 확인
    assert!(found.is_ok(), "Note retrieval should succeed");
    let found_note = found.unwrap();
    assert!(found_note.is_some(), "Note should be found");
    let found_note = found_note.unwrap();
    assert_eq!(found_note.note, note_text);

    cleanup_test_data(&pool, user_id, project_id, series_id).await;
}

/// Repository 테스트 2: 전역 Note 생성 및 조회
#[tokio::test]
#[ignore]
async fn test_create_and_find_global_note() {
    let pool = get_test_pool().await;
    let (user_id, _project_id, series_id) = setup_test_data(&pool).await;
    let repo = SeriesUserNoteRepositoryImpl::new(pool.clone());

    // When: 전역 Note 생성 (project_id = None)
    let note_text = "전역 메모입니다";
    let result = repo
        .create_or_update(series_id, user_id, None, note_text.to_string())
        .await;

    // Then: 성공 확인
    assert!(result.is_ok(), "Global note creation should succeed");
    let note = result.unwrap();
    assert_eq!(note.project_id, None);

    // When: 전역 Note 조회
    let found = repo
        .find_by_series_user_project(series_id, user_id, None)
        .await;

    // Then: 조회 성공 확인
    assert!(found.is_ok(), "Global note retrieval should succeed");
    let found_note = found.unwrap();
    assert!(found_note.is_some(), "Global note should be found");
    assert_eq!(found_note.unwrap().note, note_text);

    cleanup_test_data(&pool, user_id, 0, series_id).await;
}

/// Repository 테스트 3: Note 업데이트 (UPSERT)
#[tokio::test]
#[ignore]
async fn test_update_note() {
    let pool = get_test_pool().await;
    let (user_id, project_id, series_id) = setup_test_data(&pool).await;
    let repo = SeriesUserNoteRepositoryImpl::new(pool.clone());

    // Given: 초기 Note 생성
    let initial_note = "초기 메모";
    repo.create_or_update(series_id, user_id, Some(project_id), initial_note.to_string())
        .await
        .unwrap();

    // When: 같은 키로 Note 업데이트
    let updated_note = "업데이트된 메모";
    let result = repo
        .create_or_update(series_id, user_id, Some(project_id), updated_note.to_string())
        .await;

    // Then: 업데이트 성공 확인
    assert!(result.is_ok(), "Note update should succeed");
    let note = result.unwrap();
    assert_eq!(note.note, updated_note);
    assert_ne!(note.created_at, note.updated_at, "updated_at should be different");

    // When: 조회하여 업데이트 확인
    let found = repo
        .find_by_series_user_project(series_id, user_id, Some(project_id))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(found.note, updated_note);

    cleanup_test_data(&pool, user_id, project_id, series_id).await;
}

/// Repository 테스트 4: Series의 모든 Note 조회
#[tokio::test]
#[ignore]
async fn test_find_all_notes_by_series() {
    let pool = get_test_pool().await;
    let (user_id, project_id, series_id) = setup_test_data(&pool).await;
    let repo = SeriesUserNoteRepositoryImpl::new(pool.clone());

    // Given: 여러 사용자 Note 생성
    let user2_id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO security_user (username, email, password_hash, is_active)
         VALUES ($1, $2, 'hashed_password', true)
         RETURNING id",
    )
    .bind(format!("test_user2_{}", uuid::Uuid::new_v4()))
    .bind(format!("test2_{}@example.com", uuid::Uuid::new_v4()))
    .fetch_one(&pool)
    .await
    .unwrap();

    repo.create_or_update(series_id, user_id, Some(project_id), "User 1 note".to_string())
        .await
        .unwrap();
    repo.create_or_update(series_id, user2_id, Some(project_id), "User 2 note".to_string())
        .await
        .unwrap();

    // When: Series의 모든 Note 조회
    let result = repo.find_by_series(series_id, Some(project_id)).await;

    // Then: 2개의 Note 조회 확인
    assert!(result.is_ok(), "Should retrieve all notes");
    let notes = result.unwrap();
    assert_eq!(notes.len(), 2, "Should have 2 notes");

    cleanup_test_data(&pool, user_id, project_id, series_id).await;
    sqlx::query("DELETE FROM security_user WHERE id = $1")
        .bind(user2_id)
        .execute(&pool)
        .await
        .ok();
}

/// Repository 테스트 5: Note 삭제
#[tokio::test]
#[ignore]
async fn test_delete_note() {
    let pool = get_test_pool().await;
    let (user_id, project_id, series_id) = setup_test_data(&pool).await;
    let repo = SeriesUserNoteRepositoryImpl::new(pool.clone());

    // Given: Note 생성
    repo.create_or_update(series_id, user_id, Some(project_id), "삭제될 메모".to_string())
        .await
        .unwrap();

    // When: Note 삭제
    let result = repo.delete(series_id, user_id, Some(project_id)).await;

    // Then: 삭제 성공 확인
    assert!(result.is_ok(), "Note deletion should succeed");
    assert_eq!(result.unwrap(), true, "Should return true when note is deleted");

    // When: 삭제 후 조회
    let found = repo
        .find_by_series_user_project(series_id, user_id, Some(project_id))
        .await
        .unwrap();

    // Then: Note가 없어야 함
    assert!(found.is_none(), "Note should be deleted");

    cleanup_test_data(&pool, user_id, project_id, series_id).await;
}

/// Repository 테스트 6: 존재하지 않는 Note 삭제
#[tokio::test]
#[ignore]
async fn test_delete_nonexistent_note() {
    let pool = get_test_pool().await;
    let (user_id, project_id, series_id) = setup_test_data(&pool).await;
    let repo = SeriesUserNoteRepositoryImpl::new(pool.clone());

    // When: 존재하지 않는 Note 삭제 시도
    let result = repo.delete(series_id, user_id, Some(project_id)).await;

    // Then: false 반환 (Note가 없음)
    assert!(result.is_ok(), "Delete should not error");
    assert_eq!(result.unwrap(), false, "Should return false when note doesn't exist");

    cleanup_test_data(&pool, user_id, project_id, series_id).await;
}

/// Repository 테스트 7: 프로젝트별 Note와 전역 Note 분리
#[tokio::test]
#[ignore]
async fn test_project_and_global_notes_separation() {
    let pool = get_test_pool().await;
    let (user_id, project_id, series_id) = setup_test_data(&pool).await;
    let repo = SeriesUserNoteRepositoryImpl::new(pool.clone());

    // Given: 프로젝트별 Note와 전역 Note 생성
    repo.create_or_update(series_id, user_id, Some(project_id), "프로젝트 메모".to_string())
        .await
        .unwrap();
    repo.create_or_update(series_id, user_id, None, "전역 메모".to_string())
        .await
        .unwrap();

    // When: 프로젝트별 Note 조회
    let project_note = repo
        .find_by_series_user_project(series_id, user_id, Some(project_id))
        .await
        .unwrap();

    // Then: 프로젝트별 Note만 조회됨
    assert!(project_note.is_some());
    assert_eq!(project_note.unwrap().note, "프로젝트 메모");

    // When: 전역 Note 조회
    let global_note = repo
        .find_by_series_user_project(series_id, user_id, None)
        .await
        .unwrap();

    // Then: 전역 Note만 조회됨
    assert!(global_note.is_some());
    assert_eq!(global_note.unwrap().note, "전역 메모");

    cleanup_test_data(&pool, user_id, project_id, series_id).await;
}

