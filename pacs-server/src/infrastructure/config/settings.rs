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
    pub dcm4chee: Dcm4cheeConfig,
    pub sync: Option<SyncConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
    #[serde(default = "default_server_mode")] 
    pub mode: ServerMode,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum ServerMode {
    #[serde(rename = "full")] Full,
    #[serde(rename = "api-only")] ApiOnly,
    #[serde(rename = "sync-only")] SyncOnly,
}

fn default_server_mode() -> ServerMode { ServerMode::Full }

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
    pub admin_username: String,
    pub admin_password: String,
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

#[derive(Debug, Deserialize, Clone)]
pub struct Dcm4cheeConfig {
    pub base_url: String,
    pub qido_path: String,      // e.g., /dcm4chee-arc/aets/DCM4CHEE/rs
    pub wado_path: String,      // e.g., /dcm4chee-arc/aets/DCM4CHEE/wado
    pub aet: String,            // AE Title
    pub username: Option<String>,
    pub password: Option<String>,
    pub timeout_ms: u64,
    #[serde(default)]
    pub db: Option<Dcm4cheeDbConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SyncConfig {
    #[serde(default = "default_sync_enabled")] 
    pub enabled: bool,
    #[serde(default = "default_sync_interval")] 
    pub interval_sec: u64,
    pub default_project_id: Option<i32>,
}

fn default_sync_enabled() -> bool { true }
fn default_sync_interval() -> u64 { 30 }

#[derive(Debug, Deserialize, Clone)]
pub struct Dcm4cheeDbConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
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
                mode: match env::var("APP_SERVER__MODE").or_else(|_| env::var("SERVER_MODE")).unwrap_or_else(|_| "full".to_string()).to_lowercase().as_str() {
                    "api-only" => ServerMode::ApiOnly,
                    "sync-only" => ServerMode::SyncOnly,
                    _ => ServerMode::Full,
                },
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
                    .unwrap_or_else(|_| "dcm4che".to_string()),
                client_id: env::var("APP_KEYCLOAK__CLIENT_ID")
                    .or_else(|_| env::var("KEYCLOAK_CLIENT_ID"))
                    .unwrap_or_else(|_| "pacs-server".to_string()),
                client_secret: env::var("APP_KEYCLOAK__CLIENT_SECRET")
                    .or_else(|_| env::var("KEYCLOAK_CLIENT_SECRET"))
                    .unwrap_or_else(|_| "".to_string()),
                admin_username: env::var("APP_KEYCLOAK__ADMIN_USERNAME")
                    .or_else(|_| env::var("KEYCLOAK_ADMIN_USERNAME"))
                    .unwrap_or_else(|_| "admin".to_string()),
                admin_password: env::var("APP_KEYCLOAK__ADMIN_PASSWORD")
                    .or_else(|_| env::var("KEYCLOAK_ADMIN_PASSWORD"))
                    .unwrap_or_else(|_| "adminPassword123!".to_string()),
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
            object_storage: {
                let provider = env::var("APP_OBJECT_STORAGE__PROVIDER")
                    .or_else(|_| env::var("OBJECT_STORAGE_PROVIDER"))
                    .unwrap_or_else(|_| "s3".to_string());
                let bucket_name = env::var("APP_OBJECT_STORAGE__BUCKET_NAME")
                    .or_else(|_| env::var("OBJECT_STORAGE_BUCKET_NAME"))
                    .unwrap_or_else(|_| "pacs-masks".to_string());
                let region = env::var("APP_OBJECT_STORAGE__REGION")
                    .or_else(|_| env::var("OBJECT_STORAGE_REGION"))
                    .unwrap_or_else(|_| "us-east-1".to_string());
                let endpoint = env::var("APP_OBJECT_STORAGE__ENDPOINT")
                    .or_else(|_| env::var("OBJECT_STORAGE_ENDPOINT"))
                    .unwrap_or_else(|_| "".to_string());
                let access_key = env::var("APP_OBJECT_STORAGE__ACCESS_KEY_ID")
                    .or_else(|_| env::var("OBJECT_STORAGE_ACCESS_KEY_ID"))
                    .unwrap_or_else(|_| "".to_string());
                let secret_key = env::var("APP_OBJECT_STORAGE__SECRET_ACCESS_KEY")
                    .or_else(|_| env::var("OBJECT_STORAGE_SECRET_ACCESS_KEY"))
                    .unwrap_or_else(|_| "".to_string());

                // 디버깅: 환경 변수 로드 정보 출력
                println!("🔧 Object Storage 설정 로드:");
                println!("   Provider: {}", provider);
                println!("   Bucket: {}", bucket_name);
                println!("   Region: {}", region);
                println!("   Endpoint: {}", if endpoint.is_empty() { "None".to_string() } else { endpoint.clone() });
                println!("   Access Key: {} (길이: {})", 
                    if access_key.is_empty() { "EMPTY".to_string() } else { format!("{}...{}", &access_key[..access_key.len().min(8)], &access_key[access_key.len().saturating_sub(4)..]) },
                    access_key.len()
                );
                println!("   Secret Key: {} (길이: {})", 
                    if secret_key.is_empty() { "EMPTY".to_string() } else { format!("{}...{}", &secret_key[..secret_key.len().min(8)], &secret_key[secret_key.len().saturating_sub(4)..]) },
                    secret_key.len()
                );

                ObjectStorageConfig {
                    provider,
                    bucket_name,
                    region,
                    endpoint,
                    access_key,
                    secret_key,
                }
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
            dcm4chee: Dcm4cheeConfig {
                base_url: env::var("APP_DCM4CHEE__BASE_URL")
                    .or_else(|_| env::var("DCM4CHEE_BASE_URL"))
                    .unwrap_or_else(|_| "http://localhost:8080".to_string()),
                qido_path: env::var("APP_DCM4CHEE__QIDO_PATH")
                    .or_else(|_| env::var("DCM4CHEE_QIDO_PATH"))
                    .unwrap_or_else(|_| "/dcm4chee-arc/aets/DCM4CHEE/rs".to_string()),
                wado_path: env::var("APP_DCM4CHEE__WADO_PATH")
                    .or_else(|_| env::var("DCM4CHEE_WADO_PATH"))
                    .unwrap_or_else(|_| "/dcm4chee-arc/aets/DCM4CHEE/wado".to_string()),
                aet: env::var("APP_DCM4CHEE__AET").or_else(|_| env::var("DCM4CHEE_AET")).unwrap_or_else(|_| "DCM4CHEE".to_string()),
                username: env::var("APP_DCM4CHEE__USERNAME").ok().or_else(| | env::var("DCM4CHEE_USERNAME").ok()),
                password: env::var("APP_DCM4CHEE__PASSWORD").ok().or_else(| | env::var("DCM4CHEE_PASSWORD").ok()),
                timeout_ms: env::var("APP_DCM4CHEE__TIMEOUT_MS")
                    .or_else(|_| env::var("DCM4CHEE_TIMEOUT_MS"))
                    .unwrap_or_else(|_| "5000".to_string())
                    .parse()
                    .unwrap_or(5000),
                db: {
                    // Optional nested DB config for DCM4CHEE
                    let host = env::var("APP_DCM4CHEE__DB__HOST").or_else(|_| env::var("DCM4CHEE_DB_HOST")).ok();
                    let port = env::var("APP_DCM4CHEE__DB__PORT").or_else(|_| env::var("DCM4CHEE_DB_PORT")).ok();
                    let database = env::var("APP_DCM4CHEE__DB__DATABASE").or_else(|_| env::var("DCM4CHEE_DB_DATABASE")).ok();
                    let username = env::var("APP_DCM4CHEE__DB__USERNAME").or_else(|_| env::var("DCM4CHEE_DB_USERNAME")).ok();
                    let password = env::var("APP_DCM4CHEE__DB__PASSWORD").or_else(|_| env::var("DCM4CHEE_DB_PASSWORD")).ok();

                    if let (Some(host), Some(port), Some(database), Some(username), Some(password)) = (host, port, database, username, password) {
                        Some(Dcm4cheeDbConfig {
                            host,
                            port: port.parse().unwrap_or(5432),
                            database,
                            username,
                            password,
                        })
                    } else { None }
                },
            },
            sync: Some(SyncConfig {
                enabled: env::var("APP_SYNC__ENABLED").or_else(|_| env::var("SYNC_ENABLED")).unwrap_or_else(|_| "true".to_string()).parse().unwrap_or(true),
                interval_sec: env::var("APP_SYNC__INTERVAL_SEC").or_else(|_| env::var("SYNC_INTERVAL_SEC")).unwrap_or_else(|_| "30".to_string()).parse().unwrap_or(30),
                default_project_id: env::var("APP_SYNC__DEFAULT_PROJECT_ID").or_else(|_| env::var("SYNC_DEFAULT_PROJECT_ID")).ok().and_then(|s| s.parse::<i32>().ok()),
            }),
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

    use serial_test::serial;

    #[test]
    #[serial]
    fn test_database_url_from_config() {
        // Clear env var to ensure we test config-based URL
        env::remove_var("DATABASE_URL");

        let settings = Settings {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
                workers: 4,
                mode: ServerMode::Full,
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
                admin_username: "admin".to_string(),
                admin_password: "adminPassword123!".to_string(),
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
            dcm4chee: Dcm4cheeConfig {
                base_url: "http://localhost:8080".to_string(),
                qido_path: "/dcm4chee-arc/aets/DCM4CHEE/rs".to_string(),
                wado_path: "/dcm4chee-arc/aets/DCM4CHEE/wado".to_string(),
                aet: "DCM4CHEE".to_string(),
                username: Some("admin".to_string()),
                password: Some("adminPassword123!".to_string()),
                timeout_ms: 5000,
                db: None,
            },
            sync: Some(SyncConfig { enabled: true, interval_sec: 30, default_project_id: Some(1) }),
        };

        let url = settings.database_url();
        assert_eq!(url, "postgres://admin:password@localhost:5432/testdb");
    }

    #[test]
    #[serial]
    fn test_database_url_from_env() {
        // Ensure APP-specific override is cleared for this test
        env::remove_var("APP_DATABASE_URL");
        env::set_var("DATABASE_URL", "postgres://user:pass@host:5432/db");

        let settings = Settings {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
                workers: 4,
                mode: ServerMode::Full,
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
                admin_username: "admin".to_string(),
                admin_password: "adminPassword123!".to_string(),
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
            dcm4chee: Dcm4cheeConfig {
                base_url: "http://localhost:8080".to_string(),
                qido_path: "/dcm4chee-arc/aets/DCM4CHEE/rs".to_string(),
                wado_path: "/dcm4chee-arc/aets/DCM4CHEE/wado".to_string(),
                aet: "DCM4CHEE".to_string(),
                username: Some("admin".to_string()),
                password: Some("adminPassword123!".to_string()),
                timeout_ms: 5000,
                db: None,
            },
            sync: Some(SyncConfig { enabled: true, interval_sec: 30, default_project_id: Some(1) }),
        };

        let url = settings.database_url();
        assert_eq!(url, "postgres://user:pass@host:5432/db");

        env::remove_var("DATABASE_URL");
        env::remove_var("APP_DATABASE_URL");
    }
}
