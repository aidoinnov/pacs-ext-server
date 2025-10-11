use std::collections::HashMap;
use std::sync::Arc;

use pacs_server::application::use_cases::MaskUseCase;
use pacs_server::domain::services::{MaskService, MaskGroupService, ServiceError};
use pacs_server::domain::entities::{Mask, MaskGroup, NewMask, UpdateMask, NewMaskGroup, UpdateMaskGroup, MaskStats, MaskGroupStats};
use pacs_server::application::dto::mask_dto::{
    CreateMaskRequest, UpdateMaskRequest, DownloadUrlRequest
};
use pacs_server::application::services::{SignedUrlService, SignedUrlRequest, SignedUrlResponse, SignedUrlError};

// Mock MaskService for testing
#[derive(Clone)]
struct MockMaskService {
    masks: HashMap<i32, Mask>,
}

impl MockMaskService {
    fn new() -> Self {
        Self {
            masks: HashMap::new(),
        }
    }

    fn add_mask(&mut self, mask: Mask) {
        self.masks.insert(mask.id, mask);
    }
}

#[async_trait::async_trait]
impl MaskService for MockMaskService {
    async fn create_mask(&self, new_mask: &NewMask) -> Result<Mask, ServiceError> {
        let mask = Mask {
            id: (self.masks.len() + 1) as i32,
            mask_group_id: new_mask.mask_group_id,
            slice_index: new_mask.slice_index,
            sop_instance_uid: new_mask.sop_instance_uid.clone(),
            label_name: new_mask.label_name.clone(),
            file_path: new_mask.file_path.clone(),
            mime_type: Some(new_mask.mime_type.as_ref().unwrap_or(&"image/png".to_string()).clone()),
            file_size: new_mask.file_size,
            checksum: new_mask.checksum.clone(),
            width: new_mask.width,
            height: new_mask.height,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        Ok(mask)
    }

    async fn get_mask_by_id(&self, id: i32) -> Result<Option<Mask>, ServiceError> {
        Ok(self.masks.get(&id).cloned())
    }

    async fn update_mask(&self, id: i32, update_mask: &UpdateMask) -> Result<Mask, ServiceError> {
        if let Some(mut mask) = self.masks.get(&id).cloned() {
            if let Some(slice_index) = update_mask.slice_index {
                mask.slice_index = Some(slice_index);
            }
            if let Some(sop_instance_uid) = &update_mask.sop_instance_uid {
                mask.sop_instance_uid = Some(sop_instance_uid.clone());
            }
            if let Some(label_name) = &update_mask.label_name {
                mask.label_name = Some(label_name.clone());
            }
            if let Some(file_path) = &update_mask.file_path {
                mask.file_path = file_path.clone();
            }
            if let Some(mime_type) = &update_mask.mime_type {
                mask.mime_type = Some(mime_type.clone());
            }
            if let Some(file_size) = update_mask.file_size {
                mask.file_size = Some(file_size);
            }
            if let Some(checksum) = &update_mask.checksum {
                mask.checksum = Some(checksum.clone());
            }
            if let Some(width) = update_mask.width {
                mask.width = Some(width);
            }
            if let Some(height) = update_mask.height {
                mask.height = Some(height);
            }
            mask.updated_at = chrono::Utc::now();
            Ok(mask)
        } else {
            Err(ServiceError::NotFound("Mask not found".to_string()))
        }
    }

    async fn delete_mask(&self, id: i32) -> Result<(), ServiceError> {
        if self.masks.contains_key(&id) {
            Ok(())
        } else {
            Err(ServiceError::NotFound("Mask not found".to_string()))
        }
    }

    async fn list_masks(
        &self,
        mask_group_id: Option<i32>,
        _sop_instance_uid: Option<String>,
        _label_name: Option<String>,
        _mime_type: Option<String>,
        _offset: Option<i64>,
        _limit: Option<i64>,
    ) -> Result<Vec<Mask>, ServiceError> {
        let masks: Vec<Mask> = self.masks.values()
            .filter(|mask| mask_group_id.map_or(true, |id| mask.mask_group_id == id))
            .cloned()
            .collect();
        Ok(masks)
    }

    async fn get_mask_stats(&self, _mask_group_id: Option<i32>) -> Result<MaskStats, ServiceError> {
        let mut stats = MaskStats::new();
        stats.total_masks = self.masks.len() as i64;
        stats.total_size_bytes = self.masks.values()
            .map(|mask| mask.file_size.unwrap_or(0))
            .sum();
        stats.calculate_average_file_size();
        Ok(stats)
    }

    async fn count_masks(
        &self,
        mask_group_id: Option<i32>,
        _sop_instance_uid: Option<String>,
        _label_name: Option<String>,
        _mime_type: Option<String>,
    ) -> Result<i64, ServiceError> {
        let count = self.masks.values()
            .filter(|mask| mask_group_id.map_or(true, |id| mask.mask_group_id == id))
            .count() as i64;
        Ok(count)
    }

    async fn can_access_mask(&self, _user_id: i32, _mask_id: i32) -> Result<bool, ServiceError> {
        Ok(true)
    }

    async fn can_create_mask(&self, _user_id: i32, _mask_group_id: i32) -> Result<bool, ServiceError> {
        Ok(true)
    }
}

// Mock MaskGroupService for testing
#[derive(Clone)]
struct MockMaskGroupService {
    mask_groups: HashMap<i32, MaskGroup>,
}

impl MockMaskGroupService {
    fn new() -> Self {
        Self {
            mask_groups: HashMap::new(),
        }
    }

