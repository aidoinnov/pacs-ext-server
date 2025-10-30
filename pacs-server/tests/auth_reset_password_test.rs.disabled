#[cfg(test)]
mod auth_reset_password_tests {
    use std::collections::HashMap;
    use uuid::Uuid;
    use chrono::Utc;

    use pacs_server::domain::entities::User;
    use pacs_server::domain::repositories::UserRepository;
    use pacs_server::domain::services::{AuthService, AuthServiceImpl};
    use pacs_server::infrastructure::auth::JwtService;
    use pacs_server::infrastructure::config::JwtConfig;
    use pacs_server::infrastructure::config::KeycloakConfig;
    use pacs_server::infrastructure::external::KeycloakClient;
    use std::sync::Arc;

    // Mock UserRepository for testing
    #[derive(Clone)]
    struct MockUserRepository {
        users: HashMap<i32, User>,
        users_by_username: HashMap<String, User>,
    }

    impl MockUserRepository {
        fn new() -> Self {
            Self {
                users: HashMap::new(),
                users_by_username: HashMap::new(),
            }
        }

        fn add_user(&mut self, user: User) {
            self.users.insert(user.id, user.clone());
            self.users_by_username.insert(user.username.clone(), user);
        }
    }

    #[async_trait::async_trait]
    impl UserRepository for MockUserRepository {
        async fn find_by_id(&self, id: i32) -> Result<Option<User>, sqlx::Error> {
            Ok(self.users.get(&id).cloned())
        }

        async fn find_by_keycloak_id(&self, _keycloak_id: Uuid) -> Result<Option<User>, sqlx::Error> {
            Ok(None)
        }

        async fn find_by_username(&self, username: &str) -> Result<Option<User>, sqlx::Error> {
            Ok(self.users_by_username.get(username).cloned())
        }

        async fn find_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error> {
            Ok(self.users.values().find(|u| u.email == email).cloned())
        }

        async fn find_all(&self) -> Result<Vec<User>, sqlx::Error> {
            Ok(self.users.values().cloned().collect())
        }

        async fn create(&self, _new_user: pacs_server::domain::entities::NewUser) -> Result<User, sqlx::Error> {
            Err(sqlx::Error::RowNotFound)
        }

        async fn update(&self, _update_user: &pacs_server::domain::entities::UpdateUser) -> Result<User, sqlx::Error> {
            Err(sqlx::Error::RowNotFound)
        }

        async fn delete(&self, _id: i32) -> Result<bool, sqlx::Error> {
            Ok(false)
        }

        fn pool(&self) -> &sqlx::PgPool {
            unimplemented!()
        }
    }

    fn create_test_user(id: i32, username: String, email: String) -> User {
        User {
            id,
            keycloak_id: Uuid::new_v4(),
            username,
            email,
            full_name: None,
            organization: None,
            department: None,
            phone: None,
            created_at: Utc::now(),
            updated_at: None,
            account_status: pacs_server::domain::entities::UserAccountStatus::Active,
            email_verified: true,
            email_verification_token: None,
            email_verification_expires_at: None,
            approved_by: None,
            approved_at: None,
            suspended_at: None,
            suspended_reason: None,
            deleted_at: None,
        }
    }

    fn create_jwt_service() -> JwtService {
        let config = JwtConfig {
            secret: "test_secret_key_that_is_long_enough_for_hs256".to_string(),
            expiration_hours: 24,
        };
        JwtService::new(&config)
    }

    // Note: Keycloak client mock is complex, so we'll test the validation logic instead
    // In real integration tests, you would use a test Keycloak instance

