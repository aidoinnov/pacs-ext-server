/**
 * 테스트 API 컨트롤러
 * 
 * 기능:
 * - Project Data Access 시나리오 구성
 * - 시나리오 초기화
 */

use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;
use crate::infrastructure::external::KeycloakClient;

#[derive(Debug, Serialize)]
pub struct SetupResponse {
    pub project_id: i32,
    pub users: Vec<UserInfo>,
    pub studies: Vec<StudyInfo>,
    pub access_records: i32,
}

#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: i32,
    pub username: String,
    pub full_name: String,
}

#[derive(Debug, Serialize)]
pub struct StudyInfo {
    pub id: i32,
    pub study_uid: String,
    pub study_description: String,
}

/// Project Data Access 시나리오 구성
pub async fn setup_project_data_access_scenario(
    pool: web::Data<Arc<PgPool>>,
) -> impl Responder {
    // 1. 프로젝트 생성
    let project_id = match sqlx::query_scalar::<_, i32>(
        "INSERT INTO security_project (name, description, status, is_active)
         VALUES ($1, $2, $3::project_status, $4)
         ON CONFLICT (name) DO UPDATE SET
            description = EXCLUDED.description,
            status = EXCLUDED.status
         RETURNING id"
    )
    .bind("심장질환 공동 연구")
    .bind("다기관 공동 연구 프로젝트")
    .bind("IN_PROGRESS")
    .bind(true)
    .fetch_one(pool.as_ref().as_ref())
    .await
    {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("프로젝트 생성 실패: {}", e)
            }));
        }
    };

    // 2. 사용자 생성
    let users = vec![
        ("dr_kim", "Dr. Kim (책임연구원)", "dr.kim@hospital.com"),
        ("dr_lee", "Dr. Lee (A병원)", "dr.lee@hospital-a.com"),
        ("dr_park", "Dr. Park (B병원)", "dr.park@hospital-b.com"),
        ("dr_choi", "Dr. Choi (임시 협력자)", "dr.choi@temp.com"),
    ];

    let mut user_ids = Vec::new();
    let mut user_infos = Vec::new();

    for (username, full_name, email) in users {
        let keycloak_id = Uuid::new_v4();
        
        match sqlx::query_scalar::<_, i32>(
            "INSERT INTO security_user (username, email, full_name, keycloak_id, account_status)
             VALUES ($1, $2, $3, $4, 'ACTIVE'::user_account_status_enum)
             ON CONFLICT (username) DO UPDATE SET
                email = EXCLUDED.email,
                full_name = EXCLUDED.full_name
             RETURNING id"
        )
        .bind(username)
        .bind(email)
        .bind(full_name)
        .bind(keycloak_id)
        .fetch_one(pool.as_ref().as_ref())
        .await
        {
            Ok(id) => {
                user_ids.push(id);
                user_infos.push(UserInfo {
                    id,
                    username: username.to_string(),
                    full_name: full_name.to_string(),
                });
            }
            Err(e) => {
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("사용자 생성 실패: {}", e)
                }));
            }
        }
    }

    // 3. 프로젝트에 사용자 할당
    for user_id in &user_ids {
        if let Err(e) = sqlx::query(
            "INSERT INTO security_user_project (user_id, project_id)
             VALUES ($1, $2)
             ON CONFLICT (user_id, project_id) DO NOTHING"
        )
        .bind(user_id)
        .bind(project_id)
        .execute(pool.as_ref().as_ref())
        .await
        {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("프로젝트 사용자 할당 실패: {}", e)
            }));
        }
    }

    // 4. Study 데이터 생성
    let studies = vec![
        ("1.2.840.113619.2.55.3.A.1", "CT Chest - A병원 환자1", "A-P001", "김철수", "2025-01-10"),
        ("1.2.840.113619.2.55.3.A.2", "MRI Brain - A병원 환자2", "A-P002", "이영희", "2025-01-11"),
        ("1.2.840.113619.2.55.3.A.3", "CT Abdomen - A병원 환자3", "A-P003", "박민수", "2025-01-12"),
        ("1.2.840.113619.2.55.3.B.1", "CT Chest - B병원 환자1", "B-P001", "최지훈", "2025-01-13"),
        ("1.2.840.113619.2.55.3.B.2", "MRI Spine - B병원 환자2", "B-P002", "정수진", "2025-01-14"),
        ("1.2.840.113619.2.55.3.B.3", "CT Heart - B병원 환자3", "B-P003", "강민호", "2025-01-15"),
        ("1.2.840.113619.2.55.3.VIP.1", "CT Full Body - VIP 환자", "VIP-001", "VIP 환자", "2025-01-16"),
    ];

    let mut study_ids = Vec::new();
    let mut study_infos = Vec::new();

    for (study_uid, study_description, patient_id, patient_name, study_date) in studies {
        match sqlx::query_scalar::<_, i32>(
            "INSERT INTO project_data_study (study_uid, study_description, patient_id, patient_name, study_date)
             VALUES ($1, $2, $3, $4, $5::date)
             ON CONFLICT (study_uid) DO UPDATE SET
                study_description = EXCLUDED.study_description
             RETURNING id"
        )
        .bind(study_uid)
        .bind(study_description)
        .bind(patient_id)
        .bind(patient_name)
        .bind(study_date)
        .fetch_one(pool.as_ref().as_ref())
        .await
        {
            Ok(id) => {
                study_ids.push(id);
                study_infos.push(StudyInfo {
                    id,
                    study_uid: study_uid.to_string(),
                    study_description: study_description.to_string(),
                });
            }
            Err(e) => {
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Study 생성 실패: {}", e)
                }));
            }
        }
    }

    // 5. 프로젝트에 Study 할당
    for study_id in &study_ids {
        if let Err(e) = sqlx::query(
            "INSERT INTO project_data (project_id, resource_level, study_id)
             VALUES ($1, 'STUDY'::resource_level_enum, $2)
             ON CONFLICT (project_id, study_id, series_id, instance_id) DO NOTHING"
        )
        .bind(project_id)
        .bind(study_id)
        .execute(pool.as_ref().as_ref())
        .await
        {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("프로젝트 Study 할당 실패: {}", e)
            }));
        }
    }

    // 6. 접근 제어 설정
    // Dr. Kim (user_ids[0]): 레코드 없음 → 전체 접근
    // Dr. Lee (user_ids[1]): A병원 Study만 (study_ids[0,1,2])
    // Dr. Park (user_ids[2]): B병원 Study만 (study_ids[3,4,5])
    // Dr. Choi (user_ids[3]): Study 1개만 7일간 읽기 전용 (study_ids[0])

    let mut access_records = 0;

    // Dr. Lee: A병원 Study
    for i in 0..3 {
        if let Err(e) = sqlx::query(
            "INSERT INTO project_data_access (user_id, project_id, resource_level, study_id, status, access_scope)
             VALUES ($1, $2, 'STUDY'::resource_level_enum, $3, 'APPROVED'::data_access_status_enum, 'FULL')
             ON CONFLICT (project_id, user_id, study_id, series_id, instance_id) DO NOTHING"
        )
        .bind(user_ids[1])
        .bind(project_id)
        .bind(study_ids[i])
        .execute(pool.as_ref().as_ref())
        .await
        {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("접근 제어 설정 실패: {}", e)
            }));
        }
        access_records += 1;
    }

    // Dr. Park: B병원 Study
    for i in 3..6 {
        if let Err(e) = sqlx::query(
            "INSERT INTO project_data_access (user_id, project_id, resource_level, study_id, status, access_scope)
             VALUES ($1, $2, 'STUDY'::resource_level_enum, $3, 'APPROVED'::data_access_status_enum, 'FULL')
             ON CONFLICT (project_id, user_id, study_id, series_id, instance_id) DO NOTHING"
        )
        .bind(user_ids[2])
        .bind(project_id)
        .bind(study_ids[i])
        .execute(pool.as_ref().as_ref())
        .await
        {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("접근 제어 설정 실패: {}", e)
            }));
        }
        access_records += 1;
    }

    // Dr. Choi: Study 1개만 7일간 읽기 전용
    if let Err(e) = sqlx::query(
        "INSERT INTO project_data_access (user_id, project_id, resource_level, study_id, status, access_scope, expires_at)
         VALUES ($1, $2, 'STUDY'::resource_level_enum, $3, 'APPROVED'::data_access_status_enum, 'READ_ONLY', NOW() + INTERVAL '7 days')
         ON CONFLICT (project_id, user_id, study_id, series_id, instance_id) DO NOTHING"
    )
    .bind(user_ids[3])
    .bind(project_id)
    .bind(study_ids[0])
    .execute(pool.as_ref().as_ref())
    .await
    {
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("접근 제어 설정 실패: {}", e)
        }));
    }
    access_records += 1;

    HttpResponse::Ok().json(SetupResponse {
        project_id,
        users: user_infos,
        studies: study_infos,
        access_records,
    })
}

