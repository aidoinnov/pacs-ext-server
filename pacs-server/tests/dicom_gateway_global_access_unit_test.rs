use sqlx::{PgPool, postgres::PgPoolOptions};
use std::env;

/// 단위 테스트: has_global_dicom_access() 함수
///
/// 테스트 케이스:
/// 1. SUPER_ADMIN 사용자 - 전체 권한 있음
/// 2. ADMIN 사용자 - 전체 권한 있음
/// 3. 일반 사용자 - 전체 권한 없음
/// 4. 존재하지 않는 사용자 - 전체 권한 없음

/// 사용자가 DICOM 전체 접근 권한을 가지고 있는지 확인
async fn has_global_dicom_access(user_id: i32, pool: &sqlx::PgPool) -> bool {
    sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(
            SELECT 1 FROM security_user_project sup
            INNER JOIN security_role r ON sup.role_id = r.id
            INNER JOIN security_role_capability src ON r.id = src.role_id
            INNER JOIN security_capability c ON src.capability_id = c.id
            WHERE sup.user_id = $1
              AND c.name = 'DICOM_GLOBAL_ACCESS'
        )"
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
    .unwrap_or(false)
}

async fn setup_pool() -> PgPool {
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://pacs_extension_admin:PacsExtension2024@localhost:5456/pacs_extension".to_string());

    PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create pool")
}

#[tokio::test]
#[ignore] // DATABASE_URL 환경 변수 필요
async fn test_has_global_dicom_access_super_admin() -> sqlx::Result<()> {
    let pool = setup_pool().await;
    // Given: SUPER_ADMIN 사용자 생성
    let user_id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO security_user (username, email, keycloak_id, created_at, updated_at)
         VALUES ('super_admin_user', 'super@test.com', gen_random_uuid(), NOW(), NOW())
         RETURNING id"
    )
    .fetch_one(&pool)
    .await?;

    // SUPER_ADMIN Role 조회
    let super_admin_role_id = sqlx::query_scalar::<_, i32>(
        "SELECT id FROM security_role WHERE name = 'SUPER_ADMIN'"
    )
    .fetch_one(&pool)
    .await?;

    // 프로젝트 생성
    let project_id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO security_project (name, description, created_at, updated_at)
         VALUES ('Test Project', 'Test', NOW(), NOW())
         RETURNING id"
    )
    .fetch_one(&pool)
    .await?;

    // 사용자에게 SUPER_ADMIN Role 할당
    sqlx::query(
        "INSERT INTO security_user_project (user_id, project_id, role_id, created_at, updated_at)
         VALUES ($1, $2, $3, NOW(), NOW())"
    )
    .bind(user_id)
    .bind(project_id)
    .bind(super_admin_role_id)
    .execute(&pool)
    .await?;

    // When: 전체 권한 확인
    let has_access = has_global_dicom_access(user_id, &pool).await;

    // Then: 권한 있음
    assert!(has_access, "SUPER_ADMIN should have global DICOM access");

    Ok(())
}

#[tokio::test]
#[ignore] // DATABASE_URL 환경 변수 필요
async fn test_has_global_dicom_access_admin() -> sqlx::Result<()> {
    let pool = setup_pool().await;
    // Given: ADMIN 사용자 생성
    let user_id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO security_user (username, email, keycloak_id, created_at, updated_at)
         VALUES ('admin_user', 'admin@test.com', gen_random_uuid(), NOW(), NOW())
         RETURNING id"
    )
    .fetch_one(&pool)
    .await?;

    // ADMIN Role 조회
    let admin_role_id = sqlx::query_scalar::<_, i32>(
        "SELECT id FROM security_role WHERE name = 'ADMIN'"
    )
    .fetch_one(&pool)
    .await?;

    // 프로젝트 생성
    let project_id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO security_project (name, description, created_at, updated_at)
         VALUES ('Test Project 2', 'Test', NOW(), NOW())
         RETURNING id"
    )
    .fetch_one(&pool)
    .await?;

    // 사용자에게 ADMIN Role 할당
    sqlx::query(
        "INSERT INTO security_user_project (user_id, project_id, role_id, created_at, updated_at)
         VALUES ($1, $2, $3, NOW(), NOW())"
    )
    .bind(user_id)
    .bind(project_id)
    .bind(admin_role_id)
    .execute(&pool)
    .await?;

    // When: 전체 권한 확인
    let has_access = has_global_dicom_access(user_id, &pool).await;

    // Then: 권한 있음
    assert!(has_access, "ADMIN should have global DICOM access");

    Ok(())
}

