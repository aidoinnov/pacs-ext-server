#![allow(dead_code, unused_imports, unused_variables)]
use actix_web::{web, Responder};
use std::sync::Arc;

use crate::application::dto::auth_dto::{LoginRequest, LoginResponse, RefreshTokenRequest, RefreshTokenResponse, VerifyTokenResponse};
use crate::application::use_cases::auth_use_case::AuthUseCase;
use crate::domain::services::AuthServiceImpl;
use crate::infrastructure::repositories::UserRepositoryImpl;

/// 사용자 로그인
///
/// Keycloak ID, username, email을 통해 로그인하고 JWT 토큰을 발급받습니다.
#[utoipa::path(
    post,
    path = "/api/auth/login",
    tag = "auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "로그인 성공", body = LoginResponse),
        (status = 401, description = "인증 실패"),
    )
)]
pub async fn login_doc(
    auth_use_case: web::Data<Arc<AuthUseCase<AuthServiceImpl<UserRepositoryImpl>>>>,
    req: web::Json<LoginRequest>,
) -> impl Responder {
    use crate::presentation::controllers::auth_controller::AuthController;
    AuthController::<AuthServiceImpl<UserRepositoryImpl>>::login(auth_use_case, req).await
}

/// 토큰 검증
///
/// JWT 토큰의 유효성을 검증하고 사용자 정보를 반환합니다.
#[utoipa::path(
    get,
    path = "/api/auth/verify/{token}",
    tag = "auth",
    params(
        ("token" = String, Path, description = "JWT 토큰")
    ),
    responses(
        (status = 200, description = "토큰 검증 성공", body = VerifyTokenResponse),
        (status = 401, description = "유효하지 않은 토큰"),
    )
)]
pub async fn verify_token_doc(
    auth_use_case: web::Data<Arc<AuthUseCase<AuthServiceImpl<UserRepositoryImpl>>>>,
    token: web::Path<String>,
) -> impl Responder {
    use crate::presentation::controllers::auth_controller::AuthController;
    AuthController::<AuthServiceImpl<UserRepositoryImpl>>::verify_token(auth_use_case, token).await
}

/// 토큰 갱신
///
/// Keycloak의 refresh token을 사용하여 새로운 access token을 발급받습니다.
#[utoipa::path(
    post,
    path = "/api/auth/refresh",
    tag = "auth",
    request_body = RefreshTokenRequest,
    responses(
        (status = 200, description = "토큰 갱신 성공", body = RefreshTokenResponse),
        (status = 401, description = "유효하지 않은 refresh token"),
    )
)]
pub async fn refresh_token_doc(
    auth_use_case: web::Data<Arc<AuthUseCase<AuthServiceImpl<UserRepositoryImpl>>>>,
    req: web::Json<RefreshTokenRequest>,
) -> impl Responder {
    use crate::presentation::controllers::auth_controller::AuthController;
    AuthController::<AuthServiceImpl<UserRepositoryImpl>>::refresh_token(auth_use_case, req).await
}
