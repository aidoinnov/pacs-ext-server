use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Object Storage 서비스 에러 타입
#[derive(Debug, thiserror::Error)]
pub enum ObjectStorageError {
    #[error("S3 operation failed: {0}")]
    S3Error(String),
    
    #[error("MinIO operation failed: {0}")]
    MinIOError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
}

/// 업로드된 파일 정보
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadedFile {
    pub file_path: String,
    pub file_size: i64,
    pub checksum: Option<String>,
    pub mime_type: Option<String>,
    pub last_modified: Option<String>,
}

/// Signed URL 생성 옵션
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedUrlOptions {
    pub ttl_seconds: u64,
    pub content_type: Option<String>,
    pub content_disposition: Option<String>,
    pub metadata: Option<std::collections::HashMap<String, String>>,
}

impl Default for SignedUrlOptions {
    fn default() -> Self {
        Self {
            ttl_seconds: 600, // 10 minutes
            content_type: None,
            content_disposition: None,
            metadata: None,
        }
    }
}

/// Object Storage 서비스 trait
#[async_trait]
pub trait ObjectStorageService: Send + Sync {
    /// 업로드용 Signed URL 생성
    async fn generate_upload_url(
        &self,
        file_path: &str,
        options: SignedUrlOptions,
    ) -> Result<String, ObjectStorageError>;
    
    /// 다운로드용 Signed URL 생성
    async fn generate_download_url(
        &self,
        file_path: &str,
        ttl_seconds: u64,
    ) -> Result<String, ObjectStorageError>;
    
    /// 파일 삭제
    async fn delete_file(&self, file_path: &str) -> Result<(), ObjectStorageError>;
    
    /// 파일 메타데이터 조회
    async fn get_file_metadata(
        &self,
        file_path: &str,
    ) -> Result<UploadedFile, ObjectStorageError>;
    
    /// 파일 존재 여부 확인
    async fn file_exists(&self, file_path: &str) -> Result<bool, ObjectStorageError>;
    
    /// 파일 목록 조회 (prefix 기반)
    async fn list_files(
        &self,
        prefix: &str,
        max_keys: Option<i32>,
    ) -> Result<Vec<String>, ObjectStorageError>;
    
    /// 파일 복사
    async fn copy_file(
        &self,
        source_path: &str,
        destination_path: &str,
    ) -> Result<(), ObjectStorageError>;
    
    /// 파일 이동 (복사 후 삭제)
    async fn move_file(
        &self,
        source_path: &str,
        destination_path: &str,
    ) -> Result<(), ObjectStorageError>;
}

/// Object Storage 서비스 팩토리
pub struct ObjectStorageServiceFactory;

impl ObjectStorageServiceFactory {
    /// 설정에 따라 적절한 Object Storage 서비스 생성
    pub async fn create(
        provider: &str,
        bucket_name: &str,
        region: &str,
        endpoint: &str,
        access_key: &str,
        secret_key: &str,
    ) -> Result<Box<dyn ObjectStorageService>, ObjectStorageError> {
        match provider.to_lowercase().as_str() {
            "s3" => {
                let s3_service = crate::infrastructure::external::s3_service::S3ObjectStorageService::new(
                    bucket_name,
                    region,
                    access_key,
                    secret_key,
                ).await?;
                Ok(Box::new(s3_service))
            }
            _ => Err(ObjectStorageError::ConfigError(
                format!("Unsupported object storage provider: {}. Only 's3' is supported.", provider)
            ))
        }
    }
}

/// Object Storage 서비스 빌더
pub struct ObjectStorageServiceBuilder {
    provider: String,
    bucket_name: String,
    region: String,
    endpoint: String,
    access_key: String,
    secret_key: String,
}

impl ObjectStorageServiceBuilder {
    pub fn new() -> Self {
        Self {
            provider: "s3".to_string(),
            bucket_name: String::new(),
            region: "us-east-1".to_string(),
            endpoint: String::new(),
            access_key: String::new(),
            secret_key: String::new(),
        }
    }
    
    pub fn provider(mut self, provider: &str) -> Self {
        self.provider = provider.to_string();
        self
    }
    
    pub fn bucket_name(mut self, bucket_name: &str) -> Self {
        self.bucket_name = bucket_name.to_string();
        self
    }
    
    pub fn region(mut self, region: &str) -> Self {
        self.region = region.to_string();
        self
    }
    
    pub fn endpoint(mut self, endpoint: &str) -> Self {
        self.endpoint = endpoint.to_string();
        self
    }
    
    pub fn credentials(mut self, access_key: &str, secret_key: &str) -> Self {
        self.access_key = access_key.to_string();
        self.secret_key = secret_key.to_string();
        self
    }
    
    pub async fn build(self) -> Result<Box<dyn ObjectStorageService>, ObjectStorageError> {
        ObjectStorageServiceFactory::create(
            &self.provider,
            &self.bucket_name,
            &self.region,
            &self.endpoint,
            &self.access_key,
            &self.secret_key,
        ).await
    }
}

impl Default for ObjectStorageServiceBuilder {
    fn default() -> Self {
        Self::new()
    }
}