/// 시나리오 초기화
pub async fn cleanup_project_data_access_scenario(
    pool: web::Data<Arc<PgPool>>,
    project_id: web::Path<i32>,
) -> impl Responder {
    let project_id = project_id.into_inner();

    // 프로젝트 삭제 (CASCADE로 관련 데이터 모두 삭제)
    match sqlx::query("DELETE FROM security_project WHERE id = $1")
        .bind(project_id)
        .execute(pool.as_ref().as_ref())
        .await
    {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "message": "시나리오 초기화 완료"
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("초기화 실패: {}", e)
        })),
    }
}

/// Series API 테스트 Setup 응답
#[derive(Debug, Serialize)]
pub struct SeriesTestSetupResponse {
    pub project_id: i32,
    pub project_name: String,
    pub study_id: i32,
    pub study_uid: String,
    pub patient_id: String,
    pub series_count: i32,
    pub series_uids: Vec<String>,
}

/// Series API 테스트 시나리오 구성
///
/// 구성 내용:
/// 1. 테스트용 프로젝트 생성
/// 2. Study 데이터 생성 (1개)
/// 3. Series 데이터 생성 (3개 - CT, MRI, XR)
/// 4. 프로젝트에 Study 할당
/// 5. Study에 Series 할당
pub async fn setup_series_api_scenario(
    pool: web::Data<Arc<PgPool>>,
) -> impl Responder {
    let timestamp = chrono::Utc::now().timestamp();
    let project_name = format!("Series API Test {}", timestamp);

    // 1. 프로젝트 생성
    let project_id = match sqlx::query_scalar::<_, i32>(
        "INSERT INTO security_project (name, description, status, is_active)
         VALUES ($1, $2, $3::project_status, $4)
         RETURNING id"
    )
    .bind(&project_name)
    .bind("Series API 테스트용 프로젝트")
    .bind("IN_PROGRESS")
    .bind(true)
    .fetch_one(pool.as_ref().as_ref())
    .await
    {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("프로젝트 생성 실패: {}", e)
            }));
        }
    };

    // 2. Study 데이터 생성
    let study_uid = format!("1.2.840.113619.2.55.3.SERIES_TEST.{}", timestamp);
    let patient_id = format!("PAT_SERIES_{}", timestamp);
    let patient_name = "Series^Test^Patient";
    let study_description = "Series API Test Study";
    let study_date = "2025-01-20";

    let study_id = match sqlx::query_scalar::<_, i32>(
        "INSERT INTO project_data_study (study_uid, study_description, patient_id, patient_name, study_date)
         VALUES ($1, $2, $3, $4, $5::date)
         RETURNING id"
    )
    .bind(&study_uid)
    .bind(study_description)
    .bind(&patient_id)
    .bind(patient_name)
    .bind(study_date)
    .fetch_one(pool.as_ref().as_ref())
    .await
    {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Study 생성 실패: {}", e)
            }));
        }
    };

    // 3. 프로젝트에 Study 할당
    if let Err(e) = sqlx::query(
        "INSERT INTO project_data (project_id, study_id)
         VALUES ($1, $2)"
    )
    .bind(project_id)
    .bind(study_id)
    .execute(pool.as_ref().as_ref())
    .await
    {
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Study 할당 실패: {}", e)
        }));
    }

    // 4. Series 데이터 생성 (3개 - CT, MRI, XR)
    let series_data = vec![
        (format!("1.2.840.113619.2.55.3.SERIES_TEST.{}.1", timestamp), "CT", "Axial CT", 1),
        (format!("1.2.840.113619.2.55.3.SERIES_TEST.{}.2", timestamp), "MR", "T1 Sagittal", 2),
        (format!("1.2.840.113619.2.55.3.SERIES_TEST.{}.3", timestamp), "CR", "Chest PA", 3),
    ];

    let mut series_uids = Vec::new();

    for (series_uid, modality, series_description, series_number) in series_data {
        // Series 레코드 생성
        if let Err(e) = sqlx::query(
            "INSERT INTO project_data_series (study_id, series_uid, modality, series_description, series_number)
             VALUES ($1, $2, $3, $4, $5)"
        )
        .bind(study_id)
        .bind(&series_uid)
        .bind(modality)
        .bind(series_description)
        .bind(series_number)
        .execute(pool.as_ref().as_ref())
        .await
        {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Series 생성 실패: {}", e)
            }));
        }

        series_uids.push(series_uid);
    }

    HttpResponse::Ok().json(SeriesTestSetupResponse {
        project_id,
        project_name,
        study_id,
        study_uid,
        patient_id,
        series_count: series_uids.len() as i32,
        series_uids,
    })
}

