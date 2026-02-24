//! Accounts Management API
//!
//! REST API endpoints for managing WhatsApp accounts (create, list, get, delete).
//! Account IDs are UUIDs. Lookup by phone number is also supported.
//! All endpoints require secret key authentication.

use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use serde_json::json;

use crate::{
    models::account::{
        AccountActionResponse, CreateAccountRequest, DeleteAccountQuery, DeleteAccountResponse,
        ListAccountsQuery, UpdateAccountConfigRequest,
    },
    services::AccountManager,
};

// === API Handlers ===

/// List all accounts
#[utoipa::path(
    get,
    path = "/api/v1/accounts",
    tag = "Accounts",
    responses(
        (status = 200, description = "List of accounts", body = AccountListResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_accounts(
    State(manager): State<Arc<AccountManager>>,
    Query(_query): Query<ListAccountsQuery>,
) -> impl IntoResponse {
    let accounts = manager.list_accounts().await;
    Json(accounts)
}

/// Create a new account
#[utoipa::path(
    post,
    path = "/api/v1/accounts",
    tag = "Accounts",
    request_body = CreateAccountRequest,
    responses(
        (status = 201, description = "Account created", body = CreateAccountResponse),
        (status = 400, description = "Invalid request"),
        (status = 409, description = "Account already exists"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_account(
    State(manager): State<Arc<AccountManager>>,
    Json(request): Json<CreateAccountRequest>,
) -> impl IntoResponse {
    match manager.create_account(request).await {
        Ok(response) => (StatusCode::CREATED, Json(json!(response))).into_response(),
        Err(e) => {
            let error_msg = e.to_string();
            let status = if error_msg.contains("already exists") {
                StatusCode::CONFLICT
            } else if error_msg.contains("Invalid") {
                StatusCode::BAD_REQUEST
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            (
                status,
                Json(json!({
                    "error": "create_failed",
                    "message": error_msg
                })),
            )
                .into_response()
        }
    }
}

/// Get account info
#[utoipa::path(
    get,
    path = "/api/v1/accounts/{account_id}",
    tag = "Accounts",
    params(
        ("account_id" = String, Path, description = "Account ID (UUID or phone number)")
    ),
    responses(
        (status = 200, description = "Account info", body = AccountInfo),
        (status = 404, description = "Account not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_account(
    State(manager): State<Arc<AccountManager>>,
    Path(account_id): Path<String>,
) -> impl IntoResponse {
    match manager.get_account(&account_id).await {
        Some(account) => {
            let info = account.info().await;
            Json(json!(info)).into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "not_found",
                "message": format!("Account '{}' not found", account_id)
            })),
        )
            .into_response(),
    }
}

/// Delete an account
#[utoipa::path(
    delete,
    path = "/api/v1/accounts/{account_id}",
    tag = "Accounts",
    params(
        ("account_id" = String, Path, description = "Account ID (UUID or phone number)"),
        ("delete_data" = bool, Query, description = "Delete all account data")
    ),
    responses(
        (status = 200, description = "Account deleted", body = DeleteAccountResponse),
        (status = 404, description = "Account not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_account(
    State(manager): State<Arc<AccountManager>>,
    Path(account_id): Path<String>,
    Query(query): Query<DeleteAccountQuery>,
) -> impl IntoResponse {
    match manager
        .delete_account(&account_id, query.delete_data)
        .await
    {
        Ok(account_id) => Json(json!(DeleteAccountResponse {
            message: if query.delete_data {
                "Account and all data deleted".to_string()
            } else {
                "Account deleted, data preserved".to_string()
            },
            account_id,
            data_deleted: query.delete_data,
        }))
        .into_response(),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "delete_failed",
                "message": e.to_string()
            })),
        )
            .into_response(),
    }
}

/// Start an account's browser
#[utoipa::path(
    post,
    path = "/api/v1/accounts/{account_id}/start",
    tag = "Account",
    params(
        ("account_id" = String, Path, description = "Account ID (UUID or phone number)")
    ),
    responses(
        (status = 200, description = "Account started", body = AccountActionResponse),
        (status = 404, description = "Account not found"),
        (status = 409, description = "Account already running"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn start_account(
    State(manager): State<Arc<AccountManager>>,
    Path(account_id): Path<String>,
) -> impl IntoResponse {
    let account = match manager.get_account(&account_id).await {
        Some(acc) => acc,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "not_found",
                    "message": format!("Account '{}' not found", account_id)
                })),
            )
                .into_response()
        }
    };

    let account_id = account.id;

    match account.start().await {
        Ok(()) => Json(json!(AccountActionResponse {
            message: "Account started successfully".to_string(),
            account_id,
        }))
        .into_response(),
        Err(e) => {
            let error_msg = e.to_string();
            let status = if error_msg.contains("already running")
                || error_msg.contains("already starting")
            {
                StatusCode::CONFLICT
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            (
                status,
                Json(json!({
                    "error": "start_failed",
                    "message": error_msg
                })),
            )
                .into_response()
        }
    }
}

