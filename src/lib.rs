//! # WhatsApp Engine Rust Library
//! 
//! A powerful, production-ready Rust library for WhatsApp Web automation with comprehensive
//! authentication, messaging, and browser management capabilities.
//! 
//! ## Overview
//! 
//! WhatsApp Engine provides a clean, async API for automating WhatsApp Web operations.
//! It can be used as a standalone library in your Rust applications or as a REST API server.
//! 
//! ## Key Features
//! 
//! - **� Dual Authentication**: QR Code and Phone Number authentication methods
//! - **💬 Complete Messaging API**: Send text, files, and multimedia messages
//! - **👥 Contact & Chat Management**: Retrieve contacts and chat information
//! - **⚡ High Performance**: Built with async Rust and Tokio for maximum throughput
//! - **�️ Robust Error Handling**: Comprehensive error types with retry guidance
//! - **🔧 Flexible Configuration**: Environment variables, files, or programmatic setup
//! - **🐳 Production Ready**: Docker support, health checks, and monitoring
//! - **🎯 Dual Mode**: Use as library or standalone API server
//! 
//! ## Quick Start
//! 
//! Add to your `Cargo.toml`:
//! 
//! ```toml
//! [dependencies]
//! whatsapp-engine = "0.1.0"
//! tokio = { version = "1.0", features = ["full"] }
//! ```
//! 
//! Basic usage example:
//! 
//! ```rust,no_run
//! use whatsapp_engine::{WhatsAppEngine, Result};
//! use tokio::time::{sleep, Duration};
//! 
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     // Initialize with default configuration
//!     let engine = WhatsAppEngine::with_defaults().await?;
//!     
//!     // Authenticate using QR code method
//!     if !engine.is_authenticated().await? {
//!         let qr_code = engine.authenticate_with_qr().await?;
//!         println!("Scan this QR code: {}", qr_code.data);
//!         
//!         // Wait for authentication completion
//!         while !engine.is_authenticated().await? {
//!             sleep(Duration::from_secs(2)).await;
//!         }
//!         println!("✅ Authenticated successfully!");
//!     }
//!     
//!     // Send a message
//!     let result = engine.send_message("1234567890", "Hello from WhatsApp Engine!").await?;
//!     if result.success {
//!         println!("✅ Message sent successfully!");
//!         if let Some(message_id) = result.message_id {
//!             println!("📝 Message ID: {}", message_id);
//!         }
//!     } else {
//!         println!("❌ Failed to send message: {:?}", result.error);
//!     }
//!     
//!     // Proper cleanup
//!     engine.close().await?;
//!     
//!     Ok(())
//! }
//! ```
//! 
//! ## Authentication Methods
//! 
//! ### QR Code Authentication
//! 
//! Best for interactive setups and development:
//! 
//! ```rust,no_run
//! # use whatsapp_engine::{WhatsAppEngine, Result};
//! # async fn example() -> Result<()> {
//! let engine = WhatsAppEngine::with_defaults().await?;
//! let qr = engine.authenticate_with_qr().await?;
//! 
//! println!("QR Code: {}", qr.data);
//! println!("Expires at: {}", qr.expires_at.unwrap_or_default());
//! 
//! // Display QR code and wait for scan...
//! # Ok(())
//! # }
//! ```
//! 
//! ### Phone Number Authentication
//! 
//! Best for automated deployments:
//! 
//! ```rust,no_run
//! # use whatsapp_engine::{WhatsAppEngine, Result};
//! # async fn example() -> Result<()> {
//! let engine = WhatsAppEngine::with_defaults().await?;
//! let result = engine.authenticate_with_phone("+1234567890").await?;
//! 
//! if result.success {
//!     if let Some(code) = result.verification_code {
//!         println!("Enter this code in WhatsApp: {}", code);
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//! 
//! ## Configuration
//! 
//! ### Using Default Configuration
//! 
//! The engine loads configuration from environment variables and `config/app.toml`:
//! 
//! ```rust,no_run
//! # use whatsapp_engine::{WhatsAppEngine, Result};
//! # async fn example() -> Result<()> {
//! let engine = WhatsAppEngine::with_defaults().await?;
//! # Ok(())
//! # }
//! ```
//! 
//! ### Custom Configuration
//! 
//! ```rust,no_run
//! # use whatsapp_engine::{WhatsAppEngine, AppConfig, BrowserConfig, Result};
//! # async fn example() -> Result<()> {
//! let config = AppConfig {
//!     browser: BrowserConfig {
//!         headless: true,
//!         timeout_ms: 30000,
//!         args: vec!["--no-sandbox".to_string()],
//!     },
//!     // ... other configuration fields
//!     # ..Default::default()
//! };
//! 
//! let engine = WhatsAppEngine::new(config).await?;
//! # Ok(())
//! # }
//! ```
//! 
//! ## Error Handling
//! 
//! The library provides rich error types with retry guidance:
//! 
//! ```rust,no_run
//! # use whatsapp_engine::{WhatsAppEngine, WhatsAppError, Result};
//! # async fn example() -> Result<()> {
//! let engine = WhatsAppEngine::with_defaults().await?;
//! 
//! match engine.send_message("invalid", "test").await {
//!     Ok(result) => println!("Success: {:?}", result),
//!     Err(WhatsAppError::InvalidInput { field, reason }) => {
//!         println!("Invalid {}: {}", field, reason);
//!     }
//!     Err(WhatsAppError::Authentication(msg)) => {
//!         println!("Auth required: {}", msg);
//!     }
//!     Err(e) if e.is_retryable() => {
//!         println!("Retryable error: {}", e);
//!         if let Some(delay) = e.retry_delay_seconds() {
//!             println!("Retry after {}s", delay);
//!         }
//!     }
//!     Err(e) => println!("Error: {}", e),
//! }
//! # Ok(())
//! # }
//! ```
//! 
//! ## Examples
//! 
//! See the [`examples/`](https://github.com/your-org/whatsapp-engine-rust/tree/main/examples) 
//! directory for comprehensive usage examples:
//! 
//! - [`basic_usage.rs`](https://github.com/your-org/whatsapp-engine-rust/blob/main/examples/basic_usage.rs) - Complete library usage example
//! - [`custom_server.rs`](https://github.com/your-org/whatsapp-engine-rust/blob/main/examples/custom_server.rs) - Custom API server setup
//! 
//! For detailed developer documentation, see [`docs/DEVELOPER_GUIDE.md`](https://github.com/your-org/whatsapp-engine-rust/blob/main/docs/DEVELOPER_GUIDE.md).

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

