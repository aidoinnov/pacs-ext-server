use actix_web::{test, web, App};
use pacs_server::application::dto::project_user_dto::AddMemberRequest;
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

/// 테스트 1: 이미 멤버인 사용자에게 역할 추가 (role_id = NULL → role_id = 설정값)
#[actix_web::test]
#[ignore] // 실제 DB 필요
async fn test_add_member_with_existing_membership() {
    // Given: 데이터베이스 연결
    let database_url = std::env::var("APP_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://pacs_extension_admin:PacsExtension2024@localhost:5456/pacs_extension".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    // Given: 테스트 데이터 생성 (멤버십 있지만 role_id = NULL)
    let (user_id, project_id, role_id) = setup_test_data_with_null_role(&pool).await;

    // Given: Use case 및 컨트롤러 설정
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

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(use_case.clone()))
            .route(
                "/api/projects/{project_id}/members",
                web::post().to(
                    project_user_controller::add_project_member::<
                        ProjectServiceImpl<
                            ProjectRepositoryImpl,
                            UserRepositoryImpl,
                            RoleRepositoryImpl,
                        >,
                        UserServiceImpl<UserRepositoryImpl, ProjectRepositoryImpl>,
                        ProjectDataServiceImpl<
                            ProjectDataRepositoryImpl,
                            ProjectDataAccessRepositoryImpl,
                        >,
                    >,
                ),
            ),
    )
    .await;

    // When: 멤버 추가 API 호출 (이미 멤버이지만 role_id = NULL)
    let req = test::TestRequest::post()
        .uri(&format!("/api/projects/{}/members", project_id))
        .set_json(AddMemberRequest {
            user_id,
            role_id: Some(role_id),
        })
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

    // 데이터베이스에서 역할이 업데이트되었는지 확인
    let updated_role: Option<(i32,)> = sqlx::query_as(
        "SELECT role_id FROM security_user_project WHERE user_id = $1 AND project_id = $2",
    )
    .bind(user_id)
    .bind(project_id)
    .fetch_optional(&pool)
    .await
    .unwrap();

    assert!(updated_role.is_some(), "Membership should exist");
    assert_eq!(
        updated_role.unwrap().0,
        role_id,
        "Role should be updated to the specified role"
    );

    // Cleanup
    cleanup_test_data(&pool, user_id, project_id).await;
}

/// 테스트 2: 멤버가 아닌 사용자를 역할과 함께 추가
#[actix_web::test]
#[ignore] // 실제 DB 필요
async fn test_add_new_member_with_role() {
    // Given: 데이터베이스 연결
    let database_url = std::env::var("APP_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://pacs_extension_admin:PacsExtension2024@localhost:5456/pacs_extension".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    // Given: 테스트 데이터 생성 (멤버십 없음)
    let (user_id, project_id, role_id) = setup_test_data_without_membership(&pool).await;

    // Given: Use case 및 컨트롤러 설정
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

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(use_case.clone()))
            .route(
                "/api/projects/{project_id}/members",
                web::post().to(
                    project_user_controller::add_project_member::<
                        ProjectServiceImpl<
                            ProjectRepositoryImpl,
                            UserRepositoryImpl,
                            RoleRepositoryImpl,
                        >,
                        UserServiceImpl<UserRepositoryImpl, ProjectRepositoryImpl>,
                        ProjectDataServiceImpl<
                            ProjectDataRepositoryImpl,
                            ProjectDataAccessRepositoryImpl,
                        >,
                    >,
                ),
            ),
    )
    .await;

    // When: 멤버 추가 API 호출
    let req = test::TestRequest::post()
        .uri(&format!("/api/projects/{}/members", project_id))
        .set_json(AddMemberRequest {
            user_id,
            role_id: Some(role_id),
        })
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

    assert!(membership.is_some(), "User should be added as a member");
    assert_eq!(
        membership.unwrap().0,
        role_id,
        "Role should be assigned correctly"
    );

    // Cleanup
    cleanup_test_data(&pool, user_id, project_id).await;
}

/// 헬퍼: 테스트 데이터 생성 (멤버십 있지만 role_id = NULL)
async fn setup_test_data_with_null_role(pool: &sqlx::PgPool) -> (i32, i32, i32) {
    let unique_suffix = Uuid::new_v4().to_string()[..8].to_string();

    // 사용자 생성
    let user_id: i32 = sqlx::query_scalar(
        "INSERT INTO security_user (keycloak_id, username, email) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(Uuid::new_v4())
    .bind(format!("test_user_{}", unique_suffix))
    .bind(format!("test_user_{}@example.com", unique_suffix))
    .fetch_one(pool)
    .await
    .unwrap();

    // 프로젝트 생성
    let project_id: i32 = sqlx::query_scalar(
        "INSERT INTO security_project (name, description) VALUES ($1, $2) RETURNING id",
    )
    .bind(format!("Test Project {}", unique_suffix))
    .bind("Test project for integration testing")
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

    // 멤버십 추가 (role_id = NULL)
    sqlx::query(
        "INSERT INTO security_user_project (user_id, project_id, role_id) VALUES ($1, $2, NULL)",
    )
    .bind(user_id)
    .bind(project_id)
    .execute(pool)
    .await
    .unwrap();

    (user_id, project_id, role_id.0)
}




/// 헬퍼: 테스트 데이터 생성 (멤버십 없음)
async fn setup_test_data_without_membership(pool: &sqlx::PgPool) -> (i32, i32, i32) {
    let unique_suffix = Uuid::new_v4().to_string()[..8].to_string();

    // 사용자 생성
    let user_id: i32 = sqlx::query_scalar(
        "INSERT INTO security_user (keycloak_id, username, email) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(Uuid::new_v4())
    .bind(format!("test_user_{}", unique_suffix))
    .bind(format!("test_user_{}@example.com", unique_suffix))
    .fetch_one(pool)
    .await
    .unwrap();

    // 프로젝트 생성
    let project_id: i32 = sqlx::query_scalar(
        "INSERT INTO security_project (name, description) VALUES ($1, $2) RETURNING id",
    )
    .bind(format!("Test Project {}", unique_suffix))
    .bind("Test project for integration testing")
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

    // 멤버십 추가하지 않음 (Non-member)

    (user_id, project_id, role_id.0)
}

/// 헬퍼: 테스트 데이터 정리
async fn cleanup_test_data(pool: &sqlx::PgPool, user_id: i32, project_id: i32) {
    // 멤버십 삭제
    sqlx::query("DELETE FROM security_user_project WHERE user_id = $1 AND project_id = $2")
        .bind(user_id)
        .bind(project_id)
        .execute(pool)
        .await
        .ok();

    // 사용자 삭제
    sqlx::query("DELETE FROM security_user WHERE id = $1")
        .bind(user_id)
        .execute(pool)
        .await
        .ok();

    // 프로젝트 삭제
    sqlx::query("DELETE FROM security_project WHERE id = $1")
        .bind(project_id)
        .execute(pool)
        .await
        .ok();
}
