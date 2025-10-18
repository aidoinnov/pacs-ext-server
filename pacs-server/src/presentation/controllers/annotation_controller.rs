use actix_web::{web, HttpResponse, Responder, HttpRequest};
use serde_json::json;
use std::sync::Arc;
use crate::application::dto::annotation_dto::{
    CreateAnnotationRequest, UpdateAnnotationRequest,
    AnnotationResponse, AnnotationListResponse
};
use crate::application::use_cases::AnnotationUseCase;
use crate::domain::services::annotation_service::AnnotationService;
use crate::domain::ServiceError;
use crate::infrastructure::repositories::{AnnotationRepositoryImpl, UserRepositoryImpl, ProjectRepositoryImpl};
use crate::domain::services::AnnotationServiceImpl;

pub struct AnnotationController;

impl AnnotationController {
    pub fn new() -> Self {
        Self
    }
}

#[utoipa::path(
    post,
    path = "/api/annotations",
    tag = "annotations",
    request_body = CreateAnnotationRequest,
    responses(
        (status = 201, description = "Annotation created successfully", body = AnnotationResponse),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "User or Project not found"),
    )
)]
pub async fn create_annotation(
    req: web::Json<CreateAnnotationRequest>,
    use_case: web::Data<Arc<AnnotationUseCase<AnnotationServiceImpl<AnnotationRepositoryImpl, UserRepositoryImpl, ProjectRepositoryImpl>>>>,
    _http_req: HttpRequest,
) -> impl Responder {
    // TODO: 실제 인증에서 user_id와 project_id를 가져와야 함
    // 현재는 임시로 하드코딩된 값 사용
    // 테스트를 위해 실제 데이터베이스에 있는 값 사용
    // let user_id = 336; // 실제로는 JWT에서 추출 (getuser)
    // let user_id = req.body().user_id;
    // let project_id = 302; // 실제로는 요청에서 추출하거나 기본값 (Get Project)
    // let project_id = req.body().project_id;
    // project_id는 요청 body에서 가져오거나 기본값 사용
    let user_id = req.user_id.unwrap_or(1);
    let project_id = req.project_id.unwrap_or(299); // 또는 적절한 기본값


    match use_case.create_annotation(req.into_inner(), user_id, project_id).await {
        Ok(annotation) => HttpResponse::Created().json(annotation),
        Err(ServiceError::NotFound(msg)) => HttpResponse::NotFound().json(json!({
            "error": "Not Found",
            "message": msg
        })),
        Err(ServiceError::Unauthorized(msg)) => HttpResponse::Unauthorized().json(json!({
            "error": "Unauthorized",
            "message": msg
        })),
        Err(ServiceError::ValidationError(msg)) => HttpResponse::BadRequest().json(json!({
            "error": "Validation Error",
            "message": msg
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": "Internal Server Error",
            "message": e.to_string()
        })),
    }
}

#[utoipa::path(
    get,
    path = "/api/annotations/{annotation_id}",
    tag = "annotations",
    params(
        ("annotation_id" = i32, Path, description = "Annotation ID")
    ),
    responses(
        (status = 200, description = "Get annotation successfully", body = AnnotationResponse),
        (status = 404, description = "Annotation not found"),
    )
)]
pub async fn get_annotation(
    annotation_id: web::Path<i32>,
    use_case: web::Data<Arc<AnnotationUseCase<AnnotationServiceImpl<AnnotationRepositoryImpl, UserRepositoryImpl, ProjectRepositoryImpl>>>>,
) -> impl Responder {
    match use_case.get_annotation_by_id(*annotation_id).await {
        Ok(annotation) => HttpResponse::Ok().json(annotation),
        Err(ServiceError::NotFound(msg)) => HttpResponse::NotFound().json(json!({
            "error": "Not Found",
            "message": msg
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": "Internal Server Error",
            "message": e.to_string()
        })),
    }
}

