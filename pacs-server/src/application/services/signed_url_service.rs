use async_trait::async_trait;
use std::collections::HashMap;
use thiserror::Error;
use crate::application::services::object_storage_service::{
    ObjectStorageService, ObjectStorageError, SignedUrlOptions,
};

/// Signed URL 서비스 에러
#[derive(Debug, Error)]
pub enum SignedUrlError {
    #[error("Object storage error: {0}")]
    ObjectStorageError(#[from] ObjectStorageError),
    
    #[error("Invalid TTL: {0}")]
    InvalidTtl(String),
    
    #[error("Invalid file path: {0}")]
    InvalidFilePath(String),
    
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Signed URL 옵션
#[derive(Debug, Clone)]
pub struct SignedUrlRequest {
    pub file_path: String,
    pub ttl_seconds: Option<u64>,
    pub content_type: Option<String>,
    pub content_disposition: Option<String>,
    pub metadata: Option<HashMap<String, String>>,
    pub acl: Option<String>,
}

impl SignedUrlRequest {
    /// 새로운 Signed URL 요청 생성
    pub fn new(file_path: String) -> Self {
        Self {
            file_path,
            ttl_seconds: None,
            content_type: None,
            content_disposition: None,
            metadata: None,
            acl: None,
        }
    }
    
    /// TTL 설정
    pub fn with_ttl(mut self, ttl_seconds: u64) -> Self {
        self.ttl_seconds = Some(ttl_seconds);
        self
    }
    
    /// Content-Type 설정
    pub fn with_content_type(mut self, content_type: String) -> Self {
        self.content_type = Some(content_type);
        self
    }
    
    /// Content-Disposition 설정
    pub fn with_content_disposition(mut self, content_disposition: String) -> Self {
        self.content_disposition = Some(content_disposition);
        self
    }
    
    /// 메타데이터 설정
    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = Some(metadata);
        self
    }
    
    /// ACL 설정
    pub fn with_acl(mut self, acl: String) -> Self {
        self.acl = Some(acl);
        self
    }
    
    /// 어노테이션 ID 메타데이터 추가
    pub fn with_annotation_id(mut self, annotation_id: i32) -> Self {
        let mut metadata = self.metadata.unwrap_or_default();
        metadata.insert("annotation_id".to_string(), annotation_id.to_string());
        self.metadata = Some(metadata);
        self
    }
    
    /// 사용자 ID 메타데이터 추가
    pub fn with_user_id(mut self, user_id: i32) -> Self {
        let mut metadata = self.metadata.unwrap_or_default();
        metadata.insert("user_id".to_string(), user_id.to_string());
        self.metadata = Some(metadata);
        self
    }
    
    /// 마스크 그룹 ID 메타데이터 추가
    pub fn with_mask_group_id(mut self, mask_group_id: i32) -> Self {
        let mut metadata = self.metadata.unwrap_or_default();
        metadata.insert("mask_group_id".to_string(), mask_group_id.to_string());
        self.metadata = Some(metadata);
        self
    }
    
    /// 슬라이스 인덱스 메타데이터 추가
    pub fn with_slice_index(mut self, slice_index: i32) -> Self {
        let mut metadata = self.metadata.unwrap_or_default();
        metadata.insert("slice_index".to_string(), slice_index.to_string());
        self.metadata = Some(metadata);
        self
    }
}

/// Signed URL 응답
#[derive(Debug, Clone)]
pub struct SignedUrlResponse {
    pub url: String,
    pub file_path: String,
    pub ttl_seconds: u64,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub method: String, // "PUT" or "GET"
}

impl SignedUrlResponse {
    /// 새로운 Signed URL 응답 생성
    pub fn new(
        url: String,
        file_path: String,
        ttl_seconds: u64,
        method: String,
    ) -> Self {
        let expires_at = chrono::Utc::now() + chrono::Duration::seconds(ttl_seconds as i64);
        
        Self {
            url,
            file_path,
            ttl_seconds,
            expires_at,
            method,
        }
    }
    
    /// URL이 만료되었는지 확인
    pub fn is_expired(&self) -> bool {
        chrono::Utc::now() > self.expires_at
    }
    
