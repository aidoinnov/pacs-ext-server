use async_trait::async_trait;
use crate::domain::entities::{Mask, NewMask, UpdateMask, MaskStats};
use crate::domain::services::ServiceError;

/// 마스크 Repository trait
#[async_trait]
pub trait MaskRepository: Send + Sync {
    /// 마스크 생성
    async fn create(&self, mask: &NewMask) -> Result<Mask, ServiceError>;
    
    /// ID로 마스크 조회
    async fn find_by_id(&self, id: i32) -> Result<Option<Mask>, ServiceError>;
    
    /// 마스크 그룹 ID로 마스크 목록 조회
    async fn find_by_mask_group_id(&self, mask_group_id: i32) -> Result<Vec<Mask>, ServiceError>;
    
    /// SOP Instance UID로 마스크 조회
    async fn find_by_sop_instance_uid(&self, sop_instance_uid: &str) -> Result<Vec<Mask>, ServiceError>;
    
    /// 라벨 이름으로 마스크 검색
    async fn find_by_label_name(&self, label_name: &str) -> Result<Vec<Mask>, ServiceError>;
    
    /// MIME 타입으로 마스크 필터링
    async fn find_by_mime_type(&self, mime_type: &str) -> Result<Vec<Mask>, ServiceError>;
    
    /// 슬라이스 인덱스로 마스크 조회
    async fn find_by_slice_index(&self, mask_group_id: i32, slice_index: i32) -> Result<Vec<Mask>, ServiceError>;
    
    /// 모든 마스크 조회 (페이징 지원)
    async fn find_all(
        &self,
        offset: Option<i64>,
        limit: Option<i64>,
    ) -> Result<Vec<Mask>, ServiceError>;
    
    /// 마스크 업데이트
    async fn update(&self, id: i32, update: &UpdateMask) -> Result<Mask, ServiceError>;
    
    /// 마스크 삭제
    async fn delete(&self, id: i32) -> Result<(), ServiceError>;
    
    /// 마스크 그룹의 모든 마스크 삭제
    async fn delete_by_mask_group_id(&self, mask_group_id: i32) -> Result<(), ServiceError>;
    
    /// 마스크 존재 여부 확인
    async fn exists(&self, id: i32) -> Result<bool, ServiceError>;
    
    /// 마스크 개수 조회
    async fn count(&self) -> Result<i64, ServiceError>;
    
    /// 마스크 그룹의 마스크 개수 조회
    async fn count_by_mask_group_id(&self, mask_group_id: i32) -> Result<i64, ServiceError>;
    
    /// 라벨별 마스크 개수 조회
    async fn count_by_label_name(&self, label_name: &str) -> Result<i64, ServiceError>;
    
    /// MIME 타입별 마스크 개수 조회
    async fn count_by_mime_type(&self, mime_type: &str) -> Result<i64, ServiceError>;
    
    /// 마스크 통계 조회
    async fn get_stats(&self) -> Result<MaskStats, ServiceError>;
    
    /// 마스크 그룹별 마스크 통계 조회
    async fn get_stats_by_mask_group_id(&self, mask_group_id: i32) -> Result<MaskStats, ServiceError>;
    
    /// 라벨별 마스크 통계 조회
    async fn get_stats_by_label_name(&self, label_name: &str) -> Result<MaskStats, ServiceError>;
    
    /// MIME 타입별 마스크 통계 조회
    async fn get_stats_by_mime_type(&self, mime_type: &str) -> Result<MaskStats, ServiceError>;
    
    /// 검색 조건으로 마스크 검색
    async fn search(
        &self,
        query: &str,
        mask_group_id: Option<i32>,
        label_name: Option<&str>,
        mime_type: Option<&str>,
        offset: Option<i64>,
        limit: Option<i64>,
    ) -> Result<Vec<Mask>, ServiceError>;
    
    /// 최근 생성된 마스크 조회
    async fn find_recent(
        &self,
        days: i32,
        limit: Option<i64>,
    ) -> Result<Vec<Mask>, ServiceError>;
    
    /// 마스크 그룹별 최근 생성된 마스크 조회
    async fn find_recent_by_mask_group_id(
        &self,
        mask_group_id: i32,
        days: i32,
        limit: Option<i64>,
    ) -> Result<Vec<Mask>, ServiceError>;
    
    /// 파일 경로로 마스크 조회
    async fn find_by_file_path(&self, file_path: &str) -> Result<Option<Mask>, ServiceError>;
    
    /// 체크섬으로 마스크 조회
    async fn find_by_checksum(&self, checksum: &str) -> Result<Vec<Mask>, ServiceError>;
    
    /// 파일 크기 범위로 마스크 조회
    async fn find_by_file_size_range(
        &self,
        min_size: i64,
        max_size: i64,
        offset: Option<i64>,
        limit: Option<i64>,
    ) -> Result<Vec<Mask>, ServiceError>;
    
    /// 이미지 크기로 마스크 조회
    async fn find_by_dimensions(
        &self,
        width: i32,
        height: i32,
        offset: Option<i64>,
        limit: Option<i64>,
    ) -> Result<Vec<Mask>, ServiceError>;
}
