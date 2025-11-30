use actix_web::{web, HttpResponse, Responder, HttpRequest};
use serde_json::json;
use std::sync::Arc;

use crate::application::dto::permission_dto::PaginationQuery;
use crate::application::dto::user_dto::{
    CreateUserRequest, MeQuery, PaginationInfo, UpdateUserRequest, UserListQuery,
    UserListResponse, UserProjectQuery, UserQuery, UserResponse,
};
use crate::application::use_cases::user_use_case::UserUseCase;
use crate::domain::services::user_service::UserService;
use crate::infrastructure::auth::{extract_user_id_from_request, JwtService};
use crate::infrastructure::repositories::UserRepositoryImpl;
use crate::domain::repositories::UserRepository; // bring trait for find_by_id into scope

pub struct UserController<U: UserService> {
    user_use_case: Arc<UserUseCase<U>>,
}

/// 내 프로필 조회 (토큰 기반 또는 쿼리 파라미터)
/// 
/// user_id 추출 우선순위:
/// 1. JWT 토큰에서 추출 (Authorization: Bearer ...)
/// 2. 쿼리 파라미터에서 추출 (?user_id=xxx)
#[utoipa::path(
    get,
    path = "/api/users/me",
    tag = "users",
    params(
        ("user_id" = Option<i32>, Query, description = "User ID (optional, used if JWT token is not available)"),
        ("project_id" = Option<i32>, Query, description = "Project ID (optional, returns role_name if provided)")
    ),
    responses(
        (status = 200, description = "Current user profile", body = UserResponse),
        (status = 401, description = "Unauthorized - user_id could not be determined"),
        (status = 404, description = "User not found")
    )
)]
pub async fn get_me<U: UserService + 'static>(
    req: HttpRequest,
    jwt: web::Data<Arc<JwtService>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
    user_use_case: web::Data<Arc<UserUseCase<U>>>,
    query: web::Query<MeQuery>,
) -> impl Responder {
    // 1순위: JWT 토큰에서 user_id 추출
    let user_id = match extract_user_id_from_request(&req, &jwt, &user_repo).await {
        Some(id) if id > 0 => Some(id),
        _ => None,
    };

    // 2순위: 쿼리 파라미터에서 user_id 추출
    let user_id = user_id.or_else(|| {
        query.user_id.filter(|&id| id > 0)
    });

    match user_id {
        Some(user_id) => {
            match user_use_case
                .get_user_by_id_with_project_role(user_id, query.project_id)
                .await
            {
                Ok(user) => HttpResponse::Ok().json(user),
                Err(e) => HttpResponse::NotFound().json(json!({
                    "error": format!("User not found: {}", e)
                })),
            }
        }
        None => HttpResponse::Unauthorized().json(json!({
            "error": "Unauthorized",
            "message": "User ID is required. Provide JWT token or user_id query parameter."
        })),
    }
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

        match user_use_case
            .list_users(page, page_size, sort_by, sort_order, search)
            .await
        {
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

/// 사용자 조회 (Path parameter)
#[utoipa::path(
    get,
    path = "/api/users/{user_id}",
    tag = "users",
    params(
        ("user_id" = i32, Path, description = "User ID"),
        ("project_id" = Option<i32>, Query, description = "Project ID (optional, returns role_name if provided)")
    ),
    responses(
        (status = 200, description = "User retrieved successfully", body = UserResponse),
        (status = 404, description = "User not found")
    )
)]
pub async fn get_user<U: UserService + 'static>(
    user_use_case: web::Data<Arc<UserUseCase<U>>>,
    user_id: web::Path<i32>,
    query: web::Query<UserProjectQuery>,
) -> impl Responder {
    match user_use_case
        .get_user_by_id_with_project_role(*user_id, query.project_id)
        .await
    {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(e) => HttpResponse::NotFound().json(json!({
            "error": format!("User not found: {}", e)
        })),
    }
}

/// 사용자 조회 (Query parameter)
#[utoipa::path(
    get,
    path = "/api/users/info",
    tag = "users",
    params(
        ("user_id" = i32, Query, description = "User ID (required)"),
        ("project_id" = Option<i32>, Query, description = "Project ID (optional, returns role_name if provided)")
    ),
    responses(
        (status = 200, description = "User retrieved successfully", body = UserResponse),
        (status = 400, description = "Bad Request - user_id is required"),
        (status = 404, description = "User not found")
    )
)]
pub async fn get_user_by_query<U: UserService + 'static>(
    user_use_case: web::Data<Arc<UserUseCase<U>>>,
    query: web::Query<UserQuery>,
) -> impl Responder {
    // user_id 필수 검증
    if query.user_id <= 0 {
        return HttpResponse::BadRequest().json(json!({
            "error": "Bad Request",
            "message": "user_id is required and must be greater than 0"
        }));
    }

    match user_use_case
        .get_user_by_id_with_project_role(query.user_id, query.project_id)
        .await
    {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(e) => HttpResponse::NotFound().json(json!({
            "error": format!("User not found: {}", e)
        })),
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

/// 사용자가 속한 프로젝트 목록 조회 (역할 정보 포함)
#[utoipa::path(
    get,
    path = "/api/users/{user_id}/projects",
    params(
        ("user_id" = i32, Path, description = "User ID"),
        ("page" = Option<i32>, Query, description = "Page number (default: 1)"),
        ("page_size" = Option<i32>, Query, description = "Page size (default: 20, max: 100)")
    ),
    responses(
        (status = 200, description = "User projects retrieved successfully"),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "users"
)]
pub async fn get_user_projects<U: UserService>(
    path: web::Path<i32>,
    query: web::Query<PaginationQuery>,
    user_service: web::Data<Arc<U>>,
) -> impl Responder {
    let user_id = path.into_inner();
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20).min(100);

    match user_service
        .get_user_projects_with_roles(user_id, page, page_size)
        .await
    {
        Ok((projects, total_count)) => {
            let total_pages = (total_count as f64 / page_size as f64).ceil() as i32;
            HttpResponse::Ok().json(json!({
                "projects": projects,
                "total_count": total_count,
                "page": page,
                "page_size": page_size,
                "total_pages": total_pages
            }))
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to get user projects: {}", e)
        })),
    }
}

pub fn configure_routes<U: UserService + 'static>(
    cfg: &mut web::ServiceConfig,
    user_use_case: Arc<UserUseCase<U>>,
    user_service: Arc<U>,
) {
    cfg.app_data(web::Data::new(user_use_case))
        .app_data(web::Data::new(user_service))
        .service(
            web::scope("/users")
                .route("", web::get().to(UserController::<U>::list_users))
                .route("", web::post().to(UserController::<U>::create_user))
                .route("/me", web::get().to(get_me::<U>))
                .route("/info", web::get().to(get_user_by_query::<U>))
                .route(
                    "/username/{username}",
                    web::get().to(UserController::<U>::get_user_by_username),
                )
                .route("/{user_id}/projects", web::get().to(get_user_projects::<U>))
                .route("/{user_id}", web::get().to(get_user::<U>))
                .route("/{user_id}", web::put().to(update_user::<U>)),
        );
}
