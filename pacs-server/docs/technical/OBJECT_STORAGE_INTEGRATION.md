# ☁️ PACS 마스크 업로드 Object Storage 연동

## 📋 개요
PACS 마스크 업로드 시스템을 위한 Object Storage (S3/MinIO) 연동 구현 문서입니다.

## 🏗️ 아키텍처 설계

### 1. 서비스 계층 구조
```
ObjectStorageService (Trait)
├── S3ObjectStorageService (AWS S3 구현체)
├── MinIOObjectStorageService (MinIO 구현체)
└── ObjectStorageServiceFactory (팩토리 패턴)
```

### 2. 의존성 구조
```toml
[dependencies]
aws-sdk-s3 = "1.0"
aws-config = "1.0"
tokio-util = { version = "0.7", features = ["codec"] }
thiserror = "1.0"
```

## 🔧 핵심 컴포넌트

### 1. ObjectStorageService Trait
```rust
#[async_trait]
pub trait ObjectStorageService: Send + Sync {
    async fn upload_file(
        &self,
        bucket_name: &str,
        file_path: &str,
        data: &[u8],
        content_type: Option<&str>,
    ) -> Result<UploadedFile, ObjectStorageError>;

    async fn download_file(
        &self,
        bucket_name: &str,
        file_path: &str,
    ) -> Result<Vec<u8>, ObjectStorageError>;

    async fn delete_file(
        &self,
        bucket_name: &str,
        file_path: &str,
    ) -> Result<(), ObjectStorageError>;

    async fn generate_signed_url(
        &self,
        bucket_name: &str,
        file_path: &str,
        operation: SignedUrlOperation,
        options: SignedUrlOptions,
    ) -> Result<String, ObjectStorageError>;

    async fn file_exists(
        &self,
        bucket_name: &str,
        file_path: &str,
    ) -> Result<bool, ObjectStorageError>;
}
```

### 2. 에러 처리
```rust
#[derive(Debug, thiserror::Error)]
pub enum ObjectStorageError {
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Upload failed: {0}")]
    UploadError(String),
    
    #[error("Download failed: {0}")]
    DownloadError(String),
    
    #[error("Delete failed: {0}")]
    DeleteError(String),
    
    #[error("Signed URL generation failed: {0}")]
    SignedUrlError(String),
    
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
}
```

### 3. 데이터 구조체
```rust
pub struct UploadedFile {
    pub file_path: String,
    pub file_size: u64,
    pub content_type: String,
    pub etag: String,
    pub last_modified: DateTime<Utc>,
}

pub struct SignedUrlOptions {
    pub ttl_seconds: u64,
    pub content_type: Option<String>,
    pub content_disposition: Option<String>,
    pub metadata: HashMap<String, String>,
    pub acl: Option<String>,
}

pub enum SignedUrlOperation {
    Get,
    Put,
    Delete,
}
```

## 🚀 구현체 상세

### 1. S3ObjectStorageService
AWS S3를 위한 구현체입니다.

#### 주요 기능
- AWS SDK v1.0 사용
- 자동 리전 감지
- 버킷 존재 확인
- 에러 매핑

#### 설정 예시
```rust
let s3_service = S3ObjectStorageService::new(
    "my-bucket",
    "us-east-1",
).await?;
```

### 2. MinIOObjectStorageService
MinIO를 위한 S3 호환 구현체입니다.

#### 주요 기능
- S3 호환 API 사용
- 커스텀 엔드포인트 지원
- 로컬 개발 환경 최적화

#### 설정 예시
```rust
let minio_service = MinIOObjectStorageService::new(
    "my-bucket",
    "us-east-1",
    "http://localhost:9000",
).await?;
```

### 3. ObjectStorageServiceFactory
설정에 따라 적절한 서비스를 생성하는 팩토리입니다.

```rust
impl ObjectStorageServiceFactory {
    pub async fn create(
        provider: &str,
        bucket_name: &str,
        region: &str,
        endpoint: Option<&str>,
        access_key: Option<&str>,
        secret_key: Option<&str>,
    ) -> Result<Box<dyn ObjectStorageService>, ObjectStorageError> {
        match provider {
            "s3" => {
                let service = S3ObjectStorageService::new(
                    bucket_name,
                    region,
                ).await?;
                Ok(Box::new(service))
            }
            "minio" => {
                let service = MinIOObjectStorageService::new(
                    bucket_name,
                    region,
                    endpoint.unwrap_or("http://localhost:9000"),
                ).await?;
                Ok(Box::new(service))
            }
            _ => Err(ObjectStorageError::ConfigError(
                format!("Unsupported provider: {}", provider)
            ))
        }
    }
}
```

## ⚙️ 설정 관리

