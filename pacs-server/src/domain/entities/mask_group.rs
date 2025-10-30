//! # 마스크 그룹 엔티티 모듈
//! 
//! 이 모듈은 의료 영상의 마스크 그룹 정보를 나타내는 엔티티들을 정의합니다.
//! 마스크 그룹은 여러 개의 마스크 파일을 하나의 논리적 단위로 관리하는 컨테이너입니다.

// UTC 시간대의 날짜/시간 처리를 위한 chrono 라이브러리
use chrono::{DateTime, Utc};
// JSON 직렬화/역직렬화를 위한 serde 라이브러리
use serde::{Deserialize, Serialize};
// 해시맵을 위한 표준 라이브러리
use std::collections::HashMap;
// OpenAPI 스키마 생성을 위한 utoipa 라이브러리
use utoipa::ToSchema;
// SQLx를 통한 데이터베이스 행 매핑을 위한 트레이트
use sqlx::FromRow;

/// 마스크 그룹을 나타내는 엔티티
/// 
/// 이 구조체는 데이터베이스의 `annotation_mask_group` 테이블과 매핑되며,
/// 여러 개의 마스크 파일을 하나의 그룹으로 관리합니다.
/// 
/// # 필드
/// - `id`: 데이터베이스에서 자동 생성되는 고유 식별자
/// - `annotation_id`: 마스크 그룹이 속한 어노테이션의 ID
/// - `group_name`: 마스크 그룹의 이름 (선택사항)
/// - `model_name`: AI 모델명 (선택사항)
/// - `version`: 모델 버전 (선택사항)
/// - `modality`: 의료 영상 모달리티 (CT, MR, X-Ray 등)
/// - `slice_count`: 슬라이스 개수 (선택사항)
/// - `mask_type`: 마스크 타입 (segmentation, bounding_box, manual 등)
/// - `description`: 마스크 그룹에 대한 설명 (선택사항)
/// - `created_by`: 마스크 그룹을 생성한 사용자의 ID (선택사항)
/// - `created_at`: 마스크 그룹이 생성된 시각
/// - `updated_at`: 마스크 그룹이 마지막으로 수정된 시각
/// 
/// # 예시
/// ```ignore
/// let mask_group = MaskGroup {
///     id: 1,
///     annotation_id: 1,
///     group_name: Some("Liver_Segmentation_v2".to_string()),
///     model_name: Some("UNet".to_string()),
///     version: Some("1.0.0".to_string()),
///     modality: Some("CT".to_string()),
///     slice_count: Some(50),
///     mask_type: Some("segmentation".to_string()),
///     description: Some("간 분할을 위한 AI 모델 결과".to_string()),
///     created_by: Some(1),
///     created_at: Utc::now(),
///     updated_at: Utc::now(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, FromRow)]
pub struct MaskGroup {
    /// 데이터베이스에서 자동 생성되는 고유 식별자
    pub id: i32,
    /// 마스크 그룹이 속한 어노테이션의 ID
    pub annotation_id: i32,
    /// 마스크 그룹의 이름 (선택사항)
    pub group_name: Option<String>,
    /// AI 모델명 (선택사항)
    pub model_name: Option<String>,
    /// 모델 버전 (선택사항)
    pub version: Option<String>,
    /// 의료 영상 모달리티 (CT, MR, X-Ray 등)
    pub modality: Option<String>,
    /// 슬라이스 개수 (선택사항)
    pub slice_count: Option<i32>,
    /// 마스크 타입 (segmentation, bounding_box, manual 등)
    pub mask_type: Option<String>,
    /// 마스크 그룹에 대한 설명 (선택사항)
    pub description: Option<String>,
    /// 마스크 그룹을 생성한 사용자의 ID (선택사항)
    pub created_by: Option<i32>,
    /// 마스크 그룹이 생성된 시각
    pub created_at: DateTime<Utc>,
    /// 마스크 그룹이 마지막으로 수정된 시각
    pub updated_at: DateTime<Utc>,
}

