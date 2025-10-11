// External service integrations (e.g., Keycloak, S3, MinIO) will be defined here

pub mod s3_service;
pub mod minio_service;

pub use s3_service::S3ObjectStorageService;
pub use minio_service::MinIOObjectStorageService;
