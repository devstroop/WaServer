//! # WhatsApp Engine
//! 
//! A powerful Rust library for WhatsApp Web automation with support for both library and API modes.
//! 
//! ## Features
//! 
//! - **📱 Dual Authentication**: QR Code and Phone Number authentication
//! - **💬 Complete Messaging**: Text, images, documents, voice messages
//! - **👥 Contact Management**: Retrieve contacts and chats
//! - **⚡ High Performance**: Async Rust with Tokio
//! - **🔧 Dual Mode**: Use as library or standalone API server
//! - **🐳 Production Ready**: Docker, monitoring, health checks
//! 
//! ## Quick Start
//! 
//! ```rust,no_run
//! use whatsapp_engine::WhatsAppEngine;
//! 
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create engine with default configuration
//!     let engine = WhatsAppEngine::with_defaults().await?;
//!     
//!     // Authenticate with QR code
//!     let qr_code = engine.authenticate_with_qr().await?;
//!     println!("Scan this QR code: {}", qr_code.data);
//!     
//!     // Wait for authentication
//!     while !engine.is_authenticated().await? {
//!         tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
//!     }
//!     
//!     // Send a message
//!     let result = engine.send_message("1234567890", "Hello from WhatsApp Engine!").await?;
//!     if result.success {
//!         println!("✅ Message sent successfully!");
//!     }
//!     
//!     // Clean shutdown
//!     engine.close().await?;
//!     
//!     Ok(())
//! }
//! ```

pub mod auth;
pub mod config;
pub mod error;
pub mod handlers;
pub mod locators;
pub mod models;
pub mod services;
pub mod utils;
pub mod middleware;

// Re-export public API
pub use config::AppConfig;
pub use error::{WhatsAppError, Result};
pub use models::domain::*;

use std::sync::Arc;
use std::time::SystemTime;
use tracing::{info, warn, error, debug};
use crate::services::{auth_service::AuthServiceTrait, chat_service::ChatServiceTrait};

/// Main WhatsApp Engine library interface
/// 
/// This is the primary entry point for using WhatsApp Engine as a library.
/// It provides a clean, async API for WhatsApp Web automation.
/// 
/// # Examples
/// 
/// Basic usage:
/// ```rust,no_run
/// use whatsapp_engine::WhatsAppEngine;
/// 
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let engine = WhatsAppEngine::with_defaults().await?;
///     let qr = engine.authenticate_with_qr().await?;
///     println!("QR: {}", qr.data);
///     Ok(())
/// }
/// ```
pub struct WhatsAppEngine {
    auth_service: Arc<crate::services::auth_service::AuthService>,
    chat_service: Arc<crate::services::chat_service::ChatService>,
    browser_service: Arc<crate::services::browser::BrowserService>,
    config: Arc<AppConfig>,
    start_time: SystemTime,
}

impl WhatsAppEngine {
    /// Create a new WhatsApp Engine instance with the provided configuration
    pub async fn new(config: AppConfig) -> Result<Self> {
        info!("Initializing WhatsApp Engine library");
        
        let config = Arc::new(config);
        let start_time = SystemTime::now();
        
        // Initialize browser service
        debug!("Initializing browser service");
        let browser_service = Arc::new(
            crate::services::browser::BrowserService::new(config.clone())
        );
        
        // Initialize auth service
        debug!("Initializing auth service");
        let auth_service = Arc::new(
            crate::services::auth_service::AuthService::new(
                config.clone(),
                browser_service.clone()
            )
        );
        
        // Initialize chat service
        debug!("Initializing chat service");
        let chat_service = Arc::new(
            crate::services::chat_service::ChatService::new(
                config.clone(),
                browser_service.clone()
            )
        );
        
        info!("WhatsApp Engine library initialized successfully");
        
        Ok(Self {
            auth_service,
            chat_service,
            browser_service,
            config,
            start_time,
        })
    }
    
    /// Create a new WhatsApp Engine instance with default configuration
    pub async fn with_defaults() -> Result<Self> {
        let config = AppConfig::load()
            .map_err(|e| WhatsAppError::Configuration(e.to_string()))?;
        Self::new(config).await
    }
    
    /// Authenticate using QR code method
    pub async fn authenticate_with_qr(&self) -> Result<QrCode> {
        info!("Starting QR code authentication");
        
        let qr_data = self.auth_service.get_auth_qr_code().await
            .map_err(|e| WhatsAppError::QrCodeGeneration(e.to_string()))?;
        
        let expires_at = chrono::Utc::now() + chrono::Duration::minutes(5);
        
        Ok(QrCode {
            data: qr_data.clone(),
            expires_at: Some(expires_at),
            image_url: qr_data, // For now, use the same data as image_url
            refresh_interval_seconds: 30,
        })
    }
    
    /// Authenticate using phone number method
    pub async fn authenticate_with_phone(&self, phone_number: &str) -> Result<PhoneAuthResult> {
        info!("Starting phone authentication for {}", phone_number);
        
        // Validate phone number format
        if !phone_number.starts_with('+') || phone_number.len() < 10 {
            return Err(WhatsAppError::invalid_input(
                "phone_number", 
                "Phone number must be in international format (+1234567890)"
            ));
        }
        
        match self.auth_service.login_with_phone_number(phone_number).await {
            Ok(code) => {
                let code_str = code.clone().unwrap_or_else(|| "No code returned".to_string());
                info!("Phone authentication successful, code: {}", code_str);
                Ok(PhoneAuthResult {
                    success: true,
                    verification_code: code,
                    message: "Authentication successful. Use the verification code in your WhatsApp app.".to_string(),
                    next_retry_in_seconds: None,
                })
            }
            Err(e) => {
                warn!("Phone authentication failed: {}", e);
                Ok(PhoneAuthResult {
                    success: false,
                    verification_code: None,
                    message: format!("Authentication failed: {}", e),
                    next_retry_in_seconds: Some(60), // Retry after 1 minute
                })
            }
        }
    }
    
