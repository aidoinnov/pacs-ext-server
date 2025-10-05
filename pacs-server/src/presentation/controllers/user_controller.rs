use actix_web::{web, HttpResponse, Responder};
use serde_json::json;
use std::sync::Arc;

use crate::application::dto::user_dto::{CreateUserRequest, UpdateUserRequest, UserResponse};
use crate::application::use_cases::user_use_case::UserUseCase;
use crate::domain::services::user_service::UserService;

pub struct UserController<U: UserService> {
    user_use_case: Arc<UserUseCase<U>>,
}

impl<U: UserService> UserController<U> {
    pub fn new(user_use_case: Arc<UserUseCase<U>>) -> Self {
        Self { user_use_case }
    }

    pub async fn create_user(
        user_use_case: web::Data<Arc<UserUseCase<U>>>,
        req: web::Json<CreateUserRequest>,
    ) -> impl Responder {
        match user_use_case.create_user(req.into_inner()).await {
            Ok(user) => HttpResponse::Created().json(user),
            Err(e) => HttpResponse::BadRequest().json(json!({
                "error": format!("Failed to create user: {}", e)
            })),
        }
    }

    pub async fn get_user(
        user_use_case: web::Data<Arc<UserUseCase<U>>>,
        user_id: web::Path<i32>,
    ) -> impl Responder {
        match user_use_case.get_user_by_id(*user_id).await {
            Ok(user) => HttpResponse::Ok().json(user),
            Err(e) => HttpResponse::NotFound().json(json!({
                "error": format!("User not found: {}", e)
            })),
        }
    }

    pub async fn get_user_by_username(
        user_use_case: web::Data<Arc<UserUseCase<U>>>,
        username: web::Path<String>,
    ) -> impl Responder {
        match user_use_case.get_user_by_username(&username).await {
            Ok(user) => HttpResponse::Ok().json(user),
            Err(e) => HttpResponse::NotFound().json(json!({
                "error": format!("User not found: {}", e)
            })),
        }
    }

}


pub fn configure_routes<U: UserService + 'static>(
    cfg: &mut web::ServiceConfig,
    user_use_case: Arc<UserUseCase<U>>,
) {
    cfg.app_data(web::Data::new(user_use_case))
        .service(
            web::scope("/users")
                .route("", web::post().to(UserController::<U>::create_user))
                .route("/{user_id}", web::get().to(UserController::<U>::get_user))
                .route(
                    "/username/{username}",
                    web::get().to(UserController::<U>::get_user_by_username),
                ),
        );
}
