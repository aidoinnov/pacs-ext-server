use async_trait::async_trait;
use crate::domain::view_selection::ViewSelection;
use crate::domain::view_selection::repositories::ViewSelectionRepository;
use crate::infrastructure::redis::RedisConnection;
use redis::AsyncCommands;
use std::sync::Arc;

/// Redis 기반 ViewSelectionRepository 구현체
#[derive(Clone)]
pub struct ViewSelectionRepositoryImpl {
    redis: Arc<RedisConnection>,
    key_prefix: String,
}

impl ViewSelectionRepositoryImpl {
    /// 새로운 ViewSelectionRepositoryImpl을 생성합니다.
    ///
    /// # Arguments
    /// * `redis` - Redis 연결
    /// * `key_prefix` - Redis 키 접두사 (기본: "view_selection:")
    pub fn new(redis: Arc<RedisConnection>, key_prefix: Option<String>) -> Self {
        Self {
            redis,
            key_prefix: key_prefix.unwrap_or_else(|| "view_selection:".to_string()),
        }
    }

    /// Selection ID를 Redis 키로 변환합니다.
    fn to_redis_key(&self, selection_id: &str) -> String {
        format!("{}{}", self.key_prefix, selection_id)
    }
}

#[async_trait]
impl ViewSelectionRepository for ViewSelectionRepositoryImpl {
    async fn save(&self, selection: &ViewSelection) -> Result<(), String> {
        let key = self.to_redis_key(&selection.selection_id);
        let mut conn = self.redis.get_connection().await
            .map_err(|e| format!("Failed to get Redis connection: {}", e))?;

        // ViewSelection을 JSON으로 직렬화
        let json = serde_json::to_string(selection)
            .map_err(|e| format!("Failed to serialize ViewSelection: {}", e))?;

        // TTL 계산 (초 단위)
        let ttl_sec = (selection.expires_at - selection.created_at)
            .num_seconds()
            .max(1) as u64;

        // Redis에 저장 (SET with TTL)
        conn.set_ex::<String, String, ()>(key, json, ttl_sec)
            .await
            .map_err(|e| format!("Failed to save to Redis: {}", e))?;

        Ok(())
    }

    async fn find_by_id(&self, selection_id: &str) -> Result<Option<ViewSelection>, String> {
        let key = self.to_redis_key(selection_id);
        let mut conn = self.redis.get_connection().await
            .map_err(|e| format!("Failed to get Redis connection: {}", e))?;

        // Redis에서 조회
        let json: Option<String> = conn
            .get(key)
            .await
            .map_err(|e| format!("Failed to get from Redis: {}", e))?;

        match json {
            Some(json_str) => {
                // JSON을 ViewSelection으로 역직렬화
                let selection: ViewSelection = serde_json::from_str(&json_str)
                    .map_err(|e| format!("Failed to deserialize ViewSelection: {}", e))?;

                // 만료 확인
                if selection.is_expired() {
                    // 만료된 경우 삭제
                    let _ = self.delete(selection_id).await;
                    Ok(None)
                } else {
                    Ok(Some(selection))
                }
            }
            None => Ok(None),
        }
    }

    async fn extend_ttl(&self, selection_id: &str, ttl_sec: u64) -> Result<(), String> {
        let key = self.to_redis_key(selection_id);
        let mut conn = self.redis.get_connection().await
            .map_err(|e| format!("Failed to get Redis connection: {}", e))?;

        // 기존 Selection 조회
        let json: Option<String> = conn
            .get::<String, Option<String>>(key.clone())
            .await
            .map_err(|e| format!("Failed to get from Redis: {}", e))?;

        match json {
            Some(json_str) => {
                // JSON을 ViewSelection으로 역직렬화
                let mut selection: ViewSelection = serde_json::from_str(&json_str)
                    .map_err(|e| format!("Failed to deserialize ViewSelection: {}", e))?;

                // 만료 확인
                if selection.is_expired() {
                    return Err("Selection has expired".to_string());
                }

                // TTL 연장
                selection.extend_ttl(ttl_sec);

                // 업데이트된 Selection 저장
                let updated_json = serde_json::to_string(&selection)
                    .map_err(|e| format!("Failed to serialize ViewSelection: {}", e))?;

                conn.set_ex::<String, String, ()>(key, updated_json, ttl_sec)
                    .await
                    .map_err(|e| format!("Failed to update TTL in Redis: {}", e))?;

                Ok(())
            }
            None => Err("Selection not found".to_string()),
        }
    }

    async fn delete(&self, selection_id: &str) -> Result<(), String> {
        let key = self.to_redis_key(selection_id);
        let mut conn = self.redis.get_connection().await
            .map_err(|e| format!("Failed to get Redis connection: {}", e))?;

        conn.del::<String, ()>(key)
            .await
            .map_err(|e| format!("Failed to delete from Redis: {}", e))?;

        Ok(())
    }
}

