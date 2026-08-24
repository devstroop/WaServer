//! WAS - WhatsApp Server Core
//!
//! The main library interface for WhatsApp Web automation.

use crate::{
    config::AppConfig,
    error::{Result, WhatsAppError},
    models::domain::*,
    services::{AuthService, AuthServiceTrait, ChatService, ChatServiceTrait},
};
use std::sync::Arc;
use std::time::SystemTime;
use tracing::{debug, error, info, warn};

use super::driver::{BrowserService, BrowserServiceConfig, DEFAULT_BROWSER_ARGS};
use super::session::SessionManager;

/// The main WAS (WhatsApp Server) library interface.
///
/// Provides a clean, async API for WhatsApp Web automation with comprehensive
/// authentication, messaging, and resource management capabilities.
pub struct WhatsAppEngine {
    auth_service: Arc<AuthService>,
    chat_service: Arc<ChatService>,
    browser_service: Arc<BrowserService>,
    session_manager: Arc<tokio::sync::Mutex<SessionManager>>,
    #[allow(dead_code)]
    config: Arc<AppConfig>,
    start_time: SystemTime,
}

impl WhatsAppEngine {
    /// Creates a new WAS instance with the provided configuration.
    pub async fn new(config: AppConfig) -> Result<Self> {
        info!("Initializing WAS (WhatsApp Server)");

        let config = Arc::new(config);
        let start_time = SystemTime::now();

        // Initialize browser service
        debug!("Initializing browser service");
        let base_dir = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| "/tmp".to_string());
        let browser_config = BrowserServiceConfig {
            user_data_dir: std::path::PathBuf::from(format!("{}/was/chrome-profile", base_dir)),
            headless: true,
            timeout_ms: 30000,
            args: DEFAULT_BROWSER_ARGS.iter().map(|s| s.to_string()).collect(),
        };
        let browser_service = Arc::new(BrowserService::new(browser_config));

        // Initialize auth service
        debug!("Initializing auth service");
        let auth_service = Arc::new(AuthService::new(config.clone(), browser_service.clone()));

        // Initialize chat service
        debug!("Initializing chat service");
        let chat_service = Arc::new(ChatService::new(config.clone(), browser_service.clone()));

        // Initialize session manager
        debug!("Initializing session manager");
        let session_dir = std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .join("sessions");
        let session_manager = Arc::new(tokio::sync::Mutex::new(SessionManager::new(session_dir)));

        info!("WAS initialized successfully");

