use pacs_server::infrastructure::auth::{Claims, JwtService, AuthMiddleware};
use pacs_server::infrastructure::config::JwtConfig;
use uuid::Uuid;

fn get_test_config() -> JwtConfig {
    JwtConfig {
        secret: "test-secret-key-at-least-32-characters-long-for-security".to_string(),
        expiration_hours: 24,
    }
}

// ========================================
// Claims Tests
// ========================================

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
fn test_claims_user_id_parsing() {
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
fn test_claims_is_expired() {
    let mut claims = Claims::new(
        1,
        Uuid::new_v4(),
        "testuser".to_string(),
        "test@example.com".to_string(),
        24,
    );

    // 미래 시간 - 유효함
    assert!(!claims.is_expired());

    // 과거 시간으로 설정 - 만료됨
    claims.exp = chrono::Utc::now().timestamp() - 3600;
    assert!(claims.is_expired());
}

// ========================================
// JwtService Tests
// ========================================

#[test]
fn test_jwt_service_create_token() {
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
    assert!(token.contains('.'));
}

#[test]
fn test_jwt_service_validate_token() {
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
    let validated_claims = jwt_service.validate_token(&token).unwrap();

    assert_eq!(validated_claims.sub, "1");
    assert_eq!(validated_claims.username, "testuser");
    assert_eq!(validated_claims.email, "test@example.com");
}

#[test]
fn test_jwt_service_invalid_token() {
    let config = get_test_config();
    let jwt_service = JwtService::new(&config);

    let result = jwt_service.validate_token("invalid.token.here");
    assert!(result.is_err());
}

#[test]
fn test_jwt_service_tampered_token() {
    let config = get_test_config();
    let jwt_service = JwtService::new(&config);

    let claims = Claims::new(
        1,
        Uuid::new_v4(),
        "testuser".to_string(),
        "test@example.com".to_string(),
        24,
    );

    let mut token = jwt_service.create_token(&claims).unwrap();

    // 토큰 변조
    token.push_str("tampered");

    let result = jwt_service.validate_token(&token);
    assert!(result.is_err());
}

#[test]
fn test_jwt_service_expired_token() {
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
}

#[test]
fn test_jwt_service_extract_bearer_token_valid() {
    let auth_header = "Bearer my-token-string";
    let token = JwtService::extract_bearer_token(auth_header).unwrap();
    assert_eq!(token, "my-token-string");
}

#[test]
fn test_jwt_service_extract_bearer_token_with_spaces() {
    let auth_header = "Bearer   my-token-string  ";
    let token = JwtService::extract_bearer_token(auth_header).unwrap();
    assert_eq!(token, "my-token-string");
}

#[test]
fn test_jwt_service_extract_bearer_token_invalid_scheme() {
    let auth_header = "Basic my-token-string";
    let result = JwtService::extract_bearer_token(auth_header);
    assert!(result.is_err());
}

#[test]
fn test_jwt_service_extract_bearer_token_empty() {
    let auth_header = "Bearer ";
    let result = JwtService::extract_bearer_token(auth_header);
    assert!(result.is_err());
}

#[test]
fn test_jwt_service_extract_bearer_token_no_space() {
    let auth_header = "Bearer";
    let result = JwtService::extract_bearer_token(auth_header);
    assert!(result.is_err());
}

// ========================================
// AuthMiddleware Tests
// ========================================

#[test]
fn test_auth_middleware_authenticate_success() {
    let config = get_test_config();
    let jwt_service = JwtService::new(&config);
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
    assert_eq!(authenticated_claims.email, "test@example.com");
}

#[test]
fn test_auth_middleware_missing_header() {
    let config = get_test_config();
    let jwt_service = JwtService::new(&config);
    let middleware = AuthMiddleware::new(jwt_service);

    let result = middleware.authenticate(None);
    assert!(result.is_err());
}

#[test]
fn test_auth_middleware_invalid_token() {
    let config = get_test_config();
    let jwt_service = JwtService::new(&config);
    let middleware = AuthMiddleware::new(jwt_service);

    let auth_header = "Bearer invalid.token.here";
    let result = middleware.authenticate(Some(auth_header));
    assert!(result.is_err());
}

#[test]
fn test_auth_middleware_not_bearer() {
    let config = get_test_config();
    let jwt_service = JwtService::new(&config);
    let middleware = AuthMiddleware::new(jwt_service);

    let auth_header = "Basic some-credentials";
    let result = middleware.authenticate(Some(auth_header));
    assert!(result.is_err());
}

#[test]
fn test_auth_middleware_expired_token() {
    let config = get_test_config();
    let jwt_service = JwtService::new(&config);
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
}

#[test]
fn test_auth_middleware_verify_token_success() {
    let config = get_test_config();
    let jwt_service = JwtService::new(&config);
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
fn test_auth_middleware_verify_token_failure() {
    let config = get_test_config();
    let jwt_service = JwtService::new(&config);
    let middleware = AuthMiddleware::new(jwt_service);

    let auth_header = "Bearer invalid.token";
    let result = middleware.verify_token(Some(auth_header));
    assert!(result.is_err());
}

// ========================================
// Integration Tests
// ========================================

#[test]
fn test_full_authentication_flow() {
    let config = get_test_config();
    let jwt_service = JwtService::new(&config);
    let middleware = AuthMiddleware::new(jwt_service.clone());

    // 1. 사용자 정보로 Claims 생성
    let user_id = 123;
    let keycloak_id = Uuid::new_v4();
    let claims = Claims::new(
        user_id,
        keycloak_id,
        "john.doe".to_string(),
        "john@example.com".to_string(),
        24,
    );

    // 2. JWT 토큰 생성
    let token = jwt_service.create_token(&claims).unwrap();
    assert!(!token.is_empty());

    // 3. Authorization 헤더 형식으로 만들기
    let auth_header = format!("Bearer {}", token);

    // 4. 미들웨어로 인증
    let authenticated_claims = middleware.authenticate(Some(&auth_header)).unwrap();

    // 5. Claims 검증
    assert_eq!(authenticated_claims.user_id().unwrap(), user_id);
    assert_eq!(authenticated_claims.keycloak_id, keycloak_id);
    assert_eq!(authenticated_claims.username, "john.doe");
    assert_eq!(authenticated_claims.email, "john@example.com");
    assert!(!authenticated_claims.is_expired());
}

#[test]
fn test_different_secret_keys() {
    let config1 = JwtConfig {
        secret: "secret-key-1-at-least-32-characters-long".to_string(),
        expiration_hours: 24,
    };

    let config2 = JwtConfig {
        secret: "secret-key-2-at-least-32-characters-long".to_string(),
        expiration_hours: 24,
    };

    let jwt_service1 = JwtService::new(&config1);
    let jwt_service2 = JwtService::new(&config2);

    let claims = Claims::new(
        1,
        Uuid::new_v4(),
        "testuser".to_string(),
        "test@example.com".to_string(),
        24,
    );

    // service1으로 토큰 생성
    let token = jwt_service1.create_token(&claims).unwrap();

    // service2로 검증 시도 - 실패해야 함
    let result = jwt_service2.validate_token(&token);
    assert!(result.is_err());

    // service1으로 검증 - 성공해야 함
    let result = jwt_service1.validate_token(&token);
    assert!(result.is_ok());
}

#[test]
fn test_multiple_users_authentication() {
    let config = get_test_config();
    let jwt_service = JwtService::new(&config);
    let middleware = AuthMiddleware::new(jwt_service.clone());

    // 사용자 1
    let claims1 = Claims::new(
        1,
        Uuid::new_v4(),
        "user1".to_string(),
        "user1@example.com".to_string(),
        24,
    );
    let token1 = jwt_service.create_token(&claims1).unwrap();

    // 사용자 2
    let claims2 = Claims::new(
        2,
        Uuid::new_v4(),
        "user2".to_string(),
        "user2@example.com".to_string(),
        24,
    );
    let token2 = jwt_service.create_token(&claims2).unwrap();

    // 각각 인증
    let auth1 = middleware.authenticate(Some(&format!("Bearer {}", token1))).unwrap();
    let auth2 = middleware.authenticate(Some(&format!("Bearer {}", token2))).unwrap();

    assert_eq!(auth1.username, "user1");
    assert_eq!(auth2.username, "user2");
    assert_ne!(auth1.keycloak_id, auth2.keycloak_id);
}
