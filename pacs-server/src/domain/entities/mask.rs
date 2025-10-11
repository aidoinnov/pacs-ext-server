use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 개별 마스크 파일 엔티티
/// 마스크 그룹 내의 개별 마스크 파일 정보를 관리
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Mask {
    pub id: i32,
    pub mask_group_id: i32,
    pub slice_index: Option<i32>,
    pub sop_instance_uid: Option<String>,
    pub label_name: Option<String>,
    pub file_path: String,
    pub mime_type: Option<String>,
    pub file_size: Option<i64>,
    pub checksum: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 새로운 마스크 생성용 구조체
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NewMask {
    pub mask_group_id: i32,
    pub slice_index: Option<i32>,
    pub sop_instance_uid: Option<String>,
    pub label_name: Option<String>,
    pub file_path: String,
    pub mime_type: Option<String>,
    pub file_size: Option<i64>,
    pub checksum: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
}

impl NewMask {
    /// 새로운 마스크 생성
    pub fn new(
        mask_group_id: i32,
        file_path: String,
        mime_type: String,
        slice_index: Option<i32>,
        sop_instance_uid: Option<String>,
        label_name: Option<String>,
        file_size: Option<i64>,
        checksum: Option<String>,
        width: Option<i32>,
        height: Option<i32>,
    ) -> Self {
        Self {
            mask_group_id,
            slice_index,
            sop_instance_uid,
            label_name,
            file_path,
            mime_type: Some(mime_type),
            file_size,
            checksum,
            width,
            height,
        }
    }

    /// 기본값으로 마스크 생성
    pub fn with_defaults(mask_group_id: i32, file_path: String) -> Self {
        Self {
            mask_group_id,
            slice_index: None,
            sop_instance_uid: None,
            label_name: None,
            file_path,
            mime_type: Some("image/png".to_string()),
            file_size: None,
            checksum: None,
            width: None,
            height: None,
        }
    }

    /// PNG 마스크 생성
    pub fn png(
        mask_group_id: i32,
        file_path: String,
        slice_index: Option<i32>,
        sop_instance_uid: Option<String>,
        label_name: Option<String>,
        file_size: Option<i64>,
        width: Option<i32>,
        height: Option<i32>,
    ) -> Self {
        Self {
            mask_group_id,
            slice_index,
            sop_instance_uid,
            label_name,
            file_path,
            mime_type: Some("image/png".to_string()),
            file_size,
            checksum: None,
            width,
            height,
        }
    }

    /// JPEG 마스크 생성
    pub fn jpeg(
        mask_group_id: i32,
        file_path: String,
        slice_index: Option<i32>,
        sop_instance_uid: Option<String>,
        label_name: Option<String>,
        file_size: Option<i64>,
        width: Option<i32>,
        height: Option<i32>,
    ) -> Self {
        Self {
            mask_group_id,
            slice_index,
            sop_instance_uid,
            label_name,
            file_path,
            mime_type: Some("image/jpeg".to_string()),
            file_size,
            checksum: None,
            width,
            height,
        }
    }

    /// DICOM 마스크 생성
    pub fn dicom(
        mask_group_id: i32,
        file_path: String,
        slice_index: Option<i32>,
        sop_instance_uid: String,
        label_name: Option<String>,
        file_size: Option<i64>,
        width: Option<i32>,
        height: Option<i32>,
    ) -> Self {
        Self {
            mask_group_id,
            slice_index,
            sop_instance_uid: Some(sop_instance_uid),
            label_name,
            file_path,
            mime_type: Some("application/dicom".to_string()),
            file_size,
            checksum: None,
            width,
            height,
        }
    }

    /// 체크섬 설정
    pub fn with_checksum(mut self, checksum: String) -> Self {
        self.checksum = Some(checksum);
        self
    }

    /// 파일 크기 설정
    pub fn with_file_size(mut self, file_size: i64) -> Self {
        self.file_size = Some(file_size);
        self
    }

    /// 이미지 크기 설정
    pub fn with_dimensions(mut self, width: i32, height: i32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }
}

impl From<Mask> for NewMask {
    fn from(mask: Mask) -> Self {
        Self {
            mask_group_id: mask.mask_group_id,
            slice_index: mask.slice_index,
            sop_instance_uid: mask.sop_instance_uid,
            label_name: mask.label_name,
            file_path: mask.file_path,
            mime_type: mask.mime_type,
            file_size: mask.file_size,
            checksum: mask.checksum,
            width: mask.width,
            height: mask.height,
        }
    }
}

/// 마스크 업데이트용 구조체
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UpdateMask {
    pub id: i32,
    pub slice_index: Option<i32>,
    pub sop_instance_uid: Option<String>,
    pub label_name: Option<String>,
    pub file_path: Option<String>,
    pub mime_type: Option<String>,
    pub file_size: Option<i64>,
    pub checksum: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
}

impl UpdateMask {
    /// 빈 업데이트 구조체 생성
    pub fn new(id: i32) -> Self {
        Self {
            id,
            slice_index: None,
            sop_instance_uid: None,
            label_name: None,
            file_path: None,
            mime_type: None,
            file_size: None,
            checksum: None,
            width: None,
            height: None,
        }
    }

    /// 슬라이스 인덱스 업데이트
    pub fn with_slice_index(mut self, slice_index: i32) -> Self {
        self.slice_index = Some(slice_index);
        self
    }

    /// SOP Instance UID 업데이트
    pub fn with_sop_instance_uid(mut self, sop_instance_uid: String) -> Self {
        self.sop_instance_uid = Some(sop_instance_uid);
        self
    }

    /// 라벨 이름 업데이트
    pub fn with_label_name(mut self, label_name: String) -> Self {
        self.label_name = Some(label_name);
        self
    }

    /// 파일 경로 업데이트
    pub fn with_file_path(mut self, file_path: String) -> Self {
        self.file_path = Some(file_path);
        self
    }

    /// MIME 타입 업데이트
    pub fn with_mime_type(mut self, mime_type: String) -> Self {
        self.mime_type = Some(mime_type);
        self
    }

    /// 파일 크기 업데이트
    pub fn with_file_size(mut self, file_size: i64) -> Self {
        self.file_size = Some(file_size);
        self
    }

    /// 체크섬 업데이트
    pub fn with_checksum(mut self, checksum: String) -> Self {
        self.checksum = Some(checksum);
        self
    }

    /// 이미지 크기 업데이트
    pub fn with_dimensions(mut self, width: i32, height: i32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }
}

impl Default for UpdateMask {
    fn default() -> Self {
        Self::new(0) // Default ID, should be set properly when used
    }
}

/// 마스크 통계 정보
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MaskStats {
    pub total_masks: i64,
    pub total_size_bytes: i64,
    pub mime_types: std::collections::HashMap<String, i64>,
    pub label_names: std::collections::HashMap<String, i64>,
    pub average_file_size: f64,
    pub largest_file_size: i64,
    pub smallest_file_size: i64,
}

impl MaskStats {
    /// 빈 통계 정보 생성
    pub fn new() -> Self {
        Self {
            total_masks: 0,
            total_size_bytes: 0,
            mime_types: std::collections::HashMap::new(),
            label_names: std::collections::HashMap::new(),
            average_file_size: 0.0,
            largest_file_size: 0,
            smallest_file_size: 0,
        }
    }

    /// MIME 타입별 통계 추가
    pub fn add_mime_type_count(&mut self, mime_type: String, count: i64) {
        *self.mime_types.entry(mime_type).or_insert(0) += count;
    }

    /// 라벨 이름별 통계 추가
    pub fn add_label_name_count(&mut self, label_name: String, count: i64) {
        *self.label_names.entry(label_name).or_insert(0) += count;
    }

    /// 평균 파일 크기 계산
    pub fn calculate_average_file_size(&mut self) {
        if self.total_masks > 0 {
            self.average_file_size = self.total_size_bytes as f64 / self.total_masks as f64;
        }
    }
}

impl Default for MaskStats {
    fn default() -> Self {
        Self::new()
    }
}

/// 마스크 파일 정보 (업로드용)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MaskFileInfo {
    pub file_name: String,
    pub mime_type: Option<String>,
    pub file_size: i64,
    pub checksum: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub slice_index: Option<i32>,
    pub sop_instance_uid: Option<String>,
    pub label_name: Option<String>,
}

