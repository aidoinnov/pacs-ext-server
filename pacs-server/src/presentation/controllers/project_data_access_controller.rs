#![allow(dead_code, unused_imports, unused_variables)]
use actix_web::{web, HttpResponse, Result};
use serde_json::json;
use std::sync::Arc;

use crate::application::dto::project_data_access_dto::*;
use crate::application::use_cases::ProjectDataAccessUseCase;
use crate::domain::ServiceError;

/// ServiceError를 HttpResponse로 변환하는 헬퍼 함수
fn handle_service_error(error: ServiceError) -> HttpResponse {
    match error {
        ServiceError::NotFound(msg) => HttpResponse::NotFound().json(json!({
            "error": "Not Found",
            "message": msg
        })),
        ServiceError::ValidationError(msg) => HttpResponse::BadRequest().json(json!({
            "error": "Validation Error",
            "message": msg
        })),
        ServiceError::Unauthorized(msg) => HttpResponse::Unauthorized().json(json!({
            "error": "Unauthorized",
            "message": msg
        })),
        ServiceError::AlreadyExists(msg) => HttpResponse::Conflict().json(json!({
            "error": "Already Exists",
            "message": msg
        })),
        ServiceError::DatabaseError(msg) => HttpResponse::InternalServerError().json(json!({
            "error": "Database Error",
            "message": msg
        })),
        _ => HttpResponse::InternalServerError().json(json!({
            "error": "Internal Server Error",
            "message": "An unexpected error occurred"
        })),
    }
}

