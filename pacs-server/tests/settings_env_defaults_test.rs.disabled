use pacs_server::infrastructure::config::Settings;
use std::env;

/// 환경변수 기본값 테스트
/// settings.rs의 with_env_defaults() 메서드를 테스트합니다.
#[cfg(test)]
mod tests {
    use super::*;

    /// 환경변수를 초기화하는 헬퍼 함수
    fn clear_env_vars() {
        // APP_ 접두사 환경변수들
        env::remove_var("APP_SERVER__HOST");
        env::remove_var("APP_SERVER__PORT");
        env::remove_var("APP_SERVER__WORKERS");
        env::remove_var("APP_DATABASE__HOST");
        env::remove_var("APP_DATABASE__PORT");
        env::remove_var("APP_DATABASE__USERNAME");
        env::remove_var("APP_DATABASE__PASSWORD");
        env::remove_var("APP_DATABASE__DATABASE");
        env::remove_var("APP_DATABASE__MAX_CONNECTIONS");
        env::remove_var("APP_DATABASE__MIN_CONNECTIONS");
        env::remove_var("APP_KEYCLOAK__URL");
        env::remove_var("APP_KEYCLOAK__REALM");
        env::remove_var("APP_KEYCLOAK__CLIENT_ID");
        env::remove_var("APP_KEYCLOAK__CLIENT_SECRET");
        env::remove_var("APP_LOGGING__LEVEL");
        env::remove_var("APP_LOGGING__FORMAT");
        env::remove_var("APP_JWT__SECRET");
        env::remove_var("APP_JWT__EXPIRATION_HOURS");
        env::remove_var("APP_CORS__ENABLED");
        env::remove_var("APP_CORS__ALLOWED_ORIGINS");
        env::remove_var("APP_CORS__ALLOWED_METHODS");
        env::remove_var("APP_CORS__ALLOWED_HEADERS");
        env::remove_var("APP_CORS__EXPOSE_HEADERS");
        env::remove_var("APP_CORS__MAX_AGE");
        env::remove_var("APP_OBJECT_STORAGE__PROVIDER");
        env::remove_var("APP_OBJECT_STORAGE__BUCKET_NAME");
        env::remove_var("APP_OBJECT_STORAGE__REGION");
        env::remove_var("APP_OBJECT_STORAGE__ENDPOINT");
        env::remove_var("APP_OBJECT_STORAGE__ACCESS_KEY_ID");
        env::remove_var("APP_OBJECT_STORAGE__SECRET_ACCESS_KEY");
        env::remove_var("APP_SIGNED_URL__DEFAULT_TTL");
        env::remove_var("APP_SIGNED_URL__MAX_TTL");

        // 단순 환경변수들
        env::remove_var("SERVER_HOST");
        env::remove_var("SERVER_PORT");
        env::remove_var("SERVER_WORKERS");
        env::remove_var("DATABASE_HOST");
        env::remove_var("DATABASE_PORT");
        env::remove_var("DATABASE_USERNAME");
        env::remove_var("DATABASE_PASSWORD");
        env::remove_var("DATABASE_NAME");
        env::remove_var("DATABASE_MAX_CONNECTIONS");
        env::remove_var("DATABASE_MIN_CONNECTIONS");
        env::remove_var("KEYCLOAK_URL");
        env::remove_var("KEYCLOAK_REALM");
        env::remove_var("KEYCLOAK_CLIENT_ID");
        env::remove_var("KEYCLOAK_CLIENT_SECRET");
        env::remove_var("LOG_LEVEL");
        env::remove_var("LOG_FORMAT");
        env::remove_var("JWT_SECRET");
        env::remove_var("JWT_EXPIRATION_HOURS");
        env::remove_var("CORS_ENABLED");
        env::remove_var("CORS_ALLOWED_ORIGINS");
        env::remove_var("CORS_ALLOWED_METHODS");
        env::remove_var("CORS_ALLOWED_HEADERS");
        env::remove_var("CORS_EXPOSE_HEADERS");
        env::remove_var("CORS_MAX_AGE");
        env::remove_var("OBJECT_STORAGE_PROVIDER");
        env::remove_var("OBJECT_STORAGE_BUCKET_NAME");
        env::remove_var("OBJECT_STORAGE_REGION");
        env::remove_var("OBJECT_STORAGE_ENDPOINT");
        env::remove_var("OBJECT_STORAGE_ACCESS_KEY_ID");
        env::remove_var("OBJECT_STORAGE_SECRET_ACCESS_KEY");
        env::remove_var("SIGNED_URL_DEFAULT_TTL");
        env::remove_var("SIGNED_URL_MAX_TTL");
    }

