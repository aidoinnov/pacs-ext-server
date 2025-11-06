use std::sync::Arc;
use sqlx::PgPool;
use pacs_server::{
    domain::{
        entities::ProjectStatus,
        services::project_service::ProjectServiceImpl,
    },
    infrastructure::repositories::{
        ProjectRepositoryImpl,
        UserRepositoryImpl,
        RoleRepositoryImpl,
    },
};

/// ProjectService의 상태 필터링 메서드 테스트
#[tokio::test]
async fn test_get_projects_with_status_filter() {
    // Given: 테스트 데이터베이스 설정
    let pool = setup_test_database().await;
    let project_repository = Arc::new(ProjectRepositoryImpl::new(pool.clone()));
    let user_repository = Arc::new(UserRepositoryImpl::new(pool.clone()));
    let role_repository = Arc::new(RoleRepositoryImpl::new(pool.clone()));
    
    let project_service = ProjectServiceImpl::new(
        project_repository.clone(),
        user_repository.clone(),
        role_repository.clone(),
    );
    
    // 테스트 데이터 생성
    create_test_projects(&pool).await;
    
    // When: IN_PROGRESS 상태의 프로젝트만 조회
    let (projects, total_count) = project_service
        .get_projects_with_status_filter(
            Some(vec![ProjectStatus::InProgress]),
            None,
            1,
            10,
        )
        .await
        .unwrap();
    
    // Then: IN_PROGRESS 상태의 프로젝트만 반환되어야 함
    assert!(!projects.is_empty());
    for project in &projects {
        assert_eq!(project.status, ProjectStatus::InProgress);
    }
    
    // 총 개수가 예상과 일치하는지 확인
    assert!(total_count > 0);
}

/// 여러 상태 필터링 테스트
#[tokio::test]
async fn test_get_projects_with_multiple_status_filter() {
    let pool = setup_test_database().await;
    let project_repository = Arc::new(ProjectRepositoryImpl::new(pool.clone()));
    let user_repository = Arc::new(UserRepositoryImpl::new(pool.clone()));
    let role_repository = Arc::new(RoleRepositoryImpl::new(pool.clone()));
    
    let project_service = ProjectServiceImpl::new(
        project_repository.clone(),
        user_repository.clone(),
        role_repository.clone(),
    );
    
    create_test_projects(&pool).await;
    
    // When: IN_PROGRESS와 COMPLETED 상태의 프로젝트 조회
    let (projects, total_count) = project_service
        .get_projects_with_status_filter(
            Some(vec![ProjectStatus::InProgress, ProjectStatus::Completed]),
            None,
            1,
            10,
        )
        .await
        .unwrap();
    
    // Then: 해당 상태의 프로젝트만 반환
    for project in &projects {
        assert!(
            project.status == ProjectStatus::InProgress || 
            project.status == ProjectStatus::Completed
        );
    }
    
    assert!(total_count > 0);
}

/// 상태 필터 없이 전체 조회 테스트
#[tokio::test]
async fn test_get_projects_without_status_filter() {
    let pool = setup_test_database().await;
    let project_repository = Arc::new(ProjectRepositoryImpl::new(pool.clone()));
    let user_repository = Arc::new(UserRepositoryImpl::new(pool.clone()));
    let role_repository = Arc::new(RoleRepositoryImpl::new(pool.clone()));
    
    let project_service = ProjectServiceImpl::new(
        project_repository.clone(),
        user_repository.clone(),
        role_repository.clone(),
    );
    
    create_test_projects(&pool).await;
    
    // When: 상태 필터 없이 전체 조회
    let (projects, total_count) = project_service
        .get_projects_with_status_filter(
            None,
            None,
            1,
            10,
        )
        .await
        .unwrap();
    
    // Then: 모든 상태의 프로젝트가 반환되어야 함
    assert!(!projects.is_empty());
    assert!(total_count > 0);
    
    // 다양한 상태가 포함되어 있는지 확인
    let statuses: std::collections::HashSet<ProjectStatus> = projects
        .iter()
        .map(|p| p.status)
        .collect();
    
    assert!(statuses.len() > 1, "다양한 상태의 프로젝트가 있어야 함");
}

/// 프로젝트 ID 필터링 테스트
#[tokio::test]
async fn test_get_projects_with_project_ids_filter() {
    let pool = setup_test_database().await;
    let project_repository = Arc::new(ProjectRepositoryImpl::new(pool.clone()));
    let user_repository = Arc::new(UserRepositoryImpl::new(pool.clone()));
    let role_repository = Arc::new(RoleRepositoryImpl::new(pool.clone()));
    
    let project_service = ProjectServiceImpl::new(
        project_repository.clone(),
        user_repository.clone(),
        role_repository.clone(),
    );
    
    // 특정 프로젝트 ID들 생성
    let project_ids = create_test_projects_with_ids(&pool).await;
    
    // When: 특정 프로젝트 ID들만 조회
    let (projects, total_count) = project_service
        .get_projects_with_status_filter(
            None,
            Some(project_ids.clone()),
            1,
            10,
        )
        .await
        .unwrap();
    
    // Then: 지정된 프로젝트 ID들만 반환되어야 함
    assert_eq!(projects.len(), project_ids.len());
    assert_eq!(total_count as usize, project_ids.len());
    
    for project in &projects {
        assert!(project_ids.contains(&project.id));
    }
}

