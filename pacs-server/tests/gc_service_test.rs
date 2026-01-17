use pacs_server::application::services::gc_service::GcService;
use pacs_server::application::services::{GcServiceImpl, ObjectStorageService};
use pacs_server::domain::entities::{Annotation, SnapshotUploadStatus, NewGcDeletionLog, GcDeletionLog};
use pacs_server::domain::repositories::{GcRepository, GcLogRepository};
use async_trait::async_trait;
use chrono::Utc;
use std::sync::{Arc, Mutex};

// ============================================================================
// Mock Implementations
// ============================================================================

#[derive(Clone)]
struct MockGcRepository {
    pending_snapshots: Arc<Mutex<Vec<Annotation>>>,
    failed_snapshots: Arc<Mutex<Vec<Annotation>>>,
}

impl MockGcRepository {
    fn new() -> Self {
        Self {
            pending_snapshots: Arc::new(Mutex::new(Vec::new())),
            failed_snapshots: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn add_pending(&self, annotation: Annotation) {
        self.pending_snapshots.lock().unwrap().push(annotation);
    }

    fn add_failed(&self, annotation: Annotation) {
        self.failed_snapshots.lock().unwrap().push(annotation);
    }
}

#[async_trait]
impl GcRepository for MockGcRepository {
    async fn find_pending_snapshots(
        &self,
        _grace_days: i32,
        _batch_size: i32,
    ) -> Result<Vec<Annotation>, sqlx::Error> {
        Ok(self.pending_snapshots.lock().unwrap().clone())
    }

    async fn find_failed_snapshots(
        &self,
        _grace_days: i32,
        _batch_size: i32,
    ) -> Result<Vec<Annotation>, sqlx::Error> {
        Ok(self.failed_snapshots.lock().unwrap().clone())
    }

    async fn update_snapshot_status(
        &self,
        _annotation_id: i32,
        _status: SnapshotUploadStatus,
    ) -> Result<(), sqlx::Error> {
        Ok(())
    }

    async fn clear_snapshot_image_key(
        &self,
        _annotation_id: i32,
    ) -> Result<(), sqlx::Error> {
        Ok(())
    }
}

#[derive(Clone)]
struct MockObjectStorage {
    should_fail: Arc<Mutex<bool>>,
    deleted_keys: Arc<Mutex<Vec<String>>>,
}

impl MockObjectStorage {
    fn new() -> Self {
        Self {
            should_fail: Arc::new(Mutex::new(false)),
            deleted_keys: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn set_should_fail(&self, fail: bool) {
        *self.should_fail.lock().unwrap() = fail;
    }

    fn get_deleted_keys(&self) -> Vec<String> {
        self.deleted_keys.lock().unwrap().clone()
    }
}

#[async_trait]
impl ObjectStorageService for MockObjectStorage {
    async fn generate_upload_url(
        &self,
        _file_path: &str,
        _options: pacs_server::application::services::SignedUrlOptions,
    ) -> Result<String, pacs_server::application::services::ObjectStorageError> {
        Ok("https://mock-upload-url.com".to_string())
    }

    async fn generate_download_url(
        &self,
        _file_path: &str,
        _ttl_seconds: u64,
    ) -> Result<String, pacs_server::application::services::ObjectStorageError> {
        Ok("https://mock-download-url.com".to_string())
    }

    async fn delete_file(
        &self,
        file_path: &str,
    ) -> Result<(), pacs_server::application::services::ObjectStorageError> {
        if *self.should_fail.lock().unwrap() {
            return Err(pacs_server::application::services::ObjectStorageError::S3Error(
                "Mock S3 delete failed".to_string(),
            ));
        }
        self.deleted_keys.lock().unwrap().push(file_path.to_string());
        Ok(())
    }

    async fn get_file_metadata(
        &self,
        _file_path: &str,
    ) -> Result<pacs_server::application::services::UploadedFile, pacs_server::application::services::ObjectStorageError> {
        Ok(pacs_server::application::services::UploadedFile {
            file_path: "mock-file.png".to_string(),
            file_size: 1024,
            checksum: None,
            mime_type: Some("image/png".to_string()),
            last_modified: None,
        })
    }

    async fn file_exists(
        &self,
        _file_path: &str,
    ) -> Result<bool, pacs_server::application::services::ObjectStorageError> {
        Ok(true)
    }

    async fn list_files(
        &self,
        _prefix: &str,
        _max_keys: Option<i32>,
    ) -> Result<Vec<String>, pacs_server::application::services::ObjectStorageError> {
        Ok(vec![])
    }

    async fn copy_file(
        &self,
        _source_path: &str,
        _destination_path: &str,
    ) -> Result<(), pacs_server::application::services::ObjectStorageError> {
        Ok(())
    }

    async fn move_file(
        &self,
        _source_path: &str,
        _destination_path: &str,
    ) -> Result<(), pacs_server::application::services::ObjectStorageError> {
        Ok(())
    }
}

#[derive(Clone)]
struct MockGcLogRepository {
    logs: Arc<Mutex<Vec<NewGcDeletionLog>>>,
}

impl MockGcLogRepository {
    fn new() -> Self {
        Self {
            logs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn get_logs(&self) -> Vec<NewGcDeletionLog> {
        self.logs.lock().unwrap().clone()
    }
}

#[async_trait]
impl GcLogRepository for MockGcLogRepository {
    async fn insert(&self, log: NewGcDeletionLog) -> Result<GcDeletionLog, sqlx::Error> {
        self.logs.lock().unwrap().push(log.clone());

        // Mock GcDeletionLog 반환
        Ok(GcDeletionLog {
            id: 1,
            annotation_id: log.annotation_id,
            snapshot_image_key: log.snapshot_image_key,
            file_size: log.file_size,
            status: log.status,
            error_message: log.error_message,
            dry_run: log.dry_run,
            deleted_at: chrono::Utc::now(),
        })
    }

    async fn find_by_date_range(
        &self,
        _start_date: chrono::DateTime<Utc>,
        _end_date: chrono::DateTime<Utc>,
    ) -> Result<Vec<GcDeletionLog>, sqlx::Error> {
        Ok(vec![])
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

fn create_test_annotation(
    id: i32,
    status: SnapshotUploadStatus,
    snapshot_key: Option<String>,
) -> Annotation {
    Annotation {
        id,
        project_id: 1,
        user_id: 1,
        study_uid: format!("study-{}", id),
        series_uid: Some(format!("series-{}", id)),
        instance_uid: Some(format!("instance-{}", id)),
        tool_name: "test-tool".to_string(),
        tool_version: Some("1.0".to_string()),
        data: serde_json::json!({}),
        is_shared: false,
        snapshot_image_key: snapshot_key,
        snapshot_status: Some(status),
        snapshot_uploaded_at: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        version: 1,
        viewer_software: None,
        description: None,
        measurement_values: None,
        label: None,
    }
}

// ============================================================================
// Tests
// ============================================================================

#[tokio::test]
async fn test_timeout_pending_snapshots_dry_run() {
    // Arrange
    let gc_repo = MockGcRepository::new();
    let object_storage = MockObjectStorage::new();
    let gc_log_repo = MockGcLogRepository::new();

    // PENDING 어노테이션 추가
    gc_repo.add_pending(create_test_annotation(
        1,
        SnapshotUploadStatus::Pending,
        Some("snapshots/1/1/test.png".to_string()),
    ));

    let service = GcServiceImpl::new(
        Arc::new(gc_repo.clone()),
        Arc::new(gc_log_repo.clone()),
        Arc::new(object_storage.clone()),
    );

    // Act
    let results = service
        .timeout_pending_snapshots(3, 100, true)
        .await
        .unwrap();

    // Assert
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].annotation_id, 1);
    assert!(results[0].success);

    // Dry-run이므로 로그만 기록되고 실제 변경은 없음
    let logs = gc_log_repo.get_logs();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].annotation_id, 1);
    assert!(logs[0].dry_run);
    assert_eq!(logs[0].status, "skipped");
}

#[tokio::test]
async fn test_timeout_pending_snapshots_actual() {
    // Arrange
    let gc_repo = MockGcRepository::new();
    let object_storage = MockObjectStorage::new();
    let gc_log_repo = MockGcLogRepository::new();

    gc_repo.add_pending(create_test_annotation(
        2,
        SnapshotUploadStatus::Pending,
        Some("snapshots/1/2/test.png".to_string()),
    ));

    let service = GcServiceImpl::new(
        Arc::new(gc_repo.clone()),
        Arc::new(gc_log_repo.clone()),
        Arc::new(object_storage.clone()),
    );

    // Act
    let results = service
        .timeout_pending_snapshots(3, 100, false)
        .await
        .unwrap();

    // Assert
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].annotation_id, 2);
    assert!(results[0].success);

    // 실제 실행이므로 success 로그 기록
    let logs = gc_log_repo.get_logs();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].annotation_id, 2);
    assert!(!logs[0].dry_run);
    assert_eq!(logs[0].status, "success");
}

