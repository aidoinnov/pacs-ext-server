# 💻 PACS 마스크 업로드 v2 코드 구현 가이드

## 📁 파일 구조

```
pacs-server/src/
├── domain/
│   ├── entities/
│   │   ├── mask_group.rs          # MaskGroup 엔티티
│   │   └── mask.rs                # Mask 엔티티
│   └── repositories/
│       ├── mask_group_repository.rs
│       └── mask_repository.rs
├── application/
│   ├── dto/
│   │   ├── mask_group_dto.rs      # 마스크 그룹 DTO
│   │   └── mask_dto.rs            # 마스크 DTO
│   ├── use_cases/
│   │   ├── mask_group_use_case.rs
│   │   └── mask_use_case.rs
│   └── services/
│       └── object_storage_service.rs
├── infrastructure/
│   ├── repositories/
│   │   ├── mask_group_repository_impl.rs
│   │   └── mask_repository_impl.rs
│   └── external/
│       ├── s3_service.rs
│       └── minio_service.rs
└── presentation/
    └── controllers/
        ├── mask_group_controller.rs
        └── mask_controller.rs
```

## 🗄️ 1. 데이터베이스 스키마

### 1.1 Migration 파일 생성
```bash
# pacs-server/migrations/003_add_mask_tables.sql
```

```sql
-- annotation_mask_group 테이블 생성
CREATE TABLE annotation_mask_group (
    id SERIAL PRIMARY KEY,
    annotation_id INTEGER NOT NULL REFERENCES annotation_annotation(id) ON DELETE CASCADE,
    group_name TEXT,
    model_name TEXT,
    version TEXT,
    modality TEXT,
    slice_count INTEGER DEFAULT 1,
    mask_type TEXT DEFAULT 'segmentation',
    description TEXT,
    created_by INTEGER,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- annotation_mask 테이블 생성
CREATE TABLE annotation_mask (
    id SERIAL PRIMARY KEY,
    mask_group_id INTEGER NOT NULL REFERENCES annotation_mask_group(id) ON DELETE CASCADE,
    slice_index INTEGER,
    label_name TEXT,
    file_path TEXT NOT NULL,
    mime_type TEXT DEFAULT 'image/png',
    file_size BIGINT,
    checksum TEXT,
    width INTEGER,
    height INTEGER,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 인덱스 생성
CREATE INDEX idx_mask_group_annotation_id ON annotation_mask_group(annotation_id);
CREATE INDEX idx_mask_mask_group_id ON annotation_mask(mask_group_id);
```

## 🏗️ 2. Domain Layer 구현

### 2.1 MaskGroup 엔티티
```rust
// src/domain/entities/mask_group.rs
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MaskGroup {
    pub id: i32,
    pub annotation_id: i32,
    pub group_name: Option<String>,
    pub model_name: Option<String>,
    pub version: Option<String>,
    pub modality: Option<String>,
    pub slice_count: i32,
    pub mask_type: String,
    pub description: Option<String>,
    pub created_by: Option<i32>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewMaskGroup {
    pub annotation_id: i32,
    pub group_name: Option<String>,
    pub model_name: Option<String>,
    pub version: Option<String>,
    pub modality: Option<String>,
    pub slice_count: i32,
    pub mask_type: String,
    pub description: Option<String>,
    pub created_by: Option<i32>,
}
```

### 2.2 Mask 엔티티
```rust
// src/domain/entities/mask.rs
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Mask {
    pub id: i32,
    pub mask_group_id: i32,
    pub slice_index: Option<i32>,
    pub label_name: Option<String>,
    pub file_path: String,
    pub mime_type: String,
    pub file_size: Option<i64>,
    pub checksum: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewMask {
    pub mask_group_id: i32,
    pub slice_index: Option<i32>,
    pub label_name: Option<String>,
    pub file_path: String,
    pub mime_type: String,
    pub file_size: Option<i64>,
    pub checksum: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
}
```

