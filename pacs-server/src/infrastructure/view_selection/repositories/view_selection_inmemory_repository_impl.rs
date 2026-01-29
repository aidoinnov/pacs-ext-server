use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::domain::view_selection::repositories::ViewSelectionRepository;
use crate::domain::view_selection::ViewSelection;

/// In-memory ViewSelection Repository 구현체
/// 
/// ⚠️ **경고**: 이 구현체는 단일 서버 환경에서만 사용해야 합니다.
/// 여러 서버 인스턴스를 사용하는 경우, Redis 기반 구현체를 사용하세요.
/// 
/// - 서버 재시작 시 모든 데이터가 손실됩니다.
/// - 여러 서버 간 데이터 공유가 불가능합니다.
/// - TTL은 조회 시점에 확인됩니다 (백그라운드 정리 없음).
pub struct ViewSelectionInMemoryRepositoryImpl {
    /// In-memory 저장소 (selection_id -> ViewSelection)
    store: Arc<RwLock<HashMap<String, ViewSelection>>>,
    
    /// 키 접두사 (테스트용)
    key_prefix: String,
}

impl ViewSelectionInMemoryRepositoryImpl {
    /// 새로운 In-memory Repository를 생성합니다.
    /// 
    /// # Arguments
    /// * `key_prefix` - 키 접두사 (None이면 "view_selection:" 사용)
    pub fn new(key_prefix: Option<String>) -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
            key_prefix: key_prefix.unwrap_or_else(|| "view_selection:".to_string()),
        }
    }

    /// Selection ID에 접두사를 추가하여 키를 생성합니다.
    fn to_key(&self, selection_id: &str) -> String {
        format!("{}{}", self.key_prefix, selection_id)
    }
}

#[async_trait]
impl ViewSelectionRepository for ViewSelectionInMemoryRepositoryImpl {
    async fn save(&self, selection: &ViewSelection) -> Result<(), String> {
        let key = self.to_key(&selection.selection_id);
        let mut store = self.store.write().await;
        store.insert(key, selection.clone());
        Ok(())
    }

    async fn find_by_id(&self, selection_id: &str) -> Result<Option<ViewSelection>, String> {
        let key = self.to_key(selection_id);
        let store = self.store.read().await;
        
        if let Some(selection) = store.get(&key) {
            // TTL 확인
            if selection.is_expired() {
                // 만료된 항목은 None 반환 (실제 삭제는 하지 않음)
                return Ok(None);
            }
            Ok(Some(selection.clone()))
        } else {
            Ok(None)
        }
    }

    async fn extend_ttl(&self, selection_id: &str, ttl_sec: u64) -> Result<(), String> {
        let key = self.to_key(selection_id);
        let mut store = self.store.write().await;
        
        if let Some(selection) = store.get_mut(&key) {
            // 만료 확인
            if selection.is_expired() {
                return Err(format!("Selection {} not found or expired", selection_id));
            }
            
            // TTL 연장
            selection.extend_ttl(ttl_sec);
            Ok(())
        } else {
            Err(format!("Selection {} not found", selection_id))
        }
    }

    async fn delete(&self, selection_id: &str) -> Result<(), String> {
        let key = self.to_key(selection_id);
        let mut store = self.store.write().await;
        store.remove(&key);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::view_selection::SelectedSeries;

    #[tokio::test]
    async fn test_save_and_find() {
        let repo = ViewSelectionInMemoryRepositoryImpl::new(Some("test:".to_string()));
        
        let series = vec![
            SelectedSeries {
                study_uid: "1.2.3".to_string(),
                series_uid: "1.2.3.4".to_string(),
            },
        ];
        
        let selection = ViewSelection::new(
            "sel_test123".to_string(),
            series,
            None,
            None,
            1,
            1800,
        );
        
        // 저장
        repo.save(&selection).await.unwrap();
        
        // 조회
        let found = repo.find_by_id("sel_test123").await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().selection_id, "sel_test123");
    }

    #[tokio::test]
    async fn test_ttl_expiration() {
        let repo = ViewSelectionInMemoryRepositoryImpl::new(Some("test:".to_string()));
        
        let series = vec![
            SelectedSeries {
                study_uid: "1.2.3".to_string(),
                series_uid: "1.2.3.4".to_string(),
            },
        ];
        
        // TTL 0초로 생성 (즉시 만료)
        let selection = ViewSelection::new(
            "sel_expired".to_string(),
            series,
            None,
            None,
            1,
            0,
        );
        
        repo.save(&selection).await.unwrap();
        
        // 1초 대기
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        
        // 만료된 항목은 None 반환
        let found = repo.find_by_id("sel_expired").await.unwrap();
        assert!(found.is_none());
    }
}