#[tokio::test]
async fn test_cleanup_failed_snapshots_dry_run() {
    // Arrange
    let gc_repo = MockGcRepository::new();
    let object_storage = MockObjectStorage::new();
    let gc_log_repo = MockGcLogRepository::new();

    gc_repo.add_failed(create_test_annotation(
        3,
        SnapshotUploadStatus::Failed,
        Some("snapshots/1/3/test.png".to_string()),
    ));

    let service = GcServiceImpl::new(
        Arc::new(gc_repo.clone()),
        Arc::new(gc_log_repo.clone()),
        Arc::new(object_storage.clone()),
    );

    // Act
    let results = service
        .cleanup_failed_snapshots(7, 100, true)
        .await
        .unwrap();

    // Assert
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].annotation_id, 3);
    assert!(results[0].success);

    // Dry-run이므로 S3 삭제 안됨
    assert_eq!(object_storage.get_deleted_keys().len(), 0);

    // 로그 확인
    let logs = gc_log_repo.get_logs();
    assert_eq!(logs.len(), 1);
    assert!(logs[0].dry_run);
    assert_eq!(logs[0].status, "skipped");
}

#[tokio::test]
async fn test_cleanup_failed_snapshots_actual_success() {
    // Arrange
    let gc_repo = MockGcRepository::new();
    let object_storage = MockObjectStorage::new();
    let gc_log_repo = MockGcLogRepository::new();

    let snapshot_key = "snapshots/1/4/test.png".to_string();
    gc_repo.add_failed(create_test_annotation(
        4,
        SnapshotUploadStatus::Failed,
        Some(snapshot_key.clone()),
    ));

    let service = GcServiceImpl::new(
        Arc::new(gc_repo.clone()),
        Arc::new(gc_log_repo.clone()),
        Arc::new(object_storage.clone()),
    );

    // Act
    let results = service
        .cleanup_failed_snapshots(7, 100, false)
        .await
        .unwrap();

    // Assert
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].annotation_id, 4);
    assert!(results[0].success);

    // S3 삭제 확인
    let deleted = object_storage.get_deleted_keys();
    assert_eq!(deleted.len(), 1);
    assert_eq!(deleted[0], snapshot_key);

    // 로그 확인
    let logs = gc_log_repo.get_logs();
    assert_eq!(logs.len(), 1);
    assert!(!logs[0].dry_run);
    assert_eq!(logs[0].status, "success");
}