#[tokio::test]
#[ignore] // DATABASE_URL 환경 변수 필요
async fn test_has_global_dicom_access_regular_user() -> sqlx::Result<()> {
    let pool = setup_pool().await;
    // Given: 일반 사용자 생성
    let user_id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO security_user (username, email, keycloak_id, created_at, updated_at)
         VALUES ('regular_user', 'user@test.com', gen_random_uuid(), NOW(), NOW())
         RETURNING id"
    )
    .fetch_one(&pool)
    .await?;

    // USER Role 조회
    let user_role_id = sqlx::query_scalar::<_, i32>(
        "SELECT id FROM security_role WHERE name = 'USER'"
    )
    .fetch_one(&pool)
    .await?;

    // 프로젝트 생성
    let project_id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO security_project (name, description, created_at, updated_at)
         VALUES ('Test Project 3', 'Test', NOW(), NOW())
         RETURNING id"
    )
    .fetch_one(&pool)
    .await?;

    // 사용자에게 USER Role 할당
    sqlx::query(
        "INSERT INTO security_user_project (user_id, project_id, role_id, created_at, updated_at)
         VALUES ($1, $2, $3, NOW(), NOW())"
    )
    .bind(user_id)
    .bind(project_id)
    .bind(user_role_id)
    .execute(&pool)
    .await?;

    // When: 전체 권한 확인
    let has_access = has_global_dicom_access(user_id, &pool).await;

    // Then: 권한 없음
    assert!(!has_access, "Regular USER should NOT have global DICOM access");

    Ok(())
}

#[tokio::test]
#[ignore] // DATABASE_URL 환경 변수 필요
async fn test_has_global_dicom_access_nonexistent_user() -> sqlx::Result<()> {
    let pool = setup_pool().await;
    // Given: 존재하지 않는 사용자 ID
    let nonexistent_user_id = 999999;

    // When: 전체 권한 확인
    let has_access = has_global_dicom_access(nonexistent_user_id, &pool).await;

    // Then: 권한 없음
    assert!(!has_access, "Nonexistent user should NOT have global DICOM access");

    Ok(())
}

#[tokio::test]
#[ignore] // DATABASE_URL 환경 변수 필요
async fn test_has_global_dicom_access_project_admin() -> sqlx::Result<()> {
    let pool = setup_pool().await;
    // Given: PROJECT_ADMIN 사용자 생성
    let user_id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO security_user (username, email, keycloak_id, created_at, updated_at)
         VALUES ('project_admin_user', 'padmin@test.com', gen_random_uuid(), NOW(), NOW())
         RETURNING id"
    )
    .fetch_one(&pool)
    .await?;

    // PROJECT_ADMIN Role 조회
    let project_admin_role_id = sqlx::query_scalar::<_, i32>(
        "SELECT id FROM security_role WHERE name = 'PROJECT_ADMIN'"
    )
    .fetch_one(&pool)
    .await?;

    // 프로젝트 생성
    let project_id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO security_project (name, description, created_at, updated_at)
         VALUES ('Test Project 4', 'Test', NOW(), NOW())
         RETURNING id"
    )
    .fetch_one(&pool)
    .await?;

    // 사용자에게 PROJECT_ADMIN Role 할당
    sqlx::query(
        "INSERT INTO security_user_project (user_id, project_id, role_id, created_at, updated_at)
         VALUES ($1, $2, $3, NOW(), NOW())"
    )
    .bind(user_id)
    .bind(project_id)
    .bind(project_admin_role_id)
    .execute(&pool)
    .await?;

    // When: 전체 권한 확인
    let has_access = has_global_dicom_access(user_id, &pool).await;

    // Then: 권한 없음 (PROJECT_ADMIN은 전체 권한 없음)
    assert!(!has_access, "PROJECT_ADMIN should NOT have global DICOM access");

    Ok(())
}

#[tokio::test]
#[ignore] // DATABASE_URL 환경 변수 필요
async fn test_has_global_dicom_access_viewer() -> sqlx::Result<()> {
    let pool = setup_pool().await;
    // Given: VIEWER 사용자 생성
    let user_id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO security_user (username, email, keycloak_id, created_at, updated_at)
         VALUES ('viewer_user', 'viewer@test.com', gen_random_uuid(), NOW(), NOW())
         RETURNING id"
    )
    .fetch_one(&pool)
    .await?;

    // VIEWER Role 조회
    let viewer_role_id = sqlx::query_scalar::<_, i32>(
        "SELECT id FROM security_role WHERE name = 'VIEWER'"
    )
    .fetch_one(&pool)
    .await?;

    // 프로젝트 생성
    let project_id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO security_project (name, description, created_at, updated_at)
         VALUES ('Test Project 5', 'Test', NOW(), NOW())
         RETURNING id"
    )
    .fetch_one(&pool)
    .await?;

    // 사용자에게 VIEWER Role 할당
    sqlx::query(
        "INSERT INTO security_user_project (user_id, project_id, role_id, created_at, updated_at)
         VALUES ($1, $2, $3, NOW(), NOW())"
    )
    .bind(user_id)
    .bind(project_id)
    .bind(viewer_role_id)
    .execute(&pool)
    .await?;

    // When: 전체 권한 확인
    let has_access = has_global_dicom_access(user_id, &pool).await;

    // Then: 권한 없음
    assert!(!has_access, "VIEWER should NOT have global DICOM access");

    Ok(())
}