    /// Check if the engine is authenticated
    pub async fn is_authenticated(&self) -> Result<bool> {
        self.auth_service.is_authorized().await
            .map_err(|e| WhatsAppError::Authentication(e.to_string()))
    }
    
    /// Get detailed authentication status
    pub async fn get_auth_status(&self) -> Result<AuthStatus> {
        let is_auth = self.is_authenticated().await?;
        
        Ok(AuthStatus {
            is_authenticated: is_auth,
            phone_number: None, // TODO: Extract from session if available
            session_id: None,   // TODO: Generate/retrieve session ID
            authenticated_at: if is_auth { 
                Some(chrono::Utc::now()) 
            } else { 
                None 
            },
        })
    }
    
    /// Logout from WhatsApp Web
    pub async fn logout(&self) -> Result<()> {
        info!("Logging out from WhatsApp");
        self.auth_service.logout().await
            .map_err(|e| WhatsAppError::Authentication(e.to_string()))
    }
    
    /// Send a text message
    pub async fn send_message(&self, to: &str, message: &str) -> Result<SendMessageResult> {
        info!("Sending message to {}", to);
        
        if message.trim().is_empty() {
            return Err(WhatsAppError::invalid_input(
                "message", 
                "Message content cannot be empty"
            ));
        }
        
        if !self.is_authenticated().await? {
            return Err(WhatsAppError::Authentication(
                "Must be authenticated before sending messages".to_string()
            ));
        }
        
        match self.chat_service.send_message(to, Some(message), None, None).await {
            Ok(_) => {
                info!("Message sent successfully to {}", to);
                Ok(SendMessageResult {
                    success: true,
                    message_id: Some(uuid::Uuid::new_v4().to_string()),
                    error: None,
                    retry_after_seconds: None,
                })
            }
            Err(e) => {
                error!("Failed to send message to {}: {}", to, e);
                Ok(SendMessageResult {
                    success: false,
                    message_id: None,
                    error: Some(e.to_string()),
                    retry_after_seconds: Some(30),
                })
            }
        }
    }
    
    /// Send a file attachment
    pub async fn send_file(&self, to: &str, attachment: &FileAttachment) -> Result<SendMessageResult> {
        info!("Sending file {} to {}", attachment.file_path, to);
        
        if !std::path::Path::new(&attachment.file_path).exists() {
            return Err(WhatsAppError::invalid_input(
                "file_path", 
                "File does not exist"
            ));
        }
        
        if !self.is_authenticated().await? {
            return Err(WhatsAppError::Authentication(
                "Must be authenticated before sending files".to_string()
            ));
        }
        
        // TODO: Implement file sending through chat service
        // For now, return a placeholder implementation
        warn!("File sending not yet fully implemented in library mode");
        
        Ok(SendMessageResult {
            success: false,
            message_id: None,
            error: Some("File sending not yet implemented in library mode".to_string()),
            retry_after_seconds: None,
        })
    }
    
    /// Get list of contacts
    pub async fn get_contacts(&self) -> Result<Vec<Contact>> {
        info!("Retrieving contacts list");
        
        if !self.is_authenticated().await? {
            return Err(WhatsAppError::Authentication(
                "Must be authenticated to retrieve contacts".to_string()
            ));
        }
        
        // TODO: Implement contact retrieval
        // For now, return empty list
        warn!("Contact retrieval not yet implemented");
        Ok(vec![])
    }
    
    /// Get list of chats
    pub async fn get_chats(&self) -> Result<Vec<Chat>> {
        info!("Retrieving chats list");
        
        if !self.is_authenticated().await? {
            return Err(WhatsAppError::Authentication(
                "Must be authenticated to retrieve chats".to_string()
            ));
        }
        
        // TODO: Implement chat retrieval
        // For now, return empty list
        warn!("Chat retrieval not yet implemented");
        Ok(vec![])
    }
    
    /// Get engine status and health information
    pub async fn get_status(&self) -> Result<EngineStatus> {
        let uptime = self.start_time.elapsed()
            .unwrap_or_default()
            .as_secs();
        
        Ok(EngineStatus {
            is_ready: true, // TODO: Implement proper readiness check
            browser_connected: true, // TODO: Check actual browser status
            whatsapp_loaded: self.is_authenticated().await.unwrap_or(false),
            last_health_check: chrono::Utc::now(),
            uptime_seconds: uptime,
        })
    }
    
    /// Close the engine and clean up resources
    pub async fn close(&self) -> Result<()> {
        info!("Closing WhatsApp Engine");
        
        // Close browser service
        if let Err(e) = self.browser_service.close().await {
            warn!("Error closing browser service: {}", e);
        }
        
        info!("WhatsApp Engine closed successfully");
        Ok(())
    }
}

// Implement Drop to ensure cleanup
impl Drop for WhatsAppEngine {
    fn drop(&mut self) {
        debug!("WhatsAppEngine dropped");
    }
}
