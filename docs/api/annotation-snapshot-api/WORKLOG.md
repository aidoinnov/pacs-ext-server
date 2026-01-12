# Annotation Snapshot API 구현 작업 로그

> **작업 시작일**: 2026-01-11
> **목적**: DICOM Viewer 어노테이션 스냅샷 이미지를 S3에 저장하는 기능 구현
> **참고 문서**: [ARCHITECTURE.md](./ARCHITECTURE.md)

---

## 📋 작업 개요

DICOM Viewer에서 생성된 어노테이션 화면을 캡쳐하여 S3에 저장하고, DB에는 경로만 저장하는 기능을 구현합니다.

**핵심 기능**:
- ✅ Presigned URL 기반 직접 업로드 (서버 부하 최소화)
- ✅ 어노테이션과 스냅샷 이미지 연결
- ✅ 권한 기반 접근 제어
- ✅ 업로드 완료 후 DB 업데이트

---

## 🎯 구현 단계

### **1단계: DB 마이그레이션** 📊

#### 1.1 마이그레이션 파일 확인

파일: `pacs-server/migrations/036_add_snapshot_image_to_annotations.sql`

**추가되는 컬럼**:
- `snapshot_image_key`: S3 object key (VARCHAR 512)
- `snapshot_status`: 업로드 상태 (ENUM: pending/uploading/completed/failed)
- `snapshot_uploaded_at`: 업로드 완료 시간 (TIMESTAMP)

```sql
-- Migration: Add snapshot image support to annotations
-- Created: 2026-01-11
-- Description: S3에 저장된 어노테이션 스냅샷 이미지 경로 및 상태를 저장하기 위한 컬럼 추가

-- 스냅샷 상태 ENUM 타입 생성
CREATE TYPE snapshot_upload_status AS ENUM (
    'pending',      -- URL 생성됨, 업로드 대기 중
    'uploading',    -- 업로드 진행 중
    'completed',    -- 업로드 완료
    'failed'        -- 업로드 실패
);

-- 스냅샷 이미지 키 컬럼 추가
ALTER TABLE annotation_annotation
ADD COLUMN snapshot_image_key VARCHAR(512) NULL,
ADD COLUMN snapshot_status snapshot_upload_status NULL DEFAULT NULL,
ADD COLUMN snapshot_uploaded_at TIMESTAMP NULL;

-- 컬럼 주석 추가
COMMENT ON COLUMN annotation_annotation.snapshot_image_key IS 'S3에 저장된 스냅샷 이미지의 object key';
COMMENT ON COLUMN annotation_annotation.snapshot_status IS '스냅샷 업로드 상태 (pending/uploading/completed/failed)';
COMMENT ON COLUMN annotation_annotation.snapshot_uploaded_at IS '스냅샷 업로드 완료 시간';

-- 인덱스 추가 (이미지가 있는 어노테이션 조회 최적화)
CREATE INDEX idx_annotation_snapshot_image_key
ON annotation_annotation(snapshot_image_key)
WHERE snapshot_image_key IS NOT NULL;

-- 인덱스 추가 (업로드 상태별 조회 최적화)
CREATE INDEX idx_annotation_snapshot_status
ON annotation_annotation(snapshot_status)
WHERE snapshot_status IS NOT NULL;
```

**상태 전이 다이어그램**:
```
NULL → pending → uploading → completed
                     ↓
                  failed
```

#### 1.2 마이그레이션 적용

**방법 1: SQLx CLI 사용 (권장)**
```bash
cd pacs-server
sqlx migrate run
```

**방법 2: 직접 PostgreSQL 연결**
```bash
# DB 터널 확인 (start-all.sh로 이미 실행 중)
psql -h localhost -p 5432 -U admin -d pacs_db

# 마이그레이션 실행
\i migrations/036_add_snapshot_image_to_annotations.sql

# 확인
\d annotation_annotation

# 종료
\q
```

**방법 3: Python 스크립트**
```bash
cd pacs-server
python3 << 'EOF'
import psycopg2
import os

conn = psycopg2.connect(
    host="localhost",
    port=5432,
    database="pacs_db",
    user="admin",
    password=os.getenv("DB_PASSWORD", "admin123")
)

with open("migrations/036_add_snapshot_image_to_annotations.sql", "r") as f:
    sql = f.read()

cursor = conn.cursor()
cursor.execute(sql)
conn.commit()

print("✅ Migration applied successfully!")

cursor.execute("""
    SELECT column_name, data_type, is_nullable
    FROM information_schema.columns
    WHERE table_name = 'annotation_annotation'
    AND column_name = 'snapshot_image_key'
""")
print(f"Column added: {cursor.fetchone()}")

cursor.close()
conn.close()
EOF
```

#### 1.3 적용 확인
```bash
psql -h localhost -p 5432 -U admin -d pacs_db -c "
SELECT column_name, data_type, character_maximum_length, is_nullable
FROM information_schema.columns
WHERE table_name = 'annotation_annotation'
AND column_name IN ('snapshot_image_key', 'snapshot_status', 'snapshot_uploaded_at')
ORDER BY column_name;
"
```

**예상 결과**:
```
     column_name      |          data_type          | character_maximum_length | is_nullable
----------------------+-----------------------------+--------------------------+-------------
 snapshot_image_key   | character varying           |                      512 | YES
 snapshot_status      | USER-DEFINED                |                          | YES
 snapshot_uploaded_at | timestamp without time zone |                          | YES
```

**ENUM 타입 확인**:
```bash
psql -h localhost -p 5432 -U admin -d pacs_db -c "
SELECT enumlabel FROM pg_enum
WHERE enumtypid = 'snapshot_upload_status'::regtype
ORDER BY enumsortorder;
"
```

**예상 결과**:
```
 enumlabel
-----------
 pending
 uploading
 completed
 failed
```

---

### **2단계: Domain Layer** 🏗️

#### 2.1 Entity 수정

파일: `pacs-server/src/domain/entities/annotation.rs`

**⚠️ 현재 프로젝트 구조**: `UpdateAnnotation` entity는 없고, repository에서 개별 파라미터로 업데이트 처리

기존 구조체에 스냅샷 관련 필드 추가:

```rust
// ✅ 이미 추가됨 (88-90줄)
pub struct Annotation {
    // ... 기존 필드들 ...
    pub snapshot_image_key: Option<String>,
    pub snapshot_status: Option<SnapshotUploadStatus>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ✅ 이미 추가됨 (220-222줄)
pub struct NewAnnotation {
    // ... 기존 필드들 ...
    pub snapshot_image_key: Option<String>,
    pub snapshot_status: Option<SnapshotUploadStatus>,
}
```

**추가 필요**: `snapshot_uploaded_at` 필드

```rust
// Annotation 구조체에 추가
pub struct Annotation {
    // ... 기존 필드들 ...
    pub snapshot_image_key: Option<String>,
    pub snapshot_status: Option<SnapshotUploadStatus>,
    pub snapshot_uploaded_at: Option<DateTime<Utc>>,  // 🆕 추가 필요
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

**작업 내용**:
- [x] `SnapshotUploadStatus` ENUM 타입 추가 (이미 완료)
- [x] `Annotation` 구조체에 `snapshot_image_key`, `snapshot_status` 추가 (이미 완료)
- [x] `Annotation` 구조체에 `snapshot_uploaded_at` 추가 (이미 완료 - 92줄)
- [x] `NewAnnotation` 구조체에 스냅샷 필드 추가 (이미 완료)

---

### **3단계: Infrastructure Layer - Repository** 🗄️

#### 3.1 Repository 인터페이스 확장

파일: `pacs-server/src/domain/repositories/annotation_repository.rs`

**새로운 메서드 추가**:

```rust
#[async_trait]
pub trait AnnotationRepository: Send + Sync {
    // ... 기존 메서드들 ...

