use pacs_server::application::dto::series_user_note_dto::CreateOrUpdateSeriesNoteRequest;
use pacs_server::application::use_cases::SeriesUserNoteUseCase;
use pacs_server::domain::services::{SeriesUserNoteService, SeriesUserNoteServiceImpl};
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
        "INSERT INTO security_user (username, email, password_hash, is_active, full_name)
         VALUES ($1, $2, 'hashed_password', true, $3)
         RETURNING id",
    )
    .bind(format!("test_user_{}", uuid::Uuid::new_v4()))
    .bind(format!("test_{}@example.com", uuid::Uuid::new_v4()))
    .bind("테스트 사용자".to_string())
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

fn setup_use_case(pool: PgPool) -> Arc<SeriesUserNoteUseCase<SeriesUserNoteServiceImpl<SeriesUserNoteRepositoryImpl, UserRepositoryImpl, ProjectRepositoryImpl, ProjectDataRepositoryImpl>, UserRepositoryImpl>> {
    let note_repo = SeriesUserNoteRepositoryImpl::new(pool.clone());
    let user_repo = UserRepositoryImpl::new(pool.clone());
    let project_repo = ProjectRepositoryImpl::new(pool.clone());
    let project_data_repo = Arc::new(ProjectDataRepositoryImpl::new(pool.clone()));

    let note_service = SeriesUserNoteServiceImpl::new(
        note_repo,
        user_repo.clone(),
        project_repo,
        project_data_repo,
    );

    Arc::new(SeriesUserNoteUseCase::new(
        Arc::new(note_service),
        Arc::new(user_repo),
    ))
}

/// Use Case 테스트 1: Note 생성 및 DTO 변환
#[tokio::test]
#[ignore]
async fn test_create_note_with_dto_conversion() {
    let pool = get_test_pool().await;
    let (user_id, project_id, series_id) = setup_test_data(&pool).await;
    let use_case = setup_use_case(pool.clone());

    // When: DTO를 사용하여 Note 생성
    let request = CreateOrUpdateSeriesNoteRequest {
        note: "Use Case 테스트 메모".to_string(),
    };

    let result = use_case
        .create_or_update_note(series_id, user_id, Some(project_id), request)
        .await;

    // Then: 성공 및 DTO 형식 확인
    assert!(result.is_ok(), "Note creation should succeed");
    let response = result.unwrap();
    assert_eq!(response.series_id, series_id);
    assert_eq!(response.user_id, user_id);
    assert_eq!(response.project_id, Some(project_id));
    assert_eq!(response.note, "Use Case 테스트 메모");

    cleanup_test_data(&pool, user_id, project_id, series_id).await;
}

/// Use Case 테스트 2: Note 조회 및 DTO 변환
#[tokio::test]
#[ignore]
async fn test_get_note_with_dto_conversion() {
    let pool = get_test_pool().await;
    let (user_id, project_id, series_id) = setup_test_data(&pool).await;
    let use_case = setup_use_case(pool.clone());

    // Given: Note 생성
    let request = CreateOrUpdateSeriesNoteRequest {
        note: "조회 테스트 메모".to_string(),
    };
    use_case
        .create_or_update_note(series_id, user_id, Some(project_id), request)
        .await
        .unwrap();

    // When: Note 조회
    let result = use_case.get_note(series_id, user_id, Some(project_id)).await;

    // Then: DTO 형식으로 반환 확인
    assert!(result.is_ok(), "Note retrieval should succeed");
    let response = result.unwrap();
    assert!(response.is_some(), "Note should be found");
    let note = response.unwrap();
    assert_eq!(note.note, "조회 테스트 메모");

    cleanup_test_data(&pool, user_id, project_id, series_id).await;
}