    #[test]
    fn test_settings_with_default_values() {
        clear_env_vars();
        
        // 환경변수 완전 초기화를 위해 추가 정리
        env::remove_var("APP_SERVER__HOST");
        env::remove_var("APP_SERVER__PORT");
        env::remove_var("APP_SERVER__WORKERS");
        env::remove_var("APP_DATABASE__HOST");
        env::remove_var("APP_DATABASE__PORT");
        env::remove_var("APP_DATABASE__USERNAME");
        env::remove_var("APP_DATABASE__PASSWORD");
        env::remove_var("APP_DATABASE__DATABASE");
        env::remove_var("APP_DATABASE__MAX_CONNECTIONS");
        env::remove_var("APP_DATABASE__MIN_CONNECTIONS");
        env::remove_var("APP_KEYCLOAK__URL");
        env::remove_var("APP_KEYCLOAK__REALM");
        env::remove_var("APP_KEYCLOAK__CLIENT_ID");
        env::remove_var("APP_KEYCLOAK__CLIENT_SECRET");
        env::remove_var("APP_LOGGING__LEVEL");
        env::remove_var("APP_LOGGING__FORMAT");
        env::remove_var("APP_JWT__SECRET");
        env::remove_var("APP_JWT__EXPIRATION_HOURS");
        env::remove_var("APP_CORS__ENABLED");
        env::remove_var("APP_CORS__ALLOWED_ORIGINS");
        env::remove_var("APP_CORS__ALLOWED_METHODS");
        env::remove_var("APP_CORS__ALLOWED_HEADERS");
        env::remove_var("APP_CORS__EXPOSE_HEADERS");
        env::remove_var("APP_CORS__MAX_AGE");
        env::remove_var("APP_OBJECT_STORAGE__PROVIDER");
        env::remove_var("APP_OBJECT_STORAGE__BUCKET_NAME");
        env::remove_var("APP_OBJECT_STORAGE__REGION");
        env::remove_var("APP_OBJECT_STORAGE__ENDPOINT");
        env::remove_var("APP_OBJECT_STORAGE__ACCESS_KEY_ID");
        env::remove_var("APP_OBJECT_STORAGE__SECRET_ACCESS_KEY");
        env::remove_var("APP_SIGNED_URL__DEFAULT_TTL");
        env::remove_var("APP_SIGNED_URL__MAX_TTL");
        env::remove_var("SERVER_HOST");
        env::remove_var("SERVER_PORT");
        env::remove_var("SERVER_WORKERS");
        env::remove_var("DATABASE_HOST");
        env::remove_var("DATABASE_PORT");
        env::remove_var("DATABASE_USERNAME");
        env::remove_var("DATABASE_PASSWORD");
        env::remove_var("DATABASE_NAME");
        env::remove_var("DATABASE_MAX_CONNECTIONS");
        env::remove_var("DATABASE_MIN_CONNECTIONS");
        env::remove_var("KEYCLOAK_URL");
        env::remove_var("KEYCLOAK_REALM");
        env::remove_var("KEYCLOAK_CLIENT_ID");
        env::remove_var("KEYCLOAK_CLIENT_SECRET");
        env::remove_var("LOG_LEVEL");
        env::remove_var("LOG_FORMAT");
        env::remove_var("JWT_SECRET");
        env::remove_var("JWT_EXPIRATION_HOURS");
        env::remove_var("CORS_ENABLED");
        env::remove_var("CORS_ALLOWED_ORIGINS");
        env::remove_var("CORS_ALLOWED_METHODS");
        env::remove_var("CORS_ALLOWED_HEADERS");
        env::remove_var("CORS_EXPOSE_HEADERS");
        env::remove_var("CORS_MAX_AGE");
        env::remove_var("OBJECT_STORAGE_PROVIDER");
        env::remove_var("OBJECT_STORAGE_BUCKET_NAME");
        env::remove_var("OBJECT_STORAGE_REGION");
        env::remove_var("OBJECT_STORAGE_ENDPOINT");
        env::remove_var("OBJECT_STORAGE_ACCESS_KEY_ID");
        env::remove_var("OBJECT_STORAGE_SECRET_ACCESS_KEY");
        env::remove_var("SIGNED_URL_DEFAULT_TTL");
        env::remove_var("SIGNED_URL_MAX_TTL");

        let settings = Settings::with_env_defaults().expect("Failed to create settings with defaults");

        // 서버 설정 검증
        assert_eq!(settings.server.host, "0.0.0.0");
        assert_eq!(settings.server.port, 8080);
        assert_eq!(settings.server.workers, 4);

        // 데이터베이스 설정 검증
        assert_eq!(settings.database.host, "localhost");
        assert_eq!(settings.database.port, 5432);
        assert_eq!(settings.database.username, "admin");
        assert_eq!(settings.database.password, "admin123");
        assert_eq!(settings.database.database, "pacs_db");
        assert_eq!(settings.database.max_connections, 10);
        assert_eq!(settings.database.min_connections, 2);

        // Keycloak 설정 검증
        assert_eq!(settings.keycloak.url, "http://localhost:8080");
        assert_eq!(settings.keycloak.realm, "pacs");
        assert_eq!(settings.keycloak.client_id, "pacs-server");
        assert_eq!(settings.keycloak.client_secret, "");

        // 로깅 설정 검증
        assert_eq!(settings.logging.level, "info");
        assert_eq!(settings.logging.format, "json");

        // JWT 설정 검증
        assert_eq!(settings.jwt.secret, "your-secret-key-change-this-in-production");
        assert_eq!(settings.jwt.expiration_hours, 24);

        // CORS 설정 검증
        assert_eq!(settings.cors.enabled, false);
        assert_eq!(settings.cors.allowed_origins, vec!["http://localhost:3000", "http://localhost:8080"]);
        assert_eq!(settings.cors.allowed_methods, vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"]);
        assert_eq!(settings.cors.allowed_headers, vec!["Content-Type", "Authorization", "X-Requested-With"]);
        assert_eq!(settings.cors.expose_headers, vec!["Content-Length", "X-Total-Count"]);
        assert_eq!(settings.cors.max_age, 3600);

        // 객체 저장소 설정 검증
        assert_eq!(settings.object_storage.provider, "s3");
        assert_eq!(settings.object_storage.bucket_name, "pacs-masks");
        assert_eq!(settings.object_storage.region, "us-east-1");
        assert_eq!(settings.object_storage.endpoint, "");
        assert_eq!(settings.object_storage.access_key, "");
        assert_eq!(settings.object_storage.secret_key, "");

        // 서명된 URL 설정 검증
        assert_eq!(settings.signed_url.default_ttl, 600);
        assert_eq!(settings.signed_url.max_ttl, 3600);
    }

    #[test]
    fn test_settings_with_app_prefix_env_vars() {
        clear_env_vars();

        // APP_ 접두사 환경변수 설정
        env::set_var("APP_SERVER__HOST", "192.168.1.100");
        env::set_var("APP_SERVER__PORT", "9090");
        env::set_var("APP_DATABASE__HOST", "db.example.com");
        env::set_var("APP_DATABASE__PORT", "5433");
        env::set_var("APP_JWT__SECRET", "custom-jwt-secret");
        env::set_var("APP_CORS__ENABLED", "true");

        let settings = Settings::with_env_defaults().expect("Failed to create settings with env vars");

        // APP_ 접두사 환경변수가 우선 적용되는지 확인
        assert_eq!(settings.server.host, "192.168.1.100");
        assert_eq!(settings.server.port, 9090);
        assert_eq!(settings.database.host, "db.example.com");
        assert_eq!(settings.database.port, 5433);
        assert_eq!(settings.jwt.secret, "custom-jwt-secret");
        assert_eq!(settings.cors.enabled, true);

        // 설정되지 않은 값들은 기본값 사용
        assert_eq!(settings.server.workers, 4);
        assert_eq!(settings.database.username, "admin");
    }

    #[test]
    fn test_settings_with_simple_env_vars() {
        clear_env_vars();

        // 단순 환경변수 설정 (APP_ 접두사 없음)
        env::set_var("SERVER_HOST", "10.0.0.1");
        env::set_var("SERVER_PORT", "8081");
        env::set_var("DATABASE_HOST", "localhost");
        env::set_var("JWT_SECRET", "simple-jwt-secret");

        let settings = Settings::with_env_defaults().expect("Failed to create settings with simple env vars");

        // 단순 환경변수가 적용되는지 확인
        assert_eq!(settings.server.host, "10.0.0.1");
        assert_eq!(settings.server.port, 8081);
        assert_eq!(settings.database.host, "localhost");
        assert_eq!(settings.jwt.secret, "simple-jwt-secret");

        // 설정되지 않은 값들은 기본값 사용
        assert_eq!(settings.server.workers, 4);
        assert_eq!(settings.database.port, 5432);
    }

    #[test]
    fn test_settings_env_var_priority() {
        clear_env_vars();

        // APP_ 접두사와 단순 환경변수 모두 설정
        env::set_var("APP_SERVER__HOST", "app-prefix-host");
        env::set_var("SERVER_HOST", "simple-host");
        env::set_var("APP_DATABASE__HOST", "app-db-host");
        env::set_var("DATABASE_HOST", "simple-db-host");

        let settings = Settings::with_env_defaults().expect("Failed to create settings with priority test");

        // APP_ 접두사가 우선 적용되는지 확인
        assert_eq!(settings.server.host, "app-prefix-host");
        assert_eq!(settings.database.host, "app-db-host");
    }

    #[test]
    fn test_settings_cors_parsing() {
        clear_env_vars();

        // CORS 설정을 쉼표로 구분된 문자열로 설정
        env::set_var("APP_CORS__ALLOWED_ORIGINS", "https://example.com,https://app.example.com");
        env::set_var("APP_CORS__ALLOWED_METHODS", "GET,POST,PUT");
        env::set_var("APP_CORS__ALLOWED_HEADERS", "Authorization,Content-Type");

        let settings = Settings::with_env_defaults().expect("Failed to create settings with CORS parsing");

        // 쉼표로 구분된 문자열이 올바르게 파싱되는지 확인
        assert_eq!(settings.cors.allowed_origins, vec!["https://example.com", "https://app.example.com"]);
        assert_eq!(settings.cors.allowed_methods, vec!["GET", "POST", "PUT"]);
        assert_eq!(settings.cors.allowed_headers, vec!["Authorization", "Content-Type"]);
    }

    #[test]
    fn test_settings_numeric_parsing() {
        clear_env_vars();

        // 숫자 값들을 문자열로 설정
        env::set_var("APP_SERVER__PORT", "9999");
        env::set_var("APP_SERVER__WORKERS", "8");
        env::set_var("APP_DATABASE__MAX_CONNECTIONS", "20");
        env::set_var("APP_CORS__MAX_AGE", "7200");

        let settings = Settings::with_env_defaults().expect("Failed to create settings with numeric parsing");

        // 문자열이 올바르게 숫자로 파싱되는지 확인
        assert_eq!(settings.server.port, 9999);
        assert_eq!(settings.server.workers, 8);
        assert_eq!(settings.database.max_connections, 20);
        assert_eq!(settings.cors.max_age, 7200);
    }

    #[test]
    fn test_settings_invalid_numeric_fallback() {
        clear_env_vars();

        // 잘못된 숫자 값 설정
        env::set_var("APP_SERVER__PORT", "invalid-port");
        env::set_var("APP_SERVER__WORKERS", "not-a-number");

        let settings = Settings::with_env_defaults().expect("Failed to create settings with invalid numeric");

        // 잘못된 값은 기본값으로 fallback되는지 확인
        assert_eq!(settings.server.port, 8080); // 기본값
        assert_eq!(settings.server.workers, 4); // 기본값
    }

    #[test]
    fn test_settings_boolean_parsing() {
        clear_env_vars();

        // 불린 값들을 문자열로 설정
        env::set_var("APP_CORS__ENABLED", "true");
        env::set_var("APP_CORS__ENABLED", "false");

        let settings = Settings::with_env_defaults().expect("Failed to create settings with boolean parsing");

        // 마지막 설정값이 적용되는지 확인
        assert_eq!(settings.cors.enabled, false);
    }

    #[test]
    fn test_database_url_generation() {
        clear_env_vars();

        let settings = Settings::with_env_defaults().expect("Failed to create settings for database URL test");

        let database_url = settings.database_url();
        assert_eq!(database_url, "postgres://admin:admin123@localhost:5432/pacs_db");
    }

    #[test]
    fn test_database_url_with_env_override() {
        clear_env_vars();

        // DATABASE_URL 환경변수 설정
        env::set_var("DATABASE_URL", "postgres://user:pass@host:5432/db");

        let settings = Settings::with_env_defaults().expect("Failed to create settings for database URL override test");

        let database_url = settings.database_url();
        assert_eq!(database_url, "postgres://user:pass@host:5432/db");

        env::remove_var("DATABASE_URL");
    }
}
