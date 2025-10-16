//! # 마스크 엔티티 모듈
//! 
//! 이 모듈은 의료 영상의 개별 마스크 파일 정보를 나타내는 엔티티들을 정의합니다.
//! 마스크는 의료 영상에서 특정 영역을 표시하는 이미지 파일로, AI 모델이나 의료진에 의해 생성됩니다.

// UTC 시간대의 날짜/시간 처리를 위한 chrono 라이브러리
use chrono::{DateTime, Utc};
// JSON 직렬화/역직렬화를 위한 serde 라이브러리
use serde::{Deserialize, Serialize};

/// 개별 마스크 파일을 나타내는 엔티티
/// 
/// 이 구조체는 데이터베이스의 `annotation_mask` 테이블과 매핑되며,
/// 마스크 그룹 내의 개별 마스크 파일 정보를 관리합니다.
/// 
/// # 필드
/// - `id`: 데이터베이스에서 자동 생성되는 고유 식별자
/// - `mask_group_id`: 마스크가 속한 마스크 그룹의 ID
/// - `slice_index`: 슬라이스 인덱스 (선택사항)
/// - `sop_instance_uid`: DICOM SOP Instance UID (선택사항)
/// - `label_name`: 마스크의 라벨 이름 (선택사항)
/// - `file_path`: 마스크 파일의 저장 경로
/// - `mime_type`: 파일의 MIME 타입 (선택사항)
/// - `file_size`: 파일 크기 (바이트, 선택사항)
/// - `checksum`: 파일의 체크섬 (선택사항)
/// - `width`: 이미지 너비 (픽셀, 선택사항)
/// - `height`: 이미지 높이 (픽셀, 선택사항)
/// - `created_at`: 마스크가 생성된 시각
/// - `updated_at`: 마스크가 마지막으로 수정된 시각
/// 
/// # 예시
/// ```rust
/// let mask = Mask {
///     id: 1,
///     mask_group_id: 1,
///     slice_index: Some(1),
///     sop_instance_uid: Some("1.2.3.4.5.6.7.8.9.10".to_string()),
///     label_name: Some("liver".to_string()),
///     file_path: "mask/group1/slice_001.png".to_string(),
///     mime_type: Some("image/png".to_string()),
///     file_size: Some(1024),
///     checksum: Some("abc123".to_string()),
///     width: Some(512),
///     height: Some(512),
///     created_at: Utc::now(),
///     updated_at: Utc::now(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Mask {
    /// 데이터베이스에서 자동 생성되는 고유 식별자
    pub id: i32,
    /// 마스크가 속한 마스크 그룹의 ID
    pub mask_group_id: i32,
    /// 슬라이스 인덱스 (선택사항)
    pub slice_index: Option<i32>,
    /// DICOM SOP Instance UID (선택사항)
    pub sop_instance_uid: Option<String>,
    /// 마스크의 라벨 이름 (선택사항)
    pub label_name: Option<String>,
    /// 마스크 파일의 저장 경로
    pub file_path: String,
    /// 파일의 MIME 타입 (선택사항)
    pub mime_type: Option<String>,
    /// 파일 크기 (바이트, 선택사항)
    pub file_size: Option<i64>,
    /// 파일의 체크섬 (선택사항)
    pub checksum: Option<String>,
    /// 이미지 너비 (픽셀, 선택사항)
    pub width: Option<i32>,
    /// 이미지 높이 (픽셀, 선택사항)
    pub height: Option<i32>,
    /// 마스크가 생성된 시각
    pub created_at: DateTime<Utc>,
    /// 마스크가 마지막으로 수정된 시각
    pub updated_at: DateTime<Utc>,
}

