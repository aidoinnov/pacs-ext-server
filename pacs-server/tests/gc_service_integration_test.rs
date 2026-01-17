use pacs_server::application::services::{GcService, GcServiceImpl, ObjectStorageService, ObjectStorageError, SignedUrlOptions, UploadedFile};
use pacs_server::domain::entities::SnapshotUploadStatus;
use pacs_server::domain::repositories::{GcRepository, GcLogRepository};
use pacs_server::infrastructure::repositories::{GcLogRepositoryImpl, GcRepositoryImpl};
use async_trait::async_trait;
use chrono::Duration;
use sqlx::PgPool;
use std::sync::Arc;

// ============================================================================
// Test Helpers
// ============================================================================

async fn get_test_pool() -> PgPool {
    let database_url = std::env::var("APP_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .unwrap_or_else(|_| {
            "postgres://aido@localhost:5432/pacs_db".to_string()
        });

    PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

async fn setup_test_data(pool: &PgPool) -> (i32, i32) {
    // 1. 테스트 사용자 생성
    let user_id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO security_user (keycloak_id, username, email, created_at)
         VALUES ($1, $2, $3, NOW())
         RETURNING id",
    )
    .bind(format!("test-keycloak-{}", uuid::Uuid::new_v4()))
    .bind(format!("test_user_{}", uuid::Uuid::new_v4()))
    .bind(format!("test_{}@example.com", uuid::Uuid::new_v4()))
    .fetch_one(pool)
    .await
    .expect("Failed to create test user");

    // 2. 테스트 프로젝트 생성
    let project_id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO security_project (name, description, is_active, created_at)
         VALUES ($1, 'Test GC Project', true, NOW())
         RETURNING id",
    )
    .bind(format!("test_gc_project_{}", uuid::Uuid::new_v4()))
    .fetch_one(pool)
    .await
    .expect("Failed to create test project");

    (user_id, project_id)
}

async fn create_test_annotation(
    pool: &PgPool,
    user_id: i32,
    project_id: i32,
    status: SnapshotUploadStatus,
    days_ago: i64,
    snapshot_key: Option<String>,
) -> i32 {
    let created_at = Utc::now() - Duration::days(days_ago);
    let uploaded_at = if status == SnapshotUploadStatus::Completed {
        Some(created_at)
    } else {
        None
    };

    sqlx::query_scalar::<_, i32>(
        "INSERT INTO annotation_annotation (
            project_id, user_id, study_uid, series_uid, instance_uid,
            tool_name, data, is_shared, created_at, updated_at,
            snapshot_image_key, snapshot_status, snapshot_uploaded_at
         ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
         RETURNING id",
    )
    .bind(project_id)
    .bind(user_id)
    .bind(format!("test-study-{}", uuid::Uuid::new_v4()))
    .bind("test-series")
    .bind("test-instance")
    .bind("test-tool")
    .bind(serde_json::json!({}))
    .bind(false)
    .bind(created_at)
    .bind(created_at)
    .bind(snapshot_key)
    .bind(status.to_string())
    .bind(uploaded_at)
    .fetch_one(pool)
    .await
    .expect("Failed to create test annotation")
}

async fn cleanup_test_data(pool: &PgPool, user_id: i32, project_id: i32) {
    // GC 로그 삭제
    sqlx::query(
        "DELETE FROM gc_deletion_log 
         WHERE annotation_id IN (
             SELECT id FROM annotation_annotation WHERE project_id = $1
         )",
    )
    .bind(project_id)
    .execute(pool)
    .await
    .ok();

    // 어노테이션 삭제
    sqlx::query("DELETE FROM annotation_annotation WHERE project_id = $1")
        .bind(project_id)
        .execute(pool)
        .await
        .ok();

    // 프로젝트 삭제
    sqlx::query("DELETE FROM security_project WHERE id = $1")
        .bind(project_id)
        .execute(pool)
        .await
        .ok();

    // 사용자 삭제
    sqlx::query("DELETE FROM security_user WHERE id = $1")
        .bind(user_id)
        .execute(pool)
        .await
        .ok();
}