/// 페이지네이션 테스트
#[tokio::test]
async fn test_get_projects_pagination() {
    let pool = setup_test_database().await;
    let project_repository = Arc::new(ProjectRepositoryImpl::new(pool.clone()));
    let user_repository = Arc::new(UserRepositoryImpl::new(pool.clone()));
    let role_repository = Arc::new(RoleRepositoryImpl::new(pool.clone()));
    
    let project_service = ProjectServiceImpl::new(
        project_repository.clone(),
        user_repository.clone(),
        role_repository.clone(),
    );
    
    create_test_projects(&pool).await;
    
    // When: 첫 번째 페이지 조회 (페이지 크기 2)
    let (projects_page1, total_count) = project_service
        .get_projects_with_status_filter(
            None,
            None,
            1,
            2,
        )
        .await
        .unwrap();
    
    // 두 번째 페이지 조회
    let (projects_page2, _) = project_service
        .get_projects_with_status_filter(
            None,
            None,
            2,
            2,
        )
        .await
        .unwrap();
    
    // Then: 페이지네이션이 올바르게 작동해야 함
    assert!(projects_page1.len() <= 2);
    assert!(projects_page2.len() <= 2);
    assert!(total_count > 0);
    
    // 첫 번째 페이지와 두 번째 페이지의 프로젝트가 중복되지 않아야 함
    let page1_ids: std::collections::HashSet<i32> = projects_page1
        .iter()
        .map(|p| p.id)
        .collect();
    let page2_ids: std::collections::HashSet<i32> = projects_page2
        .iter()
        .map(|p| p.id)
        .collect();
    
    assert!(page1_ids.is_disjoint(&page2_ids), "페이지 간 중복이 없어야 함");
}

/// 사용자-프로젝트-역할 매트릭스 조회 테스트
#[tokio::test]
async fn test_get_user_project_roles_matrix() {
    let pool = setup_test_database().await;
    let project_repository = Arc::new(ProjectRepositoryImpl::new(pool.clone()));
    let user_repository = Arc::new(UserRepositoryImpl::new(pool.clone()));
    let role_repository = Arc::new(RoleRepositoryImpl::new(pool.clone()));
    
    let project_service = ProjectServiceImpl::new(
        project_repository.clone(),
        user_repository.clone(),
        role_repository.clone(),
    );
    
    // 테스트 데이터 생성 (프로젝트, 사용자, 역할 관계)
    let (project_ids, user_ids) = create_test_matrix_data(&pool).await;
    
    // When: 매트릭스 조회
    let relationships = project_service
        .get_user_project_roles_matrix(project_ids.clone(), user_ids.clone())
        .await
        .unwrap();
    
    // Then: 모든 프로젝트-사용자 조합이 반환되어야 함
    assert_eq!(relationships.len(), project_ids.len() * user_ids.len());
    
    // 각 관계가 올바른 프로젝트와 사용자를 참조하는지 확인
    for relationship in &relationships {
        assert!(project_ids.contains(&relationship.project_id));
        assert!(user_ids.contains(&relationship.user_id));
    }
}

/// 테스트 데이터베이스 설정
async fn setup_test_database() -> PgPool {
    // 실제 구현에서는 테스트용 데이터베이스 URL을 사용
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://admin:admin123@localhost:5432/pacs_db".to_string());
    
    sqlx::PgPool::connect(&database_url).await.unwrap()
}

/// 테스트용 프로젝트 생성
async fn create_test_projects(pool: &PgPool) {
    let projects = vec![
        ("Test Project 1", "IN_PROGRESS"),
        ("Test Project 2", "COMPLETED"),
        ("Test Project 3", "PREPARING"),
        ("Test Project 4", "IN_PROGRESS"),
        ("Test Project 5", "ON_HOLD"),
    ];
    
    for (name, status) in projects {
        sqlx::query!(
            "INSERT INTO security_project (name, description, is_active, status) 
             VALUES ($1, 'Test Description', true, $2)",
            name,
            status
        )
        .execute(pool)
        .await
        .unwrap();
    }
}

/// 특정 ID를 가진 테스트 프로젝트 생성
async fn create_test_projects_with_ids(pool: &PgPool) -> Vec<i32> {
    let projects = vec![
        ("Test Project A", "IN_PROGRESS"),
        ("Test Project B", "COMPLETED"),
    ];
    
    let mut project_ids = Vec::new();
    
    for (name, status) in projects {
        let result =         sqlx::query!(
            "INSERT INTO security_project (name, description, is_active, status) 
             VALUES ($1, 'Test Description', true, $2) 
             RETURNING id",
            name,
            status
        )
        .fetch_one(pool)
        .await
        .unwrap();
        
        project_ids.push(result.id);
    }
    
    project_ids
}

/// 매트릭스 테스트 데이터 생성
async fn create_test_matrix_data(pool: &PgPool) -> (Vec<i32>, Vec<i32>) {
    // 프로젝트 생성
    let project_result = sqlx::query!(
        "INSERT INTO security_project (name, description, is_active, status) 
         VALUES ('Matrix Test Project', 'Test Description', true, 'IN_PROGRESS') 
         RETURNING id"
    )
    .fetch_one(pool)
    .await
    .unwrap();
    
    let project_id = project_result.id;
    
    // 사용자 생성
    let user_result1 = sqlx::query!(
        "INSERT INTO security_user (keycloak_id, username, email) 
         VALUES (gen_random_uuid(), 'matrix_user1', 'matrix1@example.com') 
         RETURNING id"
    )
    .fetch_one(pool)
    .await
    .unwrap();
    
    let user_result2 = sqlx::query!(
        "INSERT INTO security_user (keycloak_id, username, email) 
         VALUES (gen_random_uuid(), 'matrix_user2', 'matrix2@example.com') 
         RETURNING id"
    )
    .fetch_one(pool)
    .await
    .unwrap();
    
    let user_id1 = user_result1.id;
    let user_id2 = user_result2.id;
    
    (vec![project_id], vec![user_id1, user_id2])
}
