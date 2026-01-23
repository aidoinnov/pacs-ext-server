use crate::infrastructure::redis::RedisConnection;
use redis::AsyncCommands;
use serde_json::Value as JsonValue;
use std::sync::Arc;

/// QIDO-RS 응답 캐싱 서비스
/// 
/// Dcm4chee QIDO-RS API 응답을 Redis에 캐싱하여 성능을 개선합니다.
#[derive(Clone)]
pub struct QidoCacheService {
    redis: Arc<RedisConnection>,
    ttl_seconds: u64,
}

impl QidoCacheService {
    /// 새로운 QIDO 캐시 서비스 생성
    /// 
    /// # Arguments
    /// * `redis` - Redis 연결
    /// * `ttl_seconds` - 캐시 TTL (초 단위, 기본값: 60초)
    pub fn new(redis: Arc<RedisConnection>, ttl_seconds: Option<u64>) -> Self {
        Self {
            redis,
            ttl_seconds: ttl_seconds.unwrap_or(60),
        }
    }

    /// 캐시 키 생성
    /// 
    /// # Arguments
    /// * `endpoint` - QIDO 엔드포인트 타입 (studies, series, instances)
    /// * `study_uid` - Study Instance UID
    /// * `series_uid` - Series Instance UID (선택)
    /// * `project_id` - 프로젝트 ID (선택)
    /// * `params_hash` - 쿼리 파라미터 해시 (선택)
    fn cache_key(
        endpoint: &str,
        study_uid: &str,
        series_uid: Option<&str>,
        project_id: Option<i32>,
        params_hash: Option<&str>,
    ) -> String {
        let mut key = format!("qido:{}:{}", endpoint, study_uid);
        
        if let Some(series) = series_uid {
            key.push_str(&format!(":{}", series));
        }
        
        if let Some(pid) = project_id {
            key.push_str(&format!(":p{}", pid));
        }
        
        if let Some(hash) = params_hash {
            key.push_str(&format!(":h{}", hash));
        }
        
        key
    }

    /// Series 목록 캐시 조회
    /// 
    /// # Arguments
    /// * `study_uid` - Study Instance UID
    /// * `project_id` - 프로젝트 ID
    /// * `params_hash` - 쿼리 파라미터 해시 (선택)
    pub async fn get_series(
        &self,
        study_uid: &str,
        project_id: Option<i32>,
        params_hash: Option<&str>,
    ) -> Result<Option<JsonValue>, String> {
        let key = Self::cache_key("series", study_uid, None, project_id, params_hash);
        self.get_cached_json(&key).await
    }

    /// Series 목록 캐시 저장
    pub async fn set_series(
        &self,
        study_uid: &str,
        project_id: Option<i32>,
        params_hash: Option<&str>,
        data: &JsonValue,
    ) -> Result<(), String> {
        let key = Self::cache_key("series", study_uid, None, project_id, params_hash);
        self.set_cached_json(&key, data).await
    }

    /// Instance 목록 캐시 조회
    pub async fn get_instances(
        &self,
        study_uid: &str,
        series_uid: &str,
        project_id: Option<i32>,
        params_hash: Option<&str>,
    ) -> Result<Option<JsonValue>, String> {
        let key = Self::cache_key("instances", study_uid, Some(series_uid), project_id, params_hash);
        self.get_cached_json(&key).await
    }

    /// Instance 목록 캐시 저장
    pub async fn set_instances(
        &self,
        study_uid: &str,
        series_uid: &str,
        project_id: Option<i32>,
        params_hash: Option<&str>,
        data: &JsonValue,
    ) -> Result<(), String> {
        let key = Self::cache_key("instances", study_uid, Some(series_uid), project_id, params_hash);
        self.set_cached_json(&key, data).await
    }