/// Stop an account's browser
#[utoipa::path(
    delete,
    path = "/api/v1/accounts/{account_id}/stop",
    tag = "Account",
    params(
        ("account_id" = String, Path, description = "Account ID (UUID or phone number)")
    ),
    responses(
        (status = 200, description = "Account stopped", body = AccountActionResponse),
        (status = 404, description = "Account not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn stop_account(
    State(manager): State<Arc<AccountManager>>,
    Path(account_id): Path<String>,
) -> impl IntoResponse {
    let account = match manager.get_account(&account_id).await {
        Some(acc) => acc,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "not_found",
                    "message": format!("Account '{}' not found", account_id)
                })),
            )
                .into_response()
        }
    };

    let account_id = account.id;

    match account.stop().await {
        Ok(()) => Json(json!(AccountActionResponse {
            message: "Account stopped successfully".to_string(),
            account_id,
        }))
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "stop_failed",
                "message": e.to_string()
            })),
        )
            .into_response(),
    }
}

/// Get live screenshot of account's browser
#[utoipa::path(
    get,
    path = "/api/v1/accounts/{account_id}/screenshot",
    tag = "Account",
    params(
        ("account_id" = String, Path, description = "Account ID (UUID or phone number)")
    ),
    responses(
        (status = 200, description = "PNG screenshot", content_type = "image/png"),
        (status = 404, description = "Account not found"),
        (status = 503, description = "Browser not running"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn screenshot(
    State(manager): State<Arc<AccountManager>>,
    Path(account_id): Path<String>,
) -> impl IntoResponse {
    let account = match manager.get_account(&account_id).await {
        Some(acc) => acc,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "not_found",
                    "message": format!("Account '{}' not found", account_id)
                })),
            )
                .into_response()
        }
    };

    if !account.browser_service().is_running().await {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "error": "browser_not_running",
                "message": "Browser is not running. Start the account first via POST /api/v1/accounts/{account_id}/start"
            })),
        ).into_response();
    }

    match account.browser_service().screenshot().await {
        Ok(png_data) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "image/png")],
            png_data,
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "screenshot_failed",
                "message": e.to_string()
            })),
        )
            .into_response(),
    }
}

/// Get account configuration
#[utoipa::path(
    get,
    path = "/api/v1/accounts/{account_id}/config",
    tag = "Account",
    params(
        ("account_id" = String, Path, description = "Account ID (UUID or phone number)")
    ),
    responses(
        (status = 200, description = "Account configuration", body = AccountConfig),
        (status = 404, description = "Account not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_account_config(
    State(manager): State<Arc<AccountManager>>,
    Path(account_id): Path<String>,
) -> impl IntoResponse {
    let account = match manager.get_account(&account_id).await {
        Some(acc) => acc,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "not_found",
                    "message": format!("Account '{}' not found", account_id)
                })),
            )
                .into_response()
        }
    };

    let config = account.get_config().await;
    Json(config).into_response()
}

/// Update account configuration
#[utoipa::path(
    put,
    path = "/api/v1/accounts/{account_id}/config",
    tag = "Account",
    params(
        ("account_id" = String, Path, description = "Account ID (UUID or phone number)")
    ),
    request_body = UpdateAccountConfigRequest,
    responses(
        (status = 200, description = "Configuration updated", body = AccountConfig),
        (status = 404, description = "Account not found"),
        (status = 400, description = "Invalid configuration"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_account_config(
    State(manager): State<Arc<AccountManager>>,
    Path(account_id): Path<String>,
    Json(request): Json<UpdateAccountConfigRequest>,
) -> impl IntoResponse {
    let account = match manager.get_account(&account_id).await {
        Some(acc) => acc,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "not_found",
                    "message": format!("Account '{}' not found", account_id)
                })),
            )
                .into_response()
        }
    };

    match account.update_config(request).await {
        Ok(config) => Json(json!({
            "message": "Configuration updated successfully",
            "config": config,
            "restart_required": true
        }))
        .into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "config_update_failed",
                "message": e.to_string()
            })),
        )
            .into_response(),
    }
}

/// Reset an account
///
/// Stops the browser and wipes all session data (chrome profile, sessions, media).
/// The account itself is preserved — only runtime data is cleared.
/// Use this to start fresh without deleting and re-creating the account.
#[utoipa::path(
    post,
    path = "/api/v1/accounts/{account_id}/reset",
    tag = "Account",
    params(
        ("account_id" = String, Path, description = "Account ID (UUID or phone number)")
    ),
    responses(
        (status = 200, description = "Account reset", body = AccountActionResponse),
        (status = 404, description = "Account not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn reset_account(
    State(manager): State<Arc<AccountManager>>,
    Path(account_id): Path<String>,
) -> impl IntoResponse {
    let account_ref = match manager.get_account(&account_id).await {
        Some(acc) => acc,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "not_found",
                    "message": format!("Account '{}' not found", account_id)
                })),
            )
                .into_response()
        }
    };

    let id = account_ref.id;

    match account_ref.reset().await {
        Ok(()) => Json(json!(AccountActionResponse {
            message: "Account reset — all session data cleared".to_string(),
            account_id: id,
        }))
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "reset_failed",
                "message": e.to_string()
            })),
        )
            .into_response(),
    }
}
