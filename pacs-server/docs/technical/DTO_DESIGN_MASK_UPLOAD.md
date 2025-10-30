# 📋 PACS 마스크 업로드 DTO 설계

## 📋 개요
PACS 마스크 업로드 시스템을 위한 Data Transfer Object (DTO) 설계 문서입니다. API 요청/응답과 내부 데이터 구조 간의 변환을 담당합니다.

## 🏗️ DTO 계층 구조

### 1. 마스크 그룹 관련 DTO
```
MaskGroup DTOs
├── CreateMaskGroupRequest
├── UpdateMaskGroupRequest
├── MaskGroupResponse
├── MaskGroupListResponse
├── MaskGroupDetailResponse
├── SignedUrlRequest
├── SignedUrlResponse
├── CompleteUploadRequest
└── CompleteUploadResponse
```

### 2. 마스크 관련 DTO
```
Mask DTOs
├── MaskResponse
├── CreateMaskRequest
├── UpdateMaskRequest
├── ListMasksRequest
├── MaskListResponse
├── DownloadUrlRequest
├── DownloadUrlResponse
└── MaskStatsResponse
```

## 🔧 마스크 그룹 DTO 상세

### 1. CreateMaskGroupRequest
마스크 그룹 생성 요청 DTO입니다.

```rust
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateMaskGroupRequest {
    /// 마스크 그룹 이름
    #[schema(example = "Liver Segmentation v1.0")]
    pub group_name: Option<String>,
    
    /// AI 모델 이름
    #[schema(example = "UNet3D")]
    pub model_name: Option<String>,
    
    /// 모델 버전
    #[schema(example = "1.0.0")]
    pub version: Option<String>,
    
    /// 의료 영상 모달리티
    #[schema(example = "CT")]
    pub modality: Option<String>,
    
    /// 슬라이스 개수
    #[schema(example = 100)]
    pub slice_count: Option<i32>,
    
    /// 마스크 타입
    #[schema(example = "segmentation")]
    pub mask_type: Option<String>,
    
    /// 그룹 설명
    #[schema(example = "간 분할을 위한 AI 모델 결과")]
    pub description: Option<String>,
}
```

### 2. MaskGroupResponse
마스크 그룹 응답 DTO입니다.

```rust
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MaskGroupResponse {
    /// 마스크 그룹 ID
    #[schema(example = 1)]
    pub id: i32,
    
    /// 연결된 어노테이션 ID
    #[schema(example = 123)]
    pub annotation_id: i32,
    
    /// 마스크 그룹 이름
    #[schema(example = "Liver Segmentation v1.0")]
    pub group_name: Option<String>,
    
    /// AI 모델 이름
    #[schema(example = "UNet3D")]
    pub model_name: Option<String>,
    
    /// 모델 버전
    #[schema(example = "1.0.0")]
    pub version: Option<String>,
    
    /// 의료 영상 모달리티
    #[schema(example = "CT")]
    pub modality: Option<String>,
    
    /// 슬라이스 개수
    #[schema(example = 100)]
    pub slice_count: Option<i32>,
    
    /// 마스크 타입
    #[schema(example = "segmentation")]
    pub mask_type: Option<String>,
    
    /// 그룹 설명
    #[schema(example = "간 분할을 위한 AI 모델 결과")]
    pub description: Option<String>,
    
    /// 생성자 ID
    #[schema(example = 1)]
    pub created_by: Option<i32>,
    
    /// 생성 시간
    #[schema(example = "2025-10-07T10:30:00Z")]
    pub created_at: DateTime<Utc>,
    
    /// 수정 시간
    #[schema(example = "2025-10-07T10:30:00Z")]
    pub updated_at: DateTime<Utc>,
}
```

### 3. SignedUrlRequest
Signed URL 발급 요청 DTO입니다.

```rust
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SignedUrlRequest {
    /// 파일 경로
    #[schema(example = "annotations/123/masks/slice_001.png")]
    pub file_path: String,
    
    /// TTL (초)
    #[schema(example = 600)]
    pub ttl_seconds: Option<u64>,
    
    /// 콘텐츠 타입
    #[schema(example = "image/png")]
    pub content_type: Option<String>,
    
    /// 콘텐츠 디스포지션
    #[schema(example = "attachment; filename=\"slice_001.png\"")]
    pub content_disposition: Option<String>,
    
    /// 메타데이터
    #[schema(example = "{\"slice_index\": 1, \"label_name\": \"liver\"}")]
    pub metadata: Option<HashMap<String, String>>,
    
    /// ACL 설정
    #[schema(example = "private")]
    pub acl: Option<String>,
    
    /// 어노테이션 ID
    #[schema(example = 123)]
    pub annotation_id: Option<i32>,
    
    /// 사용자 ID
    #[schema(example = 1)]
    pub user_id: Option<i32>,
    
    /// 마스크 그룹 ID
    #[schema(example = 1)]
    pub mask_group_id: Option<i32>,
    
    /// 슬라이스 인덱스
    #[schema(example = 1)]
    pub slice_index: Option<i32>,
}
```

