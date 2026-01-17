use async_trait::async_trait;
use std::sync::Arc;
use crate::application::services::{GcService, GcResult, ObjectStorageService};
use crate::domain::entities::annotation::SnapshotUploadStatus;
use crate::domain::entities::gc_deletion_log::NewGcDeletionLog;
use crate::domain::repositories::{GcRepository, GcLogRepository};
use crate::domain::ServiceError;

pub struct GcServiceImpl {
    gc_repository: Arc<dyn GcRepository>,
    gc_log_repository: Arc<dyn GcLogRepository>,
    object_storage: Arc<dyn ObjectStorageService>,
}

impl GcServiceImpl {
    pub fn new(
        gc_repository: Arc<dyn GcRepository>,
        gc_log_repository: Arc<dyn GcLogRepository>,
        object_storage: Arc<dyn ObjectStorageService>,
    ) -> Self {
        Self {
            gc_repository,
            gc_log_repository,
            object_storage,
        }
    }
}

#[async_trait]
impl GcService for GcServiceImpl {
    async fn timeout_pending_snapshots(
        &self,
        grace_days: i32,
        batch_size: i32,
        dry_run: bool,
    ) -> Result<Vec<GcResult>, ServiceError> {
        // 1. PENDING 상태 조회
        let annotations = self.gc_repository
            .find_pending_snapshots(grace_days, batch_size)
            .await?;

        let mut results = Vec::new();

        // 2. 각 어노테이션 처리
        for annotation in annotations {
            let snapshot_key = annotation.snapshot_image_key
                .clone()
                .unwrap_or_default();

            if dry_run {
                // Dry-run: 로그만 기록 (best-effort)
                if let Err(log_err) = self.gc_log_repository.insert(NewGcDeletionLog {
                    annotation_id: annotation.id,
                    snapshot_image_key: snapshot_key.clone(),
                    file_size: None,
                    dry_run: true,
                    status: "skipped".to_string(),
                    error_message: None,
                }).await {
                    eprintln!("Warning: Failed to log dry-run for annotation {}: {}", annotation.id, log_err);
                }

                results.push(GcResult {
                    annotation_id: annotation.id,
                    snapshot_image_key: snapshot_key,
                    success: true,
                    error_message: None,
                });
            } else {
                // 실제 실행: 상태를 FAILED로 변경
                match self.gc_repository
                    .update_snapshot_status(annotation.id, SnapshotUploadStatus::Failed)
                    .await
                {
                    Ok(_) => {
                        // 성공 로그 (best-effort)
                        if let Err(log_err) = self.gc_log_repository.insert(NewGcDeletionLog {
                            annotation_id: annotation.id,
                            snapshot_image_key: snapshot_key.clone(),
                            file_size: None,
                            dry_run: false,
                            status: "success".to_string(),
                            error_message: None,
                        }).await {
                            eprintln!("Warning: Failed to log timeout success for annotation {}: {}", annotation.id, log_err);
                        }

                        results.push(GcResult {
                            annotation_id: annotation.id,
                            snapshot_image_key: snapshot_key,
                            success: true,
                            error_message: None,
                        });
                    }
                    Err(e) => {
                        // 실패 로그 (best-effort)
                        let error_msg = e.to_string();
                        if let Err(log_err) = self.gc_log_repository.insert(NewGcDeletionLog {
                            annotation_id: annotation.id,
                            snapshot_image_key: snapshot_key.clone(),
                            file_size: None,
                            dry_run: false,
                            status: "failed".to_string(),
                            error_message: Some(error_msg.clone()),
                        }).await {
                            eprintln!("Warning: Failed to log timeout failure for annotation {}: {}", annotation.id, log_err);
                        }

                        results.push(GcResult {
                            annotation_id: annotation.id,
                            snapshot_image_key: snapshot_key,
                            success: false,
                            error_message: Some(error_msg),
                        });
                    }
                }
            }
        }

        Ok(results)
    }

