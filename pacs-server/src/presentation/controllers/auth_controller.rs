use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use crate::application::dto::auth_dto::{LoginRequest, RefreshTokenRequest};
use crate::application::dto::user_registration_dto::*;
use crate::application::use_cases::auth_use_case::AuthUseCase;
use crate::application::use_cases::user_registration_use_case::UserRegistrationUseCase;
use crate::domain::services::auth_service::AuthService;
use crate::domain::repositories::UserRepository;
use crate::infrastructure::services::UserRegistrationServiceImpl;
use crate::infrastructure::auth::claims::Claims;
use crate::infrastructure::auth::jwt_service::JwtService;

#[derive(Debug, Deserialize, Serialize)]
pub struct TestTokenRequest {
    pub keycloak_id: Uuid,
    pub username: String,
}

#[derive(Debug, Serialize)]
pub struct TestTokenResponse {
    pub token: String,
    pub user_id: i32,
    pub username: String,
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct KeycloakTokenRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct KeycloakTokenResponse {
    pub access_token: String,
    pub expires_in: i32,
    pub refresh_expires_in: i32,
    pub refresh_token: String,
    pub token_type: String,
}

pub struct AuthController<A: AuthService> {
    auth_use_case: Arc<AuthUseCase<A>>,
}

impl<A: AuthService> AuthController<A> {
    pub fn new(auth_use_case: Arc<AuthUseCase<A>>) -> Self {
        Self { auth_use_case }
    }

    pub async fn login(
        auth_use_case: web::Data<Arc<AuthUseCase<A>>>,
        req: web::Json<LoginRequest>,
    ) -> impl Responder {
        match auth_use_case.login(req.into_inner()).await {
            Ok(response) => HttpResponse::Ok().json(response),
            Err(e) => HttpResponse::Unauthorized().json(json!({
                "error": format!("Login failed: {}", e)
            })),
        }
    }

    pub async fn verify_token(
        auth_use_case: web::Data<Arc<AuthUseCase<A>>>,
        token: web::Path<String>,
    ) -> impl Responder {
        match auth_use_case.verify_token(&token).await {
            Ok(response) => HttpResponse::Ok().json(response),
            Err(e) => HttpResponse::Unauthorized().json(json!({
                "valid": false,
                "error": format!("{}", e)
            })),
        }
    }

    pub async fn signup(
        user_registration_use_case: web::Data<
            Arc<UserRegistrationUseCase<UserRegistrationServiceImpl>>,
        >,
        req: web::Json<SignupRequest>,
    ) -> impl Responder {
        match user_registration_use_case.signup(req.into_inner()).await {
            Ok(response) => HttpResponse::Created().json(response),
            Err(e) => HttpResponse::BadRequest().json(json!({
                "error": format!("Signup failed: {}", e)
            })),
        }
    }

    pub async fn verify_email(
        user_registration_use_case: web::Data<
            Arc<UserRegistrationUseCase<UserRegistrationServiceImpl>>,
        >,
        req: web::Json<VerifyEmailRequest>,
    ) -> impl Responder {
        let user_id = req.user_id;
        match user_registration_use_case.verify_email(user_id).await {
            Ok(response) => HttpResponse::Ok().json(response),
            Err(e) => HttpResponse::BadRequest().json(json!({
                "error": format!("Email verification failed: {}", e)
            })),
        }
    }

    pub async fn approve_user(
        user_registration_use_case: web::Data<
            Arc<UserRegistrationUseCase<UserRegistrationServiceImpl>>,
        >,
        req: web::Json<ApproveUserRequest>,
    ) -> impl Responder {
        let user_id = req.user_id;
        let admin_id = 1; // TODO: 실제 관리자 ID로 교체 필요
        match user_registration_use_case
            .approve_user(user_id, admin_id)
            .await
        {
            Ok(response) => HttpResponse::Ok().json(response),
            Err(e) => HttpResponse::BadRequest().json(json!({
                "error": format!("User approval failed: {}", e)
            })),
        }
    }

    pub async fn delete_account(
        user_registration_use_case: web::Data<
            Arc<UserRegistrationUseCase<UserRegistrationServiceImpl>>,
        >,
        path: web::Path<i32>,
    ) -> impl Responder {
        let user_id = path.into_inner();
        match user_registration_use_case
            .delete_account(user_id, None)
            .await
        {
            Ok(response) => HttpResponse::Ok().json(response),
            Err(e) => HttpResponse::BadRequest().json(json!({
                "error": format!("Account deletion failed: {}", e)
            })),
        }
    }

    pub async fn refresh_token(
        auth_use_case: web::Data<Arc<AuthUseCase<A>>>,
        req: web::Json<RefreshTokenRequest>,
    ) -> impl Responder {
        match auth_use_case.refresh_token(req.into_inner()).await {
            Ok(response) => HttpResponse::Ok().json(response),
            Err(e) => HttpResponse::Unauthorized().json(json!({
                "error": format!("Token refresh failed: {}", e)
            })),
        }
    }

    pub async fn find_username(
        auth_use_case: web::Data<Arc<AuthUseCase<A>>>,
        req: web::Json<crate::application::dto::auth_dto::FindUsernameRequest>,
    ) -> impl Responder {
        match auth_use_case.find_username(&req.email).await {
            Ok(response) => HttpResponse::Ok().json(response),
            Err(e) => HttpResponse::BadRequest().json(json!({
                "error": format!("아이디 찾기 실패: {}", e)
            })),
        }
    }

