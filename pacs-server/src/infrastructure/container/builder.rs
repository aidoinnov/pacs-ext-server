use std::sync::Arc;
use crate::infrastructure::config::Settings;
use crate::infrastructure::auth::JwtService;
use crate::infrastructure::external::*;
use crate::infrastructure::repositories::*;
use crate::infrastructure::services::*;
use crate::domain::services::*;
use crate::application::use_cases::*;
use sqlx::postgres::PgPoolOptions;

use super::app_container::AppContainer;

pub async fn build_container(settings: &Settings) -> AppContainer {
    // =========================
    // Database
    // =========================
    let pool = PgPoolOptions::new()
        .max_connections(settings.database.max_connections)
        .min_connections(settings.database.min_connections)
        .connect(&settings.database_url())
        .await
        .expect("DB connect failed");

    // =========================
    // External Clients
    // =========================
    let jwt = Arc::new(JwtService::new(&settings.jwt));
    let keycloak = Arc::new(KeycloakClient::new(settings.keycloak.clone()));
    let qido = Arc::new(Dcm4cheeQidoClient::new(settings.dcm4chee.clone()));

    // =========================
    // Repositories
    // =========================
    let user_repo = Arc::new(UserRepositoryImpl::new(pool.clone()));
    let project_repo = Arc::new(ProjectRepositoryImpl::new(pool.clone()));
    let role_repo = Arc::new(RoleRepositoryImpl::new(pool.clone()));
    let permission_repo = Arc::new(PermissionRepositoryImpl::new(pool.clone()));
    let annotation_repo = Arc::new(AnnotationRepositoryImpl::new(pool.clone()));
    let mask_repo = Arc::new(MaskRepositoryImpl::new(pool.clone()));
    let mask_group_repo = Arc::new(MaskGroupRepositoryImpl::new(pool.clone()));
    let project_data_repo = Arc::new(ProjectDataRepositoryImpl::new(pool.clone()));
    let project_data_access_repo =
        Arc::new(ProjectDataAccessRepositoryImpl::new(pool.clone()));
    let series_user_note_repo =
        Arc::new(SeriesUserNoteRepositoryImpl::new(pool.clone()));
    let access_log_repo = Arc::new(AccessLogRepositoryImpl::new(pool.clone()));
    let capability_repo = Arc::new(CapabilityRepositoryImpl::new(pool.clone()));

    // =========================
    // Domain Services
    // =========================
    let auth_service = Arc::new(AuthServiceImpl::new(
        user_repo.clone(),
        jwt.clone(),
        keycloak.clone(),
    ));

    let user_service =
        Arc::new(UserServiceImpl::new(user_repo.clone(), project_repo.clone()));

    let project_service = Arc::new(ProjectServiceImpl::new(
        project_repo.clone(),
        user_repo.clone(),
        role_repo.clone(),
    ));

    let permission_service = Arc::new(
        PermissionServiceImpl::new(permission_repo.clone(), role_repo.clone())
    );

    let access_control_service = Arc::new(
        AccessControlServiceImpl::new(
            access_log_repo.clone(),
            user_repo.clone(),
            project_repo.clone(),
            role_repo.clone(),
            permission_repo.clone(),
        )
    );

    let annotation_service = Arc::new(
        AnnotationServiceImpl::new(
            annotation_repo.clone(),
            user_repo.clone(),
            project_repo.clone(),
        )
    );

    let mask_group_service = Arc::new(
        MaskGroupServiceImpl::new(
            mask_group_repo.clone(),
            annotation_repo.clone(),
            user_repo.clone(),
        )
    );

    let mask_service = Arc::new(
        MaskServiceImpl::new(
            mask_repo.clone(),
            mask_group_repo.clone(),
            user_repo.clone(),
        )
    );

    let project_data_service = Arc::new(
        ProjectDataServiceImpl::new(
            project_data_repo.clone(),
            project_data_access_repo.clone(),
        )
    );

    let series_user_note_service = Arc::new(
        SeriesUserNoteServiceImpl::new(
            series_user_note_repo.clone(),
            user_repo.clone(),
            project_repo.clone(),
            project_data_repo.clone(),
        )
    );

    let capability_service =
        Arc::new(CapabilityServiceImpl::new(capability_repo.clone()));

    // =========================
    // UseCases
    // =========================
    let auth_uc = Arc::new(AuthUseCase::new(auth_service.clone()));
    let user_uc = Arc::new(UserUseCase::new(user_service.clone()));
    let project_uc = Arc::new(ProjectUseCase::new(project_service.clone()));
    let permission_uc = Arc::new(PermissionUseCase::new(permission_service.clone()));
    let access_control_uc =
        Arc::new(AccessControlUseCase::new(access_control_service.clone()));

    let annotation_uc = Arc::new(
        AnnotationUseCase::new(
            annotation_service.clone(),
            user_repo.clone(),
            access_control_service.clone(),
        )
    );

    let mask_group_uc =
        Arc::new(MaskGroupUseCase::new(mask_group_service.clone(), /* signed url */));

    let mask_uc =
        Arc::new(MaskUseCase::new(
            mask_service.clone(),
            mask_group_service.clone(),
            /* signed url */
        ));

    let project_user_uc =
        Arc::new(ProjectUserUseCase::new(
            project_service.clone(),
            user_service.clone(),
            project_data_service.clone(),
        ));

    let project_user_matrix_uc =
        Arc::new(ProjectUserMatrixUseCase::new(
            project_service.clone(),
            user_service.clone(),
        ));

    let user_project_matrix_uc =
        Arc::new(UserProjectMatrixUseCase::new(
            user_service.clone(),
            project_service.clone(),
        ));

    let role_permission_matrix_uc =
        Arc::new(RolePermissionMatrixUseCase::new(permission_service.clone()));

    let role_capability_matrix_uc =
        Arc::new(RoleCapabilityMatrixUseCase::new(capability_service.clone()));

    let project_data_access_uc =
        Arc::new(ProjectDataAccessUseCase::new(
            project_data_service.clone(),
            project_service.clone(),
        ));

    let user_registration_uc =
        Arc::new(UserRegistrationUseCase::new(
            UserRegistrationServiceImpl::new(pool.clone(), keycloak.clone())
        ));

    let series_user_note_uc =
        Arc::new(SeriesUserNoteUseCase::new(
            series_user_note_service.clone(),
            user_repo.clone(),
        ));

    AppContainer {
        pool,
        jwt,
        keycloak,
        qido,

        user_repo,
        project_repo,
        role_repo,
        permission_repo,
        annotation_repo,
        mask_repo,
        mask_group_repo,
        project_data_repo,
        project_data_access_repo,
        series_user_note_repo,
        access_log_repo,
        capability_repo,

        auth_service,
        user_service,
        project_service,
        permission_service,
        access_control_service,
        annotation_service,
        mask_service,
        mask_group_service,
        project_data_service,
        series_user_note_service,
        capability_service,

        auth_uc,
        user_uc,
        project_uc,
        permission_uc,
        access_control_uc,
        annotation_uc,
        mask_uc,
        mask_group_uc,
        project_user_uc,
        project_user_matrix_uc,
        user_project_matrix_uc,
        role_permission_matrix_uc,
        role_capability_matrix_uc,
        project_data_access_uc,
        user_registration_uc,
        series_user_note_uc,

        sync_service: None,
    }
}
