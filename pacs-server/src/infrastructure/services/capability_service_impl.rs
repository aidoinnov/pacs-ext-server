use crate::domain::entities::{Capability, NewCapability, Permission, Role, UpdateCapability};
use crate::domain::repositories::CapabilityRepository;
use crate::domain::services::CapabilityService;
use crate::domain::ServiceError;
use async_trait::async_trait;
use std::sync::Arc;

pub struct CapabilityServiceImpl<CR> {
    capability_repository: Arc<CR>,
}

impl<CR> CapabilityServiceImpl<CR>
where
    CR: CapabilityRepository + Send + Sync,
{
    pub fn new(capability_repository: Arc<CR>) -> Self {
        Self {
            capability_repository,
        }
    }
}

#[async_trait]
impl<CR> CapabilityService for CapabilityServiceImpl<CR>
where
    CR: CapabilityRepository + Send + Sync,
{
    async fn get_capability(&self, id: i32) -> Result<Capability, ServiceError> {
        self.capability_repository
            .find_by_id(id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
            .ok_or_else(|| ServiceError::NotFound(format!("Capability with id {} not found", id)))
    }

    async fn get_all_capabilities(&self) -> Result<Vec<Capability>, ServiceError> {
        self.capability_repository
            .find_all()
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    async fn get_capabilities_by_category(
        &self,
        category: &str,
    ) -> Result<Vec<Capability>, ServiceError> {
        self.capability_repository
            .find_by_category(category)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    async fn get_capability_with_permissions(
        &self,
        id: i32,
    ) -> Result<(Capability, Vec<Permission>), ServiceError> {
        let capability = self.get_capability(id).await?;
        let permissions = self
            .capability_repository
            .get_capability_permissions(id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        Ok((capability, permissions))
    }

    async fn create_capability(
        &self,
        new_capability: NewCapability,
    ) -> Result<Capability, ServiceError> {
        // 이름 중복 확인
        if let Ok(existing) = self.capability_repository.find_all().await {
            if existing.iter().any(|c| c.name == new_capability.name) {
                return Err(ServiceError::ValidationError(format!(
                    "Capability with name '{}' already exists",
                    new_capability.name
                )));
            }
        }

        self.capability_repository
            .create(new_capability)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    async fn update_capability(
        &self,
        id: i32,
        update: UpdateCapability,
    ) -> Result<Capability, ServiceError> {
        // Capability 존재 확인
        self.get_capability(id).await?;

        self.capability_repository
            .update(id, update)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    async fn delete_capability(&self, id: i32) -> Result<(), ServiceError> {
        // Capability 존재 확인
        self.get_capability(id).await?;

        let deleted = self
            .capability_repository
            .delete(id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        if !deleted {
            return Err(ServiceError::NotFound(format!(
                "Capability with id {} not found",
                id
            )));
        }

        Ok(())
    }

    async fn add_permission_to_capability(
        &self,
        capability_id: i32,
        permission_id: i32,
    ) -> Result<(), ServiceError> {
        // Capability 존재 확인
        self.get_capability(capability_id).await?;

        self.capability_repository
            .add_capability_permission(capability_id, permission_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    async fn remove_permission_from_capability(
        &self,
        capability_id: i32,
        permission_id: i32,
    ) -> Result<(), ServiceError> {
        // Capability 존재 확인
        self.get_capability(capability_id).await?;

        self.capability_repository
            .remove_capability_permission(capability_id, permission_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    async fn assign_capability_to_role(
        &self,
        role_id: i32,
        capability_id: i32,
    ) -> Result<(), ServiceError> {
        // Capability 존재 확인
        self.get_capability(capability_id).await?;

        self.capability_repository
            .assign_capability_to_role(role_id, capability_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    async fn remove_capability_from_role(
        &self,
        role_id: i32,
        capability_id: i32,
    ) -> Result<(), ServiceError> {
        self.capability_repository
            .remove_capability_from_role(role_id, capability_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    async fn get_role_capabilities(&self, role_id: i32) -> Result<Vec<Capability>, ServiceError> {
        self.capability_repository
            .get_role_capabilities(role_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    async fn get_global_role_capability_matrix_paginated(
        &self,
        page: i32,
        size: i32,
        search: Option<&str>,
        scope: Option<&str>,
    ) -> Result<(Vec<Role>, Vec<Capability>, Vec<(i32, i32)>, i64), ServiceError> {
        self.capability_repository
            .get_global_role_capability_matrix_paginated(page, size, search, scope)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    async fn get_global_role_capability_matrix(
        &self,
    ) -> Result<(Vec<Role>, Vec<Capability>, Vec<(i32, i32)>), ServiceError> {
        self.capability_repository
            .get_global_role_capability_matrix()
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }

    async fn get_project_role_capability_matrix(
        &self,
        project_id: i32,
    ) -> Result<(Vec<Role>, Vec<Capability>, Vec<(i32, i32)>), ServiceError> {
        self.capability_repository
            .get_project_role_capability_matrix(project_id)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))
    }
}