    fn add_mask_group(&mut self, mask_group: MaskGroup) {
        self.mask_groups.insert(mask_group.id, mask_group);
    }
}

#[async_trait::async_trait]
impl MaskGroupService for MockMaskGroupService {
    async fn create_mask_group(&self, _new_mask_group: &NewMaskGroup) -> Result<MaskGroup, ServiceError> {
        let mask_group = MaskGroup {
            id: (self.mask_groups.len() + 1) as i32,
            annotation_id: 1,
            group_name: Some("Test Mask Group".to_string()),
            model_name: Some("TestModel".to_string()),
            version: Some("1.0.0".to_string()),
            modality: Some("CT".to_string()),
            slice_count: Some(10),
            mask_type: Some("segmentation".to_string()),
            description: Some("Test description".to_string()),
            created_by: Some(1),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        Ok(mask_group)
    }

    async fn get_mask_group_by_id(&self, id: i32) -> Result<Option<MaskGroup>, ServiceError> {
        Ok(self.mask_groups.get(&id).cloned())
    }

    async fn update_mask_group(&self, _id: i32, _update_mask_group: &UpdateMaskGroup) -> Result<MaskGroup, ServiceError> {
        let mask_group = MaskGroup {
            id: 1,
            annotation_id: 1,
            group_name: Some("Updated Mask Group".to_string()),
            model_name: Some("UpdatedModel".to_string()),
            version: Some("2.0.0".to_string()),
            modality: Some("MR".to_string()),
            slice_count: Some(20),
            mask_type: Some("bounding_box".to_string()),
            description: Some("Updated description".to_string()),
            created_by: Some(1),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        Ok(mask_group)
    }

    async fn delete_mask_group(&self, id: i32) -> Result<(), ServiceError> {
        if self.mask_groups.contains_key(&id) {
            Ok(())
        } else {
            Err(ServiceError::NotFound("Mask group not found".to_string()))
        }
    }

    async fn list_mask_groups(
        &self,
        _annotation_id: Option<i32>,
        _created_by: Option<i32>,
        _modality: Option<String>,
        _mask_type: Option<String>,
        _offset: Option<i64>,
        _limit: Option<i64>,
    ) -> Result<Vec<MaskGroup>, ServiceError> {
        let mask_groups: Vec<MaskGroup> = self.mask_groups.values().cloned().collect();
        Ok(mask_groups)
    }

    async fn get_masks_in_group(&self, _mask_group_id: i32) -> Result<Vec<Mask>, ServiceError> {
        Ok(vec![])
    }

    async fn get_mask_group_stats(&self, _annotation_id: Option<i32>) -> Result<MaskGroupStats, ServiceError> {
        let mut stats = MaskGroupStats::new();
        stats.total_groups = self.mask_groups.len() as i64;
        Ok(stats)
    }

    async fn count_mask_groups(
        &self,
        _annotation_id: Option<i32>,
        _created_by: Option<i32>,
        _modality: Option<String>,
        _mask_type: Option<String>,
    ) -> Result<i64, ServiceError> {
        Ok(self.mask_groups.len() as i64)
    }

    async fn can_access_mask_group(&self, _user_id: i32, _mask_group_id: i32) -> Result<bool, ServiceError> {
        Ok(true)
    }

    async fn can_create_mask_group(&self, _user_id: i32, _annotation_id: i32) -> Result<bool, ServiceError> {
        Ok(true)
    }
}

// Mock SignedUrlService for testing
struct MockSignedUrlService;

#[async_trait::async_trait]
impl SignedUrlService for MockSignedUrlService {
    async fn generate_upload_url(&self, _request: SignedUrlRequest) -> Result<SignedUrlResponse, SignedUrlError> {
        Ok(SignedUrlResponse::new(
            "https://example.com/upload".to_string(),
            "test/path".to_string(),
            3600,
            "PUT".to_string(),
        ))
    }

    async fn generate_download_url(&self, _request: SignedUrlRequest) -> Result<SignedUrlResponse, SignedUrlError> {
        Ok(SignedUrlResponse::new(
            "https://example.com/download".to_string(),
            "test/path".to_string(),
            3600,
            "GET".to_string(),
        ))
    }

    async fn generate_mask_upload_url(
        &self,
        _annotation_id: i32,
        _mask_group_id: i32,
        _file_name: String,
        _content_type: String,
        _expires_in: Option<u64>,
        _user_id: Option<i32>,
    ) -> Result<SignedUrlResponse, SignedUrlError> {
        Ok(SignedUrlResponse::new(
            "https://example.com/mask/upload".to_string(),
            "mask/test/path".to_string(),
            3600,
            "PUT".to_string(),
        ))
    }

    async fn generate_mask_download_url(
        &self,
        _file_path: String,
        _expires_in: Option<u64>,
    ) -> Result<SignedUrlResponse, SignedUrlError> {
        Ok(SignedUrlResponse::new(
            "https://example.com/mask/download".to_string(),
            "mask/test/path".to_string(),
            3600,
            "GET".to_string(),
        ))
    }

    async fn generate_annotation_upload_url(
        &self,
        _annotation_id: i32,
        _file_name: String,
        _content_type: String,
        _expires_in: Option<u64>,
        _user_id: Option<i32>,
    ) -> Result<SignedUrlResponse, SignedUrlError> {
        Ok(SignedUrlResponse::new(
            "https://example.com/annotation/upload".to_string(),
            "annotation/test/path".to_string(),
            3600,
            "PUT".to_string(),
        ))
    }

    async fn generate_annotation_download_url(
        &self,
        _file_path: String,
        _expires_in: Option<u64>,
    ) -> Result<SignedUrlResponse, SignedUrlError> {
        Ok(SignedUrlResponse::new(
            "https://example.com/annotation/download".to_string(),
            "annotation/test/path".to_string(),
            3600,
            "GET".to_string(),
        ))
    }
}

fn create_test_mask() -> Mask {
    Mask {
        id: 1,
        mask_group_id: 1,
        slice_index: Some(0),
        sop_instance_uid: Some("1.2.3.4.5".to_string()),
        label_name: Some("liver".to_string()),
        file_path: "test/mask.png".to_string(),
        mime_type: Some("image/png".to_string()),
        file_size: Some(1024),
        checksum: Some("abc123".to_string()),
        width: Some(512),
        height: Some(512),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}

fn create_test_mask_group() -> MaskGroup {
    MaskGroup {
        id: 1,
        annotation_id: 1,
        group_name: Some("Test Mask Group".to_string()),
        model_name: Some("TestModel".to_string()),
        version: Some("1.0.0".to_string()),
        modality: Some("CT".to_string()),
        slice_count: Some(10),
        mask_type: Some("segmentation".to_string()),
        description: Some("Test description".to_string()),
        created_by: Some(1),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}

#[tokio::test]
async fn test_mask_use_case_create_mask_success() {
    let mock_mask_service = MockMaskService::new();
    let mask_use_case = MaskUseCase::new(
        Arc::new(mock_mask_service),
        Arc::new(MockMaskGroupService::new()),
        Arc::new(MockSignedUrlService),
    );

    let create_request = CreateMaskRequest {
        mask_group_id: 1,
        slice_index: Some(0),
        sop_instance_uid: Some("1.2.3.4.5".to_string()),
        label_name: Some("liver".to_string()),
        file_path: "test/mask.png".to_string(),
        mime_type: "image/png".to_string(),
        file_size: Some(1024),
        checksum: Some("abc123".to_string()),
        width: Some(512),
        height: Some(512),
    };

    let result = mask_use_case.create_mask(create_request, 1).await;
    assert!(result.is_ok());
    let mask_response = result.unwrap();
    assert_eq!(mask_response.mask_group_id, 1);
    assert_eq!(mask_response.file_path, "test/mask.png");
    assert_eq!(mask_response.mime_type, "image/png");
}

#[tokio::test]
async fn test_mask_use_case_get_mask_success() {
    let mut mock_mask_service = MockMaskService::new();
    let mask = create_test_mask();
    mock_mask_service.add_mask(mask.clone());
    
    let mask_use_case = MaskUseCase::new(
        Arc::new(mock_mask_service),
        Arc::new(MockMaskGroupService::new()),
        Arc::new(MockSignedUrlService),
    );

    let result = mask_use_case.get_mask(mask.id, 1).await;
    assert!(result.is_ok());
    let mask_response = result.unwrap();
    assert_eq!(mask_response.id, mask.id);
    assert_eq!(mask_response.mask_group_id, mask.mask_group_id);
}

#[tokio::test]
async fn test_mask_use_case_update_mask_success() {
    let mut mock_mask_service = MockMaskService::new();
    let mask = create_test_mask();
    mock_mask_service.add_mask(mask);
    
    let mask_use_case = MaskUseCase::new(
        Arc::new(mock_mask_service),
        Arc::new(MockMaskGroupService::new()),
        Arc::new(MockSignedUrlService),
    );

    let update_request = UpdateMaskRequest {
        slice_index: Some(1),
        sop_instance_uid: Some("1.2.3.4.6".to_string()),
        label_name: Some("spleen".to_string()),
        file_path: Some("test/mask_updated.png".to_string()),
        mime_type: Some("image/jpeg".to_string()),
        file_size: Some(2048),
        checksum: Some("def456".to_string()),
        width: Some(1024),
        height: Some(1024),
    };

    let result = mask_use_case.update_mask(1, update_request, 1).await;
    assert!(result.is_ok());
    let mask_response = result.unwrap();
    assert_eq!(mask_response.id, 1);
}

#[tokio::test]
async fn test_mask_use_case_delete_mask_success() {
    let mut mock_mask_service = MockMaskService::new();
    let mask = create_test_mask();
    mock_mask_service.add_mask(mask);
    
    let mask_use_case = MaskUseCase::new(
        Arc::new(mock_mask_service),
        Arc::new(MockMaskGroupService::new()),
        Arc::new(MockSignedUrlService),
    );

    let result = mask_use_case.delete_mask(1, 1).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_mask_use_case_generate_download_url_success() {
    let mut mock_mask_service = MockMaskService::new();
    let mask = create_test_mask();
    mock_mask_service.add_mask(mask);
    
    let mask_use_case = MaskUseCase::new(
        Arc::new(mock_mask_service),
        Arc::new(MockMaskGroupService::new()),
        Arc::new(MockSignedUrlService),
    );

    let download_request = DownloadUrlRequest {
        mask_id: 1,
        file_path: "test/mask.png".to_string(),
        expires_in: Some(3600),
    };

    let result = mask_use_case.generate_download_url(download_request, 1).await;
    assert!(result.is_ok());
    let download_response = result.unwrap();
    assert!(!download_response.download_url.is_empty());
}

#[tokio::test]
async fn test_mask_use_case_get_mask_stats_success() {
    let mut mock_mask_service = MockMaskService::new();
    let mask1 = create_test_mask();
    let mask2 = Mask {
        id: 2,
        mask_group_id: 1,
        slice_index: Some(1),
        sop_instance_uid: Some("1.2.3.4.6".to_string()),
        label_name: Some("spleen".to_string()),
        file_path: "test/mask2.png".to_string(),
        mime_type: Some("image/png".to_string()),
        file_size: Some(2048),
        checksum: Some("def456".to_string()),
        width: Some(1024),
        height: Some(1024),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    mock_mask_service.add_mask(mask1);
    mock_mask_service.add_mask(mask2);
    
    let mask_use_case = MaskUseCase::new(
        Arc::new(mock_mask_service),
        Arc::new(MockMaskGroupService::new()),
        Arc::new(MockSignedUrlService),
    );

    let result = mask_use_case.get_mask_stats(Some(1), 1).await;
    assert!(result.is_ok());
    let stats_response = result.unwrap();
    assert_eq!(stats_response.total_masks, 2);
    assert!(stats_response.total_size_bytes > 0);
}