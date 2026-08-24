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
    #[serde(default)]
    pub storage: StorageConfig,
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
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    3000
}

/// Authentication configuration
///
/// `secret_key` is **opt-in**: when set (≥16 chars), the static-key superadmin
/// Bearer path is enabled; when absent/empty, that auth method is disabled and
/// access works via user access tokens only. There is no default key.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthConfig {
    /// Static secret key for Bearer authentication (optional)
    #[serde(default)]
    pub secret_key: Option<String>,
    /// Web-session lifetime in hours (default 168 = 7 days)
    #[serde(default = "default_session_ttl")]
    pub session_ttl_hours: u64,
    /// Brute-force protection for auth endpoints (#44)
    #[serde(default)]
    pub rate_limits: AuthRateLimits,
}

/// Auth endpoint throttling — sliding window of failures per key
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct AuthRateLimits {
    /// Failures allowed within the window before blocking (0 = disabled)
    pub max_failures: Option<u32>,
    /// Window length in minutes (also drives Retry-After)
    pub window_minutes: Option<u64>,
}

impl AuthRateLimits {
    pub fn max_failures(&self) -> u32 {
        self.max_failures.unwrap_or(5)
    }
    pub fn window_minutes(&self) -> u64 {
        self.window_minutes.unwrap_or(15)
    }
}

fn default_session_ttl() -> u64 {
    168
}

impl AuthConfig {
    /// Normalized key — trims and treats empty as unset
    pub fn effective_secret_key(&self) -> Option<&str> {
        match &self.secret_key {
            Some(k) if !k.trim().is_empty() => Some(k.trim()),
            _ => None,
        }
    }
}

/// Logging configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
}

fn default_log_level() -> String {
    "info".to_string()
}

/// Cache configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CacheConfig {
    #[serde(default = "default_cache_ttl")]
    pub default_ttl_minutes: u64,
}

fn default_cache_ttl() -> u64 {
    15
}

/// Storage/maintenance configuration
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct StorageConfig {
    /// Staged uploads older than this are purged hourly (#46)
    pub staging_ttl_hours: Option<u64>,
}

impl StorageConfig {
    pub fn effective_staging_ttl_hours(&self) -> u64 {
        self.staging_ttl_hours.unwrap_or(24)
    }
}

/// CORS configuration
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct CorsConfig {
    pub allow_origins: Vec<String>,
    pub allow_methods: Vec<String>,
    pub allow_headers: Vec<String>,
}

/// Rate limiting and resource limits
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LimitsConfig {
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent_requests: usize,
    #[serde(default = "default_request_timeout_ms")]
    pub request_timeout_ms: u64,
    #[serde(default = "default_max_upload_size")]
    pub max_upload_size: usize,
}

fn default_max_concurrent() -> usize {
    100
}

fn default_request_timeout_ms() -> u64 {
    30_000
}

fn default_max_upload_size() -> usize {
    10 * 1024 * 1024
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 3000,
            },
            auth: AuthConfig {
                secret_key: None,
                session_ttl_hours: 168,
                rate_limits: AuthRateLimits::default(),
            },
            logging: LoggingConfig {
                level: "info".to_string(),
            },
            cache: CacheConfig {
                default_ttl_minutes: 15,
            },
            storage: StorageConfig::default(),
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
    /// Secret key is opt-in: when set it must be strong (≥16 chars); when
    /// absent the static-key auth path is simply disabled.
    pub fn validate(&self) -> Result<(), String> {
        if self.server.port == 0 {
            return Err("Server port cannot be 0".to_string());
        }

        if let Some(secret) = self.auth.effective_secret_key() {
            crate::application::auth::SecretValidator::validate(secret)?;
        }

        if self.limits.max_concurrent_requests == 0 {
            return Err("Max concurrent requests cannot be 0".to_string());
        }

        Ok(())
    }
}