    /// 남은 시간 (초)
    pub fn remaining_seconds(&self) -> i64 {
        let now = chrono::Utc::now();
        let remaining = self.expires_at - now;
        remaining.num_seconds().max(0)
    }
}

/// Signed URL 서비스 trait
#[async_trait]
pub trait SignedUrlService: Send + Sync {
    /// 업로드용 Signed URL 생성 (PUT)
    async fn generate_upload_url(
        &self,
        request: SignedUrlRequest,
    ) -> Result<SignedUrlResponse, SignedUrlError>;
    
    /// 다운로드용 Signed URL 생성 (GET)
    async fn generate_download_url(
        &self,
        request: SignedUrlRequest,
    ) -> Result<SignedUrlResponse, SignedUrlError>;
    
    /// 마스크 업로드용 Signed URL 생성
    async fn generate_mask_upload_url(
        &self,
        annotation_id: i32,
        mask_group_id: i32,
        file_name: String,
        content_type: String,
        ttl_seconds: Option<u64>,
        user_id: Option<i32>,
    ) -> Result<SignedUrlResponse, SignedUrlError>;
    
    /// 마스크 다운로드용 Signed URL 생성
    async fn generate_mask_download_url(
        &self,
        file_path: String,
        ttl_seconds: Option<u64>,
    ) -> Result<SignedUrlResponse, SignedUrlError>;
    
    /// 어노테이션 데이터 업로드용 Signed URL 생성
    async fn generate_annotation_upload_url(
        &self,
        annotation_id: i32,
        file_name: String,
        content_type: String,
        ttl_seconds: Option<u64>,
        user_id: Option<i32>,
    ) -> Result<SignedUrlResponse, SignedUrlError>;
    
    /// 어노테이션 데이터 다운로드용 Signed URL 생성
    async fn generate_annotation_download_url(
        &self,
        file_path: String,
        ttl_seconds: Option<u64>,
    ) -> Result<SignedUrlResponse, SignedUrlError>;
}

/// Signed URL 서비스 구현
pub struct SignedUrlServiceImpl {
    object_storage: Box<dyn ObjectStorageService>,
    default_ttl: u64,
    max_ttl: u64,
}

impl SignedUrlServiceImpl {
    /// 새로운 Signed URL 서비스 생성
    pub fn new(
        object_storage: Box<dyn ObjectStorageService>,
        default_ttl: u64,
        max_ttl: u64,
    ) -> Self {
        Self {
            object_storage,
            default_ttl,
            max_ttl,
        }
    }
    
    /// TTL 검증
    fn validate_ttl(&self, ttl_seconds: u64) -> Result<(), SignedUrlError> {
        if ttl_seconds == 0 {
            return Err(SignedUrlError::InvalidTtl("TTL cannot be zero".to_string()));
        }
        
        if ttl_seconds > self.max_ttl {
            return Err(SignedUrlError::InvalidTtl(
                format!("TTL cannot exceed {} seconds", self.max_ttl)
            ));
        }
        
        Ok(())
    }
    
    /// 파일 경로 검증
    fn validate_file_path(&self, file_path: &str) -> Result<(), SignedUrlError> {
        if file_path.is_empty() {
            return Err(SignedUrlError::InvalidFilePath("File path cannot be empty".to_string()));
        }
        
        if file_path.contains("..") {
            return Err(SignedUrlError::InvalidFilePath("File path cannot contain '..'".to_string()));
        }
        
        if file_path.starts_with('/') {
            return Err(SignedUrlError::InvalidFilePath("File path cannot start with '/'".to_string()));
        }
        
        Ok(())
    }
    
    /// Signed URL 옵션 생성
    fn create_signed_url_options(
        &self,
        request: &SignedUrlRequest,
        ttl_seconds: u64,
    ) -> SignedUrlOptions {
        SignedUrlOptions {
            ttl_seconds,
            content_type: request.content_type.clone(),
            content_disposition: request.content_disposition.clone(),
            metadata: request.metadata.clone(),
        }
    }
}

#[async_trait]
impl SignedUrlService for SignedUrlServiceImpl {
    async fn generate_upload_url(
        &self,
        request: SignedUrlRequest,
    ) -> Result<SignedUrlResponse, SignedUrlError> {
        // 파일 경로 검증
        self.validate_file_path(&request.file_path)?;
        
        // TTL 설정
        let ttl_seconds = request.ttl_seconds.unwrap_or(self.default_ttl);
        self.validate_ttl(ttl_seconds)?;
        
        // Signed URL 옵션 생성
        let options = self.create_signed_url_options(&request, ttl_seconds);
        
        // Object Storage에서 업로드 URL 생성
        let url = self.object_storage
            .generate_upload_url(&request.file_path, options)
            .await?;
        
        Ok(SignedUrlResponse::new(
            url,
            request.file_path,
            ttl_seconds,
            "PUT".to_string(),
        ))
    }
    
