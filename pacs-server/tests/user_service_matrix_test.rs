use std::sync::Arc;
use sqlx::PgPool;
use pacs_server::{
    domain::{
        entities::User,
        services::user_service::UserServiceImpl,
    },
    infrastructure::repositories::{
        user_repository_impl::UserRepositoryImpl,
        project_repository_impl::ProjectRepositoryImpl,
    },
};

/// UserService의 필터링 메서드 테스트
#[tokio::test]
async fn test_get_users_with_filter() {
    // Given: 테스트 데이터베이스 설정
    let pool = setup_test_database().await;
    let user_repository = Arc::new(UserRepositoryImpl::new(pool.clone()));
    let project_repository = Arc::new(ProjectRepositoryImpl::new(pool.clone()));
    
    let user_service = UserServiceImpl::new(
        user_repository.clone(),
        project_repository.clone(),
    );
    
    // 테스트 데이터 생성
    create_test_users(&pool).await;
    
    // When: 필터 없이 전체 사용자 조회
    let (users, total_count) = user_service
        .get_users_with_filter(
            None,
            1,
            10,
        )
        .await
        .unwrap();
    
    // Then: 모든 사용자가 반환되어야 함
    assert!(!users.is_empty());
    assert!(total_count > 0);
}

/// 특정 사용자 ID 필터링 테스트
#[tokio::test]
async fn test_get_users_with_user_ids_filter() {
    let pool = setup_test_database().await;
    let user_repository = Arc::new(UserRepositoryImpl::new(pool.clone()));
    let project_repository = Arc::new(ProjectRepositoryImpl::new(pool.clone()));
    
    let user_service = UserServiceImpl::new(
        user_repository.clone(),
        project_repository.clone(),
    );
    
    // 특정 사용자 ID들 생성
    let user_ids = create_test_users_with_ids(&pool).await;
    
    // When: 특정 사용자 ID들만 조회
    let (users, total_count) = user_service
        .get_users_with_filter(
            Some(user_ids.clone()),
            1,
            10,
        )
        .await
        .unwrap();
    
    // Then: 지정된 사용자 ID들만 반환되어야 함
    assert_eq!(users.len(), user_ids.len());
    assert_eq!(total_count as usize, user_ids.len());
    
    for user in &users {
        assert!(user_ids.contains(&user.id));
    }
}

/// 사용자 페이지네이션 테스트
#[tokio::test]
async fn test_get_users_pagination() {
    let pool = setup_test_database().await;
    let user_repository = Arc::new(UserRepositoryImpl::new(pool.clone()));
    let project_repository = Arc::new(ProjectRepositoryImpl::new(pool.clone()));
    
    let user_service = UserServiceImpl::new(
        user_repository.clone(),
        project_repository.clone(),
    );
    
    create_test_users(&pool).await;
    
    // When: 첫 번째 페이지 조회 (페이지 크기 2)
    let (users_page1, total_count) = user_service
        .get_users_with_filter(
            None,
            1,
            2,
        )
        .await
        .unwrap();
    
    // 두 번째 페이지 조회
    let (users_page2, _) = user_service
        .get_users_with_filter(
            None,
            2,
            2,
        )
        .await
        .unwrap();
    
    // Then: 페이지네이션이 올바르게 작동해야 함
    assert!(users_page1.len() <= 2);
    assert!(users_page2.len() <= 2);
    assert!(total_count > 0);
    
    // 첫 번째 페이지와 두 번째 페이지의 사용자가 중복되지 않아야 함
    let page1_ids: std::collections::HashSet<i32> = users_page1
        .iter()
        .map(|u| u.id)
        .collect();
    let page2_ids: std::collections::HashSet<i32> = users_page2
        .iter()
        .map(|u| u.id)
        .collect();
    
    assert!(page1_ids.is_disjoint(&page2_ids), "페이지 간 중복이 없어야 함");
}

/// 빈 결과 테스트
#[tokio::test]
async fn test_get_users_empty_result() {
    let pool = setup_test_database().await;
    let user_repository = Arc::new(UserRepositoryImpl::new(pool.clone()));
    let project_repository = Arc::new(ProjectRepositoryImpl::new(pool.clone()));
    
    let user_service = UserServiceImpl::new(
        user_repository.clone(),
        project_repository.clone(),
    );
    
    // When: 존재하지 않는 사용자 ID로 조회
    let (users, total_count) = user_service
        .get_users_with_filter(
            Some(vec![99999, 99998]), // 존재하지 않는 ID
            1,
            10,
        )
        .await
        .unwrap();
    
    // Then: 빈 결과가 반환되어야 함
    assert!(users.is_empty());
    assert_eq!(total_count, 0);
}