/// 새로운 마스크 생성을 위한 DTO(Data Transfer Object)
/// 
/// 이 구조체는 마스크 생성 요청 시 전달되는 데이터를 나타냅니다.
/// 데이터베이스에 저장되기 전의 마스크 정보를 담고 있습니다.
/// 
/// # 필드
/// - `mask_group_id`: 마스크가 속할 마스크 그룹의 ID
/// - `slice_index`: 슬라이스 인덱스 (선택사항)
/// - `sop_instance_uid`: DICOM SOP Instance UID (선택사항)
/// - `label_name`: 마스크의 라벨 이름 (선택사항)
/// - `file_path`: 마스크 파일의 저장 경로
/// - `mime_type`: 파일의 MIME 타입 (선택사항)
/// - `file_size`: 파일 크기 (바이트, 선택사항)
/// - `checksum`: 파일의 체크섬 (선택사항)
/// - `width`: 이미지 너비 (픽셀, 선택사항)
/// - `height`: 이미지 높이 (픽셀, 선택사항)
/// 
/// # 예시
/// ```rust
/// let new_mask = NewMask {
///     mask_group_id: 1,
///     slice_index: Some(1),
///     sop_instance_uid: Some("1.2.3.4.5.6.7.8.9.10".to_string()),
///     label_name: Some("liver".to_string()),
///     file_path: "mask/group1/slice_001.png".to_string(),
///     mime_type: Some("image/png".to_string()),
///     file_size: Some(1024),
///     checksum: Some("abc123".to_string()),
///     width: Some(512),
///     height: Some(512),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NewMask {
    /// 마스크가 속할 마스크 그룹의 ID
    pub mask_group_id: i32,
    /// 슬라이스 인덱스 (선택사항)
    pub slice_index: Option<i32>,
    /// DICOM SOP Instance UID (선택사항)
    pub sop_instance_uid: Option<String>,
    /// 마스크의 라벨 이름 (선택사항)
    pub label_name: Option<String>,
    /// 마스크 파일의 저장 경로
    pub file_path: String,
    /// 파일의 MIME 타입 (선택사항)
    pub mime_type: Option<String>,
    /// 파일 크기 (바이트, 선택사항)
    pub file_size: Option<i64>,
    /// 파일의 체크섬 (선택사항)
    pub checksum: Option<String>,
    /// 이미지 너비 (픽셀, 선택사항)
    pub width: Option<i32>,
    /// 이미지 높이 (픽셀, 선택사항)
    pub height: Option<i32>,
}

impl NewMask {
    /// 새로운 마스크를 생성합니다.
    /// 
    /// # 매개변수
    /// - `mask_group_id`: 마스크가 속할 마스크 그룹의 ID
    /// - `file_path`: 마스크 파일의 저장 경로
    /// - `mime_type`: 파일의 MIME 타입
    /// - `slice_index`: 슬라이스 인덱스 (선택사항)
    /// - `sop_instance_uid`: DICOM SOP Instance UID (선택사항)
    /// - `label_name`: 마스크의 라벨 이름 (선택사항)
    /// - `file_size`: 파일 크기 (바이트, 선택사항)
    /// - `checksum`: 파일의 체크섬 (선택사항)
    /// - `width`: 이미지 너비 (픽셀, 선택사항)
    /// - `height`: 이미지 높이 (픽셀, 선택사항)
    /// 
    /// # 반환값
    /// 생성된 `NewMask` 인스턴스
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

    /// 기본값으로 마스크를 생성합니다.
    /// 
    /// 이 메서드는 가장 기본적인 설정으로 마스크를 생성합니다:
    /// - MIME 타입: "image/png"
    /// - 기타 필드들은 None으로 설정
    /// 
    /// # 매개변수
    /// - `mask_group_id`: 마스크가 속할 마스크 그룹의 ID
    /// - `file_path`: 마스크 파일의 저장 경로
    /// 
    /// # 반환값
    /// 기본값으로 설정된 `NewMask` 인스턴스
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

    /// PNG 형식의 마스크를 생성합니다.
    /// 
    /// 이 메서드는 PNG 이미지 파일을 위한 특화된 생성자입니다.
    /// 
    /// # 매개변수
    /// - `mask_group_id`: 마스크가 속할 마스크 그룹의 ID
    /// - `file_path`: 마스크 파일의 저장 경로
    /// - `slice_index`: 슬라이스 인덱스 (선택사항)
    /// - `sop_instance_uid`: DICOM SOP Instance UID (선택사항)
    /// - `label_name`: 마스크의 라벨 이름 (선택사항)
    /// - `file_size`: 파일 크기 (바이트, 선택사항)
    /// - `width`: 이미지 너비 (픽셀, 선택사항)
    /// - `height`: 이미지 높이 (픽셀, 선택사항)
    /// 
    /// # 반환값
    /// PNG 형식으로 설정된 `NewMask` 인스턴스
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