/// 프로젝트 데이터 접근 매트릭스 조회
#[utoipa::path(
    get,
    path = "/api/projects/{project_id}/data-access/matrix",
    responses(
        (status = 200, description = "프로젝트 데이터 접근 매트릭스 조회 성공", body = ProjectDataAccessMatrixResponse),
        (status = 404, description = "프로젝트를 찾을 수 없음"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("project_id" = i32, Path, description = "프로젝트 ID"),
        ("page" = Option<i32>, Query, description = "페이지 번호 (기본값: 1)"),
        ("page_size" = Option<i32>, Query, description = "페이지 크기 (기본값: 20)"),
        ("search" = Option<String>, Query, description = "검색어 (Study UID, Patient ID, Patient Name)"),
        ("status" = Option<String>, Query, description = "상태 필터 (APPROVED, DENIED, PENDING)"),
        ("user_id" = Option<i32>, Query, description = "사용자 ID 필터")
    ),
    tag = "project-data-access"
)]
pub async fn get_project_data_access_matrix(
    path: web::Path<i32>,
    query: web::Query<GetProjectDataListRequest>,
    use_case: web::Data<Arc<ProjectDataAccessUseCase>>,
) -> Result<HttpResponse, actix_web::Error> {
    let project_id = path.into_inner();
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);
    let search = query.search.clone();
    let status = query.status.clone();
    let user_id = query.user_id;

    match use_case
        .get_project_data_access_matrix(project_id, page, page_size, search, status, user_id)
        .await
    {
        Ok(matrix) => Ok(HttpResponse::Ok().json(matrix)),
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// 프로젝트 데이터 생성
#[utoipa::path(
    post,
    path = "/api/projects/{project_id}/data",
    request_body = CreateProjectDataRequest,
    responses(
        (status = 201, description = "프로젝트 데이터 생성 성공", body = CreateProjectDataResponse),
        (status = 400, description = "잘못된 요청"),
        (status = 409, description = "이미 존재하는 Study"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("project_id" = i32, Path, description = "프로젝트 ID")
    ),
    tag = "project-data-access"
)]
pub async fn create_project_data(
    path: web::Path<i32>,
    request: web::Json<CreateProjectDataRequest>,
    use_case: web::Data<Arc<ProjectDataAccessUseCase>>,
) -> Result<HttpResponse, actix_web::Error> {
    let project_id = path.into_inner();

    match use_case
        .create_project_data(project_id, request.into_inner())
        .await
    {
        Ok(response) => Ok(HttpResponse::Created().json(response)),
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// 개별 접근 권한 수정
#[utoipa::path(
    put,
    path = "/api/projects/{project_id}/data/{data_id}/access/{user_id}",
    request_body = UpdateDataAccessRequest,
    responses(
        (status = 200, description = "접근 권한 수정 성공", body = UpdateDataAccessResponse),
        (status = 400, description = "잘못된 요청"),
        (status = 404, description = "데이터 또는 사용자를 찾을 수 없음"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("project_id" = i32, Path, description = "프로젝트 ID"),
        ("data_id" = i32, Path, description = "데이터 ID"),
        ("user_id" = i32, Path, description = "사용자 ID")
    ),
    tag = "project-data-access"
)]
pub async fn update_data_access(
    path: web::Path<(i32, i32, i32)>,
    request: web::Json<UpdateDataAccessRequest>,
    use_case: web::Data<Arc<ProjectDataAccessUseCase>>,
) -> Result<HttpResponse, actix_web::Error> {
    let (project_id, data_id, user_id) = path.into_inner();

    match use_case
        .update_data_access(data_id, user_id, request.into_inner())
        .await
    {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// 일괄 접근 권한 수정
#[utoipa::path(
    put,
    path = "/api/projects/{project_id}/data/{data_id}/access/batch",
    request_body = BatchUpdateDataAccessRequest,
    responses(
        (status = 200, description = "일괄 접근 권한 수정 성공", body = BatchUpdateDataAccessResponse),
        (status = 400, description = "잘못된 요청"),
        (status = 404, description = "데이터를 찾을 수 없음"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("project_id" = i32, Path, description = "프로젝트 ID"),
        ("data_id" = i32, Path, description = "데이터 ID")
    ),
    tag = "project-data-access"
)]
pub async fn batch_update_data_access(
    path: web::Path<(i32, i32)>,
    request: web::Json<BatchUpdateDataAccessRequest>,
    use_case: web::Data<Arc<ProjectDataAccessUseCase>>,
) -> Result<HttpResponse, actix_web::Error> {
    let (project_id, data_id) = path.into_inner();

    match use_case
        .batch_update_data_access(data_id, request.into_inner())
        .await
    {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// 접근 요청
#[utoipa::path(
    post,
    path = "/api/projects/{project_id}/data/{data_id}/access/request",
    responses(
        (status = 200, description = "접근 요청 성공", body = RequestDataAccessResponse),
        (status = 400, description = "잘못된 요청"),
        (status = 404, description = "데이터를 찾을 수 없음"),
        (status = 409, description = "이미 접근 요청이 존재함"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("project_id" = i32, Path, description = "프로젝트 ID"),
        ("data_id" = i32, Path, description = "데이터 ID")
    ),
    tag = "project-data-access"
)]
pub async fn request_data_access(
    path: web::Path<(i32, i32)>,
    use_case: web::Data<Arc<ProjectDataAccessUseCase>>,
) -> Result<HttpResponse, actix_web::Error> {
    let (project_id, data_id) = path.into_inner();
    // TODO: Get user_id from authentication context
    let user_id = 1; // Mock user ID

    match use_case.request_data_access(data_id, user_id).await {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// 상태별 접근 권한 조회
#[utoipa::path(
    get,
    path = "/api/data-access/status/{status}",
    responses(
        (status = 200, description = "상태별 접근 권한 조회 성공", body = Vec<DataAccessInfo>),
        (status = 400, description = "잘못된 상태 값"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("status" = String, Path, description = "접근 상태 (APPROVED, DENIED, PENDING)"),
        ("page" = Option<i32>, Query, description = "페이지 번호 (기본값: 1)"),
        ("page_size" = Option<i32>, Query, description = "페이지 크기 (기본값: 20)")
    ),
    tag = "project-data-access"
)]
pub async fn get_access_by_status(
    path: web::Path<String>,
    query: web::Query<GetProjectDataListRequest>,
    use_case: web::Data<Arc<ProjectDataAccessUseCase>>,
) -> Result<HttpResponse, actix_web::Error> {
    let status = path.into_inner();
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);

    match use_case.get_access_by_status(status, page, page_size).await {
        Ok(access_list) => Ok(HttpResponse::Ok().json(access_list)),
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// 사용자별 접근 권한 조회
#[utoipa::path(
    get,
    path = "/api/users/{user_id}/data-access",
    responses(
        (status = 200, description = "사용자별 접근 권한 조회 성공", body = Vec<DataAccessInfo>),
        (status = 404, description = "사용자를 찾을 수 없음"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("user_id" = i32, Path, description = "사용자 ID"),
        ("page" = Option<i32>, Query, description = "페이지 번호 (기본값: 1)"),
        ("page_size" = Option<i32>, Query, description = "페이지 크기 (기본값: 20)")
    ),
    tag = "project-data-access"
)]
pub async fn get_user_access_list(
    path: web::Path<i32>,
    query: web::Query<GetProjectDataListRequest>,
    use_case: web::Data<Arc<ProjectDataAccessUseCase>>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = path.into_inner();
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);

    match use_case
        .get_user_access_list(user_id, page, page_size)
        .await
    {
        Ok(access_list) => Ok(HttpResponse::Ok().json(access_list)),
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// 프로젝트 Study 목록 조회
#[utoipa::path(
    get,
    path = "/api/project-data/{project_id}/studies",
    responses(
        (status = 200, description = "Study 목록 조회 성공", body = GetProjectStudiesResponse),
        (status = 404, description = "프로젝트를 찾을 수 없음"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("project_id" = i32, Path, description = "프로젝트 ID"),
        ("page" = Option<i32>, Query, description = "페이지 번호 (기본값: 1)"),
        ("page_size" = Option<i32>, Query, description = "페이지 크기 (기본값: 20, 최대: 100)"),
        ("patient_id" = Option<String>, Query, description = "환자 ID 필터"),
        ("study_date_from" = Option<String>, Query, description = "Study 시작일 필터 (YYYY-MM-DD)"),
        ("study_date_to" = Option<String>, Query, description = "Study 종료일 필터 (YYYY-MM-DD)")
    ),
    tag = "project-data"
)]
pub async fn get_project_studies(
    path: web::Path<i32>,
    query: web::Query<GetProjectStudiesRequest>,
    use_case: web::Data<Arc<ProjectDataAccessUseCase>>,
) -> Result<HttpResponse, actix_web::Error> {
    let project_id = path.into_inner();
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20).min(100); // 최대 100개

    match use_case.get_studies(project_id, page, page_size).await {
        Ok((studies, total)) => {
            let total_pages = (total as f64 / page_size as f64).ceil() as i64;

            let studies_info: Vec<StudyInfo> = studies
                .into_iter()
                .map(|s| StudyInfo {
                    id: s.id,
                    study_uid: s.study_uid,
                    study_description: s.study_description,
                    patient_id: s.patient_id,
                    patient_name: s.patient_name,
                    patient_birth_date: s.patient_birth_date.map(|d| d.to_string()),
                    study_date: s.study_date.map(|d| d.to_string()),
                    created_at: s.created_at.to_rfc3339(),
                    updated_at: s.updated_at.to_rfc3339(),
                })
                .collect();

            let response = GetProjectStudiesResponse {
                success: true,
                studies: studies_info,
                pagination: PaginationInfo {
                    page,
                    page_size,
                    total_items: total,
                    total_pages,
                },
            };

            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// 프로젝트 Series 목록 조회 (Study별)
#[utoipa::path(
    get,
    path = "/api/project-data/{project_id}/studies/{study_id}/series",
    responses(
        (status = 200, description = "Series 목록 조회 성공", body = GetProjectSeriesResponse),
        (status = 404, description = "Study를 찾을 수 없음"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("project_id" = i32, Path, description = "프로젝트 ID"),
        ("study_id" = i32, Path, description = "Study ID")
    ),
    tag = "project-data"
)]
pub async fn get_study_series(
    path: web::Path<(i32, i32)>,
    use_case: web::Data<Arc<ProjectDataAccessUseCase>>,
) -> Result<HttpResponse, actix_web::Error> {
    let (project_id, study_id) = path.into_inner();

    // Study 정보 조회
    let study = match use_case.get_study(study_id).await {
        Ok(s) => s,
        Err(e) => return Ok(handle_service_error(e)),
    };

    // Series 목록 조회
    match use_case.get_series_by_study(study_id).await {
        Ok(series_list) => {
            let series_with_study: Vec<SeriesWithStudyInfo> = series_list
                .into_iter()
                .map(|s| SeriesWithStudyInfo {
                    study: StudyInfo {
                        id: study.id,
                        study_uid: study.study_uid.clone(),
                        study_description: study.study_description.clone(),
                        patient_id: study.patient_id.clone(),
                        patient_name: study.patient_name.clone(),
                        patient_birth_date: study.patient_birth_date.map(|d| d.to_string()),
                        study_date: study.study_date.map(|d| d.to_string()),
                        created_at: study.created_at.to_rfc3339(),
                        updated_at: study.updated_at.to_rfc3339(),
                    },
                    series: SeriesInfo {
                        id: s.id,
                        series_uid: s.series_uid,
                        series_description: s.series_description,
                        modality: s.modality,
                        series_number: s.series_number,
                        created_at: s.created_at.to_rfc3339(),
                    },
                    assigned_at: s.created_at.to_rfc3339(),
                })
                .collect();

            let total_count = series_with_study.len();

            let response = GetProjectSeriesResponse {
                success: true,
                series: series_with_study,
                pagination: PaginationInfo {
                    page: 1,
                    page_size: total_count as i32,
                    total_items: total_count as i64,
                    total_pages: 1,
                },
            };

            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// 프로젝트 Instance 목록 조회 (Series별)
#[utoipa::path(
    get,
    path = "/api/project-data/{project_id}/series/{series_id}/instances",
    responses(
        (status = 200, description = "Instance 목록 조회 성공", body = GetProjectInstancesResponse),
        (status = 404, description = "Series를 찾을 수 없음"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("project_id" = i32, Path, description = "프로젝트 ID"),
        ("series_id" = i32, Path, description = "Series ID")
    ),
    tag = "project-data"
)]
pub async fn get_series_instances(
    path: web::Path<(i32, i32)>,
    use_case: web::Data<Arc<ProjectDataAccessUseCase>>,
) -> Result<HttpResponse, actix_web::Error> {
    let (project_id, series_id) = path.into_inner();

    // Series 정보 조회
    let series = match use_case.get_series(series_id).await {
        Ok(s) => s,
        Err(e) => return Ok(handle_service_error(e)),
    };

    // Instance 목록 조회
    match use_case.get_instances_by_series(series_id).await {
        Ok(instance_list) => {
            let instances_with_series: Vec<InstanceWithSeriesInfo> = instance_list
                .into_iter()
                .map(|i| InstanceWithSeriesInfo {
                    series: SeriesInfo {
                        id: series.id,
                        series_uid: series.series_uid.clone(),
                        series_description: series.series_description.clone(),
                        modality: series.modality.clone(),
                        series_number: series.series_number,
                        created_at: series.created_at.to_rfc3339(),
                    },
                    instance: InstanceInfo {
                        id: i.id,
                        instance_uid: i.instance_uid,
                        sop_class_uid: i.sop_class_uid,
                        instance_number: i.instance_number,
                        created_at: i.created_at.to_rfc3339(),
                    },
                    assigned_at: i.created_at.to_rfc3339(),
                })
                .collect();

            let total_count = instances_with_series.len();

            let response = GetProjectInstancesResponse {
                success: true,
                instances: instances_with_series,
                pagination: PaginationInfo {
                    page: 1,
                    page_size: total_count as i32,
                    total_items: total_count as i64,
                    total_pages: 1,
                },
            };

            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// 프로젝트에 Series 할당
#[utoipa::path(
    post,
    path = "/api/projects/{project_id}/series/assign",
    request_body = AssignSeriesToProjectRequest,
    responses(
        (status = 200, description = "Series 할당 성공", body = AssignSeriesToProjectResponse),
        (status = 404, description = "프로젝트 또는 부모 Study를 찾을 수 없음"),
        (status = 409, description = "이미 할당된 Series"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("project_id" = i32, Path, description = "프로젝트 ID")
    ),
    tag = "project-data-assignment"
)]
pub async fn assign_series_to_project(
    path: web::Path<i32>,
    request: web::Json<AssignSeriesToProjectRequest>,
    use_case: web::Data<Arc<ProjectDataAccessUseCase>>,
) -> Result<HttpResponse, actix_web::Error> {
    let project_id = path.into_inner();

    match use_case
        .assign_series_to_project(project_id, request.into_inner())
        .await
    {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// 프로젝트에 Study 할당
#[utoipa::path(
    post,
    path = "/api/projects/{project_id}/studies/assign",
    request_body = AssignStudyToProjectRequest,
    responses(
        (status = 200, description = "Study 할당 성공", body = AssignStudyToProjectResponse),
        (status = 404, description = "프로젝트를 찾을 수 없음"),
        (status = 409, description = "이미 할당된 Study"),
        (status = 500, description = "서버 내부 오류")
    ),
    params(
        ("project_id" = i32, Path, description = "프로젝트 ID")
    ),
    tag = "project-data-assignment"
)]
pub async fn assign_study_to_project(
    path: web::Path<i32>,
    request: web::Json<AssignStudyToProjectRequest>,
    use_case: web::Data<Arc<ProjectDataAccessUseCase>>,
) -> Result<HttpResponse, actix_web::Error> {
    let project_id = path.into_inner();

    match use_case
        .assign_study_to_project(project_id, request.into_inner())
        .await
    {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(e) => Ok(handle_service_error(e)),
    }
}

/// 라우트 설정
pub fn configure_routes(cfg: &mut web::ServiceConfig, use_case: Arc<ProjectDataAccessUseCase>) {
    cfg.app_data(web::Data::new(use_case))
        .service(
            // 별도 scope 사용하여 경로 충돌 방지
            web::scope("/project-data")
                .route(
                    "/{project_id}/data-access/matrix",
                    web::get().to(get_project_data_access_matrix),
                )
                .route("/{project_id}/data", web::post().to(create_project_data))
                .route(
                    "/{project_id}/data/{data_id}/access/{user_id}",
                    web::put().to(update_data_access),
                )
                .route(
                    "/{project_id}/data/{data_id}/access/batch",
                    web::put().to(batch_update_data_access),
                )
                .route(
                    "/{project_id}/data/{data_id}/access/request",
                    web::post().to(request_data_access),
                )
                // Study/Series/Instance 목록 조회 엔드포인트
                .route("/{project_id}/studies", web::get().to(get_project_studies))
                .route(
                    "/{project_id}/studies/{study_id}/series",
                    web::get().to(get_study_series),
                )
                .route(
                    "/{project_id}/series/{series_id}/instances",
                    web::get().to(get_series_instances),
                ),
        )
        // 데이터 할당 API - scope 없이 직접 등록 (project_controller와 충돌 방지)
        .route(
            "/projects/{project_id}/series/assign",
            web::post().to(assign_series_to_project),
        )
        .route(
            "/projects/{project_id}/studies/assign",
            web::post().to(assign_study_to_project),
        )
        .service(
            web::scope("/data-access")
                .route("/status/{status}", web::get().to(get_access_by_status)),
        )
        .service(
            web::scope("/users/{user_id}")
                .route("/data-access", web::get().to(get_user_access_list)),
        );
}
