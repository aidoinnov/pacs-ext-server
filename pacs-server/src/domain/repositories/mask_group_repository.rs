use async_trait::async_trait;
use crate::domain::entities::{MaskGroup, NewMaskGroup, UpdateMaskGroup, MaskGroupStats};
use crate::domain::services::ServiceError;

/// 마스크 그룹 Repository trait
#[async_trait]
pub trait MaskGroupRepository: Send + Sync {
    /// 마스크 그룹 생성
    async fn create(&self, mask_group: &NewMaskGroup) -> Result<MaskGroup, ServiceError>;
    
    /// ID로 마스크 그룹 조회
    async fn find_by_id(&self, id: i32) -> Result<Option<MaskGroup>, ServiceError>;
    
    /// 어노테이션 ID로 마스크 그룹 목록 조회
    async fn find_by_annotation_id(&self, annotation_id: i32) -> Result<Vec<MaskGroup>, ServiceError>;
    
    /// 사용자 ID로 마스크 그룹 목록 조회
    async fn find_by_user_id(&self, user_id: i32) -> Result<Vec<MaskGroup>, ServiceError>;
    
    /// 그룹 이름으로 마스크 그룹 검색
    async fn find_by_group_name(&self, group_name: &str) -> Result<Vec<MaskGroup>, ServiceError>;
    
    /// 모달리티로 마스크 그룹 필터링
    async fn find_by_modality(&self, modality: &str) -> Result<Vec<MaskGroup>, ServiceError>;
    
    /// 마스크 타입으로 마스크 그룹 필터링
    async fn find_by_mask_type(&self, mask_type: &str) -> Result<Vec<MaskGroup>, ServiceError>;
    
    /// 모든 마스크 그룹 조회 (페이징 지원)
    async fn find_all(
        &self,
        offset: Option<i64>,
        limit: Option<i64>,
    ) -> Result<Vec<MaskGroup>, ServiceError>;
    
    /// 마스크 그룹 업데이트
    async fn update(&self, id: i32, update: &UpdateMaskGroup) -> Result<MaskGroup, ServiceError>;
    
    /// 마스크 그룹 삭제
    async fn delete(&self, id: i32) -> Result<(), ServiceError>;
    
    /// 어노테이션의 모든 마스크 그룹 삭제
    async fn delete_by_annotation_id(&self, annotation_id: i32) -> Result<(), ServiceError>;
    
    /// 마스크 그룹 존재 여부 확인
    async fn exists(&self, id: i32) -> Result<bool, ServiceError>;
    
    /// 마스크 그룹 개수 조회
    async fn count(&self) -> Result<i64, ServiceError>;
    
    /// 어노테이션의 마스크 그룹 개수 조회
    async fn count_by_annotation_id(&self, annotation_id: i32) -> Result<i64, ServiceError>;
    
    /// 사용자의 마스크 그룹 개수 조회
    async fn count_by_user_id(&self, user_id: i32) -> Result<i64, ServiceError>;
    
    /// 마스크 그룹 통계 조회
    async fn get_stats(&self) -> Result<MaskGroupStats, ServiceError>;
    
    /// 사용자별 마스크 그룹 통계 조회
    async fn get_stats_by_user_id(&self, user_id: i32) -> Result<MaskGroupStats, ServiceError>;
    
    /// 어노테이션별 마스크 그룹 통계 조회
    async fn get_stats_by_annotation_id(&self, annotation_id: i32) -> Result<MaskGroupStats, ServiceError>;
    
    /// 모달리티별 마스크 그룹 통계 조회
    async fn get_stats_by_modality(&self, modality: &str) -> Result<MaskGroupStats, ServiceError>;
    
    /// 마스크 타입별 마스크 그룹 통계 조회
    async fn get_stats_by_mask_type(&self, mask_type: &str) -> Result<MaskGroupStats, ServiceError>;
    
    /// 검색 조건으로 마스크 그룹 검색
    async fn search(
        &self,
        query: &str,
        modality: Option<&str>,
        mask_type: Option<&str>,
        user_id: Option<i32>,
        offset: Option<i64>,
        limit: Option<i64>,
    ) -> Result<Vec<MaskGroup>, ServiceError>;
    
    /// 최근 생성된 마스크 그룹 조회
    async fn find_recent(
        &self,
        days: i32,
        limit: Option<i64>,
    ) -> Result<Vec<MaskGroup>, ServiceError>;
    
    /// 사용자별 최근 생성된 마스크 그룹 조회
    async fn find_recent_by_user_id(
        &self,
        user_id: i32,
        days: i32,
        limit: Option<i64>,
    ) -> Result<Vec<MaskGroup>, ServiceError>;
}