### 2.3 Repository Traits
```rust
// src/domain/repositories/mask_group_repository.rs
use async_trait::async_trait;
use crate::domain::entities::{MaskGroup, NewMaskGroup};

#[async_trait]
pub trait MaskGroupRepository: Send + Sync {
    async fn create(&self, new_mask_group: NewMaskGroup) -> Result<MaskGroup, sqlx::Error>;
    async fn find_by_id(&self, id: i32) -> Result<Option<MaskGroup>, sqlx::Error>;
    async fn find_by_annotation_id(&self, annotation_id: i32) -> Result<Vec<MaskGroup>, sqlx::Error>;
    async fn update(&self, id: i32, mask_group: MaskGroup) -> Result<Option<MaskGroup>, sqlx::Error>;
    async fn delete(&self, id: i32) -> Result<bool, sqlx::Error>;
}
```

## 📦 3. Application Layer 구현

### 3.1 DTOs
```rust
// src/application/dto/mask_group_dto.rs
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use chrono::NaiveDateTime;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CreateMaskGroupRequest {
    /// 그룹 이름
    #[schema(example = "Liver_Segmentation_v2")]
    pub group_name: Option<String>,
    
    /// AI 모델 이름
    #[schema(example = "monai_unet")]
    pub model_name: Option<String>,
    
    /// 버전 정보
    #[schema(example = "v2.1.0")]
    pub version: Option<String>,
    
    /// 의료 영상 모달리티
    #[schema(example = "CT")]
    pub modality: Option<String>,
    
    /// 예상 슬라이스 수
    #[schema(example = 120)]
    pub slice_count: i32,
    
    /// 마스크 타입
    #[schema(example = "segmentation")]
    pub mask_type: String,
    
    /// 설명
    #[schema(example = "간 세그멘테이션 결과")]
    pub description: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MaskGroupResponse {
    pub id: i32,
    pub annotation_id: i32,
    pub group_name: Option<String>,
    pub model_name: Option<String>,
    pub version: Option<String>,
    pub modality: Option<String>,
    pub slice_count: i32,
    pub mask_type: String,
    pub description: Option<String>,
    pub created_by: Option<i32>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct SignedUrlRequest {
    /// 파일명
    #[schema(example = "0001_liver.png")]
    pub filename: String,
    
    /// MIME 타입
    #[schema(example = "image/png")]
    pub mime_type: String,
    
    /// 파일 크기 (바이트)
    #[schema(example = 1024000)]
    pub file_size: Option<i64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SignedUrlResponse {
    /// 업로드용 Signed URL
    pub upload_url: String,
    
    /// S3 파일 경로
    pub file_path: String,
    
    /// 만료 시간 (초)
    pub expires_in: u64,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CompleteUploadRequest {
    /// 실제 업로드된 슬라이스 수
    #[schema(example = 120)]
    pub slice_count: i32,
    
    /// 라벨 목록
    #[schema(example = ["liver", "spleen"])]
    pub labels: Vec<String>,
}
```

### 3.2 Object Storage Service
```rust
// src/application/services/object_storage_service.rs
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SignedUrlConfig {
    pub bucket_name: String,
    pub region: String,
    pub ttl_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadedFile {
    pub file_path: String,
    pub file_size: i64,
    pub checksum: Option<String>,
}

#[async_trait]
pub trait ObjectStorageService: Send + Sync {
    async fn generate_upload_url(
        &self,
        file_path: &str,
        mime_type: &str,
        ttl_seconds: u64,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;
    
    async fn generate_download_url(
        &self,
        file_path: &str,
        ttl_seconds: u64,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;
    
    async fn delete_file(&self, file_path: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    
    async fn get_file_metadata(
        &self,
        file_path: &str,
    ) -> Result<UploadedFile, Box<dyn std::error::Error + Send + Sync>>;
}
```

## 🔧 4. Infrastructure Layer 구현

