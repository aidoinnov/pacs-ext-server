use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::domain::entities::mask_group::MaskGroupStats;

/// 마스크 그룹 생성 요청 DTO
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CreateMaskGroupRequest {
    /// 그룹 이름
    /// 마스크 그룹을 식별하는 이름 (예: Liver_Segmentation_v2)
    #[schema(example = "Liver_Segmentation_v2")]
    pub group_name: Option<String>,
    
    /// AI 모델 이름
    /// 마스크를 생성한 AI 모델의 이름
    #[schema(example = "monai_unet")]
    pub model_name: Option<String>,
    
    /// 버전 정보
    /// 모델 또는 마스크의 버전 정보
    #[schema(example = "v2.1.0")]
    pub version: Option<String>,
    
    /// 의료 영상 모달리티
    /// CT, MR, US 등 의료 영상의 모달리티
    #[schema(example = "CT")]
    pub modality: Option<String>,
    
    /// 예상 슬라이스 수
    /// 업로드할 마스크 슬라이스의 예상 개수
    #[schema(example = 120)]
    pub slice_count: i32,
    
    /// 마스크 타입
    /// segmentation, detection, classification 등
    #[schema(example = "segmentation")]
    pub mask_type: String,
    
    /// 설명
    /// 마스크 그룹에 대한 추가 설명이나 메모
    #[schema(example = "간 세그멘테이션 결과")]
    pub description: Option<String>,
}

/// 마스크 그룹 응답 DTO
#[derive(Debug, Serialize, ToSchema)]
pub struct MaskGroupResponse {
    /// 마스크 그룹 ID
    pub id: i32,
    
    /// 어노테이션 ID
    pub annotation_id: i32,
    
    /// 그룹 이름
    pub group_name: Option<String>,
    
    /// AI 모델 이름
    pub model_name: Option<String>,
    
    /// 버전 정보
    pub version: Option<String>,
    
    /// 의료 영상 모달리티
    pub modality: Option<String>,
    
    /// 슬라이스 수
    pub slice_count: i32,
    
    /// 마스크 타입
    pub mask_type: String,
    
    /// 설명
    pub description: Option<String>,
    
    /// 생성자 ID
    pub created_by: Option<i32>,
    
    /// 생성 시간
    pub created_at: String,
    
    /// 수정 시간
    pub updated_at: String,
}

/// 마스크 그룹 업데이트 요청 DTO
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct UpdateMaskGroupRequest {
    /// 그룹 이름
    #[schema(example = "Updated_Liver_Segmentation")]
    pub group_name: Option<String>,
    
    /// AI 모델 이름
    #[schema(example = "monai_unet_v2")]
    pub model_name: Option<String>,
    
    /// 버전 정보
    #[schema(example = "v2.2.0")]
    pub version: Option<String>,
    
    /// 의료 영상 모달리티
    #[schema(example = "MR")]
    pub modality: Option<String>,
    
    /// 슬라이스 수
    #[schema(example = 150)]
    pub slice_count: Option<i32>,
    
    /// 마스크 타입
    #[schema(example = "detection")]
    pub mask_type: Option<String>,
    
    /// 설명
    #[schema(example = "업데이트된 간 세그멘테이션 결과")]
    pub description: Option<String>,
}

/// Signed URL 요청 DTO
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct SignedUrlRequest {
    /// 마스크 그룹 ID
    /// 업로드할 마스크가 속할 그룹의 ID
    #[schema(example = 1)]
    pub mask_group_id: i32,
    
    /// 파일명
    /// 업로드할 파일의 이름 (확장자 포함)
    #[schema(example = "0001_liver.png")]
    pub filename: String,
    
    /// MIME 타입
    /// 파일의 MIME 타입
    #[schema(example = "image/png")]
    pub mime_type: String,
    
    /// 파일 크기 (바이트)
    /// 업로드할 파일의 크기
    #[schema(example = 1024000)]
    pub file_size: Option<i64>,
    
    /// 슬라이스 인덱스
    /// 볼륨 내 슬라이스의 인덱스
    #[schema(example = 1)]
    pub slice_index: Option<i32>,
    
    /// SOP Instance UID
    /// DICOM 표준의 SOP Instance UID
    #[schema(example = "1.2.840.113619.2.55.3.604688119.868.1234567890.3")]
    pub sop_instance_uid: Option<String>,
    
    /// 라벨 이름
    /// 마스크의 라벨 이름 (예: liver, spleen)
    #[schema(example = "liver")]
    pub label_name: Option<String>,
    
    /// TTL (Time To Live) 초
    /// Signed URL의 유효 시간 (초 단위)
    #[schema(example = 3600)]
    pub ttl_seconds: Option<u64>,
}

