use std::env;
use chrono::{DateTime, Utc, NaiveDateTime};

use pacs_server::domain::entities::{
    mask_group::{MaskGroup, NewMaskGroup, UpdateMaskGroup, MaskGroupStats},
    mask::{Mask, NewMask, UpdateMask, MaskStats},
};

// Helper functions to create test data
fn create_test_annotation() -> (i32, i32) {
    // This would normally be set up in a test database
    // For now, we'll assume annotation_id=1 and user_id=101 exist
    (1, 101)
}

fn create_test_new_mask_group() -> NewMaskGroup {
    NewMaskGroup::new(
        1, // annotation_id
        Some("Integration Test Mask Group".to_string()),
        Some("Test Model v2.0".to_string()),
        Some("2.0.0".to_string()),
        Some("CT".to_string()),
        25, // slice_count
        "segmentation".to_string(), // mask_type
        Some("Integration test description".to_string()),
        Some(101), // created_by
    )
}

fn create_test_new_mask() -> NewMask {
    NewMask::new(
        1, // mask_group_id (will be set after creating mask group)
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

#[tokio::test]
async fn test_mask_group_entity_validation() {
    let new_mask_group = create_test_new_mask_group();
    
    // Test basic validation
    assert_eq!(new_mask_group.annotation_id, 1);
    assert_eq!(new_mask_group.group_name, Some("Integration Test Mask Group".to_string()));
    assert_eq!(new_mask_group.model_name, Some("Test Model v2.0".to_string()));
    assert_eq!(new_mask_group.version, Some("2.0.0".to_string()));
    assert_eq!(new_mask_group.modality, Some("CT".to_string()));
    assert_eq!(new_mask_group.slice_count, Some(25));
    assert_eq!(new_mask_group.mask_type, Some("segmentation".to_string()));
    assert_eq!(new_mask_group.description, Some("Integration test description".to_string()));
    assert_eq!(new_mask_group.created_by, Some(101));
}

#[tokio::test]
async fn test_mask_entity_validation() {
    let new_mask = create_test_new_mask();
    
    // Test basic validation
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
async fn test_mask_group_stats_validation() {
    let mut stats = MaskGroupStats::new();
    stats.total_groups = 5;
    stats.total_masks = 50;
    stats.total_size_bytes = 1024000;
    stats.add_modality_count("CT".to_string(), 3);
    stats.add_modality_count("MR".to_string(), 2);
    stats.add_mask_type_count("segmentation".to_string(), 4);
    stats.add_mask_type_count("bounding_box".to_string(), 1);
    
    assert_eq!(stats.total_groups, 5);
    assert_eq!(stats.total_masks, 50);
    assert_eq!(stats.total_size_bytes, 1024000);
    assert_eq!(stats.modalities.get("CT"), Some(&3));
    assert_eq!(stats.modalities.get("MR"), Some(&2));
    assert_eq!(stats.mask_types.get("segmentation"), Some(&4));
    assert_eq!(stats.mask_types.get("bounding_box"), Some(&1));
}

#[tokio::test]
async fn test_mask_stats_validation() {
    let mut stats = MaskStats::new();
    stats.total_masks = 10;
    stats.total_size_bytes = 1024000;
    stats.add_mime_type_count("image/png".to_string(), 8);
    stats.add_mime_type_count("image/jpeg".to_string(), 2);
    stats.add_label_name_count("lung_nodule".to_string(), 6);
    stats.add_label_name_count("liver_lesion".to_string(), 4);
    stats.calculate_average_file_size();
    
    assert_eq!(stats.total_masks, 10);
    assert_eq!(stats.total_size_bytes, 1024000);
    assert_eq!(stats.average_file_size, 102400.0);
    assert_eq!(stats.mime_types.get("image/png"), Some(&8));
    assert_eq!(stats.mime_types.get("image/jpeg"), Some(&2));
    assert_eq!(stats.label_names.get("lung_nodule"), Some(&6));
    assert_eq!(stats.label_names.get("liver_lesion"), Some(&4));
}

#[tokio::test]
async fn test_update_mask_group_validation() {
    let update_mask_group = UpdateMaskGroup::new(1)
        .with_group_name("Updated Integration Test Mask Group".to_string())
        .with_description("Updated integration test description".to_string());
    
    assert_eq!(update_mask_group.id, 1);
    assert_eq!(update_mask_group.group_name, Some("Updated Integration Test Mask Group".to_string()));
    assert_eq!(update_mask_group.description, Some("Updated integration test description".to_string()));
}

#[tokio::test]
async fn test_update_mask_validation() {
    let update_mask = UpdateMask::new(1)
        .with_label_name("updated_lung_nodule".to_string())
        .with_file_size(204800);
    
    assert_eq!(update_mask.id, 1);
    assert_eq!(update_mask.label_name, Some("updated_lung_nodule".to_string()));
    assert_eq!(update_mask.file_size, Some(204800));
}