    /// JPEG 형식의 마스크를 생성합니다.
    /// 
    /// 이 메서드는 JPEG 이미지 파일을 위한 특화된 생성자입니다.
    /// 
    /// # 매개변수
    /// - `mask_group_id`: 마스크가 속할 마스크 그룹의 ID
    /// - `file_path`: 마스크 파일의 저장 경로
    /// - `slice_index`: 슬라이스 인덱스 (선택사항)
    /// - `sop_instance_uid`: DICOM SOP Instance UID (선택사항)
    /// - `label_name`: 마스크의 라벨 이름 (선택사항)
    /// - `file_size`: 파일 크기 (바이트, 선택사항)
    /// - `width`: 이미지 너비 (픽셀, 선택사항)
    /// - `height`: 이미지 높이 (픽셀, 선택사항)
    /// 
    /// # 반환값
    /// JPEG 형식으로 설정된 `NewMask` 인스턴스
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

    /// DICOM 형식의 마스크를 생성합니다.
    /// 
    /// 이 메서드는 DICOM 파일을 위한 특화된 생성자입니다.
    /// DICOM 파일은 SOP Instance UID가 필수이므로 매개변수로 받습니다.
    /// 
    /// # 매개변수
    /// - `mask_group_id`: 마스크가 속할 마스크 그룹의 ID
    /// - `file_path`: 마스크 파일의 저장 경로
    /// - `slice_index`: 슬라이스 인덱스 (선택사항)
    /// - `sop_instance_uid`: DICOM SOP Instance UID (필수)
    /// - `label_name`: 마스크의 라벨 이름 (선택사항)
    /// - `file_size`: 파일 크기 (바이트, 선택사항)
    /// - `width`: 이미지 너비 (픽셀, 선택사항)
    /// - `height`: 이미지 높이 (픽셀, 선택사항)
    /// 
    /// # 반환값
    /// DICOM 형식으로 설정된 `NewMask` 인스턴스
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

    /// 체크섬을 설정합니다.
    /// 
    /// # 매개변수
    /// - `checksum`: 파일의 체크섬 값
    /// 
    /// # 반환값
    /// 체크섬이 설정된 `NewMask` 인스턴스
    pub fn with_checksum(mut self, checksum: String) -> Self {
        self.checksum = Some(checksum);
        self
    }

    /// 파일 크기를 설정합니다.
    /// 
    /// # 매개변수
    /// - `file_size`: 파일 크기 (바이트)
    /// 
    /// # 반환값
    /// 파일 크기가 설정된 `NewMask` 인스턴스
    pub fn with_file_size(mut self, file_size: i64) -> Self {
        self.file_size = Some(file_size);
        self
    }

    /// 이미지 크기를 설정합니다.
    /// 
    /// # 매개변수
    /// - `width`: 이미지 너비 (픽셀)
    /// - `height`: 이미지 높이 (픽셀)
    /// 
    /// # 반환값
    /// 이미지 크기가 설정된 `NewMask` 인스턴스
    pub fn with_dimensions(mut self, width: i32, height: i32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }
}

/// `Mask`에서 `NewMask`으로의 변환을 위한 From 트레이트 구현
/// 
/// 이 구현은 기존의 `Mask` 엔티티를 `NewMask` DTO로 변환할 때 사용됩니다.
/// 주로 업데이트 작업에서 기존 데이터를 기반으로 새로운 데이터를 생성할 때 활용됩니다.
impl From<Mask> for NewMask {
    /// `Mask`을 `NewMask`으로 변환합니다.
    /// 
    /// # 매개변수
    /// - `mask`: 변환할 `Mask` 인스턴스
    /// 
    /// # 반환값
    /// 변환된 `NewMask` 인스턴스
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

/// 마스크 업데이트를 위한 DTO(Data Transfer Object)
/// 
/// 이 구조체는 마스크 업데이트 요청 시 전달되는 데이터를 나타냅니다.
/// 업데이트할 필드만 포함하며, None인 필드는 업데이트하지 않습니다.
/// 
/// # 필드
/// - `id`: 업데이트할 마스크의 ID
/// - `slice_index`: 새로운 슬라이스 인덱스 (선택사항)
/// - `sop_instance_uid`: 새로운 DICOM SOP Instance UID (선택사항)
/// - `label_name`: 새로운 라벨 이름 (선택사항)
/// - `file_path`: 새로운 파일 경로 (선택사항)
/// - `mime_type`: 새로운 MIME 타입 (선택사항)
/// - `file_size`: 새로운 파일 크기 (선택사항)
/// - `checksum`: 새로운 체크섬 (선택사항)
/// - `width`: 새로운 이미지 너비 (선택사항)
/// - `height`: 새로운 이미지 높이 (선택사항)
/// 
/// # 예시
/// ```rust
/// let update = UpdateMask::new(1)
///     .with_slice_index(5)
///     .with_label_name("spleen".to_string())
///     .with_file_size(2048);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UpdateMask {
    /// 업데이트할 마스크의 ID
    pub id: i32,
    /// 새로운 슬라이스 인덱스 (선택사항)
    pub slice_index: Option<i32>,
    /// 새로운 DICOM SOP Instance UID (선택사항)
    pub sop_instance_uid: Option<String>,
    /// 새로운 라벨 이름 (선택사항)
    pub label_name: Option<String>,
    /// 새로운 파일 경로 (선택사항)
    pub file_path: Option<String>,
    /// 새로운 MIME 타입 (선택사항)
    pub mime_type: Option<String>,
    /// 새로운 파일 크기 (선택사항)
    pub file_size: Option<i64>,
    /// 새로운 체크섬 (선택사항)
    pub checksum: Option<String>,
    /// 새로운 이미지 너비 (선택사항)
    pub width: Option<i32>,
    /// 새로운 이미지 높이 (선택사항)
    pub height: Option<i32>,
}

