// MCP (Model Context Protocol) Server over SSE
//
// This module implements an MCP server that exposes WAS (WhatsApp Server) functionality
// as tools that AI agents can use. Communication happens over Server-Sent Events (SSE).

use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
    Json,
};
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{collections::HashMap, convert::Infallible, sync::Arc, time::Duration};
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::services::whatsapp::WhatsAppService;

// ============================================================================
// MCP Protocol Types
// ============================================================================

/// MCP JSON-RPC Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Option<Value>,
}

/// MCP JSON-RPC Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<McpError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// MCP Tool Definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

/// MCP Tool Call Result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolResult {
    pub content: Vec<McpContent>,
    #[serde(rename = "isError", skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

/// MCP Server Info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerInfo {
    pub name: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol_version: Option<String>,
}

/// MCP Capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<McpToolsCapability>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompts: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolsCapability {
    #[serde(rename = "listChanged", skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

// ============================================================================
// SSE Session Management
// ============================================================================

/// Active MCP SSE session
#[derive(Debug)]
pub struct McpSession {
    pub id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub tx: mpsc::Sender<McpResponse>,
}

/// MCP SSE State shared across handlers
#[derive(Clone)]
pub struct McpState {
    pub sessions: Arc<RwLock<HashMap<String, McpSession>>>,
    pub whatsapp_service: Arc<WhatsAppService>,
}

impl McpState {
    pub fn new(whatsapp_service: Arc<WhatsAppService>) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            whatsapp_service,
        }
    }
}

// ============================================================================
// Tool Definitions
// ============================================================================

fn get_available_tools() -> Vec<McpTool> {
    vec![
        McpTool {
            name: "whatsapp_get_auth_status".to_string(),
            description: "Check if WhatsApp Web is currently authenticated and get sender ID".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        McpTool {
            name: "whatsapp_get_qr_code".to_string(),
            description: "Get QR code for WhatsApp Web authentication. Returns base64-encoded QR code image.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        McpTool {
            name: "whatsapp_login_with_phone".to_string(),
            description: "Initiate phone number authentication for WhatsApp Web. Returns a linking code to enter in the mobile app.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "phone_number": {
                        "type": "string",
                        "description": "Phone number with country code (e.g., +1234567890)"
                    }
                },
                "required": ["phone_number"]
            }),
        },
        McpTool {
            name: "whatsapp_logout".to_string(),
            description: "Log out from WhatsApp Web session".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        McpTool {
            name: "whatsapp_send_message".to_string(),
            description: "Send a text message, file, or both to a WhatsApp contact or group".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "phone": {
                        "type": "string",
                        "description": "Recipient phone number with country code (e.g., +1234567890)"
                    },
                    "message": {
                        "type": "string",
                        "description": "Text message content to send (optional if file_path is provided)"
                    },
                    "file_path": {
                        "type": "string",
                        "description": "Absolute path to a file to send (image, video, or document). Can be sent alone or with a caption (message)"
                    }
                },
                "required": ["phone"]
            }),
        },
        McpTool {
            name: "whatsapp_health_check".to_string(),
            description: "Check the health status of the WAS (WhatsApp Server) service".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
    ]
}

// ============================================================================
// Tool Execution
// ============================================================================

