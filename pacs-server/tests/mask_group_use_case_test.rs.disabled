use std::sync::Arc;
use async_trait::async_trait;
use chrono::Utc;
use mockall::mock;

use pacs_server::application::use_cases::MaskGroupUseCase;
use pacs_server::application::dto::mask_group_dto::{
    CreateMaskGroupRequest, UpdateMaskGroupRequest, SignedUrlRequest, 
    CompleteUploadRequest
};
use pacs_server::domain::entities::{MaskGroup, NewMaskGroup, UpdateMaskGroup, MaskGroupStats, Mask};
use pacs_server::domain::services::MaskGroupService;
use pacs_server::domain::ServiceError;
use pacs_server::application::services::{SignedUrlService, SignedUrlError};

// Create mock for MaskGroupService
mock! {
    MaskGroupService {}

    #[async_trait]
    impl MaskGroupService for MaskGroupService {
        async fn create_mask_group(&self, new_mask_group: &NewMaskGroup) -> Result<MaskGroup, ServiceError>;
        async fn get_mask_group_by_id(&self, id: i32) -> Result<Option<MaskGroup>, ServiceError>;
        async fn update_mask_group(&self, id: i32, update_mask_group: &UpdateMaskGroup) -> Result<MaskGroup, ServiceError>;
        async fn delete_mask_group(&self, id: i32) -> Result<(), ServiceError>;
        async fn list_mask_groups(
            &self,
            annotation_id: Option<i32>,
            created_by: Option<i32>,
            modality: Option<String>,
            mask_type: Option<String>,
            offset: Option<i64>,
            limit: Option<i64>,
        ) -> Result<Vec<MaskGroup>, ServiceError>;
        async fn get_masks_in_group(&self, mask_group_id: i32) -> Result<Vec<Mask>, ServiceError>;
        async fn get_mask_group_stats(&self, annotation_id: Option<i32>) -> Result<MaskGroupStats, ServiceError>;
        async fn count_mask_groups(
            &self,
            annotation_id: Option<i32>,
            created_by: Option<i32>,
            modality: Option<String>,
            mask_type: Option<String>,
        ) -> Result<i64, ServiceError>;
        async fn can_access_mask_group(&self, user_id: i32, mask_group_id: i32) -> Result<bool, ServiceError>;
        async fn can_create_mask_group(&self, user_id: i32, annotation_id: i32) -> Result<bool, ServiceError>;
    }
}

// Create mock for SignedUrlService
mock! {
    SignedUrlService {}

    #[async_trait]
    impl SignedUrlService for SignedUrlService {
        async fn generate_upload_url(
            &self,
            request: pacs_server::application::services::SignedUrlRequest,
        ) -> Result<pacs_server::application::services::SignedUrlResponse, SignedUrlError>;
        async fn generate_download_url(
            &self,
            request: pacs_server::application::services::SignedUrlRequest,
        ) -> Result<pacs_server::application::services::SignedUrlResponse, SignedUrlError>;
            async fn generate_mask_upload_url(
                &self,
                annotation_id: i32,
                mask_group_id: i32,
                file_name: String,
                content_type: String,
                ttl_seconds: Option<u64>,
                user_id: Option<i32>,
            ) -> Result<pacs_server::application::services::SignedUrlResponse, SignedUrlError>;
        async fn generate_mask_download_url(
            &self,
            file_path: String,
            ttl_seconds: Option<u64>,
        ) -> Result<pacs_server::application::services::SignedUrlResponse, SignedUrlError>;
            async fn generate_annotation_upload_url(
                &self,
                annotation_id: i32,
                file_name: String,
                content_type: String,
                ttl_seconds: Option<u64>,
                user_id: Option<i32>,
            ) -> Result<pacs_server::application::services::SignedUrlResponse, SignedUrlError>;
        async fn generate_annotation_download_url(
            &self,
            file_path: String,
            ttl_seconds: Option<u64>,
        ) -> Result<pacs_server::application::services::SignedUrlResponse, SignedUrlError>;
    }
}


