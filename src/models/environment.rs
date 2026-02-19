// Environment Configuration
//
// Production-ready environment variable handling and configuration management

use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    pub environment: Environment,
    pub log_level: String,
    pub rust_log: String,
    pub debug_mode: bool,
    pub health_check_interval: u64,
    /// Directory for persistent data (database, media files)
    pub data_directory: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Environment {
    Development,
    Staging,
    Production,
}

impl Default for EnvironmentConfig {
    fn default() -> Self {
        Self {
            environment: Environment::Development,
            log_level: "info".to_string(),
            rust_log: "was=info".to_string(),
            debug_mode: false,
            health_check_interval: 30,
            data_directory: Some("data".to_string()),
        }
    }
}

impl EnvironmentConfig {
    pub fn from_env() -> Self {
        let mut config = Self::default();

        // Environment
        if let Ok(env_str) = env::var("ENVIRONMENT") {
            config.environment = match env_str.to_lowercase().as_str() {
                "production" | "prod" => Environment::Production,
                "staging" | "stage" => Environment::Staging,
                _ => Environment::Development,
            };
        }

        // Log level
        if let Ok(log_level) = env::var("LOG_LEVEL") {
            config.log_level = log_level;
        }

        // RUST_LOG override
        if let Ok(rust_log) = env::var("RUST_LOG") {
            config.rust_log = rust_log;
        } else {
            // Set default RUST_LOG based on environment
            config.rust_log = match config.environment {
                Environment::Production => "was=warn,error".to_string(),
                Environment::Staging => "was=info".to_string(),
                Environment::Development => "was=debug".to_string(),
            };
        }

        // Debug mode
        if let Ok(debug_str) = env::var("DEBUG") {
            config.debug_mode = debug_str.to_lowercase() == "true" || debug_str == "1";
        } else {
            config.debug_mode = matches!(config.environment, Environment::Development);
        }

        // Health check interval
        if let Ok(interval_str) = env::var("HEALTH_CHECK_INTERVAL") {
            if let Ok(interval) = interval_str.parse::<u64>() {
                config.health_check_interval = interval;
            }
        }

        // Data directory
        if let Ok(data_dir) = env::var("DATA_DIRECTORY") {
            config.data_directory = Some(data_dir);
        }

        config
    }

    pub fn is_production(&self) -> bool {
        self.environment == Environment::Production
    }

    pub fn is_staging(&self) -> bool {
        self.environment == Environment::Staging
    }

    pub fn is_development(&self) -> bool {
        self.environment == Environment::Development
    }

    pub fn get_rust_log_filter(&self) -> String {
        self.rust_log.clone()
    }

    pub fn should_show_detailed_errors(&self) -> bool {
        !self.is_production()
    }

    pub fn should_enable_request_logging(&self) -> bool {
        self.debug_mode || !self.is_production()
    }
}
