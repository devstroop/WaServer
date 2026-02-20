// Environment Configuration
//
// Production-ready environment variable handling and configuration management

use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    pub environment: Environment,
    pub log_level: String,
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

        // Log level (standard RUST_LOG)
        if let Ok(log_level) = env::var("RUST_LOG") {
            config.log_level = log_level;
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

    /// Get RUST_LOG filter string from log_level
    pub fn get_rust_log_filter(&self) -> String {
        // If already contains '=' (e.g., "was=debug"), use as-is
        if self.log_level.contains('=') {
            self.log_level.clone()
        } else {
            // Simple level like "info" -> "was=info"
            format!("was={}", self.log_level)
        }
    }

    pub fn is_debug(&self) -> bool {
        matches!(self.log_level.as_str(), "debug" | "trace") 
            || self.log_level.contains("debug") 
            || self.log_level.contains("trace")
    }

    pub fn should_show_detailed_errors(&self) -> bool {
        !self.is_production()
    }

    pub fn should_enable_request_logging(&self) -> bool {
        self.is_debug() || !self.is_production()
    }
}
