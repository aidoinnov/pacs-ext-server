//! SW Information Use Case 통합 테스트

use pacs_server::application::use_cases::SwInformationUseCase;
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
async fn test_list_returns_success_response() {
    let pool = get_test_pool().await;
    let repo = Arc::new(SwInformationRepositoryImpl::new(pool));
    let use_case = Arc::new(SwInformationUseCase::new(repo));

    let result = use_case.list().await;
    assert!(result.is_ok());

    let response = result.unwrap();
    assert!(response.success);
    assert!(response.total_count >= 0);
}

#[tokio::test]
async fn test_get_by_id_returns_some_when_exists() {
    let pool = get_test_pool().await;
    let repo = Arc::new(SwInformationRepositoryImpl::new(pool));
    let use_case = Arc::new(SwInformationUseCase::new(repo));

    let list = use_case.list().await.unwrap();
    assert!(!list.items.is_empty());

    let id = list.items[0].id;
    let result = use_case.get_by_id(id).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_some());
}

#[tokio::test]
async fn test_get_by_id_returns_none_when_not_exists() {
    let pool = get_test_pool().await;
    let repo = Arc::new(SwInformationRepositoryImpl::new(pool));
    let use_case = Arc::new(SwInformationUseCase::new(repo));

    let result = use_case.get_by_id(99999).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}
