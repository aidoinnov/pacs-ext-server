# Orphan Snapshot/Mask 정리 가이드

> **작성일**: 2026-01-12  
> **목적**: DB에 없는 S3 오브젝트 정리 (고급 기능)

---

## 📋 개요

### 목적
- DB에 레코드가 없는 S3 오브젝트 삭제
- 수동 삭제, 마이그레이션 오류 등으로 발생한 Orphan 리소스 정리

### 주의 사항
⚠️ **위험도 높음** - 잘못된 구현 시 정상 파일 삭제 가능
- 충분한 테스트 필요
- Dry-run 모드로 2주 이상 검증 권장
- 프로덕션 배포 전 백업 필수

---

## 1️⃣ Orphan 발생 원인

### 시나리오 1: DB 삭제 후 S3 삭제 실패
```rust
// 트랜잭션 외부에서 S3 삭제
db.delete_annotation(id).await?;  // ✅ 성공
s3.delete_object(key).await?;     // ❌ 실패 → Orphan 발생
```

### 시나리오 2: 수동 DB 삭제
```sql
-- 관리자가 직접 DB 레코드 삭제
DELETE FROM annotations WHERE id = 123;
-- S3 파일은 남아있음 → Orphan
```

### 시나리오 3: 마이그레이션 오류
```sql
-- 마이그레이션 중 일부 레코드만 삭제
DELETE FROM annotations WHERE created_at < '2020-01-01';
-- S3 파일은 삭제 안 함 → Orphan
```

---

## 2️⃣ S3 경로 구조

### Snapshot 경로
```
s3://bucket/annotations/{annotation_id}/snapshots/{timestamp}_{filename}

예시:
s3://pacs-storage/annotations/123/snapshots/20260112_143000_image.png
s3://pacs-storage/annotations/456/snapshots/20260110_120000_screenshot.jpg
```

### Mask 경로
```
s3://bucket/annotations/{annotation_id}/masks/{mask_group_id}/v{version}/{filename}

예시:
s3://pacs-storage/annotations/123/masks/1/v1/mask.png
s3://pacs-storage/annotations/123/masks/1/v2/mask.png
s3://pacs-storage/annotations/456/masks/2/v1/mask.png
```

---

## 3️⃣ Orphan 탐지 알고리즘

### 전체 흐름

```
1. S3 prefix 스캔 (annotations/)
   ↓
2. 각 오브젝트의 annotation_id 파싱
   ↓
3. DB에서 annotation_id 존재 확인
   ↓
4. 없으면 Orphan으로 분류
   ↓
5. Grace period 확인 (7일)
   ↓
6. 삭제 (또는 Dry-run 로그)
```

### Snapshot Orphan 탐지

```rust
pub async fn find_orphan_snapshots(
    &self,
    grace_period_days: i32,
    batch_size: i32,
) -> Result<Vec<OrphanResource>, ServiceError> {
    let mut orphans = Vec::new();
    let grace_cutoff = Utc::now() - Duration::days(grace_period_days as i64);

    // 1. S3 prefix 스캔
    let prefix = "annotations/";
    let mut continuation_token: Option<String> = None;

    loop {
        let mut request = self.s3_client
            .list_objects_v2()
            .bucket(&self.bucket_name)
            .prefix(prefix)
            .max_keys(1000);

        if let Some(token) = continuation_token {
            request = request.continuation_token(token);
        }

        let response = request.send().await?;

        // 2. 각 오브젝트 처리
        if let Some(contents) = response.contents {
            for object in contents {
                let key = object.key.unwrap_or_default();
                
                // Snapshot 경로만 필터링
                if !key.contains("/snapshots/") {
                    continue;
                }

                // 3. annotation_id 파싱
                let annotation_id = match parse_annotation_id_from_snapshot(&key) {
                    Some(id) => id,
                    None => {
                        warn!("Failed to parse annotation_id from key: {}", key);
                        continue;
                    }
                };

                // 4. DB 존재 확인
                let exists = self.annotation_repository
                    .exists(annotation_id)
                    .await?;

                if !exists {
                    // 5. Grace period 확인
                    let last_modified = object.last_modified.unwrap();
                    if last_modified < grace_cutoff {
                        orphans.push(OrphanResource {
                            s3_key: key,
                            annotation_id: Some(annotation_id),
                            last_modified,
                            size: object.size.unwrap_or(0),
                        });

                        // 배치 크기 제한
                        if orphans.len() >= batch_size as usize {
                            return Ok(orphans);
                        }
                    }
                }
            }
        }

        // 다음 페이지
        continuation_token = response.next_continuation_token;
        if continuation_token.is_none() {
            break;
        }
    }

    Ok(orphans)
}

// annotation_id 파싱 헬퍼
fn parse_annotation_id_from_snapshot(key: &str) -> Option<i32> {
    // "annotations/123/snapshots/20260112_143000_image.png"
    // → 123
    let parts: Vec<&str> = key.split('/').collect();
    if parts.len() >= 2 && parts[0] == "annotations" {
        parts[1].parse::<i32>().ok()
    } else {
        None
    }
}
```

