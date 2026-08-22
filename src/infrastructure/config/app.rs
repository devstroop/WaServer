use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::env::EnvironmentConfig;

/// Application configuration structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub auth: AuthConfig,
    pub logging: LoggingConfig,
    pub cache: CacheConfig,
    pub cors: CorsConfig,
    pub limits: LimitsConfig,
    pub environment: EnvironmentConfig,
    #[serde(default)]
    pub swagger: SwaggerConfig,
    /// Multi-instance configuration
    #[serde(default)]
    pub instances: Option<InstancesConfig>,
}

/// Multi-instance configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InstancesConfig {
    /// Base directory for all instance data (default: ~/.was/accounts)
    pub base_directory: Option<PathBuf>,
    /// Default browser settings for new instances
    #[serde(default)]
    pub defaults: InstanceDefaultsConfig,
}

/// Default settings for new instances
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct InstanceDefaultsConfig {
    /// Default headless mode
    pub headless: Option<bool>,
    /// Default idle timeout in seconds before auto-sleep (0 = never, default 300)
    #[serde(default = "default_global_idle_timeout")]
    pub idle_timeout: u64,
}

fn default_global_idle_timeout() -> u64 {
    300
}

/// Swagger UI configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SwaggerConfig {
    /// Enable Swagger UI
    #[serde(default = "default_swagger_enabled")]
    pub enabled: bool,
    /// Swagger UI path
    #[serde(default = "default_swagger_path")]
    pub path: String,
}

fn default_swagger_enabled() -> bool {
    true
}

fn default_swagger_path() -> String {
    "/api-docs".to_string()
}

impl Default for SwaggerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            path: "/api-docs".to_string(),
        }
    }
}

/// Server configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

/// Authentication configuration
/// Static secret key for Bearer authentication
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthConfig {
    /// Static secret key for Bearer authentication
    pub secret_key: String,
}

/// Logging configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoggingConfig {
    pub level: String,
}

/// Cache configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CacheConfig {
    pub default_ttl_minutes: u64,
}

/// CORS configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CorsConfig {
    pub allow_origins: Vec<String>,
    pub allow_methods: Vec<String>,
    pub allow_headers: Vec<String>,
}

/// Rate limiting and resource limits
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LimitsConfig {
    pub max_concurrent_requests: usize,
    pub request_timeout_ms: u64,
    pub max_upload_size: usize,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 3000,
            },
            auth: AuthConfig {
                secret_key: "change-this-secret-key-in-production".to_string(),
            },
            logging: LoggingConfig {
                level: "info".to_string(),
            },
            cache: CacheConfig {
                default_ttl_minutes: 15,
            },
            cors: CorsConfig {
                allow_origins: vec!["*".to_string()],
                allow_methods: vec![
                    "GET".to_string(),
                    "POST".to_string(),
                    "PUT".to_string(),
                    "DELETE".to_string(),
                    "OPTIONS".to_string(),
                ],
                allow_headers: vec!["authorization".to_string(), "content-type".to_string()],
            },
            limits: LimitsConfig {
                max_concurrent_requests: 10,
                request_timeout_ms: 60000,
                max_upload_size: 10 * 1024 * 1024, // 10MB
            },
            environment: EnvironmentConfig::default(),
            swagger: SwaggerConfig::default(),
            instances: None,
        }
    }
}

impl AppConfig {
    /// Load configuration from file and environment variables
    pub fn load() -> Result<Self, config::ConfigError> {
        // First load environment config
        let env_config = EnvironmentConfig::from_env();

        let mut builder = config::Config::builder()
            .add_source(config::File::with_name("config/app").required(false))
            .add_source(config::Environment::with_prefix("WAS").separator("__"));

        // Try to load from current directory if config/app doesn't exist
        if std::fs::metadata("config/app.toml").is_err() {
            builder = builder.add_source(config::File::with_name("app").required(false));
        }

        let config = builder.build()?;
        let mut app_config: AppConfig = config.try_deserialize()?;

        // Override with environment config
        app_config.environment = env_config;

        Ok(app_config)
    }

    /// Validate configuration values
    pub fn validate(&self) -> Result<(), String> {
        if self.server.port == 0 {
            return Err("Server port cannot be 0".to_string());
        }

        // Validate static secret key
        if self.auth.secret_key == "change-this-secret-key-in-production" {
            return Err("Please change the default secret_key in configuration".to_string());
        }

        if self.auth.secret_key.len() < 16 {
            return Err("secret_key must be at least 16 characters long".to_string());
        }

        if self.limits.max_concurrent_requests == 0 {
            return Err("Max concurrent requests cannot be 0".to_string());
        }

        Ok(())
    }
}
