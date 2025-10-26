use utoipa::OpenApi;
use crate::presentation::controllers::auth_controller_docs::*;
use crate::presentation::controllers::annotation_controller::*;
use crate::presentation::controllers::project_controller::*;
use crate::presentation::controllers::mask_group_controller::*;
use crate::presentation::controllers::project_user_matrix_controller::*;
use crate::presentation::controllers::user_project_matrix_controller;
use crate::presentation::controllers::role_permission_matrix_controller::*;
use crate::presentation::controllers::project_data_access_controller::*;
use crate::application::dto::auth_dto::*;
use crate::application::dto::user_dto::*;
use crate::application::dto::project_dto::*;
use crate::application::dto::annotation_dto::*;
use crate::application::dto::mask_group_dto::*;
use crate::application::dto::permission_dto::*;
use crate::application::dto::project_user_dto::{UserWithRoleResponse, ProjectWithRoleResponse, AssignRoleRequest, BatchAssignRolesRequest, UserRoleAssignment, RoleAssignmentResponse, BatchRoleAssignmentResponse, FailedAssignment};
use crate::application::dto::project_user_matrix_dto::*;
use crate::application::dto::user_project_matrix_dto::*;
use crate::application::dto::role_permission_matrix_dto::*;
use crate::application::dto::project_data_access_dto::*;
use crate::application::dto::user_registration_dto::*;