    /// 스냅샷 정보 업데이트
    async fn update_snapshot(
        &self,
        annotation_id: i32,
        snapshot_image_key: String,
        snapshot_status: SnapshotUploadStatus,
        snapshot_uploaded_at: Option<DateTime<Utc>>,
    ) -> Result<Option<Annotation>, RepositoryError>;
}
```

#### 3.2 Repository 구현

파일: `pacs-server/src/infrastructure/repositories/annotation_repository_impl.rs`

**작업 내용**:
- [ ] `get_annotation_by_id` 쿼리에 스냅샷 필드 SELECT 추가
- [ ] `update_snapshot` 메서드 구현

**1. 모든 SELECT 쿼리 수정** (기존 쿼리에 필드 추가):

```rust
// ✅ 구현 완료 - 모든 조회 메서드에 적용됨
async fn find_by_id(&self, id: i32) -> Result<Option<Annotation>, sqlx::Error> {
    sqlx::query_as::<_, Annotation>(
        "SELECT id, project_id, user_id, study_uid, series_uid, instance_uid,
                tool_name, tool_version, data, is_shared,
                snapshot_image_key, snapshot_status, snapshot_uploaded_at,  -- 🆕 추가
                created_at, updated_at, version,
                viewer_software, description, measurement_values, label
         FROM annotation_annotation
         WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&self.pool)
    .await
}

