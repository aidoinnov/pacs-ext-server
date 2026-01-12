# Snapshot GC 구현 가이드

> **작성일**: 2026-01-12  
> **목적**: Snapshot GC 배치 작업 구현 상세 가이드

---

## 📋 구현 개요

### 목표
- PENDING 상태 타임아웃 처리 (3일)
- FAILED 상태 Snapshot S3 삭제 (7일 grace period)
- Orphan Snapshot 정리 (DB에 없는 S3 오브젝트)

### 전제 조건
- ✅ `snapshot_upload_status` ENUM 존재
- ✅ `snapshot_uploaded_at` 필드 존재
- ✅ `snapshot_image_key` 필드 존재
- ✅ 상태 전이 로직 완벽 구현

---

## 1️⃣ 아키텍처

> **참고**: 상세한 아키텍처 설계는 [06-아키텍처-및-바이너리-구조.md](./06-아키텍처-및-바이너리-구조.md) 참조

### 디렉토리 구조

```
pacs-server/
├── Cargo.toml                        # 멀티 바이너리 설정
├── src/
│   ├── lib.rs                        # 공통 라이브러리
│   ├── bin/
│   │   ├── server.rs                 # API 서버 바이너리
│   │   └── gc_runner.rs              # GC 배치 바이너리 ⭐
│   ├── application/
│   │   └── services/
│   │       └── gc_service.rs         # GC 비즈니스 로직 (공유)
│   └── infrastructure/
│       └── repositories/
│           └── gc_repository.rs      # GC DB 쿼리 (공유)
└── migrations/
    └── 039_create_gc_deletion_log.sql
```

### 컴포넌트 다이어그램

```
┌─────────────────┐
│   gc_runner     │  (별도 바이너리)
│   (CronJob)     │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   GcService     │  (비즈니스 로직 - lib.rs에서 공유)
│                 │
│ - run_snapshot_gc()
│ - timeout_pending()
│ - cleanup_failed()
│ - cleanup_orphans()
└────────┬────────┘
         │
         ├──────────────┬──────────────┐
         ▼              ▼              ▼
┌─────────────┐  ┌─────────────┐  ┌─────────────┐
│GcRepository │  │S3Service    │  │LogRepository│
│             │  │             │  │             │
│- find_*()   │  │- delete()   │  │- insert()   │
└─────────────┘  └─────────────┘  └─────────────┘
```

### 바이너리 분리 전략

- **API 서버** (`pacs-server`): Axum, Tower 등 웹 프레임워크 포함
- **GC 배치** (`gc_runner`): Clap만 포함, 더 작은 바이너리
- **공통 코드**: `lib.rs`로 공유 (services, repositories, models)

---

## 2️⃣ Job A: PENDING 타임아웃

### 목적
- 3일 이상 `pending` 상태인 Snapshot을 `failed`로 전환
- 클라이언트가 업로드를 완료하지 않은 경우 처리

### SQL 쿼리

```sql
-- pending 상태이고 3일 이상 지난 Annotation 조회
SELECT 
    id,
    snapshot_image_key,
    snapshot_upload_status,
    snapshot_uploaded_at,
    created_at,
    updated_at
FROM annotations
WHERE snapshot_upload_status = 'pending'
  AND updated_at < NOW() - INTERVAL '3 days'
ORDER BY updated_at ASC
LIMIT $1;  -- batch_size
```

### 상태 업데이트

```sql
-- pending → failed 전환
UPDATE annotations
SET 
    snapshot_upload_status = 'failed',
    updated_at = NOW()
WHERE id = $1
  AND snapshot_upload_status = 'pending';
```

### Rust 구현 예시

