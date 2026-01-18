use actix_web::{web, HttpResponse, Responder};
use serde_json::json;
use std::sync::Arc;

use crate::domain::entities::{CreateSubject, SubjectDetail, UpdateSubject};
use crate::domain::services::SubjectService;
use crate::domain::ServiceError;

/// Subject 생성
///
/// 새로운 Subject를 생성합니다.
#[utoipa::path(
    post,
    path = "/api/projects/{project_id}/subjects",
    tag = "subjects",
    params(
        ("project_id" = i32, Path, description = "Project ID")
    ),
    request_body = CreateSubject,
    responses(
        (status = 201, description = "Subject created successfully", body = Subject),
        (status = 400, description = "Invalid request or validation error"),
        (status = 404, description = "Project not found"),
        (status = 409, description = "Subject code or Patient ID already exists"),
    )
)]
pub async fn create_subject<S: SubjectService + 'static>(
    subject_service: web::Data<Arc<S>>,
    project_id: web::Path<i32>,
    req: web::Json<CreateSubject>,
) -> impl Responder {
    let mut new_subject = req.into_inner();
    new_subject.project_id = *project_id;

    match subject_service.create_subject(new_subject).await {
        Ok(subject) => HttpResponse::Created().json(subject),
        Err(e) => {
            let status = match e {
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

/// Subject 조회
///
/// Subject ID로 Subject를 조회합니다.
#[utoipa::path(
    get,
    path = "/api/subjects/{id}",
    tag = "subjects",
    params(
        ("id" = i32, Path, description = "Subject ID")
    ),
    responses(
        (status = 200, description = "Subject retrieved successfully", body = Subject),
        (status = 404, description = "Subject not found"),
    )
)]
pub async fn get_subject<S: SubjectService + 'static>(
    subject_service: web::Data<Arc<S>>,
    id: web::Path<i32>,
) -> impl Responder {
    match subject_service.get_subject(*id).await {
        Ok(subject) => HttpResponse::Ok().json(subject),
        Err(e) => HttpResponse::NotFound().json(json!({
            "error": format!("{}", e)
        })),
    }
}

/// Subject 상세 조회
///
/// Subject ID로 Subject 상세 정보를 조회합니다 (통계 포함).
#[utoipa::path(
    get,
    path = "/api/subjects/{id}/detail",
    tag = "subjects",
    params(
        ("id" = i32, Path, description = "Subject ID")
    ),
    responses(
        (status = 200, description = "Subject detail retrieved successfully", body = SubjectDetail),
        (status = 404, description = "Subject not found"),
    )
)]
pub async fn get_subject_detail<S: SubjectService + 'static>(
    subject_service: web::Data<Arc<S>>,
    id: web::Path<i32>,
) -> impl Responder {
    match subject_service.get_subject_detail(*id).await {
        Ok(detail) => HttpResponse::Ok().json(detail),
        Err(e) => HttpResponse::NotFound().json(json!({
            "error": format!("{}", e)
        })),
    }
}

/// 프로젝트의 Subject 목록 조회
///
/// 프로젝트 ID로 모든 Subject를 조회합니다.
#[utoipa::path(
    get,
    path = "/api/projects/{project_id}/subjects",
    tag = "subjects",
    params(
        ("project_id" = i32, Path, description = "Project ID")
    ),
    responses(
        (status = 200, description = "Subjects retrieved successfully", body = Vec<Subject>),
        (status = 404, description = "Project not found"),
    )
)]
pub async fn get_subjects_by_project<S: SubjectService + 'static>(
    subject_service: web::Data<Arc<S>>,
    project_id: web::Path<i32>,
) -> impl Responder {
    match subject_service.get_subjects_by_project(*project_id).await {
        Ok(subjects) => HttpResponse::Ok().json(subjects),
        Err(e) => {
            let status = match e {
                ServiceError::NotFound(_) => HttpResponse::NotFound(),
                _ => HttpResponse::InternalServerError(),
            };
            status.json(json!({
                "error": format!("{}", e)
            }))
        }
    }
}

/// Subject 수정
///
/// Subject ID로 Subject를 수정합니다.
#[utoipa::path(
    put,
    path = "/api/subjects/{id}",
    tag = "subjects",
    params(
        ("id" = i32, Path, description = "Subject ID")
    ),
    request_body = UpdateSubject,
    responses(
        (status = 200, description = "Subject updated successfully", body = Subject),
        (status = 400, description = "Invalid request or validation error"),
        (status = 404, description = "Subject not found"),
        (status = 409, description = "Subject code or Patient ID already exists"),
    )
)]
pub async fn update_subject<S: SubjectService + 'static>(
    subject_service: web::Data<Arc<S>>,
    id: web::Path<i32>,
    req: web::Json<UpdateSubject>,
) -> impl Responder {
    match subject_service
        .update_subject(*id, req.into_inner())
        .await
    {
        Ok(subject) => HttpResponse::Ok().json(subject),
        Err(e) => {
            let status = match e {
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

/// Subject 삭제
///
/// Subject ID로 Subject를 삭제합니다.
#[utoipa::path(
    delete,
    path = "/api/subjects/{id}",
    tag = "subjects",
    params(
        ("id" = i32, Path, description = "Subject ID")
    ),
    responses(
        (status = 204, description = "Subject deleted successfully"),
        (status = 400, description = "Cannot delete subject with timepoints"),
        (status = 404, description = "Subject not found"),
    )
)]
pub async fn delete_subject<S: SubjectService + 'static>(
    subject_service: web::Data<Arc<S>>,
    id: web::Path<i32>,
) -> impl Responder {
    match subject_service.delete_subject(*id).await {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(e) => {
            let status = match e {
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

/// 라우트 설정
pub fn configure_routes<S: SubjectService + 'static>(
    cfg: &mut web::ServiceConfig,
    subject_service: Arc<S>,
) {
    cfg.app_data(web::Data::new(subject_service))
        .service(
            web::scope("/projects/{project_id}/subjects")
                .route("", web::post().to(create_subject::<S>))
                .route("", web::get().to(get_subjects_by_project::<S>)),
        )
        .service(
            web::scope("/subjects")
                .route("/{id}", web::get().to(get_subject::<S>))
                .route("/{id}/detail", web::get().to(get_subject_detail::<S>))
                .route("/{id}", web::put().to(update_subject::<S>))
                .route("/{id}", web::delete().to(delete_subject::<S>)),
        );
}