### Mask Orphan 탐지

```rust
pub async fn find_orphan_masks(
    &self,
    grace_period_days: i32,
    batch_size: i32,
) -> Result<Vec<OrphanResource>, ServiceError> {
    let mut orphans = Vec::new();
    let grace_cutoff = Utc::now() - Duration::days(grace_period_days as i64);

    // 1. S3 prefix 스캔
    let prefix = "annotations/";
    let mut continuation_token: Option<String> = None;

    loop {
        let mut request = self.s3_client
            .list_objects_v2()
            .bucket(&self.bucket_name)
            .prefix(prefix)
            .max_keys(1000);

        if let Some(token) = continuation_token {
            request = request.continuation_token(token);
        }

        let response = request.send().await?;

        // 2. 각 오브젝트 처리
        if let Some(contents) = response.contents {
            for object in contents {
                let key = object.key.unwrap_or_default();
                
                // Mask 경로만 필터링
                if !key.contains("/masks/") {
                    continue;
                }

                // 3. annotation_id, mask_group_id, version 파싱
                let (annotation_id, mask_group_id, version) = 
                    match parse_mask_path(&key) {
                        Some(parsed) => parsed,
                        None => {
                            warn!("Failed to parse mask path: {}", key);
                            continue;
                        }
                    };

                // 4. DB 존재 확인
                let exists = self.mask_repository
                    .exists(annotation_id, mask_group_id, version)
                    .await?;

                if !exists {
                    // 5. Grace period 확인
                    let last_modified = object.last_modified.unwrap();
                    if last_modified < grace_cutoff {
                        orphans.push(OrphanResource {
                            s3_key: key,
                            annotation_id: Some(annotation_id),
                            mask_id: None,
                            mask_group_id: Some(mask_group_id),
                            version: Some(version),
                            last_modified,
                            size: object.size.unwrap_or(0),
                        });

                        // 배치 크기 제한
                        if orphans.len() >= batch_size as usize {
                            return Ok(orphans);
                        }
                    }
                }
            }
        }

        // 다음 페이지
        continuation_token = response.next_continuation_token;
        if continuation_token.is_none() {
            break;
        }
    }

    Ok(orphans)
}

// Mask 경로 파싱 헬퍼
fn parse_mask_path(key: &str) -> Option<(i32, i64, i32)> {
    // "annotations/123/masks/1/v2/mask.png"
    // → (annotation_id: 123, mask_group_id: 1, version: 2)
    let parts: Vec<&str> = key.split('/').collect();
    if parts.len() >= 5 
        && parts[0] == "annotations" 
        && parts[2] == "masks" 
        && parts[4].starts_with('v') 
    {
        let annotation_id = parts[1].parse::<i32>().ok()?;
        let mask_group_id = parts[3].parse::<i64>().ok()?;
        let version = parts[4][1..].parse::<i32>().ok()?;  // "v2" → 2
        Some((annotation_id, mask_group_id, version))
    } else {
        None
    }
}
```

---