### 1. 환경 변수 설정
```bash
# AWS S3 설정
AWS_ACCESS_KEY_ID=your_access_key
AWS_SECRET_ACCESS_KEY=your_secret_key
S3_BUCKET_NAME=pacs-masks
S3_REGION=us-east-1

# MinIO 설정 (로컬 개발용)
MINIO_ENDPOINT=http://localhost:9000
MINIO_ACCESS_KEY=minioadmin
MINIO_SECRET_KEY=minioadmin
```

### 2. 설정 파일 (config/default.toml)
```toml
[object_storage]
provider = "s3"  # or "minio"
bucket_name = "pacs-masks"
region = "us-east-1"
endpoint = ""  # MinIO용 (AWS S3는 빈 문자열)
access_key = ""
secret_key = ""

[signed_url]
default_ttl = 600  # 10분
max_ttl = 3600     # 1시간
```

## 🔐 보안 고려사항

### 1. IAM 정책 예시
```json
{
    "Version": "2012-10-17",
    "Statement": [
        {
            "Effect": "Allow",
            "Action": [
                "s3:GetObject",
                "s3:PutObject",
                "s3:DeleteObject"
            ],
            "Resource": "arn:aws:s3:::pacs-masks/annotations/*"
        },
        {
            "Effect": "Allow",
            "Action": "s3:ListBucket",
            "Resource": "arn:aws:s3:::pacs-masks"
        }
    ]
}
```

### 2. Signed URL 보안
- TTL 제한 (기본 10분, 최대 1시간)
- 특정 경로 prefix 제한
- HTTPS 강제
- 적절한 권한 설정

### 3. 파일 업로드 보안
- 파일 타입 검증
- 파일 크기 제한
- 악성 파일 스캔 (향후 구현)
- 개인정보 포함 파일명 금지

## 📊 성능 최적화

### 1. 병렬 업로드
```rust
// 여러 파일을 병렬로 업로드
let upload_tasks: Vec<_> = files.into_iter()
    .map(|file| {
        let service = service.clone();
        tokio::spawn(async move {
            service.upload_file(
                bucket_name,
                &file.path,
                &file.data,
                Some(&file.content_type),
            ).await
        })
    })
    .collect();

let results = futures::future::join_all(upload_tasks).await;
```

### 2. 청크 업로드 (향후 구현)
- 대용량 파일을 작은 청크로 분할
- 멀티파트 업로드 지원
- 실패 시 재시도 로직

### 3. 캐싱 전략
- 메타데이터 캐싱
- Signed URL 캐싱 (짧은 TTL)
- CDN 연동 (향후 구현)

## 🧪 테스트 전략

### 1. 단위 테스트
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;
    
    #[tokio::test]
    async fn test_upload_file() {
        // Mock S3 서버 설정
        let mut server = Server::new_async().await;
        let mock = server.mock("PUT", "/test-bucket/test-file")
            .with_status(200)
            .create();
            
        // 테스트 실행
        let service = S3ObjectStorageService::new("test-bucket", "us-east-1").await.unwrap();
        let result = service.upload_file("test-bucket", "test-file", b"test data", None).await;
        
        assert!(result.is_ok());
        mock.assert();
    }
}
```

### 2. 통합 테스트
- 실제 S3/MinIO 환경에서 테스트
- 네트워크 오류 시뮬레이션
- 대용량 파일 업로드 테스트

## 📈 모니터링 및 로깅

### 1. 메트릭 수집
- 업로드/다운로드 성공률
- 평균 응답 시간
- 에러 발생률
- 저장소 사용량

### 2. 로그 레벨
```rust
// 업로드 성공 로그
info!("File uploaded successfully: bucket={}, path={}, size={}", 
      bucket_name, file_path, file_size);

// 에러 로그
error!("Upload failed: bucket={}, path={}, error={}", 
       bucket_name, file_path, error);
```

## 🔄 마이그레이션 전략

### 1. 기존 파일 마이그레이션
- 기존 파일 시스템에서 Object Storage로 이전
- 배치 처리로 점진적 마이그레이션
- 데이터 무결성 검증

### 2. 롤백 계획
- 기존 파일 시스템 백업 유지
- 설정 변경으로 쉽게 전환 가능
- 데이터 손실 방지

## 📚 참고 자료
- [AWS S3 Rust SDK 문서](https://docs.rs/aws-sdk-s3/latest/aws_sdk_s3/)
- [MinIO Rust 클라이언트](https://docs.min.io/docs/rust-client-quickstart-guide.html)
- [AWS S3 Signed URL 가이드](https://docs.aws.amazon.com/AmazonS3/latest/userguide/PresignedUrlUploadObject.html)

---
**작성일**: 2025-10-07  
**작성자**: AI Assistant  
**버전**: 1.0
