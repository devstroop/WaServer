// Custom API server using WhatsApp Engine as a library
//
// This example shows how to build your own REST API server
// using the WhatsApp Engine library, with custom endpoints
// and business logic.
//
// Run with: cargo run --example custom_server

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tracing::{info, warn, error};
use whatsapp_engine::{WhatsAppEngine, Result as WaResult};

// Custom API models
#[derive(Serialize, Deserialize)]
struct QuickSendRequest {
    message: String,
}

#[derive(Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
    timestamp: String,
}

#[derive(Serialize)]
struct ServerStatus {
    server_name: String,
    version: String,
    uptime_seconds: u64,
    whatsapp_authenticated: bool,
    total_messages_sent: u64,
}

// Application state
struct AppState {
    engine: Arc<WhatsAppEngine>,
    start_time: std::time::SystemTime,
    message_counter: std::sync::atomic::AtomicU64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    info!("🚀 Starting Custom WhatsApp API Server");
    
    // Initialize WhatsApp Engine
    let engine = Arc::new(WhatsAppEngine::with_defaults().await?);
    
    // Create application state
    let state = Arc::new(AppState {
        engine,
        start_time: std::time::SystemTime::now(),
        message_counter: std::sync::atomic::AtomicU64::new(0),
    });
    
    // Build custom API routes
    let app = Router::new()
        // Health and status endpoints
        .route("/", get(root_handler))
        .route("/status", get(server_status))
        .route("/health", get(health_check))
        
        // Authentication endpoints
        .route("/auth/qr", post(generate_qr))
        .route("/auth/phone/:phone", post(authenticate_phone))
        .route("/auth/status", get(auth_status))
        .route("/auth/logout", post(logout))
        
        // Messaging endpoints - simplified API
        .route("/send/:phone", post(quick_send_message))
        .route("/send/:phone/:message", get(send_message_get)) // GET for easy testing
        .route("/message", post(send_message_post))
        
        // Contact and chat endpoints
        .route("/contacts", get(get_contacts))
        .route("/chats", get(get_chats))
        
        // Add CORS support
        .layer(CorsLayer::permissive())
        .with_state(state);
    