/// The main WhatsApp Engine library interface.
/// 
/// `WhatsAppEngine` is the primary entry point for using WhatsApp Engine as a library.
/// It provides a clean, async API for WhatsApp Web automation with comprehensive
/// authentication, messaging, and resource management capabilities.
/// 
/// ## Key Capabilities
/// 
/// - **Authentication**: QR code and phone number authentication methods
/// - **Messaging**: Send text messages and file attachments
/// - **Contact Management**: Retrieve contacts and chat information
/// - **Session Management**: Persistent authentication across restarts
/// - **Resource Management**: Automatic browser lifecycle management
/// - **Error Handling**: Rich error types with retry guidance
/// 
/// ## Lifecycle
/// 
/// 1. **Creation**: Use [`WhatsAppEngine::new`] or [`WhatsAppEngine::with_defaults`]
/// 2. **Authentication**: Call [`authenticate_with_qr`] or [`authenticate_with_phone`]
/// 3. **Operations**: Send messages, retrieve data, etc.
/// 4. **Cleanup**: Always call [`close`] for proper resource cleanup
/// 
/// ## Thread Safety
/// 
/// `WhatsAppEngine` is designed to be used from a single async context, but the underlying
/// services are thread-safe and can be shared across tasks when needed.
/// 
/// ## Examples
/// 
/// ### Basic Usage
/// 
/// ```rust,no_run
/// use whatsapp_engine::{WhatsAppEngine, Result};
/// 
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let engine = WhatsAppEngine::with_defaults().await?;
///     
///     // Authenticate if needed
///     if !engine.is_authenticated().await? {
///         let qr = engine.authenticate_with_qr().await?;
///         println!("Scan QR: {}", qr.data);
///         
///         // Wait for authentication...
///         while !engine.is_authenticated().await? {
///             tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
///         }
///     }
///     
///     // Send message
///     let result = engine.send_message("1234567890", "Hello!").await?;
///     println!("Message sent: {}", result.success);
///     
///     engine.close().await?;
///     Ok(())
/// }
/// ```
/// 
/// ### Custom Configuration
/// 
/// ```rust,no_run
/// use whatsapp_engine::{WhatsAppEngine, AppConfig, BrowserConfig, Result};
/// 
/// async fn create_custom_engine() -> Result<WhatsAppEngine> {
///     let config = AppConfig {
///         browser: BrowserConfig {
///             headless: false,  // Show browser for debugging
///             timeout_ms: 60000,
///             args: vec!["--no-sandbox".to_string()],
///         },
///         // ... other config fields
///         ..Default::default()
///     };
///     
///     WhatsAppEngine::new(config).await
/// }
/// ```
/// 
/// ### Error Handling
/// 
/// ```rust,no_run
/// use whatsapp_engine::{WhatsAppEngine, WhatsAppError, Result};
/// 
/// async fn robust_send(engine: &WhatsAppEngine, phone: &str, msg: &str) -> Result<bool> {
///     match engine.send_message(phone, msg).await {
///         Ok(result) => Ok(result.success),
///         Err(WhatsAppError::Authentication(_)) => {
///             // Need to authenticate first
///             Ok(false)
///         }
///         Err(e) if e.is_retryable() => {
///             // Could retry this operation
///             println!("Retryable error: {}", e);
///             Ok(false)
///         }
///         Err(e) => Err(e), // Propagate other errors
///     }
/// }
/// ```
/// 
/// [`authenticate_with_qr`]: WhatsAppEngine::authenticate_with_qr
/// [`authenticate_with_phone`]: WhatsAppEngine::authenticate_with_phone
/// [`close`]: WhatsAppEngine::close
pub struct WhatsAppEngine {
    auth_service: Arc<crate::services::auth_service::AuthService>,
    chat_service: Arc<crate::services::chat_service::ChatService>,
    browser_service: Arc<crate::services::browser::BrowserService>,
    config: Arc<AppConfig>,
    start_time: SystemTime,
}

