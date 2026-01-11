use pacs_server::application::dto::view_selection_dto::{CreateViewSelectionRequest, SelectedSeriesDto};
use pacs_server::application::use_cases::ViewSelectionUseCase;
use pacs_server::domain::view_selection::{SelectedSeries, ViewSelection};
use pacs_server::domain::view_selection::repositories::ViewSelectionRepository;
use pacs_server::domain::view_selection::services::ViewSelectionService;
use pacs_server::domain::ServiceError;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

// Mock Repository
struct MockViewSelectionRepository {
    storage: Arc<Mutex<HashMap<String, ViewSelection>>>,
}

impl MockViewSelectionRepository {
    fn new() -> Self {
        Self {
            storage: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl ViewSelectionRepository for MockViewSelectionRepository {
    async fn save(&self, selection: &ViewSelection) -> Result<(), String> {
        let mut storage = self.storage.lock().await;
        storage.insert(selection.selection_id.clone(), selection.clone());
        Ok(())
    }

    async fn find_by_id(&self, selection_id: &str) -> Result<Option<ViewSelection>, String> {
        let storage = self.storage.lock().await;
        Ok(storage.get(selection_id).cloned())
    }

    async fn extend_ttl(&self, selection_id: &str, ttl_sec: u64) -> Result<(), String> {
        let mut storage = self.storage.lock().await;
        if let Some(selection) = storage.get_mut(selection_id) {
            selection.extend_ttl(ttl_sec);
            Ok(())
        } else {
            Err("Selection not found".to_string())
        }
    }

    async fn delete(&self, selection_id: &str) -> Result<(), String> {
        let mut storage = self.storage.lock().await;
        storage.remove(selection_id);
        Ok(())
    }
}

// Mock Service
struct MockViewSelectionService {
    repo: Arc<MockViewSelectionRepository>,
    default_ttl: u64,
}

impl MockViewSelectionService {
    fn new(repo: Arc<MockViewSelectionRepository>, default_ttl: u64) -> Self {
        Self { repo, default_ttl }
    }
}

#[async_trait]
impl ViewSelectionService for MockViewSelectionService {
    async fn create_selection(
        &self,
        series: Vec<SelectedSeries>,
        user_id: i32,
        ttl_sec: u64,
    ) -> Result<ViewSelection, ServiceError> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let hex: String = (0..6)
            .map(|_| format!("{:x}", rng.gen_range(0..16)))
            .collect();
        let selection_id = format!("sel_{}", hex);
        
        let selection = ViewSelection::new(selection_id, series, user_id, ttl_sec);
        self.repo.save(&selection).await
            .map_err(|e| ServiceError::DatabaseError(e))?;
        Ok(selection)
    }

    async fn get_selection(&self, selection_id: &str) -> Result<Option<ViewSelection>, ServiceError> {
        self.repo.find_by_id(selection_id).await
            .map_err(|e| ServiceError::DatabaseError(e))
    }

    async fn extend_ttl(&self, selection_id: &str, ttl_sec: u64) -> Result<(), ServiceError> {
        self.repo.extend_ttl(selection_id, ttl_sec).await
            .map_err(|e| {
                if e.contains("not found") {
                    ServiceError::NotFound(format!("Selection {} not found", selection_id))
                } else {
                    ServiceError::DatabaseError(e)
                }
            })
    }

    async fn delete_selection(&self, selection_id: &str) -> Result<(), ServiceError> {
        self.repo.delete(selection_id).await
            .map_err(|e| ServiceError::DatabaseError(e))
    }
}

#[tokio::test]
async fn test_create_selection_success() {
    let repo = Arc::new(MockViewSelectionRepository::new());
    let service = Arc::new(MockViewSelectionService::new(repo.clone(), 1800));
    let use_case = ViewSelectionUseCase::new(service, 1800);

    let request = CreateViewSelectionRequest {
        series: vec![
            SelectedSeriesDto {
                study_uid: "1.2.3".to_string(),
                series_uid: "1.2.3.4".to_string(),
            },
            SelectedSeriesDto {
                study_uid: "2.3.4".to_string(),
                series_uid: "2.3.4.5".to_string(),
            },
        ],
    };

    let result = use_case.create_selection(request, 1, None).await;
    assert!(result.is_ok());
    
    let response = result.unwrap();
    assert!(response.selection_id.starts_with("sel_"));
    assert_eq!(response.selection_id.len(), 10); // "sel_" + 6 hex chars
}

#[tokio::test]
async fn test_create_selection_empty_series() {
    let repo = Arc::new(MockViewSelectionRepository::new());
    let service = Arc::new(MockViewSelectionService::new(repo, 1800));
    let use_case = ViewSelectionUseCase::new(service, 1800);

    let request = CreateViewSelectionRequest {
        series: vec![],
    };

    let result = use_case.create_selection(request, 1, None).await;
    assert!(result.is_err());
    
    if let Err(ServiceError::ValidationError(msg)) = result {
        assert!(msg.contains("empty"));
    } else {
        panic!("Expected ValidationError");
    }
}

#[tokio::test]
async fn test_get_selection_success() {
    let repo = Arc::new(MockViewSelectionRepository::new());
    let service = Arc::new(MockViewSelectionService::new(repo.clone(), 1800));
    let use_case = ViewSelectionUseCase::new(service.clone(), 1800);

    // 먼저 Selection 생성
    let request = CreateViewSelectionRequest {
        series: vec![SelectedSeriesDto {
            study_uid: "1.2.3".to_string(),
            series_uid: "1.2.3.4".to_string(),
        }],
    };

    let create_result = use_case.create_selection(request, 1, None).await.unwrap();
    let selection_id = create_result.selection_id;

    // Selection 조회
    let result = use_case.get_selection(&selection_id).await;
    assert!(result.is_ok());
    
    let selection = result.unwrap();
    assert!(selection.is_some());
    let selection = selection.unwrap();
    assert_eq!(selection.selection_id, selection_id);
    assert_eq!(selection.series.len(), 1);
    assert_eq!(selection.user_id, 1);
}

#[tokio::test]
async fn test_get_selection_not_found() {
    let repo = Arc::new(MockViewSelectionRepository::new());
    let service = Arc::new(MockViewSelectionService::new(repo, 1800));
    let use_case = ViewSelectionUseCase::new(service, 1800);

    let result = use_case.get_selection("sel_nonexistent").await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[tokio::test]
async fn test_extend_ttl_success() {
    let repo = Arc::new(MockViewSelectionRepository::new());
    let service = Arc::new(MockViewSelectionService::new(repo.clone(), 1800));
    let use_case = ViewSelectionUseCase::new(service.clone(), 1800);

    // Selection 생성
    let request = CreateViewSelectionRequest {
        series: vec![SelectedSeriesDto {
            study_uid: "1.2.3".to_string(),
            series_uid: "1.2.3.4".to_string(),
        }],
    };

    let create_result = use_case.create_selection(request, 1, None).await.unwrap();
    let selection_id = create_result.selection_id;

    // TTL 연장
    let result = use_case.extend_ttl(&selection_id, Some(3600)).await;
    assert!(result.is_ok());

    // 조회하여 TTL이 연장되었는지 확인
    let selection = use_case.get_selection(&selection_id).await.unwrap().unwrap();
    let diff = (selection.expires_at - selection.created_at).num_seconds();
    assert!(diff >= 3600);
}

#[tokio::test]
async fn test_extend_ttl_not_found() {
    let repo = Arc::new(MockViewSelectionRepository::new());
    let service = Arc::new(MockViewSelectionService::new(repo, 1800));
    let use_case = ViewSelectionUseCase::new(service, 1800);

    let result = use_case.extend_ttl("sel_nonexistent", Some(3600)).await;
    assert!(result.is_err());
    
    if let Err(ServiceError::NotFound(_)) = result {
        // Expected
    } else {
        panic!("Expected NotFound error");
    }
}

#[tokio::test]
async fn test_delete_selection_success() {
    let repo = Arc::new(MockViewSelectionRepository::new());
    let service = Arc::new(MockViewSelectionService::new(repo.clone(), 1800));
    let use_case = ViewSelectionUseCase::new(service.clone(), 1800);

    // Selection 생성
    let request = CreateViewSelectionRequest {
        series: vec![SelectedSeriesDto {
            study_uid: "1.2.3".to_string(),
            series_uid: "1.2.3.4".to_string(),
        }],
    };

    let create_result = use_case.create_selection(request, 1, None).await.unwrap();
    let selection_id = create_result.selection_id;

    // 삭제
    let result = use_case.delete_selection(&selection_id).await;
    assert!(result.is_ok());

    // 조회하여 삭제되었는지 확인
    let selection = use_case.get_selection(&selection_id).await.unwrap();
    assert!(selection.is_none());
}


