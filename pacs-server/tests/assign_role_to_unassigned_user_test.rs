use actix_web::{test, web, App};
use pacs_server::application::use_cases::project_user_use_case::ProjectUserUseCase;
use pacs_server::domain::services::{ProjectServiceImpl, UserServiceImpl};
use pacs_server::infrastructure::repositories::{
    ProjectDataAccessRepositoryImpl, ProjectDataRepositoryImpl, ProjectRepositoryImpl,
    RoleRepositoryImpl, UserRepositoryImpl,
};
use pacs_server::infrastructure::services::ProjectDataServiceImpl;
use pacs_server::presentation::controllers::project_user_controller;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use uuid::Uuid;

/// 통합 테스트: Unassigned 상태 사용자에게 역할 할당
///
/// 시나리오:
/// 1. 사용자와 프로젝트 생성
/// 2. 사용자를 프로젝트에 추가 (role_id = NULL, Unassigned 상태)
/// 3. 역할 할당 API 호출
/// 4. 역할이 정상적으로 할당되었는지 확인
#[actix_web::test]
#[ignore]
async fn test_assign_role_to_unassigned_user() {
    // Given: 데이터베이스 연결
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL")
        .or_else(|_| std::env::var("APP_DATABASE_URL"))
        .expect("DATABASE_URL or APP_DATABASE_URL not set for test");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    // 테스트 데이터 생성
    let (user_id, project_id, role_id) = setup_test_data(&pool).await;

    // 서비스 및 UseCase 설정
    let user_repo1 = UserRepositoryImpl::new(pool.clone());
    let user_repo2 = UserRepositoryImpl::new(pool.clone());
    let project_repo1 = ProjectRepositoryImpl::new(pool.clone());
    let project_repo2 = ProjectRepositoryImpl::new(pool.clone());
    let role_repo = RoleRepositoryImpl::new(pool.clone());
    let project_data_repo = Arc::new(ProjectDataRepositoryImpl::new(pool.clone()));
    let project_data_access_repo = Arc::new(ProjectDataAccessRepositoryImpl::new(pool.clone()));

    let project_data_service = Arc::new(ProjectDataServiceImpl::new(
        project_data_repo.clone(),
        project_data_access_repo.clone(),
    ));

    let user_service = Arc::new(UserServiceImpl::new(user_repo1, project_repo1));
    let project_service = Arc::new(ProjectServiceImpl::new(project_repo2, user_repo2, role_repo));

    let use_case = Arc::new(ProjectUserUseCase::new(
        project_service.clone(),
        user_service,
        project_data_service.clone(),
    ));

    // 앱 설정
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(use_case.clone()))
            .app_data(web::Data::new(
                pacs_server::application::use_cases::project_data_access_use_case::ProjectDataAccessUseCase::new(
                    project_data_service.clone(),
                    project_service.clone(),
                ),
            ))
            .service(web::scope("/api").configure(|cfg| {
                project_user_controller::configure_routes(
                    cfg,
                    use_case.clone(),
                    Arc::new(
                        pacs_server::application::use_cases::project_data_access_use_case::ProjectDataAccessUseCase::new(
                            Arc::new(ProjectDataServiceImpl::new(
                                project_data_repo.clone(),
                                project_data_access_repo.clone(),
                            )),
                            project_service.clone(),
                        ),
                    ),
                )
            })),
    )
    .await;

    // When: 역할 할당 API 호출
    let req = test::TestRequest::put()
        .uri(&format!("/api/projects/{}/users/{}/role", project_id, user_id))
        .set_json(serde_json::json!({
            "role_id": role_id
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Then: 성공 응답 확인
    let status = resp.status();
    let body: serde_json::Value = test::read_body_json(resp).await;

    if !status.is_success() {
        eprintln!("❌ API Error Response: {}", serde_json::to_string_pretty(&body).unwrap());
    }

    assert!(
        status.is_success(),
        "API should return success, got: {:?}, body: {:?}",
        status,
        body
    );
    assert_eq!(body["user_id"].as_i64().unwrap(), user_id as i64);
    assert_eq!(body["project_id"].as_i64().unwrap(), project_id as i64);
    assert_eq!(body["role_id"].as_i64().unwrap(), role_id as i64);

    // 데이터베이스에서 역할이 실제로 할당되었는지 확인
    let assigned_role: Option<(i32,)> = sqlx::query_as(
        "SELECT role_id FROM security_user_project WHERE user_id = $1 AND project_id = $2",
    )
    .bind(user_id)
    .bind(project_id)
    .fetch_optional(&pool)
    .await
    .unwrap();

    assert!(assigned_role.is_some(), "Role should be assigned in database");
    assert_eq!(
        assigned_role.unwrap().0,
        role_id,
        "Assigned role should match requested role"
    );

    // 정리
    cleanup_test_data(&pool, user_id, project_id).await;
}

/// 통합 테스트: 멤버가 아닌 사용자에게 역할 할당 (자동 멤버 추가)
///
/// 시나리오:
/// 1. 사용자와 프로젝트 생성
/// 2. 사용자를 프로젝트에 추가하지 않음 (멤버가 아님)
/// 3. 역할 할당 API 호출
/// 4. 자동으로 멤버로 추가되면서 역할이 할당되는지 확인
#[actix_web::test]
#[ignore]
async fn test_assign_role_to_non_member_user() {
    // Given: 데이터베이스 연결
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL")
        .or_else(|_| std::env::var("APP_DATABASE_URL"))
        .expect("DATABASE_URL or APP_DATABASE_URL not set for test");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    // 테스트 데이터 생성 (멤버 추가 없이)
    let (user_id, project_id, role_id) = setup_test_data_without_membership(&pool).await;

    // 서비스 및 UseCase 설정
    let user_repo1 = UserRepositoryImpl::new(pool.clone());
    let user_repo2 = UserRepositoryImpl::new(pool.clone());
    let project_repo1 = ProjectRepositoryImpl::new(pool.clone());
    let project_repo2 = ProjectRepositoryImpl::new(pool.clone());
    let role_repo = RoleRepositoryImpl::new(pool.clone());
    let project_data_repo = Arc::new(ProjectDataRepositoryImpl::new(pool.clone()));
    let project_data_access_repo = Arc::new(ProjectDataAccessRepositoryImpl::new(pool.clone()));

    let project_data_service = Arc::new(ProjectDataServiceImpl::new(
        project_data_repo.clone(),
        project_data_access_repo.clone(),
    ));

    let user_service = Arc::new(UserServiceImpl::new(user_repo1, project_repo1));
    let project_service = Arc::new(ProjectServiceImpl::new(project_repo2, user_repo2, role_repo));

    let use_case = Arc::new(ProjectUserUseCase::new(
        project_service.clone(),
        user_service,
        project_data_service.clone(),
    ));

    // 앱 설정
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(use_case.clone()))
            .app_data(web::Data::new(
                pacs_server::application::use_cases::project_data_access_use_case::ProjectDataAccessUseCase::new(
                    project_data_service.clone(),
                    project_service.clone(),
                ),
            ))
            .service(web::scope("/api").configure(|cfg| {
                project_user_controller::configure_routes(
                    cfg,
                    use_case.clone(),
                    Arc::new(
                        pacs_server::application::use_cases::project_data_access_use_case::ProjectDataAccessUseCase::new(
                            Arc::new(ProjectDataServiceImpl::new(
                                project_data_repo.clone(),
                                project_data_access_repo.clone(),
                            )),
                            project_service.clone(),
                        ),
                    ),
                )
            })),
    )
    .await;

    // When: 역할 할당 API 호출
    let req = test::TestRequest::put()
        .uri(&format!("/api/projects/{}/users/{}/role", project_id, user_id))
        .set_json(serde_json::json!({
            "role_id": role_id
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Then: 성공 응답 확인
    let status = resp.status();
    let body: serde_json::Value = test::read_body_json(resp).await;

    if !status.is_success() {
        eprintln!("❌ API Error Response: {}", serde_json::to_string_pretty(&body).unwrap());
    }

    assert!(
        status.is_success(),
        "API should return success, got: {:?}, body: {:?}",
        status,
        body
    );

    // 데이터베이스에서 멤버로 추가되었고 역할이 할당되었는지 확인
    let membership: Option<(i32,)> = sqlx::query_as(
        "SELECT role_id FROM security_user_project WHERE user_id = $1 AND project_id = $2",
    )
    .bind(user_id)
    .bind(project_id)
    .fetch_optional(&pool)
    .await
    .unwrap();

    assert!(
        membership.is_some(),
        "User should be added as member with role"
    );
    assert_eq!(
        membership.unwrap().0,
        role_id,
        "Assigned role should match requested role"
    );

    // 정리
    cleanup_test_data(&pool, user_id, project_id).await;
}

/// 테스트 데이터 생성 (Unassigned 상태로)
async fn setup_test_data(pool: &sqlx::PgPool) -> (i32, i32, i32) {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    // 사용자 생성
    let keycloak_id = Uuid::new_v4();
    let username = format!("test_unassigned_user_{}", timestamp);
    let email = format!("{}@example.com", username);

    let user_id: (i32,) = sqlx::query_as(
        "INSERT INTO security_user (keycloak_id, username, email) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(keycloak_id)
    .bind(&username)
    .bind(&email)
    .fetch_one(pool)
    .await
    .unwrap();

    // 프로젝트 생성
    let project_name = format!("Test Unassigned Project {}", timestamp);
    let description = format!("Test project for unassigned user test {}", timestamp);

    let project_id: (i32,) = sqlx::query_as(
        "INSERT INTO security_project (name, description, status) VALUES ($1, $2, 'IN_PROGRESS') RETURNING id",
    )
    .bind(&project_name)
    .bind(&description)
    .fetch_one(pool)
    .await
    .unwrap();

    // 역할 조회 (PROJECT_ADMIN)
    let role_id: (i32,) = sqlx::query_as(
        "SELECT id FROM security_role WHERE name = 'PROJECT_ADMIN' LIMIT 1",
    )
    .fetch_one(pool)
    .await
    .unwrap();

    // 사용자를 프로젝트에 추가 (role_id = NULL, Unassigned 상태)
    sqlx::query(
        "INSERT INTO security_user_project (user_id, project_id, role_id) VALUES ($1, $2, NULL)",
    )
    .bind(user_id.0)
    .bind(project_id.0)
    .execute(pool)
    .await
    .unwrap();

    (user_id.0, project_id.0, role_id.0)
}

/// 테스트 데이터 생성 (멤버십 없이)
async fn setup_test_data_without_membership(pool: &sqlx::PgPool) -> (i32, i32, i32) {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    // 사용자 생성
    let keycloak_id = Uuid::new_v4();
    let username = format!("test_non_member_user_{}", timestamp);
    let email = format!("{}@example.com", username);

    let user_id: (i32,) = sqlx::query_as(
        "INSERT INTO security_user (keycloak_id, username, email) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(keycloak_id)
    .bind(&username)
    .bind(&email)
    .fetch_one(pool)
    .await
    .unwrap();

    // 프로젝트 생성
    let project_name = format!("Test Non Member Project {}", timestamp);
    let description = format!("Test project for non-member user test {}", timestamp);

    let project_id: (i32,) = sqlx::query_as(
        "INSERT INTO security_project (name, description, status) VALUES ($1, $2, 'IN_PROGRESS') RETURNING id",
    )
    .bind(&project_name)
    .bind(&description)
    .fetch_one(pool)
    .await
    .unwrap();

    // 역할 조회 (PROJECT_ADMIN)
    let role_id: (i32,) = sqlx::query_as(
        "SELECT id FROM security_role WHERE name = 'PROJECT_ADMIN' LIMIT 1",
    )
    .fetch_one(pool)
    .await
    .unwrap();

    // 멤버십 추가하지 않음 (non-member 상태)

    (user_id.0, project_id.0, role_id.0)
}

/// 테스트 데이터 정리
async fn cleanup_test_data(pool: &sqlx::PgPool, user_id: i32, project_id: i32) {
    // 멤버십 삭제
    let _ = sqlx::query("DELETE FROM security_user_project WHERE user_id = $1 AND project_id = $2")
        .bind(user_id)
        .bind(project_id)
        .execute(pool)
        .await;

    // 사용자 삭제
    let _ = sqlx::query("DELETE FROM security_user WHERE id = $1")
        .bind(user_id)
        .execute(pool)
        .await;

    // 프로젝트 삭제
    let _ = sqlx::query("DELETE FROM security_project WHERE id = $1")
        .bind(project_id)
        .execute(pool)
        .await;
}

