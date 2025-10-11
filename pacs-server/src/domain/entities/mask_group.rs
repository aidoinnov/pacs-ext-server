use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;
use sqlx::FromRow;

/// 마스크 그룹 엔티티
/// 여러 개의 마스크 파일을 하나의 그룹으로 관리
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, FromRow)]
pub struct MaskGroup {
    pub id: i32,
    pub annotation_id: i32,
    pub group_name: Option<String>,
    pub model_name: Option<String>,
    pub version: Option<String>,
    pub modality: Option<String>,
    pub slice_count: Option<i32>,
    pub mask_type: Option<String>,
    pub description: Option<String>,
    pub created_by: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 새로운 마스크 그룹 생성용 구조체
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NewMaskGroup {
    pub annotation_id: i32,
    pub group_name: Option<String>,
    pub model_name: Option<String>,
    pub version: Option<String>,
    pub modality: Option<String>,
    pub slice_count: Option<i32>,
    pub mask_type: Option<String>,
    pub description: Option<String>,
    pub created_by: Option<i32>,
}

impl NewMaskGroup {
    /// 새로운 마스크 그룹 생성
    pub fn new(
        annotation_id: i32,
        group_name: Option<String>,
        model_name: Option<String>,
        version: Option<String>,
        modality: Option<String>,
        slice_count: i32,
        mask_type: String,
        description: Option<String>,
        created_by: Option<i32>,
    ) -> Self {
        Self {
            annotation_id,
            group_name,
            model_name,
            version,
            modality,
            slice_count: Some(slice_count),
            mask_type: Some(mask_type),
            description,
            created_by,
        }
    }

    /// 기본값으로 마스크 그룹 생성
    pub fn with_defaults(annotation_id: i32, created_by: Option<i32>) -> Self {
        Self {
            annotation_id,
            group_name: None,
            model_name: None,
            version: None,
            modality: None,
            slice_count: Some(1),
            mask_type: Some("segmentation".to_string()),
            description: None,
            created_by,
        }
    }

    /// AI 모델 정보가 포함된 마스크 그룹 생성
    pub fn with_ai_model(
        annotation_id: i32,
        group_name: String,
        model_name: String,
        version: String,
        modality: String,
        slice_count: i32,
        created_by: Option<i32>,
    ) -> Self {
        Self {
            annotation_id,
            group_name: Some(group_name),
            model_name: Some(model_name),
            version: Some(version),
            modality: Some(modality),
            slice_count: Some(slice_count),
            mask_type: Some("segmentation".to_string()),
            description: None,
            created_by,
        }
    }

    /// 수동 생성 마스크 그룹
    pub fn manual(
        annotation_id: i32,
        group_name: String,
        modality: String,
        slice_count: i32,
        description: Option<String>,
        created_by: Option<i32>,
    ) -> Self {
        Self {
            annotation_id,
            group_name: Some(group_name),
            model_name: None,
            version: None,
            modality: Some(modality),
            slice_count: Some(slice_count),
            mask_type: Some("manual".to_string()),
            description,
            created_by,
        }
    }
}

impl From<MaskGroup> for NewMaskGroup {
    fn from(mask_group: MaskGroup) -> Self {
        Self {
            annotation_id: mask_group.annotation_id,
            group_name: mask_group.group_name,
            model_name: mask_group.model_name,
            version: mask_group.version,
            modality: mask_group.modality,
            slice_count: mask_group.slice_count,
            mask_type: mask_group.mask_type,
            description: mask_group.description,
            created_by: mask_group.created_by,
        }
    }
}

/// 마스크 그룹 업데이트용 구조체
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UpdateMaskGroup {
    pub id: i32,
    pub group_name: Option<String>,
    pub model_name: Option<String>,
    pub version: Option<String>,
    pub modality: Option<String>,
    pub slice_count: Option<i32>,
    pub mask_type: Option<String>,
    pub description: Option<String>,
}

impl UpdateMaskGroup {
    /// 빈 업데이트 구조체 생성
    pub fn new(id: i32) -> Self {
        Self {
            id,
            group_name: None,
            model_name: None,
            version: None,
            modality: None,
            slice_count: None,
            mask_type: None,
            description: None,
        }
    }

