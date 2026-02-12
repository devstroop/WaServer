use serde::{Deserialize, Serialize};

use environment::EnvironmentConfig;

pub mod environment;

/// Application configuration structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub browser: BrowserConfig,
    pub auth: AuthConfig,
    pub logging: LoggingConfig,
    pub cache: CacheConfig,
    pub cors: CorsConfig,
    pub limits: LimitsConfig,
    pub environment: EnvironmentConfig,
    #[serde(default)]
    pub mcp: McpConfig,
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
    pub api_token: String,
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
                api_token: "your-secure-api-token-change-this".to_string(),
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
            mcp: McpConfig::default(),
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
        if let Err(_) = std::fs::metadata("config/app.toml") {
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

        if self.auth.api_token == "your-secure-api-token-change-this" {
            return Err("Please change the default API token in configuration".to_string());
        }

        if self.auth.api_token.len() < 16 {
            return Err("API token must be at least 16 characters long".to_string());
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
