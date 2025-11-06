use pacs_server::infrastructure::config::Settings;

/// 환경변수 기본값 테스트 (단순화된 버전)
/// settings.rs의 with_env_defaults() 메서드를 테스트합니다.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_with_env_defaults_creation() {
        // with_env_defaults() 메서드가 정상적으로 작동하는지 확인
        let result = Settings::with_env_defaults();
        assert!(result.is_ok(), "Settings::with_env_defaults() should succeed");
        
        let settings = result.unwrap();
        
        // 기본값들이 설정되어 있는지 확인
        assert!(!settings.server.host.is_empty(), "Server host should not be empty");
        assert!(settings.server.port > 0, "Server port should be positive");
        assert!(settings.server.workers > 0, "Server workers should be positive");
        
        assert!(!settings.database.host.is_empty(), "Database host should not be empty");
        assert!(settings.database.port > 0, "Database port should be positive");
        assert!(!settings.database.username.is_empty(), "Database username should not be empty");
        assert!(!settings.database.password.is_empty(), "Database password should not be empty");
        assert!(!settings.database.database.is_empty(), "Database name should not be empty");
        assert!(settings.database.max_connections > 0, "Database max_connections should be positive");
        assert!(settings.database.min_connections > 0, "Database min_connections should be positive");
        
        assert!(!settings.keycloak.url.is_empty(), "Keycloak URL should not be empty");
        assert!(!settings.keycloak.realm.is_empty(), "Keycloak realm should not be empty");
        assert!(!settings.keycloak.client_id.is_empty(), "Keycloak client_id should not be empty");
        
        assert!(!settings.logging.level.is_empty(), "Logging level should not be empty");
        assert!(!settings.logging.format.is_empty(), "Logging format should not be empty");
        
        assert!(!settings.jwt.secret.is_empty(), "JWT secret should not be empty");
        assert!(settings.jwt.expiration_hours > 0, "JWT expiration_hours should be positive");
        
        assert!(!settings.cors.allowed_origins.is_empty(), "CORS allowed_origins should not be empty");
        assert!(!settings.cors.allowed_methods.is_empty(), "CORS allowed_methods should not be empty");
        assert!(!settings.cors.allowed_headers.is_empty(), "CORS allowed_headers should not be empty");
        assert!(!settings.cors.expose_headers.is_empty(), "CORS expose_headers should not be empty");
        assert!(settings.cors.max_age > 0, "CORS max_age should be positive");
        
        assert!(!settings.object_storage.provider.is_empty(), "Object storage provider should not be empty");
        assert!(!settings.object_storage.bucket_name.is_empty(), "Object storage bucket_name should not be empty");
        assert!(!settings.object_storage.region.is_empty(), "Object storage region should not be empty");
        
        assert!(settings.signed_url.default_ttl > 0, "Signed URL default_ttl should be positive");
        assert!(settings.signed_url.max_ttl > 0, "Signed URL max_ttl should be positive");
        assert!(settings.signed_url.max_ttl >= settings.signed_url.default_ttl, "max_ttl should be >= default_ttl");
    }

    #[test]
    fn test_database_url_generation() {
        let settings = Settings::with_env_defaults().expect("Failed to create settings");
        let database_url = settings.database_url();
        
        // 데이터베이스 URL이 올바른 형식인지 확인
        assert!(database_url.starts_with("postgres://"), "Database URL should start with postgres://");
        assert!(database_url.contains(&settings.database.username), "Database URL should contain username");
        assert!(database_url.contains(&settings.database.password), "Database URL should contain password");
        assert!(database_url.contains(&settings.database.host), "Database URL should contain host");
        assert!(database_url.contains(&settings.database.port.to_string()), "Database URL should contain port");
        assert!(database_url.contains(&settings.database.database), "Database URL should contain database name");
    }

    #[test]
    fn test_settings_consistency() {
        let settings = Settings::with_env_defaults().expect("Failed to create settings");
        
        // 설정 값들의 일관성 확인
        assert!(settings.server.workers > 0, "Server workers should be positive");
        assert!(settings.database.max_connections >= settings.database.min_connections, 
                "max_connections should be >= min_connections");
        assert!(settings.signed_url.max_ttl >= settings.signed_url.default_ttl, 
                "max_ttl should be >= default_ttl");
        
        // CORS 설정이 올바른 형식인지 확인
        for origin in &settings.cors.allowed_origins {
            assert!(!origin.is_empty(), "CORS origin should not be empty");
        }
        
        for method in &settings.cors.allowed_methods {
            assert!(!method.is_empty(), "CORS method should not be empty");
        }
        
        for header in &settings.cors.allowed_headers {
            assert!(!header.is_empty(), "CORS header should not be empty");
        }
        
        for header in &settings.cors.expose_headers {
            assert!(!header.is_empty(), "CORS expose header should not be empty");
        }
    }

    #[test]
    fn test_settings_clone() {
        let settings1 = Settings::with_env_defaults().expect("Failed to create settings");
        let settings2 = settings1.clone();
        
        // 클론된 설정이 원본과 동일한지 확인
        assert_eq!(settings1.server.host, settings2.server.host);
        assert_eq!(settings1.server.port, settings2.server.port);
        assert_eq!(settings1.server.workers, settings2.server.workers);
        
        assert_eq!(settings1.database.host, settings2.database.host);
        assert_eq!(settings1.database.port, settings2.database.port);
        assert_eq!(settings1.database.username, settings2.database.username);
        assert_eq!(settings1.database.password, settings2.database.password);
        assert_eq!(settings1.database.database, settings2.database.database);
        assert_eq!(settings1.database.max_connections, settings2.database.max_connections);
        assert_eq!(settings1.database.min_connections, settings2.database.min_connections);
        
        assert_eq!(settings1.jwt.secret, settings2.jwt.secret);
        assert_eq!(settings1.jwt.expiration_hours, settings2.jwt.expiration_hours);
        
        assert_eq!(settings1.cors.enabled, settings2.cors.enabled);
        assert_eq!(settings1.cors.allowed_origins, settings2.cors.allowed_origins);
        assert_eq!(settings1.cors.allowed_methods, settings2.cors.allowed_methods);
        assert_eq!(settings1.cors.allowed_headers, settings2.cors.allowed_headers);
        assert_eq!(settings1.cors.expose_headers, settings2.cors.expose_headers);
        assert_eq!(settings1.cors.max_age, settings2.cors.max_age);
        
        assert_eq!(settings1.object_storage.provider, settings2.object_storage.provider);
        assert_eq!(settings1.object_storage.bucket_name, settings2.object_storage.bucket_name);
        assert_eq!(settings1.object_storage.region, settings2.object_storage.region);
        assert_eq!(settings1.object_storage.endpoint, settings2.object_storage.endpoint);
        assert_eq!(settings1.object_storage.access_key, settings2.object_storage.access_key);
        assert_eq!(settings1.object_storage.secret_key, settings2.object_storage.secret_key);
        
        assert_eq!(settings1.signed_url.default_ttl, settings2.signed_url.default_ttl);
        assert_eq!(settings1.signed_url.max_ttl, settings2.signed_url.max_ttl);
    }
}