// Mock Object Storage (통합 테스트에서는 실제 S3 대신 Mock 사용)
#[derive(Clone)]
struct MockObjectStorage;

#[async_trait]
impl ObjectStorageService for MockObjectStorage {
    async fn generate_upload_url(
        &self,
        _file_path: &str,
        _options: SignedUrlOptions,
    ) -> Result<String, ObjectStorageError> {
        Ok("https://mock-upload-url.com".to_string())
    }

    async fn generate_download_url(
        &self,
        _file_path: &str,
        _ttl_seconds: u64,
    ) -> Result<String, ObjectStorageError> {
        Ok("https://mock-download-url.com".to_string())
    }

    async fn delete_file(&self, _file_path: &str) -> Result<(), ObjectStorageError> {
        // Mock: 항상 성공
        Ok(())
    }

    async fn get_file_metadata(
        &self,
        _file_path: &str,
    ) -> Result<UploadedFile, ObjectStorageError> {
        Ok(UploadedFile {
            file_path: "mock-file.png".to_string(),
            file_size: 1024,
            checksum: None,
            mime_type: Some("image/png".to_string()),
            last_modified: None,
        })
    }

    async fn file_exists(&self, _file_path: &str) -> Result<bool, ObjectStorageError> {
        Ok(true)
    }

    async fn list_files(
        &self,
        _prefix: &str,
        _max_keys: Option<i32>,
    ) -> Result<Vec<String>, ObjectStorageError> {
        Ok(vec![])
    }

    async fn copy_file(
        &self,
        _source_path: &str,
        _destination_path: &str,
    ) -> Result<(), ObjectStorageError> {
        Ok(())
    }

    async fn move_file(
        &self,
        _source_path: &str,
        _destination_path: &str,
    ) -> Result<(), ObjectStorageError> {
        Ok(())
    }
}

// ============================================================================
// Integration Tests
// ============================================================================

/// 통합 테스트 1: PENDING 타임아웃 - Dry-run
#[tokio::test]
#[ignore] // 실제 DB 필요
async fn test_timeout_pending_snapshots_dry_run_integration() {
    let pool = get_test_pool().await;
    let (user_id, project_id) = setup_test_data(&pool).await;

    // PENDING 어노테이션 생성 (4일 전)
    let annotation_id = create_test_annotation(
        &pool,
        user_id,
        project_id,
        SnapshotUploadStatus::Pending,
        4,
        Some(format!("snapshots/{}/{}/test.png", project_id, uuid::Uuid::new_v4())),
    )
    .await;

    // GC Service 초기화
    let pool_arc = Arc::new(pool.clone());
    let gc_repo = Arc::new(GcRepositoryImpl::new(pool_arc.clone()));
    let object_storage = Arc::new(MockObjectStorage);
    let gc_log_repo = Arc::new(GcLogRepositoryImpl::new(pool_arc.clone()));
    let service = GcServiceImpl::new(gc_repo.clone(), gc_log_repo, object_storage);

    // Act: Dry-run 실행
    let results = service
        .timeout_pending_snapshots(3, 100, true)
        .await
        .expect("Failed to timeout pending snapshots");

    // Assert
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].annotation_id, annotation_id);
    assert!(results[0].success);

    // Dry-run이므로 상태가 변경되지 않았는지 확인
    let status: String = sqlx::query_scalar(
        "SELECT snapshot_status FROM annotation_annotation WHERE id = $1",
    )
    .bind(annotation_id)
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch annotation status");

    assert_eq!(status, "pending");

    // GC 로그 확인
    let log_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM gc_deletion_log WHERE annotation_id = $1 AND dry_run = true",
    )
    .bind(annotation_id)
    .fetch_one(&pool)
    .await
    .expect("Failed to count logs");

    assert_eq!(log_count, 1);

    // Cleanup
    cleanup_test_data(&pool, user_id, project_id).await;
}