/// Signed URL 응답 DTO
#[derive(Debug, Serialize, ToSchema)]
pub struct SignedUrlResponse {
    /// 업로드용 Signed URL
    /// 파일을 업로드할 수 있는 서명된 URL
    #[schema(example = "https://s3.example.com/mask/123/17/0001_liver.png?X-Amz-Algorithm=...")]
    pub upload_url: String,
    
    /// 다운로드용 Signed URL
    /// 파일을 다운로드할 수 있는 서명된 URL
    #[schema(example = "https://s3.example.com/mask/123/17/0001_liver.png?X-Amz-Algorithm=...")]
    pub download_url: String,
    
    /// S3 파일 경로
    /// 업로드될 파일의 S3 경로
    #[schema(example = "mask/123/17/0001_liver.png")]
    pub file_path: String,
    
    /// 만료 시간 (초)
    /// Signed URL의 만료 시간
    #[schema(example = 600)]
    pub expires_in: u64,
    
    /// 만료 시간 (ISO 8601)
    /// Signed URL의 만료 시간
    #[schema(example = "2024-01-01T12:00:00Z")]
    pub expires_at: String,
}

/// 업로드 완료 요청 DTO
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CompleteUploadRequest {
    /// 마스크 그룹 ID
    /// 업로드가 완료된 마스크 그룹의 ID
    #[schema(example = 1)]
    pub mask_group_id: i32,
    
    /// 실제 업로드된 슬라이스 수
    /// 업로드가 완료된 슬라이스의 실제 개수
    #[schema(example = 120)]
    pub slice_count: i32,
    
    /// 라벨 목록
    /// 업로드된 마스크에 포함된 라벨들의 목록
    #[schema(example = json!(["liver", "spleen"]))]
    pub labels: Vec<String>,
    
    /// 업로드된 파일 목록
    /// 업로드가 완료된 파일들의 목록
    #[schema(example = json!(["0001_liver.png", "0002_liver.png"]))]
    pub uploaded_files: Vec<String>,
}

/// 업로드 완료 응답 DTO
#[derive(Debug, Serialize, ToSchema)]
pub struct CompleteUploadResponse {
    /// 성공 여부
    #[schema(example = true)]
    pub success: bool,
    
    /// 처리 상태
    #[schema(example = "success")]
    pub status: String,
    
    /// 처리된 마스크 수
    #[schema(example = 120)]
    pub processed_masks: i32,
    
    /// 업로드된 파일 목록
    /// 업로드가 완료된 파일들의 목록
    #[schema(example = json!(["0001_liver.png", "0002_liver.png"]))]
    pub uploaded_files: Vec<String>,
    
    /// 메시지
    #[schema(example = "Upload completed successfully")]
    pub message: String,
}

/// 마스크 그룹 목록 응답 DTO
#[derive(Debug, Serialize, ToSchema)]
pub struct MaskGroupListResponse {
    /// 마스크 그룹 목록
    pub mask_groups: Vec<MaskGroupResponse>,
    
    /// 전체 개수
    pub total_count: i64,
    
    /// 오프셋
    pub offset: i64,
    
    /// 제한 개수
    pub limit: i64,
}

/// 마스크 그룹 상세 응답 DTO
#[derive(Debug, Serialize, ToSchema)]
pub struct MaskGroupDetailResponse {
    /// 마스크 그룹 ID
    pub id: i32,
    
    /// 어노테이션 ID
    pub annotation_id: i32,
    
    /// 그룹 이름
    pub group_name: Option<String>,
    
    /// AI 모델 이름
    pub model_name: Option<String>,
    
    /// 버전 정보
    pub version: Option<String>,
    
    /// 의료 영상 모달리티
    pub modality: Option<String>,
    
    /// 슬라이스 수
    pub slice_count: Option<i32>,
    
    /// 마스크 타입
    pub mask_type: Option<String>,
    
    /// 설명
    pub description: Option<String>,
    
    /// 생성자 ID
    pub created_by: Option<i32>,
    
    /// 생성 시간
    pub created_at: String,
    
    /// 수정 시간
    pub updated_at: String,
    
    /// 통계 정보
    pub stats: MaskGroupStats,
}