/// Use Case 테스트 3: Series의 모든 Note 조회 (사용자 정보 포함)
#[tokio::test]
#[ignore]
async fn test_get_all_notes_with_user_info() {
    let pool = get_test_pool().await;
    let (user_id, project_id, series_id) = setup_test_data(&pool).await;
    let use_case = setup_use_case(pool.clone());

    // Given: 여러 사용자 Note 생성
    let user2_id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO security_user (username, email, password_hash, is_active, full_name)
         VALUES ($1, $2, 'hashed_password', true, $3)
         RETURNING id",
    )
    .bind(format!("test_user2_{}", uuid::Uuid::new_v4()))
    .bind(format!("test2_{}@example.com", uuid::Uuid::new_v4()))
    .bind("두 번째 사용자".to_string())
    .fetch_one(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO security_user_project (user_id, project_id)
         VALUES ($1, $2)",
    )
    .bind(user2_id)
    .bind(project_id)
    .execute(&pool)
    .await
    .unwrap();

    use_case
        .create_or_update_note(
            series_id,
            user_id,
            Some(project_id),
            CreateOrUpdateSeriesNoteRequest {
                note: "User 1 note".to_string(),
            },
        )
        .await
        .unwrap();

    use_case
        .create_or_update_note(
            series_id,
            user2_id,
            Some(project_id),
            CreateOrUpdateSeriesNoteRequest {
                note: "User 2 note".to_string(),
            },
        )
        .await
        .unwrap();

    // When: Series의 모든 Note 조회
    let result = use_case.get_notes_by_series(series_id, Some(project_id)).await;

    // Then: 사용자 정보 포함하여 반환 확인
    assert!(result.is_ok(), "Should retrieve all notes");
    let notes = result.unwrap();
    assert_eq!(notes.len(), 2, "Should have 2 notes");
    assert_eq!(notes[0].user.id, user_id);
    assert_eq!(notes[0].user.username, format!("test_user_{}", user_id));
    assert_eq!(notes[1].user.id, user2_id);

    cleanup_test_data(&pool, user_id, project_id, series_id).await;
    sqlx::query("DELETE FROM security_user_project WHERE user_id = $1 AND project_id = $2")
        .bind(user2_id)
        .bind(project_id)
        .execute(&pool)
        .await
        .ok();
    sqlx::query("DELETE FROM security_user WHERE id = $1")
        .bind(user2_id)
        .execute(&pool)
        .await
        .ok();
}

/// Use Case 테스트 4: Note 삭제
#[tokio::test]
#[ignore]
async fn test_delete_note() {
    let pool = get_test_pool().await;
    let (user_id, project_id, series_id) = setup_test_data(&pool).await;
    let use_case = setup_use_case(pool.clone());

    // Given: Note 생성
    use_case
        .create_or_update_note(
            series_id,
            user_id,
            Some(project_id),
            CreateOrUpdateSeriesNoteRequest {
                note: "삭제될 메모".to_string(),
            },
        )
        .await
        .unwrap();

    // When: Note 삭제
    let result = use_case.delete_note(series_id, user_id, Some(project_id)).await;

    // Then: 삭제 성공 확인
    assert!(result.is_ok(), "Note deletion should succeed");

    // When: 삭제 후 조회
    let found = use_case.get_note(series_id, user_id, Some(project_id)).await.unwrap();

    // Then: Note가 없어야 함
    assert!(found.is_none(), "Note should be deleted");

    cleanup_test_data(&pool, user_id, project_id, series_id).await;
}

/// Use Case 테스트 5: 전역 Note 생성 및 조회
#[tokio::test]
#[ignore]
async fn test_global_note_operations() {
    let pool = get_test_pool().await;
    let (user_id, _project_id, series_id) = setup_test_data(&pool).await;
    let use_case = setup_use_case(pool.clone());

    // When: 전역 Note 생성
    let request = CreateOrUpdateSeriesNoteRequest {
        note: "전역 메모".to_string(),
    };

    let result = use_case
        .create_or_update_note(series_id, user_id, None, request)
        .await;

    // Then: 성공 확인
    assert!(result.is_ok(), "Global note creation should succeed");
    let note = result.unwrap();
    assert_eq!(note.project_id, None);

    // When: 전역 Note 조회
    let found = use_case.get_note(series_id, user_id, None).await.unwrap();

    // Then: 조회 성공 확인
    assert!(found.is_some(), "Global note should be found");
    assert_eq!(found.unwrap().note, "전역 메모");

    cleanup_test_data(&pool, user_id, 0, series_id).await;
}

/// Use Case 테스트 6: 프로젝트별 Note와 전역 Note 분리
#[tokio::test]
#[ignore]
async fn test_project_and_global_notes_separation() {
    let pool = get_test_pool().await;
    let (user_id, project_id, series_id) = setup_test_data(&pool).await;
    let use_case = setup_use_case(pool.clone());

    // Given: 프로젝트별 Note와 전역 Note 생성
    use_case
        .create_or_update_note(
            series_id,
            user_id,
            Some(project_id),
            CreateOrUpdateSeriesNoteRequest {
                note: "프로젝트 메모".to_string(),
            },
        )
        .await
        .unwrap();

    use_case
        .create_or_update_note(
            series_id,
            user_id,
            None,
            CreateOrUpdateSeriesNoteRequest {
                note: "전역 메모".to_string(),
            },
        )
        .await
        .unwrap();

    // When: 프로젝트별 Note 조회
    let project_note = use_case
        .get_note(series_id, user_id, Some(project_id))
        .await
        .unwrap();

    // Then: 프로젝트별 Note만 조회됨
    assert!(project_note.is_some());
    assert_eq!(project_note.unwrap().note, "프로젝트 메모");

    // When: 전역 Note 조회
    let global_note = use_case.get_note(series_id, user_id, None).await.unwrap();

    // Then: 전역 Note만 조회됨
    assert!(global_note.is_some());
    assert_eq!(global_note.unwrap().note, "전역 메모");

    cleanup_test_data(&pool, user_id, project_id, series_id).await;
}