impl UpdateMask {
    /// 빈 업데이트 구조체를 생성합니다.
    /// 
    /// 이 메서드는 지정된 ID를 가진 빈 업데이트 구조체를 생성합니다.
    /// 모든 필드는 None으로 설정되어 있어, 필요한 필드만 체이닝 메서드로 설정할 수 있습니다.
    /// 
    /// # 매개변수
    /// - `id`: 업데이트할 마스크의 ID
    /// 
    /// # 반환값
    /// 빈 필드로 초기화된 `UpdateMask` 인스턴스
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

    /// 슬라이스 인덱스를 설정합니다.
    /// 
    /// # 매개변수
    /// - `slice_index`: 새로운 슬라이스 인덱스
    /// 
    /// # 반환값
    /// 슬라이스 인덱스가 설정된 `UpdateMask` 인스턴스
    pub fn with_slice_index(mut self, slice_index: i32) -> Self {
        self.slice_index = Some(slice_index);
        self
    }

    /// SOP Instance UID를 설정합니다.
    /// 
    /// # 매개변수
    /// - `sop_instance_uid`: 새로운 DICOM SOP Instance UID
    /// 
    /// # 반환값
    /// SOP Instance UID가 설정된 `UpdateMask` 인스턴스
    pub fn with_sop_instance_uid(mut self, sop_instance_uid: String) -> Self {
        self.sop_instance_uid = Some(sop_instance_uid);
        self
    }

    /// 라벨 이름을 설정합니다.
    /// 
    /// # 매개변수
    /// - `label_name`: 새로운 라벨 이름
    /// 
    /// # 반환값
    /// 라벨 이름이 설정된 `UpdateMask` 인스턴스
    pub fn with_label_name(mut self, label_name: String) -> Self {
        self.label_name = Some(label_name);
        self
    }

    /// 파일 경로를 설정합니다.
    /// 
    /// # 매개변수
    /// - `file_path`: 새로운 파일 경로
    /// 
    /// # 반환값
    /// 파일 경로가 설정된 `UpdateMask` 인스턴스
    pub fn with_file_path(mut self, file_path: String) -> Self {
        self.file_path = Some(file_path);
        self
    }

    /// MIME 타입을 설정합니다.
    /// 
    /// # 매개변수
    /// - `mime_type`: 새로운 MIME 타입
    /// 
    /// # 반환값
    /// MIME 타입이 설정된 `UpdateMask` 인스턴스
    pub fn with_mime_type(mut self, mime_type: String) -> Self {
        self.mime_type = Some(mime_type);
        self
    }

    /// 파일 크기를 설정합니다.
    /// 
    /// # 매개변수
    /// - `file_size`: 새로운 파일 크기 (바이트)
    /// 
    /// # 반환값
    /// 파일 크기가 설정된 `UpdateMask` 인스턴스
    pub fn with_file_size(mut self, file_size: i64) -> Self {
        self.file_size = Some(file_size);
        self
    }