    async fn generate_download_url(
        &self,
        request: SignedUrlRequest,
    ) -> Result<SignedUrlResponse, SignedUrlError> {
        // 파일 경로 검증
        self.validate_file_path(&request.file_path)?;
        
        // TTL 설정
        let ttl_seconds = request.ttl_seconds.unwrap_or(self.default_ttl);
        self.validate_ttl(ttl_seconds)?;
        
        // Object Storage에서 다운로드 URL 생성
        let url = self.object_storage
            .generate_download_url(&request.file_path, ttl_seconds)
            .await?;
        
        Ok(SignedUrlResponse::new(
            url,
            request.file_path,
            ttl_seconds,
            "GET".to_string(),
        ))
    }
    
    async fn generate_mask_upload_url(
        &self,
        annotation_id: i32,
        mask_group_id: i32,
        file_name: String,
        content_type: String,
        ttl_seconds: Option<u64>,
        user_id: Option<i32>,
    ) -> Result<SignedUrlResponse, SignedUrlError> {
        // 마스크 파일 경로 생성
        let file_path = format!("masks/annotation_{}/group_{}/{}", annotation_id, mask_group_id, file_name);
        
        // 메타데이터 설정
        let mut metadata = HashMap::new();
        metadata.insert("annotation_id".to_string(), annotation_id.to_string());
        metadata.insert("mask_group_id".to_string(), mask_group_id.to_string());
        metadata.insert("file_type".to_string(), "mask".to_string());
        
        if let Some(user_id) = user_id {
            metadata.insert("user_id".to_string(), user_id.to_string());
        }
        
        // Signed URL 요청 생성
        let request = SignedUrlRequest::new(file_path)
            .with_ttl(ttl_seconds.unwrap_or(self.default_ttl))
            .with_content_type(content_type)
            .with_metadata(metadata);
        
        // 업로드 URL 생성
        self.generate_upload_url(request).await
    }
    
    async fn generate_mask_download_url(
        &self,
        file_path: String,
        ttl_seconds: Option<u64>,
    ) -> Result<SignedUrlResponse, SignedUrlError> {
        // Signed URL 요청 생성
        let request = SignedUrlRequest::new(file_path)
            .with_ttl(ttl_seconds.unwrap_or(self.default_ttl));
        
        // 다운로드 URL 생성
        self.generate_download_url(request).await
    }
    
    async fn generate_annotation_upload_url(
        &self,
        annotation_id: i32,
        file_name: String,
        content_type: String,
        ttl_seconds: Option<u64>,
        user_id: Option<i32>,
    ) -> Result<SignedUrlResponse, SignedUrlError> {
        // 어노테이션 파일 경로 생성
        let file_path = format!("annotations/annotation_{}/{}", annotation_id, file_name);
        
        // 메타데이터 설정
        let mut metadata = HashMap::new();
        metadata.insert("annotation_id".to_string(), annotation_id.to_string());
        metadata.insert("file_type".to_string(), "annotation".to_string());
        
        if let Some(user_id) = user_id {
            metadata.insert("user_id".to_string(), user_id.to_string());
        }
        
        // Signed URL 요청 생성
        let request = SignedUrlRequest::new(file_path)
            .with_ttl(ttl_seconds.unwrap_or(self.default_ttl))
            .with_content_type(content_type)
            .with_metadata(metadata);
        
        // 업로드 URL 생성
        self.generate_upload_url(request).await
    }
    
