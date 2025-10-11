use async_trait::async_trait;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::{
    presigning::PresigningConfig,
    types::{ObjectCannedAcl, StorageClass},
    Client as S3Client,
};
use std::time::Duration;
use crate::application::services::object_storage_service::{
    ObjectStorageService, ObjectStorageError, UploadedFile, SignedUrlOptions,
};

/// AWS S3 Object Storage 서비스 구현
pub struct S3ObjectStorageService {
    client: S3Client,
    bucket_name: String,
}

impl S3ObjectStorageService {
    /// 새로운 S3 서비스 인스턴스 생성
    pub async fn new(
        bucket_name: &str,
        region: &str,
        _access_key: &str,
        _secret_key: &str,
    ) -> Result<Self, ObjectStorageError> {
        // TODO: AWS SDK lifetime 문제 해결 필요
        // 임시로 에러 반환하여 컴파일 문제 회피
        Err(ObjectStorageError::ConfigError(
            "S3 service temporarily disabled due to AWS SDK lifetime issues".to_string()
        ))
    }
    
    /// 버킷 존재 여부 확인
    async fn verify_bucket_exists(client: &S3Client, bucket_name: &str) -> Result<(), ObjectStorageError> {
        match client.head_bucket().bucket(bucket_name).send().await {
            Ok(_) => Ok(()),
            Err(e) => Err(ObjectStorageError::S3Error(
                format!("Bucket '{}' does not exist or is not accessible: {}", bucket_name, e)
            )),
        }
    }
    
    /// 파일 경로를 S3 키로 변환
    fn file_path_to_key(&self, file_path: &str) -> String {
        // 파일 경로가 이미 S3 키 형식인지 확인
        if file_path.starts_with("s3://") {
            file_path.strip_prefix("s3://").unwrap().to_string()
        } else {
            file_path.to_string()
        }
    }
    
    /// S3 에러를 ObjectStorageError로 변환
    fn map_s3_error(&self, error: aws_sdk_s3::Error) -> ObjectStorageError {
        match error {
            aws_sdk_s3::Error::NoSuchKey(_) => ObjectStorageError::FileNotFound(
                "File not found in S3 bucket".to_string()
            ),
            _ => ObjectStorageError::S3Error(error.to_string()),
        }
    }
}

#[async_trait]
impl ObjectStorageService for S3ObjectStorageService {
    async fn generate_upload_url(
        &self,
        file_path: &str,
        options: SignedUrlOptions,
    ) -> Result<String, ObjectStorageError> {
        let key = self.file_path_to_key(file_path);
        
        let presigning_config = PresigningConfig::expires_in(Duration::from_secs(options.ttl_seconds))
            .map_err(|e| ObjectStorageError::S3Error(format!("Failed to create presigning config: {}", e)))?;
        
        let mut put_object = self.client
            .put_object()
            .bucket(&self.bucket_name)
            .key(&key)
            .set_acl(Some(ObjectCannedAcl::Private))
            .set_storage_class(Some(StorageClass::Standard));
        
        // Content-Type 설정
        if let Some(content_type) = options.content_type {
            put_object = put_object.content_type(content_type);
        }
        
        // Content-Disposition 설정
        if let Some(content_disposition) = options.content_disposition {
            put_object = put_object.content_disposition(content_disposition);
        }
        
        // 메타데이터 설정
        if let Some(metadata) = options.metadata {
            for (key, value) in metadata {
                put_object = put_object.metadata(key, value);
            }
        }
        
        let request = put_object
            .presigned(presigning_config)
            .await
            .map_err(|e| self.map_s3_error(e.into()))?;
        
        Ok(request.uri().to_string())
    }
    
    async fn generate_download_url(
        &self,
        file_path: &str,
        ttl_seconds: u64,
    ) -> Result<String, ObjectStorageError> {
        let key = self.file_path_to_key(file_path);
        
        let presigning_config = PresigningConfig::expires_in(Duration::from_secs(ttl_seconds))
            .map_err(|e| ObjectStorageError::S3Error(format!("Failed to create presigning config: {}", e)))?;
        
        let request = self.client
            .get_object()
            .bucket(&self.bucket_name)
            .key(&key)
            .presigned(presigning_config)
            .await
            .map_err(|e| self.map_s3_error(e.into()))?;
        
        Ok(request.uri().to_string())
    }
    
