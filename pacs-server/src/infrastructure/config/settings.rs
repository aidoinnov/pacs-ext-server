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

impl Settings {
    /// Load settings with environment variable priority
    /// Priority (highest to lowest):
    /// 1. Environment variables (with APP_ prefix)
    /// 2. .env file
    /// 3. config/{environment}.toml (default: development)
    /// 4. config/default.toml
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
        };

        let url = settings.database_url();
        assert_eq!(url, "postgres://user:pass@host:5432/db");

        env::remove_var("DATABASE_URL");
    }
}