async fn execute_tool(
    tool_name: &str,
    arguments: &Value,
    whatsapp_service: &WhatsAppService,
) -> McpToolResult {
    match tool_name {
        "whatsapp_get_auth_status" => match whatsapp_service.auth_service().is_authorized().await {
            Ok(authorized) => {
                let sender_id = if authorized {
                    whatsapp_service
                        .auth_service()
                        .get_sender_id()
                        .await
                        .ok()
                        .flatten()
                } else {
                    None
                };

                McpToolResult {
                    content: vec![McpContent {
                        content_type: "text".to_string(),
                        text: json!({
                            "authorized": authorized,
                            "sender_id": sender_id
                        })
                        .to_string(),
                    }],
                    is_error: None,
                }
            }
            Err(e) => McpToolResult {
                content: vec![McpContent {
                    content_type: "text".to_string(),
                    text: format!("Error checking auth status: {}", e),
                }],
                is_error: Some(true),
            },
        },

        "whatsapp_get_qr_code" => match whatsapp_service.auth_service().get_auth_qr_code().await {
            Ok(qr_code) => McpToolResult {
                content: vec![McpContent {
                    content_type: "text".to_string(),
                    text: json!({
                        "qr_code": qr_code,
                        "instructions": "Scan this QR code with WhatsApp mobile app to authenticate"
                    })
                    .to_string(),
                }],
                is_error: None,
            },
            Err(e) => McpToolResult {
                content: vec![McpContent {
                    content_type: "text".to_string(),
                    text: format!("Error getting QR code: {}", e),
                }],
                is_error: Some(true),
            },
        },

        "whatsapp_login_with_phone" => {
            let phone_number = arguments
                .get("phone_number")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            if phone_number.is_empty() {
                return McpToolResult {
                    content: vec![McpContent {
                        content_type: "text".to_string(),
                        text: "Error: phone_number is required".to_string(),
                    }],
                    is_error: Some(true),
                };
            }

            match whatsapp_service
                .auth_service()
                .login_with_phone_number(phone_number)
                .await
            {
                Ok(code) => {
                    let formatted_code = code.map(|c| c.replace(",", ""));
                    McpToolResult {
                        content: vec![McpContent {
                            content_type: "text".to_string(),
                            text: json!({
                                "linking_code": formatted_code,
                                "instructions": "Enter this code in WhatsApp mobile app: Settings > Linked Devices > Link a Device"
                            }).to_string(),
                        }],
                        is_error: None,
                    }
                }
                Err(e) => McpToolResult {
                    content: vec![McpContent {
                        content_type: "text".to_string(),
                        text: format!("Error with phone authentication: {}", e),
                    }],
                    is_error: Some(true),
                },
            }
        }

        "whatsapp_logout" => match whatsapp_service.auth_service().logout().await {
            Ok(_) => McpToolResult {
                content: vec![McpContent {
                    content_type: "text".to_string(),
                    text: json!({
                        "success": true,
                        "message": "Successfully logged out from WhatsApp Web"
                    })
                    .to_string(),
                }],
                is_error: None,
            },
            Err(e) => McpToolResult {
                content: vec![McpContent {
                    content_type: "text".to_string(),
                    text: format!("Error during logout: {}", e),
                }],
                is_error: Some(true),
            },
        },

        "whatsapp_send_message" => {
            let phone = arguments
                .get("phone")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let message = arguments.get("message").and_then(|v| v.as_str());
            let file_path = arguments.get("file_path").and_then(|v| v.as_str());

            if phone.is_empty() {
                return McpToolResult {
                    content: vec![McpContent {
                        content_type: "text".to_string(),
                        text: "Error: phone is required".to_string(),
                    }],
                    is_error: Some(true),
                };
            }

            // At least one of message or file_path is required
            if message.is_none() && file_path.is_none() {
                return McpToolResult {
                    content: vec![McpContent {
                        content_type: "text".to_string(),
                        text: "Error: either message or file_path is required".to_string(),
                    }],
                    is_error: Some(true),
                };
            }

            // Verify file exists if provided
            if let Some(path) = file_path {
                if !std::path::Path::new(path).exists() {
                    return McpToolResult {
                        content: vec![McpContent {
                            content_type: "text".to_string(),
                            text: format!("Error: file not found: {}", path),
                        }],
                        is_error: Some(true),
                    };
                }
            }

            match whatsapp_service
                .chat_service()
                .send_message(phone, message, file_path, None)
                .await
            {
                Ok(_) => McpToolResult {
                    content: vec![McpContent {
                        content_type: "text".to_string(),
                        text: json!({
                            "success": true,
                            "message": format!("Message sent successfully to {}", phone)
                        })
                        .to_string(),
                    }],
                    is_error: None,
                },
                Err(e) => McpToolResult {
                    content: vec![McpContent {
                        content_type: "text".to_string(),
                        text: format!("Error sending message: {}", e),
                    }],
                    is_error: Some(true),
                },
            }
        }

        "whatsapp_health_check" => {
            let is_busy = whatsapp_service.is_busy().await;
            let auth_ok = whatsapp_service
                .auth_service()
                .is_authorized()
                .await
                .unwrap_or(false);

            McpToolResult {
                content: vec![McpContent {
                    content_type: "text".to_string(),
                    text: json!({
                        "status": if auth_ok { "healthy" } else { "degraded" },
                        "authenticated": auth_ok,
                        "service_busy": is_busy,
                        "version": "0.2.0"
                    })
                    .to_string(),
                }],
                is_error: None,
            }
        }

        _ => McpToolResult {
            content: vec![McpContent {
                content_type: "text".to_string(),
                text: format!("Unknown tool: {}", tool_name),
            }],
            is_error: Some(true),
        },
    }
}