fn create_test_mask_group() -> MaskGroup {
    MaskGroup {
        id: 1,
        annotation_id: 1,
        group_name: Some("Test Group".to_string()),
        model_name: Some("test_model".to_string()),
        version: Some("1.0.0".to_string()),
        modality: Some("CT".to_string()),
        slice_count: Some(100),
        mask_type: Some("segmentation".to_string()),
        description: Some("Test description".to_string()),
        created_by: Some(1),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

fn create_test_mask_group_stats() -> MaskGroupStats {
    MaskGroupStats {
        total_groups: 1,
        total_masks: 10,
        total_size_bytes: 1024000,
        modalities: std::collections::HashMap::new(),
        mask_types: std::collections::HashMap::new(),
    }
}

#[tokio::test]
async fn test_mask_group_use_case_create_mask_group_success() {
    let mut mock_mask_group_service = MockMaskGroupService::new();
    let mut mock_signed_url_service = MockSignedUrlService::new();
    
    // Setup expectations
    mock_mask_group_service
        .expect_can_create_mask_group()
        .with(mockall::predicate::eq(1), mockall::predicate::eq(1))
        .times(1)
        .returning(|_, _| Ok(true));
    
    mock_mask_group_service
        .expect_create_mask_group()
        .times(1)
        .returning(|_| {
            Ok(create_test_mask_group())
        });
    
    let mask_group_use_case = MaskGroupUseCase::new(
        Arc::new(mock_mask_group_service),
        Arc::new(mock_signed_url_service),
    );

    let request = CreateMaskGroupRequest {
        group_name: Some("Test Group".to_string()),
        model_name: Some("test_model".to_string()),
        version: Some("1.0.0".to_string()),
        modality: Some("CT".to_string()),
        slice_count: 100,
        mask_type: "segmentation".to_string(),
        description: Some("Test description".to_string()),
    };

    let result = mask_group_use_case.create_mask_group(request.clone(), 1).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.annotation_id, 1);
    assert_eq!(response.group_name, Some("Test Group".to_string()));
    assert_eq!(response.slice_count, 100);
    assert_eq!(response.mask_type, "segmentation");
}

#[tokio::test]
async fn test_mask_group_use_case_get_mask_group_success() {
    let mut mock_mask_group_service = MockMaskGroupService::new();
    let mut mock_signed_url_service = MockSignedUrlService::new();
    
    // Setup expectations
    mock_mask_group_service
        .expect_can_access_mask_group()
        .with(mockall::predicate::eq(1), mockall::predicate::eq(1))
        .times(1)
        .returning(|_, _| Ok(true));
    
    mock_mask_group_service
        .expect_get_mask_group_by_id()
        .with(mockall::predicate::eq(1))
        .times(1)
        .returning(|_| Ok(Some(create_test_mask_group())));
    
    mock_mask_group_service
        .expect_get_mask_group_stats()
        .with(mockall::predicate::eq(Some(1)))
        .times(1)
        .returning(|_| Ok(create_test_mask_group_stats()));
    
    let mask_group_use_case = MaskGroupUseCase::new(
        Arc::new(mock_mask_group_service),
        Arc::new(mock_signed_url_service),
    );

    let result = mask_group_use_case.get_mask_group(1, 1).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.id, 1);
    assert_eq!(response.annotation_id, 1);
    assert_eq!(response.group_name, Some("Test Group".to_string()));
}

#[tokio::test]
async fn test_mask_group_use_case_get_mask_group_not_found() {
    let mut mock_mask_group_service = MockMaskGroupService::new();
    let mut mock_signed_url_service = MockSignedUrlService::new();
    
    // Setup expectations
    mock_mask_group_service
        .expect_can_access_mask_group()
        .with(mockall::predicate::eq(1), mockall::predicate::eq(999))
        .times(1)
        .returning(|_, _| Ok(true));
    
    mock_mask_group_service
        .expect_get_mask_group_by_id()
        .with(mockall::predicate::eq(999))
        .times(1)
        .returning(|_| Ok(None));
    
    let mask_group_use_case = MaskGroupUseCase::new(
        Arc::new(mock_mask_group_service),
        Arc::new(mock_signed_url_service),
    );

    let result = mask_group_use_case.get_mask_group(999, 1).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Mask group with ID 999 not found"));
}

#[tokio::test]
async fn test_mask_group_use_case_list_mask_groups_success() {
    let mut mock_mask_group_service = MockMaskGroupService::new();
    let mut mock_signed_url_service = MockSignedUrlService::new();
    
    // Setup expectations
    mock_mask_group_service
        .expect_list_mask_groups()
        .times(1)
        .returning(|_, _, _, _, _, _| Ok(vec![create_test_mask_group()]));
    
    mock_mask_group_service
        .expect_count_mask_groups()
        .times(1)
        .returning(|_, _, _, _| Ok(1));
    
    let mask_group_use_case = MaskGroupUseCase::new(
        Arc::new(mock_mask_group_service),
        Arc::new(mock_signed_url_service),
    );

    let result = mask_group_use_case.list_mask_groups(Some(1), 1, Some(0), Some(10)).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.mask_groups.len(), 1);
    assert_eq!(response.total_count, 1);
    assert_eq!(response.offset, 0);
    assert_eq!(response.limit, 10);
}