        Ok(Self {
            auth_service,
            chat_service,
            browser_service,
            session_manager,
            config,
            start_time,
        })
    }

    /// Creates a new WAS instance with default configuration.
    pub async fn with_defaults() -> Result<Self> {
        let config = AppConfig::load().map_err(|e| WhatsAppError::Configuration(e.to_string()))?;
        Self::new(config).await
    }

    /// Initiates QR code authentication.
    pub async fn authenticate_with_qr(&self) -> Result<QrCode> {
        info!("Starting QR code authentication");

        let qr_data = self
            .auth_service
            .get_auth_qr_code()
            .await
            .map_err(|e| WhatsAppError::QrCodeGeneration(e.to_string()))?;

        let expires_at = chrono::Utc::now() + chrono::Duration::minutes(5);

        Ok(QrCode {
            data: qr_data.clone(),
            expires_at: Some(expires_at),
            image_url: qr_data,
            refresh_interval_seconds: 30,
        })
    }

    /// Authenticate using phone number.
    pub async fn authenticate_with_phone(&self, phone_number: &str) -> Result<PhoneAuthResult> {
        info!("Starting phone authentication for {}", phone_number);

        if !phone_number.starts_with('+') || phone_number.len() < 10 {
            return Err(WhatsAppError::invalid_input(
                "phone_number",
                "Phone number must be in international format (+1234567890)",
            ));
        }

        match self
            .auth_service
            .login_with_phone_number(phone_number)
            .await
        {
            Ok(code) => {
                let code_str = code.clone().unwrap_or_else(|| "No code".to_string());
                info!("Phone auth successful, code: {}", code_str);
                Ok(PhoneAuthResult {
                    success: true,
                    verification_code: code,
                    message: "Use the verification code in your WhatsApp app.".to_string(),
                    next_retry_in_seconds: None,
                })
            }
            Err(e) => {
                warn!("Phone authentication failed: {}", e);
                Ok(PhoneAuthResult {
                    success: false,
                    verification_code: None,
                    message: format!("Authentication failed: {}", e),
                    next_retry_in_seconds: Some(60),
                })
            }
        }
    }

    /// Check if authenticated.
    pub async fn is_authenticated(&self) -> Result<bool> {
        self.auth_service
            .is_authorized()
            .await
            .map_err(|e| WhatsAppError::Authentication(e.to_string()))
    }

    /// Get detailed authentication status.
    pub async fn get_auth_status(&self) -> Result<AuthStatus> {
        let is_auth = self.is_authenticated().await?;
        let session_guard = self.session_manager.lock().await;
        let current_session = session_guard.get_current_session();

        Ok(AuthStatus {
            is_authenticated: is_auth,
            phone_number: current_session.and_then(|s| s.phone_number.clone()),
            session_id: current_session.map(|s| s.session_id.clone()),
            authenticated_at: if is_auth {
                current_session
                    .map(|s| s.authenticated_at)
                    .or_else(|| Some(chrono::Utc::now()))
            } else {
                None
            },
        })
    }

    /// Logout from WhatsApp Web.
    pub async fn logout(&self) -> Result<()> {
        info!("Logging out");
        self.auth_service
            .logout()
            .await
            .map_err(|e| WhatsAppError::Authentication(e.to_string()))
    }

    /// Send a text message.
    pub async fn send_message(&self, to: &str, message: &str) -> Result<SendMessageResult> {
        info!("Sending message to {}", to);

        if message.trim().is_empty() {
            return Err(WhatsAppError::invalid_input(
                "message",
                "Message cannot be empty",
            ));
        }

        if !self.is_authenticated().await? {
            return Err(WhatsAppError::Authentication(
                "Must authenticate first".to_string(),
            ));
        }

        match self
            .chat_service
            .send_message(to, Some(message), None, None)
            .await
        {
            Ok(_) => {
                info!("Message sent to {}", to);
                Ok(SendMessageResult {
                    success: true,
                    message_id: Some(uuid::Uuid::new_v4().to_string()),
                    error: None,
                    retry_after_seconds: None,
                })
            }
            Err(e) => {
                error!("Failed to send to {}: {}", to, e);
                Ok(SendMessageResult {
                    success: false,
                    message_id: None,
                    error: Some(e.to_string()),
                    retry_after_seconds: Some(30),
                })
            }
        }
    }

    /// Send a file attachment.
    pub async fn send_file(
        &self,
        to: &str,
        attachment: &FileAttachment,
    ) -> Result<SendMessageResult> {
        info!("Sending file to {}", to);

        if !std::path::Path::new(&attachment.file_path).exists() {
            return Err(WhatsAppError::invalid_input("file_path", "File not found"));
        }

        if !self.is_authenticated().await? {
            return Err(WhatsAppError::Authentication(
                "Must authenticate first".to_string(),
            ));
        }

        match self
            .chat_service
            .send_message(
                to,
                attachment.caption.as_deref(),
                Some(&attachment.file_path),
                None,
            )
            .await
        {
            Ok(_) => {
                info!("File sent to {}", to);
                Ok(SendMessageResult {
                    success: true,
                    message_id: Some(uuid::Uuid::new_v4().to_string()),
                    error: None,
                    retry_after_seconds: None,
                })
            }
            Err(e) => {
                error!("Failed to send file to {}: {}", to, e);
                Ok(SendMessageResult {
                    success: false,
                    message_id: None,
                    error: Some(e.to_string()),
                    retry_after_seconds: Some(30),
                })
            }
        }
    }

    /// Get engine status.
    pub async fn get_status(&self) -> Result<EngineStatus> {
        let uptime = self.start_time.elapsed().unwrap_or_default().as_secs();

        Ok(EngineStatus {
            is_ready: true,
            browser_connected: true,
            whatsapp_loaded: self.is_authenticated().await.unwrap_or(false),
            last_health_check: chrono::Utc::now(),
            uptime_seconds: uptime,
        })
    }

    /// Close and cleanup.
    pub async fn close(&self) -> Result<()> {
        info!("Closing WAS");

        if let Err(e) = self.browser_service.close().await {
            warn!("Error closing browser: {}", e);
        }

        info!("WAS closed");
        Ok(())
    }
}

impl Drop for WhatsAppEngine {
    fn drop(&mut self) {
        debug!("WhatsAppEngine dropped");
    }
}