## 4️⃣ 성능 최적화

### 문제: S3 스캔 느림
- 수백만 개 오브젝트 스캔 시 수 시간 소요 가능

### 해결 1: 병렬 처리

```rust
use tokio::task::JoinSet;

pub async fn find_orphans_parallel(
    &self,
    grace_period_days: i32,
    batch_size: i32,
) -> Result<Vec<OrphanResource>, ServiceError> {
    let mut tasks = JoinSet::new();
    
    // annotation_id 범위별로 병렬 스캔
    // 예: 0-999, 1000-1999, 2000-2999, ...
    for range_start in (0..10000).step_by(1000) {
        let range_end = range_start + 999;
        let prefix = format!("annotations/{}/", range_start);
        
        tasks.spawn(async move {
            self.scan_prefix_range(prefix, range_end, grace_period_days).await
        });
    }
    
    // 결과 수집
    let mut all_orphans = Vec::new();
    while let Some(result) = tasks.join_next().await {
        let orphans = result??;
        all_orphans.extend(orphans);
        
        if all_orphans.len() >= batch_size as usize {
            break;
        }
    }
    
    Ok(all_orphans)
}
```

### 해결 2: 증분 스캔

```rust
// 마지막 스캔 위치 저장
CREATE TABLE gc_scan_checkpoint (
    id SERIAL PRIMARY KEY,
    resource_type VARCHAR(50) NOT NULL,
    last_scanned_key VARCHAR(512) NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

// 이전 위치부터 재개
let checkpoint = self.get_last_checkpoint("snapshot").await?;
let request = self.s3_client
    .list_objects_v2()
    .bucket(&self.bucket_name)
    .prefix("annotations/")
    .start_after(checkpoint.last_scanned_key);  // ⭐ 이전 위치부터
```

---

## 5️⃣ 안전 장치

### 1. Whitelist (삭제 제외 목록)

```rust
// 특정 annotation_id는 절대 삭제 안 함
const WHITELIST_ANNOTATION_IDS: &[i32] = &[1, 2, 3];  // 테스트 데이터 등

fn is_whitelisted(annotation_id: i32) -> bool {
    WHITELIST_ANNOTATION_IDS.contains(&annotation_id)
}
```

### 2. 삭제 전 재확인

```rust
// S3 스캔 후 삭제 직전에 DB 재확인
for orphan in orphans {
    // 삭제 직전 재확인 (Race condition 방지)
    let exists = self.annotation_repository
        .exists(orphan.annotation_id)
        .await?;
    
    if exists {
        warn!("Annotation {} was created after scan, skipping", orphan.annotation_id);
        continue;
    }
    
    // 삭제 진행
    self.s3_service.delete_object(&orphan.s3_key).await?;
}
```

### 3. 삭제 속도 제한

```rust
use tokio::time::{sleep, Duration};

// 초당 최대 100개 삭제
for orphan in orphans {
    self.s3_service.delete_object(&orphan.s3_key).await?;
    sleep(Duration::from_millis(10)).await;  // 10ms 대기
}
```

---

## 6️⃣ 배포 계획

### Phase 1: Dry-run (2주)
```bash
GC_ORPHAN_ENABLED=true
GC_ORPHAN_DRY_RUN=true
GC_ORPHAN_GRACE_DAYS=7
```

### Phase 2: 제한적 실행 (1주)
```bash
GC_ORPHAN_ENABLED=true
GC_ORPHAN_DRY_RUN=false
GC_ORPHAN_BATCH_SIZE=100  # ⚠️ 소량만
```

### Phase 3: 전체 활성화
```bash
GC_ORPHAN_BATCH_SIZE=1000
```

---

## 📊 체크리스트

- [ ] S3 경로 파싱 로직 테스트
- [ ] DB 존재 확인 쿼리 최적화
- [ ] Dry-run 2주 검증
- [ ] 오탐 0건 확인
- [ ] 성능 테스트 (대량 오브젝트)
- [ ] Whitelist 설정
- [ ] 삭제 속도 제한 설정
- [ ] 프로덕션 배포