/// 새로운 마스크 그룹 생성을 위한 DTO(Data Transfer Object)
/// 
/// 이 구조체는 마스크 그룹 생성 요청 시 전달되는 데이터를 나타냅니다.
/// 데이터베이스에 저장되기 전의 마스크 그룹 정보를 담고 있습니다.
/// 
/// # 필드
/// - `annotation_id`: 마스크 그룹이 속할 어노테이션의 ID
/// - `group_name`: 생성할 마스크 그룹의 이름 (선택사항)
/// - `model_name`: AI 모델명 (선택사항)
/// - `version`: 모델 버전 (선택사항)
/// - `modality`: 의료 영상 모달리티 (선택사항)
/// - `slice_count`: 슬라이스 개수 (선택사항)
/// - `mask_type`: 마스크 타입 (선택사항)
/// - `description`: 마스크 그룹에 대한 설명 (선택사항)
/// - `created_by`: 마스크 그룹을 생성할 사용자의 ID (선택사항)
/// 
/// # 예시
/// ```ignore
/// let new_mask_group = NewMaskGroup {
///     annotation_id: 1,
///     group_name: Some("Liver_Segmentation_v2".to_string()),
///     model_name: Some("UNet".to_string()),
///     version: Some("1.0.0".to_string()),
///     modality: Some("CT".to_string()),
///     slice_count: Some(50),
///     mask_type: Some("segmentation".to_string()),
///     description: Some("간 분할을 위한 AI 모델 결과".to_string()),
///     created_by: Some(1),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NewMaskGroup {
    /// 마스크 그룹이 속할 어노테이션의 ID
    pub annotation_id: i32,
    /// 생성할 마스크 그룹의 이름 (선택사항)
    pub group_name: Option<String>,
    /// AI 모델명 (선택사항)
    pub model_name: Option<String>,
    /// 모델 버전 (선택사항)
    pub version: Option<String>,
    /// 의료 영상 모달리티 (선택사항)
    pub modality: Option<String>,
    /// 슬라이스 개수 (선택사항)
    pub slice_count: Option<i32>,
    /// 마스크 타입 (선택사항)
    pub mask_type: Option<String>,
    /// 마스크 그룹에 대한 설명 (선택사항)
    pub description: Option<String>,
    /// 마스크 그룹을 생성할 사용자의 ID (선택사항)
    pub created_by: Option<i32>,
}

impl NewMaskGroup {
    /// 새로운 마스크 그룹을 생성합니다.
    /// 
    /// # 매개변수
    /// - `annotation_id`: 마스크 그룹이 속할 어노테이션의 ID
    /// - `group_name`: 마스크 그룹의 이름 (선택사항)
    /// - `model_name`: AI 모델명 (선택사항)
    /// - `version`: 모델 버전 (선택사항)
    /// - `modality`: 의료 영상 모달리티 (선택사항)
    /// - `slice_count`: 슬라이스 개수 (필수)
    /// - `mask_type`: 마스크 타입 (필수)
    /// - `description`: 마스크 그룹에 대한 설명 (선택사항)
    /// - `created_by`: 마스크 그룹을 생성할 사용자의 ID (선택사항)
    /// 
    /// # 반환값
    /// 생성된 `NewMaskGroup` 인스턴스
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

    /// 기본값으로 마스크 그룹을 생성합니다.
    /// 
    /// 이 메서드는 가장 기본적인 설정으로 마스크 그룹을 생성합니다:
    /// - 슬라이스 개수: 1
    /// - 마스크 타입: "segmentation"
    /// - 기타 필드들은 None으로 설정
    /// 
    /// # 매개변수
    /// - `annotation_id`: 마스크 그룹이 속할 어노테이션의 ID
    /// - `created_by`: 마스크 그룹을 생성할 사용자의 ID (선택사항)
    /// 
    /// # 반환값
    /// 기본값으로 설정된 `NewMaskGroup` 인스턴스
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

    /// AI 모델 정보가 포함된 마스크 그룹을 생성합니다.
    /// 
    /// 이 메서드는 AI 모델에 의해 생성된 마스크 그룹을 위한 특화된 생성자입니다.
    /// 
    /// # 매개변수
    /// - `annotation_id`: 마스크 그룹이 속할 어노테이션의 ID
    /// - `group_name`: 마스크 그룹의 이름
    /// - `model_name`: AI 모델명
    /// - `version`: 모델 버전
    /// - `modality`: 의료 영상 모달리티
    /// - `slice_count`: 슬라이스 개수
    /// - `created_by`: 마스크 그룹을 생성할 사용자의 ID (선택사항)
    /// 
    /// # 반환값
    /// AI 모델 정보가 설정된 `NewMaskGroup` 인스턴스
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