    /// Study 목록 캐시 조회
    pub async fn get_studies(
        &self,
        project_id: Option<i32>,
        params_hash: Option<&str>,
    ) -> Result<Option<JsonValue>, String> {
        let key = Self::cache_key("studies", "*", None, project_id, params_hash);
        self.get_cached_json(&key).await
    }

    /// Study 목록 캐시 저장
    pub async fn set_studies(
        &self,
        project_id: Option<i32>,
        params_hash: Option<&str>,
        data: &JsonValue,
    ) -> Result<(), String> {
        let key = Self::cache_key("studies", "*", None, project_id, params_hash);
        self.set_cached_json(&key, data).await
    }

    /// 캐시 무효화 (특정 Study)
    pub async fn invalidate_study(&self, study_uid: &str) -> Result<(), String> {
        let pattern = format!("qido:*:{}:*", study_uid);
        self.delete_pattern(&pattern).await
    }

    /// 캐시 무효화 (특정 프로젝트)
    pub async fn invalidate_project(&self, project_id: i32) -> Result<(), String> {
        let pattern = format!("qido:*:p{}:*", project_id);
        self.delete_pattern(&pattern).await
    }

    // ========== Private Helper Methods ==========

    /// Redis에서 JSON 데이터 조회
    async fn get_cached_json(&self, key: &str) -> Result<Option<JsonValue>, String> {
        let mut conn = self.redis
            .get_connection()
            .await
            .map_err(|e| format!("Failed to get Redis connection: {}", e))?;

        let cached: Option<String> = conn
            .get(key)
            .await
            .map_err(|e| format!("Failed to get from Redis: {}", e))?;

        match cached {
            Some(json_str) => {
                let value = serde_json::from_str(&json_str)
                    .map_err(|e| format!("Failed to deserialize JSON: {}", e))?;
                tracing::debug!("Cache HIT: {}", key);
                Ok(Some(value))
            }
            None => {
                tracing::debug!("Cache MISS: {}", key);
                Ok(None)
            }
        }
    }

    /// Redis에 JSON 데이터 저장 (TTL 포함)
    async fn set_cached_json(&self, key: &str, data: &JsonValue) -> Result<(), String> {
        let mut conn = self.redis
            .get_connection()
            .await
            .map_err(|e| format!("Failed to get Redis connection: {}", e))?;

        let json_str = serde_json::to_string(data)
            .map_err(|e| format!("Failed to serialize JSON: {}", e))?;

        conn.set_ex::<&str, String, ()>(key, json_str, self.ttl_seconds)
            .await
            .map_err(|e| format!("Failed to set to Redis: {}", e))?;

        tracing::debug!("Cache SET: {} (TTL: {}s)", key, self.ttl_seconds);
        Ok(())
    }

    /// 패턴 매칭으로 캐시 삭제
    async fn delete_pattern(&self, pattern: &str) -> Result<(), String> {
        let mut conn = self.redis
            .get_connection()
            .await
            .map_err(|e| format!("Failed to get Redis connection: {}", e))?;

        // SCAN으로 패턴 매칭 키 찾기
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(pattern)
            .query_async(&mut conn)
            .await
            .map_err(|e| format!("Failed to scan keys: {}", e))?;

        if !keys.is_empty() {
            conn.del::<Vec<String>, ()>(keys.clone())
                .await
                .map_err(|e| format!("Failed to delete keys: {}", e))?;

            tracing::info!("Cache INVALIDATE: {} ({} keys deleted)", pattern, keys.len());
        }

        Ok(())
    }

    /// 쿼리 파라미터 해시 생성 (Vec<(String, String)> 타입)
    pub fn hash_params(params: &Vec<(String, String)>) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();

        // 정렬된 키-값 쌍으로 해시 생성 (일관성 보장)
        let mut sorted_params = params.clone();
        sorted_params.sort_by(|(k1, _), (k2, _)| k1.cmp(k2));

        for (key, value) in sorted_params {
            key.hash(&mut hasher);
            value.hash(&mut hasher);
        }

        format!("{:x}", hasher.finish())
    }
}