/// 통합 테스트 2: PENDING 타임아웃 - 실제 실행
#[tokio::test]
#[ignore] // 실제 DB 필요
async fn test_timeout_pending_snapshots_actual_integration() {
    let pool = get_test_pool().await;
    let (user_id, project_id) = setup_test_data(&pool).await;

    // PENDING 어노테이션 생성 (4일 전)
    let annotation_id = create_test_annotation(
        &pool,
        user_id,
        project_id,
        SnapshotUploadStatus::Pending,
        4,
        Some(format!("snapshots/{}/{}/test.png", project_id, uuid::Uuid::new_v4())),
    )
    .await;

    // GC Service 초기화
    let pool_arc = Arc::new(pool.clone());
    let gc_repo = Arc::new(GcRepositoryImpl::new(pool_arc.clone()));
    let object_storage = Arc::new(MockObjectStorage);
    let gc_log_repo = Arc::new(GcLogRepositoryImpl::new(pool_arc.clone()));
    let service = GcServiceImpl::new(gc_repo.clone(), gc_log_repo, object_storage);

    // Act: 실제 실행
    let results = service
        .timeout_pending_snapshots(3, 100, false)
        .await
        .expect("Failed to timeout pending snapshots");

    // Assert
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].annotation_id, annotation_id);
    assert!(results[0].success);

    // 상태가 FAILED로 변경되었는지 확인
    let status: String = sqlx::query_scalar(
        "SELECT snapshot_status FROM annotation_annotation WHERE id = $1",
    )
    .bind(annotation_id)
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch annotation status");

    assert_eq!(status, "failed");

    // GC 로그 확인
    let log_status: String = sqlx::query_scalar(
        "SELECT status FROM gc_deletion_log WHERE annotation_id = $1 AND dry_run = false",
    )
    .bind(annotation_id)
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch log status");

    assert_eq!(log_status, "success");

    // Cleanup
    cleanup_test_data(&pool, user_id, project_id).await;
}

/// 통합 테스트 3: Grace Period 검증
#[tokio::test]
#[ignore] // 실제 DB 필요
async fn test_grace_period_validation_integration() {
    let pool = get_test_pool().await;
    let (user_id, project_id) = setup_test_data(&pool).await;

    // PENDING 어노테이션 생성 (2일 전 - grace period 미만)
    let annotation_id = create_test_annotation(
        &pool,
        user_id,
        project_id,
        SnapshotUploadStatus::Pending,
        2,
        Some(format!("snapshots/{}/{}/test.png", project_id, uuid::Uuid::new_v4())),
    )
    .await;

    // GC Service 초기화
    let pool_arc = Arc::new(pool.clone());
    let gc_repo = Arc::new(GcRepositoryImpl::new(pool_arc.clone()));
    let object_storage = Arc::new(MockObjectStorage);
    let gc_log_repo = Arc::new(GcLogRepositoryImpl::new(pool_arc.clone()));
    let service = GcServiceImpl::new(gc_repo.clone(), gc_log_repo, object_storage);

    // Act: Grace period 3일로 실행
    let results = service
        .timeout_pending_snapshots(3, 100, false)
        .await
        .expect("Failed to timeout pending snapshots");

    // Assert: Grace period 미만이므로 처리되지 않음
    assert_eq!(results.len(), 0);

    // 상태가 변경되지 않았는지 확인
    let status: String = sqlx::query_scalar(
        "SELECT snapshot_status FROM annotation_annotation WHERE id = $1",
    )
    .bind(annotation_id)
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch annotation status");

    assert_eq!(status, "pending");

    // Cleanup
    cleanup_test_data(&pool, user_id, project_id).await;
}

