use utoipa::OpenApi;
use crate::presentation::controllers::auth_controller_docs::*;
use crate::presentation::controllers::annotation_controller::*;
use crate::presentation::controllers::project_controller::*;
use crate::application::dto::auth_dto::*;
use crate::application::dto::user_dto::*;
use crate::application::dto::project_dto::*;
use crate::application::dto::annotation_dto::*;

#[derive(OpenApi)]
#[openapi(
    paths(
        // Auth endpoints
        login_doc,
        verify_token_doc,
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
        )
    ),
    tags(
        (name = "auth", description = "Authentication endpoints - 인증 관련 API"),
        (name = "users", description = "User management endpoints - 사용자 관리 API"),
        (name = "projects", description = "Project management endpoints - 프로젝트 관리 API"),
        (name = "permissions", description = "Permission management endpoints - 권한 관리 API"),
        (name = "access-control", description = "Access control endpoints - 접근 제어 API"),
        (name = "annotations", description = "Annotation management endpoints - 어노테이션 관리 API"),
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
    )
)]
pub struct ApiDoc;