#[utoipa::path(
    get,
    path = "/api/annotations",
    tag = "annotations",
    params(
        ("study_instance_uid" = Option<String>, Query, description = "Study Instance UID로 필터링"),
        ("user_id" = Option<i32>, Query, description = "사용자 ID로 필터링"),
        ("project_id" = Option<i32>, Query, description = "프로젝트 ID로 필터링"),
        ("viewer_software" = Option<String>, Query, description = "뷰어 소프트웨어로 필터링"),
    ),
    responses(
        (status = 200, description = "List annotations successfully", body = AnnotationListResponse),
    )
)]
pub async fn list_annotations(
    query: web::Query<std::collections::HashMap<String, String>>,
    use_case: web::Data<Arc<AnnotationUseCase<AnnotationServiceImpl<AnnotationRepositoryImpl, UserRepositoryImpl, ProjectRepositoryImpl>>>>,
) -> impl Responder {
    // TODO: 실제로는 인증에서 user_id를 가져와야 함
    // 기본값으로 1을 사용하지만, 쿼리 파라미터가 있으면 그것을 사용
    let mut user_id = 336;

    // 쿼리 파라미터에서 user_id 추출
    if let Some(user_id_str) = query.get("user_id") {
        if let Ok(user_id_param) = user_id_str.parse::<i32>() {
            user_id = user_id_param;
        }
    }

    // viewer_software 파라미터 추출
    let viewer_software = query.get("viewer_software").map(|s| s.as_str());

    // 쿼리 파라미터에 따라 다른 메서드 호출
    let result = if let Some(study_uid) = query.get("study_instance_uid") {
        use_case.get_annotations_by_study_with_viewer(study_uid, viewer_software).await
    } else if let Some(project_id_str) = query.get("project_id") {
        if let Ok(project_id) = project_id_str.parse::<i32>() {
            use_case.get_annotations_by_project_with_viewer(project_id, viewer_software).await
        } else {
            use_case.get_annotations_by_user_with_viewer(user_id, viewer_software).await
        }
    } else {
        // 기본적으로 사용자의 annotation 목록 반환 (user_id 쿼리 파라미터가 있으면 그것을 사용)
        use_case.get_annotations_by_user_with_viewer(user_id, viewer_software).await
    };

    match result {
        Ok(annotations) => HttpResponse::Ok().json(annotations),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": "Internal Server Error",
            "message": e.to_string()
        })),
    }
}

#[utoipa::path(
    put,
    path = "/api/annotations/{annotation_id}",
    tag = "annotations",
    request_body = UpdateAnnotationRequest,
    params(
        ("annotation_id" = i32, Path, description = "Annotation ID")
    ),
    responses(
        (status = 200, description = "Annotation updated successfully", body = AnnotationResponse),
        (status = 404, description = "Annotation not found"),
        (status = 400, description = "Invalid request"),
    )
)]
pub async fn update_annotation(
    annotation_id: web::Path<i32>,
    req: web::Json<UpdateAnnotationRequest>,
    use_case: web::Data<Arc<AnnotationUseCase<AnnotationServiceImpl<AnnotationRepositoryImpl, UserRepositoryImpl, ProjectRepositoryImpl>>>>,
) -> impl Responder {
    match use_case.update_annotation(*annotation_id, req.into_inner()).await {
        Ok(annotation) => HttpResponse::Ok().json(annotation),
        Err(ServiceError::NotFound(msg)) => HttpResponse::NotFound().json(json!({
            "error": "Not Found",
            "message": msg
        })),
        Err(ServiceError::ValidationError(msg)) => HttpResponse::BadRequest().json(json!({
            "error": "Validation Error",
            "message": msg
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": "Internal Server Error",
            "message": e.to_string()
        })),
    }
}

#[utoipa::path(
    delete,
    path = "/api/annotations/{annotation_id}",
    tag = "annotations",
    params(
        ("annotation_id" = i32, Path, description = "Annotation ID")
    ),
    responses(
        (status = 200, description = "Annotation deleted successfully"),
        (status = 404, description = "Annotation not found"),
    )
)]
pub async fn delete_annotation(
    annotation_id: web::Path<i32>,
    use_case: web::Data<Arc<AnnotationUseCase<AnnotationServiceImpl<AnnotationRepositoryImpl, UserRepositoryImpl, ProjectRepositoryImpl>>>>,
) -> impl Responder {
    match use_case.delete_annotation(*annotation_id).await {
        Ok(_) => HttpResponse::Ok().json(json!({
            "message": "Annotation deleted successfully"
        })),
        Err(ServiceError::NotFound(msg)) => HttpResponse::NotFound().json(json!({
            "error": "Not Found",
            "message": msg
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": "Internal Server Error",
            "message": e.to_string()
        })),
    }
}

pub fn configure_routes(cfg: &mut web::ServiceConfig, use_case: Arc<AnnotationUseCase<AnnotationServiceImpl<AnnotationRepositoryImpl, UserRepositoryImpl, ProjectRepositoryImpl>>>) {
    cfg.app_data(web::Data::new(use_case))
        .service(
            web::scope("/annotations")
                .route("", web::post().to(create_annotation))
                .route("", web::get().to(list_annotations))
                .route("/{annotation_id}", web::get().to(get_annotation))
                .route("/{annotation_id}", web::put().to(update_annotation))
                .route("/{annotation_id}", web::delete().to(delete_annotation)),
        );
}
