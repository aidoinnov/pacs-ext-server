use crate::domain::entities::institution::{
    NewProjectDataInstitution, NewSecurityInstitution, ProjectDataInstitution, SecurityInstitution,
};
use crate::domain::repositories::InstitutionRepository;
use sqlx::PgPool;

pub struct InstitutionRepositoryImpl {
    pub pool: PgPool,
}

impl InstitutionRepositoryImpl {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl InstitutionRepository for InstitutionRepositoryImpl {
    async fn create_security_institution(
        &self,
        new_inst: &NewSecurityInstitution,
    ) -> Result<SecurityInstitution, sqlx::Error> {
        let rec = sqlx::query_as::<_, SecurityInstitution>(
            "INSERT INTO security_institution \
             (institution_code, institution_name, institution_type, address, phone, email) \
             VALUES ($1, $2, $3, $4, $5, $6) \
             RETURNING id, institution_code, institution_name, institution_type, address, phone, email, is_active, created_at, updated_at",
        )
        .bind(&new_inst.institution_code)
        .bind(&new_inst.institution_name)
        .bind(&new_inst.institution_type)
        .bind(&new_inst.address)
        .bind(&new_inst.phone)
        .bind(&new_inst.email)
        .fetch_one(&self.pool)
        .await?;
        Ok(rec)
    }

    async fn find_security_institution_by_id(
        &self,
        id: i32,
    ) -> Result<Option<SecurityInstitution>, sqlx::Error> {
        let rec = sqlx::query_as::<_, SecurityInstitution>(
            "SELECT id, institution_code, institution_name, institution_type, address, phone, email, is_active, created_at, updated_at \
             FROM security_institution WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(rec)
    }

    async fn find_security_institution_by_code(
        &self,
        code: &str,
    ) -> Result<Option<SecurityInstitution>, sqlx::Error> {
        let rec = sqlx::query_as::<_, SecurityInstitution>(
            "SELECT id, institution_code, institution_name, institution_type, address, phone, email, is_active, created_at, updated_at \
             FROM security_institution WHERE institution_code = $1",
        )
        .bind(code)
        .fetch_optional(&self.pool)
        .await?;
        Ok(rec)
    }

    async fn create_data_institution(
        &self,
        new_inst: &NewProjectDataInstitution,
    ) -> Result<ProjectDataInstitution, sqlx::Error> {
        let rec = sqlx::query_as::<_, ProjectDataInstitution>(
            "INSERT INTO project_data_institution \
             (institution_code, institution_name, institution_type, address, phone, email) \
             VALUES ($1, $2, $3, $4, $5, $6) \
             RETURNING id, institution_code, institution_name, institution_type, address, phone, email, is_active, created_at, updated_at",
        )
        .bind(&new_inst.institution_code)
        .bind(&new_inst.institution_name)
        .bind(&new_inst.institution_type)
        .bind(&new_inst.address)
        .bind(&new_inst.phone)
        .bind(&new_inst.email)
        .fetch_one(&self.pool)
        .await?;
        Ok(rec)
    }

    async fn find_data_institution_by_id(
        &self,
        id: i32,
    ) -> Result<Option<ProjectDataInstitution>, sqlx::Error> {
        let rec = sqlx::query_as::<_, ProjectDataInstitution>(
            "SELECT id, institution_code, institution_name, institution_type, address, phone, email, is_active, created_at, updated_at \
             FROM project_data_institution WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(rec)
    }

    async fn find_data_institution_by_code(
        &self,
        code: &str,
    ) -> Result<Option<ProjectDataInstitution>, sqlx::Error> {
        let rec = sqlx::query_as::<_, ProjectDataInstitution>(
            "SELECT id, institution_code, institution_name, institution_type, address, phone, email, is_active, created_at, updated_at \
             FROM project_data_institution WHERE institution_code = $1",
        )
        .bind(code)
        .fetch_optional(&self.pool)
        .await?;
        Ok(rec)
    }
}
