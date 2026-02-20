//! Accounts Management API
//!
//! REST API endpoints for managing WhatsApp accounts (create, list, get, delete).
//! Account IDs are UUIDs. Lookup by phone number is also supported.
//! These endpoints do NOT require X-Account-Id header.

use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;

use crate::{
    models::account::{
        CreateAccountRequest, ListAccountsQuery, DeleteAccountResponse,
        DeleteAccountQuery, AccountActionResponse,
    },
    services::AccountManager,
};

// === API Handlers ===

/// List all accounts
///
/// Returns a list of all registered WhatsApp accounts.
#[utoipa::path(
    get,
    path = "/api/v1/accounts",
    tag = "Accounts",
    responses(
        (status = 200, description = "List of accounts", body = AccountListResponse),
    )
)]
pub async fn list_accounts(
    State(manager): State<Arc<AccountManager>>,
    Query(_query): Query<ListAccountsQuery>,
) -> impl IntoResponse {
    let response = manager.list_accounts().await;
    Json(response)
}

/// Create a new account
///
/// Creates a new WhatsApp account container with isolated data directory.
#[utoipa::path(
    post,
    path = "/api/v1/accounts",
    tag = "Accounts",
    request_body = CreateAccountRequest,
    responses(
        (status = 201, description = "Account created", body = CreateAccountResponse),
        (status = 400, description = "Invalid request"),
        (status = 409, description = "Account already exists"),
    )
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
///
/// Returns detailed information about a specific account.
#[utoipa::path(
    get,
    path = "/api/v1/accounts/{id}",
    tag = "Accounts",
    params(
        ("id" = String, Path, description = "Account ID (UUID or phone number)")
    ),
    responses(
        (status = 200, description = "Account info", body = AccountInfo),
        (status = 404, description = "Account not found"),
    )
)]
pub async fn get_account(
    State(manager): State<Arc<AccountManager>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match manager.get_account(&id).await {
        Some(account) => {
            let info = account.info().await;
            Json(json!(info)).into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "not_found",
                "message": format!("Account '{}' not found", id)
            })),
        )
            .into_response(),
    }
}

/// Delete an account
///
/// Deletes an account and optionally all its data.
#[utoipa::path(
    delete,
    path = "/api/v1/accounts/{id}",
    tag = "Accounts",
    params(
        ("id" = String, Path, description = "Account ID (UUID or phone number)"),
        ("delete_data" = bool, Query, description = "Delete all account data")
    ),
    responses(
        (status = 200, description = "Account deleted", body = DeleteAccountResponse),
        (status = 404, description = "Account not found"),
    )
)]
pub async fn delete_account(
    State(manager): State<Arc<AccountManager>>,
    Path(id): Path<String>,
    Query(query): Query<DeleteAccountQuery>,
) -> impl IntoResponse {
    match manager.delete_account(&id, query.delete_data).await {
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
///
/// Launches the browser for a specific account and navigates to WhatsApp Web.
#[utoipa::path(
    post,
    path = "/api/v1/accounts/{id}/start",
    tag = "Accounts",
    params(
        ("id" = String, Path, description = "Account ID (UUID or phone number)")
    ),
    responses(
        (status = 200, description = "Account started", body = AccountActionResponse),
        (status = 404, description = "Account not found"),
        (status = 409, description = "Account already running"),
    )
)]
pub async fn start_account(
    State(manager): State<Arc<AccountManager>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    // First get the account to retrieve its UUID
    let account = match manager.get_account(&id).await {
        Some(acc) => acc,
        None => return (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "not_found",
                "message": format!("Account '{}' not found", id)
            })),
        ).into_response(),
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
            let status = if error_msg.contains("already running") || error_msg.contains("already starting")
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
///
/// Stops the browser for a specific account.
#[utoipa::path(
    post,
    path = "/api/v1/accounts/{id}/stop",
    tag = "Accounts",
    params(
        ("id" = String, Path, description = "Account ID (UUID or phone number)")
    ),
    responses(
        (status = 200, description = "Account stopped", body = AccountActionResponse),
        (status = 404, description = "Account not found"),
    )
)]
pub async fn stop_account(
    State(manager): State<Arc<AccountManager>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    // First get the account to retrieve its UUID
    let account = match manager.get_account(&id).await {
        Some(acc) => acc,
        None => return (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "not_found",
                "message": format!("Account '{}' not found", id)
            })),
        ).into_response(),
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

/// Discover existing accounts
///
/// Scans the filesystem for existing account directories and loads them.
#[utoipa::path(
    post,
    path = "/api/v1/accounts/discover",
    tag = "Accounts",
    responses(
        (status = 200, description = "Accounts discovered"),
    )
)]
pub async fn discover_accounts(
    State(manager): State<Arc<AccountManager>>,
) -> impl IntoResponse {
    match manager.discover_accounts().await {
        Ok(discovered) => Json(json!({
            "message": format!("Discovered {} accounts", discovered.len()),
            "discovered": discovered,
        }))
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "discover_failed",
                "message": e.to_string()
            })),
        )
            .into_response(),
    }
}