    /// 체크섬을 설정합니다.
    /// 
    /// # 매개변수
    /// - `checksum`: 새로운 체크섬 (MD5, SHA256 등)
    /// 
    /// # 반환값
    /// 체크섬이 설정된 `UpdateMask` 인스턴스
    pub fn with_checksum(mut self, checksum: String) -> Self {
        self.checksum = Some(checksum);
        self
    }

    /// 이미지 크기를 설정합니다.
    /// 
    /// # 매개변수
    /// - `width`: 새로운 이미지 너비 (픽셀)
    /// - `height`: 새로운 이미지 높이 (픽셀)
    /// 
    /// # 반환값
    /// 이미지 크기가 설정된 `UpdateMask` 인스턴스
    pub fn with_dimensions(mut self, width: i32, height: i32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }
}

/// `UpdateMask`의 기본값을 제공하는 Default 트레이트 구현
/// 
/// 이 구현은 ID가 0인 빈 업데이트 구조체를 반환합니다.
/// 실제 사용 시에는 올바른 ID로 설정해야 합니다.
impl Default for UpdateMask {
    /// 기본값으로 `UpdateMask`를 생성합니다.
    /// 
    /// # 반환값
    /// ID가 0으로 설정된 빈 `UpdateMask` 인스턴스
    /// 
    /// # 주의사항
    /// 실제 사용 시에는 올바른 ID로 설정해야 합니다.
    fn default() -> Self {
        Self::new(0) // Default ID, should be set properly when used
    }
}

/// 마스크 통계 정보를 나타내는 구조체
/// 
/// 이 구조체는 마스크와 관련된 다양한 통계 정보를 저장합니다.
/// API 응답에서 사용되며, OpenAPI 스키마 생성에도 활용됩니다.
/// 
/// # 필드
/// - `total_masks`: 전체 마스크 파일 수
/// - `total_size_bytes`: 전체 마스크 파일 크기 (바이트)
/// - `mime_types`: MIME 타입별 마스크 파일 수 (해시맵)
/// - `label_names`: 라벨 이름별 마스크 파일 수 (해시맵)
/// - `average_file_size`: 평균 파일 크기 (바이트)
/// - `largest_file_size`: 가장 큰 파일 크기 (바이트)
/// - `smallest_file_size`: 가장 작은 파일 크기 (바이트)
/// 
/// # 예시
/// ```rust
/// let mut stats = MaskStats::new();
/// stats.total_masks = 100;
/// stats.total_size_bytes = 1024000;
/// stats.add_mime_type_count("image/png".to_string(), 80);
/// stats.add_label_name_count("liver".to_string(), 60);
/// stats.calculate_average_file_size();
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MaskStats {
    /// 전체 마스크 파일 수
    pub total_masks: i64,
    /// 전체 마스크 파일 크기 (바이트)
    pub total_size_bytes: i64,
    /// MIME 타입별 마스크 파일 수 (해시맵)
    pub mime_types: std::collections::HashMap<String, i64>,
    /// 라벨 이름별 마스크 파일 수 (해시맵)
    pub label_names: std::collections::HashMap<String, i64>,
    /// 평균 파일 크기 (바이트)
    pub average_file_size: f64,
    /// 가장 큰 파일 크기 (바이트)
    pub largest_file_size: i64,
    /// 가장 작은 파일 크기 (바이트)
    pub smallest_file_size: i64,
}

impl MaskStats {
    /// 빈 통계 정보를 생성합니다.
    /// 
    /// 이 메서드는 모든 필드가 0 또는 빈 해시맵으로 초기화된 통계 구조체를 생성합니다.
    /// 
    /// # 반환값
    /// 빈 필드로 초기화된 `MaskStats` 인스턴스
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

    /// MIME 타입별 통계를 추가합니다.
    /// 
    /// 이 메서드는 특정 MIME 타입의 마스크 파일 수를 증가시킵니다.
    /// 해당 MIME 타입이 이미 존재하면 기존 값에 추가하고, 없으면 새로 생성합니다.
    /// 
    /// # 매개변수
    /// - `mime_type`: MIME 타입 (예: "image/png", "image/jpeg")
    /// - `count`: 추가할 마스크 파일 수
    /// 
    /// # 예시
    /// ```rust
    /// let mut stats = MaskStats::new();
    /// stats.add_mime_type_count("image/png".to_string(), 5);
    /// stats.add_mime_type_count("image/png".to_string(), 3); // PNG는 총 8개가 됨
    /// ```
    pub fn add_mime_type_count(&mut self, mime_type: String, count: i64) {
        *self.mime_types.entry(mime_type).or_insert(0) += count;
    }

