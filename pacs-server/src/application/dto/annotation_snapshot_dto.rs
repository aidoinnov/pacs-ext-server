use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct SnapshotUploadUrlRequest{
    /// 파일명    
    #[schema(example="snapshot_20260111_12000.png")]
    pub filename: String,
    /// MIME 타입 (image/png, image/jpeg, image/webp)
    #[schema(example="image/png")]
    pub mime_type: String,
    /// 파일크기(바이트)
    #[schema(example=524288)]
    pub file_size: Option<i64>,
    /// TTL (초, 기본값: 600)
    pub ttl_seconds: Option<u64>,
}



#[derive(Debug, Serialize, ToSchema)]
pub struct SnapshotUploadUrlResponse{
    /// 업로드용 Presigned URL
    #[schema(example = "https://s3.amazonaws.com/bucket/snapshots/1/123/20260111_120000_image.png?X-Amz-...")]
    pub upload_url: String,

    /// 다운로드용 Presigned URL
    #[schema(example = "https://s3.amazonaws.com/bucket/snapshots/1/123/20260111_120000_image.png?X-Amz-...")]
    pub download_url: String,

    /// S3 object key (DB에 저장할 값)
    #[schema(example = "snapshots/1/123/20260111_120000_snapshot.png")]
    pub image_key: String,
    /// 만료 시간 (초))
    #[schema(example= 600)]
    pub expires_in: u64,
    /// 만료 시간 (ISO 8601)
    #[schema(example = "2026-01-11T13:00:00Z")]
    pub expires_at: String,
}


/// 스냅샷 업로드 완료 요청
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CompleteSnapshotUploadRequest {
    /// S3 object key (업로드 URL 생성 시 받은 값)
    #[schema(example = "annotations/123/snapshots/snapshot_20260111_120000.png")]
    pub image_key: String,


    /// 업로드 성공 여부(optional, 기본값: true)
    /// false인 경우, 실패로 처리하고, uploaded_at은 NULL로 유지
    #[schema(example = true)]
    pub success: Option<bool>,
}

// ⚠️ 주의: uploaded_at은 사용자가 보내는 게 아니라 서버에서 자동 생성!

/// 스냅샷 업로드 상태 응답
#[derive(Debug, Serialize, ToSchema)]
pub struct SnapshotStatusResponse {
    /// 어노테이션 ID
    pub annotation_id: i32,

    // S3 object key
    pub image_key: Option<String>,

    /// 업로드 상태
    #[schema(example="completed")]
    pub status: String,

    // 업로드 완료 시간
    pub uploaded_at: Option<String>
}