    /// 수동으로 생성된 마스크 그룹을 생성합니다.
    /// 
    /// 이 메서드는 의료진이 수동으로 생성한 마스크 그룹을 위한 특화된 생성자입니다.
    /// 
    /// # 매개변수
    /// - `annotation_id`: 마스크 그룹이 속할 어노테이션의 ID
    /// - `group_name`: 마스크 그룹의 이름
    /// - `modality`: 의료 영상 모달리티
    /// - `slice_count`: 슬라이스 개수
    /// - `description`: 마스크 그룹에 대한 설명 (선택사항)
    /// - `created_by`: 마스크 그룹을 생성할 사용자의 ID (선택사항)
    /// 
    /// # 반환값
    /// 수동 생성 마스크 그룹으로 설정된 `NewMaskGroup` 인스턴스
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

/// `MaskGroup`에서 `NewMaskGroup`으로의 변환을 위한 From 트레이트 구현
/// 
/// 이 구현은 기존의 `MaskGroup` 엔티티를 `NewMaskGroup` DTO로 변환할 때 사용됩니다.
/// 주로 업데이트 작업에서 기존 데이터를 기반으로 새로운 데이터를 생성할 때 활용됩니다.
impl From<MaskGroup> for NewMaskGroup {
    /// `MaskGroup`을 `NewMaskGroup`으로 변환합니다.
    /// 
    /// # 매개변수
    /// - `mask_group`: 변환할 `MaskGroup` 인스턴스
    /// 
    /// # 반환값
    /// 변환된 `NewMaskGroup` 인스턴스
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

/// 마스크 그룹 업데이트를 위한 DTO(Data Transfer Object)
/// 
/// 이 구조체는 마스크 그룹 업데이트 요청 시 전달되는 데이터를 나타냅니다.
/// 업데이트할 필드만 포함하며, None인 필드는 업데이트하지 않습니다.
/// 
/// # 필드
/// - `id`: 업데이트할 마스크 그룹의 ID
/// - `group_name`: 새로운 그룹 이름 (선택사항)
/// - `model_name`: 새로운 모델명 (선택사항)
/// - `version`: 새로운 모델 버전 (선택사항)
/// - `modality`: 새로운 모달리티 (선택사항)
/// - `slice_count`: 새로운 슬라이스 개수 (선택사항)
/// - `mask_type`: 새로운 마스크 타입 (선택사항)
/// - `description`: 새로운 설명 (선택사항)
/// 
/// # 예시
/// ```ignore
/// let update = UpdateMaskGroup::new(1)
///     .with_group_name("Updated_Group".to_string())
///     .with_model_info("NewModel".to_string(), "2.0.0".to_string())
///     .with_modality("MR".to_string());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UpdateMaskGroup {
    /// 업데이트할 마스크 그룹의 ID
    pub id: i32,
    /// 새로운 그룹 이름 (선택사항)
    pub group_name: Option<String>,
    /// 새로운 모델명 (선택사항)
    pub model_name: Option<String>,
    /// 새로운 모델 버전 (선택사항)
    pub version: Option<String>,
    /// 새로운 모달리티 (선택사항)
    pub modality: Option<String>,
    /// 새로운 슬라이스 개수 (선택사항)
    pub slice_count: Option<i32>,
    /// 새로운 마스크 타입 (선택사항)
    pub mask_type: Option<String>,
    /// 새로운 설명 (선택사항)
    pub description: Option<String>,
}

impl UpdateMaskGroup {
    /// 빈 업데이트 구조체를 생성합니다.
    /// 
    /// 이 메서드는 지정된 ID를 가진 빈 업데이트 구조체를 생성합니다.
    /// 모든 필드는 None으로 설정되어 있어, 필요한 필드만 체이닝 메서드로 설정할 수 있습니다.
    /// 
    /// # 매개변수
    /// - `id`: 업데이트할 마스크 그룹의 ID
    /// 
    /// # 반환값
    /// 빈 필드로 초기화된 `UpdateMaskGroup` 인스턴스
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

    /// 그룹 이름을 설정합니다.
    /// 
    /// # 매개변수
    /// - `group_name`: 새로운 그룹 이름
    /// 
    /// # 반환값
    /// 그룹 이름이 설정된 `UpdateMaskGroup` 인스턴스
    pub fn with_group_name(mut self, group_name: String) -> Self {
        self.group_name = Some(group_name);
        self
    }

    /// 모델 정보를 설정합니다.
    /// 
    /// # 매개변수
    /// - `model_name`: 새로운 모델명
    /// - `version`: 새로운 모델 버전
    /// 
    /// # 반환값
    /// 모델 정보가 설정된 `UpdateMaskGroup` 인스턴스
    pub fn with_model_info(mut self, model_name: String, version: String) -> Self {
        self.model_name = Some(model_name);
        self.version = Some(version);
        self
    }

