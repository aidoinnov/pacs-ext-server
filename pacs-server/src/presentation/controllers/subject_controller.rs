use actix_web::{web, HttpResponse, Responder};
use serde_json::json;
use std::sync::Arc;

use crate::domain::entities::{
    CreateRecistLesion, CreateRecistLesionAnnotationMap, CreateRecistLesionRequest, CreateSubject,
    CreateSubjectRequest, CreateTimePoint, RecistLesion, RecistLesionDetail, RecistLesionType,
    StudyInfo, Subject, SubjectDetail, TimePoint, UpdateRecistLesion, UpdateSubject,
};
use crate::domain::services::{SubjectService, TimePointService};
use crate::domain::ServiceError;
use crate::application::use_cases::RecistLesionUseCase;

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
    request_body = CreateSubjectRequest,
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
    req: web::Json<CreateSubjectRequest>,
) -> impl Responder {
    let req_data = req.into_inner();
    let new_subject = CreateSubject {
        project_id: *project_id,
        subject_code: req_data.subject_code,
        patient_id: req_data.patient_id,
        patient_name: req_data.patient_name,
        patient_birth_date: req_data.patient_birth_date,
    };

    match subject_service.create_subject(new_subject).await {
        Ok(subject) => HttpResponse::Created().json(subject),
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

// ============================================================================
// TimePoint 래퍼 핸들러 (경로 파라미터 이름 변환용)
// ============================================================================

/// TimePoint 생성 래퍼 (id -> subject_id 변환)
async fn create_timepoint_wrapper<T: TimePointService + 'static>(
    timepoint_service: web::Data<Arc<T>>,
    id: web::Path<i32>,
    req: web::Json<CreateTimePoint>,
) -> impl Responder {
    let mut new_timepoint = req.into_inner();
    new_timepoint.subject_id = *id;

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

/// Subject의 TimePoint 목록 조회 래퍼
async fn get_timepoints_wrapper<T: TimePointService + 'static>(
    timepoint_service: web::Data<Arc<T>>,
    id: web::Path<i32>,
) -> impl Responder {
    match timepoint_service.get_timepoints_by_subject(*id).await {
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

/// Subject의 미할당 Study 목록 조회 래퍼
async fn get_unassigned_studies_wrapper<T: TimePointService + 'static>(
    timepoint_service: web::Data<Arc<T>>,
    id: web::Path<i32>,
) -> impl Responder {
    match timepoint_service
        .get_unassigned_studies_by_subject(*id)
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

// ============================================================================
// 라우트 설정
// ============================================================================

/// 라우트 설정 (Subject + TimePoint)
///
/// Note: RECIST Lesion 라우트는 별도의 configure_recist_lesion_routes 함수로 분리되어 있습니다.
pub fn configure_routes<S: SubjectService + 'static, T: TimePointService + 'static>(
    cfg: &mut web::ServiceConfig,
    subject_service: Arc<S>,
    timepoint_service: Arc<T>,
) {
    cfg.app_data(web::Data::new(subject_service))
        .app_data(web::Data::new(timepoint_service))
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
                .route("/{id}", web::delete().to(delete_subject::<S>))
                // TimePoint 관련 라우트 추가 (래퍼 함수 사용)
                .route("/{id}/timepoints", web::post().to(create_timepoint_wrapper::<T>))
                .route("/{id}/timepoints", web::get().to(get_timepoints_wrapper::<T>))
                .route("/{id}/studies/unassigned", web::get().to(get_unassigned_studies_wrapper::<T>)),
        );
}

// ============================================================================
// RECIST Lesion Endpoints
// ============================================================================

/// Subject의 RECIST Lesion 목록 조회
#[utoipa::path(
    get,
    path = "/api/recist-lesions/subjects/{subject_id}",
    tag = "recist-lesions",
    params(
        ("subject_id" = i32, Path, description = "Subject ID"),
        ("lesion_type" = Option<String>, Query, description = "Lesion type filter (target/non_target)")
    ),
    responses(
        (status = 200, description = "Lesion list retrieved successfully", body = Vec<RecistLesion>),
        (status = 404, description = "Subject not found"),
    )
)]
pub async fn list_lesions<R, S, T>(
    lesion_use_case: web::Data<Arc<RecistLesionUseCase<R, S, T>>>,
    subject_id: web::Path<i32>,
    query: web::Query<serde_json::Value>,
) -> impl Responder
where
    R: crate::domain::repositories::RecistLesionRepository + 'static,
    S: crate::domain::repositories::SubjectRepository + 'static,
    T: crate::domain::repositories::TimePointRepository + 'static,
{
    let lesion_type = query
        .get("lesion_type")
        .and_then(|v| v.as_str())
        .and_then(|s| match s.to_lowercase().as_str() {
            "target" => Some(RecistLesionType::Target),
            "non_target" => Some(RecistLesionType::NonTarget),
            _ => None,
        });

    match lesion_use_case
        .list_lesions_by_subject(*subject_id, lesion_type)
        .await
    {
        Ok(lesions) => HttpResponse::Ok().json(lesions),
        Err(e) => {
            let mut status = match e {
                ServiceError::NotFound(_) => HttpResponse::NotFound(),
                _ => HttpResponse::InternalServerError(),
            };
            status.json(json!({"error": format!("{}", e)}))
        }
    }
}

