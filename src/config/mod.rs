use serde::{Deserialize, Serialize};

use environment::EnvironmentConfig;

pub mod environment;

/// Application configuration structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub browser: BrowserConfig,
    pub auth: AuthConfig,
    #[serde(default)]
    pub local_auth: LocalAuthConfig,
    pub logging: LoggingConfig,
    pub cache: CacheConfig,
    pub cors: CorsConfig,
    pub limits: LimitsConfig,
    pub environment: EnvironmentConfig,
    #[serde(default)]
    pub web: WebConfig,
    #[serde(default)]
    pub swagger: SwaggerConfig,
    #[serde(default)]
    pub mcp: McpConfig,
    #[serde(default)]
    pub webhooks: WebhookConfig,
}

/// Webhook configuration for event callbacks
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WebhookConfig {
    /// Enable webhook callbacks
    pub enabled: bool,
    /// Webhook endpoints
    #[serde(default)]
    pub endpoints: Vec<WebhookEndpointConfig>,
    /// Request timeout in milliseconds
    pub timeout_ms: u64,
    /// Number of retry attempts on failure
    pub retry_count: u32,
    /// Delay between retries in milliseconds
    pub retry_delay_ms: u64,
}

/// Individual webhook endpoint configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WebhookEndpointConfig {
    /// Webhook URL
    pub url: String,
    /// Optional HMAC secret for signature verification
    pub secret: Option<String>,
    /// Optional custom headers
    pub headers: Option<std::collections::HashMap<String, String>>,
}

impl Default for WebhookConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            endpoints: vec![],
            timeout_ms: 5000,
            retry_count: 3,
            retry_delay_ms: 1000,
        }
    }
}

/// MCP (Model Context Protocol) configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct McpConfig {
    /// Enable MCP server (requires mcp feature)
    pub enabled: bool,
    /// MCP endpoint path
    pub endpoint: String,
    /// Enable SSE transport
    pub sse_enabled: bool,
    /// SSE heartbeat interval in seconds
    pub heartbeat_interval_secs: u64,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            endpoint: "/mcp".to_string(),
            sse_enabled: true,
            heartbeat_interval_secs: 30,
        }
    }
}

/// Web UI configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WebConfig {
    /// Enable Web UI serving
    #[serde(default = "default_web_enabled")]
    pub enabled: bool,
    /// Path to the frontend build directory
    #[serde(default = "default_web_path")]
    pub path: String,
}

fn default_web_enabled() -> bool {
    true
}

fn default_web_path() -> String {
    "app/dist".to_string()
}

impl Default for WebConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            path: "app/dist".to_string(),
        }
    }
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
    "/swagger-ui".to_string()
}

impl Default for SwaggerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            path: "/swagger-ui".to_string(),
        }
    }
}

/// Server configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

/// Browser configuration for Playwright
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BrowserConfig {
    pub headless: bool,
    pub timeout_ms: u64,
    pub args: Vec<String>,
}

/// Authentication configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthConfig {
    /// Enable Bearer token authentication
    #[serde(default = "default_auth_enabled")]
    pub enabled: bool,
    /// API token for Bearer authentication
    pub api_token: String,
}

fn default_auth_enabled() -> bool {
    true
}

/// Local authentication configuration (JWT-based)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LocalAuthConfig {
    /// Enable local authentication with JWT tokens
    #[serde(default)]
    pub enabled: bool,
    /// JWT secret key for signing tokens
    #[serde(default = "default_jwt_secret")]
    pub jwt_secret: String,
    /// Access token expiry in hours
    #[serde(default = "default_token_expiry_hours")]
    pub token_expiry_hours: i64,
    /// Refresh token expiry in days
    #[serde(default = "default_refresh_token_expiry_days")]
    pub refresh_token_expiry_days: i64,
    /// Default admin username
    #[serde(default = "default_username")]
    pub default_username: String,
    /// Default admin password
    #[serde(default = "default_password")]
    pub default_password: String,
}

fn default_jwt_secret() -> String {
    "your-super-secret-jwt-key-change-this-in-production-32chars".to_string()
}

fn default_token_expiry_hours() -> i64 {
    24
}

fn default_refresh_token_expiry_days() -> i64 {
    7
}

fn default_username() -> String {
    "admin".to_string()
}

fn default_password() -> String {
    "admin123".to_string()
}

impl Default for LocalAuthConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            jwt_secret: default_jwt_secret(),
            token_expiry_hours: default_token_expiry_hours(),
            refresh_token_expiry_days: default_refresh_token_expiry_days(),
            default_username: default_username(),
            default_password: default_password(),
        }
    }
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
            browser: BrowserConfig {
                headless: false,
                timeout_ms: 30000,
                args: vec![
                    "--disable-blink-features=AutomationControlled".to_string(),
                    "--no-sandbox".to_string(),
                    "--disable-setuid-sandbox".to_string(),
                    "--disable-dev-shm-usage".to_string(),
                    "--disable-extensions".to_string(),
                    "--disable-popup-blocking".to_string(),
                    "--disable-gpu".to_string(),
                    "--disable-software-rasterizer".to_string(),
                ],
            },
            auth: AuthConfig {
                enabled: true,
                api_token: "your-secure-api-token-change-this".to_string(),
            },
            local_auth: LocalAuthConfig::default(),
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
            web: WebConfig::default(),
            swagger: SwaggerConfig::default(),
            mcp: McpConfig::default(),
            webhooks: WebhookConfig::default(),
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
            .add_source(config::Environment::with_prefix("WHATSAPP").separator("__"));

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

        // Validate auth config - either static token or local auth must be enabled
        if self.auth.enabled && !self.local_auth.enabled {
            // Static token auth mode
            if self.auth.api_token == "your-secure-api-token-change-this" {
                return Err("Please change the default API token in configuration".to_string());
            }

            if self.auth.api_token.len() < 16 {
                return Err("API token must be at least 16 characters long".to_string());
            }
        }

        // Validate local auth config
        if self.local_auth.enabled {
            if self.local_auth.jwt_secret == "your-super-secret-jwt-key-change-in-production" {
                tracing::warn!("Using default JWT secret - please change in production!");
            }

            if self.local_auth.jwt_secret.len() < 32 {
                return Err("JWT secret must be at least 32 characters long".to_string());
            }
        }

        if self.browser.timeout_ms == 0 {
            return Err("Browser timeout cannot be 0".to_string());
        }

        if self.limits.max_concurrent_requests == 0 {
            return Err("Max concurrent requests cannot be 0".to_string());
        }

        Ok(())
    }
}