### 4.1 S3 Service 구현
```rust
// src/infrastructure/external/s3_service.rs
use aws_sdk_s3::Client as S3Client;
use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::types::{ObjectCannedAcl, StorageClass};
use std::time::Duration;
use crate::application::services::{ObjectStorageService, UploadedFile, SignedUrlConfig};

pub struct S3ObjectStorageService {
    client: S3Client,
    config: SignedUrlConfig,
}

impl S3ObjectStorageService {
    pub fn new(client: S3Client, config: SignedUrlConfig) -> Self {
        Self { client, config }
    }
}

#[async_trait]
impl ObjectStorageService for S3ObjectStorageService {
    async fn generate_upload_url(
        &self,
        file_path: &str,
        mime_type: &str,
        ttl_seconds: u64,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let presigning_config = PresigningConfig::expires_in(Duration::from_secs(ttl_seconds))
            .map_err(|e| format!("Failed to create presigning config: {}", e))?;

        let request = self
            .client
            .put_object()
            .bucket(&self.config.bucket_name)
            .key(file_path)
            .content_type(mime_type)
            .set_acl(Some(ObjectCannedAcl::Private))
            .set_storage_class(Some(StorageClass::Standard))
            .presigned(presigning_config)
            .await
            .map_err(|e| format!("Failed to generate presigned URL: {}", e))?;

        Ok(request.uri().to_string())
    }

    async fn generate_download_url(
        &self,
        file_path: &str,
        ttl_seconds: u64,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let presigning_config = PresigningConfig::expires_in(Duration::from_secs(ttl_seconds))
            .map_err(|e| format!("Failed to create presigning config: {}", e))?;

        let request = self
            .client
            .get_object()
            .bucket(&self.config.bucket_name)
            .key(file_path)
            .presigned(presigning_config)
            .await
            .map_err(|e| format!("Failed to generate presigned URL: {}", e))?;

        Ok(request.uri().to_string())
    }

    async fn delete_file(&self, file_path: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.client
            .delete_object()
            .bucket(&self.config.bucket_name)
            .key(file_path)
            .send()
            .await
            .map_err(|e| format!("Failed to delete file: {}", e))?;

        Ok(())
    }

    async fn get_file_metadata(
        &self,
        file_path: &str,
    ) -> Result<UploadedFile, Box<dyn std::error::Error + Send + Sync>> {
        let response = self
            .client
            .head_object()
            .bucket(&self.config.bucket_name)
            .key(file_path)
            .send()
            .await
            .map_err(|e| format!("Failed to get file metadata: {}", e))?;

        Ok(UploadedFile {
            file_path: file_path.to_string(),
            file_size: response.content_length().unwrap_or(0),
            checksum: response.e_tag().map(|s| s.to_string()),
        })
    }
}
```