/// RECIST Lesion 생성
#[utoipa::path(
    post,
    path = "/api/recist-lesions/subjects/{subject_id}",
    tag = "recist-lesions",
    params(
        ("subject_id" = i32, Path, description = "Subject ID")
    ),
    request_body = CreateRecistLesionRequest,
    responses(
        (status = 201, description = "Lesion created successfully", body = RecistLesion),
        (status = 400, description = "Validation error (e.g., max 5 Target Lesions)"),
        (status = 404, description = "Subject or Baseline TimePoint not found"),
    )
)]
pub async fn create_lesion<R, S, T>(
    lesion_use_case: web::Data<Arc<RecistLesionUseCase<R, S, T>>>,
    subject_id: web::Path<i32>,
    req: web::Json<CreateRecistLesionRequest>,
) -> impl Responder
where
    R: crate::domain::repositories::RecistLesionRepository + 'static,
    S: crate::domain::repositories::SubjectRepository + 'static,
    T: crate::domain::repositories::TimePointRepository + 'static,
{
    let request = req.into_inner();

    // CreateRecistLesionRequest를 CreateRecistLesion으로 변환
    // project_id는 Use Case에서 Subject 조회 시 검증됨
    let new_lesion = CreateRecistLesion {
        project_id: 0, // Use Case에서 Subject 조회 후 검증됨
        subject_id: *subject_id,
        lesion_type: request.lesion_type,
        baseline_timepoint_id: request.baseline_timepoint_id,
        organ_site: request.organ_site,
        description: request.description,
    };

    match lesion_use_case.create_lesion(new_lesion).await {
        Ok(lesion) => HttpResponse::Created().json(lesion),
        Err(e) => {
            let mut status = match e {
                ServiceError::NotFound(_) => HttpResponse::NotFound(),
                ServiceError::ValidationError(_) => HttpResponse::BadRequest(),
                _ => HttpResponse::InternalServerError(),
            };
            status.json(json!({"error": format!("{}", e)}))
        }
    }
}

/// RECIST Lesion 상세 조회
#[utoipa::path(
    get,
    path = "/api/recist-lesions/{id}",
    tag = "recist-lesions",
    params(
        ("id" = i32, Path, description = "Lesion ID")
    ),
    responses(
        (status = 200, description = "Lesion detail retrieved successfully", body = RecistLesionDetail),
        (status = 404, description = "Lesion not found"),
    )
)]
pub async fn get_lesion_detail<R, S, T>(
    lesion_use_case: web::Data<Arc<RecistLesionUseCase<R, S, T>>>,
    id: web::Path<i32>,
) -> impl Responder
where
    R: crate::domain::repositories::RecistLesionRepository + 'static,
    S: crate::domain::repositories::SubjectRepository + 'static,
    T: crate::domain::repositories::TimePointRepository + 'static,
{
    match lesion_use_case.get_lesion_detail(*id).await {
        Ok(detail) => HttpResponse::Ok().json(detail),
        Err(e) => {
            let mut status = match e {
                ServiceError::NotFound(_) => HttpResponse::NotFound(),
                _ => HttpResponse::InternalServerError(),
            };
            status.json(json!({"error": format!("{}", e)}))
        }
    }
}

/// RECIST Lesion 수정
#[utoipa::path(
    put,
    path = "/api/recist-lesions/{id}",
    tag = "recist-lesions",
    params(
        ("id" = i32, Path, description = "Lesion ID")
    ),
    request_body = UpdateRecistLesion,
    responses(
        (status = 200, description = "Lesion updated successfully", body = RecistLesion),
        (status = 404, description = "Lesion not found"),
    )
)]
pub async fn update_lesion<R, S, T>(
    lesion_use_case: web::Data<Arc<RecistLesionUseCase<R, S, T>>>,
    id: web::Path<i32>,
    req: web::Json<UpdateRecistLesion>,
) -> impl Responder
where
    R: crate::domain::repositories::RecistLesionRepository + 'static,
    S: crate::domain::repositories::SubjectRepository + 'static,
    T: crate::domain::repositories::TimePointRepository + 'static,
{
    match lesion_use_case.update_lesion(*id, req.into_inner()).await {
        Ok(lesion) => HttpResponse::Ok().json(lesion),
        Err(e) => {
            let mut status = match e {
                ServiceError::NotFound(_) => HttpResponse::NotFound(),
                _ => HttpResponse::InternalServerError(),
            };
            status.json(json!({"error": format!("{}", e)}))
        }
    }
}

