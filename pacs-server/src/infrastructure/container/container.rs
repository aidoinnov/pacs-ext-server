use std::sync::Arc;
use sqlx::PgPool;

use crate::infrastructure::repositories::*;
use crate::domain::services::*;
use crate::application::use-cases::*;
use crate::infrastructure::auth::JwtService;
use crate::infrastructure::config::Settings;


pub struct AppConatiner {
    // =================
    // Infra (Shared)
    // =================
    pub pool: PgPool
    pub jwt: Arc<JwtService>,
    pub keycloak: Arc<KeycloakClient>,
    pub qido: Arc<Dcm4cheeQidoClient>,

    // =========================
    // Repositories
    // =========================
    pub user_repo: Arc<UserRepositoryImpl>,
    pub project_repo: Arc<ProjectRepositoryImpl>,
    pub role_repo: Arc<RoleRepositoryImpl>,
    pub permission_repo: Arc<PermissionRepositoryImpl>,
    pub annotation_repo: Arc<AnnotationRepositoryImpl>,
    pub mask_repo: Arc<MaskRepositoryImpl>,
    pub mask_group_repo: Arc<MaskGroupRepositoryImpl>,
    pub project_data_repo: Arc<ProjectDataRepositoryImpl>,
    pub project_data_access_repo: Arc<ProjectDataAccessRepositoryImpl>,
    pub series_user_note_repo: Arc<SeriesUserNoteRepositoryImpl>,
    pub access_log_repo: Arc<AccessLogRepositoryImpl>,
    pub capability_repo: Arc<CapabilityRepositoryImpl>,
    
    // =========================
    // Domain Services
    // =========================
    pub auth_service: Arc<AuthServiceImpl>,
    pub user_service: Arc<UserServiceImpl>,
    pub project_service: Arc<ProjectServiceImpl>,
    pub permission_service: Arc<PermissionServiceImpl<PermissionRepositoryImpl, RoleRepositoryImpl>>,
    pub access_control_service: Arc<AccessControlServiceImpl>,
    pub annotation_service: Arc<AnnotationServiceImpl<
        AnnotationRepositoryImpl,
        UserRepositoryImpl,
        ProjectRepositoryImpl,
    >>,
    pub mask_service: Arc<MaskServiceImpl>,
    pub mask_group_service: Arc<MaskGroupServiceImpl>,
    pub project_data_service: Arc<ProjectDataServiceImpl>,
    pub series_user_note_service: Arc<SeriesUserNoteServiceImpl>,
    pub capability_service: Arc<CapabilityServiceImpl>,
    

    // =========================
    // Application UseCases
    // =========================
    pub auth_uc: Arc<AuthUseCase>,
    pub user_uc: Arc<UserUseCase>,
    pub project_uc: Arc<ProjectUseCase>,
    pub permission_uc: Arc<PermissionUseCase>,
    pub access_control_uc: Arc<AccessControlUseCase>,
    pub annotation_uc: Arc<AnnotationUseCase>,
    pub mask_uc: Arc<MaskUseCase>,
    pub mask_group_uc: Arc<MaskGroupUseCase>,
    pub project_user_uc: Arc<ProjectUserUseCase>,
    pub project_user_matrix_uc: Arc<ProjectUserMatrixUseCase>,
    pub user_project_matrix_uc: Arc<UserProjectMatrixUseCase>,
    pub role_permission_matrix_uc: Arc<RolePermissionMatrixUseCase>,
    pub role_capability_matrix_uc: Arc<RoleCapabilityMatrixUseCase>,
    pub project_data_access_uc: Arc<ProjectDataAccessUseCase>,
    pub user_registration_uc: Arc<UserRegistrationUseCase>,
    pub series_user_note_uc: Arc<SeriesUserNoteUseCase>,


    // =========================
    // Optional / Runtime
    // =========================
    pub sync_service: Option<Arc<dyn SyncService>>,
}