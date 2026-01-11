use actix_web::{web, HttpRequest, HttpResponse};
use serde_json::json;
use std::collections::HashSet;
use std::sync::Arc;

use crate::application::dto::viewer_dto::{
	    ViewerStudyMetaRequest, ViewerStudyMetaResponse, ViewerStudyMeta,
	    ViewerSeriesMetaRequest, ViewerSeriesMetaResponse, ViewerSeriesMeta,
	    ViewerStudySeriesMetaRequest, ViewerStudySeriesMetaResponse, ViewerPaginationInfo,
	};
use crate::infrastructure::auth::{JwtService, extract_user_id_from_request};
use crate::infrastructure::external::Dcm4cheeQidoClient;
use crate::infrastructure::repositories::{UserRepositoryImpl, ProjectDataRepositoryImpl};
use crate::infrastructure::services::DicomRbacEvaluatorImpl;
use crate::presentation::controllers::dicom_gateway_controller::{extract_bearer_token, can_access_study};

/// POST /api/v1/viewer/studies/meta
/// 
/// Viewer에서 선택된 Study들의 메타데이터를 한 번에 조회하는 Batch API
/// 
/// ## 기능
/// - 여러 StudyInstanceUID에 대한 메타데이터를 한 번의 요청으로 조회
/// - RBAC 기반 접근 제어 적용
/// - 기존 QIDO Proxy 재사용 (BFF 패턴)
/// 
/// ## Request Body
/// ```json
/// {
///   "study_uids": ["1.2.840.113619.2.55.3.604688433.1234", ...],
///   "max_count": 20
/// }
/// ```
#[utoipa::path(
    post,
    path = "/api/v1/viewer/studies/meta",
    tag = "viewer",
    request_body = ViewerStudyMetaRequest,
    responses(
        (status = 200, description = "Study 메타데이터 조회 성공", body = ViewerStudyMetaResponse),
        (status = 400, description = "잘못된 요청"),
        (status = 401, description = "인증 실패"),
        (status = 403, description = "접근 권한 없음"),
    )
)]
pub async fn get_studies_meta(
    request: web::Json<ViewerStudyMetaRequest>,
    req: HttpRequest,
    qido: web::Data<Dcm4cheeQidoClient>,
    jwt: web::Data<Arc<JwtService>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
    rbac_evaluator: web::Data<Arc<DicomRbacEvaluatorImpl>>,
    project_data_repo: web::Data<Arc<ProjectDataRepositoryImpl>>,
) -> HttpResponse {
    // 1. 사용자 인증
    tracing::info!("Viewer /studies/meta: called");
    println!("Viewer /studies/meta: called");

    let user_id = match extract_user_id_from_request(&req, &jwt, &user_repo).await {
        Some(id) if id > 0 => id,
        _ => {
            tracing::warn!("Viewer /studies/meta: Unauthorized - failed to extract user_id");
            return HttpResponse::Unauthorized().json(json!({
                "error": "UNAUTHORIZED",
                "message": "Invalid or missing authorization token"
            }));
        }
    };

    let request = request.into_inner();

    // 2. 요청 검증
    if request.study_uids.is_empty() {
        return HttpResponse::BadRequest().json(json!({
            "error": "INVALID_REQUEST",
            "message": "study_uids is required and must be a non-empty array"
        }));
    }

    // 페이지네이션 파라미터 처리
    let page = request.page.unwrap_or(1).max(1);
    let page_size = request.page_size.unwrap_or(50).clamp(1, 200);

    tracing::info!(
        "Viewer /studies/meta: user_id={}, study_count={}, page={}, page_size={}",
        user_id,
        request.study_uids.len(),
        page,
        page_size
    );

    // 3. Bearer 토큰 추출
    let bearer_opt = extract_bearer_token(&req);
    if let Some(ref token) = bearer_opt {
        tracing::debug!("Viewer /studies/meta: Using Bearer token (length: {})", token.len());
    } else {
        tracing::warn!("Viewer /studies/meta: No Bearer token found");
    }

    // 4. 각 Study에 대해 QIDO 호출 및 RBAC 검증
    let mut all_studies: Vec<ViewerStudyMeta> = Vec::new();

    for study_uid in request.study_uids {
        // QIDO 호출: GET /rs/studies/{studyUID}
        match qido
            .qido_study_by_uid_with_bearer(bearer_opt.as_deref(), &study_uid)
            .await
        {
            Ok(json) => {
                // QIDO는 단일 Study를 배열로 반환할 수 있음
                let study_json = if let Some(array) = json.as_array() {
                    if let Some(first) = array.first() {
                        first
                    } else {
                        tracing::warn!("Viewer /studies/meta: Study {} not found in QIDO", study_uid);
                        continue;
                    }
                } else {
                    &json
                };

                // RBAC 검증: 사용자가 이 Study에 접근 가능한지 확인
                // 사용자가 속한 모든 프로젝트에 대해 확인
                let user_projects = match rbac_evaluator.get_user_project_ids(user_id).await {
                    Ok(projects) => projects,
                    Err(e) => {
                        tracing::error!("Viewer /studies/meta: Failed to get user projects: {:?}", e);
                        continue;
                    }
                };

                let mut has_access = false;
                for project_id in user_projects {
                    // RBAC 평가
                    use crate::domain::services::DicomRbacEvaluator;
                    let rbac_result = (**rbac_evaluator)
                        .evaluate_study_uid(user_id, project_id, &study_uid)
                        .await;

                    // project_data_access 확인
                    let data_access = can_access_study(
                        user_id,
                        project_id,
                        &study_uid,
                        project_data_repo.pool(),
                    )
                    .await;

                    if rbac_result.allowed && data_access {
                        has_access = true;
                        break;
                    }
                }

                if !has_access {
                    tracing::warn!(
                        "Viewer /studies/meta: User {} has no access to study {}",
                        user_id,
                        study_uid
                    );
                    continue;
                }

                // ViewerStudyMeta로 변환
                let study_meta = ViewerStudyMeta::from_dicomweb_json(study_json);
                all_studies.push(study_meta);
            }
            Err(e) => {
                tracing::error!(
                    "Viewer /studies/meta: Failed to fetch study {}: {:?}",
                    study_uid,
                    e
                );
                // 개별 Study 실패는 무시하고 계속 진행
            }
        }
    }

    let total_items = all_studies.len();

    // 5. 응답 반환
    if total_items == 0 {
        return HttpResponse::NotFound().json(json!({
            "error": "NOT_FOUND",
            "message": "No accessible studies found"
        }));
    }

    // 6. 페이지네이션 적용
    let total_pages = ((total_items as f64) / (page_size as f64)).ceil() as i32;
    let offset = ((page - 1) * page_size) as usize;
    let end = (offset + page_size as usize).min(total_items);

    let paginated_studies = if offset < total_items {
        all_studies[offset..end].to_vec()
    } else {
        vec![]
    };

    tracing::info!(
        "Viewer /studies/meta: Found {} studies, page={}, page_size={}, returning {} studies",
        total_items,
        page,
        page_size,
        paginated_studies.len()
    );

    // 7. 페이지네이션 정보 생성
    let pagination = ViewerPaginationInfo {
        page,
        page_size,
        total_items: total_items as i32,
        total_pages,
        has_next: page < total_pages,
        has_previous: page > 1,
    };

    HttpResponse::Ok().json(ViewerStudyMetaResponse {
        studies: paginated_studies,
        pagination,
    })
}

