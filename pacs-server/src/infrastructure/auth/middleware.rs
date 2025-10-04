use super::{Claims, JwtService};

/// 인증 미들웨어
/// HTTP 요청에서 JWT 토큰을 검증하고 Claims를 추출합니다
pub struct AuthMiddleware {
    jwt_service: JwtService,
}

impl AuthMiddleware {
    pub fn new(jwt_service: JwtService) -> Self {
        Self { jwt_service }
    }

    /// Authorization 헤더에서 토큰을 추출하고 검증
    pub fn authenticate(&self, authorization_header: Option<&str>) -> Result<Claims, AuthError> {
        let auth_header = authorization_header.ok_or(AuthError::MissingToken)?;

        let token = JwtService::extract_bearer_token(auth_header)
            .map_err(|e| AuthError::InvalidToken(e.to_string()))?;

        let claims = self
            .jwt_service
            .validate_token(&token)
            .map_err(|e| match e {
                super::jwt_service::JwtError::ExpiredToken => AuthError::ExpiredToken,
                _ => AuthError::InvalidToken(e.to_string()),
            })?;

        Ok(claims)
    }

    /// 토큰이 유효한지만 확인 (Claims 불필요)
    pub fn verify_token(&self, authorization_header: Option<&str>) -> Result<(), AuthError> {
        self.authenticate(authorization_header)?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum AuthError {
    MissingToken,
    InvalidToken(String),
    ExpiredToken,
    Unauthorized(String),
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::MissingToken => write!(f, "Missing authorization token"),
            AuthError::InvalidToken(msg) => write!(f, "Invalid token: {}", msg),
            AuthError::ExpiredToken => write!(f, "Token has expired"),
            AuthError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
        }
    }
}

impl std::error::Error for AuthError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::config::JwtConfig;
    use uuid::Uuid;

    fn get_test_jwt_service() -> JwtService {
        let config = JwtConfig {
            secret: "test-secret-key-at-least-32-characters-long".to_string(),
            expiration_hours: 24,
        };
        JwtService::new(&config)
    }

    #[test]
    fn test_authenticate_success() {
        let jwt_service = get_test_jwt_service();
        let middleware = AuthMiddleware::new(jwt_service.clone());

        let claims = Claims::new(
            1,
            Uuid::new_v4(),
            "testuser".to_string(),
            "test@example.com".to_string(),
            24,
        );

        let token = jwt_service.create_token(&claims).unwrap();
        let auth_header = format!("Bearer {}", token);

        let result = middleware.authenticate(Some(&auth_header));
        assert!(result.is_ok());

        let authenticated_claims = result.unwrap();
        assert_eq!(authenticated_claims.username, "testuser");
    }

    #[test]
    fn test_authenticate_missing_token() {
        let jwt_service = get_test_jwt_service();
        let middleware = AuthMiddleware::new(jwt_service);

        let result = middleware.authenticate(None);
        assert!(result.is_err());

        if let Err(AuthError::MissingToken) = result {
            // Expected
        } else {
            panic!("Expected MissingToken error");
        }
    }

    #[test]
    fn test_authenticate_invalid_token() {
        let jwt_service = get_test_jwt_service();
        let middleware = AuthMiddleware::new(jwt_service);

        let auth_header = "Bearer invalid.token.here";
        let result = middleware.authenticate(Some(auth_header));
        assert!(result.is_err());
    }

    #[test]
    fn test_authenticate_not_bearer() {
        let jwt_service = get_test_jwt_service();
        let middleware = AuthMiddleware::new(jwt_service);

        let auth_header = "Basic some-credentials";
        let result = middleware.authenticate(Some(auth_header));
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_token_success() {
        let jwt_service = get_test_jwt_service();
        let middleware = AuthMiddleware::new(jwt_service.clone());

        let claims = Claims::new(
            1,
            Uuid::new_v4(),
            "testuser".to_string(),
            "test@example.com".to_string(),
            24,
        );

        let token = jwt_service.create_token(&claims).unwrap();
        let auth_header = format!("Bearer {}", token);

        let result = middleware.verify_token(Some(&auth_header));
        assert!(result.is_ok());
    }

    #[test]
    fn test_expired_token() {
        let jwt_service = get_test_jwt_service();
        let middleware = AuthMiddleware::new(jwt_service.clone());

        let mut claims = Claims::new(
            1,
            Uuid::new_v4(),
            "testuser".to_string(),
            "test@example.com".to_string(),
            24,
        );

        // 과거 시간으로 설정
        claims.exp = chrono::Utc::now().timestamp() - 3600;

        let token = jwt_service.create_token(&claims).unwrap();
        let auth_header = format!("Bearer {}", token);

        let result = middleware.authenticate(Some(&auth_header));
        assert!(result.is_err());

        if let Err(AuthError::ExpiredToken) = result {
            // Expected
        } else {
            panic!("Expected ExpiredToken error");
        }
    }
}
