use async_trait::async_trait;
use aws_sdk_s3::Client as S3Client;
use aws_sdk_s3::config::Credentials;
use aws_sdk_s3::presigning::PresigningConfig;
use std::time::Duration;
use crate::application::services::object_storage_service::{ObjectStorageService, ObjectStorageError, UploadedFile, SignedUrlOptions};

/// AWS S3를 사용한 객체 스토리지 서비스 구현
pub struct S3ObjectStorageService {
    client: S3Client,
    bucket_name: String,
    region: String,
}

impl S3ObjectStorageService {
    /// 새로운 S3 객체 스토리지 서비스 인스턴스 생성
    pub async fn new(
        bucket_name: String,
        region: String,
        access_key: String,
        secret_key: String,
    ) -> Result<Self, ObjectStorageError> {
        let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(aws_config::Region::new(region.clone()))
            .credentials_provider(Credentials::new(
                access_key,
                secret_key,
                None,
                None,
                "pacs-server",
            ))
            .load()
            .await;

        let client = S3Client::new(&config);

        Ok(S3ObjectStorageService {
            client,
            bucket_name,
            region,
        })
    }

    /// 객체 키 생성 (프리픽스 포함)
    fn generate_object_key(&self, file_path: &str) -> String {
        format!("pacs-masks/{}", file_path)
    }
}

#[async_trait]
impl ObjectStorageService for S3ObjectStorageService {
    async fn generate_upload_url(
        &self,
        file_path: &str,
        options: SignedUrlOptions,
    ) -> Result<String, ObjectStorageError> {
        let object_key = self.generate_object_key(file_path);
        
        let mut put_request = self.client
            .put_object()
            .bucket(&self.bucket_name)
            .key(&object_key);

        if let Some(content_type) = &options.content_type {
            put_request = put_request.content_type(content_type);
        }

        let presigned = put_request
            .presigned(PresigningConfig::expires_in(Duration::from_secs(options.ttl_seconds))
                .map_err(|e| ObjectStorageError::S3Error(e.to_string()))?)
            .await
            .map_err(|e| ObjectStorageError::S3Error(e.to_string()))?;

        Ok(presigned.uri().to_string())
    }

    async fn generate_download_url(
        &self,
        file_path: &str,
        ttl_seconds: u64,
    ) -> Result<String, ObjectStorageError> {
        let object_key = self.generate_object_key(file_path);
        
        let presigned = self.client
            .get_object()
            .bucket(&self.bucket_name)
            .key(&object_key)
            .presigned(PresigningConfig::expires_in(Duration::from_secs(ttl_seconds))
                .map_err(|e| ObjectStorageError::S3Error(e.to_string()))?)
            .await
            .map_err(|e| ObjectStorageError::S3Error(e.to_string()))?;

        Ok(presigned.uri().to_string())
    }

    async fn delete_file(&self, file_path: &str) -> Result<(), ObjectStorageError> {
        let object_key = self.generate_object_key(file_path);
        
        self.client
            .delete_object()
            .bucket(&self.bucket_name)
            .key(&object_key)
            .send()
            .await
            .map_err(|e| ObjectStorageError::S3Error(e.to_string()))?;

        Ok(())
    }

    async fn file_exists(&self, file_path: &str) -> Result<bool, ObjectStorageError> {
        let object_key = self.generate_object_key(file_path);
        
        match self.client
            .head_object()
            .bucket(&self.bucket_name)
            .key(&object_key)
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(e) => {
                if e.to_string().contains("NoSuchKey") {
                    Ok(false)
                } else {
                    Err(ObjectStorageError::S3Error(e.to_string()))
                }
            }
        }
    }

    async fn get_file_metadata(&self, file_path: &str) -> Result<UploadedFile, ObjectStorageError> {
        let object_key = self.generate_object_key(file_path);
        
        let result = self.client
            .head_object()
            .bucket(&self.bucket_name)
            .key(&object_key)
            .send()
            .await
            .map_err(|e| ObjectStorageError::S3Error(e.to_string()))?;

        Ok(UploadedFile {
            file_path: file_path.to_string(),
            file_size: result.content_length().unwrap_or(0),
            checksum: result.e_tag().map(|s| s.to_string()),
            mime_type: result.content_type().map(|s| s.to_string()),
            last_modified: result.last_modified()
                .map(|dt| dt.to_string()),
        })
    }

    async fn list_files(
        &self,
        prefix: &str,
        max_keys: Option<i32>,
    ) -> Result<Vec<String>, ObjectStorageError> {
        let mut list_request = self.client
            .list_objects_v2()
            .bucket(&self.bucket_name)
            .prefix(prefix);

        if let Some(max) = max_keys {
            list_request = list_request.max_keys(max);
        }

        let result = list_request
            .send()
            .await
            .map_err(|e| ObjectStorageError::S3Error(e.to_string()))?;

        let mut files = Vec::new();
        
        if let Some(contents) = result.contents {
            for object in contents {
                if let Some(key) = object.key {
                    let file_path = key.strip_prefix("pacs-masks/")
                        .unwrap_or(&key)
                        .to_string();
                    files.push(file_path);
                }
            }
        }

        Ok(files)
    }

    async fn copy_file(
        &self,
        source_path: &str,
        destination_path: &str,
    ) -> Result<(), ObjectStorageError> {
        let source_key = self.generate_object_key(source_path);
        let destination_key = self.generate_object_key(destination_path);
        
        self.client
            .copy_object()
            .bucket(&self.bucket_name)
            .copy_source(format!("{}/{}", self.bucket_name, source_key))
            .key(&destination_key)
            .send()
            .await
            .map_err(|e| ObjectStorageError::S3Error(e.to_string()))?;

        Ok(())
    }

    async fn move_file(
        &self,
        source_path: &str,
        destination_path: &str,
    ) -> Result<(), ObjectStorageError> {
        // S3에서는 move가 copy + delete로 구현됨
        self.copy_file(source_path, destination_path).await?;
        self.delete_file(source_path).await?;
        Ok(())
    }
}