    /// 모달리티를 설정합니다.
    /// 
    /// # 매개변수
    /// - `modality`: 새로운 모달리티 (CT, MR, X-Ray 등)
    /// 
    /// # 반환값
    /// 모달리티가 설정된 `UpdateMaskGroup` 인스턴스
    pub fn with_modality(mut self, modality: String) -> Self {
        self.modality = Some(modality);
        self
    }

    /// 슬라이스 개수를 설정합니다.
    /// 
    /// # 매개변수
    /// - `slice_count`: 새로운 슬라이스 개수
    /// 
    /// # 반환값
    /// 슬라이스 개수가 설정된 `UpdateMaskGroup` 인스턴스
    pub fn with_slice_count(mut self, slice_count: i32) -> Self {
        self.slice_count = Some(slice_count);
        self
    }

    /// 마스크 타입을 설정합니다.
    /// 
    /// # 매개변수
    /// - `mask_type`: 새로운 마스크 타입 (segmentation, bounding_box, manual 등)
    /// 
    /// # 반환값
    /// 마스크 타입이 설정된 `UpdateMaskGroup` 인스턴스
    pub fn with_mask_type(mut self, mask_type: String) -> Self {
        self.mask_type = Some(mask_type);
        self
    }

    /// 설명을 설정합니다.
    /// 
    /// # 매개변수
    /// - `description`: 새로운 설명
    /// 
    /// # 반환값
    /// 설명이 설정된 `UpdateMaskGroup` 인스턴스
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}

/// `UpdateMaskGroup`의 기본값을 제공하는 Default 트레이트 구현
/// 
/// 이 구현은 ID가 0인 빈 업데이트 구조체를 반환합니다.
/// 실제 사용 시에는 올바른 ID로 설정해야 합니다.
impl Default for UpdateMaskGroup {
    /// 기본값으로 `UpdateMaskGroup`을 생성합니다.
    /// 
    /// # 반환값
    /// ID가 0으로 설정된 빈 `UpdateMaskGroup` 인스턴스
    /// 
    /// # 주의사항
    /// 실제 사용 시에는 올바른 ID로 설정해야 합니다.
    fn default() -> Self {
        Self::new(0) // Default ID, should be set properly when used
    }
}

/// 마스크 그룹 통계 정보를 나타내는 구조체
/// 
/// 이 구조체는 마스크 그룹과 관련된 다양한 통계 정보를 저장합니다.
/// API 응답에서 사용되며, OpenAPI 스키마 생성에도 활용됩니다.
/// 
/// # 필드
/// - `total_groups`: 전체 마스크 그룹 수
/// - `total_masks`: 전체 마스크 파일 수
/// - `total_size_bytes`: 전체 마스크 파일 크기 (바이트)
/// - `modalities`: 모달리티별 마스크 그룹 수 (해시맵)
/// - `mask_types`: 마스크 타입별 마스크 그룹 수 (해시맵)
/// 
/// # 예시
/// ```ignore
/// let mut stats = MaskGroupStats::new();
/// stats.total_groups = 5;
/// stats.total_masks = 150;
/// stats.total_size_bytes = 1024000;
/// stats.add_modality_count("CT".to_string(), 3);
/// stats.add_mask_type_count("segmentation".to_string(), 4);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct MaskGroupStats {
    /// 전체 마스크 그룹 수
    pub total_groups: i64,
    /// 전체 마스크 파일 수
    pub total_masks: i64,
    /// 전체 마스크 파일 크기 (바이트)
    pub total_size_bytes: i64,
    /// 모달리티별 마스크 그룹 수 (해시맵)
    pub modalities: HashMap<String, i64>,
    /// 마스크 타입별 마스크 그룹 수 (해시맵)
    pub mask_types: HashMap<String, i64>,
}

impl MaskGroupStats {
    /// 빈 통계 정보를 생성합니다.
    /// 
    /// 이 메서드는 모든 필드가 0 또는 빈 해시맵으로 초기화된 통계 구조체를 생성합니다.
    /// 
    /// # 반환값
    /// 빈 필드로 초기화된 `MaskGroupStats` 인스턴스
    pub fn new() -> Self {
        Self {
            total_groups: 0,
            total_masks: 0,
            total_size_bytes: 0,
            modalities: HashMap::new(),
            mask_types: HashMap::new(),
        }
    }