/// 통합 테스트 4: FAILED 스냅샷 정리 - Dry-run
#[tokio::test]
#[ignore] // 실제 DB 필요
async fn test_cleanup_failed_snapshots_dry_run_integration() {
    let pool = get_test_pool().await;
    let (user_id, project_id) = setup_test_data(&pool).await;

    let snapshot_key = format!("snapshots/{}/{}/test.png", project_id, uuid::Uuid::new_v4());

    // FAILED 어노테이션 생성 (8일 전)
    let annotation_id = create_test_annotation(
        &pool,
        user_id,
        project_id,
        SnapshotUploadStatus::Failed,
        8,
        Some(snapshot_key.clone()),
    )
    .await;

    // GC Service 초기화
    let pool_arc = Arc::new(pool.clone());
    let gc_repo = Arc::new(GcRepositoryImpl::new(pool_arc.clone()));
    let object_storage = Arc::new(MockObjectStorage);
    let gc_log_repo = Arc::new(GcLogRepositoryImpl::new(pool_arc.clone()));
    let service = GcServiceImpl::new(gc_repo.clone(), gc_log_repo, object_storage);

    // Act: Dry-run 실행
    let results = service
        .cleanup_failed_snapshots(7, 100, true)
        .await
        .expect("Failed to cleanup failed snapshots");

    // Assert
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].annotation_id, annotation_id);
    assert!(results[0].success);

    // Dry-run이므로 snapshot_image_key가 변경되지 않았는지 확인
    let key: Option<String> = sqlx::query_scalar(
        "SELECT snapshot_image_key FROM annotation_annotation WHERE id = $1",
    )
    .bind(annotation_id)
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch snapshot key");

    assert_eq!(key, Some(snapshot_key));

    // Cleanup
    cleanup_test_data(&pool, user_id, project_id).await;
}

/// 통합 테스트 5: FAILED 스냅샷 정리 - 실제 실행
#[tokio::test]
#[ignore] // 실제 DB 필요
async fn test_cleanup_failed_snapshots_actual_integration() {
    let pool = get_test_pool().await;
    let (user_id, project_id) = setup_test_data(&pool).await;

    let snapshot_key = format!("snapshots/{}/{}/test.png", project_id, uuid::Uuid::new_v4());

    // FAILED 어노테이션 생성 (8일 전)
    let annotation_id = create_test_annotation(
        &pool,
        user_id,
        project_id,
        SnapshotUploadStatus::Failed,
        8,
        Some(snapshot_key.clone()),
    )
    .await;

    // GC Service 초기화
    let pool_arc = Arc::new(pool.clone());
    let gc_repo = Arc::new(GcRepositoryImpl::new(pool_arc.clone()));
    let object_storage = Arc::new(MockObjectStorage);
    let gc_log_repo = Arc::new(GcLogRepositoryImpl::new(pool_arc.clone()));
    let service = GcServiceImpl::new(gc_repo.clone(), gc_log_repo, object_storage);

    // Act: 실제 실행
    let results = service
        .cleanup_failed_snapshots(7, 100, false)
        .await
        .expect("Failed to cleanup failed snapshots");

    // Assert
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].annotation_id, annotation_id);
    assert!(results[0].success);

    // snapshot_image_key가 NULL로 변경되었는지 확인
    let key: Option<String> = sqlx::query_scalar(
        "SELECT snapshot_image_key FROM annotation_annotation WHERE id = $1",
    )
    .bind(annotation_id)
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch snapshot key");

    assert_eq!(key, None);

    // GC 로그 확인
    let log_status: String = sqlx::query_scalar(
        "SELECT status FROM gc_deletion_log WHERE annotation_id = $1 AND dry_run = false",
    )
    .bind(annotation_id)
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch log status");

    assert_eq!(log_status, "success");

    // Cleanup
    cleanup_test_data(&pool, user_id, project_id).await;
}

