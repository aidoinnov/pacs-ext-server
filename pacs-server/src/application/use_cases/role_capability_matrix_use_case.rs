use crate::application::dto::role_capability_matrix_dto::*;
use crate::domain::services::CapabilityService;
use crate::domain::ServiceError;
use std::collections::HashMap;
use std::sync::Arc;

pub struct RoleCapabilityMatrixUseCase {
    capability_service: Arc<dyn CapabilityService>,
}

impl RoleCapabilityMatrixUseCase {
    pub fn new(capability_service: Arc<dyn CapabilityService>) -> Self {
        Self { capability_service }
    }

    /// 전역 Role-Capability 매트릭스 조회 (페이지네이션 및 검색 포함)
    pub async fn get_global_matrix_paginated(
        &self,
        page: i32,
        size: i32,
        search: Option<String>,
        scope: Option<String>,
    ) -> Result<RoleCapabilityMatrixResponse, ServiceError> {
        let (roles, capabilities, assignments, total_count) = self
            .capability_service
            .get_global_role_capability_matrix_paginated(
                page,
                size,
                search.as_deref(),
                scope.as_deref(),
            )
            .await?;

        // 페이지네이션 정보 계산
        let total_pages = (total_count as f64 / size as f64).ceil() as i32;
        let pagination = PaginationInfo {
            current_page: page,
            page_size: size,
            total_pages,
            total_items: total_count,
            has_next: page < total_pages,
            has_previous: page > 1,
        };

        // 역할 정보 변환
        let role_infos: Vec<RoleInfo> = roles
            .into_iter()
            .map(|role| RoleInfo {
                id: role.id,
                name: role.name,
                description: role.description,
                scope: role.scope,
            })
            .collect();

        // Capability를 카테고리별로 그룹화
        let mut capabilities_by_category: HashMap<String, Vec<CapabilityInfo>> = HashMap::new();
        for capability in capabilities {
            // 성능 최적화: permission_count를 0으로 고정 (N+1 쿼리 문제 해결)
            let capability_info = CapabilityInfo {
                id: capability.id,
                name: capability.name,
                display_name: capability.display_name,
                display_label: capability.display_label,
                description: capability.description,
                category: capability.category.clone(),
                category_label: capability.category_label.clone(),
                permission_count: 0, // 임시로 0으로 고정
            };

            capabilities_by_category
                .entry(capability.category)
                .or_insert_with(Vec::new)
                .push(capability_info);
        }

        // 각 카테고리 내에서 Capability 정렬 (display_name 순)
        for capabilities in capabilities_by_category.values_mut() {
            capabilities.sort_by(|a, b| a.display_name.cmp(&b.display_name));
        }

        // 할당 정보 변환
        let assignment_set: std::collections::HashSet<(i32, i32)> =
            assignments.into_iter().collect();
        let assignments: Vec<RoleCapabilityAssignment> = role_infos
            .iter()
            .flat_map(|role| {
                let assignment_set = assignment_set.clone();
                capabilities_by_category
                    .values()
                    .flatten()
                    .map(move |capability| RoleCapabilityAssignment {
                        role_id: role.id,
                        capability_id: capability.id,
                        assigned: assignment_set.contains(&(role.id, capability.id)),
                    })
            })
            .collect();

        Ok(RoleCapabilityMatrixResponse {
            roles: role_infos,
            capabilities_by_category,
            assignments,
            pagination,
        })
    }

    /// 전역 Role-Capability 매트릭스 조회 (기존 - 하위 호환성)
    pub async fn get_global_matrix(&self) -> Result<RoleCapabilityMatrixResponse, ServiceError> {
        let (roles, capabilities, assignments) = self
            .capability_service
            .get_global_role_capability_matrix()
            .await?;

        // 역할 정보 변환
        let role_infos: Vec<RoleInfo> = roles
            .into_iter()
            .map(|role| RoleInfo {
                id: role.id,
                name: role.name,
                description: role.description,
                scope: role.scope,
            })
            .collect();

        // Capability를 카테고리별로 그룹화
        let mut capabilities_by_category: HashMap<String, Vec<CapabilityInfo>> = HashMap::new();
        for capability in capabilities {
            // 성능 최적화: permission_count를 0으로 고정 (N+1 쿼리 문제 해결)
            let capability_info = CapabilityInfo {
                id: capability.id,
                name: capability.name,
                display_name: capability.display_name,
                display_label: capability.display_label,
                description: capability.description,
                category: capability.category.clone(),
                category_label: capability.category_label.clone(),
                permission_count: 0, // 임시로 0으로 고정
            };

            capabilities_by_category
                .entry(capability.category)
                .or_insert_with(Vec::new)
                .push(capability_info);
        }

        // 각 카테고리 내에서 Capability 정렬 (display_name 순)
        for capabilities in capabilities_by_category.values_mut() {
            capabilities.sort_by(|a, b| a.display_name.cmp(&b.display_name));
        }

        // 할당 정보 변환
        let assignment_set: std::collections::HashSet<(i32, i32)> =
            assignments.into_iter().collect();
        let assignments: Vec<RoleCapabilityAssignment> = role_infos
            .iter()
            .flat_map(|role| {
                let assignment_set = assignment_set.clone();
                capabilities_by_category
                    .values()
                    .flatten()
                    .map(move |capability| RoleCapabilityAssignment {
                        role_id: role.id,
                        capability_id: capability.id,
                        assigned: assignment_set.contains(&(role.id, capability.id)),
                    })
            })
            .collect();

        // 빈 페이지네이션 정보 (하위 호환성)
        let pagination = PaginationInfo {
            current_page: 1,
            page_size: role_infos.len() as i32,
            total_pages: 1,
            total_items: role_infos.len() as i64,
            has_next: false,
            has_previous: false,
        };

        Ok(RoleCapabilityMatrixResponse {
            roles: role_infos,
            capabilities_by_category,
            assignments,
            pagination,
        })
    }

