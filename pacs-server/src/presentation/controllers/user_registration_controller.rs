#![allow(dead_code, unused_imports, unused_variables)]
use actix_web::{web, HttpResponse};
use std::sync::Arc;
use crate::application::dto::user_registration_dto::*;
use crate::application::use_cases::UserRegistrationUseCase;
use crate::domain::ServiceError;
use crate::domain::services::UserRegistrationService;

/// 회원가입 API 엔드포인트
/// 
/// 새로운 사용자를 등록합니다. Keycloak과 데이터베이스에 원자적으로 사용자를 생성합니다.
/// 
/// # Arguments
/// * `req` - 회원가입 요청 데이터
/// * `use_case` - 사용자 등록 유스케이스
/// 
/// # Returns
/// * `Ok(HttpResponse)` - 회원가입 성공 (201 Created)
/// * `Err(ServiceError)` - 실패 시 에러
#[utoipa::path(
    post,
    path = "/api/auth/signup",
    request_body = SignupRequest,
    responses(
        (status = 201, description = "회원가입 성공", body = SignupResponse),
        (status = 400, description = "잘못된 요청 데이터"),
        (status = 409, description = "이미 존재하는 사용자명 또는 이메일"),
        (status = 500, description = "서버 내부 오류")
    ),
    tag = "auth"
)]
pub async fn signup<S: UserRegistrationService>(
    req: web::Json<SignupRequest>,
    use_case: web::Data<Arc<UserRegistrationUseCase<S>>>,
) -> Result<HttpResponse, ServiceError> {
    let response = use_case.signup(req.into_inner()).await?;
    Ok(HttpResponse::Created().json(response))
}

/// 이메일 인증 완료 처리 API 엔드포인트
/// 
/// 사용자가 이메일 인증을 완료했을 때 호출됩니다.
/// 계정 상태를 PENDING_EMAIL에서 PENDING_APPROVAL로 변경합니다.
/// 
/// # Arguments
/// * `req` - 이메일 인증 요청 데이터
/// * `use_case` - 사용자 등록 유스케이스
/// 
/// # Returns
/// * `Ok(HttpResponse)` - 이메일 인증 성공 (200 OK)
/// * `Err(ServiceError)` - 실패 시 에러
#[utoipa::path(
    post,
    path = "/api/auth/verify-email",
    request_body = VerifyEmailRequest,
    responses(
        (status = 200, description = "이메일 인증 성공", body = VerifyEmailResponse),
        (status = 400, description = "잘못된 요청"),
        (status = 404, description = "사용자를 찾을 수 없음"),
        (status = 500, description = "서버 내부 오류")
    ),
    tag = "auth"
)]
pub async fn verify_email<S: UserRegistrationService>(
    req: web::Json<VerifyEmailRequest>,
    use_case: web::Data<Arc<UserRegistrationUseCase<S>>>,
) -> Result<HttpResponse, ServiceError> {
    let response = use_case.verify_email(req.user_id).await?;
    Ok(HttpResponse::Ok().json(response))
}

/// 사용자 승인 API 엔드포인트 (관리자 전용)
/// 
/// 관리자가 사용자를 승인할 때 호출됩니다.
/// Keycloak에서 사용자를 활성화하고 계정 상태를 ACTIVE로 변경합니다.
/// 
/// # Arguments
/// * `req` - 사용자 승인 요청 데이터
/// * `use_case` - 사용자 등록 유스케이스
/// 
/// # Returns
/// * `Ok(HttpResponse)` - 승인 성공 (200 OK)
/// * `Err(ServiceError)` - 실패 시 에러
#[utoipa::path(
    post,
    path = "/api/admin/users/approve",
    request_body = ApproveUserRequest,
    responses(
        (status = 200, description = "승인 성공", body = ApproveUserResponse),
        (status = 403, description = "권한 없음"),
        (status = 404, description = "사용자를 찾을 수 없음"),
        (status = 500, description = "서버 내부 오류")
    ),
    tag = "admin",
    security(("bearer_auth" = []))
)]
pub async fn approve_user<S: UserRegistrationService>(
    req: web::Json<ApproveUserRequest>,
    use_case: web::Data<Arc<UserRegistrationUseCase<S>>>,
    // TODO: Extract admin_id from JWT token in middleware
) -> Result<HttpResponse, ServiceError> {
    let admin_id = 1; // TODO: Get from auth middleware
    let response = use_case.approve_user(req.user_id, admin_id).await?;
    Ok(HttpResponse::Ok().json(response))
}