    /// 라벨 이름별 통계를 추가합니다.
    /// 
    /// 이 메서드는 특정 라벨 이름의 마스크 파일 수를 증가시킵니다.
    /// 해당 라벨 이름이 이미 존재하면 기존 값에 추가하고, 없으면 새로 생성합니다.
    /// 
    /// # 매개변수
    /// - `label_name`: 라벨 이름 (예: "liver", "spleen", "kidney")
    /// - `count`: 추가할 마스크 파일 수
    /// 
    /// # 예시
    /// ```rust
    /// let mut stats = MaskStats::new();
    /// stats.add_label_name_count("liver".to_string(), 10);
    /// stats.add_label_name_count("spleen".to_string(), 5);
    /// ```
    pub fn add_label_name_count(&mut self, label_name: String, count: i64) {
        *self.label_names.entry(label_name).or_insert(0) += count;
    }

    /// 평균 파일 크기를 계산합니다.
    /// 
    /// 이 메서드는 전체 파일 크기를 전체 파일 수로 나누어 평균 파일 크기를 계산합니다.
    /// 파일이 없는 경우 평균은 0.0으로 유지됩니다.
    /// 
    /// # 예시
    /// ```rust
    /// let mut stats = MaskStats::new();
    /// stats.total_masks = 100;
    /// stats.total_size_bytes = 1024000;
    /// stats.calculate_average_file_size(); // 평균: 10240.0 바이트
    /// ```
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

/// 마스크 파일 정보를 나타내는 구조체 (업로드용)
/// 
/// 이 구조체는 마스크 파일 업로드 시 필요한 메타데이터를 저장합니다.
/// 파일 업로드 API에서 사용되며, 파일 검증 및 처리에 활용됩니다.
/// 
/// # 필드
/// - `file_name`: 업로드할 파일의 이름
/// - `mime_type`: 파일의 MIME 타입 (선택사항)
/// - `file_size`: 파일 크기 (바이트)
/// - `checksum`: 파일의 체크섬 (선택사항)
/// - `width`: 이미지 너비 (픽셀, 선택사항)
/// - `height`: 이미지 높이 (픽셀, 선택사항)
/// - `slice_index`: 슬라이스 인덱스 (선택사항)
/// - `sop_instance_uid`: DICOM SOP Instance UID (선택사항)
/// - `label_name`: 라벨 이름 (선택사항)
/// 
/// # 예시
/// ```rust
/// let file_info = MaskFileInfo::png(
///     "slice_001.png".to_string(),
///     1024,
///     512,
///     512,
///     Some(1),
///     Some("1.2.3.4.5.6.7.8.9.10".to_string()),
///     Some("liver".to_string()),
/// );
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MaskFileInfo {
    /// 업로드할 파일의 이름
    pub file_name: String,
    /// 파일의 MIME 타입 (선택사항)
    pub mime_type: Option<String>,
    /// 파일 크기 (바이트)
    pub file_size: i64,
    /// 파일의 체크섬 (선택사항)
    pub checksum: Option<String>,
    /// 이미지 너비 (픽셀, 선택사항)
    pub width: Option<i32>,
    /// 이미지 높이 (픽셀, 선택사항)
    pub height: Option<i32>,
    /// 슬라이스 인덱스 (선택사항)
    pub slice_index: Option<i32>,
    /// DICOM SOP Instance UID (선택사항)
    pub sop_instance_uid: Option<String>,
    /// 라벨 이름 (선택사항)
    pub label_name: Option<String>,
}

impl MaskFileInfo {
    /// 새로운 마스크 파일 정보를 생성합니다.
    /// 
    /// 이 메서드는 모든 필드를 직접 지정하여 파일 정보를 생성합니다.
    /// 
    /// # 매개변수
    /// - `file_name`: 업로드할 파일의 이름
    /// - `mime_type`: 파일의 MIME 타입
    /// - `file_size`: 파일 크기 (바이트)
    /// - `checksum`: 파일의 체크섬 (선택사항)
    /// - `width`: 이미지 너비 (픽셀, 선택사항)
    /// - `height`: 이미지 높이 (픽셀, 선택사항)
    /// - `slice_index`: 슬라이스 인덱스 (선택사항)
    /// - `sop_instance_uid`: DICOM SOP Instance UID (선택사항)
    /// - `label_name`: 라벨 이름 (선택사항)
    /// 
    /// # 반환값
    /// 생성된 `MaskFileInfo` 인스턴스
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

