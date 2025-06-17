# WhatsApp Engine - Library Quick Reference

This is a quick reference guide for developers using WhatsApp Engine as a Rust library.

## 🚀 Quick Setup

```toml
# Cargo.toml
[dependencies]
whatsapp-engine = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
```

```rust
use whatsapp_engine::{WhatsAppEngine, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let engine = WhatsAppEngine::with_defaults().await?;
    // Use engine...
    engine.close().await?;
    Ok(())
}
```

## 🔑 Authentication

### QR Code (Interactive)
```rust
let qr = engine.authenticate_with_qr().await?;
println!("Scan: {}", qr.data);

while !engine.is_authenticated().await? {
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
}
```

### Phone Number (Automated)
```rust
let result = engine.authenticate_with_phone("+1234567890").await?;
if result.success {
    if let Some(code) = result.verification_code {
        println!("Code: {}", code);
    }
}
```

## 💬 Messaging

### Text Messages
```rust
let result = engine.send_message("1234567890", "Hello!").await?;
if result.success {
    println!("Sent! ID: {:?}", result.message_id);
}
```

### File Attachments
```rust
use whatsapp_engine::FileAttachment;

let attachment = FileAttachment {
    file_path: "document.pdf".to_string(),
    file_name: Some("Document.pdf".to_string()),
    mime_type: Some("application/pdf".to_string()),
    caption: Some("Please review 📄".to_string()),
};

let result = engine.send_file("1234567890", &attachment).await?;
```

## 🔧 Error Handling

```rust
use whatsapp_engine::WhatsAppError;

match engine.send_message("invalid", "test").await {
    Ok(result) => println!("Success: {:?}", result),
    Err(WhatsAppError::InvalidInput { field, reason }) => {
        println!("Invalid {}: {}", field, reason);
    }
    Err(WhatsAppError::Authentication(msg)) => {
        println!("Auth required: {}", msg);
    }
    Err(e) if e.is_retryable() => {
        println!("Retryable: {}", e);
        if let Some(delay) = e.retry_delay_seconds() {
            tokio::time::sleep(tokio::time::Duration::from_secs(delay as u64)).await;
            // Retry...
        }
    }
    Err(e) => println!("Error: {}", e),
}
```

## 📊 Status Monitoring

```rust
// Check authentication
let is_auth = engine.is_authenticated().await?;
let auth_status = engine.get_auth_status().await?;

// Engine health
let status = engine.get_status().await?;
println!("Ready: {}, Uptime: {}s", status.is_ready, status.uptime_seconds);
```

## ⚙️ Configuration

### Environment Variables
```bash
BROWSER_HEADLESS=true
BROWSER_TIMEOUT_MS=30000
LOGGING_LEVEL=info
```

### Custom Config
```rust
use whatsapp_engine::{AppConfig, BrowserConfig};

let config = AppConfig {
    browser: BrowserConfig {
        headless: false,  // Show browser
        timeout_ms: 60000,
        args: vec!["--no-sandbox".to_string()],
    },
    // ... other fields
};

let engine = WhatsAppEngine::new(config).await?;
```

## 🎯 Common Patterns

### Retry Logic
```rust
async fn send_with_retry(engine: &WhatsAppEngine, phone: &str, msg: &str) -> Result<bool> {
    for attempt in 1..=3 {
        match engine.send_message(phone, msg).await {
            Ok(result) => return Ok(result.success),
            Err(e) if e.is_retryable() => {
                let delay = e.retry_delay_seconds().unwrap_or(2_u32.pow(attempt));
                tokio::time::sleep(tokio::time::Duration::from_secs(delay as u64)).await;
            }
            Err(e) => return Err(e),
        }
    }
    Ok(false)
}
```

### Bulk Operations
```rust
use futures::future::join_all;

let phones = vec!["123", "456", "789"];
let message = "Bulk message";

let futures: Vec<_> = phones.iter()
    .map(|phone| engine.send_message(phone, message))
    .collect();

let results = join_all(futures).await;
```

### Health Monitoring
```rust
async fn monitor_health(engine: &WhatsAppEngine) {
    loop {
        let status = engine.get_status().await.unwrap();
        if !status.is_ready || !status.browser_connected {
            println!("🚨 Engine unhealthy!");
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
    }
}
```

## 📚 Key Types

```rust
// Results
pub struct SendMessageResult {
    pub success: bool,
    pub message_id: Option<String>,
    pub error: Option<String>,
    pub retry_after_seconds: Option<u32>,
}

// Authentication
pub struct QrCode {
    pub data: String,  // Base64 PNG
    pub expires_at: Option<DateTime<Utc>>,
    pub image_url: String,
    pub refresh_interval_seconds: u32,
}

// Status
pub struct EngineStatus {
    pub is_ready: bool,
    pub browser_connected: bool,
    pub whatsapp_loaded: bool,
    pub uptime_seconds: u64,
}
```

## 🔗 Resources

- **📖 Full Guide**: [`docs/DEVELOPER_GUIDE.md`](docs/DEVELOPER_GUIDE.md)
- **🚀 Examples**: [`examples/`](examples/) directory
- **📚 API Docs**: Run `cargo doc --open`
- **🎯 Basic Example**: [`examples/basic_usage.rs`](examples/basic_usage.rs)
- **⚡ Integration**: [`examples/library_integration.rs`](examples/library_integration.rs)