/// 계정 삭제 API 엔드포인트
/// 
/// 사용자 계정을 삭제합니다. Keycloak과 데이터베이스에서 원자적으로 삭제합니다.
/// 감사 로그는 별도 보관됩니다.
/// 
/// # Arguments
/// * `path` - 경로에서 추출한 사용자 ID
/// * `use_case` - 사용자 등록 유스케이스
/// 
/// # Returns
/// * `Ok(HttpResponse)` - 삭제 성공 (200 OK)
/// * `Err(ServiceError)` - 실패 시 에러
#[utoipa::path(
    delete,
    path = "/api/users/{user_id}",
    params(
        ("user_id" = i32, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "삭제 성공", body = DeleteAccountResponse),
        (status = 403, description = "권한 없음"),
        (status = 404, description = "사용자를 찾을 수 없음"),
        (status = 500, description = "서버 내부 오류")
    ),
    tag = "users",
    security(("bearer_auth" = []))
)]
pub async fn delete_account<S: UserRegistrationService>(
    path: web::Path<i32>,
    use_case: web::Data<Arc<UserRegistrationUseCase<S>>>,
    // TODO: Extract actor_id from JWT token in middleware
) -> Result<HttpResponse, ServiceError> {
    let user_id = path.into_inner();
    let actor_id = Some(1); // TODO: Get from auth middleware
    
    let response = use_case.delete_account(user_id, actor_id).await?;
    Ok(HttpResponse::Ok().json(response))
}

/// 사용자 상태 조회 API 엔드포인트
/// 
/// 사용자의 현재 계정 상태를 조회합니다.
/// 
/// # Arguments
/// * `path` - 경로에서 추출한 사용자 ID
/// * `use_case` - 사용자 등록 유스케이스
/// 
/// # Returns
/// * `Ok(HttpResponse)` - 조회 성공 (200 OK)
/// * `Err(ServiceError)` - 실패 시 에러
#[utoipa::path(
    get,
    path = "/api/users/{user_id}/status",
    params(
        ("user_id" = i32, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "조회 성공", body = UserStatusResponse),
        (status = 404, description = "사용자를 찾을 수 없음"),
        (status = 500, description = "서버 내부 오류")
    ),
    tag = "users"
)]
pub async fn get_user_status<S: UserRegistrationService>(
    path: web::Path<i32>,
    use_case: web::Data<Arc<UserRegistrationUseCase<S>>>,
) -> Result<HttpResponse, ServiceError> {
    let user_id = path.into_inner();
    
    // TODO: Implement user status retrieval
    // For now, return a placeholder response
    let response = UserStatusResponse {
        user_id,
        username: "placeholder".to_string(),
        email: "placeholder@example.com".to_string(),
        account_status: "UNKNOWN".to_string(),
        email_verified: false,
        is_approved: false,
        approved_by: None,
        approved_at: None,
    };
    
    Ok(HttpResponse::Ok().json(response))
}

/// 라우팅 설정
/// 
/// 사용자 등록 관련 API 엔드포인트들을 설정합니다.
/// 
/// # Arguments
/// * `cfg` - Actix-web 서비스 설정
pub fn configure_routes<S: UserRegistrationService + 'static>(cfg: &mut web::ServiceConfig, use_case: Arc<UserRegistrationUseCase<S>>) {
    cfg
        .app_data(web::Data::new(use_case))
        .route("/auth/signup", web::post().to(signup::<S>))
        .route("/auth/verify-email", web::post().to(verify_email::<S>))
        .route("/admin/users/approve", web::post().to(approve_user::<S>))
        .route("/users/{user_id}", web::delete().to(delete_account::<S>))
        .route("/users/{user_id}/status", web::get().to(get_user_status::<S>));
}
