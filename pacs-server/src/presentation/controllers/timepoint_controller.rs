use actix_web::{web, HttpResponse, Responder};
use serde_json::json;
use std::sync::Arc;

use crate::domain::entities::{
    AssignStudies, AssignmentResult, CreateTimePoint, StudyInfo, TimePoint, TimePointStudies,
    UnassignStudies, UpdateTimePoint,
};
use crate::domain::services::TimePointService;
use crate::domain::ServiceError;

/// TimePoint 생성
///
/// 새로운 TimePoint를 생성합니다.
#[utoipa::path(
    post,
    path = "/api/subjects/{subject_id}/timepoints",
    tag = "timepoints",
    params(
        ("subject_id" = i32, Path, description = "Subject ID")
    ),
    request_body = CreateTimePoint,
    responses(
        (status = 201, description = "TimePoint created successfully", body = TimePoint),
        (status = 400, description = "Invalid request or validation error"),
        (status = 404, description = "Subject not found"),
        (status = 409, description = "Baseline already exists or TimePoint name duplicate"),
    )
)]
pub async fn create_timepoint<T: TimePointService + 'static>(
    timepoint_service: web::Data<Arc<T>>,
    subject_id: web::Path<i32>,
    req: web::Json<CreateTimePoint>,
) -> impl Responder {
    let mut new_timepoint = req.into_inner();
    new_timepoint.subject_id = *subject_id;

    match timepoint_service.create_timepoint(new_timepoint).await {
        Ok(timepoint) => HttpResponse::Created().json(timepoint),
        Err(e) => {
            let mut status = match e {
                ServiceError::NotFound(_) => HttpResponse::NotFound(),
                ServiceError::AlreadyExists(_) => HttpResponse::Conflict(),
                ServiceError::ValidationError(_) => HttpResponse::BadRequest(),
                _ => HttpResponse::InternalServerError(),
            };
            status.json(json!({
                "error": format!("{}", e)
            }))
        }
    }
}

/// TimePoint 조회
///
/// TimePoint ID로 TimePoint를 조회합니다.
#[utoipa::path(
    get,
    path = "/api/timepoints/{id}",
    tag = "timepoints",
    params(
        ("id" = i32, Path, description = "TimePoint ID")
    ),
    responses(
        (status = 200, description = "TimePoint retrieved successfully", body = TimePoint),
        (status = 404, description = "TimePoint not found"),
    )
)]
pub async fn get_timepoint<T: TimePointService + 'static>(
    timepoint_service: web::Data<Arc<T>>,
    id: web::Path<i32>,
) -> impl Responder {
    match timepoint_service.get_timepoint(*id).await {
        Ok(timepoint) => HttpResponse::Ok().json(timepoint),
        Err(e) => HttpResponse::NotFound().json(json!({
            "error": format!("{}", e)
        })),
    }
}

/// Subject의 TimePoint 목록 조회
///
/// Subject ID로 모든 TimePoint를 조회합니다.
#[utoipa::path(
    get,
    path = "/api/subjects/{subject_id}/timepoints",
    tag = "timepoints",
    params(
        ("subject_id" = i32, Path, description = "Subject ID")
    ),
    responses(
        (status = 200, description = "TimePoints retrieved successfully", body = Vec<TimePoint>),
        (status = 404, description = "Subject not found"),
    )
)]
pub async fn get_timepoints_by_subject<T: TimePointService + 'static>(
    timepoint_service: web::Data<Arc<T>>,
    subject_id: web::Path<i32>,
) -> impl Responder {
    match timepoint_service
        .get_timepoints_by_subject(*subject_id)
        .await
    {
        Ok(timepoints) => HttpResponse::Ok().json(timepoints),
        Err(e) => {
            let mut status = match e {
                ServiceError::NotFound(_) => HttpResponse::NotFound(),
                _ => HttpResponse::InternalServerError(),
            };
            status.json(json!({
                "error": format!("{}", e)
            }))
        }
    }
}

