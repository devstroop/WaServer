// Library Integration Example
//
// This example demonstrates key patterns for integrating WhatsApp Engine
// as a library in your Rust applications.
//
// Run with: cargo run --example library_integration

use whatsapp_engine::{WhatsAppEngine, WhatsAppError, FileAttachment, Result};
use tokio::time::{sleep, Duration, timeout};
use tracing::{info, warn, error};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize structured logging
    tracing_subscriber::fmt()
        .with_env_filter("whatsapp_engine=info,library_integration=info")
        .init();

    info!("🚀 Starting WhatsApp Engine Library Integration Example");

    // Initialize engine with error handling
    let engine = match WhatsAppEngine::with_defaults().await {
        Ok(engine) => {
            info!("✅ WhatsApp Engine initialized successfully");
            engine
        }
        Err(e) => {
            error!("❌ Failed to initialize engine: {}", e);
            return Err(e);
        }
    };

    // Demonstrate authentication patterns
    if let Err(e) = demonstrate_authentication(&engine).await {
        error!("Authentication demonstration failed: {}", e);
    }

    // Demonstrate messaging with error handling
    if let Err(e) = demonstrate_messaging(&engine).await {
        error!("Messaging demonstration failed: {}", e);
    }

    // Demonstrate status monitoring
    if let Err(e) = demonstrate_monitoring(&engine).await {
        error!("Monitoring demonstration failed: {}", e);
    }

    // Clean shutdown
    info!("🧹 Performing clean shutdown...");
    engine.close().await?;
    info!("👋 Library integration example completed!");

    Ok(())
}

/// Demonstrates authentication patterns and session management
async fn demonstrate_authentication(engine: &WhatsAppEngine) -> Result<()> {
    info!("🔐 Demonstrating authentication patterns...");

    // Check existing session
    match engine.is_authenticated().await {
        Ok(true) => {
            info!("✅ Already authenticated from previous session");
            
            // Get detailed status
            let status = engine.get_auth_status().await?;
            info!("📊 Auth status: {:?}", status);
            
            return Ok(());
        }
        Ok(false) => {
            info!("🔑 No existing session, starting authentication...");
        }
        Err(e) => {
            warn!("⚠️ Could not check authentication status: {}", e);
        }
    }

    // Method 1: QR Code Authentication (recommended for interactive use)
    info!("📱 Starting QR code authentication...");
    
    match engine.authenticate_with_qr().await {
        Ok(qr) => {
            info!("🎯 QR code generated successfully");
            info!("📋 QR Data: {} (first 50 chars)", &qr.data.chars().take(50).collect::<String>());
            
            if let Some(expires_at) = qr.expires_at {
                info!("⏰ QR code expires at: {}", expires_at);
            }
            
            // Wait for authentication with timeout
            info!("⏳ Waiting for QR code scan (max 2 minutes)...");
            
            let auth_result = timeout(Duration::from_secs(120), async {
                let mut attempts = 0;
                while !engine.is_authenticated().await.unwrap_or(false) {
                    sleep(Duration::from_secs(3)).await;
                    attempts += 1;
                    
                    if attempts % 10 == 0 {
                        info!("Still waiting for QR scan... ({}s)", attempts * 3);
                    }
                }
                Ok::<(), WhatsAppError>(())
            }).await;

            match auth_result {
                Ok(_) => {
                    info!("🎉 QR code authentication successful!");
                }
                Err(_) => {
                    warn!("⏰ QR code authentication timed out");
                    // In real applications, you might want to generate a new QR code
                }
            }
        }
        Err(e) => {
            error!("❌ QR code generation failed: {}", e);
            
            // Method 2: Phone Authentication (fallback or for automated setups)
            info!("📞 Trying phone authentication as fallback...");
            demonstrate_phone_auth(engine).await?;
        }
    }

    Ok(())
}

/// Demonstrates phone number authentication
async fn demonstrate_phone_auth(engine: &WhatsAppEngine) -> Result<()> {
    // Note: Replace with actual phone number for testing
    let phone_number = "+1234567890"; // This should be a real number in production
    
    info!("📞 Starting phone authentication for {}", phone_number);
    
    match engine.authenticate_with_phone(phone_number).await {
        Ok(result) => {
            if result.success {
                info!("✅ Phone authentication initiated successfully");
                
                if let Some(code) = result.verification_code {
                    info!("🔢 Verification code: {}", code);
                    info!("💡 Enter this code in your WhatsApp mobile app");
                }
                
                // Wait for authentication completion
                let mut attempts = 0;
                while !engine.is_authenticated().await.unwrap_or(false) && attempts < 30 {
                    sleep(Duration::from_secs(2)).await;
                    attempts += 1;
                }
                
                if engine.is_authenticated().await.unwrap_or(false) {
                    info!("🎉 Phone authentication completed!");
                } else {
                    warn!("⏰ Phone authentication did not complete in time");
                }
            } else {
                warn!("❌ Phone authentication failed: {}", result.message);
                
                if let Some(retry_after) = result.next_retry_in_seconds {
                    warn!("🔄 Can retry after {} seconds", retry_after);
                }
            }
        }
        Err(e) => {
            error!("❌ Phone authentication error: {}", e);
        }
    }
    
    Ok(())
}

