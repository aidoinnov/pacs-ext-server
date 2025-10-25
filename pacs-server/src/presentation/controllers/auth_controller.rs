use actix_web::{web, HttpResponse, Responder};
use serde_json::json;
use std::sync::Arc;

use crate::application::dto::auth_dto::{LoginRequest, LoginResponse, RefreshTokenRequest, RefreshTokenResponse};
use crate::application::dto::user_registration_dto::*;
use crate::application::use_cases::auth_use_case::AuthUseCase;
use crate::application::use_cases::user_registration_use_case::UserRegistrationUseCase;
use crate::domain::services::auth_service::AuthService;
use crate::infrastructure::services::UserRegistrationServiceImpl;

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
        user_registration_use_case: web::Data<Arc<UserRegistrationUseCase<UserRegistrationServiceImpl>>>,
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
        user_registration_use_case: web::Data<Arc<UserRegistrationUseCase<UserRegistrationServiceImpl>>>,
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
        user_registration_use_case: web::Data<Arc<UserRegistrationUseCase<UserRegistrationServiceImpl>>>,
        req: web::Json<ApproveUserRequest>,
    ) -> impl Responder {
        let user_id = req.user_id;
        let admin_id = 1; // TODO: 실제 관리자 ID로 교체 필요
        match user_registration_use_case.approve_user(user_id, admin_id).await {
            Ok(response) => HttpResponse::Ok().json(response),
            Err(e) => HttpResponse::BadRequest().json(json!({
                "error": format!("User approval failed: {}", e)
            })),
        }
    }

    pub async fn delete_account(
        user_registration_use_case: web::Data<Arc<UserRegistrationUseCase<UserRegistrationServiceImpl>>>,
        path: web::Path<i32>,
    ) -> impl Responder {
        let user_id = path.into_inner();
        match user_registration_use_case.delete_account(user_id, None).await {
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

}

pub fn configure_routes<A: AuthService + 'static>(
    cfg: &mut web::ServiceConfig,
    auth_use_case: Arc<AuthUseCase<A>>,
    user_registration_use_case: Arc<UserRegistrationUseCase<UserRegistrationServiceImpl>>,
) {
    cfg.app_data(web::Data::new(auth_use_case))
        .app_data(web::Data::new(user_registration_use_case))
        .service(
            web::scope("/auth")
                .route("/login", web::post().to(AuthController::<A>::login))
                .route(
                    "/verify/{token}",
                    web::get().to(AuthController::<A>::verify_token),
                )
                .route("/refresh", web::post().to(AuthController::<A>::refresh_token))
                .route("/signup", web::post().to(AuthController::<A>::signup))
                .route("/verify-email", web::post().to(AuthController::<A>::verify_email))
                .route("/admin/users/approve", web::post().to(AuthController::<A>::approve_user))
                .route("/users/{user_id}", web::delete().to(AuthController::<A>::delete_account)),
        );
}
