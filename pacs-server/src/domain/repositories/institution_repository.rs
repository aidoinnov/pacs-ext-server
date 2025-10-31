use crate::domain::entities::institution::{
    NewProjectDataInstitution, NewSecurityInstitution, ProjectDataInstitution, SecurityInstitution,
};

#[async_trait::async_trait]
pub trait InstitutionRepository: Send + Sync {
    // security_institution
    async fn create_security_institution(
        &self,
        new_inst: &NewSecurityInstitution,
    ) -> Result<SecurityInstitution, sqlx::Error>;
    async fn find_security_institution_by_id(
        &self,
        id: i32,
    ) -> Result<Option<SecurityInstitution>, sqlx::Error>;
    async fn find_security_institution_by_code(
        &self,
        code: &str,
    ) -> Result<Option<SecurityInstitution>, sqlx::Error>;

    // project_data_institution
    async fn create_data_institution(
        &self,
        new_inst: &NewProjectDataInstitution,
    ) -> Result<ProjectDataInstitution, sqlx::Error>;
    async fn find_data_institution_by_id(
        &self,
        id: i32,
    ) -> Result<Option<ProjectDataInstitution>, sqlx::Error>;
    async fn find_data_institution_by_code(
        &self,
        code: &str,
    ) -> Result<Option<ProjectDataInstitution>, sqlx::Error>;
}
