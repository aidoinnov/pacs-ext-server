use std::sync::Arc;
use std::collections::HashMap;
use chrono::{DateTime, Utc, NaiveDateTime};

use pacs_server::domain::entities::{
    mask::{Mask, NewMask, UpdateMask, MaskStats},
    mask_group::{MaskGroup, NewMaskGroup},
    user::User,
};

// Helper functions to create test data
fn create_test_mask_group() -> MaskGroup {
    MaskGroup {
        id: 1,
        annotation_id: 1,
        group_name: Some("Test Mask Group".to_string()),
        model_name: Some("Test Model".to_string()),
        version: Some("1.0.0".to_string()),
        modality: Some("CT".to_string()),
        slice_count: Some(10),
        mask_type: Some("segmentation".to_string()),
        description: Some("Test description".to_string()),
        created_by: Some(101),
        created_at: DateTime::from_naive_utc_and_offset(
            NaiveDateTime::from_timestamp_opt(1640995200, 0).unwrap(),
            Utc,
        ),
        updated_at: DateTime::from_naive_utc_and_offset(
            NaiveDateTime::from_timestamp_opt(1640995200, 0).unwrap(),
            Utc,
        ),
    }
}

fn create_test_mask() -> Mask {
    Mask {
        id: 1,
        mask_group_id: 1,
        slice_index: Some(1),
        sop_instance_uid: Some("1.2.3.4.5.6.7.8.9.1.1".to_string()),
        label_name: Some("lung_nodule".to_string()),
        file_path: "masks/annotation_1/group_1/slice_001.png".to_string(),
        mime_type: Some("image/png".to_string()),
        file_size: Some(102400),
        checksum: Some("md5-checksum".to_string()),
        width: Some(512),
        height: Some(512),
        created_at: DateTime::from_naive_utc_and_offset(
            NaiveDateTime::from_timestamp_opt(1640995200, 0).unwrap(),
            Utc,
        ),
        updated_at: DateTime::from_naive_utc_and_offset(
            NaiveDateTime::from_timestamp_opt(1640995200, 0).unwrap(),
            Utc,
        ),
    }
}

fn create_test_new_mask() -> NewMask {
    NewMask::new(
        1, // mask_group_id
        "masks/annotation_1/group_1/slice_001.png".to_string(), // file_path
        "image/png".to_string(), // mime_type
        Some(1), // slice_index
        Some("1.2.3.4.5.6.7.8.9.1.1".to_string()), // sop_instance_uid
        Some("lung_nodule".to_string()), // label_name
        Some(102400), // file_size
        Some("md5-checksum-12345".to_string()), // checksum
        Some(512), // width
        Some(512), // height
    )
}

fn create_test_update_mask() -> UpdateMask {
    UpdateMask::new(1)
        .with_label_name("updated_lung_nodule".to_string())
        .with_file_size(204800)
}

fn create_test_mask_stats() -> MaskStats {
    let mut stats = MaskStats::new();
    stats.total_masks = 10;
    stats.total_size_bytes = 1024000;
    stats.add_mime_type_count("image/png".to_string(), 8);
    stats.add_mime_type_count("image/jpeg".to_string(), 2);
    stats.add_label_name_count("lung_nodule".to_string(), 6);
    stats.add_label_name_count("liver_lesion".to_string(), 4);
    stats.calculate_average_file_size();
    stats
}

#[tokio::test]
async fn test_mask_entity_creation() {
    let mask = create_test_mask();
    
    assert_eq!(mask.id, 1);
    assert_eq!(mask.mask_group_id, 1);
    assert_eq!(mask.slice_index, Some(1));
    assert_eq!(mask.sop_instance_uid, Some("1.2.3.4.5.6.7.8.9.1.1".to_string()));
    assert_eq!(mask.label_name, Some("lung_nodule".to_string()));
    assert_eq!(mask.file_path, "masks/annotation_1/group_1/slice_001.png");
    assert_eq!(mask.mime_type, Some("image/png".to_string()));
    assert_eq!(mask.file_size, Some(102400));
    assert_eq!(mask.checksum, Some("md5-checksum".to_string()));
    assert_eq!(mask.width, Some(512));
    assert_eq!(mask.height, Some(512));
}

#[tokio::test]
async fn test_new_mask_creation() {
    let new_mask = create_test_new_mask();
    
    assert_eq!(new_mask.mask_group_id, 1);
    assert_eq!(new_mask.slice_index, Some(1));
    assert_eq!(new_mask.sop_instance_uid, Some("1.2.3.4.5.6.7.8.9.1.1".to_string()));
    assert_eq!(new_mask.label_name, Some("lung_nodule".to_string()));
    assert_eq!(new_mask.file_path, "masks/annotation_1/group_1/slice_001.png");
    assert_eq!(new_mask.mime_type, Some("image/png".to_string()));
    assert_eq!(new_mask.file_size, Some(102400));
    assert_eq!(new_mask.checksum, Some("md5-checksum-12345".to_string()));
    assert_eq!(new_mask.width, Some(512));
    assert_eq!(new_mask.height, Some(512));
}

#[tokio::test]
async fn test_update_mask_creation() {
    let update_mask = create_test_update_mask();
    
    assert_eq!(update_mask.id, 1);
    assert_eq!(update_mask.label_name, Some("updated_lung_nodule".to_string()));
    assert_eq!(update_mask.file_size, Some(204800));
}

#[tokio::test]
async fn test_mask_stats_creation() {
    let stats = create_test_mask_stats();
    
    assert_eq!(stats.total_masks, 10);
    assert_eq!(stats.total_size_bytes, 1024000);
    assert_eq!(stats.average_file_size, 102400.0);
    assert_eq!(stats.mime_types.get("image/png"), Some(&8));
    assert_eq!(stats.mime_types.get("image/jpeg"), Some(&2));
    assert_eq!(stats.label_names.get("lung_nodule"), Some(&6));
    assert_eq!(stats.label_names.get("liver_lesion"), Some(&4));
}

#[tokio::test]
async fn test_mask_group_entity_creation() {
    let mask_group = create_test_mask_group();
    
    assert_eq!(mask_group.id, 1);
    assert_eq!(mask_group.annotation_id, 1);
    assert_eq!(mask_group.group_name, Some("Test Mask Group".to_string()));
    assert_eq!(mask_group.model_name, Some("Test Model".to_string()));
    assert_eq!(mask_group.version, Some("1.0.0".to_string()));
    assert_eq!(mask_group.modality, Some("CT".to_string()));
    assert_eq!(mask_group.slice_count, Some(10));
    assert_eq!(mask_group.mask_type, Some("segmentation".to_string()));
    assert_eq!(mask_group.description, Some("Test description".to_string()));
    assert_eq!(mask_group.created_by, Some(101));
}