#[derive(OpenApi)]
#[openapi(
    paths(
        // Auth endpoints
        login_doc,
        verify_token_doc,
        refresh_token_doc,
        // Annotation endpoints
        create_annotation,
        get_annotation,
        list_annotations,
        update_annotation,
        delete_annotation,
        // Project endpoints
        create_project,
        get_project,
        list_projects,
        get_active_projects,
        // Mask Group endpoints
        create_mask_group,
        get_mask_group,
        list_mask_groups,
        update_mask_group,
        delete_mask_group,
        generate_upload_url,
        complete_upload,
        // Project User Matrix endpoints
        get_matrix,
        // User Project Matrix endpoints
        user_project_matrix_controller::get_matrix,
        // Role Permission Matrix endpoints
        get_global_matrix,
        get_project_matrix,
        update_global_permission_assignment,
        update_project_permission_assignment,
        // Project Data Access endpoints
        get_project_data_access_matrix,
        create_project_data,
        update_data_access,
        batch_update_data_access,
        request_data_access,
        get_access_by_status,
        get_user_access_list,
        // User Registration endpoints (TODO: Add OpenAPI annotations)
        // signup,
        // verify_email,
        // approve_user,
        // delete_account,
        // get_user_status,
    ),
    components(
        schemas(
            // Auth DTOs
            LoginRequest,
            LoginResponse,
            RefreshTokenRequest,
            RefreshTokenResponse,
            VerifyTokenResponse,
            // User DTOs
            CreateUserRequest,
            UpdateUserRequest,
            UserResponse,
            // Project DTOs
            CreateProjectRequest,
            UpdateProjectRequest,
            ProjectResponse,
            // Annotation DTOs
            CreateAnnotationRequest,
            UpdateAnnotationRequest,
            AnnotationResponse,
            AnnotationListResponse,
            // Mask Group DTOs
            CreateMaskGroupRequest,
            UpdateMaskGroupRequest,
            MaskGroupResponse,
            MaskGroupListResponse,
            MaskGroupDetailResponse,
            SignedUrlRequest,
            SignedUrlResponse,
            CompleteUploadRequest,
            CompleteUploadResponse,
            // Permission DTOs
            RoleWithPermissionsResponse,
            RolesWithPermissionsListResponse,
            PaginationQuery,
            // Project User DTOs
            UserWithRoleResponse,
            ProjectWithRoleResponse,
            AssignRoleRequest,
            BatchAssignRolesRequest,
            UserRoleAssignment,
            RoleAssignmentResponse,
            BatchRoleAssignmentResponse,
            FailedAssignment,
            // Project User Matrix DTOs
            UserRoleCell,
            ProjectUserMatrixRow,
            ProjectUserMatrixResponse,
            crate::application::dto::project_user_matrix_dto::UserInfo,
            MatrixPagination,
            MatrixQueryParams,
            // User Project Matrix DTOs
            ProjectRoleCell,
            UserProjectMatrixRow,
            UserProjectMatrixResponse,
            ProjectInfo,
            UserProjectMatrixPagination,
            UserProjectMatrixQueryParams,
            // Role Permission Matrix DTOs
            crate::application::dto::role_permission_matrix_dto::RoleInfo,
            crate::application::dto::role_permission_matrix_dto::PermissionInfo,
            RolePermissionAssignment,
            RolePermissionMatrixResponse,
            crate::application::dto::role_permission_matrix_dto::AssignPermissionRequest,
            AssignPermissionResponse,
            // Project Data Access DTOs
            ProjectDataInfo,
            DataAccessInfo,
            crate::application::dto::project_dto::PaginationInfo,
            ProjectDataAccessMatrixResponse,
            CreateProjectDataRequest,
            CreateProjectDataResponse,
            UpdateDataAccessRequest,
            UpdateDataAccessResponse,
            BatchUpdateDataAccessRequest,
            BatchUpdateDataAccessResponse,
            RequestDataAccessResponse,
            GetProjectDataListRequest,
            ProjectDataListResponse,
            // User Registration DTOs
            SignupRequest,
            VerifyEmailRequest,
            ApproveUserRequest,
            UserStatusResponse,
        )
    ),
    tags(
        (name = "auth", description = "Authentication endpoints - 인증 관련 API"),
        (name = "users", description = "User management endpoints - 사용자 관리 API"),
        (name = "projects", description = "Project management endpoints - 프로젝트 관리 API"),
        (name = "roles", description = "Role management endpoints - 역할 관리 API"),
        (name = "permissions", description = "Permission management endpoints - 권한 관리 API"),
        (name = "access-control", description = "Access control endpoints - 접근 제어 API"),
        (name = "annotations", description = "Annotation management endpoints - 어노테이션 관리 API"),
        (name = "mask-groups", description = "Mask Group management endpoints - 마스크 그룹 관리 API"),
        (name = "project-users", description = "Project User Role management endpoints - 프로젝트 사용자 역할 관리 API"),
        (name = "project-user-matrix", description = "Project User Matrix endpoints - 프로젝트 사용자 매트릭스 API"),
        (name = "user-project-matrix", description = "User Project Matrix endpoints - 유저 프로젝트 매트릭스 API"),
        (name = "role-permission-matrix", description = "Role Permission Matrix endpoints - 역할 권한 매트릭스 API"),
        (name = "project-data-access", description = "Project Data Access endpoints - 프로젝트 데이터 접근 관리 API"),
        (name = "user-registration", description = "User Registration endpoints - 사용자 등록 및 계정 관리 API"),
    ),
    info(
        title = "PACS Extension Server API",
        version = "0.1.0",
        description = r#"
# PACS Extension Server API Documentation

이 API는 PACS (Picture Archiving and Communication System) 확장 서버의 RESTful API입니다.

## 주요 기능
- 🔐 **Authentication**: JWT 기반 사용자 인증
- 👥 **User Management**: 사용자 관리 및 프로젝트 멤버 관리
- 📁 **Project Management**: 프로젝트 생성, 조회, 관리
- 🔑 **Permission Management**: 역할 기반 권한 관리 (RBAC)
- 🛡️ **Access Control**: 세밀한 접근 제어 및 로깅
- 📝 **Annotations**: DICOM 이미지 어노테이션 관리

## 인증 방법
대부분의 API는 JWT 토큰이 필요합니다.
`Authorization: Bearer <token>` 헤더를 포함하세요.
        "#,
        contact(
            name = "API Support",
            email = "support@pacs-server.com"
        ),
        license(
            name = "MIT",
        )
    ),
    servers(
        (url = "http://localhost:8080", description = "Local development server"),
        (url = "http://0.0.0.0:8080", description = "Local server (all interfaces)"),
        (url = "https://api.pacs-server.com", description = "Production server"),
    )
)]
pub struct ApiDoc;
