// MCP (Model Context Protocol) Server over SSE
//
// This module implements an MCP server that exposes WAS (WhatsApp Server) functionality
// as tools that AI agents can use. Communication happens over Server-Sent Events (SSE).
//
// Now uses AccountManager for multi-account support.

use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
    Json, Router,
};
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{collections::HashMap, convert::Infallible, sync::Arc, time::Duration};
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::services::{AccountManager, WhatsAppAccount};

// ============================================================================
// MCP Protocol Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Option<Value>,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

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

// ============================================================================
// SSE Session Management
// ============================================================================

#[derive(Debug)]
pub struct McpSession {
    pub id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub tx: mpsc::Sender<McpResponse>,
    /// Optional account ID to use for this session
    pub account_id: Option<String>,
}

#[derive(Clone)]
pub struct McpState {
    pub sessions: Arc<RwLock<HashMap<String, McpSession>>>,
    pub account_manager: Arc<AccountManager>,
}

impl McpState {
    pub fn new(account_manager: Arc<AccountManager>) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            account_manager,
        }
    }

    /// Get a running account for MCP operations
    /// Priority: 1) Session account_id, 2) First running account
    pub async fn get_account(
        &self,
        session_account_id: Option<&str>,
    ) -> Option<Arc<WhatsAppAccount>> {
        // If session has a specific account, use that
        if let Some(account_id) = session_account_id {
            if let Some(account) = self.account_manager.get_account(account_id).await {
                return Some(account);
            }
        }

        // Otherwise, find the first running account
        let account_list = self.account_manager.list_accounts().await;
        for info in &account_list.accounts {
            if matches!(info.status, crate::models::account::AccountStatus::Running) {
                return self.account_manager.get_account_by_id(info.id).await;
            }
        }

        // No running account, try to get any account
        if let Some(info) = account_list.accounts.first() {
            return self.account_manager.get_account_by_id(info.id).await;
        }

        None
    }
}

// ============================================================================
// Tool Definitions
// ============================================================================

