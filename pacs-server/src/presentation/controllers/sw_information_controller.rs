//! SW Information API 컨트롤러
//!
//! 의료영상저장장치 소프트웨어 정보 조회 API

use actix_web::{web, HttpResponse, Result};
use serde_json::json;
use std::sync::Arc;

use crate::application::dto::sw_information_dto::SwInformationListResponse;
use crate::application::use_cases::SwInformationUseCase;
use crate::domain::sw_information::SwInformationRepository;
use crate::domain::ServiceError;

fn handle_service_error(error: ServiceError) -> HttpResponse {
    match error {
        ServiceError::NotFound(msg) => HttpResponse::NotFound().json(json!({
            "error": "Not Found",
            "message": msg
        })),
        _ => HttpResponse::InternalServerError().json(json!({
            "error": "Internal Server Error",
            "message": "An unexpected error occurred"
        })),
    }
}

/// SW Information 목록 조회
#[utoipa::path(
    get,
    path = "/api/sw-information",
    responses(
        (status = 200, description = "목록 조회 성공", body = SwInformationListResponse),
        (status = 500, description = "서버 내부 오류")
    ),
    tag = "sw-information"
)]
pub async fn list_sw_information<R>(
    use_case: web::Data<Arc<SwInformationUseCase<R>>>,
) -> Result<HttpResponse, actix_web::Error>
where
    R: SwInformationRepository + 'static,
{
    match use_case.list().await {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// SW Information 상세 조회
#[utoipa::path(
    get,
    path = "/api/sw-information/{id}",
    responses(
        (status = 200, description = "상세 조회 성공"),
        (status = 404, description = "Not Found"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(("id" = i32, Path, description = "SW Information ID")),
    tag = "sw-information"
)]
pub async fn get_sw_information<R>(
    path: web::Path<i32>,
    use_case: web::Data<Arc<SwInformationUseCase<R>>>,
) -> Result<HttpResponse, actix_web::Error>
where
    R: SwInformationRepository + 'static,
{
    let id = path.into_inner();
    match use_case.get_by_id(id).await {
        Ok(Some(response)) => Ok(HttpResponse::Ok().json(response)),
        Ok(None) => Ok(HttpResponse::NotFound().json(json!({
            "error": "Not Found",
            "message": "SW Information not found"
        }))),
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// 라우팅 설정
pub fn configure_routes<R>(cfg: &mut actix_web::web::ServiceConfig, use_case: std::sync::Arc<SwInformationUseCase<R>>)
where
    R: SwInformationRepository + 'static,
{
    cfg.app_data(web::Data::new(use_case)).service(
        web::scope("/sw-information")
            .route("", web::get().to(list_sw_information::<R>))
            .route("/{id}", web::get().to(get_sw_information::<R>)),
    );
}