/// RECIST Lesion 삭제
#[utoipa::path(
    delete,
    path = "/api/recist-lesions/{id}",
    tag = "recist-lesions",
    params(
        ("id" = i32, Path, description = "Lesion ID")
    ),
    responses(
        (status = 204, description = "Lesion deleted successfully"),
        (status = 404, description = "Lesion not found"),
    )
)]
pub async fn delete_lesion<R, S, T>(
    lesion_use_case: web::Data<Arc<RecistLesionUseCase<R, S, T>>>,
    id: web::Path<i32>,
) -> impl Responder
where
    R: crate::domain::repositories::RecistLesionRepository + 'static,
    S: crate::domain::repositories::SubjectRepository + 'static,
    T: crate::domain::repositories::TimePointRepository + 'static,
{
    match lesion_use_case.delete_lesion(*id).await {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(e) => {
            let mut status = match e {
                ServiceError::NotFound(_) => HttpResponse::NotFound(),
                _ => HttpResponse::InternalServerError(),
            };
            status.json(json!({"error": format!("{}", e)}))
        }
    }
}

/// Lesion에 Annotation 연결
#[utoipa::path(
    post,
    path = "/api/recist-lesions/{id}/annotations",
    tag = "recist-lesions",
    params(
        ("id" = i32, Path, description = "Lesion ID")
    ),
    request_body = CreateRecistLesionAnnotationMap,
    responses(
        (status = 201, description = "Annotation linked successfully"),
        (status = 404, description = "Lesion or TimePoint not found"),
    )
)]
pub async fn link_annotation<R, S, T>(
    lesion_use_case: web::Data<Arc<RecistLesionUseCase<R, S, T>>>,
    id: web::Path<i32>,
    req: web::Json<CreateRecistLesionAnnotationMap>,
) -> impl Responder
where
    R: crate::domain::repositories::RecistLesionRepository + 'static,
    S: crate::domain::repositories::SubjectRepository + 'static,
    T: crate::domain::repositories::TimePointRepository + 'static,
{
    let mut mapping = req.into_inner();
    mapping.lesion_id = *id;

    match lesion_use_case.link_annotation(mapping).await {
        Ok(_) => HttpResponse::Created().json(json!({"message": "Annotation linked successfully"})),
        Err(e) => {
            let mut status = match e {
                ServiceError::NotFound(_) => HttpResponse::NotFound(),
                _ => HttpResponse::InternalServerError(),
            };
            status.json(json!({"error": format!("{}", e)}))
        }
    }
}

/// RECIST Lesion 라우트 설정
///
/// Note: 이 함수는 별도로 호출되어야 하며, configure_routes와 독립적입니다.
/// main.rs에서 별도의 .configure() 블록으로 등록됩니다.
///
/// 라우트 충돌 방지를 위해 명시적으로 web::scope를 사용합니다.
pub fn configure_recist_lesion_routes<R, S, T>(
    cfg: &mut web::ServiceConfig,
    lesion_use_case: Arc<RecistLesionUseCase<R, S, T>>,
)
where
    R: crate::domain::repositories::RecistLesionRepository + 'static,
    S: crate::domain::repositories::SubjectRepository + 'static,
    T: crate::domain::repositories::TimePointRepository + 'static,
{
    cfg.app_data(web::Data::new(lesion_use_case.clone()))
        // RECIST Lesion 전용 스코프 - Subject 스코프와 완전히 분리
        .service(
            web::scope("/recist-lesions")
                // Subject의 Lesion 목록 조회 및 생성
                .service(
                    web::resource("/subjects/{subject_id}")
                        .route(web::get().to(list_lesions::<R, S, T>))
                        .route(web::post().to(create_lesion::<R, S, T>)),
                )
                // Lesion 상세 조회, 수정, 삭제
                .service(
                    web::resource("/{id}")
                        .route(web::get().to(get_lesion_detail::<R, S, T>))
                        .route(web::put().to(update_lesion::<R, S, T>))
                        .route(web::delete().to(delete_lesion::<R, S, T>)),
                )
                // Annotation 연결
                .service(
                    web::resource("/{id}/annotations")
                        .route(web::post().to(link_annotation::<R, S, T>)),
                ),
        );
}