```rust
pub async fn timeout_pending_snapshots(
    &self,
    grace_period_days: i32,
    batch_size: i32,
    dry_run: bool,
) -> Result<Vec<GcResult>, ServiceError> {
    let batch_id = Uuid::new_v4().to_string();
    let mut results = Vec::new();

    // 1. 타임아웃 대상 조회
    let annotations = self.gc_repository
        .find_pending_snapshots(grace_period_days, batch_size)
        .await?;

    for annotation in annotations {
        let result = if dry_run {
            // Dry-run: 로그만 기록
            GcResult {
                batch_id: batch_id.clone(),
                resource_type: "snapshot".to_string(),
                s3_key: annotation.snapshot_image_key.clone(),
                annotation_id: Some(annotation.id),
                reason: "pending_timeout".to_string(),
                dry_run: true,
                status: "skipped".to_string(),
                grace_period_days: Some(grace_period_days),
                ..Default::default()
            }
        } else {
            // 실제 실행: 상태 업데이트
            match self.gc_repository
                .update_snapshot_status(annotation.id, SnapshotUploadStatus::Failed)
                .await
            {
                Ok(_) => GcResult {
                    batch_id: batch_id.clone(),
                    resource_type: "snapshot".to_string(),
                    s3_key: annotation.snapshot_image_key.clone(),
                    annotation_id: Some(annotation.id),
                    reason: "pending_timeout".to_string(),
                    dry_run: false,
                    status: "success".to_string(),
                    grace_period_days: Some(grace_period_days),
                    ..Default::default()
                },
                Err(e) => GcResult {
                    batch_id: batch_id.clone(),
                    resource_type: "snapshot".to_string(),
                    s3_key: annotation.snapshot_image_key.clone(),
                    annotation_id: Some(annotation.id),
                    reason: "pending_timeout".to_string(),
                    dry_run: false,
                    status: "failed".to_string(),
                    error_message: Some(e.to_string()),
                    ..Default::default()
                },
            }
        };

        // 2. 로그 기록
        self.log_repository.insert(result.clone()).await?;
        results.push(result);
    }

    Ok(results)
}
```

---

## 3️⃣ Job B: FAILED Snapshot 정리

### 목적
- 7일 이상 `failed` 상태인 Snapshot의 S3 오브젝트 삭제
- DB 레코드는 유지 (감사 추적)

### SQL 쿼리

```sql
-- failed 상태이고 7일 이상 지난 Snapshot 조회
SELECT 
    id,
    snapshot_image_key,
    snapshot_upload_status,
    snapshot_uploaded_at,
    updated_at
FROM annotations
WHERE snapshot_upload_status = 'failed'
  AND snapshot_image_key IS NOT NULL
  AND updated_at < NOW() - INTERVAL '7 days'
ORDER BY updated_at ASC
LIMIT $1;  -- batch_size
```

### S3 삭제

```rust
// 배치 삭제 (최대 1000개)
let delete_objects: Vec<ObjectIdentifier> = snapshots
    .iter()
    .map(|s| ObjectIdentifier::builder()
        .key(&s.snapshot_image_key)
        .build()
        .unwrap())
    .collect();

let delete_request = Delete::builder()
    .set_objects(Some(delete_objects))
    .build()
    .unwrap();

self.s3_client
    .delete_objects()
    .bucket(&self.bucket_name)
    .delete(delete_request)
    .send()
    .await?;
```

### Rust 구현 예시