// 동일한 패턴이 다음 메서드들에도 적용됨:
// - find_by_project_id, find_by_user_id, find_by_study_uid
// - find_by_series_uid, find_by_instance_uid
// - find_by_project_and_study, find_by_project_and_series
// - find_shared_annotations
```

**2. update_snapshot 메서드 구현**:

```rust
// ✅ 구현 완료 - annotation_repository_impl.rs:424-459
async fn update_snapshot(
    &self,
    id: i32,
    snapshot_image_key: String,
    snapshot_status: Option<SnapshotUploadStatus>,
    snapshot_uploaded_at: Option<DateTime<Utc>>,
) -> Result<Option<Annotation>, sqlx::Error> {
    let mut tx = self.pool.begin().await?;

    // 스냅샷 정보 업데이트
    let updated_annotation = sqlx::query_as::<_, Annotation>(
        "UPDATE annotation_annotation
         SET snapshot_image_key = $2,
             snapshot_status = $3,
             snapshot_uploaded_at = $4,
             updated_at = CURRENT_TIMESTAMP
         WHERE id = $1
         RETURNING id, project_id, user_id, study_uid, series_uid, instance_uid,
                   tool_name, tool_version, data, is_shared,
                   snapshot_image_key, snapshot_status, snapshot_uploaded_at,
                   created_at, updated_at, version,
                   viewer_software, description, measurement_values, label"
    )
    .bind(id)
    .bind(&snapshot_image_key)
    .bind(&snapshot_status)
    .bind(snapshot_uploaded_at)
    .fetch_optional(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(updated_annotation)
}
```

**작업 내용**:
- [x] `AnnotationRepository` trait에 `update_snapshot` 메서드 추가
- [x] `SnapshotUploadStatus` import 추가
- [x] 모든 SELECT 쿼리에 스냅샷 필드 3개 추가:
  - [x] `find_by_id`
  - [x] `find_by_project_id`
  - [x] `find_by_user_id`
  - [x] `find_by_study_uid`
  - [x] `find_by_series_uid`
  - [x] `find_by_instance_uid`
  - [x] `find_by_project_and_study`
  - [x] `find_by_project_and_series`
  - [x] `find_shared_annotations`
  - [x] `delete` (내부 SELECT)
- [x] 모든 RETURNING 절에 스냅샷 필드 3개 추가:
  - [x] `create`
  - [x] `update`
  - [x] `update_with_measurements`
  - [x] `update_with_version_check`
- [x] `update_snapshot` 메서드 구현

---

### **4단계: Application Layer - DTO** 📦

#### 4.1 DTO 파일 생성

새 파일: `pacs-server/src/application/dto/annotation_snapshot_dto.rs`

```rust
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// 스냅샷 업로드 URL 요청
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct SnapshotUploadUrlRequest {
    /// 파일명
    #[schema(example = "snapshot_20260111_120000.png")]
    pub filename: String,

    /// MIME 타입 (image/png, image/jpeg, image/webp)
    #[schema(example = "image/png")]
    pub mime_type: String,

    /// 파일 크기 (바이트)
    #[schema(example = 524288)]
    pub file_size: Option<i64>,

    /// TTL (초, 기본값: 600)
    #[schema(example = 600)]
    pub ttl_seconds: Option<u64>,
}

/// 스냅샷 업로드 URL 응답
#[derive(Debug, Serialize, ToSchema)]
pub struct SnapshotUploadUrlResponse {
    /// 업로드용 Presigned URL
    #[schema(example = "https://s3.amazonaws.com/bucket/annotations/123/snapshots/image.png?X-Amz-...")]
    pub upload_url: String,

    /// 다운로드용 Presigned URL
    #[schema(example = "https://s3.amazonaws.com/bucket/annotations/123/snapshots/image.png?X-Amz-...")]
    pub download_url: String,

    /// S3 object key (DB에 저장할 값)
    #[schema(example = "annotations/123/snapshots/snapshot_20260111_120000.png")]
    pub image_key: String,

    /// 만료 시간 (초)
    #[schema(example = 600)]
    pub expires_in: u64,

    /// 만료 시간 (ISO 8601)
    #[schema(example = "2026-01-11T13:00:00Z")]
    pub expires_at: String,
}

/// 스냅샷 업로드 완료 요청
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CompleteSnapshotUploadRequest {
    /// S3 object key (업로드 URL 생성 시 받은 값)
    #[schema(example = "annotations/123/snapshots/snapshot_20260111_120000.png")]
    pub image_key: String,

    /// 업로드 성공 여부 (optional, 기본값: true)
    /// false인 경우 실패로 처리하고 uploaded_at은 NULL로 유지
    #[schema(example = true)]
    pub success: Option<bool>,
}

// ⚠️ 주의: uploaded_at은 사용자가 보내는 게 아니라 서버에서 자동 생성!

/// 스냅샷 업로드 상태 응답
#[derive(Debug, Serialize, ToSchema)]
pub struct SnapshotStatusResponse {
    /// 어노테이션 ID
    pub annotation_id: i32,

    /// S3 object key
    pub image_key: Option<String>,

    /// 업로드 상태
    #[schema(example = "completed")]
    pub status: String,

    /// 업로드 완료 시간
    pub uploaded_at: Option<String>,
}
```

#### 4.2 DTO 모듈 등록

파일: `pacs-server/src/application/dto/mod.rs`

```rust
pub mod annotation_snapshot_dto;
pub use annotation_snapshot_dto::*;
```

**작업 내용**:
- [ ] `annotation_snapshot_dto.rs` 파일 생성
- [ ] `mod.rs`에 모듈 추가
- [ ] 컴파일 확인

---

### **5단계: Use Case 확장** 🎯

#### 5.1 AnnotationService 인터페이스 확장

파일: `pacs-server/src/domain/services/annotation_service.rs`

**새로운 메서드 추가**:

```rust
#[async_trait]
pub trait AnnotationService: Send + Sync {
    // ... 기존 메서드들 ...

    /// 스냅샷 정보 업데이트
    async fn update_snapshot(
        &self,
        annotation_id: i32,
        snapshot_image_key: String,
        snapshot_status: SnapshotUploadStatus,
        snapshot_uploaded_at: Option<DateTime<Utc>>,
    ) -> Result<Annotation, ServiceError>;
}
```

**구현**:

```rust
impl AnnotationService for AnnotationServiceImpl {
    async fn update_snapshot(
        &self,
        annotation_id: i32,
        snapshot_image_key: String,
        snapshot_status: SnapshotUploadStatus,
        snapshot_uploaded_at: Option<DateTime<Utc>>,
    ) -> Result<Annotation, ServiceError> {
        // 어노테이션 존재 확인
        let _ = self.get_annotation_by_id(annotation_id).await?;

        // 스냅샷 정보 업데이트
        match self.annotation_repository
            .update_snapshot(annotation_id, snapshot_image_key, snapshot_status, snapshot_uploaded_at)
            .await?
        {
            Some(updated) => Ok(updated),
            None => Err(ServiceError::NotFound("Annotation not found".into())),
        }
    }
}
```

#### 5.2 AnnotationUseCase 구조체 수정

파일: `pacs-server/src/application/use_cases/annotation_use_case.rs`

**SignedUrlService 의존성 추가**:

```rust
pub struct AnnotationUseCase<AS, UR, ACS, SUS> {
    annotation_service: Arc<AS>,
    user_repository: Arc<UR>,
    access_control_service: Arc<ACS>,
    signed_url_service: Arc<SUS>, // 🆕 추가
}

impl<AS, UR, ACS, SUS> AnnotationUseCase<AS, UR, ACS, SUS>
where
    AS: AnnotationService,
    UR: UserRepository,
    ACS: AccessControlService,
    SUS: SignedUrlService, // 🆕 추가
{
    pub fn new(
        annotation_service: Arc<AS>,
        user_repository: Arc<UR>,
        access_control_service: Arc<ACS>,
        signed_url_service: Arc<SUS>, // 🆕 추가
    ) -> Self {
        Self {
            annotation_service,
            user_repository,
            access_control_service,
            signed_url_service, // 🆕 추가
        }
    }
}
```

#### 5.3 스냅샷 업로드 URL 생성 메서드 추가

```rust
/// 스냅샷 업로드 URL 생성
pub async fn generate_snapshot_upload_url(
    &self,
    annotation_id: i32,
    request: SnapshotUploadUrlRequest,
    user_id: i32,
) -> Result<SnapshotUploadUrlResponse, ServiceError> {
    // 1. 어노테이션 존재 확인
    let annotation = self.annotation_service
        .get_annotation_by_id(annotation_id)
        .await?
        .ok_or_else(|| ServiceError::NotFound(
            format!("Annotation {} not found", annotation_id)
        ))?;

    // 2. 권한 확인 (어노테이션 소유자만 업로드 가능)
    if annotation.user_id != user_id {
        return Err(ServiceError::Unauthorized(
            "Not authorized to upload snapshot for this annotation".to_string()
        ));
    }

    // 3. S3 경로 생성
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let image_key = format!(
        "annotations/{}/snapshots/{}_{}",
        annotation_id,
        timestamp,
        request.filename
    );

    // 4. Signed URL 요청 생성
    let signed_url_request = crate::application::services::SignedUrlRequest::new(image_key.clone())
        .with_ttl(request.ttl_seconds.unwrap_or(600))
        .with_content_type(request.mime_type.clone());

    // 5. 업로드 URL 생성
    let upload_response = self.signed_url_service
        .generate_upload_url(signed_url_request)
        .await
        .map_err(|e| ServiceError::ExternalServiceError(format!("Failed to generate upload URL: {:?}", e)))?;

    // 6. 다운로드 URL 생성
    let download_response = self.signed_url_service
        .generate_download_url(image_key.clone(), Some(request.ttl_seconds.unwrap_or(600)))
        .await
        .map_err(|e| ServiceError::ExternalServiceError(format!("Failed to generate download URL: {:?}", e)))?;

    // 7. 🆕 어노테이션 상태를 'pending'으로 업데이트
    self.annotation_service
        .update_snapshot(
            annotation_id,
            image_key.clone(),
            SnapshotUploadStatus::Pending,
            None,  // uploaded_at은 아직 NULL
        )
        .await?;

    Ok(SnapshotUploadUrlResponse {
        upload_url: upload_response.url,
        download_url: download_response.url,
        image_key,
        expires_in: upload_response.ttl_seconds,
        expires_at: upload_response.expires_at.to_rfc3339(),
    })
}
```

#### 5.4 스냅샷 업로드 완료 메서드 추가

```rust
/// 스냅샷 업로드 완료 처리
pub async fn complete_snapshot_upload(
    &self,
    annotation_id: i32,
    request: CompleteSnapshotUploadRequest,
    user_id: i32,
) -> Result<Annotation, ServiceError> {
    // 1. 어노테이션 존재 및 권한 확인
    let annotation = self.annotation_service
        .get_annotation_by_id(annotation_id)
        .await?
        .ok_or_else(|| ServiceError::NotFound(
            format!("Annotation {} not found", annotation_id)
        ))?;

    if annotation.user_id != user_id {
        return Err(ServiceError::Unauthorized(
            "Not authorized to update this annotation".to_string()
        ));
    }

    // 2. 업로드 성공/실패에 따라 상태 업데이트
    let success = request.success.unwrap_or(true);
    let new_status = if success {
        SnapshotUploadStatus::Completed
    } else {
        SnapshotUploadStatus::Failed
    };

    // 3. ⭐ 서버에서 현재 시간을 자동으로 생성
    let now = chrono::Utc::now();

    // 4. 스냅샷 정보 업데이트
    self.annotation_service
        .update_snapshot(
            annotation_id,
            request.image_key,
            new_status,
            if success { Some(now) } else { None },  // ⭐ 성공 시에만 시간 기록
        )
        .await
}
```

**핵심 포인트**:
- ✅ `chrono::Utc::now().naive_utc()`로 **서버 시간** 자동 생성
- ✅ 사용자는 `success: true/false`만 보냄
- ✅ 성공 시에만 `uploaded_at` 기록
- ✅ 실패 시에는 `uploaded_at`을 NULL로 유지

**⚠️ 중요한 설계 결정**:
- **클라이언트가 시간을 보내면 안 되는 이유**:
  1. 시간대 불일치 문제 (클라이언트 로컬 시간 vs 서버 UTC)
  2. 보안 문제 (클라이언트가 시간 조작 가능)
  3. 데이터 일관성 (모든 타임스탬프는 서버 기준)
- **서버가 자동으로 시간을 생성해야 하는 이유**:
  1. 신뢰할 수 있는 단일 시간 소스
  2. UTC 기준 통일
  3. 감사(audit) 로그의 정확성 보장

> 📝 **관련 이슈**: [ISSUE-001: 타임스탬프 필드의 책임 소재](./issues/ISSUE-001-timestamp-responsibility.md)

#### 5.5 스냅샷 상태 조회 메서드 추가

```rust
/// 스냅샷 업로드 상태 조회
pub async fn get_snapshot_status(
    &self,
    annotation_id: i32,
    user_id: i32,
) -> Result<SnapshotStatusResponse, ServiceError> {
    // 1. 어노테이션 조회
    let annotation = self.annotation_service
        .get_annotation_by_id(annotation_id)
        .await?
        .ok_or_else(|| ServiceError::NotFound(
            format!("Annotation {} not found", annotation_id)
        ))?;

    // 2. 권한 확인 (소유자만 조회 가능)
    if annotation.user_id != user_id {
        return Err(ServiceError::Unauthorized(
            "Not authorized to view this annotation".to_string()
        ));
    }

    // 3. 상태 응답 생성
    Ok(SnapshotStatusResponse {
        annotation_id,
        image_key: annotation.snapshot_image_key,
        status: annotation.snapshot_status
            .map(|s| format!("{:?}", s).to_lowercase())
            .unwrap_or_else(|| "none".to_string()),
        uploaded_at: annotation.snapshot_uploaded_at
            .map(|dt| dt.and_utc().to_rfc3339()),
    })
}
```

**작업 내용**:
- [ ] `AnnotationUseCase` 구조체에 `SignedUrlService` 제네릭 추가
- [ ] `new` 메서드에 `signed_url_service` 파라미터 추가
- [ ] `generate_snapshot_upload_url` 메서드 구현
- [ ] `complete_snapshot_upload` 메서드 구현
- [ ] 컴파일 확인

---

### **6단계: Presentation Layer - Controller** 🎮

#### 6.1 컨트롤러 핸들러 추가

파일: `pacs-server/src/presentation/controllers/annotation_controller.rs`

**스냅샷 업로드 URL 생성 핸들러**:

```rust
/// 스냅샷 업로드 URL 생성
#[utoipa::path(
    post,
    path = "/annotations/{annotation_id}/snapshot/upload-url",
    tag = "Annotations",
    request_body = SnapshotUploadUrlRequest,
    responses(
        (status = 200, description = "Upload URL generated successfully", body = SnapshotUploadUrlResponse),
        (status = 401, description = "Unauthorized - Not the annotation owner"),
        (status = 404, description = "Annotation not found"),
        (status = 500, description = "Internal server error"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn generate_snapshot_upload_url<AS, UR, ACS, SUS>(
    path: web::Path<i32>,
    request: web::Json<SnapshotUploadUrlRequest>,
    http_req: HttpRequest,
    use_case: web::Data<Arc<AnnotationUseCase<AS, UR, ACS, SUS>>>,
) -> impl Responder
where
    AS: AnnotationService + 'static,
    UR: UserRepository + 'static,
    ACS: AccessControlService + 'static,
    SUS: SignedUrlService + 'static,
{
    let annotation_id = path.into_inner();

    // X-User-ID 헤더에서 사용자 ID 추출
    let user_id = http_req
        .headers()
        .get("X-User-ID")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(1);

    match use_case.generate_snapshot_upload_url(
        annotation_id,
        request.into_inner(),
        user_id
    ).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(ServiceError::NotFound(msg)) => HttpResponse::NotFound().json(json!({
            "error": "Not Found",
            "message": msg
        })),
        Err(ServiceError::Unauthorized(msg)) => HttpResponse::Unauthorized().json(json!({
            "error": "Unauthorized",
            "message": msg
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": "Internal Server Error",
            "message": format!("{:?}", e)
        })),
    }
}
```

**스냅샷 업로드 완료 핸들러**:

```rust
/// 스냅샷 업로드 완료
#[utoipa::path(
    post,
    path = "/annotations/{annotation_id}/snapshot/complete-upload",
    tag = "Annotations",
    request_body = CompleteSnapshotUploadRequest,
    responses(
        (status = 200, description = "Snapshot upload completed", body = AnnotationResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Annotation not found"),
        (status = 500, description = "Internal server error"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn complete_snapshot_upload<AS, UR, ACS, SUS>(
    path: web::Path<i32>,
    request: web::Json<CompleteSnapshotUploadRequest>,
    http_req: HttpRequest,
    use_case: web::Data<Arc<AnnotationUseCase<AS, UR, ACS, SUS>>>,
) -> impl Responder
where
    AS: AnnotationService + 'static,
    UR: UserRepository + 'static,
    ACS: AccessControlService + 'static,
    SUS: SignedUrlService + 'static,
{
    let annotation_id = path.into_inner();

    let user_id = http_req
        .headers()
        .get("X-User-ID")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(1);

    match use_case.complete_snapshot_upload(
        annotation_id,
        request.into_inner(),
        user_id
    ).await {
        Ok(annotation) => HttpResponse::Ok().json(AnnotationResponse::from(annotation)),
        Err(ServiceError::NotFound(msg)) => HttpResponse::NotFound().json(json!({
            "error": "Not Found",
            "message": msg
        })),
        Err(ServiceError::Unauthorized(msg)) => HttpResponse::Unauthorized().json(json!({
            "error": "Unauthorized",
            "message": msg
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": "Internal Server Error",
            "message": format!("{:?}", e)
        })),
    }
}
```

#### 6.2 라우트 등록

`configure_routes` 함수에 라우트 추가:

```rust
pub fn configure_routes(cfg: &mut web::ServiceConfig, use_case: Arc<AnnotationUseCase<...>>) {
    cfg.app_data(web::Data::new(use_case))
        .service(
            web::scope("/annotations")
                // ... 기존 라우트들 ...
                .route(
                    "/{annotation_id}/snapshot/upload-url",
                    web::post().to(generate_snapshot_upload_url::<AS, UR, ACS, SUS>)
                )
                .route(
                    "/{annotation_id}/snapshot/complete-upload",
                    web::post().to(complete_snapshot_upload::<AS, UR, ACS, SUS>)
                )
        );
}
```

**작업 내용**:
- [ ] `generate_snapshot_upload_url` 핸들러 추가
- [ ] `complete_snapshot_upload` 핸들러 추가
- [ ] `configure_routes`에 라우트 등록
- [ ] OpenAPI 문서 확인
- [ ] 컴파일 확인

---

### **7단계: main.rs 의존성 주입** 🔌

#### 7.1 AnnotationUseCase 생성 시 SignedUrlService 주입

파일: `pacs-server/src/main.rs`

```rust
// AnnotationUseCase 생성 (기존 코드 수정)
let annotation_use_case = Arc::new(AnnotationUseCase::new(
    annotation_service.clone(),
    Arc::new(user_repo.clone()),
    access_control_service.clone(),
    signed_url_service.clone(), // 🆕 추가
));
```

**작업 내용**:
- [ ] `main.rs`에서 `AnnotationUseCase::new()` 호출 시 `signed_url_service` 추가
- [ ] 컴파일 확인

---

### **8단계: 테스트 작성** 🧪

#### 8.1 E2E 테스트 파일 생성

새 파일: `pacs-server/test_annotation_snapshot_e2e.py`

```python
#!/usr/bin/env python3
"""
Annotation Snapshot Upload E2E Test
어노테이션 스냅샷 이미지 업로드 기능 테스트
"""

import requests
import os
from pathlib import Path

BASE_URL = "http://localhost:8080"
USER_ID = 1

def test_snapshot_upload_workflow():
    """스냅샷 업로드 전체 워크플로우 테스트"""

    # 1. 어노테이션 생성
    print("1. Creating annotation...")
    annotation_data = {
        "study_instance_uid": "1.2.840.113619.2.55.3.604688119.868",
        "series_instance_uid": "1.2.840.113619.2.55.3.604688119.868.1",
        "annotation_data": {
            "type": "measurement",
            "value": 10.5
        },
        "label": "Test Annotation"
    }

    response = requests.post(
        f"{BASE_URL}/annotations",
        json=annotation_data,
        headers={"X-User-ID": str(USER_ID)}
    )
    assert response.status_code == 201
    annotation = response.json()
    annotation_id = annotation["id"]
    print(f"✅ Annotation created: {annotation_id}")

    # 2. 스냅샷 업로드 URL 요청
    print("\n2. Requesting upload URL...")
    upload_request = {
        "filename": "snapshot_test.png",
        "mime_type": "image/png",
        "file_size": 1024,
        "ttl_seconds": 600
    }

    response = requests.post(
        f"{BASE_URL}/annotations/{annotation_id}/snapshot/upload-url",
        json=upload_request,
        headers={"X-User-ID": str(USER_ID)}
    )
    
    assert response.status_code == 200
    upload_data = response.json()
    print(f"✅ Upload URL generated")
    print(f"   - Upload URL: {upload_data['upload_url'][:50]}...")
    print(f"   - Image Key: {upload_data['image_key']}")
    print(f"   - Expires in: {upload_data['expires_in']}s")

    # 3. S3에 이미지 업로드 (테스트 이미지 사용)
    print("\n3. Uploading image to S3...")
    test_image_path = Path(__file__).parent / "test_images" / "sample_mask_1.png"

    if test_image_path.exists():
        with open(test_image_path, "rb") as f:
            image_data = f.read()

        response = requests.put(
            upload_data["upload_url"],
            data=image_data,
            headers={"Content-Type": "image/png"}
        )
        assert response.status_code in [200, 204]
        print(f"✅ Image uploaded to S3")
    else:
        print(f"⚠️  Test image not found, skipping S3 upload")

    # 4. 업로드 완료 알림
    print("\n4. Completing upload...")
    complete_request = {
        "image_key": upload_data["image_key"]
    }

    response = requests.post(
        f"{BASE_URL}/annotations/{annotation_id}/snapshot/complete-upload",
        json=complete_request,
        headers={"X-User-ID": str(USER_ID)}
    )
    assert response.status_code == 200
    updated_annotation = response.json()
    print(f"✅ Upload completed")
    print(f"   - Snapshot Image Key: {updated_annotation.get('snapshot_image_key')}")

    # 5. 어노테이션 조회하여 확인
    print("\n5. Verifying annotation...")
    response = requests.get(
        f"{BASE_URL}/annotations/{annotation_id}",
        headers={"X-User-ID": str(USER_ID)}
    )
    assert response.status_code == 200
    annotation = response.json()
    assert annotation["snapshot_image_key"] == upload_data["image_key"]
    print(f"✅ Annotation verified with snapshot image key")

    print("\n" + "="*50)
    print("🎉 All tests passed!")
    print("="*50)

if __name__ == "__main__":
    test_snapshot_upload_workflow()
```

#### 8.2 테스트 실행

```bash
cd pacs-server
chmod +x test_annotation_snapshot_e2e.py
python3 test_annotation_snapshot_e2e.py
```

**작업 내용**:
- [ ] E2E 테스트 스크립트 작성
- [ ] 테스트 실행 및 검증
- [ ] 에러 케이스 테스트 추가

---

## � 전체 구현 완료 요약 (2026-01-11)

### ✅ Phase 1: Entity & Repository Layer

#### 1. Entity Layer
- **파일**: `pacs-server/src/domain/entities/annotation.rs`
- `Annotation` 구조체에 스냅샷 필드 3개 추가 (88-92줄):
  - `snapshot_image_key: Option<String>`
  - `snapshot_status: Option<SnapshotUploadStatus>`
  - `snapshot_uploaded_at: Option<DateTime<Utc>>`
- `NewAnnotation` 구조체에 스냅샷 필드 2개 추가 (220-222줄)

#### 2. Repository Layer
- **파일**: `pacs-server/src/domain/repositories/annotation_repository.rs`
  - `update_snapshot` 메서드 시그니처 추가 (75-81줄)
  - `SnapshotUploadStatus` import 추가

- **파일**: `pacs-server/src/infrastructure/repositories/annotation_repository_impl.rs`
  - `SnapshotUploadStatus` import 추가
  - **9개 SELECT 쿼리** 수정 (스냅샷 필드 3개 추가):
    - `find_by_id`, `find_by_project_id`, `find_by_user_id`
    - `find_by_study_uid`, `find_by_series_uid`, `find_by_instance_uid`
    - `find_by_project_and_study`, `find_by_project_and_series`
    - `find_shared_annotations`
  - **4개 RETURNING 절** 수정:
    - `create`, `update`, `update_with_measurements`, `update_with_version_check`
  - `update_snapshot` 메서드 구현 (424-459줄)

#### 3. Test Layer
- **파일**: `pacs-server/tests/entities_test.rs`
  - `test_new_annotation_creation` 테스트에 스냅샷 필드 추가 (320-321줄)

---

### ✅ Phase 2: Domain Service Layer

#### 4. Domain Service
- **파일**: `pacs-server/src/domain/services/annotation_service.rs`
  - `update_snapshot` 메서드 시그니처 추가 (trait)

- **파일**: `pacs-server/src/domain/services/annotation_service_impl.rs`
  - `update_snapshot` 메서드 구현
  - 어노테이션 존재 확인 후 repository 호출

---

### ✅ Phase 3: Application Layer

#### 5. DTO Layer
- **파일**: `pacs-server/src/application/dto/annotation_snapshot_dto.rs` (신규 생성)
  - `SnapshotUploadUrlRequest`: 업로드 URL 요청 DTO
  - `SnapshotUploadUrlResponse`: 업로드 URL 응답 DTO
  - `CompleteSnapshotUploadRequest`: 업로드 완료 요청 DTO
  - `SnapshotStatusResponse`: 스냅샷 상태 응답 DTO
  - **오타 수정**: `tll_seconds` → `ttl_seconds`

- **파일**: `pacs-server/src/application/dto/mod.rs`
  - `annotation_snapshot_dto` 모듈 등록

#### 6. Use Case Layer
- **파일**: `pacs-server/src/application/use_cases/annotation_use_case.rs`
  - `SignedUrlService` 제네릭 타입 추가
  - `new()` 메서드에 `signed_url_service` 파라미터 추가
  - **3개 메서드 구현**:
    - `generate_snapshot_upload_url()`: Presigned URL 생성 + DB 상태 'pending' 업데이트
    - `complete_snapshot_upload()`: 업로드 완료 처리 + 상태 'completed'/'failed' 업데이트
    - `get_snapshot_status()`: 스냅샷 상태 조회
  - **필수 import 추가**:
    - `SnapshotUploadUrlRequest`, `SnapshotUploadUrlResponse`
    - `CompleteSnapshotUploadRequest`, `SnapshotStatusResponse`
    - `SignedUrlService`, `SnapshotUploadStatus`

---

### ✅ Phase 4: Presentation Layer

#### 7. Controller Layer
- **파일**: `pacs-server/src/presentation/controllers/annotation_controller.rs`
  - **3개 핸들러 구현**:
    - `generate_snapshot_upload_url()`: POST `/annotations/{id}/snapshot/upload-url`
    - `complete_snapshot_upload()`: POST `/annotations/{id}/snapshot/complete-upload`
    - `get_snapshot_status()`: GET `/annotations/{id}/snapshot/status`
  - **JWT 인증 통합**:
    - `extract_user_id_from_request()` 사용
    - `JwtService`, `UserRepositoryImpl` 파라미터 추가
  - **OpenAPI 문서화**:
    - `#[utoipa::path]` 어노테이션 추가 (오타 수정: `utopia` → `utoipa`)
  - **라우트 등록**:
    - `configure_routes()`에 3개 엔드포인트 추가
  - **필수 import 추가**:
    - `SnapshotUploadUrlRequest`, `SnapshotUploadUrlResponse`
    - `SignedUrlService`, `UserRepository`, `AccessControlService`
    - `JwtService`, `UserRepositoryImpl`

#### 8. Dependency Injection
- **파일**: `pacs-server/src/main.rs`
  - `AnnotationUseCase::new()` 호출 시 `signed_url_service.clone()` 추가
  - **타입 수정**: `(*signed_url_service).clone()` → `signed_url_service.clone()`

---

### ✅ Phase 5: Type System 수정

#### 9. SignedUrlService Trait Implementation
- **파일**: `pacs-server/src/application/services/signed_url_service.rs`
  - **Blanket Implementation 추가**:
    - `Arc<SignedUrlServiceImpl>`이 `SignedUrlService` trait을 구현하도록 수정
    - 모든 trait 메서드를 `(**self)`로 위임
  - **이유**: `SignedUrlServiceImpl`은 `Box<dyn ObjectStorageService>`를 포함하므로 `Clone` derive 불가

- **파일**: `pacs-server/src/presentation/controllers/annotation_controller.rs`
  - `configure_routes()` 타입 시그니처 수정:
    - `SignedUrlServiceImpl` → `Arc<SignedUrlServiceImpl>`

---

### 🔧 주요 버그 수정

#### 컴파일 에러 해결 과정

1. **Import 누락 에러** (7개)
   - ✅ `annotation_use_case.rs`: DTO 4개, `SignedUrlService`, `SnapshotUploadStatus` import 추가
   - ✅ `annotation_controller.rs`: DTO 2개, Service 3개, Auth 2개 import 추가

2. **DTO 오타 에러**
   - ✅ `annotation_snapshot_dto.rs`: `tll_seconds` → `ttl_seconds`

3. **타입 불일치 에러** (3개)
   - ✅ `get_annotation_by_id()`는 `Result<Annotation, ServiceError>` 반환 → `.ok_or_else()` 제거
   - ✅ `generate_download_url()`은 `SignedUrlRequest` 받음 → 파라미터 수정
   - ✅ `DateTime<Utc>`는 이미 UTC → `.and_utc()` 제거

4. **함수 본문 누락 에러**
   - ✅ `generate_snapshot_upload_url()` 컨트롤러 핸들러 본문 구현

5. **함수 시그니처 에러**
   - ✅ `extract_user_id_from_request()`는 3개 파라미터 필요 + `async`
   - ✅ `jwt`, `user_repo` 파라미터 추가

6. **Clone 에러**
   - ✅ `main.rs`: `(*signed_url_service).clone()` → `signed_url_service.clone()`

7. **Trait 구현 에러**
   - ✅ `Arc<SignedUrlServiceImpl>`이 `SignedUrlService` trait 구현하도록 blanket impl 추가
   - ✅ `configure_routes()` 타입 시그니처를 `Arc<SignedUrlServiceImpl>`로 수정

---

### 🔍 주요 설계 결정

1. **ISSUE-001**: 타임스탬프 생성 책임 소재
   - ✅ 결정: 서버가 `snapshot_uploaded_at`을 자동 생성
   - ✅ 클라이언트는 `success: true/false`만 전송
   - ✅ 이유: 시간대 불일치, 보안, 데이터 일관성

2. **ISSUE-002**: UpdateAnnotation Entity 부재
   - ✅ 결정: 기존 패턴 유지, `update_snapshot` 전용 메서드 추가
   - ✅ 개별 파라미터 방식으로 구현

3. **ISSUE-003**: Arc<SignedUrlServiceImpl> Clone 문제
   - ✅ 결정: Blanket implementation으로 `Arc<T>`가 trait 구현하도록 수정
   - ✅ 이유: `Box<dyn ObjectStorageService>` 때문에 `Clone` derive 불가

---

### ✅ 최종 빌드 상태

```bash
cd pacs-server
cargo check
```

**결과**:
```
Finished `dev` profile [unoptimized + debuginfo] target(s) in 12.37s
warning: the following packages contain code that will be rejected by a future version of Rust: pacs_server v0.1.0, sqlx-postgres v0.7.4
```

- ✅ **컴파일 성공!**
- ⚠️ 경고만 있음 (기존 경고, 새로운 에러 없음)
- 🎯 **모든 타입 에러 해결 완료!**

---

## 📝 전체 구현 체크리스트

### Phase 1: 데이터베이스 & 도메인 ✅
- [x] 1.1 마이그레이션 파일 작성 및 적용
- [x] 1.2 마이그레이션 적용 확인
- [x] 2.1 `Annotation` entity에 `snapshot_uploaded_at` 필드 추가
- [x] 2.2 `NewAnnotation`에 스냅샷 필드 추가
- [x] 2.3 Use Case에서 `NewAnnotation` 생성 시 스냅샷 필드 초기화
- [x] 2.4 테스트 파일 수정 (`entities_test.rs`)
- [x] 3.1 `AnnotationRepository` trait에 `update_snapshot` 메서드 추가
- [x] 3.2 모든 SELECT 쿼리에 스냅샷 필드 3개 추가 (9개 메서드)
- [x] 3.3 모든 RETURNING 절에 스냅샷 필드 3개 추가 (4개 메서드)
- [x] 3.4 `update_snapshot` 메서드 구현
- [x] 3.5 `SnapshotUploadStatus` import 추가

### Phase 2: 도메인 서비스 ✅
- [x] 4.1 `AnnotationService` trait에 `update_snapshot` 메서드 추가
- [x] 4.2 `AnnotationServiceImpl`에 `update_snapshot` 구현

### Phase 3: 애플리케이션 레이어 ✅
- [x] 5.1 `annotation_snapshot_dto.rs` 파일 생성
- [x] 5.2 DTO 모듈 등록
- [x] 5.3 `AnnotationUseCase`에 `SignedUrlService` 제네릭 추가
- [x] 5.4 `generate_snapshot_upload_url` 메서드 구현
- [x] 5.5 `complete_snapshot_upload` 메서드 구현
- [x] 5.6 `get_snapshot_status` 메서드 구현

### Phase 4: 프레젠테이션 레이어 ✅
- [x] 6.1 `generate_snapshot_upload_url` 컨트롤러 핸들러 추가
- [x] 6.2 `complete_snapshot_upload` 컨트롤러 핸들러 추가
- [x] 6.3 `get_snapshot_status` 컨트롤러 핸들러 추가
- [x] 6.4 라우트 등록
- [x] 7.1 `main.rs`에서 의존성 주입

### Phase 5: 테스트 & 검증
- [ ] 8.1 E2E 테스트 스크립트 작성
- [ ] 8.2 테스트 실행 및 검증
- [ ] 8.3 에러 케이스 테스트
- [ ] 8.4 OpenAPI 문서 확인

---

## 🔧 Phase 6: 추가 버그 수정 (2026-01-12)

### 누락된 핸들러 함수 추가

#### 문제
- `get_snapshot_status` 핸들러 함수가 구현되지 않음
- 라우트에서 참조하지만 함수가 존재하지 않아 컴파일 에러 발생

#### 해결
**파일**: `pacs-server/src/presentation/controllers/annotation_controller.rs`

1. **`get_snapshot_status` 핸들러 추가** (1523-1573줄):
```rust
#[utoipa::path(
    get,
    path = "/annotations/{annotation_id}/snapshot/status",
    tag = "Annotations",
    responses(
        (status = 200, description = "Snapshot status retrieved", body = SnapshotStatusResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Annotation not found"),
        (status = 500, description = "Internal server error"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_snapshot_status<AS, UR, ACS, SUS>(
    path: web::Path<i32>,
    http_req: HttpRequest,
    jwt: web::Data<Arc<JwtService>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
    use_case: web::Data<Arc<AnnotationUseCase<AS, UR, ACS, SUS>>>,
) -> impl Responder
where
    AS: AnnotationService + 'static,
    UR: UserRepository + 'static,
    ACS: AccessControlService + 'static,
    SUS: SignedUrlService + 'static,
{
    let annotation_id = path.into_inner();

    // Extract user_id from JWT token
    let user_id = match extract_user_id_from_request(&http_req, &jwt, &user_repo).await {
        Some(id) if id > 0 => id,
        _ => return HttpResponse::Unauthorized().json(json!({
            "error": "Unauthorized",
            "message": "Invalid or missing authentication token"
        })),
    };

    match use_case.get_snapshot_status(annotation_id, user_id).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => AnnotationController::handle_service_error(e),
    }
}
```

2. **Import 추가** (5-8줄):
```rust
use crate::application::dto::{
    SnapshotUploadUrlRequest, SnapshotUploadUrlResponse,
    CompleteSnapshotUploadRequest, SnapshotStatusResponse,  // 🆕 추가
};
```

3. **에러 처리 수정** (1518줄):
```rust
// ❌ 잘못된 코드 (AnnotationResponse::from()이 Annotation을 받지 못함)
Ok(annotation) => HttpResponse::Ok().json(AnnotationResponse::from(annotation)),

// ✅ 수정된 코드 (기존 패턴과 일치)
Ok(annotation) => HttpResponse::Ok().json(annotation),
```

### 라우트 패턴 일관성 확인

#### 확인 사항
- ✅ **같은 파일 내 함수**: `crate::` 없이 함수명만 사용
  - 예: `generate_snapshot_upload_url`, `complete_snapshot_upload`, `get_snapshot_status`
- ✅ **다른 컨트롤러 함수**: 전체 경로 사용
  - 예: `crate::presentation::controllers::mask_group_controller::create_mask_group`

#### 최종 라우트 등록 (1812-1873줄)
```rust
.route(
    "/{annotation_id}/snapshot/upload-url",
    web::post().to(
        generate_snapshot_upload_url::<
            AnnotationServiceImpl<...>,
            UserRepositoryImpl,
            AccessControlServiceImpl<...>,
            Arc<SignedUrlServiceImpl>,
        >,
    ),
)
.route(
    "/{annotation_id}/snapshot/complete-upload",
    web::post().to(
        complete_snapshot_upload::<
            AnnotationServiceImpl<...>,
            UserRepositoryImpl,
            AccessControlServiceImpl<...>,
            Arc<SignedUrlServiceImpl>,
        >,
    ),
)
.route(
    "/{annotation_id}/snapshot/status",
    web::get().to(
        get_snapshot_status::<
            AnnotationServiceImpl<...>,
            UserRepositoryImpl,
            AccessControlServiceImpl<...>,
            Arc<SignedUrlServiceImpl>,
        >,
    ),
)
```

### 컴파일 에러 해결 과정

#### 에러 1: `get_snapshot_status` not found
```
error[E0425]: cannot find value `get_snapshot_status` in this scope
    --> src/presentation/controllers/annotation_controller.rs:1812:25
```
**해결**: `get_snapshot_status` 핸들러 함수 구현

#### 에러 2: `CompleteSnapshotUploadRequest` not found
```
error[E0412]: cannot find type `CompleteSnapshotUploadRequest` in this scope
```
**해결**: Import 추가

#### 에러 3: `SnapshotStatusResponse` not found
```
error[E0412]: cannot find type `SnapshotStatusResponse` in this scope
```
**해결**: Import 추가

#### 에러 4: Type mismatch in `complete_snapshot_upload`
```
error[E0308]: mismatched types
expected `AnnotationResponse`, found `Annotation`
```
**해결**: `AnnotationResponse::from()` 제거, 직접 `annotation` 반환

---

### ✅ 최종 빌드 상태 (Phase 6 완료)

```bash
cd pacs-server
cargo check
```

**결과**:
```
Finished `dev` profile [unoptimized + debuginfo] target(s) in 14.86s
warning: the following packages contain code that will be rejected by a future version of Rust: pacs_server v0.1.0, sqlx-postgres v0.7.4
```

- ✅ **컴파일 성공!**
- ⚠️ 경고만 있음 (기존 경고)
- 🎯 **모든 핸들러 및 라우트 구현 완료!**

---

### 📊 구현 완료 통계

#### 추가된 파일
- `pacs-server/src/application/dto/annotation_snapshot_dto.rs` (신규)

#### 수정된 파일 (총 8개)
1. `pacs-server/src/domain/entities/annotation.rs`
2. `pacs-server/src/domain/repositories/annotation_repository.rs`
3. `pacs-server/src/infrastructure/repositories/annotation_repository_impl.rs`
4. `pacs-server/src/domain/services/annotation_service.rs`
5. `pacs-server/src/domain/services/annotation_service_impl.rs`
6. `pacs-server/src/application/use_cases/annotation_use_case.rs`
7. `pacs-server/src/presentation/controllers/annotation_controller.rs`
8. `pacs-server/src/main.rs`
9. `pacs-server/src/application/services/signed_url_service.rs`
10. `pacs-server/src/application/dto/mod.rs`

#### 추가된 API 엔드포인트 (3개)
1. `POST /annotations/{id}/snapshot/upload-url` - Presigned URL 생성
2. `POST /annotations/{id}/snapshot/complete-upload` - 업로드 완료 처리
3. `GET /annotations/{id}/snapshot/status` - 스냅샷 상태 조회

#### 추가된 코드 라인 수
- DTO: ~150 lines
- Use Case: ~200 lines
- Controller: ~150 lines
- Repository: ~100 lines
- Service: ~50 lines
- Type System: ~70 lines
- **총계**: ~720 lines

---

## 🚀 구현 순서 (권장)

```
Phase 1: DB & Domain
1단계 (DB Migration) → 2단계 (Entity) → 3단계 (Repository)
    ↓
Phase 2: Domain Service
4단계 (AnnotationService)
    ↓
Phase 3: Application
5단계 (DTO) → 6단계 (Use Case)
    ↓
Phase 4: Presentation
7단계 (Controller) → 8단계 (DI)
    ↓
Phase 5: Test
9단계 (E2E Test)
```

**⚠️ 중요**: 현재 프로젝트는 `UpdateAnnotation` entity가 없고, repository에서 개별 파라미터로 업데이트를 처리합니다. 따라서 `update_snapshot` 메서드를 새로 추가해야 합니다.

---

## 🔍 API 사용 예시

### 1. 스냅샷 업로드 URL 요청

```bash
curl -X POST "http://localhost:8080/annotations/123/snapshot/upload-url" \
  -H "Content-Type: application/json" \
  -H "X-User-ID: 1" \
  -d '{
    "filename": "snapshot_20260111_120000.png",
    "mime_type": "image/png",
    "file_size": 524288,
    "ttl_seconds": 600
  }'
```

**응답**:
```json
{
  "upload_url": "https://s3.amazonaws.com/bucket/annotations/123/snapshots/...",
  "download_url": "https://s3.amazonaws.com/bucket/annotations/123/snapshots/...",
  "image_key": "annotations/123/snapshots/20260111_120000_snapshot.png",
  "expires_in": 600,
  "expires_at": "2026-01-11T13:00:00Z"
}
```

### 2. S3에 이미지 업로드

```bash
curl -X PUT "{upload_url}" \
  -H "Content-Type: image/png" \
  --data-binary @snapshot.png
```

### 3. 업로드 완료 알림

**성공 케이스**:
```bash
curl -X POST "http://localhost:8080/annotations/123/snapshot/complete-upload" \
  -H "Content-Type: application/json" \
  -H "X-User-ID: 1" \
  -d '{
    "image_key": "annotations/123/snapshots/20260111_120000_snapshot.png",
    "success": true
  }'
```

**응답** (서버가 자동으로 현재 시간 기록):
```json
{
  "id": 123,
  "snapshot_image_key": "annotations/123/snapshots/20260111_120000_snapshot.png",
  "snapshot_status": "completed",
  "snapshot_uploaded_at": "2026-01-11T12:30:45Z"  // ⭐ 서버에서 자동 생성
}
```

**실패 케이스**:
```bash
curl -X POST "http://localhost:8080/annotations/123/snapshot/complete-upload" \
  -H "Content-Type: application/json" \
  -H "X-User-ID: 1" \
  -d '{
    "image_key": "annotations/123/snapshots/20260111_120000_snapshot.png",
    "success": false
  }'
```

**응답**:
```json
{
  "id": 123,
  "snapshot_image_key": "annotations/123/snapshots/20260111_120000_snapshot.png",
  "snapshot_status": "failed",
  "snapshot_uploaded_at": null  // ⭐ 실패 시 NULL
}
```

### 4. 스냅샷 상태 조회

```bash
curl -X GET "http://localhost:8080/annotations/123/snapshot/status" \
  -H "X-User-ID: 1"
```

**응답**:
```json
{
  "annotation_id": 123,
  "image_key": "annotations/123/snapshots/20260111_120000_snapshot.png",
  "status": "completed",
  "uploaded_at": "2026-01-11T12:30:00Z"
}
```

---

## 📚 참고 자료

### 문서
- [ARCHITECTURE.md](./ARCHITECTURE.md) - 아키텍처 결정 문서
- [API_SPEC.md](./API_SPEC.md) - API 명세서
- [issues/](./issues/) - 설계 결정 및 이슈 문서

### 코드 참고
- 기존 마스크 업로드 구현: `pacs-server/src/presentation/controllers/mask_group_controller.rs`
- SignedUrlService: `pacs-server/src/application/services/signed_url_service.rs`

### 주요 이슈
- [ISSUE-001: 타임스탬프 필드의 책임 소재](./issues/ISSUE-001-timestamp-responsibility.md)

---

## ✅ 완료 기준

- [x] 모든 체크리스트 항목 완료 (Phase 1-6)
- [x] E2E 테스트 통과 ✅
- [x] OpenAPI 문서 생성 확인
- [x] 웹 관리 페이지 통합 ✅
- [x] 문서 업데이트 완료

---

## 🎉 구현 완료! (2026-01-12)

### ✅ 완료된 작업 요약

**Phase 1-6 모두 완료**:
- ✅ DB 마이그레이션 및 Entity 수정
- ✅ Repository Layer 구현
- ✅ Domain Service Layer 구현
- ✅ Application Layer (DTO + Use Case) 구현
- ✅ Presentation Layer (Controller + Routes) 구현
- ✅ Type System 수정 (Arc blanket impl)
- ✅ 모든 컴파일 에러 해결

**Phase 7: E2E 테스트 & 웹 관리 페이지 (완료)**:
- ✅ E2E 테스트 스크립트 작성 (`e2e/test_annotation_snapshot_e2e.py`)
- ✅ 테스트 실행 및 검증 (모든 테스트 통과)
- ✅ 웹 관리 페이지 통합 (`auth-dashboard`)
- ✅ 타임아웃 이슈 해결 (비동기 실행 + 타임아웃 증가)

---

## 🔧 Phase 7: E2E 테스트 & 웹 관리 페이지 (2026-01-12)

### 7.1 E2E 테스트 스크립트

**파일**: `pacs-server/e2e/test_annotation_snapshot_e2e.py`

**테스트 시나리오**:
1. ✅ Keycloak 로그인
2. ✅ 어노테이션 생성
3. ✅ 스냅샷 업로드 URL 요청
4. ✅ 테스트 이미지 생성 (PIL)
5. ✅ S3에 이미지 업로드
6. ✅ 업로드 완료 알림
7. ✅ 스냅샷 상태 조회
8. ✅ 어노테이션 조회하여 스냅샷 정보 확인

**실행 방법**:
```bash
cd pacs-server
python3 e2e/test_annotation_snapshot_e2e.py
```

**결과**:
```
🎉 모든 테스트 통과!
```

### 7.2 웹 관리 페이지 통합

**파일**: `auth-dashboard/src/components/ApiScenarioTests/AnnotationSnapshotTests.tsx`

**기능**:
1. **E2E 테스트 실행**
   - 서버 엔드포인트: `GET /api/test/annotation-snapshot-e2e`
   - 실시간 출력 표시
   - 테스트 결과 시각화

2. **CRUD 인터페이스**
   - 어노테이션 생성
   - 업로드 URL 요청
   - 업로드 완료 처리
   - 상태 조회
   - 데이터 표시 (ID, Key, Status, Uploaded At)

**접속 방법**:
1. http://localhost:3000 접속
2. 사이드바 → API 점검 클릭
3. Annotation Snapshot (📸) 클릭

### 7.3 타임아웃 이슈 해결

#### 문제
- 웹 페이지에서 E2E 테스트 실행 시 타임아웃 발생
- `complete-upload` 엔드포인트에서 S3 다운로드 시간이 오래 걸림
- Python 프로세스 실행 중 HTTP 연결 타임아웃

#### 해결 방법

**1. Python 테스트 스크립트 타임아웃 증가**
```python
# 모든 HTTP 요청 타임아웃 증가
timeout=30  # 기존 10초 → 30초
timeout=60  # complete-upload는 60초 (S3 다운로드 고려)
```

**2. Rust 서버 비동기 실행**
```rust
// tokio::task::spawn_blocking으로 비동기 실행
let result = timeout(
    Duration::from_secs(120),  // 120초 타임아웃
    tokio::task::spawn_blocking(|| {
        Command::new("python3")
            .arg("e2e/test_annotation_snapshot_e2e.py")
            .output()
    })
).await;
```

**파일**: `pacs-server/src/presentation/controllers/test_controller.rs`

**변경 사항**:
- `std::process::Command` → `tokio::task::spawn_blocking`
- 동기 실행 → 비동기 실행
- 타임아웃 10초 → 120초
- HTTP 연결 유지

### 7.4 사이드바 메뉴 추가

**파일**: `auth-dashboard/src/constants/app.constants.ts`

```typescript
{
  id: 'api-health-annotation-snapshot',
  label: 'Annotation Snapshot',
  icon: '📸',
  path: '/api-health/annotation-snapshot',
}
```

**파일**: `auth-dashboard/src/components/Dashboard.tsx`

```typescript
import AnnotationSnapshotTests from './ApiScenarioTests/AnnotationSnapshotTests';

// 렌더링
{activeMenu === API_SUB_MENU.ANNOTATION_SNAPSHOT && (
  <AnnotationSnapshotTests />
)}
```

---

## 📊 최종 구현 통계

### 추가된 파일 (총 2개)
1. `pacs-server/e2e/test_annotation_snapshot_e2e.py` (286 lines)
2. `auth-dashboard/src/components/ApiScenarioTests/AnnotationSnapshotTests.tsx` (462 lines)

### 수정된 파일 (총 12개)
1. `pacs-server/src/domain/entities/annotation.rs`
2. `pacs-server/src/domain/repositories/annotation_repository.rs`
3. `pacs-server/src/infrastructure/repositories/annotation_repository_impl.rs`
4. `pacs-server/src/domain/services/annotation_service.rs`
5. `pacs-server/src/domain/services/annotation_service_impl.rs`
6. `pacs-server/src/application/dto/annotation_snapshot_dto.rs` (신규)
7. `pacs-server/src/application/use_cases/annotation_use_case.rs`
8. `pacs-server/src/presentation/controllers/annotation_controller.rs`
9. `pacs-server/src/presentation/controllers/test_controller.rs`
10. `pacs-server/src/main.rs`
11. `auth-dashboard/src/constants/app.constants.ts`
12. `auth-dashboard/src/components/Dashboard.tsx`

### 추가된 API 엔드포인트 (4개)
1. `POST /api/annotations/{id}/snapshot/upload-url` - Presigned URL 생성
2. `POST /api/annotations/{id}/snapshot/complete-upload` - 업로드 완료 처리
3. `GET /api/annotations/{id}/snapshot/status` - 스냅샷 상태 조회
4. `GET /api/test/annotation-snapshot-e2e` - E2E 테스트 실행

### 총 코드 라인 수
- Backend (Rust): ~720 lines
- E2E Test (Python): ~286 lines
- Frontend (React): ~462 lines
- **총계**: ~1,468 lines

---

## 🎯 테스트 결과

### E2E 테스트 (직접 실행)
```bash
$ python3 e2e/test_annotation_snapshot_e2e.py

🚀 Annotation Snapshot E2E Test 시작...
✅ 로그인 성공
✅ 어노테이션 생성 성공!
✅ 업로드 URL 생성 성공!
✅ 이미지 생성 완료!
✅ S3 업로드 성공!
✅ 업로드 완료 처리 성공!
✅ 상태 조회 성공!
✅ 어노테이션 조회 성공!

🎉 모든 테스트 통과!
```

### 웹 관리 페이지 테스트
- ✅ E2E 테스트 실행 버튼 동작
- ✅ 실시간 출력 표시
- ✅ CRUD 인터페이스 동작
- ✅ 데이터 시각화
- ✅ 타임아웃 이슈 해결

---

**작업 완료**: 2026-01-12 🎉
**최종 상태**: 모든 Phase 완료, 테스트 통과, 웹 관리 페이지 통합 완료
