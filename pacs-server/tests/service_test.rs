use pacs_server::domain::entities::{RoleScope};
use pacs_server::domain::repositories::{
    UserRepository, ProjectRepository, RoleRepository,
    PermissionRepository, AccessLogRepository
};
use pacs_server::domain::services::{
    UserService, UserServiceImpl,
    ProjectService, ProjectServiceImpl,
    PermissionService, PermissionServiceImpl,
    AccessControlService, AccessControlServiceImpl,
};
use pacs_server::infrastructure::repositories::{
    UserRepositoryImpl, ProjectRepositoryImpl, RoleRepositoryImpl,
    PermissionRepositoryImpl, AccessLogRepositoryImpl,
};
use sqlx::PgPool;
use uuid::Uuid;

async fn get_test_pool() -> PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost:5432/pacs_db".to_string());

    PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to Postgres")
}

async fn cleanup_test_data(pool: &PgPool) {
    sqlx::query("DELETE FROM security_access_log").execute(pool).await.unwrap();
    sqlx::query("DELETE FROM security_user_project").execute(pool).await.unwrap();
    sqlx::query("DELETE FROM security_project_role").execute(pool).await.unwrap();
    sqlx::query("DELETE FROM security_role_permission").execute(pool).await.unwrap();
    sqlx::query("DELETE FROM security_project_permission").execute(pool).await.unwrap();
    sqlx::query("DELETE FROM security_role_access_condition").execute(pool).await.unwrap();
    sqlx::query("DELETE FROM security_project_access_condition").execute(pool).await.unwrap();
    sqlx::query("DELETE FROM security_user").execute(pool).await.unwrap();
    sqlx::query("DELETE FROM security_project").execute(pool).await.unwrap();
    sqlx::query("DELETE FROM security_role").execute(pool).await.unwrap();
}

// ========================================
// UserService Tests
// ========================================

