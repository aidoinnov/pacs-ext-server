use std::sync::Arc;
use chrono::{DateTime, Utc, NaiveDateTime};

use pacs_server::domain::entities::{
    mask_group::{MaskGroup, NewMaskGroup, UpdateMaskGroup, MaskGroupStats},
    mask::Mask,
    annotation::Annotation,
    user::User,
};

// Helper functions to create test data
fn create_test_annotation() -> Annotation {
    Annotation {
        id: 1,
        project_id: 1,
        user_id: 101,
        study_uid: "1.2.3.4.5.6.7.8.9".to_string(),
        series_uid: Some("1.2.3.4.5.6.7.8.9.1".to_string()),
        instance_uid: Some("1.2.3.4.5.6.7.8.9.1.1".to_string()),
        tool_name: "test_tool".to_string(),
        tool_version: Some("1.0.0".to_string()),
        data: serde_json::json!({"test": "data"}),
        is_shared: false,
        created_at: NaiveDateTime::from_timestamp_opt(1640995200, 0).unwrap(),
        updated_at: NaiveDateTime::from_timestamp_opt(1640995200, 0).unwrap(),
        viewer_software: Some("test_viewer".to_string()),
        description: Some("test annotation".to_string()),
    }
}

fn create_test_user() -> User {
    User {
        id: 101,
        keycloak_id: uuid::Uuid::new_v4(),
        username: "test_user".to_string(),
        email: "test@example.com".to_string(),
        created_at: NaiveDateTime::from_timestamp_opt(1640995200, 0).unwrap(),
    }
}

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

fn create_test_new_mask_group() -> NewMaskGroup {
    NewMaskGroup::new(
        1, // annotation_id
        Some("Test Mask Group".to_string()),
        Some("Test Model".to_string()),
        Some("1.0.0".to_string()),
        Some("CT".to_string()),
        10, // slice_count
        "segmentation".to_string(), // mask_type
        Some("Test description".to_string()),
        Some(101), // created_by
    )
}

fn create_test_update_mask_group() -> UpdateMaskGroup {
    UpdateMaskGroup::new(1)
        .with_group_name("Updated Mask Group".to_string())
        .with_description("Updated description".to_string())
}

fn create_test_mask_group_stats() -> MaskGroupStats {
    let mut stats = MaskGroupStats::new();
    stats.total_groups = 5;
    stats.total_masks = 50;
    stats.total_size_bytes = 1024000;
    stats.add_modality_count("CT".to_string(), 3);
    stats.add_modality_count("MR".to_string(), 2);
    stats.add_mask_type_count("segmentation".to_string(), 4);
    stats.add_mask_type_count("bounding_box".to_string(), 1);
    stats
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

#[tokio::test]
async fn test_new_mask_group_creation() {
    let new_mask_group = create_test_new_mask_group();
    
    assert_eq!(new_mask_group.annotation_id, 1);
    assert_eq!(new_mask_group.group_name, Some("Test Mask Group".to_string()));
    assert_eq!(new_mask_group.model_name, Some("Test Model".to_string()));
    assert_eq!(new_mask_group.version, Some("1.0.0".to_string()));
    assert_eq!(new_mask_group.modality, Some("CT".to_string()));
    assert_eq!(new_mask_group.slice_count, Some(10));
    assert_eq!(new_mask_group.mask_type, Some("segmentation".to_string()));
    assert_eq!(new_mask_group.description, Some("Test description".to_string()));
    assert_eq!(new_mask_group.created_by, Some(101));
}

#[tokio::test]
async fn test_update_mask_group_creation() {
    let update_mask_group = create_test_update_mask_group();
    
    assert_eq!(update_mask_group.id, 1);
    assert_eq!(update_mask_group.group_name, Some("Updated Mask Group".to_string()));
    assert_eq!(update_mask_group.description, Some("Updated description".to_string()));
}

#[tokio::test]
async fn test_mask_group_stats_creation() {
    let stats = create_test_mask_group_stats();
    
    assert_eq!(stats.total_groups, 5);
    assert_eq!(stats.total_masks, 50);
    assert_eq!(stats.total_size_bytes, 1024000);
    assert_eq!(stats.modalities.get("CT"), Some(&3));
    assert_eq!(stats.modalities.get("MR"), Some(&2));
    assert_eq!(stats.mask_types.get("segmentation"), Some(&4));
    assert_eq!(stats.mask_types.get("bounding_box"), Some(&1));
}

#[tokio::test]
async fn test_annotation_entity_creation() {
    let annotation = create_test_annotation();
    
    assert_eq!(annotation.id, 1);
    assert_eq!(annotation.project_id, 1);
    assert_eq!(annotation.user_id, 101);
    assert_eq!(annotation.study_uid, "1.2.3.4.5.6.7.8.9");
    assert_eq!(annotation.series_uid, Some("1.2.3.4.5.6.7.8.9.1".to_string()));
    assert_eq!(annotation.instance_uid, Some("1.2.3.4.5.6.7.8.9.1.1".to_string()));
    assert_eq!(annotation.tool_name, "test_tool");
    assert_eq!(annotation.tool_version, Some("1.0.0".to_string()));
    assert_eq!(annotation.is_shared, false);
    assert_eq!(annotation.viewer_software, Some("test_viewer".to_string()));
    assert_eq!(annotation.description, Some("test annotation".to_string()));
}

#[tokio::test]
async fn test_user_entity_creation() {
    let user = create_test_user();
    
    assert_eq!(user.id, 101);
    assert_eq!(user.username, "test_user");
    assert_eq!(user.email, "test@example.com");
}