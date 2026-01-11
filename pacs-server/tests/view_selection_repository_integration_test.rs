use pacs_server::domain::view_selection::{ViewSelection, SelectedSeries};
use pacs_server::domain::view_selection::repositories::ViewSelectionRepository;
use pacs_server::infrastructure::redis::RedisClientFactory;
use pacs_server::infrastructure::view_selection::ViewSelectionRepositoryImpl;
use std::sync::Arc;

async fn get_redis_connection() -> Option<Arc<pacs_server::infrastructure::redis::RedisConnection>> {
    let redis_url = std::env::var("APP_REDIS__URL")
        .or_else(|_| std::env::var("REDIS_URL"))
        .unwrap_or_else(|_| "redis://localhost:6379".to_string());

    match RedisClientFactory::create(&redis_url).await {
        Ok(conn) => Some(Arc::new(conn)),
        Err(e) => {
            eprintln!("⚠️  Redis connection failed: {} - Skipping integration tests", e);
            None
        }
    }
}

#[tokio::test]
async fn test_save_and_find_selection() {
    let redis_conn = match get_redis_connection().await {
        Some(conn) => conn,
        None => {
            eprintln!("Skipping test - Redis not available");
            return;
        }
    };

    let repo = ViewSelectionRepositoryImpl::new(redis_conn, Some("test_view_selection:".to_string()));
    
    let selection = ViewSelection::new(
        "sel_test123".to_string(),
        vec![
            SelectedSeries {
                study_uid: "1.2.3".to_string(),
                series_uid: "1.2.3.4".to_string(),
            },
        ],
        1,
        1800,
    );

    // 저장
    let result = repo.save(&selection).await;
    assert!(result.is_ok(), "Failed to save selection: {:?}", result.err());

    // 조회
    let found = repo.find_by_id("sel_test123").await;
    assert!(found.is_ok(), "Failed to find selection: {:?}", found.err());
    
    let found_selection = found.unwrap();
    assert!(found_selection.is_some(), "Selection not found after save");
    
    let found_selection = found_selection.unwrap();
    assert_eq!(found_selection.selection_id, "sel_test123");
    assert_eq!(found_selection.series.len(), 1);
    assert_eq!(found_selection.user_id, 1);

    // 정리
    let _ = repo.delete("sel_test123").await;
}

#[tokio::test]
async fn test_find_nonexistent_selection() {
    let redis_conn = match get_redis_connection().await {
        Some(conn) => conn,
        None => {
            eprintln!("Skipping test - Redis not available");
            return;
        }
    };

    let repo = ViewSelectionRepositoryImpl::new(redis_conn, Some("test_view_selection:".to_string()));

    let found = repo.find_by_id("sel_nonexistent").await;
    assert!(found.is_ok());
    assert!(found.unwrap().is_none());
}

#[tokio::test]
async fn test_extend_ttl() {
    let redis_conn = match get_redis_connection().await {
        Some(conn) => conn,
        None => {
            eprintln!("Skipping test - Redis not available");
            return;
        }
    };

    let repo = ViewSelectionRepositoryImpl::new(redis_conn, Some("test_view_selection:".to_string()));
    
    let selection = ViewSelection::new(
        "sel_test_ttl".to_string(),
        vec![SelectedSeries {
            study_uid: "1.2.3".to_string(),
            series_uid: "1.2.3.4".to_string(),
        }],
        1,
        1800,
    );

    // 저장
    repo.save(&selection).await.unwrap();

    // TTL 연장
    let result = repo.extend_ttl("sel_test_ttl", 3600).await;
    assert!(result.is_ok(), "Failed to extend TTL: {:?}", result.err());

    // 조회하여 TTL이 연장되었는지 확인
    let found = repo.find_by_id("sel_test_ttl").await.unwrap().unwrap();
    let diff = (found.expires_at - found.created_at).num_seconds();
    assert!(diff >= 3600, "TTL was not extended properly");

    // 정리
    let _ = repo.delete("sel_test_ttl").await;
}

#[tokio::test]
async fn test_extend_ttl_not_found() {
    let redis_conn = match get_redis_connection().await {
        Some(conn) => conn,
        None => {
            eprintln!("Skipping test - Redis not available");
            return;
        }
    };

    let repo = ViewSelectionRepositoryImpl::new(redis_conn, Some("test_view_selection:".to_string()));

    let result = repo.extend_ttl("sel_nonexistent", 3600).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[tokio::test]
async fn test_delete_selection() {
    let redis_conn = match get_redis_connection().await {
        Some(conn) => conn,
        None => {
            eprintln!("Skipping test - Redis not available");
            return;
        }
    };

    let repo = ViewSelectionRepositoryImpl::new(redis_conn, Some("test_view_selection:".to_string()));
    
    let selection = ViewSelection::new(
        "sel_test_delete".to_string(),
        vec![SelectedSeries {
            study_uid: "1.2.3".to_string(),
            series_uid: "1.2.3.4".to_string(),
        }],
        1,
        1800,
    );

    // 저장
    repo.save(&selection).await.unwrap();

    // 삭제
    let result = repo.delete("sel_test_delete").await;
    assert!(result.is_ok(), "Failed to delete selection: {:?}", result.err());

    // 조회하여 삭제되었는지 확인
    let found = repo.find_by_id("sel_test_delete").await.unwrap();
    assert!(found.is_none(), "Selection should be deleted");
}

#[tokio::test]
async fn test_expired_selection_auto_delete() {
    let redis_conn = match get_redis_connection().await {
        Some(conn) => conn,
        None => {
            eprintln!("Skipping test - Redis not available");
            return;
        }
    };

    let repo = ViewSelectionRepositoryImpl::new(redis_conn, Some("test_view_selection:".to_string()));
    
    // 만료된 Selection 생성 (TTL 1초)
    let mut selection = ViewSelection::new(
        "sel_test_expired".to_string(),
        vec![SelectedSeries {
            study_uid: "1.2.3".to_string(),
            series_uid: "1.2.3.4".to_string(),
        }],
        1,
        1, // 1초 TTL
    );

    // 강제로 만료 시각을 과거로 설정
    use chrono::{Duration, Utc};
    selection.expires_at = Utc::now() - Duration::seconds(1);

    // 저장
    repo.save(&selection).await.unwrap();

    // 조회 시 만료된 Selection은 자동으로 삭제되어야 함
    let found = repo.find_by_id("sel_test_expired").await.unwrap();
    assert!(found.is_none(), "Expired selection should be auto-deleted");
}


