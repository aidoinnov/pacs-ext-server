use actix_web::{web, HttpResponse, Responder};
use serde_json::json;
use std::sync::Arc;

use crate::application::dto::auth_dto::{LoginRequest, LoginResponse};
use crate::application::use_cases::auth_use_case::AuthUseCase;
use crate::domain::services::auth_service::AuthService;

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
}

pub fn configure_routes<A: AuthService + 'static>(
    cfg: &mut web::ServiceConfig,
    auth_use_case: Arc<AuthUseCase<A>>,
) {
    cfg.app_data(web::Data::new(auth_use_case))
        .service(
            web::scope("/auth")
                .route("/login", web::post().to(AuthController::<A>::login))
                .route(
                    "/verify/{token}",
                    web::get().to(AuthController::<A>::verify_token),
                ),
        );
}