### 4.2 Repository 구현
```rust
// src/infrastructure/repositories/mask_group_repository_impl.rs
use async_trait::async_trait;
use sqlx::PgPool;
use crate::domain::entities::{MaskGroup, NewMaskGroup};
use crate::domain::repositories::MaskGroupRepository;

pub struct MaskGroupRepositoryImpl {
    pool: PgPool,
}

impl MaskGroupRepositoryImpl {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MaskGroupRepository for MaskGroupRepositoryImpl {
    async fn create(&self, new_mask_group: NewMaskGroup) -> Result<MaskGroup, sqlx::Error> {
        sqlx::query_as::<_, MaskGroup>(
            "INSERT INTO annotation_mask_group (annotation_id, group_name, model_name, version, modality, slice_count, mask_type, description, created_by)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
             RETURNING id, annotation_id, group_name, model_name, version, modality, slice_count, mask_type, description, created_by, created_at"
        )
        .bind(new_mask_group.annotation_id)
        .bind(new_mask_group.group_name)
        .bind(new_mask_group.model_name)
        .bind(new_mask_group.version)
        .bind(new_mask_group.modality)
        .bind(new_mask_group.slice_count)
        .bind(new_mask_group.mask_type)
        .bind(new_mask_group.description)
        .bind(new_mask_group.created_by)
        .fetch_one(&self.pool)
        .await
    }

    async fn find_by_id(&self, id: i32) -> Result<Option<MaskGroup>, sqlx::Error> {
        sqlx::query_as::<_, MaskGroup>(
            "SELECT id, annotation_id, group_name, model_name, version, modality, slice_count, mask_type, description, created_by, created_at
             FROM annotation_mask_group WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    async fn find_by_annotation_id(&self, annotation_id: i32) -> Result<Vec<MaskGroup>, sqlx::Error> {
        sqlx::query_as::<_, MaskGroup>(
            "SELECT id, annotation_id, group_name, model_name, version, modality, slice_count, mask_type, description, created_by, created_at
             FROM annotation_mask_group WHERE annotation_id = $1 ORDER BY created_at DESC"
        )
        .bind(annotation_id)
        .fetch_all(&self.pool)
        .await
    }

    async fn update(&self, id: i32, mask_group: MaskGroup) -> Result<Option<MaskGroup>, sqlx::Error> {
        sqlx::query_as::<_, MaskGroup>(
            "UPDATE annotation_mask_group 
             SET group_name = $2, model_name = $3, version = $4, modality = $5, slice_count = $6, mask_type = $7, description = $8
             WHERE id = $1
             RETURNING id, annotation_id, group_name, model_name, version, modality, slice_count, mask_type, description, created_by, created_at"
        )
        .bind(id)
        .bind(mask_group.group_name)
        .bind(mask_group.model_name)
        .bind(mask_group.version)
        .bind(mask_group.modality)
        .bind(mask_group.slice_count)
        .bind(mask_group.mask_type)
        .bind(mask_group.description)
        .fetch_optional(&self.pool)
        .await
    }

    async fn delete(&self, id: i32) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM annotation_mask_group WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        
        Ok(result.rows_affected() > 0)
    }
}
```

## 🎮 5. Presentation Layer 구현

### 5.1 Mask Group Controller
```rust
// src/presentation/controllers/mask_group_controller.rs
use actix_web::{web, HttpResponse, Result};
use utoipa::OpenApi;
use crate::application::dto::mask_group_dto::*;
use crate::application::use_cases::mask_group_use_case::MaskGroupUseCase;

#[utoipa::path(
    post,
    path = "/api/annotations/{annotation_id}/mask-groups",
    tag = "Mask Groups",
    params(
        ("annotation_id" = i32, Path, description = "Annotation ID")
    ),
    request_body = CreateMaskGroupRequest,
    responses(
        (status = 201, description = "Mask group created successfully", body = MaskGroupResponse),
        (status = 400, description = "Bad request"),
        (status = 404, description = "Annotation not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn create_mask_group(
    annotation_id: web::Path<i32>,
    request: web::Json<CreateMaskGroupRequest>,
    use_case: web::Data<MaskGroupUseCase>,
) -> Result<HttpResponse> {
    let annotation_id = annotation_id.into_inner();
    
    match use_case.create_mask_group(annotation_id, request.into_inner()).await {
        Ok(mask_group) => Ok(HttpResponse::Created().json(mask_group)),
        Err(e) => {
            log::error!("Failed to create mask group: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to create mask group",
                "message": e.to_string()
            })))
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/annotations/{annotation_id}/mask-groups",
    tag = "Mask Groups",
    params(
        ("annotation_id" = i32, Path, description = "Annotation ID")
    ),
    responses(
        (status = 200, description = "Mask groups retrieved successfully", body = Vec<MaskGroupResponse>),
        (status = 404, description = "Annotation not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn list_mask_groups(
    annotation_id: web::Path<i32>,
    use_case: web::Data<MaskGroupUseCase>,
) -> Result<HttpResponse> {
    let annotation_id = annotation_id.into_inner();
    
    match use_case.list_mask_groups(annotation_id).await {
        Ok(mask_groups) => Ok(HttpResponse::Ok().json(mask_groups)),
        Err(e) => {
            log::error!("Failed to list mask groups: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to list mask groups",
                "message": e.to_string()
            })))
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/annotations/{annotation_id}/mask-groups/{group_id}/signed-url",
    tag = "Mask Groups",
    params(
        ("annotation_id" = i32, Path, description = "Annotation ID"),
        ("group_id" = i32, Path, description = "Mask Group ID")
    ),
    request_body = SignedUrlRequest,
    responses(
        (status = 200, description = "Signed URL generated successfully", body = SignedUrlResponse),
        (status = 400, description = "Bad request"),
        (status = 404, description = "Mask group not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn generate_signed_url(
    path: web::Path<(i32, i32)>,
    request: web::Json<SignedUrlRequest>,
    use_case: web::Data<MaskGroupUseCase>,
) -> Result<HttpResponse> {
    let (annotation_id, group_id) = path.into_inner();
    
    match use_case.generate_signed_url(annotation_id, group_id, request.into_inner()).await {
        Ok(signed_url_response) => Ok(HttpResponse::Ok().json(signed_url_response)),
        Err(e) => {
            log::error!("Failed to generate signed URL: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to generate signed URL",
                "message": e.to_string()
            })))
        }
    }
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/annotations/{annotation_id}/mask-groups")
            .route("", web::post().to(create_mask_group))
            .route("", web::get().to(list_mask_groups))
            .route("/{group_id}", web::get().to(get_mask_group))
            .route("/{group_id}", web::delete().to(delete_mask_group))
            .route("/{group_id}/signed-url", web::post().to(generate_signed_url))
            .route("/{group_id}/complete", web::post().to(complete_upload))
    );
}
```