/// 대용량 데이터 페이지네이션 테스트
#[tokio::test]
async fn test_get_users_large_dataset_pagination() {
    let pool = setup_test_database().await;
    let user_repository = Arc::new(UserRepositoryImpl::new(pool.clone()));
    let project_repository = Arc::new(ProjectRepositoryImpl::new(pool.clone()));
    
    let user_service = UserServiceImpl::new(
        user_repository.clone(),
        project_repository.clone(),
    );
    
    // 대량의 테스트 사용자 생성
    create_large_test_users(&pool, 25).await;
    
    // When: 페이지 크기 10으로 조회
    let (users_page1, total_count) = user_service
        .get_users_with_filter(
            None,
            1,
            10,
        )
        .await
        .unwrap();
    
    let (users_page2, _) = user_service
        .get_users_with_filter(
            None,
            2,
            10,
        )
        .await
        .unwrap();
    
    let (users_page3, _) = user_service
        .get_users_with_filter(
            None,
            3,
            10,
        )
        .await
        .unwrap();
    
    // Then: 페이지네이션이 올바르게 작동해야 함
    assert_eq!(users_page1.len(), 10);
    assert_eq!(users_page2.len(), 10);
    assert!(users_page3.len() <= 10); // 마지막 페이지는 5개 이하
    assert_eq!(total_count, 25);
    
    // 모든 페이지의 사용자 ID가 중복되지 않아야 함
    let mut all_ids = std::collections::HashSet::new();
    
    for user in &users_page1 {
        assert!(all_ids.insert(user.id), "중복된 사용자 ID가 있음");
    }
    for user in &users_page2 {
        assert!(all_ids.insert(user.id), "중복된 사용자 ID가 있음");
    }
    for user in &users_page3 {
        assert!(all_ids.insert(user.id), "중복된 사용자 ID가 있음");
    }
}

/// 사용자 정렬 테스트
#[tokio::test]
async fn test_get_users_ordering() {
    let pool = setup_test_database().await;
    let user_repository = Arc::new(UserRepositoryImpl::new(pool.clone()));
    let project_repository = Arc::new(ProjectRepositoryImpl::new(pool.clone()));
    
    let user_service = UserServiceImpl::new(
        user_repository.clone(),
        project_repository.clone(),
    );
    
    // 정렬 테스트용 사용자 생성 (특정 username 순서)
    create_ordered_test_users(&pool).await;
    
    // When: 사용자 조회
    let (users, _) = user_service
        .get_users_with_filter(
            None,
            1,
            10,
        )
        .await
        .unwrap();
    
    // Then: username 순으로 정렬되어야 함
    for i in 1..users.len() {
        assert!(
            users[i-1].username <= users[i].username,
            "사용자가 username 순으로 정렬되지 않음: {} > {}",
            users[i-1].username,
            users[i].username
        );
    }
}

/// 테스트 데이터베이스 설정
async fn setup_test_database() -> PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://admin:admin123@localhost:5432/pacs_db".to_string());
    
    sqlx::PgPool::connect(&database_url).await.unwrap()
}

/// 테스트용 사용자 생성
async fn create_test_users(pool: &PgPool) {
    let users = vec![
        ("testuser1", "user1@example.com"),
        ("testuser2", "user2@example.com"),
        ("testuser3", "user3@example.com"),
        ("testuser4", "user4@example.com"),
        ("testuser5", "user5@example.com"),
    ];
    
    for (username, email) in users {
        sqlx::query!(
            "INSERT INTO security_user (keycloak_id, username, email) 
             VALUES (gen_random_uuid(), $1, $2)",
            username,
            email
        )
        .execute(pool)
        .await
        .unwrap();
    }
}

/// 특정 ID를 가진 테스트 사용자 생성
async fn create_test_users_with_ids(pool: &PgPool) -> Vec<i32> {
    let users = vec![
        ("filter_user1", "filter1@example.com"),
        ("filter_user2", "filter2@example.com"),
    ];
    
    let mut user_ids = Vec::new();
    
    for (username, email) in users {
        let result = sqlx::query!(
            "INSERT INTO security_user (keycloak_id, username, email) 
             VALUES (gen_random_uuid(), $1, $2) 
             RETURNING id",
            username,
            email
        )
        .fetch_one(pool)
        .await
        .unwrap();
        
        user_ids.push(result.id);
    }
    
    user_ids
}

/// 대량의 테스트 사용자 생성
async fn create_large_test_users(pool: &PgPool, count: i32) {
    for i in 1..=count {
        sqlx::query!(
            "INSERT INTO security_user (keycloak_id, username, email) 
             VALUES (gen_random_uuid(), $1, $2)",
            format!("bulk_user_{:03}", i),
            format!("bulk{}@example.com", i)
        )
        .execute(pool)
        .await
        .unwrap();
    }
}

/// 정렬 테스트용 사용자 생성
async fn create_ordered_test_users(pool: &PgPool) {
    // 역순으로 사용자 생성 (정렬 테스트용)
    let users = vec![
        ("z_user", "z@example.com"),
        ("a_user", "a@example.com"),
        ("m_user", "m@example.com"),
    ];
    
    for (username, email) in users {
        sqlx::query!(
            "INSERT INTO security_user (keycloak_id, username, email) 
             VALUES (gen_random_uuid(), $1, $2)",
            username,
            email
        )
        .execute(pool)
        .await
        .unwrap();
    }
}