// ============================================================================
// MCP Request Handler
// ============================================================================

async fn handle_mcp_request(
    request: McpRequest,
    whatsapp_service: &WhatsAppService,
) -> McpResponse {
    let id = request.id.clone();

    match request.method.as_str() {
        "initialize" => {
            info!("MCP: Initialize request received");
            McpResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {
                            "listChanged": true
                        }
                    },
                    "serverInfo": {
                        "name": "was",
                        "version": "0.2.0"
                    }
                })),
                error: None,
            }
        }

        "notifications/initialized" | "initialized" => {
            debug!("MCP: Client initialized notification");
            McpResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(json!({})),
                error: None,
            }
        }

        "tools/list" => {
            info!("MCP: Tools list requested");
            let tools = get_available_tools();
            McpResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(json!({
                    "tools": tools
                })),
                error: None,
            }
        }

        "tools/call" => {
            let params = request.params.unwrap_or(json!({}));
            let tool_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

            info!("MCP: Tool call - {}", tool_name);
            debug!("MCP: Tool arguments: {:?}", arguments);

            let result = execute_tool(tool_name, &arguments, whatsapp_service).await;

            McpResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(serde_json::to_value(result).unwrap_or(json!({}))),
                error: None,
            }
        }

        "resources/list" => McpResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({
                "resources": []
            })),
            error: None,
        },

        "prompts/list" => McpResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({
                "prompts": []
            })),
            error: None,
        },

        "ping" => McpResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({})),
            error: None,
        },

        _ => {
            warn!("MCP: Unknown method: {}", request.method);
            McpResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(McpError {
                    code: -32601,
                    message: format!("Method not found: {}", request.method),
                    data: None,
                }),
            }
        }
    }
}

// ============================================================================
// HTTP Handlers
// ============================================================================

#[derive(Deserialize)]
pub struct SseQuery {
    #[serde(rename = "sessionId")]
    session_id: Option<String>,
}

/// SSE endpoint for MCP connection
///
/// Clients connect here to receive MCP responses via Server-Sent Events
pub async fn mcp_sse_handler(
    State(state): State<McpState>,
    Query(query): Query<SseQuery>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let session_id = query
        .session_id
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    let (tx, mut rx) = mpsc::channel::<McpResponse>(100);

    info!("MCP SSE: New connection - session_id: {}", session_id);

    // Store session
    {
        let mut sessions = state.sessions.write().await;
        sessions.insert(
            session_id.clone(),
            McpSession {
                id: session_id.clone(),
                created_at: chrono::Utc::now(),
                tx,
            },
        );
    }

    let session_id_clone = session_id.clone();

    // Create SSE stream
    let stream = async_stream::stream! {
        // Send initial endpoint event with just the URL string (not JSON)
        // This follows the legacy HTTP+SSE transport format
        let endpoint_url = format!("/mcp/message?sessionId={}", session_id_clone);
        yield Ok(Event::default()
            .event("endpoint")
            .data(endpoint_url));

        // Send session ready event
        yield Ok(Event::default()
            .event("session")
            .data(json!({
                "sessionId": session_id_clone,
                "status": "connected"
            }).to_string()));

        // Keep connection alive and forward responses
        loop {
            tokio::select! {
                Some(response) = rx.recv() => {
                    let data = serde_json::to_string(&response).unwrap_or_default();
                    yield Ok(Event::default()
                        .event("message")
                        .data(data));
                }
                _ = tokio::time::sleep(Duration::from_secs(30)) => {
                    // Send keepalive
                    yield Ok(Event::default()
                        .event("ping")
                        .data("{}"));
                }
            }
        }
    };

    Sse::new(stream).keep_alive(KeepAlive::default())
}