    /// 모달리티별 통계를 추가합니다.
    /// 
    /// 이 메서드는 특정 모달리티의 마스크 그룹 수를 증가시킵니다.
    /// 해당 모달리티가 이미 존재하면 기존 값에 추가하고, 없으면 새로 생성합니다.
    /// 
    /// # 매개변수
    /// - `modality`: 모달리티 이름 (예: "CT", "MR", "X-Ray")
    /// - `count`: 추가할 마스크 그룹 수
    /// 
    /// # 예시
    /// ```ignore
    /// let mut stats = MaskGroupStats::new();
    /// stats.add_modality_count("CT".to_string(), 5);
    /// stats.add_modality_count("CT".to_string(), 3); // CT는 총 8개가 됨
    /// ```
    pub fn add_modality_count(&mut self, modality: String, count: i64) {
        *self.modalities.entry(modality).or_insert(0) += count;
    }

    /// 마스크 타입별 통계를 추가합니다.
    /// 
    /// 이 메서드는 특정 마스크 타입의 마스크 그룹 수를 증가시킵니다.
    /// 해당 마스크 타입이 이미 존재하면 기존 값에 추가하고, 없으면 새로 생성합니다.
    /// 
    /// # 매개변수
    /// - `mask_type`: 마스크 타입 (예: "segmentation", "bounding_box", "manual")
    /// - `count`: 추가할 마스크 그룹 수
    /// 
    /// # 예시
    /// ```ignore
    /// let mut stats = MaskGroupStats::new();
    /// stats.add_mask_type_count("segmentation".to_string(), 10);
    /// stats.add_mask_type_count("manual".to_string(), 2);
    /// ```
    pub fn add_mask_type_count(&mut self, mask_type: String, count: i64) {
        *self.mask_types.entry(mask_type).or_insert(0) += count;
    }
}

/// `MaskGroupStats`의 기본값을 제공하는 Default 트레이트 구현
/// 
/// 이 구현은 `new()` 메서드를 호출하여 빈 통계 구조체를 반환합니다.
impl Default for MaskGroupStats {
    /// 기본값으로 `MaskGroupStats`를 생성합니다.
    /// 
    /// # 반환값
    /// 빈 필드로 초기화된 `MaskGroupStats` 인스턴스
    fn default() -> Self {
        Self::new()
    }
}

/// 마스크 그룹 엔티티의 단위 테스트 모듈
/// 
/// 이 모듈은 `MaskGroup`, `NewMaskGroup`, `UpdateMaskGroup`, `MaskGroupStats` 등의
/// 모든 구조체와 메서드에 대한 단위 테스트를 포함합니다.
#[cfg(test)]
mod tests {
    use super::*;

    /// `NewMaskGroup::new()` 메서드의 정상 동작을 테스트합니다.
    /// 
    /// 이 테스트는 모든 필드를 포함한 마스크 그룹 생성이 올바르게 동작하는지 확인합니다.
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

    /// `NewMaskGroup::with_defaults()` 메서드의 정상 동작을 테스트합니다.
    /// 
    /// 이 테스트는 기본값으로 마스크 그룹이 올바르게 생성되는지 확인합니다.
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

    /// `NewMaskGroup::with_ai_model()` 메서드의 정상 동작을 테스트합니다.
    /// 
    /// 이 테스트는 AI 모델 정보가 포함된 마스크 그룹이 올바르게 생성되는지 확인합니다.
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

    /// `NewMaskGroup::manual()` 메서드의 정상 동작을 테스트합니다.
    /// 
    /// 이 테스트는 수동으로 생성된 마스크 그룹이 올바르게 생성되는지 확인합니다.
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

    /// `UpdateMaskGroup`의 체이닝 메서드들의 정상 동작을 테스트합니다.
    /// 
    /// 이 테스트는 업데이트 구조체의 빌더 패턴이 올바르게 동작하는지 확인합니다.
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

    /// `MaskGroupStats`의 통계 추가 메서드들의 정상 동작을 테스트합니다.
    /// 
    /// 이 테스트는 통계 구조체의 데이터 추가 및 집계가 올바르게 동작하는지 확인합니다.
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

    /// `MaskGroup`에서 `NewMaskGroup`으로의 변환이 올바르게 동작하는지 테스트합니다.
    /// 
    /// 이 테스트는 From 트레이트 구현이 정확하게 동작하는지 확인합니다.
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
