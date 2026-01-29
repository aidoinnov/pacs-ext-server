use crate::infrastructure::redis::RedisConnection;
use redis::AsyncCommands;
use std::env;

/// Project 멤버십 캐시 서비스
/// 
/// 사용자-프로젝트 멤버십 정보를 Redis에 캐싱하여 DB 조회 부하를 절감합니다.
/// 
/// ## 캐시 전략
/// - **캐시 키**: `membership:u{user_id}:p{project_id}`
/// - **캐시 값**: `{role_id}` (멤버가 아니면 "NOT_MEMBER")
/// - **TTL**: 180초 (3분, 환경변수로 설정 가능)
/// - **무효화**: 멤버십 변경 시 수동 무효화
#[derive(Clone)]
pub struct MembershipCacheService {
    redis: RedisConnection,
    ttl_seconds: u64,
}

impl MembershipCacheService {
    /// 새로운 MembershipCacheService 생성
    /// 
    /// # Arguments
    /// * `redis` - Redis 연결
    pub fn new(redis: RedisConnection) -> Self {
        let ttl_seconds = env::var("MEMBERSHIP_CACHE_TTL_SEC")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(180); // 기본값: 3분

        Self {
            redis,
            ttl_seconds,
        }
    }

    /// 멤버십 정보 캐시 조회
    /// 
    /// # Arguments
    /// * `user_id` - 사용자 ID
    /// * `project_id` - 프로젝트 ID
    /// 
    /// # Returns
    /// * `Ok(Some(Some(role_id)))` - 캐시 HIT, 멤버임 (role_id 반환)
    /// * `Ok(Some(None))` - 캐시 HIT, 멤버 아님
    /// * `Ok(None)` - 캐시 MISS
    /// * `Err(String)` - Redis 에러
    pub async fn get_membership(
        &self,
        user_id: i32,
        project_id: i32,
    ) -> Result<Option<Option<i32>>, String> {
        let key = Self::cache_key(user_id, project_id);
        
        let mut conn = self.redis
            .get_connection()
            .await
            .map_err(|e| format!("Failed to get Redis connection: {}", e))?;

        let value: Option<String> = conn
            .get(&key)
            .await
            .map_err(|e| format!("Failed to get from Redis: {}", e))?;

        match value {
            Some(v) if v == "NOT_MEMBER" => {
                tracing::debug!("⚡ Membership cache HIT - user={}, project={}, member=false", user_id, project_id);
                Ok(Some(None)) // 캐시 HIT, 멤버 아님
            }
            Some(v) => {
                match v.parse::<i32>() {
                    Ok(role_id) => {
                        tracing::debug!("⚡ Membership cache HIT - user={}, project={}, role={}", user_id, project_id, role_id);
                        Ok(Some(Some(role_id))) // 캐시 HIT, 멤버임
                    }
                    Err(_) => {
                        tracing::warn!("Invalid cached membership value: {}", v);
                        Ok(None) // 캐시 MISS (잘못된 데이터)
                    }
                }
            }
            None => {
                tracing::debug!("🔄 Membership cache MISS - user={}, project={}", user_id, project_id);
                Ok(None) // 캐시 MISS
            }
        }
    }

    /// 멤버십 정보 캐시 저장
    /// 
    /// # Arguments
    /// * `user_id` - 사용자 ID
    /// * `project_id` - 프로젝트 ID
    /// * `role_id` - 역할 ID (None이면 멤버가 아님)
    pub async fn set_membership(
        &self,
        user_id: i32,
        project_id: i32,
        role_id: Option<i32>,
    ) -> Result<(), String> {
        let key = Self::cache_key(user_id, project_id);
        let value = match role_id {
            Some(rid) => rid.to_string(),
            None => "NOT_MEMBER".to_string(),
        };

        let mut conn = self.redis
            .get_connection()
            .await
            .map_err(|e| format!("Failed to get Redis connection: {}", e))?;

        conn.set_ex::<&str, String, ()>(&key, value, self.ttl_seconds)
            .await
            .map_err(|e| format!("Failed to set to Redis: {}", e))?;

        tracing::debug!("Cache SET: {} (TTL: {}s, role_id: {:?})", key, self.ttl_seconds, role_id);
        Ok(())
    }

    /// 멤버십 캐시 무효화
    /// 
    /// # Arguments
    /// * `user_id` - 사용자 ID
    /// * `project_id` - 프로젝트 ID
    pub async fn invalidate_membership(
        &self,
        user_id: i32,
        project_id: i32,
    ) -> Result<(), String> {
        let key = Self::cache_key(user_id, project_id);

        let mut conn = self.redis
            .get_connection()
            .await
            .map_err(|e| format!("Failed to get Redis connection: {}", e))?;

        conn.del::<&str, ()>(&key)
            .await
            .map_err(|e| format!("Failed to delete from Redis: {}", e))?;

        tracing::debug!("Cache INVALIDATE: {}", key);
        Ok(())
    }

    /// 캐시 키 생성
    fn cache_key(user_id: i32, project_id: i32) -> String {
        format!("membership:u{}:p{}", user_id, project_id)
    }
}

