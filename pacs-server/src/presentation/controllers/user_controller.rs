use actix_web::{web, HttpResponse, Responder};
use serde_json::json;
use std::sync::Arc;

use crate::application::dto::user_dto::{CreateUserRequest, UpdateUserRequest, UserResponse, UserListQuery, UserListResponse, PaginationInfo};
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

    pub async fn list_users(
        user_use_case: web::Data<Arc<UserUseCase<U>>>,
        query: web::Query<UserListQuery>,
    ) -> impl Responder {
        let page = query.page.unwrap_or(1);
        let page_size = query.page_size.unwrap_or(20).min(100);
        let sort_by = query.sort_by.as_deref().unwrap_or("username");
        let sort_order = query.sort_order.as_deref().unwrap_or("asc");
        let search = query.search.as_deref();

        match user_use_case.list_users(page, page_size, sort_by, sort_order, search).await {
            Ok((users, total)) => {
                let total_pages = if total > 0 {
                    ((total as f64) / (page_size as f64)).ceil() as i32
                } else {
                    0
                };

                HttpResponse::Ok().json(UserListResponse {
                    users: users.into_iter().map(|u| u.into()).collect(),
                    pagination: PaginationInfo {
                        page,
                        page_size,
                        total: total as i32,
                        total_pages,
                    },
                })
            }
            Err(e) => HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to list users: {}", e)
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
                .route("", web::get().to(UserController::<U>::list_users))
                .route("", web::post().to(UserController::<U>::create_user))
                .route("/{user_id}", web::get().to(UserController::<U>::get_user))
                .route("/{user_id}", web::put().to(update_user::<U>))
                .route(
                    "/username/{username}",
                    web::get().to(UserController::<U>::get_user_by_username),
                ),
        );
}