/// Demonstrates messaging operations with comprehensive error handling
async fn demonstrate_messaging(engine: &WhatsAppEngine) -> Result<()> {
    info!("💬 Demonstrating messaging operations...");

    // Check authentication before messaging
    if !engine.is_authenticated().await.unwrap_or(false) {
        warn!("⚠️ Not authenticated, skipping messaging demonstration");
        return Ok(());
    }

    // Test phone numbers - replace with real numbers for actual testing
    let test_numbers = vec![
        "1234567890",
        "0987654321",
    ];

    // Demonstrate text messaging
    for phone in &test_numbers {
        let message = format!("Hello from WhatsApp Engine! 🚀 Sent at {}", 
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"));
        
        match send_message_with_retry(engine, phone, &message, 3).await {
            Ok(true) => info!("✅ Message sent successfully to {}", phone),
            Ok(false) => warn!("⚠️ Message failed to send to {}", phone),
            Err(e) => error!("❌ Error sending to {}: {}", phone, e),
        }
        
        // Rate limiting - space out messages
        sleep(Duration::from_millis(1000)).await;
    }

    // Demonstrate file sending (when file exists)
    let test_file = "README.md"; // Use existing file for demo
    if std::path::Path::new(test_file).exists() {
        info!("📎 Demonstrating file attachment...");
        
        let attachment = FileAttachment {
            file_path: test_file.to_string(),
            file_name: Some("WhatsApp_Engine_README.md".to_string()),
            mime_type: Some("text/markdown".to_string()),
            caption: Some("📚 WhatsApp Engine documentation".to_string()),
        };

        match engine.send_file(&test_numbers[0], &attachment).await {
            Ok(result) => {
                if result.success {
                    info!("✅ File sent successfully");
                } else {
                    warn!("⚠️ File sending failed: {:?}", result.error);
                }
            }
            Err(e) => {
                warn!("❌ File sending error: {}", e);
            }
        }
    }

    // Demonstrate contact and chat retrieval
    info!("👥 Retrieving contacts...");
    match engine.get_contacts().await {
        Ok(contacts) => {
            info!("📱 Retrieved {} contacts", contacts.len());
            // Note: Implementation may return empty list if not fully implemented
        }
        Err(e) => {
            warn!("⚠️ Could not retrieve contacts: {}", e);
        }
    }

    info!("💬 Retrieving chats...");
    match engine.get_chats().await {
        Ok(chats) => {
            info!("🗣️ Retrieved {} chats", chats.len());
        }
        Err(e) => {
            warn!("⚠️ Could not retrieve chats: {}", e);
        }
    }

    Ok(())
}

/// Demonstrates health monitoring and status checks
async fn demonstrate_monitoring(engine: &WhatsAppEngine) -> Result<()> {
    info!("🔍 Demonstrating health monitoring...");

    // Get engine status
    match engine.get_status().await {
        Ok(status) => {
            info!("📊 Engine Status:");
            info!("  Ready: {}", status.is_ready);
            info!("  Browser Connected: {}", status.browser_connected);
            info!("  WhatsApp Loaded: {}", status.whatsapp_loaded);
            info!("  Uptime: {}s", status.uptime_seconds);
            info!("  Last Health Check: {}", status.last_health_check);

            // Alert on issues
            if !status.is_ready {
                warn!("🚨 Engine not ready!");
            }
            if !status.browser_connected {
                warn!("🚨 Browser connection issues!");
            }
            if !status.whatsapp_loaded {
                warn!("🚨 WhatsApp Web not loaded!");
            }
        }
        Err(e) => {
            error!("❌ Could not get engine status: {}", e);
        }
    }

    // Get authentication status
    match engine.get_auth_status().await {
        Ok(auth_status) => {
            info!("🔐 Authentication Status:");
            info!("  Authenticated: {}", auth_status.is_authenticated);
            if let Some(phone) = auth_status.phone_number {
                info!("  Phone: {}", phone);
            }
            if let Some(auth_time) = auth_status.authenticated_at {
                let duration = chrono::Utc::now() - auth_time;
                info!("  Authenticated: {} ago", format_duration(duration));
            }
        }
        Err(e) => {
            error!("❌ Could not get auth status: {}", e);
        }
    }

    Ok(())
}

/// Demonstrates robust message sending with retry logic
async fn send_message_with_retry(
    engine: &WhatsAppEngine,
    phone: &str,
    message: &str,
    max_retries: u32,
) -> Result<bool> {
    let mut attempts = 0;

    loop {
        match engine.send_message(phone, message).await {
            Ok(result) => {
                return Ok(result.success);
            }
            Err(e) => {
                attempts += 1;
                
                if e.is_retryable() && attempts <= max_retries {
                    let delay = e.retry_delay_seconds().unwrap_or(2_u32.pow(attempts.min(4)));
                    warn!("⚠️ Attempt {} failed: {}. Retrying in {}s...", attempts, e, delay);
                    sleep(Duration::from_secs(delay as u64)).await;
                } else {
                    return Err(e);
                }
            }
        }
    }
}

/// Formats a duration in a human-readable way
fn format_duration(duration: chrono::Duration) -> String {
    let total_seconds = duration.num_seconds();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}
