use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// 마스크 응답 DTO
#[derive(Debug, Serialize, ToSchema)]
pub struct MaskResponse {
    /// 마스크 ID
    pub id: i32,
    
    /// 마스크 그룹 ID
    pub mask_group_id: i32,
    
    /// 슬라이스 인덱스
    /// 볼륨 내 슬라이스의 인덱스
    pub slice_index: Option<i32>,
    
    /// SOP Instance UID
    /// DICOM 표준의 SOP Instance UID
    pub sop_instance_uid: Option<String>,
    
    /// 라벨 이름
    /// 마스크의 라벨 이름 (예: liver, spleen)
    pub label_name: Option<String>,
    
    /// 파일 경로
    /// S3/스토리지에 저장된 파일의 경로
    pub file_path: String,
    
    /// MIME 타입
    /// 파일의 MIME 타입
    pub mime_type: String,
    
    /// 파일 크기 (바이트)
    /// 파일의 크기
    pub file_size: Option<i64>,
    
    /// 체크섬
    /// 파일 무결성 검증을 위한 체크섬
    pub checksum: Option<String>,
    
    /// 이미지 너비
    /// 마스크 이미지의 너비 (픽셀)
    pub width: Option<i32>,
    
    /// 이미지 높이
    /// 마스크 이미지의 높이 (픽셀)
    pub height: Option<i32>,
    
    /// 생성 시간
    pub created_at: String,
    
    /// 수정 시간
    pub updated_at: String,
}

/// 마스크 생성 요청 DTO
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CreateMaskRequest {
    /// 마스크 그룹 ID
    /// 마스크가 속할 그룹의 ID
    #[schema(example = 1)]
    pub mask_group_id: i32,
    
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
    
    /// 파일 경로
    /// S3/스토리지에 저장될 파일의 경로
    #[schema(example = "mask/123/17/0001_liver.png")]
    pub file_path: String,
    
    /// MIME 타입
    /// 파일의 MIME 타입
    #[schema(example = "image/png")]
    pub mime_type: String,
    
    /// 파일 크기 (바이트)
    /// 파일의 크기
    #[schema(example = 1024000)]
    pub file_size: Option<i64>,
    
    /// 체크섬
    /// 파일 무결성 검증을 위한 체크섬
    #[schema(example = "d41d8cd98f00b204e9800998ecf8427e")]
    pub checksum: Option<String>,
    
    /// 이미지 너비
    /// 마스크 이미지의 너비 (픽셀)
    #[schema(example = 512)]
    pub width: Option<i32>,
    
    /// 이미지 높이
    /// 마스크 이미지의 높이 (픽셀)
    #[schema(example = 512)]
    pub height: Option<i32>,
}

/// 마스크 업데이트 요청 DTO
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct UpdateMaskRequest {
    /// 슬라이스 인덱스
    #[schema(example = 2)]
    pub slice_index: Option<i32>,
    
    /// SOP Instance UID
    #[schema(example = "1.2.840.113619.2.55.3.604688119.868.1234567890.4")]
    pub sop_instance_uid: Option<String>,
    
    /// 라벨 이름
    #[schema(example = "spleen")]
    pub label_name: Option<String>,
    
    /// 파일 경로
    #[schema(example = "mask/123/17/0002_spleen.png")]
    pub file_path: Option<String>,
    
    /// MIME 타입
    #[schema(example = "image/jpeg")]
    pub mime_type: Option<String>,
    
    /// 파일 크기 (바이트)
    #[schema(example = 2048000)]
    pub file_size: Option<i64>,
    
    /// 체크섬
    #[schema(example = "e3b0c44298fc1c149afbf4c8996fb924")]
    pub checksum: Option<String>,
    
    /// 이미지 너비
    #[schema(example = 1024)]
    pub width: Option<i32>,
    
    /// 이미지 높이
    #[schema(example = 1024)]
    pub height: Option<i32>,
}

/// 마스크 목록 조회 요청 DTO
#[derive(Debug, Deserialize, ToSchema)]
pub struct ListMasksRequest {
    /// 페이지 번호 (1부터 시작)
    #[schema(example = 1)]
    pub page: Option<i32>,
    
    /// 페이지당 항목 수
    #[schema(example = 20)]
    pub page_size: Option<i32>,
    
    /// 라벨 이름으로 필터링
    #[schema(example = "liver")]
    pub label_name: Option<String>,
    
    /// 슬라이스 인덱스로 필터링
    #[schema(example = 1)]
    pub slice_index: Option<i32>,
    
    /// SOP Instance UID로 필터링
    #[schema(example = "1.2.840.113619.2.55.3.604688119.868.1234567890.3")]
    pub sop_instance_uid: Option<String>,
}

/// 마스크 목록 응답 DTO
#[derive(Debug, Serialize, ToSchema)]
pub struct MaskListResponse {
    /// 마스크 목록
    pub masks: Vec<MaskResponse>,
    
    /// 전체 개수
    pub total_count: i64,
    
    /// 오프셋
    pub offset: i64,
    
    /// 제한 개수
    pub limit: i64,
    
    /// 현재 페이지
    pub current_page: i32,
    
    /// 페이지당 항목 수
    pub page_size: i32,
    
    /// 전체 페이지 수
    pub total_pages: i32,
}

/// 마스크 다운로드 URL 요청 DTO
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct DownloadUrlRequest {
    /// 마스크 ID
    /// 다운로드할 마스크의 ID
    #[schema(example = 1)]
    pub mask_id: i32,
    
    /// 파일 경로
    /// 다운로드할 파일의 경로
    #[schema(example = "mask/123/17/0001_liver.png")]
    pub file_path: String,
    
    /// 만료 시간 (초)
    /// 다운로드 URL의 만료 시간 (기본값: 3600초)
    #[schema(example = 3600)]
    pub expires_in: Option<u64>,
}

/// 마스크 다운로드 URL 응답 DTO
#[derive(Debug, Serialize, ToSchema)]
pub struct DownloadUrlResponse {
    /// 다운로드용 Signed URL
    /// 파일을 다운로드할 수 있는 서명된 URL
    #[schema(example = "https://s3.example.com/mask/123/17/0001_liver.png?X-Amz-Algorithm=...")]
    pub download_url: String,
    
    /// 파일 경로
    /// 다운로드할 파일의 경로
    #[schema(example = "mask/123/17/0001_liver.png")]
    pub file_path: String,
    
    /// 만료 시간 (초)
    /// 다운로드 URL의 만료 시간
    #[schema(example = 3600)]
    pub expires_in: u64,
    
    /// 만료 시간 (ISO 8601)
    /// 다운로드 URL의 만료 시간
    #[schema(example = "2024-01-01T12:00:00Z")]
    pub expires_at: String,
}

/// 마스크 통계 응답 DTO
#[derive(Debug, Serialize, ToSchema)]
pub struct MaskStatsResponse {
    /// 전체 마스크 수
    pub total_masks: i64,
    
    /// 총 파일 크기 (바이트)
    pub total_size_bytes: i64,
    
    /// 평균 파일 크기 (바이트)
    pub average_file_size: f64,
    
    /// 라벨별 마스크 수
    pub masks_by_label: std::collections::HashMap<String, i64>,
    
    /// MIME 타입별 분포
    pub mime_type_distribution: std::collections::HashMap<String, i64>,
}