/// Series API 테스트 시나리오 정리
pub async fn cleanup_series_api_scenario(
    pool: web::Data<Arc<PgPool>>,
    path: web::Path<i32>,
) -> impl Responder {
    let project_id = path.into_inner();

    // 1. project_data_series 삭제 (Study에 연결된 Series)
    if let Err(e) = sqlx::query(
        "DELETE FROM project_data_series
         WHERE study_id IN (
            SELECT study_id FROM project_data WHERE project_id = $1
         )"
    )
    .bind(project_id)
    .execute(pool.as_ref().as_ref())
    .await
    {
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Series 삭제 실패: {}", e)
        }));
    }

    // 2. project_data 삭제
    if let Err(e) = sqlx::query("DELETE FROM project_data WHERE project_id = $1")
        .bind(project_id)
        .execute(pool.as_ref().as_ref())
        .await
    {
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("프로젝트 데이터 삭제 실패: {}", e)
        }));
    }

    // 3. project_data_study 삭제 (고아 Study)
    if let Err(e) = sqlx::query(
        "DELETE FROM project_data_study
         WHERE id NOT IN (SELECT DISTINCT study_id FROM project_data)"
    )
    .execute(pool.as_ref().as_ref())
    .await
    {
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Study 삭제 실패: {}", e)
        }));
    }

    // 4. 프로젝트 삭제
    if let Err(e) = sqlx::query("DELETE FROM security_project WHERE id = $1")
        .bind(project_id)
        .execute(pool.as_ref().as_ref())
        .await
    {
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("프로젝트 삭제 실패: {}", e)
        }));
    }

    HttpResponse::Ok().json(serde_json::json!({
        "message": "Series API 테스트 시나리오 정리 완료",
        "project_id": project_id
    }))
}