    /// 그룹 이름 업데이트
    pub fn with_group_name(mut self, group_name: String) -> Self {
        self.group_name = Some(group_name);
        self
    }

    /// 모델 정보 업데이트
    pub fn with_model_info(mut self, model_name: String, version: String) -> Self {
        self.model_name = Some(model_name);
        self.version = Some(version);
        self
    }

    /// 모달리티 업데이트
    pub fn with_modality(mut self, modality: String) -> Self {
        self.modality = Some(modality);
        self
    }

    /// 슬라이스 개수 업데이트
    pub fn with_slice_count(mut self, slice_count: i32) -> Self {
        self.slice_count = Some(slice_count);
        self
    }

    /// 마스크 타입 업데이트
    pub fn with_mask_type(mut self, mask_type: String) -> Self {
        self.mask_type = Some(mask_type);
        self
    }

    /// 설명 업데이트
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}

impl Default for UpdateMaskGroup {
    fn default() -> Self {
        Self::new(0) // Default ID, should be set properly when used
    }
}

/// 마스크 그룹 통계 정보
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct MaskGroupStats {
    pub total_groups: i64,
    pub total_masks: i64,
    pub total_size_bytes: i64,
    pub modalities: HashMap<String, i64>,
    pub mask_types: HashMap<String, i64>,
}

impl MaskGroupStats {
    /// 빈 통계 정보 생성
    pub fn new() -> Self {
        Self {
            total_groups: 0,
            total_masks: 0,
            total_size_bytes: 0,
            modalities: HashMap::new(),
            mask_types: HashMap::new(),
        }
    }

    /// 모달리티별 통계 추가
    pub fn add_modality_count(&mut self, modality: String, count: i64) {
        *self.modalities.entry(modality).or_insert(0) += count;
    }

    /// 마스크 타입별 통계 추가
    pub fn add_mask_type_count(&mut self, mask_type: String, count: i64) {
        *self.mask_types.entry(mask_type).or_insert(0) += count;
    }
}

impl Default for MaskGroupStats {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_mask_group_creation() {
        let mask_group = NewMaskGroup::new(
            123,
            Some("Liver_Segmentation_v2".to_string()),
            Some("UNet".to_string()),
            Some("1.0.0".to_string()),
            Some("CT".to_string()),
            50,
            "segmentation".to_string(),
            Some("Liver segmentation for surgical planning".to_string()),
            Some(456),
        );

        assert_eq!(mask_group.annotation_id, 123);
        assert_eq!(mask_group.group_name, Some("Liver_Segmentation_v2".to_string()));
        assert_eq!(mask_group.model_name, Some("UNet".to_string()));
        assert_eq!(mask_group.version, Some("1.0.0".to_string()));
        assert_eq!(mask_group.modality, Some("CT".to_string()));
        assert_eq!(mask_group.slice_count, Some(50));
        assert_eq!(mask_group.mask_type, Some("segmentation".to_string()));
        assert_eq!(mask_group.description, Some("Liver segmentation for surgical planning".to_string()));
        assert_eq!(mask_group.created_by, Some(456));
    }

    #[test]
    fn test_new_mask_group_with_defaults() {
        let mask_group = NewMaskGroup::with_defaults(123, Some(456));

        assert_eq!(mask_group.annotation_id, 123);
        assert_eq!(mask_group.group_name, None);
        assert_eq!(mask_group.model_name, None);
        assert_eq!(mask_group.version, None);
        assert_eq!(mask_group.modality, None);
        assert_eq!(mask_group.slice_count, Some(1));
        assert_eq!(mask_group.mask_type, Some("segmentation".to_string()));
        assert_eq!(mask_group.description, None);
        assert_eq!(mask_group.created_by, Some(456));
    }

    #[test]
    fn test_new_mask_group_with_ai_model() {
        let mask_group = NewMaskGroup::with_ai_model(
            123,
            "Liver_Segmentation_v2".to_string(),
            "UNet".to_string(),
            "1.0.0".to_string(),
            "CT".to_string(),
            50,
            Some(456),
        );

        assert_eq!(mask_group.annotation_id, 123);
        assert_eq!(mask_group.group_name, Some("Liver_Segmentation_v2".to_string()));
        assert_eq!(mask_group.model_name, Some("UNet".to_string()));
        assert_eq!(mask_group.version, Some("1.0.0".to_string()));
        assert_eq!(mask_group.modality, Some("CT".to_string()));
        assert_eq!(mask_group.slice_count, Some(50));
        assert_eq!(mask_group.mask_type, Some("segmentation".to_string()));
        assert_eq!(mask_group.created_by, Some(456));
    }