```rust
pub async fn cleanup_failed_snapshots(
    &self,
    grace_period_days: i32,
    batch_size: i32,
    dry_run: bool,
) -> Result<Vec<GcResult>, ServiceError> {
    let batch_id = Uuid::new_v4().to_string();
    let mut results = Vec::new();

    // 1. 삭제 대상 조회
    let snapshots = self.gc_repository
        .find_failed_snapshots(grace_period_days, batch_size)
        .await?;

    if snapshots.is_empty() {
        return Ok(results);
    }

    // 2. S3 배치 삭제 (dry_run이 아닐 때만)
    if !dry_run {
        let delete_objects: Vec<ObjectIdentifier> = snapshots
            .iter()
            .map(|s| ObjectIdentifier::builder()
                .key(&s.snapshot_image_key)
                .build()
                .unwrap())
            .collect();

        let delete_request = Delete::builder()
            .set_objects(Some(delete_objects))
            .build()
            .unwrap();

        match self.s3_service
            .delete_objects(delete_request)
            .await
        {
            Ok(response) => {
                // 성공/실패 개별 처리
                for (idx, snapshot) in snapshots.iter().enumerate() {
                    let status = if response.deleted.as_ref()
                        .and_then(|d| d.get(idx))
                        .is_some()
                    {
                        "success"
                    } else {
                        "failed"
                    };

                    let result = GcResult {
                        batch_id: batch_id.clone(),
                        resource_type: "snapshot".to_string(),
                        s3_key: snapshot.snapshot_image_key.clone(),
                        annotation_id: Some(snapshot.id),
                        reason: "failed".to_string(),
                        dry_run: false,
                        status: status.to_string(),
                        grace_period_days: Some(grace_period_days),
                        ..Default::default()
                    };

                    self.log_repository.insert(result.clone()).await?;
                    results.push(result);
                }
            }
            Err(e) => {
                // 전체 실패
                for snapshot in snapshots {
                    let result = GcResult {
                        batch_id: batch_id.clone(),
                        resource_type: "snapshot".to_string(),
                        s3_key: snapshot.snapshot_image_key.clone(),
                        annotation_id: Some(snapshot.id),
                        reason: "failed".to_string(),
                        dry_run: false,
                        status: "failed".to_string(),
                        error_message: Some(e.to_string()),
                        ..Default::default()
                    };

                    self.log_repository.insert(result.clone()).await?;
                    results.push(result);
                }
            }
        }
    } else {
        // Dry-run: 로그만 기록
        for snapshot in snapshots {
            let result = GcResult {
                batch_id: batch_id.clone(),
                resource_type: "snapshot".to_string(),
                s3_key: snapshot.snapshot_image_key.clone(),
                annotation_id: Some(snapshot.id),
                reason: "failed".to_string(),
                dry_run: true,
                status: "skipped".to_string(),
                grace_period_days: Some(grace_period_days),
                ..Default::default()
            };

            self.log_repository.insert(result.clone()).await?;
            results.push(result);
        }
    }

    Ok(results)
}
```

---

## 4️⃣ 설정 관리

### 환경 변수

```bash
# .env
GC_DRY_RUN=true                    # Dry-run 모드
GC_BATCH_SIZE=1000                 # 배치 크기
GC_PENDING_GRACE_DAYS=3            # PENDING 유예 기간
GC_FAILED_GRACE_DAYS=7             # FAILED 유예 기간
GC_LOG_RETENTION_DAYS=365          # 로그 보존 기간
```

### Config 구조체

```rust
#[derive(Debug, Clone)]
pub struct GcConfig {
    pub dry_run: bool,
    pub batch_size: i32,
    pub pending_grace_days: i32,
    pub failed_grace_days: i32,
    pub log_retention_days: i32,
}

impl GcConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            dry_run: env::var("GC_DRY_RUN")
                .unwrap_or_else(|_| "true".to_string())
                .parse()?,
            batch_size: env::var("GC_BATCH_SIZE")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()?,
            pending_grace_days: env::var("GC_PENDING_GRACE_DAYS")
                .unwrap_or_else(|_| "3".to_string())
                .parse()?,
            failed_grace_days: env::var("GC_FAILED_GRACE_DAYS")
                .unwrap_or_else(|_| "7".to_string())
                .parse()?,
            log_retention_days: env::var("GC_LOG_RETENTION_DAYS")
                .unwrap_or_else(|_| "365".to_string())
                .parse()?,
        })
    }
}
```

---

## 5️⃣ 다음 단계

1. **Job C: Orphan Snapshot 정리** - 별도 문서 참조
2. **Mask GC 구현** - Mask 상태 관리 추가 후
3. **모니터링 & 알림** - Prometheus 메트릭 추가

