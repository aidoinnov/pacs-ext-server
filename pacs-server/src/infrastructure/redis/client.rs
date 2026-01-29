use redis::Client as RedisClient;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Redis 클라이언트 래퍼
///
/// Redis 연결을 관리하고, 연결 풀을 제공합니다.
/// MultiplexedConnection을 재사용하여 연결 누수를 방지합니다.
#[derive(Clone)]
pub struct RedisConnection {
    client: Arc<RedisClient>,
    /// 재사용 가능한 Multiplexed 연결
    /// Mutex로 보호하여 동시 접근 제어
    connection: Arc<Mutex<Option<redis::aio::MultiplexedConnection>>>,
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

        // 초기 연결 생성 및 테스트
        let mut conn = client.get_multiplexed_async_connection().await?;
        redis::cmd("PING").query_async(&mut conn).await?;

        Ok(Self {
            client: Arc::new(client),
            connection: Arc::new(Mutex::new(Some(conn))),
        })
    }

    /// 비동기 Redis 연결을 가져옵니다.
    ///
    /// 기존 연결을 재사용하거나, 연결이 끊어진 경우 새로 생성합니다.
    pub async fn get_connection(&self) -> Result<redis::aio::MultiplexedConnection, redis::RedisError> {
        let mut conn_guard = self.connection.lock().await;

        // 기존 연결이 있으면 재사용
        if let Some(conn) = conn_guard.take() {
            // 연결 유효성 검사 (PING)
            let mut test_conn = conn.clone();
            match redis::cmd("PING").query_async::<String>(&mut test_conn).await {
                Ok(_) => {
                    // 연결이 유효하면 다시 저장하고 복제본 반환
                    *conn_guard = Some(conn.clone());
                    return Ok(conn);
                }
                Err(_) => {
                    // 연결이 끊어진 경우 새로 생성
                    tracing::warn!("Redis connection lost, reconnecting...");
                }
            }
        }

        // 새 연결 생성
        let new_conn = self.client.get_multiplexed_async_connection().await?;
        *conn_guard = Some(new_conn.clone());
        Ok(new_conn)
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