### 4. SignedUrlResponse
Signed URL 응답 DTO입니다.

```rust
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SignedUrlResponse {
    /// Signed URL
    #[schema(example = "https://s3.amazonaws.com/bucket/file?X-Amz-Signature=...")]
    pub signed_url: String,
    
    /// 파일 경로
    #[schema(example = "annotations/123/masks/slice_001.png")]
    pub file_path: String,
    
    /// 만료 시간
    #[schema(example = "2025-10-07T10:40:00Z")]
    pub expires_at: DateTime<Utc>,
    
    /// HTTP 메서드
    #[schema(example = "PUT")]
    pub method: String,
    
    /// 콘텐츠 타입
    #[schema(example = "image/png")]
    pub content_type: Option<String>,
    
    /// 추가 헤더
    #[schema(example = "{\"Content-Disposition\": \"attachment\"}")]
    pub headers: Option<HashMap<String, String>>,
}
```

## 🔧 마스크 DTO 상세

### 1. MaskResponse
마스크 응답 DTO입니다.

```rust
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MaskResponse {
    /// 마스크 ID
    #[schema(example = 1)]
    pub id: i32,
    
    /// 연결된 마스크 그룹 ID
    #[schema(example = 1)]
    pub mask_group_id: i32,
    
    /// 슬라이스 인덱스
    #[schema(example = 1)]
    pub slice_index: Option<i32>,
    
    /// DICOM SOP Instance UID
    #[schema(example = "1.2.3.4.5.6.7.8.9.10")]
    pub sop_instance_uid: Option<String>,
    
    /// 라벨 이름
    #[schema(example = "liver")]
    pub label_name: Option<String>,
    
    /// 파일 경로
    #[schema(example = "annotations/123/masks/slice_001.png")]
    pub file_path: String,
    
    /// MIME 타입
    #[schema(example = "image/png")]
    pub mime_type: Option<String>,
    
    /// 파일 크기 (바이트)
    #[schema(example = 1024000)]
    pub file_size: Option<i64>,
    
    /// 파일 체크섬
    #[schema(example = "sha256:abcd1234...")]
    pub checksum: Option<String>,
    
    /// 이미지 너비
    #[schema(example = 512)]
    pub width: Option<i32>,
    
    /// 이미지 높이
    #[schema(example = 512)]
    pub height: Option<i32>,
    
    /// 생성 시간
    #[schema(example = "2025-10-07T10:30:00Z")]
    pub created_at: DateTime<Utc>,
    
    /// 수정 시간
    #[schema(example = "2025-10-07T10:30:00Z")]
    pub updated_at: DateTime<Utc>,
}
```

### 2. CreateMaskRequest
마스크 생성 요청 DTO입니다.

```rust
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateMaskRequest {
    /// 슬라이스 인덱스
    #[schema(example = 1)]
    pub slice_index: Option<i32>,
    
    /// DICOM SOP Instance UID
    #[schema(example = "1.2.3.4.5.6.7.8.9.10")]
    pub sop_instance_uid: Option<String>,
    
    /// 라벨 이름
    #[schema(example = "liver")]
    pub label_name: Option<String>,
    
    /// 파일 경로
    #[schema(example = "annotations/123/masks/slice_001.png")]
    pub file_path: String,
    
    /// MIME 타입
    #[schema(example = "image/png")]
    pub mime_type: Option<String>,
    
    /// 파일 크기 (바이트)
    #[schema(example = 1024000)]
    pub file_size: Option<i64>,
    
    /// 파일 체크섬
    #[schema(example = "sha256:abcd1234...")]
    pub checksum: Option<String>,
    
    /// 이미지 너비
    #[schema(example = 512)]
    pub width: Option<i32>,
    
    /// 이미지 높이
    #[schema(example = 512)]
    pub height: Option<i32>,
}
```

## 📊 통계 및 목록 DTO

### 1. MaskGroupListResponse
마스크 그룹 목록 응답 DTO입니다.

```rust
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MaskGroupListResponse {
    /// 마스크 그룹 목록
    pub mask_groups: Vec<MaskGroupResponse>,
    
    /// 총 개수
    #[schema(example = 100)]
    pub total_count: i64,
    
    /// 현재 페이지
    #[schema(example = 1)]
    pub page: i64,
    
    /// 페이지 크기
    #[schema(example = 20)]
    pub page_size: i64,
    
    /// 총 페이지 수
    #[schema(example = 5)]
    pub total_pages: i64,
}
```

### 2. MaskStatsResponse
마스크 통계 응답 DTO입니다.