/// Keycloak 로그인 요청
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Keycloak 로그인 응답
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

/// 테스트용 Keycloak 로그인
///
/// 테스트 시나리오에서 사용할 Bearer 토큰 획득
pub async fn test_login(
    keycloak: web::Data<Arc<KeycloakClient>>,
    body: web::Json<LoginRequest>,
) -> impl Responder {
    match keycloak.authenticate_user(&body.username, &body.password).await {
        Ok(token_response) => {
            HttpResponse::Ok().json(LoginResponse {
                access_token: token_response.access_token,
                token_type: "Bearer".to_string(),
                expires_in: token_response.expires_in,
            })
        }
        Err(e) => {
            HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Login failed",
                "details": e.to_string()
            }))
        }
    }
}

/// 라우트 설정
pub fn configure_routes(
    cfg: &mut web::ServiceConfig,
    keycloak_client: Arc<KeycloakClient>,
) {
    cfg.app_data(web::Data::new(keycloak_client))
        .service(
            web::scope("/test")
                .route("/login", web::post().to(test_login))
                .service(
                    web::scope("/project-data-access")
                        .route("/setup", web::post().to(setup_project_data_access_scenario))
                        .route("/cleanup/{project_id}", web::delete().to(cleanup_project_data_access_scenario)),
                )
                .service(
                    web::scope("/series-api")
                        .route("/setup", web::post().to(setup_series_api_scenario))
                        .route("/cleanup/{project_id}", web::delete().to(cleanup_series_api_scenario)),
                )
        );
}

