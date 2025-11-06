use crate::domain::entities::access_condition::{AccessCondition, NewAccessCondition};

#[async_trait::async_trait]
pub trait AccessConditionRepository: Send + Sync {
    async fn create(
        &self,
        new_condition: &NewAccessCondition,
    ) -> Result<AccessCondition, sqlx::Error>;
    async fn find_by_id(&self, id: i32) -> Result<Option<AccessCondition>, sqlx::Error>;
    async fn list_by_project(&self, project_id: i32) -> Result<Vec<AccessCondition>, sqlx::Error>;
    async fn list_by_role(&self, role_id: i32) -> Result<Vec<AccessCondition>, sqlx::Error>;
}