fn get_available_tools() -> Vec<McpTool> {
    vec![
        McpTool {
            name: "whatsapp_get_auth_status".to_string(),
            description: "Check if WhatsApp Web is currently authenticated and get sender ID"
                .to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        McpTool {
            name: "whatsapp_get_qr_code".to_string(),
            description:
                "Get QR code for WhatsApp Web authentication. Returns base64-encoded QR code image."
                    .to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        McpTool {
            name: "whatsapp_login_with_phone".to_string(),
            description:
                "Initiate phone number authentication for WhatsApp Web. Returns a linking code."
                    .to_string(),
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
            description: "Send a text message, file, or both to a WhatsApp contact or group"
                .to_string(),
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
                        "description": "Absolute path to a file to send"
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
        McpTool {
            name: "whatsapp_list_accounts".to_string(),
            description: "List all WhatsApp accounts managed by the server".to_string(),
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
    state: &McpState,
    session_account_id: Option<&str>,
) -> McpToolResult {
    // Handle tools that don't need an account
    match tool_name {
        "whatsapp_list_accounts" => {
            let account_list = state.account_manager.list_accounts().await;
            return McpToolResult {
                content: vec![McpContent {
                    content_type: "text".to_string(),
                    text: serde_json::to_string_pretty(&account_list).unwrap_or_default(),
                }],
                is_error: None,
            };
        }
        "whatsapp_health_check" => {
            let account_count = state.account_manager.count().await;
            let running_account = state.get_account(session_account_id).await;
            let (is_busy, auth_ok) = if let Some(account) = running_account {
                let busy = account.is_busy().await;
                let auth = account
                    .auth_service()
                    .is_authorized()
                    .await
                    .unwrap_or(false);
                (busy, auth)
            } else {
                (false, false)
            };

            return McpToolResult {
                content: vec![McpContent {
                    content_type: "text".to_string(),
                    text: json!({
                        "status": if auth_ok { "healthy" } else { "degraded" },
                        "authenticated": auth_ok,
                        "service_busy": is_busy,
                        "accounts_count": account_count,
                        "version": env!("CARGO_PKG_VERSION")
                    })
                    .to_string(),
                }],
                is_error: None,
            };
        }
        _ => {}
    }

    // Get account for tools that need one
    let account = match state.get_account(session_account_id).await {
        Some(acc) => acc,
        None => {
            return McpToolResult {
                content: vec![McpContent {
                    content_type: "text".to_string(),
                    text: "No WhatsApp account available. Create one via POST /api/v1/accounts"
                        .to_string(),
                }],
                is_error: Some(true),
            };
        }
    };

    // Check if account browser is running for most operations
    let needs_browser = !matches!(
        tool_name,
        "whatsapp_list_accounts" | "whatsapp_health_check"
    );
    if needs_browser && !account.browser_service().is_running().await {
        return McpToolResult {
            content: vec![McpContent {
                content_type: "text".to_string(),
                text: format!(
                    "Account {} browser not running. Start it via POST /api/v1/accounts/{}/start",
                    account.id, account.id
                ),
            }],
            is_error: Some(true),
        };
    }

    match tool_name {
        "whatsapp_get_auth_status" => match account.auth_service().is_authorized().await {
            Ok(authorized) => {
                let sender_id = if authorized {
                    account.auth_service().get_sender_id().await.ok().flatten()
                } else {
                    None
                };

                McpToolResult {
                    content: vec![McpContent {
                        content_type: "text".to_string(),
                        text: json!({
                            "account_id": account.id,
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

        "whatsapp_get_qr_code" => match account.auth_service().get_auth_qr_code().await {
            Ok(qr_code) => McpToolResult {
                content: vec![McpContent {
                    content_type: "text".to_string(),
                    text: json!({
                        "account_id": account.id,
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

            match account.auth_service().login_with_phone_number(phone_number).await {
                Ok(code) => {
                    // Register phone on successful auth
                    if code.is_some() {
                        let _ = account.on_whatsapp_authenticated(phone_number).await;
                    }
                    McpToolResult {
                        content: vec![McpContent {
                            content_type: "text".to_string(),
                            text: json!({
                                "account_id": account.id,
                                "linking_code": code,
                                "instructions": "Enter this code in WhatsApp mobile app under Linked Devices"
                            })
                            .to_string(),
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

        "whatsapp_logout" => match account.auth_service().logout().await {
            Ok(_) => {
                account.invalidate_auth_cache().await;
                McpToolResult {
                    content: vec![McpContent {
                        content_type: "text".to_string(),
                        text: json!({
                            "account_id": account.id,
                            "message": "Successfully logged out from WhatsApp Web"
                        })
                        .to_string(),
                    }],
                    is_error: None,
                }
            }
            Err(e) => McpToolResult {
                content: vec![McpContent {
                    content_type: "text".to_string(),
                    text: format!("Error logging out: {}", e),
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

            if message.is_none() && file_path.is_none() {
                return McpToolResult {
                    content: vec![McpContent {
                        content_type: "text".to_string(),
                        text: "Error: either message or file_path is required".to_string(),
                    }],
                    is_error: Some(true),
                };
            }

            match account
                .execute_with_busy_flag(async {
                    account
                        .chat_service()
                        .send_message(phone, message, file_path, None)
                        .await
                })
                .await
            {
                Ok(_) => {
                    account.track_message_sent();
                    McpToolResult {
                        content: vec![McpContent {
                            content_type: "text".to_string(),
                            text: json!({
                                "account_id": account.id,
                                "message": format!("Message sent successfully to {}", phone)
                            })
                            .to_string(),
                        }],
                        is_error: None,
                    }
                }
                Err(e) => {
                    account.track_error();
                    McpToolResult {
                        content: vec![McpContent {
                            content_type: "text".to_string(),
                            text: format!("Error sending message: {}", e),
                        }],
                        is_error: Some(true),
                    }
                }
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
    state: &McpState,
    session_account_id: Option<&str>,
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
                        "version": env!("CARGO_PKG_VERSION")
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

            let result = execute_tool(tool_name, &arguments, state, session_account_id).await;

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
            result: Some(json!({ "resources": [] })),
            error: None,
        },

        "prompts/list" => McpResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({ "prompts": [] })),
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
    /// Optional account ID to use for this session
    #[serde(rename = "accountId")]
    account_id: Option<String>,
}

/// SSE endpoint for MCP connection
pub async fn mcp_sse_handler(
    State(state): State<McpState>,
    Query(query): Query<SseQuery>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let session_id = query
        .session_id
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    let account_id = query.account_id;
    let (tx, mut rx) = mpsc::channel::<McpResponse>(100);

    info!(
        "MCP SSE: New connection - session_id: {}, account_id: {:?}",
        session_id, account_id
    );

    // Store session
    {
        let mut sessions = state.sessions.write().await;
        sessions.insert(
            session_id.clone(),
            McpSession {
                id: session_id.clone(),
                created_at: chrono::Utc::now(),
                tx,
                account_id: account_id.clone(),
            },
        );
    }

    let session_id_clone = session_id.clone();

    let stream = async_stream::stream! {
        let endpoint_url = format!("/mcp/message?sessionId={}", session_id_clone);
        yield Ok(Event::default()
            .event("endpoint")
            .data(endpoint_url));

        yield Ok(Event::default()
            .event("session")
            .data(json!({
                "sessionId": session_id_clone,
                "status": "connected"
            }).to_string()));

        loop {
            tokio::select! {
                Some(response) = rx.recv() => {
                    let data = serde_json::to_string(&response).unwrap_or_default();
                    yield Ok(Event::default()
                        .event("message")
                        .data(data));
                }
                _ = tokio::time::sleep(Duration::from_secs(30)) => {
                    yield Ok(Event::default()
                        .event("ping")
                        .data("{}"));
                }
            }
        }
    };

    Sse::new(stream).keep_alive(KeepAlive::default())
}

/// Streamable HTTP POST handler for MCP
pub async fn mcp_streamable_handler(
    State(state): State<McpState>,
    headers: HeaderMap,
    Json(request): Json<McpRequest>,
) -> impl IntoResponse {
    let session_id = headers
        .get("Mcp-Session-Id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    // Get account ID from header
    let account_id = headers
        .get("X-Account-Id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    info!(
        "MCP Streamable: session={}, method={}, account={:?}",
        session_id, request.method, account_id
    );

    let response = handle_mcp_request(request, &state, account_id.as_deref()).await;

    let mut resp_headers = HeaderMap::new();
    resp_headers.insert(
        "Mcp-Session-Id",
        session_id.parse().unwrap_or_else(|_| "".parse().unwrap()),
    );

    (StatusCode::OK, resp_headers, Json(response))
}

/// HTTP POST endpoint for sending MCP requests
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

    // Get account_id from query or session
    let account_id = query.account_id.clone().or_else(|| {
        // Try to get from existing session
        let sessions = state.sessions.try_read().ok()?;
        sessions.get(&session_id)?.account_id.clone()
    });

    debug!(
        "MCP Message: session={}, method={}, account={:?}",
        session_id, request.method, account_id
    );

    let response = handle_mcp_request(request, &state, account_id.as_deref()).await;

    // Send via SSE channel
    let sessions = state.sessions.read().await;
    if let Some(session) = sessions.get(&session_id) {
        if let Err(e) = session.tx.send(response.clone()).await {
            warn!("MCP: Failed to send response via SSE: {}", e);
        }
    }

    (
        StatusCode::OK,
        Json(serde_json::to_value(response).unwrap_or(json!({}))),
    )
}

/// Simple info endpoint about MCP server
pub async fn mcp_info_handler(State(state): State<McpState>) -> impl IntoResponse {
    let accounts_count = state.account_manager.count().await;
    Json(json!({
        "name": "was",
        "version": env!("CARGO_PKG_VERSION"),
        "protocol": "MCP",
        "transport": "Streamable HTTP",
        "endpoint": "/mcp",
        "accounts_count": accounts_count,
        "tools": get_available_tools().iter().map(|t| &t.name).collect::<Vec<_>>()
    }))
}

/// DELETE handler for session termination
pub async fn mcp_session_delete_handler(
    State(state): State<McpState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let session_id = headers
        .get("Mcp-Session-Id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

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
    let accounts_count = state.account_manager.count().await;
    Json(json!({
        "status": "ok",
        "active_sessions": sessions.len(),
        "accounts_count": accounts_count,
        "protocol_version": "2025-06-18"
    }))
}

// ============================================================================
// Router Builder
// ============================================================================

/// Create MCP routes that can be nested into the main app
pub fn mcp_routes<S>(account_manager: Arc<AccountManager>) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    let state = McpState::new(account_manager);

    Router::new()
        .route(
            "/",
            axum::routing::get(mcp_sse_handler)
                .post(mcp_streamable_handler)
                .delete(mcp_session_delete_handler),
        )
        .route("/message", axum::routing::post(mcp_message_handler))
        .route("/info", axum::routing::get(mcp_info_handler))
        .route("/health", axum::routing::get(mcp_health_handler))
        .with_state(state)
}
