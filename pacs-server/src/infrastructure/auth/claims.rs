use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// JWT Claims 구조체
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// Subject (사용자 ID)
    pub sub: String,

    /// Keycloak UUID
    pub keycloak_id: Uuid,

    /// Username
    pub username: String,

    /// Email
    pub email: String,

    /// Issued At (토큰 발급 시간)
    pub iat: i64,

    /// Expiration (토큰 만료 시간)
    pub exp: i64,
}

impl Claims {
    /// 새로운 Claims 생성
    pub fn new(
        user_id: i32,
        keycloak_id: Uuid,
        username: String,
        email: String,
        expiration_hours: i64,
    ) -> Self {
        let now = Utc::now();
        let expiration = now + Duration::hours(expiration_hours);

        Self {
            sub: user_id.to_string(),
            keycloak_id,
            username,
            email,
            iat: now.timestamp(),
            exp: expiration.timestamp(),
        }
    }

    /// 토큰이 만료되었는지 확인
    pub fn is_expired(&self) -> bool {
        let now = Utc::now().timestamp();
        self.exp < now
    }

    /// 사용자 ID 반환
    pub fn user_id(&self) -> Result<i32, std::num::ParseIntError> {
        self.sub.parse::<i32>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claims_creation() {
        let keycloak_id = Uuid::new_v4();
        let claims = Claims::new(
            1,
            keycloak_id,
            "testuser".to_string(),
            "test@example.com".to_string(),
            24,
        );

        assert_eq!(claims.sub, "1");
        assert_eq!(claims.keycloak_id, keycloak_id);
        assert_eq!(claims.username, "testuser");
        assert_eq!(claims.email, "test@example.com");
        assert!(!claims.is_expired());
    }

    #[test]
    fn test_user_id_parsing() {
        let claims = Claims::new(
            42,
            Uuid::new_v4(),
            "testuser".to_string(),
            "test@example.com".to_string(),
            24,
        );

        let user_id = claims.user_id().unwrap();
        assert_eq!(user_id, 42);
    }

    #[test]
    fn test_expired_token() {
        let mut claims = Claims::new(
            1,
            Uuid::new_v4(),
            "testuser".to_string(),
            "test@example.com".to_string(),
            24,
        );

        // 과거 시간으로 설정
        claims.exp = Utc::now().timestamp() - 3600;
        assert!(claims.is_expired());
    }
}