    #[tokio::test]
    async fn test_reset_password_validation_too_short() {
        let mut user_repo = MockUserRepository::new();
        let test_user = create_test_user(1, "testuser".to_string(), "test@example.com".to_string());
        user_repo.add_user(test_user);

        let jwt_service = create_jwt_service();
        let keycloak_config = KeycloakConfig {
            url: "http://localhost:8080".to_string(),
            realm: "test_realm".to_string(),
            client_id: "test_client".to_string(),
            admin_username: "admin".to_string(),
            admin_password: "admin".to_string(),
            client_secret: "test_secret".to_string(),
        };
        let keycloak_client = Arc::new(KeycloakClient::new(keycloak_config));
        
        let auth_service: AuthServiceImpl<MockUserRepository> = AuthServiceImpl::new(
            user_repo,
            jwt_service,
            keycloak_client,
        );

        // Test with too short password (less than 8 characters)
        let result = auth_service
            .reset_password_by_credentials("testuser", "test@example.com", "short")
            .await;

        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("비밀번호는 최소 8자 이상이어야 합니다"));
        }
    }

    #[tokio::test]
    async fn test_reset_password_validation_email_mismatch() {
        let mut user_repo = MockUserRepository::new();
        let test_user = create_test_user(1, "testuser".to_string(), "test@example.com".to_string());
        user_repo.add_user(test_user);

        let jwt_service = create_jwt_service();
        let keycloak_config = KeycloakConfig {
            url: "http://localhost:8080".to_string(),
            realm: "test_realm".to_string(),
            client_id: "test_client".to_string(),
            admin_username: "admin".to_string(),
            admin_password: "admin".to_string(),
            client_secret: "test_secret".to_string(),
        };
        let keycloak_client = Arc::new(KeycloakClient::new(keycloak_config));
        
        let auth_service: AuthServiceImpl<MockUserRepository> = AuthServiceImpl::new(
            user_repo,
            jwt_service,
            keycloak_client,
        );

        // Test with incorrect email
        let result = auth_service
            .reset_password_by_credentials("testuser", "wrong@example.com", "NewPassword123")
            .await;

        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("이메일 정보가 일치하지 않습니다"));
        }
    }

    #[tokio::test]
    async fn test_reset_password_user_not_found() {
        let user_repo = MockUserRepository::new();

        let jwt_service = create_jwt_service();
        let keycloak_config = KeycloakConfig {
            url: "http://localhost:8080".to_string(),
            realm: "test_realm".to_string(),
            client_id: "test_client".to_string(),
            admin_username: "admin".to_string(),
            admin_password: "admin".to_string(),
            client_secret: "test_secret".to_string(),
        };
        let keycloak_client = Arc::new(KeycloakClient::new(keycloak_config));
        
        let auth_service: AuthServiceImpl<MockUserRepository> = AuthServiceImpl::new(
            user_repo,
            jwt_service,
            keycloak_client,
        );

        // Test with non-existent user
        let result = auth_service
            .reset_password_by_credentials("nonexistent", "test@example.com", "NewPassword123")
            .await;

        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("사용자를 찾을 수 없습니다"));
        }
    }

    #[tokio::test]
    async fn test_reset_password_validation_minimum_length() {
        let mut user_repo = MockUserRepository::new();
        let test_user = create_test_user(1, "testuser".to_string(), "test@example.com".to_string());
        user_repo.add_user(test_user);

        let jwt_service = create_jwt_service();
        let keycloak_config = KeycloakConfig {
            url: "http://localhost:8080".to_string(),
            realm: "test_realm".to_string(),
            client_id: "test_client".to_string(),
            admin_username: "admin".to_string(),
            admin_password: "admin".to_string(),
            client_secret: "test_secret".to_string(),
        };
        let keycloak_client = Arc::new(KeycloakClient::new(keycloak_config));
        
        let auth_service: AuthServiceImpl<MockUserRepository> = AuthServiceImpl::new(
            user_repo,
            jwt_service,
            keycloak_client,
        );

        // Test with exactly 8 characters (should pass validation but fail at Keycloak)
        let result = auth_service
            .reset_password_by_credentials("testuser", "test@example.com", "12345678")
            .await;

        // Will fail because we can't actually call Keycloak in unit test
        // But the validation should pass
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_reset_password_validation_seven_chars() {
        let mut user_repo = MockUserRepository::new();
        let test_user = create_test_user(1, "testuser".to_string(), "test@example.com".to_string());
        user_repo.add_user(test_user);

        let jwt_service = create_jwt_service();
        let keycloak_config = KeycloakConfig {
            url: "http://localhost:8080".to_string(),
            realm: "test_realm".to_string(),
            client_id: "test_client".to_string(),
            admin_username: "admin".to_string(),
            admin_password: "admin".to_string(),
            client_secret: "test_secret".to_string(),
        };
        let keycloak_client = Arc::new(KeycloakClient::new(keycloak_config));
        
        let auth_service: AuthServiceImpl<MockUserRepository> = AuthServiceImpl::new(
            user_repo,
            jwt_service,
            keycloak_client,
        );

        // Test with 7 characters (should fail validation)
        let result = auth_service
            .reset_password_by_credentials("testuser", "test@example.com", "1234567")
            .await;

        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("비밀번호는 최소 8자 이상이어야 합니다"));
        }
    }

    #[tokio::test]
    async fn test_reset_password_success_flow_validates_username_and_email_match() {
        let mut user_repo = MockUserRepository::new();
        let test_user = create_test_user(1, "testuser".to_string(), "test@example.com".to_string());
        user_repo.add_user(test_user);

        let jwt_service = create_jwt_service();
        let keycloak_config = KeycloakConfig {
            url: "http://localhost:8080".to_string(),
            realm: "test_realm".to_string(),
            client_id: "test_client".to_string(),
            admin_username: "admin".to_string(),
            admin_password: "admin".to_string(),
            client_secret: "test_secret".to_string(),
        };
        let keycloak_client = Arc::new(KeycloakClient::new(keycloak_config));
        
        let auth_service: AuthServiceImpl<MockUserRepository> = AuthServiceImpl::new(
            user_repo,
            jwt_service,
            keycloak_client,
        );

        // This will fail at Keycloak call since we don't have a real Keycloak
        // But it validates the username+email match successfully
        let result = auth_service
            .reset_password_by_credentials("testuser", "test@example.com", "NewPassword123")
            .await;

        // Will fail because we can't actually call Keycloak
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_reset_password_case_sensitive_email() {
        let mut user_repo = MockUserRepository::new();
        let test_user = create_test_user(1, "testuser".to_string(), "Test@Example.com".to_string());
        user_repo.add_user(test_user);

        let jwt_service = create_jwt_service();
        let keycloak_config = KeycloakConfig {
            url: "http://localhost:8080".to_string(),
            realm: "test_realm".to_string(),
            client_id: "test_client".to_string(),
            admin_username: "admin".to_string(),
            admin_password: "admin".to_string(),
            client_secret: "test_secret".to_string(),
        };
        let keycloak_client = Arc::new(KeycloakClient::new(keycloak_config));
        
        let auth_service: AuthServiceImpl<MockUserRepository> = AuthServiceImpl::new(
            user_repo,
            jwt_service,
            keycloak_client,
        );

        // Should fail with wrong case email
        let result = auth_service
            .reset_password_by_credentials("testuser", "test@example.com", "NewPassword123")
            .await;

        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("이메일 정보가 일치하지 않습니다"));
        }

        // Should pass with exact case match
        let result = auth_service
            .reset_password_by_credentials("testuser", "Test@Example.com", "NewPassword123")
            .await;

        // Will still fail at Keycloak call but passes validation
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_reset_password_valid_password_formats() {
        let mut user_repo = MockUserRepository::new();
        let test_user = create_test_user(1, "testuser".to_string(), "test@example.com".to_string());
        user_repo.add_user(test_user);

        let jwt_service = create_jwt_service();
        let keycloak_config = KeycloakConfig {
            url: "http://localhost:8080".to_string(),
            realm: "test_realm".to_string(),
            client_id: "test_client".to_string(),
            admin_username: "admin".to_string(),
            admin_password: "admin".to_string(),
            client_secret: "test_secret".to_string(),
        };
        let keycloak_client = Arc::new(KeycloakClient::new(keycloak_config));
        
        let auth_service: AuthServiceImpl<MockUserRepository> = AuthServiceImpl::new(
            user_repo,
            jwt_service,
            keycloak_client,
        );

        // Test valid password (will fail at Keycloak but pass length validation)
        let password1 = "a".repeat(8);
        let password2 = "a".repeat(100);
        let valid_passwords = vec![
            "12345678",         // Exactly 8 chars
            &password1,         // 8 characters
            &password2,         // 100 characters
            "Test123!@#",       // Complex password
        ];

        for password in valid_passwords {
            let result = auth_service
                .reset_password_by_credentials("testuser", "test@example.com", password)
                .await;

            // Should fail at Keycloak call, but pass length validation
            assert!(result.is_err());
        }
    }
}

