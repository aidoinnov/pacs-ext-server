use redis::Client as RedisClient;
use std::sync::Arc;

/// Redis 클라이언트 래퍼
/// 
/// Redis 연결을 관리하고, 연결 풀을 제공합니다.
#[derive(Clone)]
pub struct RedisConnection {
    client: Arc<RedisClient>,
}

impl RedisConnection {
    /// 새로운 Redis 연결을 생성합니다.
    /// 
    /// # Arguments
    /// * `url` - Redis 연결 URL (예: "redis://localhost:6379")
    /// 
    /// # Returns
    /// * `Result<Self, redis::RedisError>` - 연결 성공 시 RedisConnection, 실패 시 에러
    pub async fn new(url: &str) -> Result<Self, redis::RedisError> {
        let client = RedisClient::open(url)?;
        
        // 연결 테스트
        let mut conn = client.get_multiplexed_async_connection().await?;
        redis::cmd("PING").query_async(&mut conn).await?;
        
        Ok(Self {
            client: Arc::new(client),
        })
    }

    /// 비동기 Redis 연결을 가져옵니다.
    pub async fn get_connection(&self) -> Result<redis::aio::MultiplexedConnection, redis::RedisError> {
        self.client.get_multiplexed_async_connection().await
    }

    /// Redis 클라이언트를 반환합니다.
    pub fn client(&self) -> Arc<RedisClient> {
        self.client.clone()
    }

    /// Redis 연결을 테스트합니다 (PING).
    pub async fn ping(&self) -> Result<String, redis::RedisError> {
        let mut conn = self.get_connection().await?;
        redis::cmd("PING").query_async(&mut conn).await
    }
}

/// Redis 클라이언트 팩토리
pub struct RedisClientFactory;

impl RedisClientFactory {
    /// Redis 연결을 생성합니다.
    /// 
    /// # Arguments
    /// * `url` - Redis 연결 URL
    /// 
    /// # Returns
    /// * `Result<RedisConnection, redis::RedisError>` - 연결 성공 시 RedisConnection, 실패 시 에러
    pub async fn create(url: &str) -> Result<RedisConnection, redis::RedisError> {
        RedisConnection::new(url).await
    }
}