/// 통합 테스트 6: 배치 처리
#[tokio::test]
#[ignore] // 실제 DB 필요
async fn test_batch_processing_integration() {
    let pool = get_test_pool().await;
    let (user_id, project_id) = setup_test_data(&pool).await;

    // 여러 개의 PENDING 어노테이션 생성
    let mut annotation_ids = Vec::new();
    for i in 0..5 {
        let annotation_id = create_test_annotation(
            &pool,
            user_id,
            project_id,
            SnapshotUploadStatus::Pending,
            4 + i, // 4~8일 전
            Some(format!("snapshots/{}/{}/test-{}.png", project_id, uuid::Uuid::new_v4(), i)),
        )
        .await;
        annotation_ids.push(annotation_id);
    }

    // GC Service 초기화
    let pool_arc = Arc::new(pool.clone());
    let gc_repo = Arc::new(GcRepositoryImpl::new(pool_arc.clone()));
    let object_storage = Arc::new(MockObjectStorage);
    let gc_log_repo = Arc::new(GcLogRepositoryImpl::new(pool_arc.clone()));
    let service = GcServiceImpl::new(gc_repo.clone(), gc_log_repo, object_storage);

    // Act: 배치 처리
    let results = service
        .timeout_pending_snapshots(3, 100, false)
        .await
        .expect("Failed to timeout pending snapshots");

    // Assert: 모두 처리되었는지 확인
    assert_eq!(results.len(), 5);
    assert!(results.iter().all(|r| r.success));

    // 모든 어노테이션이 FAILED로 변경되었는지 확인
    for annotation_id in &annotation_ids {
        let status: String = sqlx::query_scalar(
            "SELECT snapshot_status FROM annotation_annotation WHERE id = $1",
        )
        .bind(annotation_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch annotation status");

        assert_eq!(status, "failed");
    }

    // GC 로그 확인
    let log_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM gc_deletion_log WHERE annotation_id = ANY($1)",
    )
    .bind(&annotation_ids)
    .fetch_one(&pool)
    .await
    .expect("Failed to count logs");

    assert_eq!(log_count, 5);

    // Cleanup
    cleanup_test_data(&pool, user_id, project_id).await;
}