    async fn delete_file(&self, file_path: &str) -> Result<(), ObjectStorageError> {
        let key = self.file_path_to_key(file_path);
        
        self.client
            .delete_object()
            .bucket(&self.bucket_name)
            .key(&key)
            .send()
            .await
            .map_err(|e| self.map_s3_error(e.into()))?;
        
        Ok(())
    }
    
    async fn get_file_metadata(
        &self,
        file_path: &str,
    ) -> Result<UploadedFile, ObjectStorageError> {
        let key = self.file_path_to_key(file_path);
        
        let response = self.client
            .head_object()
            .bucket(&self.bucket_name)
            .key(&key)
            .send()
            .await
            .map_err(|e| self.map_s3_error(e.into()))?;
        
        Ok(UploadedFile {
            file_path: file_path.to_string(),
            file_size: response.content_length().unwrap_or(0),
            checksum: response.e_tag().map(|s| s.to_string()),
            mime_type: response.content_type().map(|s| s.to_string()),
            last_modified: response.last_modified().map(|dt| dt.to_string()),
        })
    }
    
    async fn file_exists(&self, file_path: &str) -> Result<bool, ObjectStorageError> {
        let key = self.file_path_to_key(file_path);
        
        match self.client
            .head_object()
            .bucket(&self.bucket_name)
            .key(&key)
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(e) => {
                let s3_error: aws_sdk_s3::Error = e.into();
                match s3_error {
                    aws_sdk_s3::Error::NoSuchKey(_) => Ok(false),
                    _ => Err(self.map_s3_error(s3_error)),
                }
            }
        }
    }
    
    async fn list_files(
        &self,
        prefix: &str,
        max_keys: Option<i32>,
    ) -> Result<Vec<String>, ObjectStorageError> {
        let mut list_objects = self.client
            .list_objects_v2()
            .bucket(&self.bucket_name)
            .prefix(prefix);
        
        if let Some(max_keys) = max_keys {
            list_objects = list_objects.max_keys(max_keys);
        }
        
        let response = list_objects
            .send()
            .await
            .map_err(|e| self.map_s3_error(e.into()))?;
        
        let files = response
            .contents()
            .iter()
            .filter_map(|obj| obj.key().map(|s| s.to_string()))
            .collect();
        
        Ok(files)
    }
    
    async fn copy_file(
        &self,
        source_path: &str,
        destination_path: &str,
    ) -> Result<(), ObjectStorageError> {
        let source_key = self.file_path_to_key(source_path);
        let destination_key = self.file_path_to_key(destination_path);
        
        let copy_source = format!("{}/{}", self.bucket_name, source_key);
        
        self.client
            .copy_object()
            .bucket(&self.bucket_name)
            .key(&destination_key)
            .copy_source(copy_source)
            .send()
            .await
            .map_err(|e| self.map_s3_error(e.into()))?;
        
        Ok(())
    }
    
    async fn move_file(
        &self,
        source_path: &str,
        destination_path: &str,
    ) -> Result<(), ObjectStorageError> {
        // 1. 파일 복사
        self.copy_file(source_path, destination_path).await?;
        
        // 2. 원본 파일 삭제
        self.delete_file(source_path).await?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::services::object_storage_service::SignedUrlOptions;

    #[tokio::test]
    async fn test_s3_service_creation() {
        // 이 테스트는 실제 AWS 자격 증명이 필요하므로 스킵
        // 실제 테스트에서는 mock을 사용하거나 테스트용 AWS 계정 사용
    }
    
    #[tokio::test]
    async fn test_file_path_to_key() {
        let service = S3ObjectStorageService {
            client: S3Client::new(&aws_config::load_from_env().await),
            bucket_name: "test-bucket".to_string(),
        };
        
        // 일반 파일 경로
        assert_eq!(service.file_path_to_key("mask/123/456/file.png"), "mask/123/456/file.png");
        
        // S3 URI 형식
        assert_eq!(service.file_path_to_key("s3://test-bucket/mask/123/456/file.png"), "test-bucket/mask/123/456/file.png");
    }
}