/// TimePoint 수정
///
/// TimePoint ID로 TimePoint를 수정합니다.
#[utoipa::path(
    put,
    path = "/api/timepoints/{id}",
    tag = "timepoints",
    params(
        ("id" = i32, Path, description = "TimePoint ID")
    ),
    request_body = UpdateTimePoint,
    responses(
        (status = 200, description = "TimePoint updated successfully", body = TimePoint),
        (status = 400, description = "Invalid request or validation error"),
        (status = 404, description = "TimePoint not found"),
        (status = 409, description = "Baseline already exists or TimePoint name duplicate"),
    )
)]
pub async fn update_timepoint<T: TimePointService + 'static>(
    timepoint_service: web::Data<Arc<T>>,
    id: web::Path<i32>,
    req: web::Json<UpdateTimePoint>,
) -> impl Responder {
    match timepoint_service
        .update_timepoint(*id, req.into_inner())
        .await
    {
        Ok(timepoint) => HttpResponse::Ok().json(timepoint),
        Err(e) => {
            let mut status = match e {
                ServiceError::NotFound(_) => HttpResponse::NotFound(),
                ServiceError::AlreadyExists(_) => HttpResponse::Conflict(),
                ServiceError::ValidationError(_) => HttpResponse::BadRequest(),
                _ => HttpResponse::InternalServerError(),
            };
            status.json(json!({
                "error": format!("{}", e)
            }))
        }
    }
}

/// TimePoint 삭제
///
/// TimePoint ID로 TimePoint를 삭제합니다.
#[utoipa::path(
    delete,
    path = "/api/timepoints/{id}",
    tag = "timepoints",
    params(
        ("id" = i32, Path, description = "TimePoint ID")
    ),
    responses(
        (status = 204, description = "TimePoint deleted successfully"),
        (status = 400, description = "Cannot delete timepoint with assigned studies"),
        (status = 404, description = "TimePoint not found"),
    )
)]
pub async fn delete_timepoint<T: TimePointService + 'static>(
    timepoint_service: web::Data<Arc<T>>,
    id: web::Path<i32>,
) -> impl Responder {
    match timepoint_service.delete_timepoint(*id).await {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(e) => {
            let mut status = match e {
                ServiceError::NotFound(_) => HttpResponse::NotFound(),
                ServiceError::ValidationError(_) => HttpResponse::BadRequest(),
                _ => HttpResponse::InternalServerError(),
            };
            status.json(json!({
                "error": format!("{}", e)
            }))
        }
    }
}

/// TimePoint에 Study 할당
///
/// TimePoint ID로 Study를 할당합니다 (MOVE 시맨틱).
#[utoipa::path(
    post,
    path = "/api/timepoints/{id}/studies",
    tag = "timepoints",
    params(
        ("id" = i32, Path, description = "TimePoint ID")
    ),
    request_body = AssignStudies,
    responses(
        (status = 200, description = "Studies assigned successfully", body = AssignmentResult),
        (status = 404, description = "TimePoint not found"),
    )
)]
pub async fn assign_studies<T: TimePointService + 'static>(
    timepoint_service: web::Data<Arc<T>>,
    id: web::Path<i32>,
    req: web::Json<AssignStudies>,
) -> impl Responder {
    // TODO: Get user_id from authentication context
    let user_id = 1; // Temporary hardcoded value

    match timepoint_service
        .assign_studies(*id, req.into_inner(), user_id)
        .await
    {
        Ok(result) => HttpResponse::Ok()
            // 클라이언트에게 관련 캐시를 무효화하도록 힌트 제공
            .insert_header(("X-Cache-Invalidate", "dicom-studies"))
            .json(result),
        Err(e) => {
            let mut status = match e {
                ServiceError::NotFound(_) => HttpResponse::NotFound(),
                _ => HttpResponse::InternalServerError(),
            };
            status.json(json!({
                "error": format!("{}", e)
            }))
        }
    }
}