/// POST /api/v1/viewer/series/meta
///
/// Viewer에서 선택된 Series들의 메타데이터를 한 번에 조회하는 Batch API
///
/// ## 기능
/// - Study-Series 쌍에 대한 메타데이터를 한 번의 요청으로 조회
/// - RBAC 기반 접근 제어 적용
/// - 기존 QIDO Proxy 재사용 (BFF 패턴)
/// - Study Description 포함
///
/// ## Request Body
/// ```json
/// {
///   "series_queries": [
///     {"study_uid": "1.2.840...", "series_uid": "1.2.840...1"},
///     {"study_uid": "1.2.840...", "series_uid": "1.2.840...2"}
///   ],
///   "max_count": 50
/// }
/// ```
#[utoipa::path(
    post,
    path = "/api/v1/viewer/series/meta",
    tag = "viewer",
    request_body = ViewerSeriesMetaRequest,
    responses(
        (status = 200, description = "Series 메타데이터 조회 성공", body = ViewerSeriesMetaResponse),
        (status = 400, description = "잘못된 요청"),
        (status = 401, description = "인증 실패"),
        (status = 403, description = "접근 권한 없음"),
    )
)]
pub async fn get_series_meta(
    request: web::Json<ViewerSeriesMetaRequest>,
    req: HttpRequest,
    qido: web::Data<Dcm4cheeQidoClient>,
    jwt: web::Data<Arc<JwtService>>,
    user_repo: web::Data<Arc<UserRepositoryImpl>>,
    rbac_evaluator: web::Data<Arc<DicomRbacEvaluatorImpl>>,
    project_data_repo: web::Data<Arc<ProjectDataRepositoryImpl>>,
) -> HttpResponse {
    // 1. 사용자 인증
    let user_id = match extract_user_id_from_request(&req, &jwt, &user_repo).await {
        Some(id) if id > 0 => id,
        _ => {
            tracing::warn!("Viewer /series/meta: Unauthorized - failed to extract user_id");
            return HttpResponse::Unauthorized().json(json!({
                "error": "UNAUTHORIZED",
                "message": "Invalid or missing authorization token"
            }));
        }
    };

    let request_inner = request.into_inner();

    // 2. Request 검증
    if request_inner.series_queries.is_empty() {
        return HttpResponse::BadRequest().json(json!({
            "error": "INVALID_REQUEST",
            "message": "series_queries is required and must be a non-empty array"
        }));
    }

    // 페이지네이션 파라미터 처리
    let page = request_inner.page.unwrap_or(1).max(1);
    let page_size = request_inner.page_size.unwrap_or(50).clamp(1, 200);

    // 중복 제거 (study_uid + series_uid 조합으로)
    let mut seen = std::collections::HashSet::new();
    let series_queries: Vec<_> = request_inner
        .series_queries
        .iter()
        .filter(|q| {
            let key = format!("{}|{}", q.study_uid.trim(), q.series_uid.trim());
            !q.study_uid.trim().is_empty()
                && !q.series_uid.trim().is_empty()
                && seen.insert(key)
        })
        .cloned()
        .collect();

    tracing::info!(
        "Viewer /series/meta: user_id={}, series_count={}, page={}, page_size={}",
        user_id,
        series_queries.len(),
        page,
        page_size
    );

    // 3. Bearer 토큰 추출
    let bearer_token = match extract_bearer_token(&req) {
        Some(token) => token,
        None => {
            tracing::warn!("Viewer /series/meta: No bearer token found");
            return HttpResponse::Unauthorized().json(json!({
                "error": "UNAUTHORIZED",
                "message": "Missing authorization token"
            }));
        }
    };

    // 4. 각 Study-Series 쌍에 대해 메타데이터 조회 및 권한 검증
    let mut all_series = Vec::new();

    for query in series_queries {
        let study_uid = query.study_uid.clone();
        let series_uid = query.series_uid.clone();

        // 4.1. RBAC 검증: 사용자가 이 Study에 접근 가능한지 확인
        let user_projects = match rbac_evaluator.get_user_project_ids(user_id).await {
            Ok(projects) => projects,
            Err(e) => {
                tracing::error!("Viewer /series/meta: Failed to get user projects: {:?}", e);
                continue;
            }
        };

        let mut has_access = false;
        for project_id in user_projects {
            // RBAC 평가
            use crate::domain::services::DicomRbacEvaluator;
            let rbac_result = (**rbac_evaluator)
                .evaluate_study_uid(user_id, project_id, &study_uid)
                .await;

            // project_data_access 확인
            let data_access = can_access_study(
                user_id,
                project_id,
                &study_uid,
                project_data_repo.pool(),
            )
            .await;

            if rbac_result.allowed && data_access {
                has_access = true;
                break;
            }
        }

        if !has_access {
            tracing::warn!(
                "Viewer /series/meta: User {} has no access to study {}",
                user_id,
                study_uid
            );
            continue;
        }

        // 4.2. QIDO-RS로 Series 메타데이터 조회
        let params = vec![
            ("SeriesInstanceUID".to_string(), series_uid.clone()),
        ];

        let qido_response = match qido
            .qido_series_all_with_bearer(Some(&bearer_token), params)
            .await
        {
            Ok(json) => json,
            Err(e) => {
                tracing::warn!(
                    "Viewer /series/meta: QIDO query failed for series={}: {:?}",
                    series_uid,
                    e
                );
                continue;
            }
        };

        let series_json = match qido_response.as_array() {
            Some(array) if !array.is_empty() => &array[0],
            _ => {
                tracing::debug!(
                    "Viewer /series/meta: No series found for series={}",
                    series_uid
                );
                continue;
            }
        };

        // 4.3. DTO 변환
        let mut series_meta = ViewerSeriesMeta::from_dicomweb_json(series_json);

        // 4.4. StudyDescription 설정
        // 클라이언트가 전달한 값이 있으면 사용, 없으면 QIDO 응답에서 파싱한 값 사용
        if let Some(client_study_desc) = &query.study_description {
            series_meta.study_description = Some(client_study_desc.clone());
        }
        // QIDO 응답에 StudyDescription이 없고 클라이언트도 전달하지 않았다면
        // 별도로 Study 조회 (선택사항)
        else if series_meta.study_description.is_none() {
            // Study 조회하여 Description 가져오기
            let study_params = vec![("StudyInstanceUID".to_string(), study_uid.clone())];
            if let Ok(study_response) = qido.qido_studies_with_bearer(Some(&bearer_token), study_params).await {
                if let Some(study_array) = study_response.as_array() {
                    if let Some(study_json) = study_array.first() {
                        if let Some(desc) = study_json.get("00081030")
                            .and_then(|v| v.get("Value"))
                            .and_then(|v| v.get(0))
                            .and_then(|v| v.as_str())
                        {
                            series_meta.study_description = Some(desc.to_string());
                        }
                    }
                }
            }
        }

        all_series.push(series_meta);
    }

    let total_items = all_series.len();

    // 5. 응답 반환
    if total_items == 0 {
        return HttpResponse::NotFound().json(json!({
            "error": "NOT_FOUND",
            "message": "No accessible series found"
        }));
    }

    // 6. 페이지네이션 적용
    let total_pages = ((total_items as f64) / (page_size as f64)).ceil() as i32;
    let offset = ((page - 1) * page_size) as usize;
    let end = (offset + page_size as usize).min(total_items);

    let paginated_series = if offset < total_items {
        all_series[offset..end].to_vec()
    } else {
        vec![]
    };

    tracing::info!(
        "Viewer /series/meta: Found {} series, page={}, page_size={}, returning {} series",
        total_items,
        page,
        page_size,
        paginated_series.len()
    );

    // 7. 페이지네이션 정보 생성
    let pagination = ViewerPaginationInfo {
        page,
        page_size,
        total_items: total_items as i32,
        total_pages,
        has_next: page < total_pages,
        has_previous: page > 1,
    };

    HttpResponse::Ok().json(ViewerSeriesMetaResponse {
        series: paginated_series,
        pagination,
    })
}

