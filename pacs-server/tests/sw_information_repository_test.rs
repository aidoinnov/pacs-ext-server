//! SW Information Repository 통합 테스트

use pacs_server::domain::sw_information::SwInformationRepository;
use pacs_server::infrastructure::sw_information::repositories::SwInformationRepositoryImpl;
use sqlx::PgPool;
use std::sync::Arc;

async fn get_test_pool() -> PgPool {
    let database_url = std::env::var("APP_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .unwrap_or_else(|_| {
            "postgres://pacs_extension_admin:PacsExtension2024@localhost:5456/pacs_extension"
                .to_string()
        });

    PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

#[tokio::test]
async fn test_find_all_returns_items() {
    let pool = get_test_pool().await;
    let repo = Arc::new(SwInformationRepositoryImpl::new(pool));

    let result = repo.find_all().await;
    assert!(result.is_ok(), "find_all should succeed");

    let items = result.unwrap();
    // Migration seed inserts 1 record when table is empty
    assert!(!items.is_empty(), "Should have at least 1 SW Information record");
}

#[tokio::test]
async fn test_find_by_id_returns_some_when_exists() {
    let pool = get_test_pool().await;
    let repo = Arc::new(SwInformationRepositoryImpl::new(pool));

    let all = repo.find_all().await.unwrap();
    assert!(!all.is_empty(), "Need seed data");

    let id = all[0].id;
    let result = repo.find_by_id(id).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_some());
}

#[tokio::test]
async fn test_find_by_id_returns_none_when_not_exists() {
    let pool = get_test_pool().await;
    let repo = Arc::new(SwInformationRepositoryImpl::new(pool));

    let result = repo.find_by_id(99999).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}