    pub async fn reset_password(
        auth_use_case: web::Data<Arc<AuthUseCase<A>>>,
        req: web::Json<crate::application::dto::auth_dto::ResetPasswordRequest>,
    ) -> impl Responder {
        match auth_use_case
            .reset_password(&req.username, &req.email, &req.new_password)
            .await
        {
            Ok(response) => HttpResponse::Ok().json(response),
            Err(e) => HttpResponse::BadRequest().json(json!({
                "error": format!("비밀번호 재설정 실패: {}", e)
            })),
        }
    }

    /// 테스트 토큰 생성 (개발 환경 전용)
    ///
    /// 테스트 계정의 keycloak_id로 JWT 토큰을 생성합니다.
    /// 프로덕션 환경에서는 비활성화되어야 합니다.
    pub async fn create_test_token<U: UserRepository + 'static>(
        jwt_service: web::Data<Arc<JwtService>>,
        user_repository: web::Data<Arc<U>>,
        req: web::Json<TestTokenRequest>,
    ) -> impl Responder {
        // 테스트 계정 UUID 목록 (a0000000-0000-0000-0000-00000000000X)
        let test_account_uuids = vec![
            Uuid::parse_str("a0000000-0000-0000-0000-000000000001").unwrap(), // test_super_admin
            Uuid::parse_str("a0000000-0000-0000-0000-000000000002").unwrap(), // test_admin
            Uuid::parse_str("a0000000-0000-0000-0000-000000000003").unwrap(), // test_user
        ];

        // 요청된 keycloak_id가 테스트 계정인지 확인
        if !test_account_uuids.contains(&req.keycloak_id) {
            return HttpResponse::Forbidden().json(json!({
                "error": "Only test accounts are allowed"
            }));
        }

        // 사용자 조회
        match user_repository.find_by_keycloak_id(req.keycloak_id).await {
            Ok(Some(user)) => {
                // JWT 토큰 생성
                let claims = Claims::new(
                    user.id,
                    user.keycloak_id,
                    user.username.clone(),
                    user.email.clone(),
                    24, // 24시간 유효
                );

                match jwt_service.create_token(&claims) {
                    Ok(token) => HttpResponse::Ok().json(TestTokenResponse {
                        token,
                        user_id: user.id,
                        username: user.username,
                        email: user.email,
                    }),
                    Err(e) => HttpResponse::InternalServerError().json(json!({
                        "error": format!("Token creation failed: {}", e)
                    })),
                }
            }
            Ok(None) => HttpResponse::NotFound().json(json!({
                "error": "User not found"
            })),
            Err(e) => HttpResponse::InternalServerError().json(json!({
                "error": format!("Database error: {}", e)
            })),
        }
    }

    /// Keycloak 토큰 획득 프록시 (CORS 우회용)
    pub async fn get_keycloak_token(
        req: web::Json<KeycloakTokenRequest>,
    ) -> impl Responder {
        let keycloak_url = "https://keycloak.pacs.ai-do.kr/realms/dcm4che/protocol/openid-connect/token";

        let client = reqwest::Client::new();
        let params = [
            ("grant_type", "password"),
            ("client_id", "pacs-extension-server"),
            ("client_secret", "85TSWxK8ruF750z0Qzh0tQZ8xH5h3y99"),
            ("username", req.username.as_str()),
            ("password", req.password.as_str()),
        ];

        match client.post(keycloak_url)
            .form(&params)
            .send()
            .await
        {
            Ok(response) => {
                let status = response.status();
                match response.json::<serde_json::Value>().await {
                    Ok(data) => {
                        if status.is_success() {
                            HttpResponse::Ok().json(data)
                        } else {
                            HttpResponse::build(status).json(data)
                        }
                    }
                    Err(e) => HttpResponse::InternalServerError().json(json!({
                        "error": format!("Failed to parse Keycloak response: {}", e)
                    })),
                }
            }
            Err(e) => HttpResponse::BadGateway().json(json!({
                "error": format!("Failed to connect to Keycloak: {}", e)
            })),
        }
    }
}

pub fn configure_routes<A: AuthService + 'static, U: UserRepository + 'static>(
    cfg: &mut web::ServiceConfig,
    auth_use_case: Arc<AuthUseCase<A>>,
    user_registration_use_case: Arc<UserRegistrationUseCase<UserRegistrationServiceImpl>>,
    jwt_service: Arc<JwtService>,
    user_repository: Arc<U>,
) {
    cfg.app_data(web::Data::new(auth_use_case))
        .app_data(web::Data::new(user_registration_use_case))
        .app_data(web::Data::new(jwt_service))
        .app_data(web::Data::new(user_repository))
        .service(
            web::scope("/auth")
                .route("/login", web::post().to(AuthController::<A>::login))
                .route(
                    "/verify/{token}",
                    web::get().to(AuthController::<A>::verify_token),
                )
                .route(
                    "/refresh",
                    web::post().to(AuthController::<A>::refresh_token),
                )
                .route("/signup", web::post().to(AuthController::<A>::signup))
                .route(
                    "/verify-email",
                    web::post().to(AuthController::<A>::verify_email),
                )
                .route(
                    "/find-username",
                    web::post().to(AuthController::<A>::find_username),
                )
                .route(
                    "/reset-password",
                    web::post().to(AuthController::<A>::reset_password),
                )
                .route(
                    "/test-token",
                    web::post().to(AuthController::<A>::create_test_token::<U>),
                )
                .route(
                    "/keycloak-token",
                    web::post().to(AuthController::<A>::get_keycloak_token),
                )
                .route(
                    "/admin/users/approve",
                    web::post().to(AuthController::<A>::approve_user),
                ),
        )
        // Add user registration routes separately
        .route(
            "/users/{user_id}",
            web::delete().to(AuthController::<A>::delete_account),
        );
}