#[tokio::test]
async fn test_mask_group_use_case_update_mask_group_success() {
    let mut mock_mask_group_service = MockMaskGroupService::new();
    let mut mock_signed_url_service = MockSignedUrlService::new();
    
    // Setup expectations
    mock_mask_group_service
        .expect_can_access_mask_group()
        .with(mockall::predicate::eq(1), mockall::predicate::eq(1))
        .times(1)
        .returning(|_, _| Ok(true));
    
    mock_mask_group_service
        .expect_update_mask_group()
        .times(1)
        .returning(|_, _| {
            let mut mask_group = create_test_mask_group();
            mask_group.group_name = Some("Updated Group".to_string());
            mask_group.slice_count = Some(150);
            mask_group.mask_type = Some("detection".to_string());
            Ok(mask_group)
        });
    
    let mask_group_use_case = MaskGroupUseCase::new(
        Arc::new(mock_mask_group_service),
        Arc::new(mock_signed_url_service),
    );

    let request = UpdateMaskGroupRequest {
        group_name: Some("Updated Group".to_string()),
        model_name: Some("updated_model".to_string()),
        version: Some("2.0.0".to_string()),
        modality: Some("MR".to_string()),
        slice_count: Some(150),
        mask_type: Some("detection".to_string()),
        description: Some("Updated description".to_string()),
    };

    let result = mask_group_use_case.update_mask_group(1, request, 1).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.id, 1);
    assert_eq!(response.group_name, Some("Updated Group".to_string()));
    assert_eq!(response.slice_count, 150);
    assert_eq!(response.mask_type, "detection");
}

#[tokio::test]
async fn test_mask_group_use_case_delete_mask_group_success() {
    let mut mock_mask_group_service = MockMaskGroupService::new();
    let mut mock_signed_url_service = MockSignedUrlService::new();
    
    // Setup expectations
    mock_mask_group_service
        .expect_can_access_mask_group()
        .with(mockall::predicate::eq(1), mockall::predicate::eq(1))
        .times(1)
        .returning(|_, _| Ok(true));
    
    mock_mask_group_service
        .expect_delete_mask_group()
        .with(mockall::predicate::eq(1))
        .times(1)
        .returning(|_| Ok(()));
    
    let mask_group_use_case = MaskGroupUseCase::new(
        Arc::new(mock_mask_group_service),
        Arc::new(mock_signed_url_service),
    );

    let result = mask_group_use_case.delete_mask_group(1, 1).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_mask_group_use_case_generate_upload_url_success() {
    let mut mock_mask_group_service = MockMaskGroupService::new();
    let mut mock_signed_url_service = MockSignedUrlService::new();
    
    // Setup expectations
    mock_mask_group_service
        .expect_can_access_mask_group()
        .with(mockall::predicate::eq(1), mockall::predicate::eq(1))
        .times(1)
        .returning(|_, _| Ok(true));
    
           mock_signed_url_service
               .expect_generate_mask_upload_url()
               .times(1)
               .returning(|_, _, _, _, _, _| {
                   Ok(pacs_server::application::services::SignedUrlResponse::new(
                       "https://example.com/upload/test".to_string(),
                       "test".to_string(),
                       3600,
                       "PUT".to_string(),
                   ))
               });
    
    let mask_group_use_case = MaskGroupUseCase::new(
        Arc::new(mock_mask_group_service),
        Arc::new(mock_signed_url_service),
    );

    let request = SignedUrlRequest {
        mask_group_id: 1,
        filename: "test_mask.png".to_string(),
        mime_type: "image/png".to_string(),
        file_size: Some(1024000),
        slice_index: Some(1),
        sop_instance_uid: Some("1.2.3.4.5".to_string()),
        label_name: Some("liver".to_string()),
        ttl_seconds: Some(3600),
    };

    let result = mask_group_use_case.generate_upload_url(request, 1).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.upload_url.contains("example.com"));
    assert!(response.file_path.contains("test"));
    assert_eq!(response.expires_in, 3600);
}

#[tokio::test]
async fn test_mask_group_use_case_complete_upload_success() {
    let mut mock_mask_group_service = MockMaskGroupService::new();
    let mut mock_signed_url_service = MockSignedUrlService::new();
    
    // Setup expectations
    mock_mask_group_service
        .expect_can_access_mask_group()
        .with(mockall::predicate::eq(1), mockall::predicate::eq(1))
        .times(1)
        .returning(|_, _| Ok(true));
    
    let mask_group_use_case = MaskGroupUseCase::new(
        Arc::new(mock_mask_group_service),
        Arc::new(mock_signed_url_service),
    );

    let request = CompleteUploadRequest {
        mask_group_id: 1,
        slice_count: 120,
        labels: vec!["liver".to_string(), "spleen".to_string()],
        uploaded_files: vec!["file1.png".to_string(), "file2.png".to_string()],
    };

    let result = mask_group_use_case.complete_upload(request, 1).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.success);
    assert_eq!(response.status, "success");
    assert_eq!(response.processed_masks, 120);
    assert_eq!(response.uploaded_files.len(), 2);
}