# WhatsApp Engine Rust - Developer Guide 🛠️

This guide provides comprehensive documentation for developers who want to use WhatsApp Engine as a library in their Rust projects or contribute to its development.

## 📋 Table of Contents

- [Library Overview](#-library-overview)
- [Public API Reference](#-public-api-reference)
- [Quick Start Guide](#-quick-start-guide)
- [Configuration](#-configuration)
- [Error Handling](#-error-handling)
- [Authentication Methods](#-authentication-methods)
- [Messaging Operations](#-messaging-operations)
- [Advanced Usage](#-advanced-usage)
- [Extension Points](#-extension-points)
- [Best Practices](#-best-practices)
- [Troubleshooting](#-troubleshooting)
- [Contributing](#-contributing)

## 🏗️ Library Overview

WhatsApp Engine is designed as both a standalone API server and a reusable Rust library. When used as a library, it provides a clean, async API for WhatsApp Web automation with comprehensive error handling and resource management.

### Core Architecture

```rust
// Main entry point
WhatsAppEngine {
    auth_service: Arc<AuthService>,     // Authentication & session management
    chat_service: Arc<ChatService>,     // Messaging & chat operations
    browser_service: Arc<BrowserService>, // Browser control & lifecycle
    config: Arc<AppConfig>,             // Configuration management
}
```

### Key Design Principles

- **Async-first**: All operations are async using Tokio
- **Resource safety**: Automatic cleanup with proper Drop implementation
- **Error transparency**: Rich error types with retry guidance
- **Configuration flexibility**: Environment, file, or programmatic config
- **Thread safety**: All services are `Arc`-wrapped for sharing

## 📚 Public API Reference

### Core Types

#### `WhatsAppEngine`

The main entry point for library usage.

```rust
impl WhatsAppEngine {
    // Creation
    pub async fn new(config: AppConfig) -> Result<Self>
    pub async fn with_defaults() -> Result<Self>
    
    // Authentication
    pub async fn authenticate_with_qr(&self) -> Result<QrCode>
    pub async fn authenticate_with_phone(&self, phone_number: &str) -> Result<PhoneAuthResult>
    pub async fn is_authenticated(&self) -> Result<bool>
    pub async fn get_auth_status(&self) -> Result<AuthStatus>
    pub async fn logout(&self) -> Result<()>
    
    // Messaging
    pub async fn send_message(&self, to: &str, message: &str) -> Result<SendMessageResult>
    pub async fn send_file(&self, to: &str, attachment: &FileAttachment) -> Result<SendMessageResult>
    
    // Data retrieval
    pub async fn get_contacts(&self) -> Result<Vec<Contact>>
    pub async fn get_chats(&self) -> Result<Vec<Chat>>
    
    // Status & lifecycle
    pub async fn get_status(&self) -> Result<EngineStatus>
    pub async fn close(&self) -> Result<()>
}
```

### Domain Models

#### Authentication Models

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrCode {
    pub data: String,                    // Base64 encoded PNG
    pub expires_at: Option<DateTime<Utc>>,
    pub image_url: String,
    pub refresh_interval_seconds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhoneAuthResult {
    pub success: bool,
    pub verification_code: Option<String>,
    pub message: String,
    pub next_retry_in_seconds: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthStatus {
    pub is_authenticated: bool,
    pub phone_number: Option<String>,
    pub session_id: Option<String>,
    pub authenticated_at: Option<DateTime<Utc>>,
}
```

#### Communication Models

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub id: String,
    pub name: String,
    pub phone: String,
    pub is_business: bool,
    pub profile_picture_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chat {
    pub id: String,
    pub name: String,
    pub is_group: bool,
    pub last_message: Option<String>,
    pub unread_count: u32,
    pub last_activity: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub from: String,
    pub to: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub message_type: MessageType,
    pub status: MessageStatus,
}

pub enum MessageType { Text, Image, Document, Audio, Video, Sticker, Location }
pub enum MessageStatus { Sending, Sent, Delivered, Read, Failed }
```

#### File Handling

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAttachment {
    pub file_path: String,
    pub file_name: Option<String>,
    pub mime_type: Option<String>,
    pub caption: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageResult {
    pub success: bool,
    pub message_id: Option<String>,
    pub error: Option<String>,
    pub retry_after_seconds: Option<u32>,
}
```

### Error Types

```rust
pub enum WhatsAppError {
    // Browser errors
    BrowserInit(String),
    BrowserNavigation(String),
    BrowserConnection(String),
    
    // Authentication errors
    Authentication(String),
    QrCodeGeneration(String),
    PhoneAuthentication(String),
    SessionExpired,
    
    // Communication errors
    MessageSending(String),
    ContactRetrieval(String),
    ChatNavigation(String),
    FileUpload(String),
    
    // Infrastructure errors
    Configuration(String),
    Network(String),
    Timeout { operation: String, timeout_seconds: u64 },
    
    // Validation errors
    InvalidInput { field: String, reason: String },
    ServiceNotReady { service: String },
    RateLimit { operation: String, retry_after_seconds: u32 },
    PermissionDenied { operation: String },
    
    // Generic errors
    Internal(String),
}

impl WhatsAppError {
    pub fn is_retryable(&self) -> bool
    pub fn retry_delay_seconds(&self) -> Option<u32>
}
```

## 🚀 Quick Start Guide

### 1. Add Dependency

```toml
[dependencies]
whatsapp-engine = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
tracing = "0.1"
tracing-subscriber = "0.3"
```

### 2. Basic Usage

```rust
use whatsapp_engine::{WhatsAppEngine, Result};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Create engine
    let engine = WhatsAppEngine::with_defaults().await?;
    
    // Authenticate with QR code
    if !engine.is_authenticated().await? {
        let qr = engine.authenticate_with_qr().await?;
        println!("Scan QR code: {}", qr.data);
        
        // Wait for authentication
        while !engine.is_authenticated().await? {
            sleep(Duration::from_secs(2)).await;
        }
    }
    
    // Send message
    let result = engine.send_message("1234567890", "Hello!").await?;
    if result.success {
        println!("Message sent successfully!");
    }
    
    // Cleanup
    engine.close().await?;
    
    Ok(())
}
```

### 3. Custom Configuration

```rust
use whatsapp_engine::{WhatsAppEngine, AppConfig, BrowserConfig, ServerConfig};

let config = AppConfig {
    server: ServerConfig {
        host: "localhost".to_string(),
        port: 3000,
    },
    browser: BrowserConfig {
        headless: true,
        timeout_ms: 30000,
        args: vec!["--no-sandbox".to_string()],
    },
    // ... other config fields
};

let engine = WhatsAppEngine::new(config).await?;
```

## ⚙️ Configuration

### Configuration Sources

Configuration is loaded in this priority order:

1. **Environment variables** (highest priority)
2. **Configuration file** (`config/app.toml`)
3. **Default values** (lowest priority)

### Environment Variables

```bash
# Server configuration
SERVER_HOST=localhost
SERVER_PORT=3000

# Browser configuration
BROWSER_HEADLESS=true
BROWSER_TIMEOUT_MS=30000

# Authentication
AUTH_API_TOKEN=your-secret-token

# Logging
LOGGING_LEVEL=info
```

### Configuration File Example

```toml
# config/app.toml
[server]
host = "0.0.0.0"
port = 3000

[browser]
headless = true
timeout_ms = 30000
args = ["--no-sandbox", "--disable-dev-shm-usage"]

[auth]
api_token = "your-secret-token"

[logging]
level = "info"

[cache]
enabled = true
ttl_seconds = 3600

[cors]
allowed_origins = ["*"]
allowed_methods = ["GET", "POST", "PUT", "DELETE"]

[limits]
max_file_size_bytes = 16777216  # 16MB
request_timeout_ms = 30000
```

## 🔧 Error Handling

### Error Categories

```rust
use whatsapp_engine::{WhatsAppError, Result};

async fn handle_errors() -> Result<()> {
    let engine = WhatsAppEngine::with_defaults().await?;
    
    match engine.send_message("invalid", "test").await {
        Ok(result) => {
            if !result.success {
                println!("Message failed: {:?}", result.error);
            }
        }
        Err(e) => {
            match e {
                // Retryable errors
                WhatsAppError::Network(msg) => {
                    println!("Network error (retryable): {}", msg);
                    if let Some(delay) = e.retry_delay_seconds() {
                        tokio::time::sleep(Duration::from_secs(delay as u64)).await;
                        // Retry operation
                    }
                }
                
                // Authentication required
                WhatsAppError::Authentication(msg) => {
                    println!("Need to authenticate: {}", msg);
                    // Trigger authentication flow
                }
                
                // User input validation
                WhatsAppError::InvalidInput { field, reason } => {
                    println!("Invalid {}: {}", field, reason);
                    // Show user-friendly error
                }
                
                // Rate limiting
                WhatsAppError::RateLimit { operation, retry_after_seconds } => {
                    println!("Rate limited {}, retry after {}s", operation, retry_after_seconds);
                    // Implement backoff
                }
                
                _ => {
                    println!("Other error: {}", e);
                }
            }
        }
    }
    
    Ok(())
}
```

### Retry Strategies

```rust
use tokio::time::{sleep, Duration};

async fn send_with_retry(
    engine: &WhatsAppEngine,
    phone: &str,
    message: &str,
    max_retries: u32,
) -> Result<SendMessageResult> {
    let mut attempts = 0;
    
    loop {
        match engine.send_message(phone, message).await {
            Ok(result) => return Ok(result),
            Err(e) if e.is_retryable() && attempts < max_retries => {
                attempts += 1;
                if let Some(delay) = e.retry_delay_seconds() {
                    sleep(Duration::from_secs(delay as u64)).await;
                } else {
                    sleep(Duration::from_secs(2_u64.pow(attempts))).await; // Exponential backoff
                }
            }
            Err(e) => return Err(e),
        }
    }
}
```

## 🔐 Authentication Methods

### QR Code Authentication

Best for interactive setups and development:

```rust
async fn qr_auth_flow(engine: &WhatsAppEngine) -> Result<()> {
    println!("Starting QR authentication...");
    
    let qr = engine.authenticate_with_qr().await?;
    
    // Display QR code
    println!("QR Code Data: {}", qr.data);
    if let Some(expires_at) = qr.expires_at {
        println!("Expires at: {}", expires_at);
    }
    
    // Poll for authentication
    let mut attempts = 0;
    let max_attempts = qr.expires_at
        .map(|exp| ((exp - chrono::Utc::now()).num_seconds() / 2) as u32)
        .unwrap_or(150); // Default 5 minutes
    
    while !engine.is_authenticated().await? && attempts < max_attempts {
        tokio::time::sleep(Duration::from_secs(2)).await;
        attempts += 1;
        
        if attempts % 15 == 0 {
            println!("Still waiting for QR scan... ({}s)", attempts * 2);
        }
    }
    
    if engine.is_authenticated().await? {
        println!("✅ QR authentication successful!");
    } else {
        println!("❌ QR authentication timed out");
        return Err(WhatsAppError::timeout("qr_authentication", max_attempts as u64 * 2));
    }
    
    Ok(())
}
```

### Phone Number Authentication

Best for automated deployments:

```rust
async fn phone_auth_flow(engine: &WhatsAppEngine, phone: &str) -> Result<()> {
    println!("Starting phone authentication for {}", phone);
    
    let result = engine.authenticate_with_phone(phone).await?;
    
    if result.success {
        if let Some(code) = result.verification_code {
            println!("✅ Phone authentication initiated");
            println!("🔢 Verification code: {}", code);
            println!("💡 Enter this code in WhatsApp mobile app");
            
            // Poll for completion
            let mut attempts = 0;
            while !engine.is_authenticated().await? && attempts < 60 {
                tokio::time::sleep(Duration::from_secs(2)).await;
                attempts += 1;
            }
            
            if engine.is_authenticated().await? {
                println!("✅ Phone authentication completed!");
            } else {
                println!("❌ Phone authentication timed out");
            }
        }
    } else {
        println!("❌ Phone authentication failed: {}", result.message);
        if let Some(retry_after) = result.next_retry_in_seconds {
            println!("🔄 Can retry after {} seconds", retry_after);
        }
    }
    
    Ok(())
}
```

### Session Persistence

Authentication state is automatically persisted:

```rust
async fn check_existing_session(engine: &WhatsAppEngine) -> Result<bool> {
    // Check if already authenticated from previous session
    if engine.is_authenticated().await? {
        let status = engine.get_auth_status().await?;
        println!("✅ Using existing session");
        
        if let Some(phone) = status.phone_number {
            println!("📱 Phone: {}", phone);
        }
        
        if let Some(auth_time) = status.authenticated_at {
            let duration = chrono::Utc::now() - auth_time;
            println!("⏰ Authenticated {} ago", format_duration(duration));
        }
        
        return Ok(true);
    }
    
    Ok(false)
}

fn format_duration(duration: chrono::Duration) -> String {
    let hours = duration.num_hours();
    let minutes = duration.num_minutes() % 60;
    
    if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else if minutes > 0 {
        format!("{}m", minutes)
    } else {
        format!("{}s", duration.num_seconds())
    }
}
```

## 💬 Messaging Operations

### Text Messages

```rust
async fn send_text_messages(engine: &WhatsAppEngine) -> Result<()> {
    let phones = vec!["1234567890", "0987654321"];
    let message = "Hello from WhatsApp Engine! 🚀";
    
    for phone in phones {
        match engine.send_message(phone, message).await? {
            SendMessageResult { success: true, message_id: Some(id), .. } => {
                println!("✅ Message sent to {}, ID: {}", phone, id);
            }
            SendMessageResult { success: false, error: Some(err), .. } => {
                println!("❌ Failed to send to {}: {}", phone, err);
            }
            _ => {
                println!("⚠️ Unknown result for {}", phone);
            }
        }
        
        // Rate limiting - space out messages
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    
    Ok(())
}
```

### File Attachments

```rust
use whatsapp_engine::FileAttachment;

async fn send_files(engine: &WhatsAppEngine) -> Result<()> {
    let attachment = FileAttachment {
        file_path: "/path/to/document.pdf".to_string(),
        file_name: Some("important_document.pdf".to_string()),
        mime_type: Some("application/pdf".to_string()),
        caption: Some("Please review this document 📄".to_string()),
    };
    
    // Validate file exists
    if !std::path::Path::new(&attachment.file_path).exists() {
        return Err(WhatsAppError::invalid_input(
            "file_path", 
            "File does not exist"
        ));
    }
    
    // Check file size (16MB limit)
    let metadata = std::fs::metadata(&attachment.file_path)
        .map_err(|e| WhatsAppError::FileUpload(e.to_string()))?;
    
    if metadata.len() > 16 * 1024 * 1024 {
        return Err(WhatsAppError::invalid_input(
            "file_size",
            "File too large (max 16MB)"
        ));
    }
    
    let result = engine.send_file("1234567890", &attachment).await?;
    
    if result.success {
        println!("✅ File sent successfully!");
    } else {
        println!("❌ File send failed: {:?}", result.error);
    }
    
    Ok(())
}
```

### Bulk Operations

```rust
use futures::future::join_all;

async fn send_bulk_messages(
    engine: &WhatsAppEngine,
    recipients: Vec<(&str, &str)>, // (phone, message) pairs
) -> Result<Vec<SendMessageResult>> {
    // Send messages concurrently with rate limiting
    const CONCURRENT_LIMIT: usize = 5;
    const DELAY_BETWEEN_BATCHES_MS: u64 = 1000;
    
    let mut results = Vec::new();
    
    for batch in recipients.chunks(CONCURRENT_LIMIT) {
        let batch_futures: Vec<_> = batch
            .iter()
            .map(|(phone, message)| {
                engine.send_message(phone, message)
            })
            .collect();
        
        let batch_results = join_all(batch_futures).await;
        results.extend(batch_results);
        
        // Delay between batches to avoid rate limiting
        if results.len() < recipients.len() {
            tokio::time::sleep(Duration::from_millis(DELAY_BETWEEN_BATCHES_MS)).await;
        }
    }
    
    // Log statistics
    let successful = results.iter().filter(|r| r.as_ref().map_or(false, |res| res.success)).count();
    let failed = results.len() - successful;
    
    println!("📊 Bulk send completed: {} successful, {} failed", successful, failed);
    
    Ok(results.into_iter().collect::<Result<Vec<_>>>()?)
}
```

## 🔧 Advanced Usage

### Custom Browser Configuration

```rust
use whatsapp_engine::{AppConfig, BrowserConfig};

async fn custom_browser_setup() -> Result<WhatsAppEngine> {
    let config = AppConfig {
        browser: BrowserConfig {
            headless: false, // Show browser for debugging
            timeout_ms: 60000, // 60 second timeout
            args: vec![
                "--no-sandbox".to_string(),
                "--disable-dev-shm-usage".to_string(),
                "--disable-gpu".to_string(),
                "--window-size=1920,1080".to_string(),
                "--user-agent=Mozilla/5.0 (WhatsApp Bot)".to_string(),
            ],
        },
        // ... other config
    };
    
    WhatsAppEngine::new(config).await
}
```

### Health Monitoring

```rust
async fn monitor_engine_health(engine: &WhatsAppEngine) -> Result<()> {
    loop {
        let status = engine.get_status().await?;
        
        println!("🔍 Engine Status Check:");
        println!("  Ready: {}", status.is_ready);
        println!("  Browser Connected: {}", status.browser_connected);
        println!("  WhatsApp Loaded: {}", status.whatsapp_loaded);
        println!("  Uptime: {}s", status.uptime_seconds);
        
        // Check authentication status
        match engine.is_authenticated().await {
            Ok(true) => println!("  ✅ Authenticated"),
            Ok(false) => println!("  ❌ Not authenticated"),
            Err(e) => println!("  ⚠️ Auth check failed: {}", e),
        }
        
        // Alert on issues
        if !status.is_ready || !status.browser_connected {
            println!("🚨 Engine not healthy! Consider restarting.");
        }
        
        tokio::time::sleep(Duration::from_secs(30)).await;
    }
}
```

### Resource Management

```rust
use std::sync::Arc;
use tokio::sync::Mutex;

struct EngineManager {
    engine: Arc<Mutex<Option<WhatsAppEngine>>>,
}

impl EngineManager {
    pub fn new() -> Self {
        Self {
            engine: Arc::new(Mutex::new(None)),
        }
    }
    
    pub async fn get_or_create(&self) -> Result<Arc<WhatsAppEngine>> {
        let mut guard = self.engine.lock().await;
        
        if let Some(ref engine) = *guard {
            // Check if engine is still healthy
            if engine.is_authenticated().await.unwrap_or(false) {
                return Ok(Arc::new(engine.clone())); // This won't work - need different approach
            }
        }
        
        // Create new engine
        println!("🔄 Creating new engine instance");
        let new_engine = WhatsAppEngine::with_defaults().await?;
        *guard = Some(new_engine);
        
        // Note: This is simplified - in practice you'd use Arc<WhatsAppEngine> throughout
        todo!("Implement proper Arc handling")
    }
    
    pub async fn close(&self) -> Result<()> {
        let mut guard = self.engine.lock().await;
        if let Some(engine) = guard.take() {
            engine.close().await?;
        }
        Ok(())
    }
}
```

## 🎯 Extension Points

### Custom Error Handling

```rust
use whatsapp_engine::{WhatsAppError, Result};

#[derive(Debug)]
struct CustomErrorHandler {
    retry_count: std::sync::atomic::AtomicU32,
    failure_log: Arc<Mutex<Vec<String>>>,
}

impl CustomErrorHandler {
    fn new() -> Self {
        Self {
            retry_count: std::sync::atomic::AtomicU32::new(0),
            failure_log: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    async fn handle_error(&self, error: &WhatsAppError) -> Option<Duration> {
        let count = self.retry_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        
        // Log error
        let mut log = self.failure_log.lock().await;
        log.push(format!("Attempt {}: {}", count + 1, error));
        
        // Custom retry logic
        match error {
            WhatsAppError::Network(_) => {
                if count < 5 {
                    Some(Duration::from_secs(2_u64.pow(count))) // Exponential backoff
                } else {
                    None
                }
            }
            WhatsAppError::RateLimit { retry_after_seconds, .. } => {
                Some(Duration::from_secs(*retry_after_seconds as u64))
            }
            WhatsAppError::BrowserConnection(_) => {
                if count < 3 {
                    Some(Duration::from_secs(10))
                } else {
                    None // Give up after 3 attempts
                }
            }
            _ => None, // Don't retry other errors
        }
    }
}
```

### Configuration Providers

```rust
use whatsapp_engine::AppConfig;

trait ConfigProvider {
    async fn load_config(&self) -> Result<AppConfig>;
}

struct DatabaseConfigProvider {
    db_url: String,
}

impl ConfigProvider for DatabaseConfigProvider {
    async fn load_config(&self) -> Result<AppConfig> {
        // Load configuration from database
        // This is a placeholder implementation
        todo!("Implement database config loading")
    }
}

struct RemoteConfigProvider {
    api_endpoint: String,
    api_key: String,
}

impl ConfigProvider for RemoteConfigProvider {
    async fn load_config(&self) -> Result<AppConfig> {
        // Fetch configuration from remote API
        todo!("Implement remote config loading")
    }
}

async fn create_engine_with_provider<P: ConfigProvider>(
    provider: P
) -> Result<WhatsAppEngine> {
    let config = provider.load_config().await?;
    WhatsAppEngine::new(config).await
}
```

## 📝 Best Practices

### 1. Resource Management

```rust
// ✅ Good: Always close resources
async fn good_resource_handling() -> Result<()> {
    let engine = WhatsAppEngine::with_defaults().await?;
    
    // Do work...
    
    // Always close
    engine.close().await?;
    Ok(())
}

// ✅ Better: Use RAII pattern
struct WhatsAppSession {
    engine: WhatsAppEngine,
}

impl WhatsAppSession {
    async fn new() -> Result<Self> {
        let engine = WhatsAppEngine::with_defaults().await?;
        Ok(Self { engine })
    }
}

impl Drop for WhatsAppSession {
    fn drop(&mut self) {
        // Note: Can't call async close() here
        // Consider using an async shutdown method instead
    }
}
```

### 2. Authentication Checks

```rust
// ✅ Always verify authentication before operations
async fn safe_operations(engine: &WhatsAppEngine) -> Result<()> {
    if !engine.is_authenticated().await? {
        return Err(WhatsAppError::Authentication(
            "Must authenticate before sending messages".to_string()
        ));
    }
    
    // Now safe to proceed
    engine.send_message("1234567890", "Hello!").await?;
    Ok(())
}
```

### 3. Error Propagation

```rust
// ✅ Proper error handling and propagation
async fn robust_message_sender(
    engine: &WhatsAppEngine,
    phone: &str,
    message: &str,
) -> Result<bool> {
    match engine.send_message(phone, message).await {
        Ok(result) => Ok(result.success),
        Err(WhatsAppError::Authentication(_)) => {
            // Handle auth specifically
            println!("Authentication required");
            Ok(false)
        }
        Err(e) if e.is_retryable() => {
            // Log but don't fail immediately
            println!("Retryable error: {}", e);
            Ok(false)
        }
        Err(e) => Err(e), // Propagate other errors
    }
}
```

### 4. Configuration Management

```rust
// ✅ Environment-aware configuration
use std::env;

fn create_config() -> AppConfig {
    let is_production = env::var("ENVIRONMENT").unwrap_or_default() == "production";
    
    AppConfig {
        browser: BrowserConfig {
            headless: is_production, // Show browser in development
            timeout_ms: if is_production { 30000 } else { 60000 },
            args: if is_production {
                vec!["--no-sandbox".to_string(), "--disable-dev-shm-usage".to_string()]
            } else {
                vec![]
            },
        },
        // ... other config
    }
}
```

### 5. Logging and Monitoring

```rust
use tracing::{info, warn, error, instrument};

struct InstrumentedEngine {
    inner: WhatsAppEngine,
}

impl InstrumentedEngine {
    #[instrument(skip(self))]
    async fn send_message_with_logging(
        &self,
        phone: &str,
        message: &str,
    ) -> Result<SendMessageResult> {
        info!("Sending message to {}", phone);
        
        let start = std::time::Instant::now();
        let result = self.inner.send_message(phone, message).await;
        let duration = start.elapsed();
        
        match &result {
            Ok(res) if res.success => {
                info!("Message sent successfully in {:?}", duration);
            }
            Ok(res) => {
                warn!("Message failed: {:?}", res.error);
            }
            Err(e) => {
                error!("Send error: {} (took {:?})", e, duration);
            }
        }
        
        result
    }
}
```

## 🔍 Troubleshooting

### Common Issues

#### 1. Browser Connection Problems

```rust
// Check browser service status
let status = engine.get_status().await?;
if !status.browser_connected {
    println!("Browser not connected. Try:");
    println!("- Check if Chrome/Chromium is installed");
    println!("- Verify browser arguments are valid");
    println!("- Check system resources (memory/CPU)");
    println!("- Try running with headless=false for debugging");
}
```

#### 2. Authentication Issues

```rust
// Debug authentication problems
if !engine.is_authenticated().await? {
    println!("Authentication troubleshooting:");
    println!("- QR code may have expired (try generating new one)");
    println!("- Phone number format must be +country_code_number");
    println!("- Check WhatsApp mobile app is connected to internet");
    println!("- Verify session persistence directory is writable");
}
```

#### 3. Message Sending Failures

```rust
match engine.send_message(phone, message).await {
    Err(WhatsAppError::InvalidInput { field, reason }) => {
        println!("Input validation failed:");
        println!("  Field: {}", field);
        println!("  Reason: {}", reason);
        if field == "phone_number" {
            println!("  Tip: Use international format (+1234567890)");
        }
    }
    Err(WhatsAppError::RateLimit { operation, retry_after_seconds }) => {
        println!("Rate limited: {} - wait {}s", operation, retry_after_seconds);
    }
    Err(e) => {
        println!("Send failed: {}", e);
        if e.is_retryable() {
            println!("This error is retryable");
        }
    }
    Ok(result) if !result.success => {
        println!("Message failed: {:?}", result.error);
    }
    _ => {}
}
```

### Debug Mode

```rust
// Enable debug mode for troubleshooting
let config = AppConfig {
    browser: BrowserConfig {
        headless: false, // Show browser window
        timeout_ms: 120000, // Longer timeout
        args: vec![
            "--disable-web-security".to_string(),
            "--remote-debugging-port=9222".to_string(),
        ],
    },
    logging: LoggingConfig {
        level: "debug".to_string(),
    },
    // ... other config
};

let engine = WhatsAppEngine::new(config).await?;
```

### Health Checks

```rust
async fn diagnose_engine(engine: &WhatsAppEngine) -> Result<()> {
    println!("🔍 Engine Diagnostics");
    
    // 1. Basic status
    let status = engine.get_status().await?;
    println!("Engine Status: {:#?}", status);
    
    // 2. Authentication
    match engine.get_auth_status().await {
        Ok(auth) => println!("Auth Status: {:#?}", auth),
        Err(e) => println!("Auth Check Failed: {}", e),
    }
    
    // 3. Browser connectivity test
    if !status.browser_connected {
        println!("❌ Browser connection issues detected");
        println!("   Try: restart with headless=false to see browser");
    }
    
    // 4. WhatsApp Web status
    if !status.whatsapp_loaded {
        println!("❌ WhatsApp Web not loaded");
        println!("   This usually indicates authentication issues");
    }
    
    Ok(())
}
```

## 🤝 Contributing

### Development Setup

```bash
# Clone repository
git clone https://github.com/your-org/whatsapp-engine-rust.git
cd whatsapp-engine-rust

# Install dependencies
cargo build

# Run tests
cargo test

# Run examples
cargo run --example basic_usage
```

### Code Organization

```
src/
├── lib.rs              # Public API & main WhatsAppEngine
├── error.rs            # Error types and handling
├── config/             # Configuration management
├── models/             # Domain models and data structures
├── services/           # Core business logic
│   ├── auth_service.rs     # Authentication logic
│   ├── chat_service.rs     # Messaging operations
│   ├── browser.rs          # Browser management
│   └── whatsapp.rs         # WhatsApp Web integration
├── handlers/           # HTTP API handlers (for server mode)
├── middleware/         # Request/response middleware
└── utils/              # Utilities and helpers
```

### Adding New Features

1. **Domain Models**: Add new types to `models/domain.rs`
2. **Business Logic**: Implement in appropriate service
3. **Public API**: Add methods to `WhatsAppEngine` in `lib.rs`
4. **Error Handling**: Add new error variants to `error.rs`
5. **Tests**: Add unit tests and integration tests
6. **Documentation**: Update this guide and add doc comments

### Testing Guidelines

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_engine_creation() {
        let engine = WhatsAppEngine::with_defaults().await;
        assert!(engine.is_ok());
    }
    
    #[tokio::test]
    async fn test_error_handling() {
        let engine = WhatsAppEngine::with_defaults().await.unwrap();
        
        // Test invalid input
        let result = engine.send_message("", "test").await;
        assert!(matches!(result, Err(WhatsAppError::InvalidInput { .. })));
    }
}
```

This completes the comprehensive developer guide for WhatsApp Engine Rust library. The guide covers all aspects from basic usage to advanced patterns and contribution guidelines.
