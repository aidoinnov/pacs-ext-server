#![allow(dead_code, unused_imports, unused_variables)]
use crate::application::dto::annotation_dto::{
    AnnotationListResponse, AnnotationResponse, CreateAnnotationRequest, UpdateAnnotationRequest,
};
use crate::application::use_cases::AnnotationUseCase;
use crate::domain::services::annotation_service::AnnotationService;
use crate::domain::services::{AnnotationServiceImpl, AccessControlServiceImpl};
use crate::domain::ServiceError;
use crate::infrastructure::repositories::{
    AnnotationRepositoryImpl, ProjectRepositoryImpl, UserRepositoryImpl,
    AccessLogRepositoryImpl, RoleRepositoryImpl, PermissionRepositoryImpl,
};
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use serde_json::json;
use std::sync::Arc;

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
    use_case: web::Data<
        Arc<
            AnnotationUseCase<
                AnnotationServiceImpl<
                    AnnotationRepositoryImpl,
                    UserRepositoryImpl,
                    ProjectRepositoryImpl,
                >,
                UserRepositoryImpl,
                AccessControlServiceImpl<
                    AccessLogRepositoryImpl,
                    UserRepositoryImpl,
                    ProjectRepositoryImpl,
                    RoleRepositoryImpl,
                    PermissionRepositoryImpl,
                >,
            >,
        >,
    >,
    _http_req: HttpRequest,
) -> impl Responder {
    // TODO: 실제 인증에서 user_id와 project_id를 가져와야 함
    // 현재는 요청 body에서 가져오거나 기본값 사용
    let user_id = req.user_id.unwrap_or(1);
    let project_id = req.project_id.unwrap_or(299); // 또는 적절한 기본값

    match use_case
        .create_annotation(req.into_inner(), user_id, project_id)
        .await
    {
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
    use_case: web::Data<
        Arc<
            AnnotationUseCase<
                AnnotationServiceImpl<
                    AnnotationRepositoryImpl,
                    UserRepositoryImpl,
                    ProjectRepositoryImpl,
                >,
                UserRepositoryImpl,
                AccessControlServiceImpl<
                    AccessLogRepositoryImpl,
                    UserRepositoryImpl,
                    ProjectRepositoryImpl,
                    RoleRepositoryImpl,
                    PermissionRepositoryImpl,
                >,
            >,
        >,
    >,
) -> impl Responder {
    match use_case.get_annotation_by_id(*annotation_id).await {
        Ok(annotation) => HttpResponse::Ok()
            .insert_header(("Cache-Control", "public, max-age=5"))
            .json(annotation),
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
        ("series_instance_uid" = Option<String>, Query, description = "Series Instance UID로 필터링"),
        ("sop_instance_uid" = Option<String>, Query, description = "SOP Instance UID로 필터링"),
        ("user_id" = Option<i32>, Query, description = "사용자 ID로 필터링"),
        ("project_id" = Option<i32>, Query, description = "프로젝트 ID로 필터링"),
        ("viewer_software" = Option<String>, Query, description = "뷰어 소프트웨어로 필터링"),
        ("level" = Option<String>, Query, description = "어노테이션 레벨로 필터링 (study, series, instance)"),
    ),
    responses(
        (status = 200, description = "List annotations successfully", body = AnnotationListResponse),
    )
)]
pub async fn list_annotations(
    query: web::Query<std::collections::HashMap<String, String>>,
    use_case: web::Data<
        Arc<
            AnnotationUseCase<
                AnnotationServiceImpl<
                    AnnotationRepositoryImpl,
                    UserRepositoryImpl,
                    ProjectRepositoryImpl,
                >,
                UserRepositoryImpl,
                AccessControlServiceImpl<
                    AccessLogRepositoryImpl,
                    UserRepositoryImpl,
                    ProjectRepositoryImpl,
                    RoleRepositoryImpl,
                    PermissionRepositoryImpl,
                >,
            >,
        >,
    >,
) -> impl Responder {
    // TODO: 실제로는 인증에서 user_id를 가져와야 함
    // 기본값으로 1을 사용하지만, 쿼리 파라미터가 있으면 그것을 사용
    let mut user_id = 336;

    // 쿼리 파라미터에서 user_id 추출
    let user_id_param = query.get("user_id").and_then(|s| s.parse::<i32>().ok());
    if let Some(uid) = user_id_param {
        user_id = uid;
    }

    // viewer_software 파라미터 추출
    let viewer_software = query.get("viewer_software").map(|s| s.as_str());

    // project_id 파라미터 추출
    let project_id = query.get("project_id").and_then(|s| s.parse::<i32>().ok());

    // level 파라미터 추출
    let level = query.get("level").map(|s| s.as_str());

    // 쿼리 파라미터에 따라 다른 메서드 호출
    // 우선순위: sop_instance_uid > series_instance_uid > study_instance_uid > project_id > user_id
    // 주의: user_id 파라미터가 있으면 권한 기반 필터링을 수행합니다
    let result = if let Some(sop_instance_uid) = query.get("sop_instance_uid") {
        // SOP Instance UID (가장 구체적인 필터)
        use_case
            .get_annotations_by_instance(sop_instance_uid)
            .await
            .map(|mut response| {
                // level로 필터링
                if let Some(lvl) = level {
                    match lvl {
                        "study" => {
                            // Study 레벨: series_uid와 instance_uid가 모두 비어있음
                            response.annotations.retain(|ann| {
                                ann.series_instance_uid.is_empty() && ann.sop_instance_uid.is_empty()
                            });
                        }
                        "series" => {
                            // Series 레벨: series_uid는 있고 instance_uid는 비어있음
                            response.annotations.retain(|ann| {
                                !ann.series_instance_uid.is_empty() && ann.sop_instance_uid.is_empty()
                            });
                        }
                        "instance" => {
                            // Instance 레벨: instance_uid가 있음
                            response.annotations.retain(|ann| {
                                !ann.sop_instance_uid.is_empty()
                            });
                        }
                        _ => {} // 잘못된 level 값은 무시
                    }
                    response.total = response.annotations.len();
                }

                // viewer_software로 추가 필터링
                if let Some(viewer) = viewer_software {
                    response.annotations.retain(|ann| {
                        ann.viewer_software.as_ref()
                            .map(|v| v.as_str() == viewer)
                            .unwrap_or(false)
                    });
                    response.total = response.annotations.len();
                }

                // user_id로 추가 필터링 (쿼리 파라미터에 명시된 경우)
                if query.get("user_id").is_some() {
                    response.annotations.retain(|ann| ann.user_id == user_id);
                    response.total = response.annotations.len();
                }

                response
            })
    } else if let Some(series_instance_uid) = query.get("series_instance_uid") {
        // Series Instance UID 처리
        if let Some(proj_id) = project_id {
            // series_instance_uid + project_id 조합
            // user_id 파라미터가 있으면 권한 기반 필터링 수행
            if user_id_param.is_some() {
                use_case
                    .get_annotations_by_series_and_project_with_user(user_id, series_instance_uid, proj_id)
                    .await
                    .map(|mut response| {
                        // 권한 기반 필터링: READ_ALL 권한이 없으면 본인 어노테이션만
                        // (이미 UseCase에서 처리됨)

                        // level로 필터링
                        if let Some(lvl) = level {
                            match lvl {
                                "study" => {
                                    response.annotations.retain(|ann| {
                                        ann.series_instance_uid.is_empty() && ann.sop_instance_uid.is_empty()
                                    });
                                }
                                "series" => {
                                    response.annotations.retain(|ann| {
                                        !ann.series_instance_uid.is_empty() && ann.sop_instance_uid.is_empty()
                                    });
                                }
                                "instance" => {
                                    response.annotations.retain(|ann| {
                                        !ann.sop_instance_uid.is_empty()
                                    });
                                }
                                _ => {}
                            }
                            response.total = response.annotations.len();
                        }

                        // viewer_software로 추가 필터링
                        if let Some(viewer) = viewer_software {
                            response.annotations.retain(|ann| {
                                ann.viewer_software.as_ref()
                                    .map(|v| v.as_str() == viewer)
                                    .unwrap_or(false)
                            });
                            response.total = response.annotations.len();
                        }

                        response
                    })
            } else {
                // user_id 파라미터가 없으면 기존 방식 (권한 체크 없음)
                use_case
                    .get_annotations_by_project_and_series(proj_id, series_instance_uid)
                    .await
                    .map(|mut response| {
                        // level로 필터링
                        if let Some(lvl) = level {
                            match lvl {
                                "study" => {
                                    response.annotations.retain(|ann| {
                                        ann.series_instance_uid.is_empty() && ann.sop_instance_uid.is_empty()
                                    });
                                }
                                "series" => {
                                    response.annotations.retain(|ann| {
                                        !ann.series_instance_uid.is_empty() && ann.sop_instance_uid.is_empty()
                                    });
                                }
                                "instance" => {
                                    response.annotations.retain(|ann| {
                                        !ann.sop_instance_uid.is_empty()
                                    });
                                }
                                _ => {}
                            }
                            response.total = response.annotations.len();
                        }

                        // viewer_software로 추가 필터링
                        if let Some(viewer) = viewer_software {
                            response.annotations.retain(|ann| {
                                ann.viewer_software.as_ref()
                                    .map(|v| v.as_str() == viewer)
                                    .unwrap_or(false)
                            });
                            response.total = response.annotations.len();
                        }

                        response
                    })
            }
        } else {
            // series_instance_uid만 있으면 권한 체크 없이 조회
            use_case
                .get_annotations_by_series(series_instance_uid)
                .await
                .map(|mut response| {
                    // level로 필터링
                    if let Some(lvl) = level {
                        match lvl {
                            "study" => {
                                response.annotations.retain(|ann| {
                                    ann.series_instance_uid.is_empty() && ann.sop_instance_uid.is_empty()
                                });
                            }
                            "series" => {
                                response.annotations.retain(|ann| {
                                    !ann.series_instance_uid.is_empty() && ann.sop_instance_uid.is_empty()
                                });
                            }
                            "instance" => {
                                response.annotations.retain(|ann| {
                                    !ann.sop_instance_uid.is_empty()
                                });
                            }
                            _ => {}
                        }
                        response.total = response.annotations.len();
                    }

                    // viewer_software로 추가 필터링
                    if let Some(viewer) = viewer_software {
                        response.annotations.retain(|ann| {
                            ann.viewer_software.as_ref()
                                .map(|v| v.as_str() == viewer)
                                .unwrap_or(false)
                        });
                        response.total = response.annotations.len();
                    }

                    response
                })
        }
    } else if let Some(study_uid) = query.get("study_instance_uid") {
        if let Some(proj_id) = project_id {
            // study_instance_uid + project_id 조합
            // user_id 파라미터가 있으면 권한 기반 필터링 수행
            if user_id_param.is_some() {
                use_case
                    .get_annotations_by_project_and_study_with_user(user_id, proj_id, study_uid)
                    .await
                    .map(|mut response| {
                        // level로 필터링
                        if let Some(lvl) = level {
                            match lvl {
                                "study" => {
                                    response.annotations.retain(|ann| {
                                        ann.series_instance_uid.is_empty() && ann.sop_instance_uid.is_empty()
                                    });
                                }
                                "series" => {
                                    response.annotations.retain(|ann| {
                                        !ann.series_instance_uid.is_empty() && ann.sop_instance_uid.is_empty()
                                    });
                                }
                                "instance" => {
                                    response.annotations.retain(|ann| {
                                        !ann.sop_instance_uid.is_empty()
                                    });
                                }
                                _ => {}
                            }
                            response.total = response.annotations.len();
                        }

                        // viewer_software로 추가 필터링
                        if let Some(viewer) = viewer_software {
                            response.annotations.retain(|ann| {
                                ann.viewer_software.as_ref()
                                    .map(|v| v.as_str() == viewer)
                                    .unwrap_or(false)
                            });
                            response.total = response.annotations.len();
                        }

                        response
                    })
            } else {
                // user_id 파라미터가 없으면 기존 방식 (권한 체크 없음)
                match use_case
                    .get_annotations_by_project_and_study(proj_id, study_uid)
                    .await
                {
                    Ok(mut response) => {
                        // level로 필터링
                        if let Some(lvl) = level {
                            match lvl {
                                "study" => {
                                    response.annotations.retain(|ann| {
                                        ann.series_instance_uid.is_empty() && ann.sop_instance_uid.is_empty()
                                    });
                                }
                                "series" => {
                                    response.annotations.retain(|ann| {
                                        !ann.series_instance_uid.is_empty() && ann.sop_instance_uid.is_empty()
                                    });
                                }
                                "instance" => {
                                    response.annotations.retain(|ann| {
                                        !ann.sop_instance_uid.is_empty()
                                    });
                                }
                                _ => {}
                            }
                            response.total = response.annotations.len();
                        }

                        // viewer_software로 추가 필터링
                        if let Some(viewer) = viewer_software {
                            response.annotations.retain(|ann| {
                                ann.viewer_software.as_ref()
                                    .map(|v| v.as_str() == viewer)
                                    .unwrap_or(false)
                            });
                            response.total = response.annotations.len();
                        }

                        Ok(response)
                    }
                    Err(e) => Err(e),
                }
            }
        } else {
            // study_instance_uid만 있으면 study로 필터링
            use_case
                .get_annotations_by_study_with_viewer(study_uid, viewer_software)
                .await
                .map(|mut response| {
                    // level로 필터링
                    if let Some(lvl) = level {
                        match lvl {
                            "study" => {
                                response.annotations.retain(|ann| {
                                    ann.series_instance_uid.is_empty() && ann.sop_instance_uid.is_empty()
                                });
                            }
                            "series" => {
                                response.annotations.retain(|ann| {
                                    !ann.series_instance_uid.is_empty() && ann.sop_instance_uid.is_empty()
                                });
                            }
                            "instance" => {
                                response.annotations.retain(|ann| {
                                    !ann.sop_instance_uid.is_empty()
                                });
                            }
                            _ => {}
                        }
                        response.total = response.annotations.len();
                    }

                    response
                })
        }
    } else if let Some(proj_id) = project_id {
        // project_id만 있으면 권한 기반으로 필터링
        // READ_ALL 권한이 있으면 프로젝트의 모든 annotation, 없으면 본인 annotation만
        use_case
            .get_annotations_by_project_with_permission(user_id, proj_id, viewer_software)
            .await
            .map(|mut response| {
                // level로 필터링
                if let Some(lvl) = level {
                    match lvl {
                        "study" => {
                            response.annotations.retain(|ann| {
                                ann.series_instance_uid.is_empty() && ann.sop_instance_uid.is_empty()
                            });
                        }
                        "series" => {
                            response.annotations.retain(|ann| {
                                !ann.series_instance_uid.is_empty() && ann.sop_instance_uid.is_empty()
                            });
                        }
                        "instance" => {
                            response.annotations.retain(|ann| {
                                !ann.sop_instance_uid.is_empty()
                            });
                        }
                        _ => {}
                    }
                    response.total = response.annotations.len();
                }

                response
            })
    } else {
        // 기본적으로 사용자의 annotation 목록 반환
        use_case
            .get_annotations_by_user_with_viewer(user_id, viewer_software)
            .await
            .map(|mut response| {
                // level로 필터링
                if let Some(lvl) = level {
                    match lvl {
                        "study" => {
                            response.annotations.retain(|ann| {
                                ann.series_instance_uid.is_empty() && ann.sop_instance_uid.is_empty()
                            });
                        }
                        "series" => {
                            response.annotations.retain(|ann| {
                                !ann.series_instance_uid.is_empty() && ann.sop_instance_uid.is_empty()
                            });
                        }
                        "instance" => {
                            response.annotations.retain(|ann| {
                                !ann.sop_instance_uid.is_empty()
                            });
                        }
                        _ => {}
                    }
                    response.total = response.annotations.len();
                }

                response
            })
    };

    match result {
        Ok(annotations) => HttpResponse::Ok()
            .insert_header(("Cache-Control", "public, max-age=5"))
            .json(annotations),
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
    use_case: web::Data<
        Arc<
            AnnotationUseCase<
                AnnotationServiceImpl<
                    AnnotationRepositoryImpl,
                    UserRepositoryImpl,
                    ProjectRepositoryImpl,
                >,
                UserRepositoryImpl,
                AccessControlServiceImpl<
                    AccessLogRepositoryImpl,
                    UserRepositoryImpl,
                    ProjectRepositoryImpl,
                    RoleRepositoryImpl,
                    PermissionRepositoryImpl,
                >,
            >,
        >,
    >,
) -> impl Responder {
    match use_case
        .update_annotation(*annotation_id, req.into_inner())
        .await
    {
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
    use_case: web::Data<
        Arc<
            AnnotationUseCase<
                AnnotationServiceImpl<
                    AnnotationRepositoryImpl,
                    UserRepositoryImpl,
                    ProjectRepositoryImpl,
                >,
                UserRepositoryImpl,
                AccessControlServiceImpl<
                    AccessLogRepositoryImpl,
                    UserRepositoryImpl,
                    ProjectRepositoryImpl,
                    RoleRepositoryImpl,
                    PermissionRepositoryImpl,
                >,
            >,
        >,
    >,
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

pub fn configure_routes(
    cfg: &mut web::ServiceConfig,
    use_case: Arc<
        AnnotationUseCase<
            AnnotationServiceImpl<
                AnnotationRepositoryImpl,
                UserRepositoryImpl,
                ProjectRepositoryImpl,
            >,
            UserRepositoryImpl,
            AccessControlServiceImpl<
                AccessLogRepositoryImpl,
                UserRepositoryImpl,
                ProjectRepositoryImpl,
                RoleRepositoryImpl,
                PermissionRepositoryImpl,
            >,
        >,
    >,
    mask_group_use_case: Arc<
        crate::application::use_cases::MaskGroupUseCase<
            crate::domain::services::MaskGroupServiceImpl<
                crate::infrastructure::repositories::MaskGroupRepositoryImpl,
                crate::infrastructure::repositories::AnnotationRepositoryImpl,
                crate::infrastructure::repositories::UserRepositoryImpl,
            >,
            crate::application::services::SignedUrlServiceImpl,
        >,
    >,
) {
    cfg.app_data(web::Data::new(use_case))
        .app_data(web::Data::new(mask_group_use_case))
        .service(
            web::scope("/annotations")
                .route("", web::post().to(create_annotation))
                .route("", web::get().to(list_annotations))
                .route("/{annotation_id}", web::get().to(get_annotation))
                .route("/{annotation_id}", web::put().to(update_annotation))
                .route("/{annotation_id}", web::delete().to(delete_annotation))
                // Mask Groups routes
                .route(
                    "/{annotation_id}/mask-groups",
                    web::post().to(crate::presentation::controllers::mask_group_controller::create_mask_group::<
                        crate::domain::services::MaskGroupServiceImpl<
                            crate::infrastructure::repositories::MaskGroupRepositoryImpl,
                            crate::infrastructure::repositories::AnnotationRepositoryImpl,
                            crate::infrastructure::repositories::UserRepositoryImpl,
                        >,
                        crate::application::services::SignedUrlServiceImpl,
                    >),
                )
                .route(
                    "/{annotation_id}/mask-groups",
                    web::get().to(crate::presentation::controllers::mask_group_controller::list_mask_groups::<
                        crate::domain::services::MaskGroupServiceImpl<
                            crate::infrastructure::repositories::MaskGroupRepositoryImpl,
                            crate::infrastructure::repositories::AnnotationRepositoryImpl,
                            crate::infrastructure::repositories::UserRepositoryImpl,
                        >,
                        crate::application::services::SignedUrlServiceImpl,
                    >),
                )
                .route(
                    "/{annotation_id}/mask-groups/{group_id}",
                    web::get().to(crate::presentation::controllers::mask_group_controller::get_mask_group::<
                        crate::domain::services::MaskGroupServiceImpl<
                            crate::infrastructure::repositories::MaskGroupRepositoryImpl,
                            crate::infrastructure::repositories::AnnotationRepositoryImpl,
                            crate::infrastructure::repositories::UserRepositoryImpl,
                        >,
                        crate::application::services::SignedUrlServiceImpl,
                    >),
                )
                .route(
                    "/{annotation_id}/mask-groups/{group_id}",
                    web::put().to(crate::presentation::controllers::mask_group_controller::update_mask_group::<
                        crate::domain::services::MaskGroupServiceImpl<
                            crate::infrastructure::repositories::MaskGroupRepositoryImpl,
                            crate::infrastructure::repositories::AnnotationRepositoryImpl,
                            crate::infrastructure::repositories::UserRepositoryImpl,
                        >,
                        crate::application::services::SignedUrlServiceImpl,
                    >),
                )
                .route(
                    "/{annotation_id}/mask-groups/{group_id}",
                    web::delete().to(crate::presentation::controllers::mask_group_controller::delete_mask_group::<
                        crate::domain::services::MaskGroupServiceImpl<
                            crate::infrastructure::repositories::MaskGroupRepositoryImpl,
                            crate::infrastructure::repositories::AnnotationRepositoryImpl,
                            crate::infrastructure::repositories::UserRepositoryImpl,
                        >,
                        crate::application::services::SignedUrlServiceImpl,
                    >),
                )
                .route(
                    "/{annotation_id}/mask-groups/{group_id}/upload-url",
                    web::post().to(crate::presentation::controllers::mask_group_controller::generate_upload_url::<
                        crate::domain::services::MaskGroupServiceImpl<
                            crate::infrastructure::repositories::MaskGroupRepositoryImpl,
                            crate::infrastructure::repositories::AnnotationRepositoryImpl,
                            crate::infrastructure::repositories::UserRepositoryImpl,
                        >,
                        crate::application::services::SignedUrlServiceImpl,
                    >),
                )
                .route(
                    "/{annotation_id}/mask-groups/{group_id}/complete-upload",
                    web::post().to(crate::presentation::controllers::mask_group_controller::complete_upload::<
                        crate::domain::services::MaskGroupServiceImpl<
                            crate::infrastructure::repositories::MaskGroupRepositoryImpl,
                            crate::infrastructure::repositories::AnnotationRepositoryImpl,
                            crate::infrastructure::repositories::UserRepositoryImpl,
                        >,
                        crate::application::services::SignedUrlServiceImpl,
                    >),
                )
                // Mask routes
                .route(
                    "/{annotation_id}/mask-groups/{group_id}/masks",
                    web::post().to(crate::presentation::controllers::mask_controller::create_mask::<
                        crate::domain::services::MaskServiceImpl<
                            crate::infrastructure::repositories::MaskRepositoryImpl,
                            crate::infrastructure::repositories::MaskGroupRepositoryImpl,
                            crate::infrastructure::repositories::UserRepositoryImpl,
                        >,
                        crate::domain::services::MaskGroupServiceImpl<
                            crate::infrastructure::repositories::MaskGroupRepositoryImpl,
                            crate::infrastructure::repositories::AnnotationRepositoryImpl,
                            crate::infrastructure::repositories::UserRepositoryImpl,
                        >,
                        crate::application::services::SignedUrlServiceImpl,
                    >),
                )
                .route(
                    "/{annotation_id}/mask-groups/{group_id}/masks",
                    web::get().to(crate::presentation::controllers::mask_controller::list_masks::<
                        crate::domain::services::MaskServiceImpl<
                            crate::infrastructure::repositories::MaskRepositoryImpl,
                            crate::infrastructure::repositories::MaskGroupRepositoryImpl,
                            crate::infrastructure::repositories::UserRepositoryImpl,
                        >,
                        crate::domain::services::MaskGroupServiceImpl<
                            crate::infrastructure::repositories::MaskGroupRepositoryImpl,
                            crate::infrastructure::repositories::AnnotationRepositoryImpl,
                            crate::infrastructure::repositories::UserRepositoryImpl,
                        >,
                        crate::application::services::SignedUrlServiceImpl,
                    >),
                )
                .route(
                    "/{annotation_id}/mask-groups/{group_id}/masks/{mask_id}",
                    web::get().to(crate::presentation::controllers::mask_controller::get_mask::<
                        crate::domain::services::MaskServiceImpl<
                            crate::infrastructure::repositories::MaskRepositoryImpl,
                            crate::infrastructure::repositories::MaskGroupRepositoryImpl,
                            crate::infrastructure::repositories::UserRepositoryImpl,
                        >,
                        crate::domain::services::MaskGroupServiceImpl<
                            crate::infrastructure::repositories::MaskGroupRepositoryImpl,
                            crate::infrastructure::repositories::AnnotationRepositoryImpl,
                            crate::infrastructure::repositories::UserRepositoryImpl,
                        >,
                        crate::application::services::SignedUrlServiceImpl,
                    >),
                )
                .route(
                    "/{annotation_id}/mask-groups/{group_id}/masks/{mask_id}",
                    web::put().to(crate::presentation::controllers::mask_controller::update_mask::<
                        crate::domain::services::MaskServiceImpl<
                            crate::infrastructure::repositories::MaskRepositoryImpl,
                            crate::infrastructure::repositories::MaskGroupRepositoryImpl,
                            crate::infrastructure::repositories::UserRepositoryImpl,
                        >,
                        crate::domain::services::MaskGroupServiceImpl<
                            crate::infrastructure::repositories::MaskGroupRepositoryImpl,
                            crate::infrastructure::repositories::AnnotationRepositoryImpl,
                            crate::infrastructure::repositories::UserRepositoryImpl,
                        >,
                        crate::application::services::SignedUrlServiceImpl,
                    >),
                )
                .route(
                    "/{annotation_id}/mask-groups/{group_id}/masks/{mask_id}",
                    web::delete().to(crate::presentation::controllers::mask_controller::delete_mask::<
                        crate::domain::services::MaskServiceImpl<
                            crate::infrastructure::repositories::MaskRepositoryImpl,
                            crate::infrastructure::repositories::MaskGroupRepositoryImpl,
                            crate::infrastructure::repositories::UserRepositoryImpl,
                        >,
                        crate::domain::services::MaskGroupServiceImpl<
                            crate::infrastructure::repositories::MaskGroupRepositoryImpl,
                            crate::infrastructure::repositories::AnnotationRepositoryImpl,
                            crate::infrastructure::repositories::UserRepositoryImpl,
                        >,
                        crate::application::services::SignedUrlServiceImpl,
                    >),
                )
                .route(
                    "/{annotation_id}/mask-groups/{group_id}/masks/{mask_id}/download-url",
                    web::post().to(crate::presentation::controllers::mask_controller::generate_download_url::<
                        crate::domain::services::MaskServiceImpl<
                            crate::infrastructure::repositories::MaskRepositoryImpl,
                            crate::infrastructure::repositories::MaskGroupRepositoryImpl,
                            crate::infrastructure::repositories::UserRepositoryImpl,
                        >,
                        crate::domain::services::MaskGroupServiceImpl<
                            crate::infrastructure::repositories::MaskGroupRepositoryImpl,
                            crate::infrastructure::repositories::AnnotationRepositoryImpl,
                            crate::infrastructure::repositories::UserRepositoryImpl,
                        >,
                        crate::application::services::SignedUrlServiceImpl,
                    >),
                )
                .route(
                    "/{annotation_id}/mask-groups/{group_id}/masks/stats",
                    web::get().to(crate::presentation::controllers::mask_controller::get_mask_stats::<
                        crate::domain::services::MaskServiceImpl<
                            crate::infrastructure::repositories::MaskRepositoryImpl,
                            crate::infrastructure::repositories::MaskGroupRepositoryImpl,
                            crate::infrastructure::repositories::UserRepositoryImpl,
                        >,
                        crate::domain::services::MaskGroupServiceImpl<
                            crate::infrastructure::repositories::MaskGroupRepositoryImpl,
                            crate::infrastructure::repositories::AnnotationRepositoryImpl,
                            crate::infrastructure::repositories::UserRepositoryImpl,
                        >,
                        crate::application::services::SignedUrlServiceImpl,
                    >),
                ),
        );
}