#[tokio::test]
async fn test_user_service_create_user() {
    let pool = get_test_pool().await;
    cleanup_test_data(&pool).await;

    let user_repo = UserRepositoryImpl::new(pool.clone());
    let user_service = UserServiceImpl::new(user_repo);

    let keycloak_id = Uuid::new_v4();
    let user = user_service
        .create_user("testuser".to_string(), "test@example.com".to_string(), keycloak_id)
        .await
        .unwrap();

    assert_eq!(user.username, "testuser");
    assert_eq!(user.email, "test@example.com");
    assert_eq!(user.keycloak_id, keycloak_id);

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_user_service_duplicate_keycloak_id() {
    let pool = get_test_pool().await;
    cleanup_test_data(&pool).await;

    let user_repo = UserRepositoryImpl::new(pool.clone());
    let user_service = UserServiceImpl::new(user_repo);

    let keycloak_id = Uuid::new_v4();
    user_service
        .create_user("user1".to_string(), "user1@example.com".to_string(), keycloak_id)
        .await
        .unwrap();

    // 같은 keycloak_id로 다시 생성 시도
    let result = user_service
        .create_user("user2".to_string(), "user2@example.com".to_string(), keycloak_id)
        .await;

    assert!(result.is_err());

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_user_service_duplicate_username() {
    let pool = get_test_pool().await;
    cleanup_test_data(&pool).await;

    let user_repo = UserRepositoryImpl::new(pool.clone());
    let user_service = UserServiceImpl::new(user_repo);

    user_service
        .create_user("duplicate".to_string(), "user1@example.com".to_string(), Uuid::new_v4())
        .await
        .unwrap();

    // 같은 username으로 다시 생성 시도
    let result = user_service
        .create_user("duplicate".to_string(), "user2@example.com".to_string(), Uuid::new_v4())
        .await;

    assert!(result.is_err());

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_user_service_invalid_email() {
    let pool = get_test_pool().await;
    cleanup_test_data(&pool).await;

    let user_repo = UserRepositoryImpl::new(pool.clone());
    let user_service = UserServiceImpl::new(user_repo);

    let result = user_service
        .create_user("testuser".to_string(), "invalid-email".to_string(), Uuid::new_v4())
        .await;

    assert!(result.is_err());

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_user_service_get_user_by_id() {
    let pool = get_test_pool().await;
    cleanup_test_data(&pool).await;

    let user_repo = UserRepositoryImpl::new(pool.clone());
    let user_service = UserServiceImpl::new(user_repo);

    let created = user_service
        .create_user("testuser".to_string(), "test@example.com".to_string(), Uuid::new_v4())
        .await
        .unwrap();

    let found = user_service.get_user_by_id(created.id).await.unwrap();
    assert_eq!(found.id, created.id);
    assert_eq!(found.username, "testuser");

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_user_service_get_user_by_username() {
    let pool = get_test_pool().await;
    cleanup_test_data(&pool).await;

    let user_repo = UserRepositoryImpl::new(pool.clone());
    let user_service = UserServiceImpl::new(user_repo);

    user_service
        .create_user("findme".to_string(), "test@example.com".to_string(), Uuid::new_v4())
        .await
        .unwrap();

    let found = user_service.get_user_by_username("findme").await.unwrap();
    assert_eq!(found.username, "findme");

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_user_service_delete_user() {
    let pool = get_test_pool().await;
    cleanup_test_data(&pool).await;

    let user_repo = UserRepositoryImpl::new(pool.clone());
    let user_service = UserServiceImpl::new(user_repo);

    let user = user_service
        .create_user("deleteme".to_string(), "delete@example.com".to_string(), Uuid::new_v4())
        .await
        .unwrap();

    user_service.delete_user(user.id).await.unwrap();

    let result = user_service.get_user_by_id(user.id).await;
    assert!(result.is_err());

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_user_service_user_exists() {
    let pool = get_test_pool().await;
    cleanup_test_data(&pool).await;

    let user_repo = UserRepositoryImpl::new(pool.clone());
    let user_service = UserServiceImpl::new(user_repo);

    let keycloak_id = Uuid::new_v4();
    user_service
        .create_user("testuser".to_string(), "test@example.com".to_string(), keycloak_id)
        .await
        .unwrap();

    let exists = user_service.user_exists(keycloak_id).await.unwrap();
    assert!(exists);

    let not_exists = user_service.user_exists(Uuid::new_v4()).await.unwrap();
    assert!(!not_exists);

    cleanup_test_data(&pool).await;
}

// ========================================
// ProjectService Tests
// ========================================

#[tokio::test]
async fn test_project_service_create_project() {
    let pool = get_test_pool().await;
    cleanup_test_data(&pool).await;

    let project_repo = ProjectRepositoryImpl::new(pool.clone());
    let project_service = ProjectServiceImpl::new(project_repo);

    let project = project_service
        .create_project("Test Project".to_string(), Some("Description".to_string()))
        .await
        .unwrap();

    assert_eq!(project.name, "Test Project");
    assert_eq!(project.description, Some("Description".to_string()));
    assert!(project.is_active);

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_project_service_duplicate_name() {
    let pool = get_test_pool().await;
    cleanup_test_data(&pool).await;

    let project_repo = ProjectRepositoryImpl::new(pool.clone());
    let project_service = ProjectServiceImpl::new(project_repo);

    project_service
        .create_project("Duplicate".to_string(), None)
        .await
        .unwrap();

    let result = project_service
        .create_project("Duplicate".to_string(), None)
        .await;

    assert!(result.is_err());

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_project_service_empty_name() {
    let pool = get_test_pool().await;
    cleanup_test_data(&pool).await;

    let project_repo = ProjectRepositoryImpl::new(pool.clone());
    let project_service = ProjectServiceImpl::new(project_repo);

    let result = project_service
        .create_project("   ".to_string(), None)
        .await;

    assert!(result.is_err());

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_project_service_name_too_long() {
    let pool = get_test_pool().await;
    cleanup_test_data(&pool).await;

    let project_repo = ProjectRepositoryImpl::new(pool.clone());
    let project_service = ProjectServiceImpl::new(project_repo);

    let long_name = "x".repeat(256);
    let result = project_service
        .create_project(long_name, None)
        .await;

    assert!(result.is_err());

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_project_service_get_project() {
    let pool = get_test_pool().await;
    cleanup_test_data(&pool).await;

    let project_repo = ProjectRepositoryImpl::new(pool.clone());
    let project_service = ProjectServiceImpl::new(project_repo);

    let created = project_service
        .create_project("Get Test".to_string(), None)
        .await
        .unwrap();

    let found = project_service.get_project(created.id).await.unwrap();
    assert_eq!(found.id, created.id);
    assert_eq!(found.name, "Get Test");

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_project_service_get_project_by_name() {
    let pool = get_test_pool().await;
    cleanup_test_data(&pool).await;

    let project_repo = ProjectRepositoryImpl::new(pool.clone());
    let project_service = ProjectServiceImpl::new(project_repo);

    project_service
        .create_project("FindByName".to_string(), None)
        .await
        .unwrap();

    let found = project_service.get_project_by_name("FindByName").await.unwrap();
    assert_eq!(found.name, "FindByName");

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_project_service_get_all_projects() {
    let pool = get_test_pool().await;
    cleanup_test_data(&pool).await;

    let project_repo = ProjectRepositoryImpl::new(pool.clone());
    let project_service = ProjectServiceImpl::new(project_repo);

    project_service.create_project("Project1".to_string(), None).await.unwrap();
    project_service.create_project("Project2".to_string(), None).await.unwrap();
    project_service.create_project("Project3".to_string(), None).await.unwrap();

    let all = project_service.get_all_projects().await.unwrap();
    assert!(all.len() >= 3);

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_project_service_activate_deactivate() {
    let pool = get_test_pool().await;
    cleanup_test_data(&pool).await;

    let project_repo = ProjectRepositoryImpl::new(pool.clone());
    let project_service = ProjectServiceImpl::new(project_repo);

    let project = project_service
        .create_project("Toggle".to_string(), None)
        .await
        .unwrap();

    assert!(project.is_active);

    let deactivated = project_service.deactivate_project(project.id).await.unwrap();
    assert!(!deactivated.is_active);

    let activated = project_service.activate_project(project.id).await.unwrap();
    assert!(activated.is_active);

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_project_service_get_active_projects() {
    let pool = get_test_pool().await;
    cleanup_test_data(&pool).await;

    let project_repo = ProjectRepositoryImpl::new(pool.clone());
    let project_service = ProjectServiceImpl::new(project_repo);

    let p1 = project_service.create_project("Active1".to_string(), None).await.unwrap();
    let p2 = project_service.create_project("Active2".to_string(), None).await.unwrap();
    let p3 = project_service.create_project("Inactive".to_string(), None).await.unwrap();

    project_service.deactivate_project(p3.id).await.unwrap();

    let active = project_service.get_active_projects().await.unwrap();
    let active_ids: Vec<i32> = active.iter().map(|p| p.id).collect();

    assert!(active_ids.contains(&p1.id));
    assert!(active_ids.contains(&p2.id));
    assert!(!active_ids.contains(&p3.id));

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_project_service_delete_project() {
    let pool = get_test_pool().await;
    cleanup_test_data(&pool).await;

    let project_repo = ProjectRepositoryImpl::new(pool.clone());
    let project_service = ProjectServiceImpl::new(project_repo);

    let project = project_service
        .create_project("DeleteMe".to_string(), None)
        .await
        .unwrap();

    project_service.delete_project(project.id).await.unwrap();

    let result = project_service.get_project(project.id).await;
    assert!(result.is_err());

    cleanup_test_data(&pool).await;
}

// ========================================
// PermissionService Tests
// ========================================

#[tokio::test]
async fn test_permission_service_create_role() {
    let pool = get_test_pool().await;
    cleanup_test_data(&pool).await;

    let permission_repo = PermissionRepositoryImpl::new(pool.clone());
    let role_repo = RoleRepositoryImpl::new(pool.clone());
    let permission_service = PermissionServiceImpl::new(permission_repo, role_repo);

    let role = permission_service
        .create_role("Admin".to_string(), RoleScope::Global, Some("Administrator".to_string()))
        .await
        .unwrap();

    assert_eq!(role.name, "Admin");
    assert_eq!(role.description, Some("Administrator".to_string()));
    assert_eq!(role.scope, "GLOBAL");

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_permission_service_duplicate_role_name() {
    let pool = get_test_pool().await;
    cleanup_test_data(&pool).await;

    let permission_repo = PermissionRepositoryImpl::new(pool.clone());
    let role_repo = RoleRepositoryImpl::new(pool.clone());
    let permission_service = PermissionServiceImpl::new(permission_repo, role_repo);

    permission_service
        .create_role("Duplicate".to_string(), RoleScope::Global, None)
        .await
        .unwrap();

    let result = permission_service
        .create_role("Duplicate".to_string(), RoleScope::Project, None)
        .await;

    assert!(result.is_err());

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_permission_service_empty_role_name() {
    let pool = get_test_pool().await;
    cleanup_test_data(&pool).await;

    let permission_repo = PermissionRepositoryImpl::new(pool.clone());
    let role_repo = RoleRepositoryImpl::new(pool.clone());
    let permission_service = PermissionServiceImpl::new(permission_repo, role_repo);

    let result = permission_service
        .create_role("   ".to_string(), RoleScope::Global, None)
        .await;

    assert!(result.is_err());

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_permission_service_role_name_too_long() {
    let pool = get_test_pool().await;
    cleanup_test_data(&pool).await;

    let permission_repo = PermissionRepositoryImpl::new(pool.clone());
    let role_repo = RoleRepositoryImpl::new(pool.clone());
    let permission_service = PermissionServiceImpl::new(permission_repo, role_repo);

    let long_name = "x".repeat(101);
    let result = permission_service
        .create_role(long_name, RoleScope::Global, None)
        .await;

    assert!(result.is_err());

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_permission_service_get_role() {
    let pool = get_test_pool().await;
    cleanup_test_data(&pool).await;

    let permission_repo = PermissionRepositoryImpl::new(pool.clone());
    let role_repo = RoleRepositoryImpl::new(pool.clone());
    let permission_service = PermissionServiceImpl::new(permission_repo, role_repo);

    let created = permission_service
        .create_role("GetRole".to_string(), RoleScope::Global, None)
        .await
        .unwrap();

    let found = permission_service.get_role(created.id).await.unwrap();
    assert_eq!(found.id, created.id);
    assert_eq!(found.name, "GetRole");

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_permission_service_get_global_roles() {
    let pool = get_test_pool().await;
    cleanup_test_data(&pool).await;

    let permission_repo = PermissionRepositoryImpl::new(pool.clone());
    let role_repo = RoleRepositoryImpl::new(pool.clone());
    let permission_service = PermissionServiceImpl::new(permission_repo, role_repo);

    let r1 = permission_service.create_role("GlobalRole1".to_string(), RoleScope::Global, None).await.unwrap();
    let r2 = permission_service.create_role("GlobalRole2".to_string(), RoleScope::Global, None).await.unwrap();
    permission_service.create_role("ProjectRole".to_string(), RoleScope::Project, None).await.unwrap();

    let globals = permission_service.get_global_roles().await.unwrap();
    let global_ids: Vec<i32> = globals.iter().map(|r| r.id).collect();

    assert!(global_ids.contains(&r1.id));
    assert!(global_ids.contains(&r2.id));

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_permission_service_get_project_roles() {
    let pool = get_test_pool().await;
    cleanup_test_data(&pool).await;

    let permission_repo = PermissionRepositoryImpl::new(pool.clone());
    let role_repo = RoleRepositoryImpl::new(pool.clone());
    let permission_service = PermissionServiceImpl::new(permission_repo, role_repo);

    permission_service.create_role("GlobalRole".to_string(), RoleScope::Global, None).await.unwrap();
    let r1 = permission_service.create_role("ProjectRole1".to_string(), RoleScope::Project, None).await.unwrap();
    let r2 = permission_service.create_role("ProjectRole2".to_string(), RoleScope::Project, None).await.unwrap();

    let projects = permission_service.get_project_roles().await.unwrap();
    let project_ids: Vec<i32> = projects.iter().map(|r| r.id).collect();

    assert!(project_ids.contains(&r1.id));
    assert!(project_ids.contains(&r2.id));

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_permission_service_get_roles_by_scope() {
    let pool = get_test_pool().await;
    cleanup_test_data(&pool).await;

    let permission_repo = PermissionRepositoryImpl::new(pool.clone());
    let role_repo = RoleRepositoryImpl::new(pool.clone());
    let permission_service = PermissionServiceImpl::new(permission_repo, role_repo);

    let r1 = permission_service.create_role("Global1".to_string(), RoleScope::Global, None).await.unwrap();
    let r2 = permission_service.create_role("Project1".to_string(), RoleScope::Project, None).await.unwrap();

    let globals = permission_service.get_roles_by_scope(RoleScope::Global).await.unwrap();
    let global_ids: Vec<i32> = globals.iter().map(|r| r.id).collect();
    assert!(global_ids.contains(&r1.id));

    let projects = permission_service.get_roles_by_scope(RoleScope::Project).await.unwrap();
    let project_ids: Vec<i32> = projects.iter().map(|r| r.id).collect();
    assert!(project_ids.contains(&r2.id));

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_permission_service_validate_permission_exists() {
    let pool = get_test_pool().await;
    cleanup_test_data(&pool).await;

    let permission_repo = PermissionRepositoryImpl::new(pool.clone());
    let role_repo = RoleRepositoryImpl::new(pool.clone());
    let permission_service = PermissionServiceImpl::new(permission_repo, role_repo);

    // 실제 DB에 있는 권한이 있다고 가정하거나, 테스트용 권한을 먼저 생성해야 함
    // 여기서는 존재하지 않는 권한으로 테스트
    let exists = permission_service
        .validate_permission_exists("nonexistent", "read")
        .await
        .unwrap();

    assert!(!exists);

    cleanup_test_data(&pool).await;
}

// ========================================
// AccessControlService Tests
// ========================================

#[tokio::test]
async fn test_access_control_service_log_dicom_access() {
    let pool = get_test_pool().await;
    cleanup_test_data(&pool).await;

    // 먼저 사용자 생성
    let user_repo = UserRepositoryImpl::new(pool.clone());
    let user_service = UserServiceImpl::new(user_repo);
    let user = user_service
        .create_user("testuser".to_string(), "test@example.com".to_string(), Uuid::new_v4())
        .await
        .unwrap();

    // 프로젝트 생성
    let project_repo = ProjectRepositoryImpl::new(pool.clone());
    let project_service = ProjectServiceImpl::new(project_repo);
    let project = project_service
        .create_project("TestProject".to_string(), None)
        .await
        .unwrap();

    // AccessControlService 생성
    let access_log_repo = AccessLogRepositoryImpl::new(pool.clone());
    let user_repo2 = UserRepositoryImpl::new(pool.clone());
    let project_repo2 = ProjectRepositoryImpl::new(pool.clone());
    let access_service = AccessControlServiceImpl::new(access_log_repo, user_repo2, project_repo2);

    let log = access_service
        .log_dicom_access(
            user.id,
            Some(project.id),
            "STUDY".to_string(),
            Some("1.2.3.4.5".to_string()),
            None,
            None,
            "VIEW".to_string(),
            "SUCCESS".to_string(),
            Some("192.168.1.1".to_string()),
            Some("WORKSTATION1".to_string()),
        )
        .await
        .unwrap();

    assert_eq!(log.user_id, user.id);
    assert_eq!(log.resource_type, "STUDY");
    assert_eq!(log.action, "VIEW");
    assert_eq!(log.result, "SUCCESS");

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_access_control_service_log_with_invalid_user() {
    let pool = get_test_pool().await;
    cleanup_test_data(&pool).await;

    let access_log_repo = AccessLogRepositoryImpl::new(pool.clone());
    let user_repo = UserRepositoryImpl::new(pool.clone());
    let project_repo = ProjectRepositoryImpl::new(pool.clone());
    let access_service = AccessControlServiceImpl::new(access_log_repo, user_repo, project_repo);

    let result = access_service
        .log_dicom_access(
            99999,
            None,
            "STUDY".to_string(),
            None,
            None,
            None,
            "VIEW".to_string(),
            "SUCCESS".to_string(),
            None,
            None,
        )
        .await;

    assert!(result.is_err());

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_access_control_service_get_user_access_logs() {
    let pool = get_test_pool().await;
    cleanup_test_data(&pool).await;

    // 사용자 생성
    let user_repo = UserRepositoryImpl::new(pool.clone());
    let user_service = UserServiceImpl::new(user_repo);
    let user = user_service
        .create_user("loguser".to_string(), "log@example.com".to_string(), Uuid::new_v4())
        .await
        .unwrap();

    // AccessControlService 생성
    let access_log_repo = AccessLogRepositoryImpl::new(pool.clone());
    let user_repo2 = UserRepositoryImpl::new(pool.clone());
    let project_repo = ProjectRepositoryImpl::new(pool.clone());
    let access_service = AccessControlServiceImpl::new(access_log_repo, user_repo2, project_repo);

    // 로그 3개 생성
    for i in 0..3 {
        access_service
            .log_dicom_access(
                user.id,
                None,
                "STUDY".to_string(),
                Some(format!("1.2.3.{}", i)),
                None,
                None,
                "VIEW".to_string(),
                "SUCCESS".to_string(),
                None,
                None,
            )
            .await
            .unwrap();
    }

    let logs = access_service.get_user_access_logs(user.id, 10).await.unwrap();
    assert_eq!(logs.len(), 3);

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_access_control_service_get_project_access_logs() {
    let pool = get_test_pool().await;
    cleanup_test_data(&pool).await;

    // 사용자 및 프로젝트 생성
    let user_repo = UserRepositoryImpl::new(pool.clone());
    let user_service = UserServiceImpl::new(user_repo);
    let user = user_service
        .create_user("projectuser".to_string(), "project@example.com".to_string(), Uuid::new_v4())
        .await
        .unwrap();

    let project_repo = ProjectRepositoryImpl::new(pool.clone());
    let project_service = ProjectServiceImpl::new(project_repo);
    let project = project_service
        .create_project("LogProject".to_string(), None)
        .await
        .unwrap();

    // AccessControlService
    let access_log_repo = AccessLogRepositoryImpl::new(pool.clone());
    let user_repo2 = UserRepositoryImpl::new(pool.clone());
    let project_repo2 = ProjectRepositoryImpl::new(pool.clone());
    let access_service = AccessControlServiceImpl::new(access_log_repo, user_repo2, project_repo2);

    // 프로젝트 관련 로그 생성
    for _ in 0..2 {
        access_service
            .log_dicom_access(
                user.id,
                Some(project.id),
                "STUDY".to_string(),
                None,
                None,
                None,
                "VIEW".to_string(),
                "SUCCESS".to_string(),
                None,
                None,
            )
            .await
            .unwrap();
    }

    let logs = access_service.get_project_access_logs(project.id, 10).await.unwrap();
    assert_eq!(logs.len(), 2);

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_access_control_service_count_user_access() {
    let pool = get_test_pool().await;
    cleanup_test_data(&pool).await;

    // 사용자 생성
    let user_repo = UserRepositoryImpl::new(pool.clone());
    let user_service = UserServiceImpl::new(user_repo);
    let user = user_service
        .create_user("countuser".to_string(), "count@example.com".to_string(), Uuid::new_v4())
        .await
        .unwrap();

    // AccessControlService
    let access_log_repo = AccessLogRepositoryImpl::new(pool.clone());
    let user_repo2 = UserRepositoryImpl::new(pool.clone());
    let project_repo = ProjectRepositoryImpl::new(pool.clone());
    let access_service = AccessControlServiceImpl::new(access_log_repo, user_repo2, project_repo);

    // 로그 5개 생성
    for _ in 0..5 {
        access_service
            .log_dicom_access(
                user.id,
                None,
                "STUDY".to_string(),
                None,
                None,
                None,
                "VIEW".to_string(),
                "SUCCESS".to_string(),
                None,
                None,
            )
            .await
            .unwrap();
    }

    let count = access_service.count_user_access(user.id).await.unwrap();
    assert_eq!(count, 5);

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_access_control_service_can_access_project() {
    let pool = get_test_pool().await;
    cleanup_test_data(&pool).await;

    // 사용자 및 프로젝트 생성
    let user_repo = UserRepositoryImpl::new(pool.clone());
    let user_service = UserServiceImpl::new(user_repo);
    let user = user_service
        .create_user("accessuser".to_string(), "access@example.com".to_string(), Uuid::new_v4())
        .await
        .unwrap();

    let project_repo = ProjectRepositoryImpl::new(pool.clone());
    let project_service = ProjectServiceImpl::new(project_repo);
    let active_project = project_service
        .create_project("ActiveProject".to_string(), None)
        .await
        .unwrap();

    let inactive_project = project_service
        .create_project("InactiveProject".to_string(), None)
        .await
        .unwrap();

    project_service.deactivate_project(inactive_project.id).await.unwrap();

    // AccessControlService
    let access_log_repo = AccessLogRepositoryImpl::new(pool.clone());
    let user_repo2 = UserRepositoryImpl::new(pool.clone());
    let project_repo2 = ProjectRepositoryImpl::new(pool.clone());
    let access_service = AccessControlServiceImpl::new(access_log_repo, user_repo2, project_repo2);

    // 활성 프로젝트 접근 가능
    let can_access_active = access_service
        .can_access_project(user.id, active_project.id)
        .await
        .unwrap();
    assert!(can_access_active);

    // 비활성 프로젝트 접근 불가
    let can_access_inactive = access_service
        .can_access_project(user.id, inactive_project.id)
        .await
        .unwrap();
    assert!(!can_access_inactive);

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_access_control_service_get_study_access_logs() {
    let pool = get_test_pool().await;
    cleanup_test_data(&pool).await;

    // 사용자 생성
    let user_repo = UserRepositoryImpl::new(pool.clone());
    let user_service = UserServiceImpl::new(user_repo);
    let user = user_service
        .create_user("studyuser".to_string(), "study@example.com".to_string(), Uuid::new_v4())
        .await
        .unwrap();

    // AccessControlService
    let access_log_repo = AccessLogRepositoryImpl::new(pool.clone());
    let user_repo2 = UserRepositoryImpl::new(pool.clone());
    let project_repo = ProjectRepositoryImpl::new(pool.clone());
    let access_service = AccessControlServiceImpl::new(access_log_repo, user_repo2, project_repo);

    let study_uid = "1.2.840.113619.2.55.3.123456789";

    // 특정 Study에 대한 로그 생성
    for _ in 0..3 {
        access_service
            .log_dicom_access(
                user.id,
                None,
                "STUDY".to_string(),
                Some(study_uid.to_string()),
                None,
                None,
                "VIEW".to_string(),
                "SUCCESS".to_string(),
                None,
                None,
            )
            .await
            .unwrap();
    }

    let logs = access_service.get_study_access_logs(study_uid, 10).await.unwrap();
    assert_eq!(logs.len(), 3);
    assert_eq!(logs[0].study_uid, Some(study_uid.to_string()));

    cleanup_test_data(&pool).await;
}