## ⚙️ 6. 설정 파일 업데이트

### 6.1 Cargo.toml 의존성 추가
```toml
[dependencies]
aws-sdk-s3 = "1.0"
aws-config = "1.0"
tokio-util = { version = "0.7", features = ["codec"] }
```

### 6.2 설정 파일
```toml
# config/default.toml
[object_storage]
provider = "s3" # or "minio"
bucket_name = "pacs-masks"
region = "us-east-1"
endpoint = "" # MinIO용
access_key = ""
secret_key = ""

[signed_url]
default_ttl = 600 # 10분
max_ttl = 3600 # 1시간
```

## 🧪 7. 테스트 예시

### 7.1 통합 테스트
```rust
// tests/mask_group_controller_test.rs
use actix_web::{test, web, App};
use pacs_server::presentation::controllers::mask_group_controller::configure_routes;

#[actix_web::test]
async fn test_create_mask_group() {
    let app = test::init_service(
        App::new()
            .configure(configure_routes)
            .app_data(web::Data::new(use_case))
    ).await;

    let request = serde_json::json!({
        "group_name": "Test_Liver_Segmentation",
        "model_name": "monai_unet",
        "version": "v1.0.0",
        "modality": "CT",
        "slice_count": 120,
        "mask_type": "segmentation",
        "description": "Test segmentation"
    });

    let req = test::TestRequest::post()
        .uri("/api/annotations/1/mask-groups")
        .set_json(&request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);
}
```

## 🚀 8. 실행 순서

1. **의존성 추가**: `Cargo.toml`에 AWS SDK 추가
2. **마이그레이션 실행**: DB 스키마 생성
3. **엔티티 생성**: Domain layer 구현
4. **Repository 구현**: Infrastructure layer 구현
5. **Service 구현**: Object Storage 연동
6. **Use Case 구현**: Application layer 구현
7. **Controller 구현**: Presentation layer 구현
8. **테스트 작성**: 단위/통합 테스트
9. **설정 적용**: 환경 변수 및 설정 파일
10. **배포**: 프로덕션 환경 배포

---

**작성일**: 2025-10-07  
**작성자**: AI Assistant  
**버전**: 1.0
