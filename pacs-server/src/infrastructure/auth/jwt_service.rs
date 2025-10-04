use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation, Algorithm};
use super::claims::Claims;
use crate::infrastructure::config::JwtConfig;

#[derive(Debug)]
pub enum JwtError {
    TokenCreation(String),
    TokenValidation(String),
    InvalidToken(String),
    ExpiredToken,
}

impl std::fmt::Display for JwtError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JwtError::TokenCreation(msg) => write!(f, "Token creation error: {}", msg),
            JwtError::TokenValidation(msg) => write!(f, "Token validation error: {}", msg),
            JwtError::InvalidToken(msg) => write!(f, "Invalid token: {}", msg),
            JwtError::ExpiredToken => write!(f, "Token has expired"),
        }
    }
}

impl std::error::Error for JwtError {}

/// JWT 토큰 생성 및 검증 서비스
#[derive(Clone)]
pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    validation: Validation,
}

impl JwtService {
    /// 새로운 JwtService 생성
    pub fn new(config: &JwtConfig) -> Self {
        let encoding_key = EncodingKey::from_secret(config.secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(config.secret.as_bytes());

        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;
        validation.leeway = 60; // 60초 여유

        Self {
            encoding_key,
            decoding_key,
            validation,
        }
    }

    /// JWT 토큰 생성
    pub fn create_token(&self, claims: &Claims) -> Result<String, JwtError> {
        encode(&Header::default(), claims, &self.encoding_key)
            .map_err(|e| JwtError::TokenCreation(e.to_string()))
    }

    /// JWT 토큰 검증 및 Claims 추출
    pub fn validate_token(&self, token: &str) -> Result<Claims, JwtError> {
        let token_data = decode::<Claims>(token, &self.decoding_key, &self.validation)
            .map_err(|e| {
                if e.to_string().contains("ExpiredSignature") {
                    JwtError::ExpiredToken
                } else {
                    JwtError::TokenValidation(e.to_string())
                }
            })?;

        let claims = token_data.claims;

        // 추가 만료 확인
        if claims.is_expired() {
            return Err(JwtError::ExpiredToken);
        }

        Ok(claims)
    }

    /// Bearer 토큰에서 토큰 문자열 추출
    pub fn extract_bearer_token(auth_header: &str) -> Result<String, JwtError> {
        if !auth_header.starts_with("Bearer ") {
            return Err(JwtError::InvalidToken(
                "Authorization header must start with 'Bearer '".to_string(),
            ));
        }

        let token = auth_header.trim_start_matches("Bearer ").trim();

        if token.is_empty() {
            return Err(JwtError::InvalidToken("Token is empty".to_string()));
        }

        Ok(token.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn get_test_config() -> JwtConfig {
        JwtConfig {
            secret: "test-secret-key-at-least-32-characters-long".to_string(),
            expiration_hours: 24,
        }
    }

    #[test]
    fn test_create_and_validate_token() {
        let config = get_test_config();
        let jwt_service = JwtService::new(&config);

        let claims = Claims::new(
            1,
            Uuid::new_v4(),
            "testuser".to_string(),
            "test@example.com".to_string(),
            24,
        );

        let token = jwt_service.create_token(&claims).unwrap();
        assert!(!token.is_empty());

        let validated_claims = jwt_service.validate_token(&token).unwrap();
        assert_eq!(validated_claims.sub, "1");
        assert_eq!(validated_claims.username, "testuser");
    }

    #[test]
    fn test_invalid_token() {
        let config = get_test_config();
        let jwt_service = JwtService::new(&config);

        let result = jwt_service.validate_token("invalid.token.here");
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_bearer_token() {
        let auth_header = "Bearer my-token-string";
        let token = JwtService::extract_bearer_token(auth_header).unwrap();
        assert_eq!(token, "my-token-string");
    }

    #[test]
    fn test_extract_bearer_token_invalid() {
        let auth_header = "Basic my-token-string";
        let result = JwtService::extract_bearer_token(auth_header);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_bearer_token_empty() {
        let auth_header = "Bearer ";
        let result = JwtService::extract_bearer_token(auth_header);
        assert!(result.is_err());
    }

    #[test]
    fn test_expired_token_detection() {
        let config = get_test_config();
        let jwt_service = JwtService::new(&config);

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
        let result = jwt_service.validate_token(&token);

        assert!(result.is_err());
        if let Err(JwtError::ExpiredToken) = result {
            // Expected
        } else {
            panic!("Expected ExpiredToken error");
        }
    }
}