    async fn cleanup_failed_snapshots(
        &self,
        grace_days: i32,
        batch_size: i32,
        dry_run: bool,
    ) -> Result<Vec<GcResult>, ServiceError> {
        // 1. FAILED 상태 조회
        // - grace_days: FAILED 이후 일정 기간이 지난 것만 대상으로 함 (즉시 삭제 방지)
        // - batch_size: 한 번에 처리할 최대 개수 (GC 작업 부하 제어)
        let annotations = self.gc_repository
            .find_failed_snapshots(grace_days, batch_size)
            .await?;

        // GC 처리 결과를 누적해서 반환하기 위한 벡터
        // - 성공/실패 여부 및 에러 메시지를 호출자에게 전달하기 위함
        let mut results = Vec::new();

        // 2. 각 어노테이션 처리
        for annotation in annotations {
            // snapshot_image_key는 Option<String> 이므로
            // - Some(key) 인 경우만 처리
            // - None 인 경우는 이미 스냅샷이 없거나, 대상이 아니므로 스킵            
            let snapshot_key = match &annotation.snapshot_image_key {
                Some(key) => key.clone(),
                None => continue,  // snapshot_image_key가 없으면 스킵
            };

            // 3️⃣ dry_run 모드
            // - 실제 S3 삭제는 수행하지 않음
            // - 어떤 항목이 삭제 대상이 되는지만 로그로 기록
            // - 운영 전 검증 / 시뮬레이션 용도
            if dry_run {
                // Dry-run: 로그만 기록 (best-effort)
                if let Err(log_err) = self.gc_log_repository.insert(NewGcDeletionLog {
                    annotation_id: annotation.id,
                    snapshot_image_key: snapshot_key.clone(),
                    file_size: None,          // dry-run이므로 실제 파일 조회 없음
                    dry_run: true,            // dry-run 실행임을 명시
                    status: "skipped".to_string(),
                    error_message: None,
                }).await {
                    eprintln!("Warning: Failed to log dry-run for annotation {}: {}", annotation.id, log_err);
                }

                // 결과에도 성공 처리로 기록
                // - dry-run 자체는 정상 수행되었기 때문
                results.push(GcResult {
                    annotation_id: annotation.id,
                    snapshot_image_key: snapshot_key,
                    success: true,
                    error_message: None,
                });
            } else {
                // 4️⃣ 실제 실행: S3에서 삭제
                // - 먼저 파일 크기 조회 (삭제 전에 조회해야 함)
                let file_size = match self.object_storage.get_file_metadata(&snapshot_key).await {
                    Ok(metadata) => Some(metadata.file_size),
                    Err(_) => None, // 메타데이터 조회 실패 시 None (파일이 없거나 권한 문제)
                };

                // - S3(Object Storage)에서 스냅샷 파일 삭제 시도
                match self.object_storage.delete_file(&snapshot_key).await {
                    Ok(_) => {
                        // 4-1️⃣ S3 삭제 성공
                        // - DB에서 snapshot_image_key를 NULL로 업데이트
                        // - 이렇게 해야 다음 GC 실행 시 중복 처리 방지
                        match self.gc_repository.clear_snapshot_image_key(annotation.id).await {
                            Ok(_) => {
                                // DB 업데이트 성공 - 성공 로그 기록
                                // 로그 기록 실패 시에도 계속 진행 (로그는 best-effort)
                                if let Err(log_err) = self.gc_log_repository.insert(NewGcDeletionLog {
                                    annotation_id: annotation.id,
                                    snapshot_image_key: snapshot_key.clone(),
                                    file_size,
                                    dry_run: false,
                                    status: "success".to_string(),
                                    error_message: None,
                                }).await {
                                    eprintln!("Warning: Failed to log GC success for annotation {}: {}", annotation.id, log_err);
                                }

                                results.push(GcResult {
                                    annotation_id: annotation.id,
                                    snapshot_image_key: snapshot_key.clone(),
                                    success: true,
                                    error_message: None,
                                });
                            }
                            Err(db_err) => {
                                // DB 업데이트 실패 - S3는 삭제되었지만 DB는 업데이트 안됨
                                // 이 경우 다음 GC 실행 시 S3에서 파일이 없어서 에러 발생 (허용 가능)
                                let error_msg = format!("S3 deleted but DB update failed: {}", db_err);

                                // 실패 로그 기록 (best-effort)
                                if let Err(log_err) = self.gc_log_repository.insert(NewGcDeletionLog {
                                    annotation_id: annotation.id,
                                    snapshot_image_key: snapshot_key.clone(),
                                    file_size,
                                    dry_run: false,
                                    status: "partial".to_string(),  // 부분 성공 상태
                                    error_message: Some(error_msg.clone()),
                                }).await {
                                    eprintln!("Warning: Failed to log GC partial failure for annotation {}: {}", annotation.id, log_err);
                                }

                                results.push(GcResult {
                                    annotation_id: annotation.id,
                                    snapshot_image_key: snapshot_key.clone(),
                                    success: false,
                                    error_message: Some(error_msg),
                                });
                            }
                        }
                    }
                    Err(e) => {
                        // 4-2️⃣ S3 삭제 실패
                        // - 네트워크 오류, 권한 문제, 이미 삭제된 경우 등
                        let error_msg = format!("S3 delete failed: {}", e);

                        // 실패 로그를 GC 로그 테이블에 기록 (best-effort)
                        // - 재시도 또는 사후 분석을 위해 에러 메시지 저장
                        if let Err(log_err) = self.gc_log_repository.insert(NewGcDeletionLog {
                            annotation_id: annotation.id,
                            snapshot_image_key: snapshot_key.clone(),
                            file_size,
                            dry_run: false,
                            status: "failed".to_string(),
                            error_message: Some(error_msg.clone()),
                        }).await {
                            eprintln!("Warning: Failed to log GC failure for annotation {}: {}", annotation.id, log_err);
                        }

                        // 결과에도 실패로 기록
                        results.push(GcResult {
                            annotation_id: annotation.id,
                            snapshot_image_key: snapshot_key,
                            success: false,
                            error_message: Some(error_msg),
                        });
                    }
                }
            }
        }

        Ok(results)
    }

}