impl WhatsAppEngine {
    /// Creates a new WhatsApp Engine instance with the provided configuration.
    /// 
    /// This is the primary constructor for creating a `WhatsAppEngine` with custom settings.
    /// Use [`WhatsAppEngine::with_defaults`] for quick setup with default configuration.
    /// 
    /// # Arguments
    /// 
    /// * `config` - Application configuration containing browser, authentication, and other settings
    /// 
    /// # Returns
    /// 
    /// A `Result` containing the initialized engine or a configuration/initialization error.
    /// 
    /// # Errors
    /// 
    /// This method can fail with:
    /// - [`WhatsAppError::Configuration`] - Invalid configuration values
    /// - [`WhatsAppError::BrowserInit`] - Browser service initialization failed
    /// - [`WhatsAppError::Internal`] - Other initialization errors
    /// 
    /// # Examples
    /// 
    /// ```rust,no_run
    /// use whatsapp_engine::{WhatsAppEngine, AppConfig, BrowserConfig, Result};
    /// 
    /// #[tokio::main]
    /// async fn main() -> Result<()> {
    ///     let config = AppConfig {
    ///         browser: BrowserConfig {
    ///             headless: true,
    ///             timeout_ms: 30000,
    ///             args: vec!["--no-sandbox".to_string()],
    ///         },
    ///         // ... other configuration
    ///         ..Default::default()
    ///     };
    ///     
    ///     let engine = WhatsAppEngine::new(config).await?;
    ///     // Use engine...
    ///     engine.close().await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
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
    
    /// Creates a new WhatsApp Engine instance with default configuration.
    /// 
    /// This convenience method loads configuration from environment variables and
    /// `config/app.toml` file, falling back to sensible defaults. This is the
    /// recommended way to create an engine for most use cases.
    /// 
    /// Configuration is loaded in this order (highest priority first):
    /// 1. Environment variables (e.g., `BROWSER_HEADLESS=true`)
    /// 2. Configuration file (`config/app.toml`)
    /// 3. Default values
    /// 
    /// # Returns
    /// 
    /// A `Result` containing the initialized engine with default configuration.
    /// 
    /// # Errors
    /// 
    /// This method can fail with:
    /// - [`WhatsAppError::Configuration`] - Configuration loading failed
    /// - [`WhatsAppError::BrowserInit`] - Browser initialization failed
    /// 
    /// # Examples
    /// 
    /// ```rust,no_run
    /// use whatsapp_engine::{WhatsAppEngine, Result};
    /// 
    /// #[tokio::main]
    /// async fn main() -> Result<()> {
    ///     // Simple initialization with defaults
    ///     let engine = WhatsAppEngine::with_defaults().await?;
    ///     
    ///     println!("Engine initialized successfully!");
    ///     
    ///     // Always clean up
    ///     engine.close().await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    /// 
    /// # Environment Variables
    /// 
    /// The following environment variables are recognized:
    /// 
    /// - `BROWSER_HEADLESS` - Run browser in headless mode (default: true)
    /// - `BROWSER_TIMEOUT_MS` - Browser operation timeout (default: 30000)
    /// - `SERVER_PORT` - API server port (default: 3000)
    /// - `LOGGING_LEVEL` - Log level (default: "info")
    /// 
    /// # Configuration File
    /// 
    /// Create `config/app.toml` in your project root:
    /// 
    /// ```toml
    /// [browser]
    /// headless = true
    /// timeout_ms = 30000
    /// 
    /// [logging]
    /// level = "info"
    /// ```
    pub async fn with_defaults() -> Result<Self> {
        let config = AppConfig::load()
            .map_err(|e| WhatsAppError::Configuration(e.to_string()))?;
        Self::new(config).await
    }
    