impl MaskFileInfo {
    /// 새로운 마스크 파일 정보 생성
    pub fn new(
        file_name: String,
        mime_type: String,
        file_size: i64,
        checksum: Option<String>,
        width: Option<i32>,
        height: Option<i32>,
        slice_index: Option<i32>,
        sop_instance_uid: Option<String>,
        label_name: Option<String>,
    ) -> Self {
        Self {
            file_name,
            mime_type: Some(mime_type),
            file_size,
            checksum,
            width,
            height,
            slice_index,
            sop_instance_uid,
            label_name,
        }
    }

    /// PNG 파일 정보 생성
    pub fn png(
        file_name: String,
        file_size: i64,
        width: i32,
        height: i32,
        slice_index: Option<i32>,
        sop_instance_uid: Option<String>,
        label_name: Option<String>,
    ) -> Self {
        Self {
            file_name,
            mime_type: Some("image/png".to_string()),
            file_size,
            checksum: None,
            width: Some(width),
            height: Some(height),
            slice_index,
            sop_instance_uid,
            label_name,
        }
    }

    /// JPEG 파일 정보 생성
    pub fn jpeg(
        file_name: String,
        file_size: i64,
        width: i32,
        height: i32,
        slice_index: Option<i32>,
        sop_instance_uid: Option<String>,
        label_name: Option<String>,
    ) -> Self {
        Self {
            file_name,
            mime_type: Some("image/jpeg".to_string()),
            file_size,
            checksum: None,
            width: Some(width),
            height: Some(height),
            slice_index,
            sop_instance_uid,
            label_name,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_mask_creation() {
        let mask = NewMask::new(
            123,
            "mask/group123/slice_001.png".to_string(),
            "image/png".to_string(),
            Some(1),
            Some("1.2.3.4.5.6.7.8.9.10".to_string()),
            Some("liver".to_string()),
            Some(1024),
            Some("abc123".to_string()),
            Some(512),
            Some(512),
        );

        assert_eq!(mask.mask_group_id, 123);
        assert_eq!(mask.file_path, "mask/group123/slice_001.png");
        assert_eq!(mask.mime_type, Some("image/png".to_string()));
        assert_eq!(mask.slice_index, Some(1));
        assert_eq!(mask.sop_instance_uid, Some("1.2.3.4.5.6.7.8.9.10".to_string()));
        assert_eq!(mask.label_name, Some("liver".to_string()));
        assert_eq!(mask.file_size, Some(1024));
        assert_eq!(mask.checksum, Some("abc123".to_string()));
        assert_eq!(mask.width, Some(512));
        assert_eq!(mask.height, Some(512));
    }

    #[test]
    fn test_new_mask_with_defaults() {
        let mask = NewMask::with_defaults(123, "mask/group123/slice_001.png".to_string());

        assert_eq!(mask.mask_group_id, 123);
        assert_eq!(mask.file_path, "mask/group123/slice_001.png");
        assert_eq!(mask.mime_type, Some("image/png".to_string()));
        assert_eq!(mask.slice_index, None);
        assert_eq!(mask.sop_instance_uid, None);
        assert_eq!(mask.label_name, None);
        assert_eq!(mask.file_size, None);
        assert_eq!(mask.checksum, None);
        assert_eq!(mask.width, None);
        assert_eq!(mask.height, None);
    }

    #[test]
    fn test_new_mask_png() {
        let mask = NewMask::png(
            123,
            "mask/group123/slice_001.png".to_string(),
            Some(1),
            Some("1.2.3.4.5.6.7.8.9.10".to_string()),
            Some("liver".to_string()),
            Some(1024),
            Some(512),
            Some(512),
        );

        assert_eq!(mask.mask_group_id, 123);
        assert_eq!(mask.file_path, "mask/group123/slice_001.png");
        assert_eq!(mask.mime_type, Some("image/png".to_string()));
        assert_eq!(mask.slice_index, Some(1));
        assert_eq!(mask.sop_instance_uid, Some("1.2.3.4.5.6.7.8.9.10".to_string()));
        assert_eq!(mask.label_name, Some("liver".to_string()));
        assert_eq!(mask.file_size, Some(1024));
        assert_eq!(mask.width, Some(512));
        assert_eq!(mask.height, Some(512));
    }

    #[test]
    fn test_new_mask_dicom() {
        let mask = NewMask::dicom(
            123,
            "mask/group123/slice_001.dcm".to_string(),
            Some(1),
            "1.2.3.4.5.6.7.8.9.10".to_string(),
            Some("liver".to_string()),
            Some(2048),
            Some(512),
            Some(512),
        );

        assert_eq!(mask.mask_group_id, 123);
        assert_eq!(mask.file_path, "mask/group123/slice_001.dcm");
        assert_eq!(mask.mime_type, Some("application/dicom".to_string()));
        assert_eq!(mask.slice_index, Some(1));
        assert_eq!(mask.sop_instance_uid, Some("1.2.3.4.5.6.7.8.9.10".to_string()));
        assert_eq!(mask.label_name, Some("liver".to_string()));
        assert_eq!(mask.file_size, Some(2048));
        assert_eq!(mask.width, Some(512));
        assert_eq!(mask.height, Some(512));
    }

    #[test]
    fn test_new_mask_with_checksum() {
        let mask = NewMask::with_defaults(123, "mask/group123/slice_001.png".to_string())
            .with_checksum("abc123".to_string())
            .with_file_size(1024)
            .with_dimensions(512, 512);

        assert_eq!(mask.checksum, Some("abc123".to_string()));
        assert_eq!(mask.file_size, Some(1024));
        assert_eq!(mask.width, Some(512));
        assert_eq!(mask.height, Some(512));
    }

    #[test]
    fn test_update_mask_creation() {
        let update = UpdateMask::new(1)
            .with_slice_index(5)
            .with_sop_instance_uid("1.2.3.4.5.6.7.8.9.10".to_string())
            .with_label_name("spleen".to_string())
            .with_file_path("mask/group123/slice_005.png".to_string())
            .with_mime_type("image/png".to_string())
            .with_file_size(2048)
            .with_checksum("def456".to_string())
            .with_dimensions(1024, 1024);

        assert_eq!(update.slice_index, Some(5));
        assert_eq!(update.sop_instance_uid, Some("1.2.3.4.5.6.7.8.9.10".to_string()));
        assert_eq!(update.label_name, Some("spleen".to_string()));
        assert_eq!(update.file_path, Some("mask/group123/slice_005.png".to_string()));
        assert_eq!(update.mime_type, Some("image/png".to_string()));
        assert_eq!(update.file_size, Some(2048));
        assert_eq!(update.checksum, Some("def456".to_string()));
        assert_eq!(update.width, Some(1024));
        assert_eq!(update.height, Some(1024));
    }

    #[test]
    fn test_mask_stats() {
        let mut stats = MaskStats::new();
        stats.total_masks = 10;
        stats.total_size_bytes = 10240;
        stats.add_mime_type_count("image/png".to_string(), 8);
        stats.add_mime_type_count("image/jpeg".to_string(), 2);
        stats.add_label_name_count("liver".to_string(), 6);
        stats.add_label_name_count("spleen".to_string(), 4);
        stats.calculate_average_file_size();

        assert_eq!(stats.total_masks, 10);
        assert_eq!(stats.total_size_bytes, 10240);
        assert_eq!(stats.average_file_size, 1024.0);
        assert_eq!(stats.mime_types.get("image/png"), Some(&8));
        assert_eq!(stats.mime_types.get("image/jpeg"), Some(&2));
        assert_eq!(stats.label_names.get("liver"), Some(&6));
        assert_eq!(stats.label_names.get("spleen"), Some(&4));
    }

    #[test]
    fn test_mask_file_info_png() {
        let file_info = MaskFileInfo::png(
            "slice_001.png".to_string(),
            1024,
            512,
            512,
            Some(1),
            Some("1.2.3.4.5.6.7.8.9.10".to_string()),
            Some("liver".to_string()),
        );

        assert_eq!(file_info.file_name, "slice_001.png");
        assert_eq!(file_info.mime_type, Some("image/png".to_string()));
        assert_eq!(file_info.file_size, 1024);
        assert_eq!(file_info.width, Some(512));
        assert_eq!(file_info.height, Some(512));
        assert_eq!(file_info.slice_index, Some(1));
        assert_eq!(file_info.sop_instance_uid, Some("1.2.3.4.5.6.7.8.9.10".to_string()));
        assert_eq!(file_info.label_name, Some("liver".to_string()));
    }

    #[test]
    fn test_mask_conversion() {
        let mask = Mask {
            id: 1,
            mask_group_id: 123,
            slice_index: Some(1),
            sop_instance_uid: Some("1.2.3.4.5.6.7.8.9.10".to_string()),
            label_name: Some("liver".to_string()),
            file_path: "mask/group123/slice_001.png".to_string(),
            mime_type: Some("image/png".to_string()),
            file_size: Some(1024),
            checksum: Some("abc123".to_string()),
            width: Some(512),
            height: Some(512),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let new_mask: NewMask = mask.into();

        assert_eq!(new_mask.mask_group_id, 123);
        assert_eq!(new_mask.slice_index, Some(1));
        assert_eq!(new_mask.sop_instance_uid, Some("1.2.3.4.5.6.7.8.9.10".to_string()));
        assert_eq!(new_mask.label_name, Some("liver".to_string()));
        assert_eq!(new_mask.file_path, "mask/group123/slice_001.png");
        assert_eq!(new_mask.mime_type, Some("image/png".to_string()));
        assert_eq!(new_mask.file_size, Some(1024));
        assert_eq!(new_mask.checksum, Some("abc123".to_string()));
        assert_eq!(new_mask.width, Some(512));
        assert_eq!(new_mask.height, Some(512));
    }
}