/// 통합 테스트 7: 트랜잭션 롤백 (에러 발생 시)
#[tokio::test]
#[ignore] // 실제 DB 필요
async fn test_transaction_rollback_on_error_integration() {
    let pool = get_test_pool().await;
    let (user_id, project_id) = setup_test_data(&pool).await;

    // PENDING 어노테이션 생성
    let annotation_id = create_test_annotation(
        &pool,
        user_id,
        project_id,
        SnapshotUploadStatus::Pending,
        4,
        Some(format!("snapshots/{}/{}/test.png", project_id, uuid::Uuid::new_v4())),
    )
    .await;

    // 원래 상태 저장
    let original_status: String = sqlx::query_scalar(
        "SELECT snapshot_status FROM annotation_annotation WHERE id = $1",
    )
    .bind(annotation_id)
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch original status");

    // GC Service 초기화 (에러를 발생시키는 Mock Storage)
    #[derive(Clone)]
    struct FailingObjectStorage;

    #[async_trait]
    impl ObjectStorageService for FailingObjectStorage {
        async fn generate_upload_url(
            &self,
            _file_path: &str,
            _options: SignedUrlOptions,
        ) -> Result<String, ObjectStorageError> {
            Err(ObjectStorageError::S3Error("Forced error".to_string()))
        }

        async fn generate_download_url(
            &self,
            _file_path: &str,
            _ttl_seconds: u64,
        ) -> Result<String, ObjectStorageError> {
            Err(ObjectStorageError::S3Error("Forced error".to_string()))
        }

        async fn delete_file(&self, _file_path: &str) -> Result<(), ObjectStorageError> {
            Err(ObjectStorageError::S3Error("Forced error".to_string()))
        }

        async fn get_file_metadata(
            &self,
            _file_path: &str,
        ) -> Result<UploadedFile, ObjectStorageError> {
            Err(ObjectStorageError::S3Error("Forced error".to_string()))
        }

        async fn file_exists(&self, _file_path: &str) -> Result<bool, ObjectStorageError> {
            Err(ObjectStorageError::S3Error("Forced error".to_string()))
        }

        async fn list_files(
            &self,
            _prefix: &str,
            _max_keys: Option<i32>,
        ) -> Result<Vec<String>, ObjectStorageError> {
            Err(ObjectStorageError::S3Error("Forced error".to_string()))
        }

        async fn copy_file(
            &self,
            _source_path: &str,
            _destination_path: &str,
        ) -> Result<(), ObjectStorageError> {
            Err(ObjectStorageError::S3Error("Forced error".to_string()))
        }

        async fn move_file(
            &self,
            _source_path: &str,
            _destination_path: &str,
        ) -> Result<(), ObjectStorageError> {
            Err(ObjectStorageError::S3Error("Forced error".to_string()))
        }
    }

    let pool_arc = Arc::new(pool.clone());
    let gc_repo = Arc::new(GcRepositoryImpl::new(pool_arc.clone()));
    let object_storage = Arc::new(FailingObjectStorage);
    let gc_log_repo = Arc::new(GcLogRepositoryImpl::new(pool_arc.clone()));
    let service = GcServiceImpl::new(gc_repo.clone(), gc_log_repo, object_storage);

    // Act: FAILED 정리 시도 (S3 에러 발생)
    // Note: timeout_pending은 S3를 사용하지 않으므로 cleanup_failed 사용

    // 먼저 FAILED로 변경
    sqlx::query("UPDATE annotation_annotation SET snapshot_status = 'failed' WHERE id = $1")
        .bind(annotation_id)
        .execute(&pool)
        .await
        .expect("Failed to update status");

    let results = service
        .cleanup_failed_snapshots(0, 100, false)
        .await
        .expect("Failed to cleanup failed snapshots");

    // Assert: 에러가 기록되었는지 확인
    assert_eq!(results.len(), 1);
    assert!(!results[0].success);
    assert!(results[0].error_message.is_some());

    // GC 로그에 실패가 기록되었는지 확인
    let log_status: String = sqlx::query_scalar(
        "SELECT status FROM gc_deletion_log WHERE annotation_id = $1",
    )
    .bind(annotation_id)
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch log status");

    assert_eq!(log_status, "failed");

    // Cleanup
    cleanup_test_data(&pool, user_id, project_id).await;
}

/// 통합 테스트 8: COMPLETED 상태는 처리하지 않음
#[tokio::test]
#[ignore] // 실제 DB 필요
async fn test_completed_status_not_processed_integration() {
    let pool = get_test_pool().await;
    let (user_id, project_id) = setup_test_data(&pool).await;

    // COMPLETED 어노테이션 생성 (10일 전)
    let annotation_id = create_test_annotation(
        &pool,
        user_id,
        project_id,
        SnapshotUploadStatus::Completed,
        10,
        Some(format!("snapshots/{}/{}/test.png", project_id, uuid::Uuid::new_v4())),
    )
    .await;

    // GC Service 초기화
    let pool_arc = Arc::new(pool.clone());
    let gc_repo = Arc::new(GcRepositoryImpl::new(pool_arc.clone()));
    let object_storage = Arc::new(MockObjectStorage);
    let gc_log_repo = Arc::new(GcLogRepositoryImpl::new(pool_arc.clone()));
    let service = GcServiceImpl::new(gc_repo.clone(), gc_log_repo, object_storage);

    // Act: PENDING 타임아웃 실행
    let results = service
        .timeout_pending_snapshots(3, 100, false)
        .await
        .expect("Failed to timeout pending snapshots");

    // Assert: COMPLETED는 처리되지 않음
    assert_eq!(results.len(), 0);

    // 상태가 변경되지 않았는지 확인
    let status: String = sqlx::query_scalar(
        "SELECT snapshot_status FROM annotation_annotation WHERE id = $1",
    )
    .bind(annotation_id)
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch annotation status");

    assert_eq!(status, "completed");

    // Cleanup
    cleanup_test_data(&pool, user_id, project_id).await;
}