    // Start server
    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    info!("🌍 Custom WhatsApp API Server running on http://0.0.0.0:8080");
    info!("📚 Try these endpoints:");
    info!("   GET  /status          - Server status");
    info!("   POST /auth/qr         - Get QR code");
    info!("   GET  /auth/status     - Check authentication");
    info!("   GET  /send/1234567890/Hello - Quick send message");
    info!("   POST /send/1234567890 - Send message with JSON body");
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

// Route handlers

async fn root_handler() -> Json<ApiResponse<String>> {
    success_response(Some("Custom WhatsApp API Server - Powered by WhatsApp Engine Library".to_string()))
}

async fn server_status(State(state): State<Arc<AppState>>) -> Json<ApiResponse<ServerStatus>> {
    let uptime = state.start_time.elapsed()
        .unwrap_or_default()
        .as_secs();
    
    let is_authenticated = state.engine.is_authenticated().await.unwrap_or(false);
    let message_count = state.message_counter.load(std::sync::atomic::Ordering::Relaxed);
    
    let status = ServerStatus {
        server_name: "Custom WhatsApp API".to_string(),
        version: "1.0.0".to_string(),
        uptime_seconds: uptime,
        whatsapp_authenticated: is_authenticated,
        total_messages_sent: message_count,
    };
    
    success_response(Some(status))
}

async fn health_check(State(state): State<Arc<AppState>>) -> Result<Json<ApiResponse<String>>, StatusCode> {
    match state.engine.get_status().await {
        Ok(status) => {
            if status.is_ready {
                Ok(success_response(Some("Healthy".to_string())))
            } else {
                Err(StatusCode::SERVICE_UNAVAILABLE)
            }
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn generate_qr(State(state): State<Arc<AppState>>) -> Json<ApiResponse<serde_json::Value>> {
    match state.engine.authenticate_with_qr().await {
        Ok(qr) => {
            info!("QR code generated successfully");
            let qr_data = serde_json::json!({
                "qr_code": qr.data,
                "expires_at": qr.expires_at,
                "refresh_interval": qr.refresh_interval_seconds
            });
            success_response(Some(qr_data))
        }
        Err(e) => {
            error!("Failed to generate QR code: {}", e);
            error_response(&format!("Failed to generate QR code: {}", e))
        }
    }
}

async fn authenticate_phone(
    Path(phone): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<serde_json::Value>> {
    match state.engine.authenticate_with_phone(&phone).await {
        Ok(result) => {
            info!("Phone authentication result: success={}", result.success);
            let auth_data = serde_json::json!({
                "success": result.success,
                "verification_code": result.verification_code,
                "message": result.message,
                "next_retry_in_seconds": result.next_retry_in_seconds
            });
            success_response(Some(auth_data))
        }
        Err(e) => {
            error!("Phone authentication failed: {}", e);
            error_response(&format!("Phone authentication failed: {}", e))
        }
    }
}

async fn auth_status(State(state): State<Arc<AppState>>) -> Json<ApiResponse<serde_json::Value>> {
    match state.engine.get_auth_status().await {
        Ok(status) => {
            let status_data = serde_json::json!({
                "authenticated": status.is_authenticated,
                "phone_number": status.phone_number,
                "session_id": status.session_id,
                "authenticated_at": status.authenticated_at
            });
            success_response(Some(status_data))
        }
        Err(e) => {
            error!("Failed to get auth status: {}", e);
            error_response(&format!("Failed to get auth status: {}", e))
        }
    }
}

async fn logout(State(state): State<Arc<AppState>>) -> Json<ApiResponse<String>> {
    match state.engine.logout().await {
        Ok(_) => {
            info!("Logout successful");
            success_response(Some("Logged out successfully".to_string()))
        }
        Err(e) => {
            error!("Logout failed: {}", e);
            error_response(&format!("Logout failed: {}", e))
        }
    }
}

// Quick send message via GET (easy for testing)
async fn send_message_get(
    Path((phone, message)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<serde_json::Value>> {
    send_message_internal(&state, &phone, &message).await
}

// Send message via POST with JSON body
async fn quick_send_message(
    Path(phone): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(request): Json<QuickSendRequest>,
) -> Json<ApiResponse<serde_json::Value>> {
    send_message_internal(&state, &phone, &request.message).await
}

// Send message via POST with full JSON
async fn send_message_post(
    State(state): State<Arc<AppState>>,
    Json(request): Json<serde_json::Value>,
) -> Json<ApiResponse<serde_json::Value>> {
    let phone = request.get("phone")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let message = request.get("message")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    
    if phone.is_empty() || message.is_empty() {
        return error_response("Both 'phone' and 'message' fields are required");
    }
    
    send_message_internal(&state, phone, message).await
}

async fn send_message_internal(
    state: &AppState,
    phone: &str,
    message: &str,
) -> Json<ApiResponse<serde_json::Value>> {
    info!("Sending message to {} - {}", phone, message);
    
    match state.engine.send_message(phone, message).await {
        Ok(result) => {
            if result.success {
                // Increment message counter
                state.message_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                
                info!("Message sent successfully to {}", phone);
                let response_data = serde_json::json!({
                    "success": true,
                    "message_id": result.message_id,
                    "to": phone,
                    "message": message,
                    "sent_at": chrono::Utc::now()
                });
                success_response(Some(response_data))
            } else {
                warn!("Message sending failed: {:?}", result.error);
                let response_data = serde_json::json!({
                    "success": false,
                    "error": result.error,
                    "retry_after_seconds": result.retry_after_seconds
                });
                success_response(Some(response_data))
            }
        }
        Err(e) => {
            error!("Failed to send message to {}: {}", phone, e);
            error_response(&format!("Failed to send message: {}", e))
        }
    }
}

async fn get_contacts(State(state): State<Arc<AppState>>) -> Json<ApiResponse<Vec<whatsapp_engine::Contact>>> {
    match state.engine.get_contacts().await {
        Ok(contacts) => {
            info!("Retrieved {} contacts", contacts.len());
            success_response(Some(contacts))
        }
        Err(e) => {
            error!("Failed to get contacts: {}", e);
            error_response(&format!("Failed to get contacts: {}", e))
        }
    }
}

async fn get_chats(State(state): State<Arc<AppState>>) -> Json<ApiResponse<Vec<whatsapp_engine::Chat>>> {
    match state.engine.get_chats().await {
        Ok(chats) => {
            info!("Retrieved {} chats", chats.len());
            success_response(Some(chats))
        }
        Err(e) => {
            error!("Failed to get chats: {}", e);
            error_response(&format!("Failed to get chats: {}", e))
        }
    }
}

// Helper functions for consistent responses

fn success_response<T: Serialize>(data: Option<T>) -> Json<ApiResponse<T>> {
    Json(ApiResponse {
        success: true,
        data,
        error: None,
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

fn error_response<T: Serialize>(error_message: &str) -> Json<ApiResponse<T>> {
    Json(ApiResponse {
        success: false,
        data: None,
        error: Some(error_message.to_string()),
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}