    async fn generate_annotation_download_url(
        &self,
        file_path: String,
        ttl_seconds: Option<u64>,
    ) -> Result<SignedUrlResponse, SignedUrlError> {
        // Signed URL 요청 생성
        let request = SignedUrlRequest::new(file_path)
            .with_ttl(ttl_seconds.unwrap_or(self.default_ttl));
        
        // 다운로드 URL 생성
        self.generate_download_url(request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::object_storage_mock_test::MockObjectStorageService;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_signed_url_request_creation() {
        let mut metadata = HashMap::new();
        metadata.insert("key".to_string(), "value".to_string());
        
        let request = SignedUrlRequest::new("test/file.png".to_string())
            .with_ttl(3600)
            .with_content_type("image/png".to_string())
            .with_content_disposition("attachment".to_string())
            .with_metadata(metadata.clone())
            .with_acl("private".to_string())
            .with_annotation_id(123)
            .with_user_id(456)
            .with_mask_group_id(789)
            .with_slice_index(1);
        
        assert_eq!(request.file_path, "test/file.png");
        assert_eq!(request.ttl_seconds, Some(3600));
        assert_eq!(request.content_type, Some("image/png".to_string()));
        assert_eq!(request.content_disposition, Some("attachment".to_string()));
        assert_eq!(request.acl, Some("private".to_string()));
        
        let metadata = request.metadata.unwrap();
        assert_eq!(metadata.get("annotation_id"), Some(&"123".to_string()));
        assert_eq!(metadata.get("user_id"), Some(&"456".to_string()));
        assert_eq!(metadata.get("mask_group_id"), Some(&"789".to_string()));
        assert_eq!(metadata.get("slice_index"), Some(&"1".to_string()));
    }
    
    #[tokio::test]
    async fn test_signed_url_response() {
        let response = SignedUrlResponse::new(
            "https://example.com/upload".to_string(),
            "test/file.png".to_string(),
            3600,
            "PUT".to_string(),
        );
        
        assert_eq!(response.url, "https://example.com/upload");
        assert_eq!(response.file_path, "test/file.png");
        assert_eq!(response.ttl_seconds, 3600);
        assert_eq!(response.method, "PUT");
        assert!(!response.is_expired());
        assert!(response.remaining_seconds() > 0);
    }
    
    #[tokio::test]
    async fn test_signed_url_service_impl() {
        let mock_storage = Box::new(MockObjectStorageService::new());
        let service = SignedUrlServiceImpl::new(mock_storage, 600, 3600);
        
        // TTL 검증 테스트
        assert!(service.validate_ttl(300).is_ok());
        assert!(service.validate_ttl(600).is_ok());
        assert!(service.validate_ttl(3600).is_ok());
        assert!(service.validate_ttl(0).is_err());
        assert!(service.validate_ttl(7200).is_err());
        
        // 파일 경로 검증 테스트
        assert!(service.validate_file_path("test/file.png").is_ok());
        assert!(service.validate_file_path("masks/group123/slice_001.png").is_ok());
        assert!(service.validate_file_path("").is_err());
        assert!(service.validate_file_path("../test/file.png").is_err());
        assert!(service.validate_file_path("/test/file.png").is_err());
    }
    
    #[tokio::test]
    async fn test_generate_mask_upload_url() {
        let mock_storage = Box::new(MockObjectStorageService::new());
        let service = SignedUrlServiceImpl::new(mock_storage, 600, 3600);
        
        let result = service.generate_mask_upload_url(
            123,
            456,
            "slice_001.png".to_string(),
            "image/png".to_string(),
            Some(1800),
            Some(789),
        ).await;
        
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.file_path.contains("masks/annotation_123/group_456/slice_001.png"));
        assert_eq!(response.method, "PUT");
        assert_eq!(response.ttl_seconds, 1800);
    }
    
    #[tokio::test]
    async fn test_generate_annotation_upload_url() {
        let mock_storage = Box::new(MockObjectStorageService::new());
        let service = SignedUrlServiceImpl::new(mock_storage, 600, 3600);
        
        let result = service.generate_annotation_upload_url(
            123,
            "data.json".to_string(),
            "application/json".to_string(),
            Some(1800),
            Some(456),
        ).await;
        
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.file_path.contains("annotations/annotation_123/data.json"));
        assert_eq!(response.method, "PUT");
        assert_eq!(response.ttl_seconds, 1800);
    }
}
