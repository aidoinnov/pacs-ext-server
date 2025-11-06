use crate::domain::entities::{Capability, NewCapability, Permission, Role, UpdateCapability};
use async_trait::async_trait;
use sqlx::PgPool;

#[async_trait]
pub trait CapabilityRepository: Send + Sync {
    /// ID로 Capability 조회
    async fn find_by_id(&self, id: i32) -> Result<Option<Capability>, sqlx::Error>;

    /// 모든 Capability 조회
    async fn find_all(&self) -> Result<Vec<Capability>, sqlx::Error>;

    /// 카테고리별 Capability 조회
    async fn find_by_category(&self, category: &str) -> Result<Vec<Capability>, sqlx::Error>;

    /// Capability에 매핑된 Permission 목록 조회
    async fn get_capability_permissions(
        &self,
        capability_id: i32,
    ) -> Result<Vec<Permission>, sqlx::Error>;

    /// 역할에 할당된 Capability 목록 조회
    async fn get_role_capabilities(&self, role_id: i32) -> Result<Vec<Capability>, sqlx::Error>;

    /// 새 Capability 생성
    async fn create(&self, new_capability: NewCapability) -> Result<Capability, sqlx::Error>;

    /// Capability 수정
    async fn update(&self, id: i32, update: UpdateCapability) -> Result<Capability, sqlx::Error>;

    /// Capability 삭제
    async fn delete(&self, id: i32) -> Result<bool, sqlx::Error>;

    /// Capability에 Permission 매핑 추가
    async fn add_capability_permission(
        &self,
        capability_id: i32,
        permission_id: i32,
    ) -> Result<(), sqlx::Error>;

    /// Capability에서 Permission 매핑 제거
    async fn remove_capability_permission(
        &self,
        capability_id: i32,
        permission_id: i32,
    ) -> Result<(), sqlx::Error>;

    /// 역할에 Capability 할당
    async fn assign_capability_to_role(
        &self,
        role_id: i32,
        capability_id: i32,
    ) -> Result<(), sqlx::Error>;

    /// 역할에서 Capability 제거
    async fn remove_capability_from_role(
        &self,
        role_id: i32,
        capability_id: i32,
    ) -> Result<(), sqlx::Error>;

    /// 전역 Role-Capability 매트릭스 조회 (페이지네이션 및 검색 포함)
    async fn get_global_role_capability_matrix_paginated(
        &self,
        page: i32,
        size: i32,
        search: Option<&str>,
        scope: Option<&str>,
    ) -> Result<(Vec<Role>, Vec<Capability>, Vec<(i32, i32)>, i64), sqlx::Error>;

    /// 전역 Role-Capability 매트릭스 조회 (기존 - 하위 호환성)
    async fn get_global_role_capability_matrix(
        &self,
    ) -> Result<(Vec<Role>, Vec<Capability>, Vec<(i32, i32)>), sqlx::Error>;

    /// 프로젝트별 Role-Capability 매트릭스 조회
    async fn get_project_role_capability_matrix(
        &self,
        project_id: i32,
    ) -> Result<(Vec<Role>, Vec<Capability>, Vec<(i32, i32)>), sqlx::Error>;

    /// 데이터베이스 연결 풀 반환
    fn pool(&self) -> &PgPool;
}