    /// 프로젝트별 Role-Capability 매트릭스 조회
    pub async fn get_project_matrix(
        &self,
        project_id: i32,
    ) -> Result<RoleCapabilityMatrixResponse, ServiceError> {
        let (roles, capabilities, assignments) = self
            .capability_service
            .get_project_role_capability_matrix(project_id)
            .await?;

        // 역할 정보 변환
        let role_infos: Vec<RoleInfo> = roles
            .into_iter()
            .map(|role| RoleInfo {
                id: role.id,
                name: role.name,
                description: role.description,
                scope: role.scope,
            })
            .collect();

        // Capability를 카테고리별로 그룹화
        let mut capabilities_by_category: HashMap<String, Vec<CapabilityInfo>> = HashMap::new();
        for capability in capabilities {
            // 성능 최적화: permission_count를 0으로 고정 (N+1 쿼리 문제 해결)
            let capability_info = CapabilityInfo {
                id: capability.id,
                name: capability.name,
                display_name: capability.display_name,
                display_label: capability.display_label,
                description: capability.description,
                category: capability.category.clone(),
                category_label: capability.category_label.clone(),
                permission_count: 0, // 임시로 0으로 고정
            };

            capabilities_by_category
                .entry(capability.category)
                .or_insert_with(Vec::new)
                .push(capability_info);
        }

        // 각 카테고리 내에서 Capability 정렬 (display_name 순)
        for capabilities in capabilities_by_category.values_mut() {
            capabilities.sort_by(|a, b| a.display_name.cmp(&b.display_name));
        }

        // 할당 정보 변환
        let assignment_set: std::collections::HashSet<(i32, i32)> =
            assignments.into_iter().collect();
        let assignments: Vec<RoleCapabilityAssignment> = role_infos
            .iter()
            .flat_map(|role| {
                let assignment_set = assignment_set.clone();
                capabilities_by_category
                    .values()
                    .flatten()
                    .map(move |capability| RoleCapabilityAssignment {
                        role_id: role.id,
                        capability_id: capability.id,
                        assigned: assignment_set.contains(&(role.id, capability.id)),
                    })
            })
            .collect();

        // 빈 페이지네이션 정보 (하위 호환성)
        let pagination = PaginationInfo {
            current_page: 1,
            page_size: role_infos.len() as i32,
            total_pages: 1,
            total_items: role_infos.len() as i64,
            has_next: false,
            has_previous: false,
        };

        Ok(RoleCapabilityMatrixResponse {
            roles: role_infos,
            capabilities_by_category,
            assignments,
            pagination,
        })
    }

    /// Capability 상세 조회 (매핑된 Permission 포함)
    pub async fn get_capability_detail(
        &self,
        capability_id: i32,
    ) -> Result<CapabilityDetailResponse, ServiceError> {
        let (capability, permissions) = self
            .capability_service
            .get_capability_with_permissions(capability_id)
            .await?;

        let capability_info = CapabilityInfo {
            id: capability.id,
            name: capability.name,
            display_name: capability.display_name,
            display_label: capability.display_label,
            description: capability.description,
            category: capability.category,
            category_label: capability.category_label,
            permission_count: permissions.len() as i32,
        };

        let permission_infos: Vec<PermissionInfo> = permissions
            .into_iter()
            .map(|permission| PermissionInfo {
                id: permission.id,
                category: permission.category,
                resource_type: permission.resource_type,
                action: permission.action,
            })
            .collect();

        Ok(CapabilityDetailResponse {
            capability: capability_info,
            permissions: permission_infos,
        })
    }

    /// Capability 할당/제거
    pub async fn update_capability_assignment(
        &self,
        role_id: i32,
        capability_id: i32,
        assign: bool,
    ) -> Result<(), ServiceError> {
        if assign {
            self.capability_service
                .assign_capability_to_role(role_id, capability_id)
                .await
        } else {
            self.capability_service
                .remove_capability_from_role(role_id, capability_id)
                .await
        }
    }

    /// 모든 Capability 목록 조회
    pub async fn get_all_capabilities(&self) -> Result<Vec<CapabilityInfo>, ServiceError> {
        let capabilities = self.capability_service.get_all_capabilities().await?;

        let mut capability_infos = Vec::new();
        for capability in capabilities {
            let permissions = self
                .capability_service
                .get_capability_with_permissions(capability.id)
                .await?
                .1;

            let capability_info = CapabilityInfo {
                id: capability.id,
                name: capability.name,
                display_name: capability.display_name,
                display_label: capability.display_label,
                description: capability.description,
                category: capability.category,
                category_label: capability.category_label,
                permission_count: permissions.len() as i32,
            };

            capability_infos.push(capability_info);
        }

        Ok(capability_infos)
    }

    /// 카테고리별 Capability 목록 조회
    pub async fn get_capabilities_by_category(
        &self,
        category: &str,
    ) -> Result<Vec<CapabilityInfo>, ServiceError> {
        let capabilities = self
            .capability_service
            .get_capabilities_by_category(category)
            .await?;

        let mut capability_infos = Vec::new();
        for capability in capabilities {
            let permissions = self
                .capability_service
                .get_capability_with_permissions(capability.id)
                .await?
                .1;

            let capability_info = CapabilityInfo {
                id: capability.id,
                name: capability.name,
                display_name: capability.display_name,
                display_label: capability.display_label,
                description: capability.description,
                category: capability.category,
                category_label: capability.category_label,
                permission_count: permissions.len() as i32,
            };

            capability_infos.push(capability_info);
        }

        Ok(capability_infos)
    }
}