/// POST /api/v1/viewer/studies/{study_uid}/series/meta
///
/// 특정 Study의 Series 메타데이터를 조회 (페이지네이션 지원)
///
/// ## 기능
/// - Study UID로 해당 Study의 Series 메타데이터 조회
/// - Request Body로 `series_uids`를 전달하면 해당 UID들만 필터링하여 조회
///   (지정하지 않으면 Study 내 모든 Series 대상)
/// - RBAC 기반 접근 제어 적용
/// - Study Description 자동 포함
/// - 페이지네이션 지원 (page, page_size)
///
/// ## Path Parameters
/// - study_uid: StudyInstanceUID
///
/// ## Request Body (ViewerStudySeriesMetaRequest)
/// - series_uids: 선택적으로 특정 SeriesInstanceUID 목록을 지정
/// - page: 페이지 번호 (1부터 시작, 기본값: 1)
/// - page_size: 페이지 크기 (기본값: 50, 최대: 200)
#[utoipa::path(
	    post,
	    path = "/api/v1/viewer/studies/{study_uid}/series/meta",
	    tag = "viewer",
	    request_body = ViewerStudySeriesMetaRequest,
	    params(
	        ("study_uid" = String, Path, description = "StudyInstanceUID")
	    ),
	    responses(
	        (status = 200, description = "Series 메타데이터 조회 성공", body = ViewerStudySeriesMetaResponse),
	        (status = 400, description = "잘못된 요청"),
	        (status = 401, description = "인증 실패"),
	        (status = 403, description = "접근 권한 없음"),
	        (status = 404, description = "Study 또는 Series를 찾을 수 없음"),
	    )
	)]
	pub async fn get_study_series_meta(
	    study_uid: web::Path<String>,
	    body: web::Json<ViewerStudySeriesMetaRequest>,
	    req: HttpRequest,
	    qido: web::Data<Dcm4cheeQidoClient>,
	    jwt: web::Data<Arc<JwtService>>,
	    user_repo: web::Data<Arc<UserRepositoryImpl>>,
	    rbac_evaluator: web::Data<Arc<DicomRbacEvaluatorImpl>>,
	    project_data_repo: web::Data<Arc<ProjectDataRepositoryImpl>>,
	) -> HttpResponse {
	    let study_uid = study_uid.into_inner();
	    let request = body.into_inner();

	    // 페이지네이션 파라미터 및 선택적 series_uids 필터 처리
	    let page = request.page.unwrap_or(1).max(1);
	    let page_size = request.page_size.unwrap_or(50).clamp(1, 200);
	    let filter_series_uids: Option<HashSet<String>> = request
	        .series_uids
	        .as_ref()
	        .map(|uids| uids.iter().cloned().collect());

	    tracing::info!("Viewer /studies/{}/series/meta: Request received", study_uid);

    // 1. Bearer Token 추출
    let bearer_token = match extract_bearer_token(&req) {
        Some(token) => token,
        None => {
            tracing::warn!("Viewer /studies/{}/series/meta: No bearer token", study_uid);
            return HttpResponse::Unauthorized().json(json!({
                "error": "UNAUTHORIZED",
                "message": "Bearer token required"
            }));
        }
    };

    // 2. User ID 추출
    let user_id = match extract_user_id_from_request(&req, &jwt, &user_repo).await {
        Some(id) if id > 0 => id,
        _ => {
            tracing::warn!("Viewer /studies/{}/series/meta: Failed to extract user_id", study_uid);
            return HttpResponse::Unauthorized().json(json!({
                "error": "UNAUTHORIZED",
                "message": "Invalid token"
            }));
        }
    };

    tracing::debug!("Viewer /studies/{}/series/meta: user_id={}", study_uid, user_id);
    tracing::debug!(
        "Viewer /studies/{}/series/meta: page={}, page_size={}, filter_series_uids_count={}",
        study_uid,
        page,
        page_size,
        request.series_uids.as_ref().map(|v| v.len()).unwrap_or(0)
    );

    // 3. RBAC 체크 - 사용자가 속한 모든 프로젝트에 대해 확인
    let user_projects = match rbac_evaluator.get_user_project_ids(user_id).await {
        Ok(projects) => projects,
        Err(e) => {
            tracing::error!("Viewer /studies/{}/series/meta: Failed to get user projects: {:?}", study_uid, e);
            return HttpResponse::InternalServerError().json(json!({
                "error": "INTERNAL_ERROR",
                "message": "Failed to verify access permissions"
            }));
        }
    };

    let mut has_access = false;
    for project_id in user_projects {
        // RBAC 평가
        use crate::domain::services::DicomRbacEvaluator;
        let rbac_result = (**rbac_evaluator)
            .evaluate_study_uid(user_id, project_id, &study_uid)
            .await;

        // project_data_access 확인
        let data_access = can_access_study(
            user_id,
            project_id,
            &study_uid,
            project_data_repo.pool(),
        )
        .await;

        if rbac_result.allowed && data_access {
            has_access = true;
            break;
        }
    }

    if !has_access {
        tracing::warn!(
            "Viewer /studies/{}/series/meta: Access denied for user_id={}",
            study_uid,
            user_id
        );
        return HttpResponse::Forbidden().json(json!({
            "error": "FORBIDDEN",
            "message": "Access denied to this study"
        }));
    }

    // 4. Study 메타데이터 조회 (StudyDescription 가져오기)
    let study_params = vec![("StudyInstanceUID".to_string(), study_uid.clone())];

    let study_description = match qido
        .qido_studies_with_bearer(Some(&bearer_token), study_params)
        .await
    {
        Ok(study_response) => {
            if let Some(study_array) = study_response.as_array() {
                if let Some(study_json) = study_array.first() {
                    study_json
                        .get("00081030")
                        .and_then(|v| v.get("Value"))
                        .and_then(|v| v.get(0))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                } else {
                    None
                }
            } else {
                None
            }
        }
        Err(e) => {
            tracing::warn!(
                "Viewer /studies/{}/series/meta: Failed to fetch study metadata: {:?}",
                study_uid,
                e
            );
            None
        }
    };

    // 5. Series 목록 조회 (Study 내 모든 Series 조회 후, 필요 시 series_uids로 필터링)
    let series_params = vec![];

    let qido_response = match qido
        .qido_series_with_bearer(Some(&bearer_token), &study_uid, series_params)
        .await
    {
        Ok(json) => json,
        Err(e) => {
            tracing::error!(
                "Viewer /studies/{}/series/meta: QIDO query failed: {:?}",
                study_uid,
                e
            );
            return HttpResponse::InternalServerError().json(json!({
                "error": "QIDO_ERROR",
                "message": format!("Failed to query series: {}", e)
            }));
        }
    };

    let series_array = match qido_response.as_array() {
        Some(array) => array,
        None => {
            tracing::warn!(
                "Viewer /studies/{}/series/meta: Invalid QIDO response format",
                study_uid
            );
            return HttpResponse::InternalServerError().json(json!({
                "error": "INVALID_RESPONSE",
                "message": "Invalid QIDO response format"
            }));
        }
    };

    // 6. Series 메타데이터 변환
    let mut all_series: Vec<ViewerSeriesMeta> = Vec::new();

    for series_json in series_array {
        let mut series_meta = ViewerSeriesMeta::from_dicomweb_json(series_json);

        // Study Description 설정
        if series_meta.study_description.is_none() {
            series_meta.study_description = study_description.clone();
        }

        // series_uids 필터가 있으면 해당 UID만 포함
        if let Some(ref filter_set) = filter_series_uids {
            if !filter_set.contains(&series_meta.series_uid) {
                continue;
            }
        }

        all_series.push(series_meta);
    }

    let total_items = all_series.len();

    if total_items == 0 {
        return HttpResponse::NotFound().json(json!({
            "error": "NOT_FOUND",
            "message": "No series found for this study"
        }));
    }

    let total_pages = ((total_items as f64) / (page_size as f64)).ceil() as i32;

    // 7. 페이지네이션 적용
    let offset = ((page - 1) * page_size) as usize;
    let end = (offset + page_size as usize).min(total_items);

    let paginated_series = if offset < total_items {
        all_series[offset..end].to_vec()
    } else {
        vec![]
    };

    tracing::info!(
        "Viewer /studies/{}/series/meta: Found {} series, page={}, page_size={}, returning {} series",
        study_uid,
        total_items,
        page,
        page_size,
        paginated_series.len()
    );

    // 9. 페이지네이션 정보 생성
    let pagination = ViewerPaginationInfo {
        page,
        page_size,
        total_items: total_items as i32,
        total_pages,
        has_next: page < total_pages,
        has_previous: page > 1,
    };

    HttpResponse::Ok().json(ViewerStudySeriesMetaResponse {
        study_uid,
        study_description,
        series: paginated_series,
        pagination,
    })
}