/// TimePoint에서 Study 해제
///
/// TimePoint ID로 Study를 해제합니다.
#[utoipa::path(
    delete,
    path = "/api/timepoints/{id}/studies",
    tag = "timepoints",
    params(
        ("id" = i32, Path, description = "TimePoint ID")
    ),
    request_body = UnassignStudies,
    responses(
        (status = 200, description = "Studies unassigned successfully"),
        (status = 404, description = "TimePoint not found"),
    )
)]
pub async fn unassign_studies<T: TimePointService + 'static>(
    timepoint_service: web::Data<Arc<T>>,
    id: web::Path<i32>,
    req: web::Json<UnassignStudies>,
) -> impl Responder {
    match timepoint_service
        .unassign_studies(*id, req.into_inner())
        .await
    {
        Ok(count) => HttpResponse::Ok()
            // 클라이언트에게 관련 캐시를 무효화하도록 힌트 제공
            .insert_header(("X-Cache-Invalidate", "dicom-studies"))
            .json(json!({
                "unassigned_count": count
            })),
        Err(e) => {
            let mut status = match e {
                ServiceError::NotFound(_) => HttpResponse::NotFound(),
                _ => HttpResponse::InternalServerError(),
            };
            status.json(json!({
                "error": format!("{}", e)
            }))
        }
    }
}

/// TimePoint의 Study 목록 조회
///
/// TimePoint ID로 할당된 Study 목록을 조회합니다.
#[utoipa::path(
    get,
    path = "/api/timepoints/{id}/studies",
    tag = "timepoints",
    params(
        ("id" = i32, Path, description = "TimePoint ID")
    ),
    responses(
        (status = 200, description = "Studies retrieved successfully", body = TimePointStudies),
        (status = 404, description = "TimePoint not found"),
    )
)]
pub async fn get_studies_by_timepoint<T: TimePointService + 'static>(
    timepoint_service: web::Data<Arc<T>>,
    id: web::Path<i32>,
) -> impl Responder {
    match timepoint_service.get_studies_by_timepoint(*id).await {
        Ok(studies) => HttpResponse::Ok().json(studies),
        Err(e) => {
            let mut status = match e {
                ServiceError::NotFound(_) => HttpResponse::NotFound(),
                _ => HttpResponse::InternalServerError(),
            };
            status.json(json!({
                "error": format!("{}", e)
            }))
        }
    }
}

/// Subject의 미할당 Study 목록 조회
///
/// Subject ID로 미할당 Study 목록을 조회합니다.
#[utoipa::path(
    get,
    path = "/api/subjects/{subject_id}/studies/unassigned",
    tag = "timepoints",
    params(
        ("subject_id" = i32, Path, description = "Subject ID")
    ),
    responses(
        (status = 200, description = "Unassigned studies retrieved successfully", body = Vec<StudyInfo>),
        (status = 404, description = "Subject not found"),
    )
)]
pub async fn get_unassigned_studies_by_subject<T: TimePointService + 'static>(
    timepoint_service: web::Data<Arc<T>>,
    subject_id: web::Path<i32>,
) -> impl Responder {
    match timepoint_service
        .get_unassigned_studies_by_subject(*subject_id)
        .await
    {
        Ok(studies) => HttpResponse::Ok().json(studies),
        Err(e) => {
            let mut status = match e {
                ServiceError::NotFound(_) => HttpResponse::NotFound(),
                _ => HttpResponse::InternalServerError(),
            };
            status.json(json!({
                "error": format!("{}", e)
            }))
        }
    }
}

/// 라우트 설정
pub fn configure_routes<T: TimePointService + 'static>(
    cfg: &mut web::ServiceConfig,
    timepoint_service: Arc<T>,
) {
    cfg.app_data(web::Data::new(timepoint_service))
        // Subject의 TimePoint 관련 엔드포인트
        .service(
            web::scope("/subjects/{subject_id}")
                .service(
                    web::resource("/timepoints")
                        .route(web::post().to(create_timepoint::<T>))
                        .route(web::get().to(get_timepoints_by_subject::<T>)),
                )
                .service(
                    web::resource("/studies/unassigned")
                        .route(web::get().to(get_unassigned_studies_by_subject::<T>)),
                ),
        )
        // TimePoint 직접 관련 엔드포인트
        .service(
            web::scope("/timepoints/{id}")
                .route("", web::get().to(get_timepoint::<T>))
                .route("", web::put().to(update_timepoint::<T>))
                .route("", web::delete().to(delete_timepoint::<T>))
                .service(
                    web::resource("/studies")
                        .route(web::post().to(assign_studies::<T>))
                        .route(web::delete().to(unassign_studies::<T>))
                        .route(web::get().to(get_studies_by_timepoint::<T>)),
                ),
        );
}


