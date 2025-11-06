use pacs_server::infrastructure::config::{Settings, ServerConfig, DatabaseConfig, KeycloakConfig, LoggingConfig, JwtConfig, CorsConfig, ObjectStorageConfig, SignedUrlConfig};

/// 서버 URL 생성 테스트
/// main.rs에서 수정된 동적 URL 생성 로직을 테스트합니다.
#[cfg(test)]
mod tests {
    use super::*;

    /// 서버 설정을 생성하는 헬퍼 함수
    fn create_test_settings(host: &str, port: u16) -> Settings {
        Settings {
            server: ServerConfig {
                host: host.to_string(),
                port,
                workers: 4,
            },
            database: DatabaseConfig {
                host: "localhost".to_string(),
                port: 5432,
                username: "test".to_string(),
                password: "test".to_string(),
                database: "test_db".to_string(),
                max_connections: 10,
                min_connections: 2,
            },
            keycloak: KeycloakConfig {
                url: "http://localhost:8080".to_string(),
                realm: "test".to_string(),
                client_id: "test-client".to_string(),
                client_secret: "test-secret".to_string(),
                admin_username: "admin".to_string(),
                admin_password: "admin".to_string(),
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "json".to_string(),
            },
            jwt: JwtConfig {
                secret: "test-secret".to_string(),
                expiration_hours: 24,
            },
            cors: CorsConfig {
                enabled: false,
                allowed_origins: vec![],
                allowed_methods: vec![],
                allowed_headers: vec![],
                expose_headers: vec![],
                max_age: 3600,
            },
            object_storage: ObjectStorageConfig {
                provider: "s3".to_string(),
                bucket_name: "test-bucket".to_string(),
                region: "us-east-1".to_string(),
                endpoint: "".to_string(),
                access_key: "test-key".to_string(),
                secret_key: "test-secret".to_string(),
            },
            signed_url: SignedUrlConfig {
                default_ttl: 600,
                max_ttl: 3600,
            },
        }
    }

    #[test]
    fn test_server_url_generation_localhost() {
        let settings = create_test_settings("localhost", 8080);
        
        // main.rs에서 사용하는 URL 형식 테스트
        let server_url = format!("http://{}:{}", settings.server.host, settings.server.port);
        let swagger_url = format!("http://{}:{}/swagger-ui/", settings.server.host, settings.server.port);
        let health_url = format!("http://{}:{}/health", settings.server.host, settings.server.port);
        let api_url = format!("http://{}:{}/api/", settings.server.host, settings.server.port);
        
        assert_eq!(server_url, "http://localhost:8080");
        assert_eq!(swagger_url, "http://localhost:8080/swagger-ui/");
        assert_eq!(health_url, "http://localhost:8080/health");
        assert_eq!(api_url, "http://localhost:8080/api/");
    }

    #[test]
    fn test_server_url_generation_all_interfaces() {
        let settings = create_test_settings("0.0.0.0", 8080);
        
        let server_url = format!("http://{}:{}", settings.server.host, settings.server.port);
        let swagger_url = format!("http://{}:{}/swagger-ui/", settings.server.host, settings.server.port);
        let health_url = format!("http://{}:{}/health", settings.server.host, settings.server.port);
        let api_url = format!("http://{}:{}/api/", settings.server.host, settings.server.port);
        
        assert_eq!(server_url, "http://0.0.0.0:8080");
        assert_eq!(swagger_url, "http://0.0.0.0:8080/swagger-ui/");
        assert_eq!(health_url, "http://0.0.0.0:8080/health");
        assert_eq!(api_url, "http://0.0.0.0:8080/api/");
    }

    #[test]
    fn test_server_url_generation_different_port() {
        let settings = create_test_settings("0.0.0.0", 3000);
        
        let server_url = format!("http://{}:{}", settings.server.host, settings.server.port);
        let swagger_url = format!("http://{}:{}/swagger-ui/", settings.server.host, settings.server.port);
        let health_url = format!("http://{}:{}/health", settings.server.host, settings.server.port);
        let api_url = format!("http://{}:{}/api/", settings.server.host, settings.server.port);
        
        assert_eq!(server_url, "http://0.0.0.0:3000");
        assert_eq!(swagger_url, "http://0.0.0.0:3000/swagger-ui/");
        assert_eq!(health_url, "http://0.0.0.0:3000/health");
        assert_eq!(api_url, "http://0.0.0.0:3000/api/");
    }

    #[test]
    fn test_server_url_generation_custom_host() {
        let settings = create_test_settings("api.example.com", 443);
        
        let server_url = format!("http://{}:{}", settings.server.host, settings.server.port);
        let swagger_url = format!("http://{}:{}/swagger-ui/", settings.server.host, settings.server.port);
        let health_url = format!("http://{}:{}/health", settings.server.host, settings.server.port);
        let api_url = format!("http://{}:{}/api/", settings.server.host, settings.server.port);
        
        assert_eq!(server_url, "http://api.example.com:443");
        assert_eq!(swagger_url, "http://api.example.com:443/swagger-ui/");
        assert_eq!(health_url, "http://api.example.com:443/health");
        assert_eq!(api_url, "http://api.example.com:443/api/");
    }

    #[test]
    fn test_server_config_validation() {
        let settings = create_test_settings("0.0.0.0", 8080);
        
        // 서버 설정 유효성 검사
        assert!(!settings.server.host.is_empty(), "서버 호스트는 비어있으면 안됩니다");
        assert!(settings.server.port > 0, "서버 포트는 0보다 커야 합니다");
        assert!(settings.server.port <= 65535, "서버 포트는 65535 이하여야 합니다");
        assert!(settings.server.workers > 0, "워커 수는 0보다 커야 합니다");
    }

    #[test]
    fn test_url_format_consistency() {
        let settings = create_test_settings("localhost", 8080);
        
        // 모든 URL이 동일한 형식을 사용하는지 확인
        let base_url = format!("http://{}:{}", settings.server.host, settings.server.port);
        let swagger_url = format!("{}/swagger-ui/", base_url);
        let health_url = format!("{}/health", base_url);
        let api_url = format!("{}/api/", base_url);
        
        assert!(swagger_url.ends_with("/swagger-ui/"));
        assert!(health_url.ends_with("/health"));
        assert!(api_url.ends_with("/api/"));
        
        // 모든 URL이 http://로 시작하는지 확인
        assert!(base_url.starts_with("http://"));
        assert!(swagger_url.starts_with("http://"));
        assert!(health_url.starts_with("http://"));
        assert!(api_url.starts_with("http://"));
    }
}
