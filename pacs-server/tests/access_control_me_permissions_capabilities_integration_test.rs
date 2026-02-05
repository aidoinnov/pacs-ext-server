//! AccessControlService get_my_permission_codes, get_my_capability_codes 통합 테스트
//!
//! 요구사항: docs/api/capability/add-job.md

use pacs_server::domain::services::{AccessControlService, AccessControlServiceImpl};
use pacs_server::infrastructure::repositories::{
    AccessLogRepositoryImpl, PermissionRepositoryImpl, ProjectRepositoryImpl,
    RoleRepositoryImpl, UserRepositoryImpl,
};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;

async fn get_test_pool() -> sqlx::PgPool {
    let database_url = std::env::var("APP_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .unwrap_or_else(|_| {
            "postgres://pacs_extension_admin:PacsExtension2024@localhost:5456/pacs_extension"
                .to_string()
        });

    PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

fn build_access_control_service(pool: &sqlx::PgPool) -> Arc<AccessControlServiceImpl<
    AccessLogRepositoryImpl,
    UserRepositoryImpl,
    ProjectRepositoryImpl,
    RoleRepositoryImpl,
    PermissionRepositoryImpl,
>> {
    Arc::new(AccessControlServiceImpl::new(
        AccessLogRepositoryImpl::new(pool.clone()),
        UserRepositoryImpl::new(pool.clone()),
        ProjectRepositoryImpl::new(pool.clone()),
        RoleRepositoryImpl::new(pool.clone()),
        PermissionRepositoryImpl::new(pool.clone()),
    ))
}

/// iaid-pacs-admin (SUPER_ADMIN) 또는 USER 역할 사용자 ID 조회
async fn find_test_user_id(pool: &sqlx::PgPool) -> Option<i32> {
    let row: Option<(i32,)> = sqlx::query_as(
        "SELECT id FROM security_user WHERE username = 'iaid-pacs-admin' OR username = 'reader1_user' LIMIT 1",
    )
    .fetch_optional(pool)
    .await
    .ok()?;

    row.map(|r| r.0)
}

#[tokio::test]
async fn test_get_my_permission_codes_returns_list() {
    let pool = get_test_pool().await;
    let service = build_access_control_service(&pool);

    let user_id = find_test_user_id(&pool).await.expect("No test user found");

    let result = service.get_my_permission_codes(user_id).await;
    assert!(result.is_ok(), "get_my_permission_codes should succeed");
    let codes = result.unwrap();
    assert!(codes.iter().all(|c| c.contains('.')), "permission format: resource_type.action");
}

#[tokio::test]
async fn test_get_my_capability_codes_returns_list() {
    let pool = get_test_pool().await;
    let service = build_access_control_service(&pool);

    let user_id = find_test_user_id(&pool).await.expect("No test user found");

    let result = service.get_my_capability_codes(user_id).await;
    assert!(result.is_ok(), "get_my_capability_codes should succeed");
    let codes = result.unwrap();
    assert!(codes.iter().all(|c| !c.is_empty()), "capability codes should be non-empty");
}

#[tokio::test]
async fn test_super_admin_has_settings_permissions() {
    let pool = get_test_pool().await;
    let service = build_access_control_service(&pool);

    let row: Option<(i32,)> = sqlx::query_as(
        "SELECT u.id FROM security_user u
         INNER JOIN security_user_global_role ugr ON u.id = ugr.user_id
         INNER JOIN security_role r ON ugr.role_id = r.id
         WHERE r.name = 'SUPER_ADMIN' AND r.scope = 'GLOBAL' LIMIT 1",
    )
    .fetch_optional(&pool)
    .await
    .ok()
    .flatten();

    let (user_id,) = match row {
        Some(r) => r,
        None => {
            eprintln!("No SUPER_ADMIN user found, skipping");
            return;
        }
    };

    let perms = service.get_my_permission_codes(user_id).await.unwrap();
    assert!(
        perms.iter().any(|p| p == "project_data.assign"),
        "SUPER_ADMIN should have project_data.assign: {:?}",
        perms
    );

    let caps = service.get_my_capability_codes(user_id).await.unwrap();
    assert!(
        caps.contains(&"ROLE_MANAGEMENT".to_string()),
        "SUPER_ADMIN should have ROLE_MANAGEMENT: {:?}",
        caps
    );
    assert!(
        caps.contains(&"PROJECT_MANAGEMENT".to_string()),
        "SUPER_ADMIN should have PROJECT_MANAGEMENT: {:?}",
        caps
    );
}

#[tokio::test]
async fn test_nonexistent_user_returns_empty_or_error() {
    let pool = get_test_pool().await;
    let service = build_access_control_service(&pool);

    let result = service.get_my_permission_codes(999999).await;
    // 존재하지 않는 사용자: DB에는 에러 없이 빈 결과 반환 (role이 없으므로)
    assert!(result.is_ok(), "should not panic for nonexistent user");
    let perms = result.unwrap();
    assert!(perms.is_empty(), "nonexistent user should have no permissions");

    let result2 = service.get_my_capability_codes(999999).await;
    assert!(result2.is_ok());
    assert!(result2.unwrap().is_empty());
}
