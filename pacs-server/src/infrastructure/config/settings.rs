use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub keycloak: KeycloakConfig,
    pub logging: LoggingConfig,
    pub jwt: JwtConfig,
    pub cors: CorsConfig,
    pub object_storage: ObjectStorageConfig,
    pub signed_url: SignedUrlConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
    pub max_connections: u32,
    pub min_connections: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct KeycloakConfig {
    pub url: String,
    pub realm: String,
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub expiration_hours: i64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CorsConfig {
    pub enabled: bool,
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub allowed_headers: Vec<String>,
    pub expose_headers: Vec<String>,
    pub max_age: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ObjectStorageConfig {
    pub provider: String,  // "s3" or "minio"
    pub bucket_name: String,
    pub region: String,
    pub endpoint: String,  // MinIO endpoint (empty for AWS S3)
    #[serde(rename = "access_key_id")]
    pub access_key: String,
    #[serde(rename = "secret_access_key")]
    pub secret_key: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SignedUrlConfig {
    pub default_ttl: u64,  // Default TTL in seconds
    pub max_ttl: u64,      // Maximum TTL in seconds
}

impl Settings {
    /// Load settings with environment variable priority
    /// Priority (highest to lowest):
    /// 1. Environment variables (with APP_ prefix)
    /// 2. .env file
    /// 3. config/{environment}.toml (default: development)
    /// 4. config/default.toml
    /// 5. Environment variable defaults (fallback)
    pub fn new() -> Result<Self, ConfigError> {
        // 1. Load .env file first (lowest priority)
        dotenvy::dotenv().ok();

        // 2. Determine run environment (default: development)
        let run_env = env::var("RUN_ENV").unwrap_or_else(|_| "development".to_string());

        let config = Config::builder()
            // Start with default config
            .add_source(File::with_name("config/default").required(false))
            // Add environment-specific config
            .add_source(File::with_name(&format!("config/{}", run_env)).required(false))
            // Add environment variables with APP_ prefix (highest priority)
            // Example: APP_SERVER__PORT=8080 -> server.port = 8080
            .add_source(
                Environment::with_prefix("APP")
                    .separator("__")
                    .try_parsing(true),
            )
            .build()?;

        config.try_deserialize()
    }

    /// Create settings with environment variable defaults
    /// This method provides fallback values from environment variables
    /// when config files are not available or incomplete
    pub fn with_env_defaults() -> Result<Self, ConfigError> {
        // Load .env file first
        dotenvy::dotenv().ok();

        // Create settings with environment variable defaults
        let settings = Settings {
            server: ServerConfig {
                host: env::var("APP_SERVER__HOST")
                    .or_else(|_| env::var("SERVER_HOST"))
                    .unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: env::var("APP_SERVER__PORT")
                    .or_else(|_| env::var("SERVER_PORT"))
                    .unwrap_or_else(|_| "8080".to_string())
                    .parse()
                    .unwrap_or(8080),
                workers: env::var("APP_SERVER__WORKERS")
                    .or_else(|_| env::var("SERVER_WORKERS"))
                    .unwrap_or_else(|_| "4".to_string())
                    .parse()
                    .unwrap_or(4),
            },
            database: DatabaseConfig {
                host: env::var("APP_DATABASE__HOST")
                    .or_else(|_| env::var("DATABASE_HOST"))
                    .unwrap_or_else(|_| "localhost".to_string()),
                port: env::var("APP_DATABASE__PORT")
                    .or_else(|_| env::var("DATABASE_PORT"))
                    .unwrap_or_else(|_| "5432".to_string())
                    .parse()
                    .unwrap_or(5432),
                username: env::var("APP_DATABASE__USERNAME")
                    .or_else(|_| env::var("DATABASE_USERNAME"))
                    .unwrap_or_else(|_| "admin".to_string()),
                password: env::var("APP_DATABASE__PASSWORD")
                    .or_else(|_| env::var("DATABASE_PASSWORD"))
                    .unwrap_or_else(|_| "admin123".to_string()),
                database: env::var("APP_DATABASE__DATABASE")
                    .or_else(|_| env::var("DATABASE_NAME"))
                    .unwrap_or_else(|_| "pacs_db".to_string()),
                max_connections: env::var("APP_DATABASE__MAX_CONNECTIONS")
                    .or_else(|_| env::var("DATABASE_MAX_CONNECTIONS"))
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()
                    .unwrap_or(10),
                min_connections: env::var("APP_DATABASE__MIN_CONNECTIONS")
                    .or_else(|_| env::var("DATABASE_MIN_CONNECTIONS"))
                    .unwrap_or_else(|_| "2".to_string())
                    .parse()
                    .unwrap_or(2),
            },
            keycloak: KeycloakConfig {
                url: env::var("APP_KEYCLOAK__URL")
                    .or_else(|_| env::var("KEYCLOAK_URL"))
                    .unwrap_or_else(|_| "http://localhost:8080".to_string()),
                realm: env::var("APP_KEYCLOAK__REALM")
                    .or_else(|_| env::var("KEYCLOAK_REALM"))
                    .unwrap_or_else(|_| "pacs".to_string()),
                client_id: env::var("APP_KEYCLOAK__CLIENT_ID")
                    .or_else(|_| env::var("KEYCLOAK_CLIENT_ID"))
                    .unwrap_or_else(|_| "pacs-server".to_string()),
                client_secret: env::var("APP_KEYCLOAK__CLIENT_SECRET")
                    .or_else(|_| env::var("KEYCLOAK_CLIENT_SECRET"))
                    .unwrap_or_else(|_| "".to_string()),
            },
            logging: LoggingConfig {
                level: env::var("APP_LOGGING__LEVEL")
                    .or_else(|_| env::var("LOG_LEVEL"))
                    .unwrap_or_else(|_| "info".to_string()),
                format: env::var("APP_LOGGING__FORMAT")
                    .or_else(|_| env::var("LOG_FORMAT"))
                    .unwrap_or_else(|_| "json".to_string()),
            },
            jwt: JwtConfig {
                secret: env::var("APP_JWT__SECRET")
                    .or_else(|_| env::var("JWT_SECRET"))
                    .unwrap_or_else(|_| "your-secret-key-change-this-in-production".to_string()),
                expiration_hours: env::var("APP_JWT__EXPIRATION_HOURS")
                    .or_else(|_| env::var("JWT_EXPIRATION_HOURS"))
                    .unwrap_or_else(|_| "24".to_string())
                    .parse()
                    .unwrap_or(24),
            },
            cors: CorsConfig {
                enabled: env::var("APP_CORS__ENABLED")
                    .or_else(|_| env::var("CORS_ENABLED"))
                    .unwrap_or_else(|_| "false".to_string())
                    .parse()
                    .unwrap_or(false),
                allowed_origins: env::var("APP_CORS__ALLOWED_ORIGINS")
                    .or_else(|_| env::var("CORS_ALLOWED_ORIGINS"))
                    .unwrap_or_else(|_| "http://localhost:3000,http://localhost:8080".to_string())
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect(),
                allowed_methods: env::var("APP_CORS__ALLOWED_METHODS")
                    .or_else(|_| env::var("CORS_ALLOWED_METHODS"))
                    .unwrap_or_else(|_| "GET,POST,PUT,DELETE,OPTIONS".to_string())
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect(),
                allowed_headers: env::var("APP_CORS__ALLOWED_HEADERS")
                    .or_else(|_| env::var("CORS_ALLOWED_HEADERS"))
                    .unwrap_or_else(|_| "Content-Type,Authorization,X-Requested-With".to_string())
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect(),
                expose_headers: env::var("APP_CORS__EXPOSE_HEADERS")
                    .or_else(|_| env::var("CORS_EXPOSE_HEADERS"))
                    .unwrap_or_else(|_| "Content-Length,X-Total-Count".to_string())
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect(),
                max_age: env::var("APP_CORS__MAX_AGE")
                    .or_else(|_| env::var("CORS_MAX_AGE"))
                    .unwrap_or_else(|_| "3600".to_string())
                    .parse()
                    .unwrap_or(3600),
            },
            object_storage: ObjectStorageConfig {
                provider: env::var("APP_OBJECT_STORAGE__PROVIDER")
                    .or_else(|_| env::var("OBJECT_STORAGE_PROVIDER"))
                    .unwrap_or_else(|_| "s3".to_string()),
                bucket_name: env::var("APP_OBJECT_STORAGE__BUCKET_NAME")
                    .or_else(|_| env::var("OBJECT_STORAGE_BUCKET_NAME"))
                    .unwrap_or_else(|_| "pacs-masks".to_string()),
                region: env::var("APP_OBJECT_STORAGE__REGION")
                    .or_else(|_| env::var("OBJECT_STORAGE_REGION"))
                    .unwrap_or_else(|_| "us-east-1".to_string()),
                endpoint: env::var("APP_OBJECT_STORAGE__ENDPOINT")
                    .or_else(|_| env::var("OBJECT_STORAGE_ENDPOINT"))
                    .unwrap_or_else(|_| "".to_string()),
                access_key: env::var("APP_OBJECT_STORAGE__ACCESS_KEY_ID")
                    .or_else(|_| env::var("OBJECT_STORAGE_ACCESS_KEY_ID"))
                    .unwrap_or_else(|_| "".to_string()),
                secret_key: env::var("APP_OBJECT_STORAGE__SECRET_ACCESS_KEY")
                    .or_else(|_| env::var("OBJECT_STORAGE_SECRET_ACCESS_KEY"))
                    .unwrap_or_else(|_| "".to_string()),
            },
            signed_url: SignedUrlConfig {
                default_ttl: env::var("APP_SIGNED_URL__DEFAULT_TTL")
                    .or_else(|_| env::var("SIGNED_URL_DEFAULT_TTL"))
                    .unwrap_or_else(|_| "600".to_string())
                    .parse()
                    .unwrap_or(600),
                max_ttl: env::var("APP_SIGNED_URL__MAX_TTL")
                    .or_else(|_| env::var("SIGNED_URL_MAX_TTL"))
                    .unwrap_or_else(|_| "3600".to_string())
                    .parse()
                    .unwrap_or(3600),
            },
        };

        Ok(settings)
    }

    /// Get database connection URL
    pub fn database_url(&self) -> String {
        // Check for direct DATABASE_URL env var first
        if let Ok(url) = env::var("DATABASE_URL") {
            return url;
        }

        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.database.username,
            self.database.password,
            self.database.host,
            self.database.port,
            self.database.database
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_url_from_config() {
        // Clear env var to ensure we test config-based URL
        env::remove_var("DATABASE_URL");

        let settings = Settings {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
                workers: 4,
            },
            database: DatabaseConfig {
                host: "localhost".to_string(),
                port: 5432,
                username: "admin".to_string(),
                password: "password".to_string(),
                database: "testdb".to_string(),
                max_connections: 10,
                min_connections: 2,
            },
            keycloak: KeycloakConfig {
                url: "http://localhost:8080".to_string(),
                realm: "test".to_string(),
                client_id: "test-client".to_string(),
                client_secret: "secret".to_string(),
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
                allowed_origins: vec!["http://localhost:3000".to_string()],
                allowed_methods: vec!["GET".to_string(), "POST".to_string()],
                allowed_headers: vec!["Content-Type".to_string()],
                expose_headers: vec!["Content-Length".to_string()],
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
        };

        let url = settings.database_url();
        assert_eq!(url, "postgres://admin:password@localhost:5432/testdb");
    }

    #[test]
    fn test_database_url_from_env() {
        env::set_var("DATABASE_URL", "postgres://user:pass@host:5432/db");

        let settings = Settings {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
                workers: 4,
            },
            database: DatabaseConfig {
                host: "localhost".to_string(),
                port: 5432,
                username: "admin".to_string(),
                password: "password".to_string(),
                database: "testdb".to_string(),
                max_connections: 10,
                min_connections: 2,
            },
            keycloak: KeycloakConfig {
                url: "http://localhost:8080".to_string(),
                realm: "test".to_string(),
                client_id: "test-client".to_string(),
                client_secret: "secret".to_string(),
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
                allowed_origins: vec!["http://localhost:3000".to_string()],
                allowed_methods: vec!["GET".to_string(), "POST".to_string()],
                allowed_headers: vec!["Content-Type".to_string()],
                expose_headers: vec!["Content-Length".to_string()],
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
        };

        let url = settings.database_url();
        assert_eq!(url, "postgres://user:pass@host:5432/db");

        env::remove_var("DATABASE_URL");
    }
}
