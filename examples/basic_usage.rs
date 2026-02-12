// Basic WhatsApp Engine library usage example
//
// This example demonstrates how to use the WhatsApp Engine as a library
// in your own Rust projects.
//
// Run with: cargo run --example basic_usage

use tokio::time::{sleep, Duration};
use whatsapp_engine::{Result, WhatsAppEngine};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging for better debugging
    tracing_subscriber::fmt::init();

    println!("🚀 Starting WhatsApp Engine Library Example");

    // Create engine with default configuration
    // This will load config from config/app.toml or environment variables
    let engine = WhatsAppEngine::with_defaults().await?;

    println!("✅ WhatsApp Engine initialized successfully");

    // Check if already authenticated
    if engine.is_authenticated().await? {
        println!("🎉 Already authenticated!");
    } else {
        println!("🔐 Starting authentication process...");

        // Method 1: QR Code Authentication (recommended for first-time setup)
        let qr_result = engine.authenticate_with_qr().await?;
        println!("📱 QR Code generated!");
        println!("📋 Scan this QR code with your WhatsApp mobile app:");
        println!("   Data: {} (base64 encoded image)", &qr_result.data[..50]);
        println!("   Expires at: {}", qr_result.expires_at);

        // Method 2: Phone Authentication (alternative)
        // Uncomment to use phone authentication instead:
        /*
        let phone_result = engine.authenticate_with_phone("+1234567890").await?;
        if phone_result.success {
            if let Some(code) = phone_result.verification_code {
                println!("📞 Phone authentication initiated");
                println!("🔢 Verification code: {}", code);
                println!("💡 Enter this code in your WhatsApp mobile app");
            }
        } else {
            println!("❌ Phone authentication failed: {}", phone_result.message);
        }
        */

        // Wait for authentication to complete
        println!("⏳ Waiting for authentication...");
        let mut attempts = 0;
        let max_attempts = 60; // Wait up to 2 minutes

        while !engine.is_authenticated().await? && attempts < max_attempts {
            print!(".");
            if attempts % 10 == 0 && attempts > 0 {
                println!(" ({}s)", attempts * 2);
            }
            sleep(Duration::from_secs(2)).await;
            attempts += 1;
        }

        if engine.is_authenticated().await? {
            println!("\n🎉 Authentication successful!");
        } else {
            println!("\n⏰ Authentication timed out. Please try again.");
            return Ok(());
        }
    }

    // Get authentication status
    let auth_status = engine.get_auth_status().await?;
    println!("📊 Auth Status: {:?}", auth_status);

    // Get engine status
    let engine_status = engine.get_status().await?;
    println!(
        "🔧 Engine Status: Ready={}, Uptime={}s",
        engine_status.is_ready, engine_status.uptime_seconds
    );

    // Example: Send a test message
    println!("💬 Sending test message...");

    // Replace with a real phone number to test messaging
    let test_phone = "1234567890"; // Change this to a real number
    let test_message = "Hello from WhatsApp Engine library! 🚀";

    let send_result = engine.send_message(test_phone, test_message).await?;

    if send_result.success {
        println!("✅ Message sent successfully!");
        if let Some(message_id) = send_result.message_id {
            println!("📝 Message ID: {}", message_id);
        }
    } else {
        println!("❌ Failed to send message: {:?}", send_result.error);
        if let Some(retry_after) = send_result.retry_after_seconds {
            println!("🔄 Retry after {} seconds", retry_after);
        }
    }

    // Example: Get contacts (when implemented)
    println!("👥 Retrieving contacts...");
    let contacts = engine.get_contacts().await?;
    println!("📱 Found {} contacts", contacts.len());

    // Example: Get chats (when implemented)
    println!("💬 Retrieving chats...");
    let chats = engine.get_chats().await?;
    println!("🗣️ Found {} chats", chats.len());

    // Example: File sending (when you want to test)
    /*
    use whatsapp_engine::FileAttachment;

    let attachment = FileAttachment {
        file_path: "path/to/your/file.jpg".to_string(),
        file_name: Some("test_image.jpg".to_string()),
        mime_type: Some("image/jpeg".to_string()),
        caption: Some("Test image from WhatsApp Engine!".to_string()),
    };

    let file_result = engine.send_file(test_phone, &attachment).await?;
    if file_result.success {
        println!("✅ File sent successfully!");
    } else {
        println!("❌ Failed to send file: {:?}", file_result.error);
    }
    */

    // Clean shutdown
    println!("🧹 Cleaning up...");
    engine.close().await?;

    println!("👋 WhatsApp Engine library example completed!");
    println!("💡 Tips for using the library:");
    println!("   - Authentication state persists between runs");
    println!("   - Use QR authentication for initial setup");
    println!("   - Phone authentication for automated setups");
    println!("   - Always call close() for proper cleanup");
    println!("   - Check is_authenticated() before sending messages");

    Ok(())
}

// Helper function to demonstrate error handling
async fn demonstrate_error_handling() -> Result<()> {
    let engine = WhatsAppEngine::with_defaults().await?;

    // Example of handling different error types
    match engine.send_message("", "test").await {
        Ok(result) => {
            println!("Message result: {:?}", result);
        }
        Err(e) => {
            match e {
                whatsapp_engine::WhatsAppError::InvalidInput { field, reason } => {
                    println!("Invalid input for {}: {}", field, reason);
                }
                whatsapp_engine::WhatsAppError::Authentication(msg) => {
                    println!("Authentication error: {}", msg);
                }
                whatsapp_engine::WhatsAppError::Timeout {
                    operation,
                    timeout_seconds,
                } => {
                    println!(
                        "Operation '{}' timed out after {}s",
                        operation, timeout_seconds
                    );
                }
                _ => {
                    println!("Other error: {}", e);
                }
            }

            // Check if error is retryable
            if e.is_retryable() {
                if let Some(delay) = e.retry_delay_seconds() {
                    println!("Error is retryable, suggested delay: {}s", delay);
                }
            }
        }
    }

    Ok(())
}
