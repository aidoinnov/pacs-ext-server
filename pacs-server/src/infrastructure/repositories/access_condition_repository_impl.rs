use crate::domain::entities::access_condition::{AccessCondition, NewAccessCondition};
use crate::domain::repositories::AccessConditionRepository;
use sqlx::PgPool;

pub struct AccessConditionRepositoryImpl {
    pub pool: PgPool,
}

impl AccessConditionRepositoryImpl {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl AccessConditionRepository for AccessConditionRepositoryImpl {
    async fn create(&self, new_condition: &NewAccessCondition) -> Result<AccessCondition, sqlx::Error> {
        let rec = sqlx::query_as::<_, AccessCondition>(
            "INSERT INTO security_access_condition \
             (resource_type, resource_level, dicom_tag, operator, value, condition_type) \
             VALUES ($1, $2, $3, $4, $5, $6) \
             RETURNING id, resource_type, resource_level, dicom_tag, operator, value, condition_type, created_at",
        )
        .bind(&new_condition.resource_type)
        .bind(&new_condition.resource_level)
        .bind(&new_condition.dicom_tag)
        .bind(&new_condition.operator)
        .bind(&new_condition.value)
        .bind(&new_condition.condition_type)
        .fetch_one(&self.pool)
        .await?;
        Ok(rec)
    }

    async fn find_by_id(&self, id: i32) -> Result<Option<AccessCondition>, sqlx::Error> {
        let rec = sqlx::query_as::<_, AccessCondition>(
            "SELECT id, resource_type, resource_level, dicom_tag, operator, value, condition_type, created_at \
             FROM security_access_condition WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(rec)
    }

    async fn list_by_project(&self, project_id: i32) -> Result<Vec<AccessCondition>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AccessCondition>(
            "SELECT ac.id, ac.resource_type, ac.resource_level, ac.dicom_tag, ac.operator, ac.value, ac.condition_type, ac.created_at \
             FROM security_access_condition ac \
             JOIN security_project_dicom_condition pc ON pc.access_condition_id = ac.id \
             WHERE pc.project_id = $1 \
             ORDER BY pc.priority DESC, ac.id ASC",
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    async fn list_by_role(&self, role_id: i32) -> Result<Vec<AccessCondition>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AccessCondition>(
            "SELECT ac.id, ac.resource_type, ac.resource_level, ac.dicom_tag, ac.operator, ac.value, ac.condition_type, ac.created_at \
             FROM security_access_condition ac \
             JOIN security_role_dicom_condition rc ON rc.access_condition_id = ac.id \
             WHERE rc.role_id = $1 \
             ORDER BY rc.priority DESC, ac.id ASC",
        )
        .bind(role_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }
}