    #[test]
    fn test_new_mask_group_manual() {
        let mask_group = NewMaskGroup::manual(
            123,
            "Manual_Liver_Segmentation".to_string(),
            "CT".to_string(),
            30,
            Some("Manually created liver segmentation".to_string()),
            Some(456),
        );

        assert_eq!(mask_group.annotation_id, 123);
        assert_eq!(mask_group.group_name, Some("Manual_Liver_Segmentation".to_string()));
        assert_eq!(mask_group.model_name, None);
        assert_eq!(mask_group.version, None);
        assert_eq!(mask_group.modality, Some("CT".to_string()));
        assert_eq!(mask_group.slice_count, Some(30));
        assert_eq!(mask_group.mask_type, Some("manual".to_string()));
        assert_eq!(mask_group.description, Some("Manually created liver segmentation".to_string()));
        assert_eq!(mask_group.created_by, Some(456));
    }

    #[test]
    fn test_update_mask_group_creation() {
        let update = UpdateMaskGroup::new(1)
            .with_group_name("Updated_Group".to_string())
            .with_model_info("UpdatedModel".to_string(), "2.0.0".to_string())
            .with_modality("MR".to_string())
            .with_slice_count(100)
            .with_mask_type("bounding_box".to_string())
            .with_description("Updated description".to_string());

        assert_eq!(update.group_name, Some("Updated_Group".to_string()));
        assert_eq!(update.model_name, Some("UpdatedModel".to_string()));
        assert_eq!(update.version, Some("2.0.0".to_string()));
        assert_eq!(update.modality, Some("MR".to_string()));
        assert_eq!(update.slice_count, Some(100));
        assert_eq!(update.mask_type, Some("bounding_box".to_string()));
        assert_eq!(update.description, Some("Updated description".to_string()));
    }

    #[test]
    fn test_mask_group_stats() {
        let mut stats = MaskGroupStats::new();
        stats.total_groups = 5;
        stats.total_masks = 150;
        stats.total_size_bytes = 1024000;
        stats.add_modality_count("CT".to_string(), 3);
        stats.add_modality_count("MR".to_string(), 2);
        stats.add_mask_type_count("segmentation".to_string(), 4);
        stats.add_mask_type_count("bounding_box".to_string(), 1);

        assert_eq!(stats.total_groups, 5);
        assert_eq!(stats.total_masks, 150);
        assert_eq!(stats.total_size_bytes, 1024000);
        assert_eq!(stats.modalities.get("CT"), Some(&3));
        assert_eq!(stats.modalities.get("MR"), Some(&2));
        assert_eq!(stats.mask_types.get("segmentation"), Some(&4));
        assert_eq!(stats.mask_types.get("bounding_box"), Some(&1));
    }

    #[test]
    fn test_mask_group_conversion() {
        let mask_group = MaskGroup {
            id: 1,
            annotation_id: 123,
            group_name: Some("Test_Group".to_string()),
            model_name: Some("TestModel".to_string()),
            version: Some("1.0.0".to_string()),
            modality: Some("CT".to_string()),
            slice_count: Some(50),
            mask_type: Some("segmentation".to_string()),
            description: Some("Test description".to_string()),
            created_by: Some(456),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let new_mask_group: NewMaskGroup = mask_group.into();

        assert_eq!(new_mask_group.annotation_id, 123);
        assert_eq!(new_mask_group.group_name, Some("Test_Group".to_string()));
        assert_eq!(new_mask_group.model_name, Some("TestModel".to_string()));
        assert_eq!(new_mask_group.version, Some("1.0.0".to_string()));
        assert_eq!(new_mask_group.modality, Some("CT".to_string()));
        assert_eq!(new_mask_group.slice_count, Some(50));
        assert_eq!(new_mask_group.mask_type, Some("segmentation".to_string()));
        assert_eq!(new_mask_group.description, Some("Test description".to_string()));
        assert_eq!(new_mask_group.created_by, Some(456));
    }
}