```rust
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MaskStatsResponse {
    /// 총 마스크 개수
    #[schema(example = 1000)]
    pub total_masks: i64,
    
    /// 총 파일 크기 (바이트)
    #[schema(example = 1024000000)]
    pub total_size_bytes: i64,
    
    /// MIME 타입별 개수
    #[schema(example = "{\"image/png\": 800, \"image/jpeg\": 200}")]
    pub mime_types: HashMap<String, i64>,
    
    /// 라벨별 개수
    #[schema(example = "{\"liver\": 500, \"lung\": 300, \"heart\": 200}")]
    pub label_names: HashMap<String, i64>,
    
    /// 평균 파일 크기
    #[schema(example = 1024000.0)]
    pub average_file_size: f64,
    
    /// 최대 파일 크기
    #[schema(example = 2048000)]
    pub largest_file_size: i64,
    
    /// 최소 파일 크기
    #[schema(example = 512000)]
    pub smallest_file_size: i64,
}
```

## 🔄 변환 로직

### 1. Entity → DTO 변환
```rust
impl From<MaskGroup> for MaskGroupResponse {
    fn from(entity: MaskGroup) -> Self {
        Self {
            id: entity.id,
            annotation_id: entity.annotation_id,
            group_name: entity.group_name,
            model_name: entity.model_name,
            version: entity.version,
            modality: entity.modality,
            slice_count: entity.slice_count,
            mask_type: entity.mask_type,
            description: entity.description,
            created_by: entity.created_by,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
        }
    }
}
```

### 2. DTO → Entity 변환
```rust
impl From<CreateMaskGroupRequest> for NewMaskGroup {
    fn from(dto: CreateMaskGroupRequest) -> Self {
        Self {
            annotation_id: 0, // API에서 설정
            group_name: dto.group_name,
            model_name: dto.model_name,
            version: dto.version,
            modality: dto.modality,
            slice_count: dto.slice_count,
            mask_type: dto.mask_type,
            description: dto.description,
            created_by: None, // 인증에서 설정
        }
    }
}
```

## 🧪 검증 로직

### 1. 입력 검증
```rust
impl CreateMaskGroupRequest {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if let Some(ref group_name) = self.group_name {
            if group_name.len() > 255 {
                return Err(ValidationError::new("group_name too long"));
            }
        }
        
        if let Some(ref model_name) = self.model_name {
            if model_name.len() > 255 {
                return Err(ValidationError::new("model_name too long"));
            }
        }
        
        if let Some(slice_count) = self.slice_count {
            if slice_count < 1 || slice_count > 10000 {
                return Err(ValidationError::new("slice_count out of range"));
            }
        }
        
        Ok(())
    }
}
```

### 2. 비즈니스 규칙 검증
```rust
impl SignedUrlRequest {
    pub fn validate_ttl(&self) -> Result<(), ValidationError> {
        if let Some(ttl) = self.ttl_seconds {
            if ttl < 60 || ttl > 3600 {
                return Err(ValidationError::new("TTL must be between 60 and 3600 seconds"));
            }
        }
        Ok(())
    }
    
    pub fn validate_file_path(&self) -> Result<(), ValidationError> {
        if self.file_path.is_empty() {
            return Err(ValidationError::new("file_path cannot be empty"));
        }
        
        if self.file_path.contains("..") {
            return Err(ValidationError::new("file_path cannot contain '..'"));
        }
        
        Ok(())
    }
}
```

## 📚 Swagger 문서화

### 1. ToSchema 구현
모든 DTO는 `ToSchema` trait을 구현하여 Swagger 문서에 자동으로 포함됩니다.

```rust
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateMaskGroupRequest {
    // ... 필드들
}
```

### 2. 예시 데이터
```rust
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateMaskGroupRequest {
    /// 마스크 그룹 이름
    #[schema(example = "Liver Segmentation v1.0")]
    pub group_name: Option<String>,
    // ... 다른 필드들
}
```

## 🔒 보안 고려사항

### 1. 입력 검증
- 길이 제한
- 특수 문자 필터링
- SQL 인젝션 방지

### 2. 민감한 정보 제외
- 내부 ID는 응답에만 포함
- 사용자 인증 정보는 별도 처리

### 3. 권한 검증
- 사용자별 데이터 접근 제어
- 어노테이션 소유권 확인

## 📈 성능 최적화

### 1. 직렬화 최적화
- 필요한 필드만 직렬화
- 중첩 객체 최소화

### 2. 메모리 사용량
- 큰 데이터는 스트리밍 처리
- 캐싱 전략 적용

## 📚 참고 자료
- [Serde 문서](https://serde.rs/)
- [Utoipa 문서](https://docs.rs/utoipa/latest/utoipa/)
- [Rust DTO 패턴](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html)

---
**작성일**: 2025-10-07  
**작성자**: AI Assistant  
**버전**: 1.0