    /// PNG 파일 정보를 생성합니다.
    /// 
    /// 이 메서드는 PNG 형식의 마스크 파일을 위한 특화된 생성자입니다.
    /// MIME 타입이 자동으로 "image/png"로 설정됩니다.
    /// 
    /// # 매개변수
    /// - `file_name`: PNG 파일의 이름
    /// - `file_size`: 파일 크기 (바이트)
    /// - `width`: 이미지 너비 (픽셀)
    /// - `height`: 이미지 높이 (픽셀)
    /// - `slice_index`: 슬라이스 인덱스 (선택사항)
    /// - `sop_instance_uid`: DICOM SOP Instance UID (선택사항)
    /// - `label_name`: 라벨 이름 (선택사항)
    /// 
    /// # 반환값
    /// PNG 파일로 설정된 `MaskFileInfo` 인스턴스
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

    /// JPEG 파일 정보를 생성합니다.
    /// 
    /// 이 메서드는 JPEG 형식의 마스크 파일을 위한 특화된 생성자입니다.
    /// MIME 타입이 자동으로 "image/jpeg"로 설정됩니다.
    /// 
    /// # 매개변수
    /// - `file_name`: JPEG 파일의 이름
    /// - `file_size`: 파일 크기 (바이트)
    /// - `width`: 이미지 너비 (픽셀)
    /// - `height`: 이미지 높이 (픽셀)
    /// - `slice_index`: 슬라이스 인덱스 (선택사항)
    /// - `sop_instance_uid`: DICOM SOP Instance UID (선택사항)
    /// - `label_name`: 라벨 이름 (선택사항)
    /// 
    /// # 반환값
    /// JPEG 파일로 설정된 `MaskFileInfo` 인스턴스
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

/// 마스크 엔티티의 단위 테스트 모듈
/// 
/// 이 모듈은 `Mask`, `NewMask`, `UpdateMask`, `MaskStats`, `MaskFileInfo` 등의
/// 모든 구조체와 메서드에 대한 단위 테스트를 포함합니다.
#[cfg(test)]
mod tests {
    use super::*;

    /// `NewMask::new()` 메서드의 정상 동작을 테스트합니다.
    /// 
    /// 이 테스트는 모든 필드를 포함한 마스크 생성이 올바르게 동작하는지 확인합니다.
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

    /// `NewMask::with_defaults()` 메서드의 정상 동작을 테스트합니다.
    /// 
    /// 이 테스트는 기본값으로 마스크가 올바르게 생성되는지 확인합니다.
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

    /// `NewMask::png()` 메서드의 정상 동작을 테스트합니다.
    /// 
    /// 이 테스트는 PNG 형식의 마스크가 올바르게 생성되는지 확인합니다.
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

    /// `NewMask::dicom()` 메서드의 정상 동작을 테스트합니다.
    /// 
    /// 이 테스트는 DICOM 형식의 마스크가 올바르게 생성되는지 확인합니다.
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

    /// `NewMask`의 체이닝 메서드들의 정상 동작을 테스트합니다.
    /// 
    /// 이 테스트는 체크섬, 파일 크기, 이미지 크기 설정이 올바르게 동작하는지 확인합니다.
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

    /// `UpdateMask`의 체이닝 메서드들의 정상 동작을 테스트합니다.
    /// 
    /// 이 테스트는 업데이트 구조체의 빌더 패턴이 올바르게 동작하는지 확인합니다.
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

    /// `MaskStats`의 통계 추가 메서드들의 정상 동작을 테스트합니다.
    /// 
    /// 이 테스트는 통계 구조체의 데이터 추가 및 집계가 올바르게 동작하는지 확인합니다.
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

    /// `MaskFileInfo::png()` 메서드의 정상 동작을 테스트합니다.
    /// 
    /// 이 테스트는 PNG 파일 정보가 올바르게 생성되는지 확인합니다.
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

    /// `Mask`에서 `NewMask`으로의 변환이 올바르게 동작하는지 테스트합니다.
    /// 
    /// 이 테스트는 From 트레이트 구현이 정확하게 동작하는지 확인합니다.
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