#[tokio::test]
async fn test_cleanup_failed_snapshots_s3_error() {
    // Arrange
    let gc_repo = MockGcRepository::new();
    let object_storage = MockObjectStorage::new();
    let gc_log_repo = MockGcLogRepository::new();

    // S3 삭제 실패 설정
    object_storage.set_should_fail(true);

    gc_repo.add_failed(create_test_annotation(
        5,
        SnapshotUploadStatus::Failed,
        Some("snapshots/1/5/test.png".to_string()),
    ));

    let service = GcServiceImpl::new(
        Arc::new(gc_repo.clone()),
        Arc::new(gc_log_repo.clone()),
        Arc::new(object_storage.clone()),
    );

    // Act
    let results = service
        .cleanup_failed_snapshots(7, 100, false)
        .await
        .unwrap();

    // Assert
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].annotation_id, 5);
    assert!(!results[0].success); // 실패
    assert!(results[0].error_message.is_some());

    // S3 삭제 안됨
    assert_eq!(object_storage.get_deleted_keys().len(), 0);

    // 실패 로그 확인
    let logs = gc_log_repo.get_logs();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].status, "failed");
    assert!(logs[0].error_message.is_some());
}

#[tokio::test]
async fn test_batch_processing() {
    // Arrange
    let gc_repo = MockGcRepository::new();
    let object_storage = MockObjectStorage::new();
    let gc_log_repo = MockGcLogRepository::new();

    // 여러 개의 PENDING 어노테이션 추가
    for i in 1..=5 {
        gc_repo.add_pending(create_test_annotation(
            i,
            SnapshotUploadStatus::Pending,
            Some(format!("snapshots/1/{}/test.png", i)),
        ));
    }

    let service = GcServiceImpl::new(
        Arc::new(gc_repo.clone()),
        Arc::new(gc_log_repo.clone()),
        Arc::new(object_storage.clone()),
    );

    // Act
    let results = service
        .timeout_pending_snapshots(3, 100, false)
        .await
        .unwrap();

    // Assert
    assert_eq!(results.len(), 5);
    assert!(results.iter().all(|r| r.success));

    // 모든 로그 확인
    let logs = gc_log_repo.get_logs();
    assert_eq!(logs.len(), 5);
    assert!(logs.iter().all(|l| l.status == "success"));
}