    /// Initiates QR code authentication for WhatsApp Web.
    /// 
    /// This method generates a QR code that can be scanned with the WhatsApp mobile app
    /// to authenticate the session. QR code authentication is recommended for:
    /// - Interactive setups and development
    /// - First-time authentication
    /// - When phone number authentication is not available
    /// 
    /// # Returns
    /// 
    /// A `Result` containing a [`QrCode`] with the following information:
    /// - `data`: Base64-encoded PNG image of the QR code
    /// - `expires_at`: When the QR code expires (typically 5 minutes)
    /// - `image_url`: URL or data URI for displaying the QR code
    /// - `refresh_interval_seconds`: How often to check for authentication
    /// 
    /// # Errors
    /// 
    /// This method can fail with:
    /// - [`WhatsAppError::QrCodeGeneration`] - Failed to generate QR code
    /// - [`WhatsAppError::BrowserConnection`] - Browser not connected
    /// - [`WhatsAppError::ServiceNotReady`] - Auth service not initialized
    /// 
    /// # Authentication Flow
    /// 
    /// 1. Call this method to get a QR code
    /// 2. Display the QR code to the user
    /// 3. User scans with WhatsApp mobile app
    /// 4. Poll [`is_authenticated`] until `true`
    /// 5. Begin using other engine methods
    /// 
    /// # Examples
    /// 
    /// ## Basic QR Authentication
    /// 
    /// ```rust,no_run
    /// use whatsapp_engine::{WhatsAppEngine, Result};
    /// use tokio::time::{sleep, Duration};
    /// 
    /// async fn qr_auth_example() -> Result<()> {
    ///     let engine = WhatsAppEngine::with_defaults().await?;
    ///     
    ///     // Generate QR code
    ///     let qr = engine.authenticate_with_qr().await?;
    ///     println!("📱 Scan this QR code with WhatsApp:");
    ///     println!("   {}", qr.data);
    ///     
    ///     if let Some(expires_at) = qr.expires_at {
    ///         println!("⏰ Expires at: {}", expires_at);
    ///     }
    ///     
    ///     // Wait for authentication
    ///     println!("⏳ Waiting for scan...");
    ///     while !engine.is_authenticated().await? {
    ///         sleep(Duration::from_secs(2)).await;
    ///         print!(".");
    ///     }
    ///     
    ///     println!("\n✅ Authentication successful!");
    ///     
    ///     engine.close().await?;
    ///     Ok(())
    /// }
    /// ```
    /// 
    /// ## With Timeout Handling
    /// 
    /// ```rust,no_run
    /// use whatsapp_engine::{WhatsAppEngine, Result, WhatsAppError};
    /// use tokio::time::{sleep, Duration, timeout};
    /// 
    /// async fn qr_auth_with_timeout() -> Result<()> {
    ///     let engine = WhatsAppEngine::with_defaults().await?;
    ///     let qr = engine.authenticate_with_qr().await?;
    ///     
    ///     println!("Scan QR code: {}", qr.data);
    ///     
    ///     // Wait up to 5 minutes for authentication
    ///     let auth_result = timeout(Duration::from_secs(300), async {
    ///         while !engine.is_authenticated().await? {
    ///             sleep(Duration::from_secs(2)).await;
    ///         }
    ///         Ok::<(), WhatsAppError>(())
    ///     }).await;
    ///     
    ///     match auth_result {
    ///         Ok(_) => println!("✅ Authenticated successfully!"),
    ///         Err(_) => println!("⏰ Authentication timed out"),
    ///     }
    ///     
    ///     engine.close().await?;
    ///     Ok(())
    /// }
    /// ```
    /// 
    /// # Note
    /// 
    /// QR codes typically expire after 5 minutes. If authentication fails due to
    /// expiration, generate a new QR code by calling this method again.
    /// 
    /// [`is_authenticated`]: WhatsAppEngine::is_authenticated
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
