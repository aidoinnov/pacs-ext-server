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

/// 사용자 정보 업데이트
#[utoipa::path(
    put,
    path = "/api/users/{user_id}",
    tag = "users",
    params(
        ("user_id" = i32, Path, description = "User ID")
    ),
    request_body = UpdateUserRequest,
    responses(
        (status = 200, description = "User updated successfully", body = UserResponse),
        (status = 400, description = "Invalid request"),
        (status = 404, description = "User not found"),
        (status = 409, description = "Email already taken")
    )
)]
pub async fn update_user<U: UserService + 'static>(
    user_use_case: web::Data<Arc<UserUseCase<U>>>,
    path: web::Path<i32>,
    req: web::Json<UpdateUserRequest>,
) -> impl Responder {
    let user_id = path.into_inner();
    
    match user_use_case.update_user(user_id, req.into_inner()).await {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(e) => {
            let mut status = match e {
                crate::domain::ServiceError::NotFound(_) => HttpResponse::NotFound(),
                crate::domain::ServiceError::AlreadyExists(_) => HttpResponse::Conflict(),
                crate::domain::ServiceError::ValidationError(_) => HttpResponse::BadRequest(),
                _ => HttpResponse::InternalServerError(),
            };
            
            status.json(json!({
                "error": format!("Failed to update user: {}", e)
            }))
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
                .route("/{user_id}", web::put().to(update_user::<U>))
                .route(
                    "/username/{username}",
                    web::get().to(UserController::<U>::get_user_by_username),
                ),
        );
}
