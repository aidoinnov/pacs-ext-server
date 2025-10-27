#[cfg(test)]
mod auth_find_username_tests {
    use std::collections::HashMap;
    use uuid::Uuid;
    use chrono::Utc;

    use pacs_server::application::dto::auth_dto::mask_email;
    use pacs_server::domain::entities::User;
    use pacs_server::domain::repositories::UserRepository;
    use pacs_server::domain::services::{AuthService, AuthServiceImpl};
    use pacs_server::infrastructure::auth::JwtService;
    use pacs_server::infrastructure::config::JwtConfig;
    use pacs_server::infrastructure::external::KeycloakClient;
    use std::sync::Arc;

    // Mock UserRepository for testing
    #[derive(Clone)]
    struct MockUserRepository {
        users: HashMap<i32, User>,
        users_by_email: HashMap<String, User>,
    }

    impl MockUserRepository {
        fn new() -> Self {
            Self {
                users: HashMap::new(),
                users_by_email: HashMap::new(),
            }
        }

        fn add_user(&mut self, user: User) {
            self.users.insert(user.id, user.clone());
            self.users_by_email.insert(user.email.clone(), user);
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
            Ok(self.users.values().find(|u| u.username == username).cloned())
        }

        async fn find_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error> {
            Ok(self.users_by_email.get(email).cloned())
        }

        async fn find_all(&self) -> Result<Vec<User>, sqlx::Error> {
            Ok(self.users.values().cloned().collect())
        }

        async fn create(&self, _new_user: pacs_server::domain::entities::NewUser) -> Result<User, sqlx::Error> {
            Err(sqlx::Error::RowNotFound)
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

    fn create_keycloak_client() -> Arc<KeycloakClient> {
        let config = pacs_server::infrastructure::config::KeycloakConfig {
            url: "http://localhost:8080".to_string(),
            realm: "test_realm".to_string(),
            client_id: "test_client".to_string(),
            admin_username: "admin".to_string(),
            admin_password: "admin".to_string(),
        };
        Arc::new(KeycloakClient::new(config))
    }

    #[tokio::test]
    async fn test_find_username_success() {
        let mut user_repo = MockUserRepository::new();
        let test_user = create_test_user(1, "john.doe".to_string(), "john@example.com".to_string());
        user_repo.add_user(test_user.clone());

        let jwt_service = create_jwt_service();
        let keycloak_client = create_keycloak_client();
        let auth_service: AuthServiceImpl<MockUserRepository> = AuthServiceImpl::new(
            user_repo,
            jwt_service,
            keycloak_client,
        );

        let result = auth_service.find_username_by_email("john@example.com").await;
        assert!(result.is_ok());
        
        let user = result.unwrap();
        assert_eq!(user.username, "john.doe");
        assert_eq!(user.email, "john@example.com");
    }

    #[tokio::test]
    async fn test_find_username_not_found() {
        let user_repo = MockUserRepository::new();

        let jwt_service = create_jwt_service();
        let keycloak_client = create_keycloak_client();
        let auth_service: AuthServiceImpl<MockUserRepository> = AuthServiceImpl::new(
            user_repo,
            jwt_service,
            keycloak_client,
        );

        let result = auth_service.find_username_by_email("nonexistent@example.com").await;
        assert!(result.is_err());
        
        if let Err(e) = result {
            assert!(e.to_string().contains("해당 이메일로 등록된 사용자가 없습니다"));
        }
    }

    #[tokio::test]
    async fn test_mask_email_function() {
        // Test normal email masking
        assert_eq!(mask_email("john@example.com"), "j***@example.com");
        assert_eq!(mask_email("jane.smith@example.com"), "j***@example.com");
        
        // Test short email (less than 2 chars before @)
        assert_eq!(mask_email("a@example.com"), "***@example.com");
        
        // Test single char
        assert_eq!(mask_email("a@example.com"), "***@example.com");
        
        // Test edge case - no @ symbol
        assert_eq!(mask_email("noat.com"), "noat.com");
    }

    #[tokio::test]
    async fn test_find_username_response_structure() {
        let mut user_repo = MockUserRepository::new();
        let test_user = create_test_user(1, "testuser".to_string(), "test@example.com".to_string());
        user_repo.add_user(test_user);

        let jwt_service = create_jwt_service();
        let keycloak_client = create_keycloak_client();
        let auth_service: AuthServiceImpl<MockUserRepository> = AuthServiceImpl::new(
            user_repo,
            jwt_service,
            keycloak_client,
        );

        let user = auth_service.find_username_by_email("test@example.com").await.unwrap();
        let masked_email = mask_email(&user.email);
        
        assert_eq!(masked_email, "t***@example.com");
    }

    #[tokio::test]
    async fn test_find_username_case_sensitive() {
        let mut user_repo = MockUserRepository::new();
        let test_user = create_test_user(1, "testuser".to_string(), "Test@Example.com".to_string());
        user_repo.add_user(test_user.clone());

        let jwt_service = create_jwt_service();
        let keycloak_client = create_keycloak_client();
        let auth_service: AuthServiceImpl<MockUserRepository> = AuthServiceImpl::new(
            user_repo,
            jwt_service,
            keycloak_client,
        );

        // Should find exact match (case sensitive)
        let result = auth_service.find_username_by_email("Test@Example.com").await;
        assert!(result.is_ok());

        // Should not find with different case
        let result = auth_service.find_username_by_email("test@example.com").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_find_username_multiple_users() {
        let mut user_repo = MockUserRepository::new();
        let user1 = create_test_user(1, "alice".to_string(), "alice@example.com".to_string());
        let user2 = create_test_user(2, "bob".to_string(), "bob@example.com".to_string());
        let user3 = create_test_user(3, "charlie".to_string(), "charlie@example.com".to_string());
        
        user_repo.add_user(user1);
        user_repo.add_user(user2);
        user_repo.add_user(user3);

        let jwt_service = create_jwt_service();
        let keycloak_client = create_keycloak_client();
        let auth_service: AuthServiceImpl<MockUserRepository> = AuthServiceImpl::new(
            user_repo,
            jwt_service,
            keycloak_client,
        );

        // Find alice
        let result = auth_service.find_username_by_email("alice@example.com").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().username, "alice");

        // Find bob
        let result = auth_service.find_username_by_email("bob@example.com").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().username, "bob");

        // Find charlie
        let result = auth_service.find_username_by_email("charlie@example.com").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().username, "charlie");
    }

    #[tokio::test]
    async fn test_find_username_empty_email() {
        let user_repo = MockUserRepository::new();

        let jwt_service = create_jwt_service();
        let keycloak_client = create_keycloak_client();
        let auth_service: AuthServiceImpl<MockUserRepository> = AuthServiceImpl::new(
            user_repo,
            jwt_service,
            keycloak_client,
        );

        let result = auth_service.find_username_by_email("").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_find_username_special_characters() {
        let mut user_repo = MockUserRepository::new();
        let test_user = create_test_user(1, "user".to_string(), "user+test@example.co.uk".to_string());
        user_repo.add_user(test_user);

        let jwt_service = create_jwt_service();
        let keycloak_client = create_keycloak_client();
        let auth_service: AuthServiceImpl<MockUserRepository> = AuthServiceImpl::new(
            user_repo,
            jwt_service,
            keycloak_client,
        );

        let result = auth_service.find_username_by_email("user+test@example.co.uk").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().username, "user");
    }
}