/// Streamable HTTP POST handler for MCP (new 2025 protocol)
///
/// Handles POST requests to the SSE endpoint for the new Streamable HTTP transport.
/// This enables clients to POST messages directly to /mcp/sse
pub async fn mcp_streamable_handler(
    State(state): State<McpState>,
    headers: axum::http::HeaderMap,
    Json(request): Json<McpRequest>,
) -> impl IntoResponse {
    // Get or create session from header
    let session_id = headers
        .get("Mcp-Session-Id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    info!(
        "MCP Streamable: session={}, method={}",
        session_id, request.method
    );

    // Handle the request
    let response = handle_mcp_request(request, &state.whatsapp_service).await;

    // For initialize, return session ID in header
    let mut resp_headers = axum::http::HeaderMap::new();
    resp_headers.insert(
        "Mcp-Session-Id",
        session_id.parse().unwrap_or_else(|_| "".parse().unwrap()),
    );

    // Return JSON response directly
    (StatusCode::OK, resp_headers, Json(response))
}

/// HTTP POST endpoint for sending MCP requests
///
/// Clients send JSON-RPC requests here, responses come via SSE
pub async fn mcp_message_handler(
    State(state): State<McpState>,
    Query(query): Query<SseQuery>,
    Json(request): Json<McpRequest>,
) -> impl IntoResponse {
    let session_id = match query.session_id {
        Some(id) => id,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "jsonrpc": "2.0",
                    "error": {
                        "code": -32600,
                        "message": "Missing sessionId query parameter"
                    }
                })),
            );
        }
    };

    debug!(
        "MCP Message: session={}, method={}",
        session_id, request.method
    );

    // Handle the request
    let response = handle_mcp_request(request, &state.whatsapp_service).await;

    // Try to send via SSE channel
    let sessions = state.sessions.read().await;
    if let Some(session) = sessions.get(&session_id) {
        if let Err(e) = session.tx.send(response.clone()).await {
            warn!("MCP: Failed to send response via SSE: {}", e);
        }
    }

    // Also return response directly for clients that prefer synchronous mode
    (
        StatusCode::OK,
        Json(serde_json::to_value(response).unwrap_or(json!({}))),
    )
}

/// Simple info endpoint about MCP server
pub async fn mcp_info_handler() -> impl IntoResponse {
    Json(json!({
        "name": "was",
        "version": "0.2.0",
        "protocol": "MCP",
        "transport": "Streamable HTTP",
        "endpoint": "/mcp",
        "tools": get_available_tools().iter().map(|t| &t.name).collect::<Vec<_>>()
    }))
}

/// DELETE handler for session termination (per MCP spec)
pub async fn mcp_session_delete_handler(
    State(state): State<McpState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    // Get session ID from header
    let session_id = headers
        .get("Mcp-Session-Id")
        .and_then(|v: &axum::http::HeaderValue| v.to_str().ok())
        .map(|s: &str| s.to_string());

    if let Some(session_id) = session_id {
        let mut sessions = state.sessions.write().await;
        if sessions.remove(&session_id).is_some() {
            info!("MCP: Session {} terminated by client", session_id);
            StatusCode::OK
        } else {
            StatusCode::NOT_FOUND
        }
    } else {
        StatusCode::BAD_REQUEST
    }
}

/// Health check for MCP service
pub async fn mcp_health_handler(State(state): State<McpState>) -> impl IntoResponse {
    let sessions = state.sessions.read().await;
    Json(json!({
        "status": "ok",
        "active_sessions": sessions.len(),
        "protocol_version": "2025-06-18"
    }))
}

// ============================================================================
// Router Builder
// ============================================================================

use axum::Router;

/// Create MCP routes that can be nested into the main app
pub fn mcp_routes<S>(whatsapp_service: Arc<WhatsAppService>) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    let state = McpState::new(whatsapp_service);

    Router::new()
        // Single MCP endpoint: GET for SSE stream, POST for messages, DELETE for session termination
        .route(
            "/",
            axum::routing::get(mcp_sse_handler)
                .post(mcp_streamable_handler)
                .delete(mcp_session_delete_handler),
        )
        .with_state(state)
}
