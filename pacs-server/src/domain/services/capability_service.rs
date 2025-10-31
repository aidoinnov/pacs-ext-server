use crate::domain::entities::{Capability, NewCapability, Permission, Role, UpdateCapability};
use crate::domain::ServiceError;
use async_trait::async_trait;

#[async_trait]
pub trait CapabilityService: Send + Sync {
    /// ID로 Capability 조회
    async fn get_capability(&self, id: i32) -> Result<Capability, ServiceError>;

    /// 모든 Capability 조회
    async fn get_all_capabilities(&self) -> Result<Vec<Capability>, ServiceError>;

    /// 카테고리별 Capability 조회
    async fn get_capabilities_by_category(
        &self,
        category: &str,
    ) -> Result<Vec<Capability>, ServiceError>;

    /// Capability와 매핑된 Permission 목록 조회
    async fn get_capability_with_permissions(
        &self,
        id: i32,
    ) -> Result<(Capability, Vec<Permission>), ServiceError>;

    /// 새 Capability 생성
    async fn create_capability(
        &self,
        new_capability: NewCapability,
    ) -> Result<Capability, ServiceError>;

    /// Capability 수정
    async fn update_capability(
        &self,
        id: i32,
        update: UpdateCapability,
    ) -> Result<Capability, ServiceError>;

    /// Capability 삭제
    async fn delete_capability(&self, id: i32) -> Result<(), ServiceError>;

    /// Capability에 Permission 매핑 추가
    async fn add_permission_to_capability(
        &self,
        capability_id: i32,
        permission_id: i32,
    ) -> Result<(), ServiceError>;

    /// Capability에서 Permission 매핑 제거
    async fn remove_permission_from_capability(
        &self,
        capability_id: i32,
        permission_id: i32,
    ) -> Result<(), ServiceError>;

    /// 역할에 Capability 할당
    async fn assign_capability_to_role(
        &self,
        role_id: i32,
        capability_id: i32,
    ) -> Result<(), ServiceError>;

    /// 역할에서 Capability 제거
    async fn remove_capability_from_role(
        &self,
        role_id: i32,
        capability_id: i32,
    ) -> Result<(), ServiceError>;

    /// 역할의 Capability 목록 조회
    async fn get_role_capabilities(&self, role_id: i32) -> Result<Vec<Capability>, ServiceError>;

    /// 전역 Role-Capability 매트릭스 조회 (페이지네이션 및 검색 포함)
    async fn get_global_role_capability_matrix_paginated(
        &self,
        page: i32,
        size: i32,
        search: Option<&str>,
        scope: Option<&str>,
    ) -> Result<(Vec<Role>, Vec<Capability>, Vec<(i32, i32)>, i64), ServiceError>;

    /// 전역 Role-Capability 매트릭스 조회 (기존 - 하위 호환성)
    async fn get_global_role_capability_matrix(
        &self,
    ) -> Result<(Vec<Role>, Vec<Capability>, Vec<(i32, i32)>), ServiceError>;

    /// 프로젝트별 Role-Capability 매트릭스 조회
    async fn get_project_role_capability_matrix(
        &self,
        project_id: i32,
    ) -> Result<(Vec<Role>, Vec<Capability>, Vec<(i32, i32)>), ServiceError>;
}
