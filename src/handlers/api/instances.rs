//! Instances Management API
//!
//! REST API endpoints for managing WhatsApp instances (create, list, get, delete).
//! Instance IDs are UUIDs. Lookup by phone number is also supported.
//! These endpoints do NOT require X-Account-Id header.
//!
//! ## Access Control
//!
//! - Users can only see and manage instances they own or have been granted access to
//! - Secret token authentication has full access to all instances
//! - Admins have full access to all instances

use std::sync::Arc;

use axum::{
    extract::{Extension, Path, Query, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use serde_json::json;

use crate::{
    models::account::{
        CreateAccountRequest, ListAccountsQuery, DeleteAccountResponse,
        DeleteAccountQuery, AccountActionResponse, UpdateInstanceConfigRequest,
    },
    models::auth::AuthenticatedUser,
    models::user::InstanceOwnership,
    services::{AccountManager, database::DatabaseService},
};

// =============================================================================
// State Types
// =============================================================================

/// Combined state for instance handlers
///
/// Provides access to both the account manager (for instance operations)
/// and the database service (for ownership/access control).
#[derive(Clone)]
pub struct InstancesState {
    /// Account manager for instance operations
    pub manager: Arc<AccountManager>,
    /// Central database for user and ownership data
    pub db: Arc<DatabaseService>,
}

impl InstancesState {
    /// Create a new InstancesState
    pub fn new(manager: Arc<AccountManager>, db: Arc<DatabaseService>) -> Self {
        Self { manager, db }
    }
}

// === API Handlers ===

/// List all instances
///
/// Returns a list of all registered WhatsApp instances.
/// - Secret token: returns all instances
/// - Admin users: returns all instances
/// - Regular users: returns only owned instances and instances shared with them
#[utoipa::path(
    get,
    path = "/api/v1/instances",
    tag = "Instances",
    responses(
        (status = 200, description = "List of instances", body = AccountListResponse),
    )
)]
pub async fn list_instances(
    State(state): State<InstancesState>,
    auth_user: Option<Extension<AuthenticatedUser>>,
    Query(_query): Query<ListAccountsQuery>,
) -> impl IntoResponse {
    // Get all accounts from manager
    let all_accounts = state.manager.list_accounts().await;

    // Filter based on user access
    match auth_user.map(|e| e.0) {
        // Secret token or admin: return all instances
        Some(AuthenticatedUser::Secret) => Json(all_accounts),
        
        Some(AuthenticatedUser::LocalUser { is_admin: true, .. }) => {
            Json(all_accounts)
        }
        
        // Regular user: filter to accessible instances
        Some(AuthenticatedUser::LocalUser { user_id, .. }) => {
            // Get list of accessible instance IDs for this user
            let accessible = state.db.list_accessible_instances(user_id)
                .unwrap_or_default();
            let accessible_ids: std::collections::HashSet<String> = accessible
                .into_iter()
                .map(|(ownership, _)| ownership.instance_id)
                .collect();
            
            // Filter accounts list
            let filtered_accounts: Vec<_> = all_accounts.accounts
                .into_iter()
                .filter(|a| accessible_ids.contains(&a.id.to_string()))
                .collect();
            
            let total = filtered_accounts.len();
            Json(crate::models::account::AccountListResponse {
                accounts: filtered_accounts,
                total,
            })
        }
        
        // No auth context (shouldn't happen with middleware)
        None => Json(all_accounts),
    }
}

/// Create a new instance
///
/// Creates a new WhatsApp instance container with isolated data directory.
/// Instance is owned by the authenticated user. Secret token creates instances
/// without ownership tracking.
#[utoipa::path(
    post,
    path = "/api/v1/instances",
    tag = "Instances",
    request_body = CreateAccountRequest,
    responses(
        (status = 201, description = "Instance created", body = CreateAccountResponse),
        (status = 400, description = "Invalid request"),
        (status = 409, description = "Instance already exists"),
    )
)]
pub async fn create_instance(
    State(state): State<InstancesState>,
    auth_user: Option<Extension<AuthenticatedUser>>,
    Json(request): Json<CreateAccountRequest>,
) -> impl IntoResponse {
    match state.manager.create_account(request).await {
        Ok(response) => {
            // Record ownership if authenticated as local user
            if let Some(Extension(AuthenticatedUser::LocalUser { user_id, username, .. })) = auth_user {
                let ownership = InstanceOwnership {
                    instance_id: response.id.to_string(),
                    owner_id: user_id,
                    display_name: Some(response.phone_number.clone()),
                    description: None,
                    is_active: true,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                };
                
                if let Err(e) = state.db.create_instance_ownership(&ownership) {
                    tracing::error!(
                        instance_id = %response.id,
                        user = %username,
                        error = %e,
                        "Failed to record instance ownership"
                    );
                    // Don't fail the request - instance is created, ownership metadata is optional
                } else {
                    tracing::info!(
                        instance_id = %response.id,
                        owner = %username,
                        owner_id = %user_id,
                        "Instance ownership recorded"
                    );
                }
            }
            
            (StatusCode::CREATED, Json(json!(response))).into_response()
        }
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

/// Get instance info
///
/// Returns detailed information about a specific instance.
/// Requires read access to the instance.
#[utoipa::path(
    get,
    path = "/api/v1/instances/{instance_id}",
    tag = "Instances",
    params(
        ("instance_id" = String, Path, description = "Instance ID (UUID or phone number)")
    ),
    responses(
        (status = 200, description = "Instance info", body = AccountInfo),
        (status = 403, description = "Access denied"),
        (status = 404, description = "Instance not found"),
    )
)]
pub async fn get_instance(
    State(state): State<InstancesState>,
    auth_user: Option<Extension<AuthenticatedUser>>,
    Path(instance_id): Path<String>,
) -> impl IntoResponse {
    let account = match state.manager.get_account(&instance_id).await {
        Some(acc) => acc,
        None => return (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "not_found",
                "message": format!("Instance '{}' not found", instance_id)
            })),
        ).into_response(),
    };
    
    // Check access permissions
    if !check_instance_access(&state.db, &auth_user, &account.id.to_string(), false) {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "access_denied",
                "message": "You don't have access to this instance"
            })),
        ).into_response();
    }
    
    let info = account.info().await;
    Json(json!(info)).into_response()
}

/// Delete an instance
///
/// Deletes an instance and optionally all its data.
/// Requires owner or admin access.
#[utoipa::path(
    delete,
    path = "/api/v1/instances/{instance_id}",
    tag = "Instances",
    params(
        ("instance_id" = String, Path, description = "Instance ID (UUID or phone number)"),
        ("delete_data" = bool, Query, description = "Delete all instance data")
    ),
    responses(
        (status = 200, description = "Instance deleted", body = DeleteAccountResponse),
        (status = 403, description = "Access denied"),
        (status = 404, description = "Instance not found"),
    )
)]
pub async fn delete_instance(
    State(state): State<InstancesState>,
    auth_user: Option<Extension<AuthenticatedUser>>,
    Path(instance_id): Path<String>,
    Query(query): Query<DeleteAccountQuery>,
) -> impl IntoResponse {
    // Resolve the instance first to get its canonical ID
    let account = match state.manager.get_account(&instance_id).await {
        Some(acc) => acc,
        None => return (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "not_found",
                "message": format!("Instance '{}' not found", instance_id)
            })),
        ).into_response(),
    };
    
    let canonical_id = account.id.to_string();
    
    // Check delete permissions (owner or admin only)
    if !check_instance_access(&state.db, &auth_user, &canonical_id, true) {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "access_denied",
                "message": "Only the instance owner or an admin can delete instances"
            })),
        ).into_response();
    }
    
    match state.manager.delete_account(&instance_id, query.delete_data).await {
        Ok(account_id) => {
            // Also delete ownership record
            if let Err(e) = state.db.delete_instance_ownership(&canonical_id) {
                tracing::warn!(
                    instance_id = %canonical_id,
                    error = %e,
                    "Failed to delete instance ownership record"
                );
            }
            
            Json(json!(DeleteAccountResponse {
                message: if query.delete_data {
                    "Instance and all data deleted".to_string()
                } else {
                    "Instance deleted, data preserved".to_string()
                },
                account_id,
                data_deleted: query.delete_data,
            })).into_response()
        }
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "delete_failed",
                "message": e.to_string()
            })),
        ).into_response(),
    }
}

/// Start an instance's browser
///
/// Launches the browser for a specific instance and navigates to WhatsApp Web.
/// Requires manage access to the instance.
#[utoipa::path(
    post,
    path = "/api/v1/instances/{instance_id}/start",
    tag = "Instances",
    params(
        ("instance_id" = String, Path, description = "Instance ID (UUID or phone number)")
    ),
    responses(
        (status = 200, description = "Instance started", body = AccountActionResponse),
        (status = 403, description = "Access denied"),
        (status = 404, description = "Instance not found"),
        (status = 409, description = "Instance already running"),
    )
)]
pub async fn start_instance(
    State(state): State<InstancesState>,
    auth_user: Option<Extension<AuthenticatedUser>>,
    Path(instance_id): Path<String>,
) -> impl IntoResponse {
    let account = match state.manager.get_account(&instance_id).await {
        Some(acc) => acc,
        None => return (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "not_found",
                "message": format!("Instance '{}' not found", instance_id)
            })),
        ).into_response(),
    };
    
    let account_id = account.id;
    
    // Check manage permissions
    if !check_instance_manage_access(&state.db, &auth_user, &account_id.to_string()) {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "access_denied",
                "message": "You don't have permission to manage this instance"
            })),
        ).into_response();
    }
    
    match account.start().await {
        Ok(()) => Json(json!(AccountActionResponse {
            message: "Instance started successfully".to_string(),
            account_id,
        })).into_response(),
        Err(e) => {
            let error_msg = e.to_string();
            let status = if error_msg.contains("already running") || error_msg.contains("already starting") {
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
            ).into_response()
        }
    }
}

/// Stop an instance's browser
///
/// Stops the browser for a specific instance.
/// Requires manage access to the instance.
#[utoipa::path(
    post,
    path = "/api/v1/instances/{instance_id}/stop",
    tag = "Instances",
    params(
        ("instance_id" = String, Path, description = "Instance ID (UUID or phone number)")
    ),
    responses(
        (status = 200, description = "Instance stopped", body = AccountActionResponse),
        (status = 403, description = "Access denied"),
        (status = 404, description = "Instance not found"),
    )
)]
pub async fn stop_instance(
    State(state): State<InstancesState>,
    auth_user: Option<Extension<AuthenticatedUser>>,
    Path(instance_id): Path<String>,
) -> impl IntoResponse {
    let account = match state.manager.get_account(&instance_id).await {
        Some(acc) => acc,
        None => return (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "not_found",
                "message": format!("Instance '{}' not found", instance_id)
            })),
        ).into_response(),
    };
    
    let account_id = account.id;
    
    // Check manage permissions
    if !check_instance_manage_access(&state.db, &auth_user, &account_id.to_string()) {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "access_denied",
                "message": "You don't have permission to manage this instance"
            })),
        ).into_response();
    }
    
    match account.stop().await {
        Ok(()) => Json(json!(AccountActionResponse {
            message: "Instance stopped successfully".to_string(),
            account_id,
        })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "stop_failed",
                "message": e.to_string()
            })),
        ).into_response(),
    }
}

/// Discover existing instances
///
/// Scans the filesystem for existing instance directories and loads them.
/// Admin only operation.
#[utoipa::path(
    post,
    path = "/api/v1/instances/discover",
    tag = "Instances",
    responses(
        (status = 200, description = "Instances discovered"),
        (status = 403, description = "Admin access required"),
    )
)]
pub async fn discover_instances(
    State(state): State<InstancesState>,
    auth_user: Option<Extension<AuthenticatedUser>>,
) -> impl IntoResponse {
    // Discover is an admin-only operation
    let is_admin = match auth_user.as_ref().map(|e| &e.0) {
        Some(AuthenticatedUser::Secret) => true,
        Some(AuthenticatedUser::LocalUser { is_admin, .. }) => *is_admin,
        None => false,
    };
    
    if !is_admin {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "access_denied",
                "message": "Instance discovery requires admin access"
            })),
        ).into_response();
    }
    
    match state.manager.discover_accounts().await {
        Ok(discovered) => Json(json!({
            "message": format!("Discovered {} instances", discovered.len()),
            "discovered": discovered,
        })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "discover_failed",
                "message": e.to_string()
            })),
        ).into_response(),
    }
}

/// Get live screenshot of instance's browser
///
/// Returns a PNG screenshot of the current browser state.
/// Useful for monitoring and diagnosing WhatsApp Web connection issues.
/// Requires read access to the instance.
#[utoipa::path(
    get,
    path = "/api/v1/instances/{instance_id}/live",
    tag = "Instances",
    params(
        ("instance_id" = String, Path, description = "Instance ID (UUID or phone number)")
    ),
    responses(
        (status = 200, description = "PNG screenshot", content_type = "image/png"),
        (status = 403, description = "Access denied"),
        (status = 404, description = "Instance not found"),
        (status = 503, description = "Browser not running"),
    )
)]
pub async fn live_screenshot(
    State(state): State<InstancesState>,
    auth_user: Option<Extension<AuthenticatedUser>>,
    Path(instance_id): Path<String>,
) -> impl IntoResponse {
    let account = match state.manager.get_account(&instance_id).await {
        Some(acc) => acc,
        None => return (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "not_found",
                "message": format!("Instance '{}' not found", instance_id)
            })),
        ).into_response(),
    };
    
    // Check read access
    if !check_instance_access(&state.db, &auth_user, &account.id.to_string(), false) {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "access_denied",
                "message": "You don't have access to this instance"
            })),
        ).into_response();
    }
    
    if !account.browser_service().is_running().await {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "error": "browser_not_running",
                "message": "Browser is not running. Start the instance first via POST /api/v1/instances/{instance_id}/start"
            })),
        ).into_response();
    }
    
    match account.browser_service().screenshot().await {
        Ok(png_data) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "image/png")],
            png_data,
        ).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "screenshot_failed",
                "message": e.to_string()
            })),
        ).into_response(),
    }
}

/// Get instance configuration
///
/// Returns the current runtime configuration for an instance.
/// Requires read access to the instance.
#[utoipa::path(
    get,
    path = "/api/v1/instances/{instance_id}/config",
    tag = "Instances",
    params(
        ("instance_id" = String, Path, description = "Instance ID (UUID or phone number)")
    ),
    responses(
        (status = 200, description = "Instance configuration", body = InstanceConfig),
        (status = 403, description = "Access denied"),
        (status = 404, description = "Instance not found"),
    )
)]
pub async fn get_instance_config(
    State(state): State<InstancesState>,
    auth_user: Option<Extension<AuthenticatedUser>>,
    Path(instance_id): Path<String>,
) -> impl IntoResponse {
    let account = match state.manager.get_account(&instance_id).await {
        Some(acc) => acc,
        None => return (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "not_found",
                "message": format!("Instance '{}' not found", instance_id)
            })),
        ).into_response(),
    };
    
    // Check read access
    if !check_instance_access(&state.db, &auth_user, &account.id.to_string(), false) {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "access_denied",
                "message": "You don't have access to this instance"
            })),
        ).into_response();
    }
    
    let config = account.get_config().await;
    Json(config).into_response()
}

/// Update instance configuration
///
/// Updates the runtime configuration for an instance. Only provided fields are updated.
/// Some changes (like browser settings) may require restarting the instance to take effect.
/// Requires manage access to the instance.
#[utoipa::path(
    put,
    path = "/api/v1/instances/{instance_id}/config",
    tag = "Instances",
    params(
        ("instance_id" = String, Path, description = "Instance ID (UUID or phone number)")
    ),
    request_body = UpdateInstanceConfigRequest,
    responses(
        (status = 200, description = "Configuration updated", body = InstanceConfig),
        (status = 403, description = "Access denied"),
        (status = 404, description = "Instance not found"),
        (status = 400, description = "Invalid configuration"),
    )
)]
pub async fn update_instance_config(
    State(state): State<InstancesState>,
    auth_user: Option<Extension<AuthenticatedUser>>,
    Path(instance_id): Path<String>,
    Json(request): Json<UpdateInstanceConfigRequest>,
) -> impl IntoResponse {
    let account = match state.manager.get_account(&instance_id).await {
        Some(acc) => acc,
        None => return (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "not_found",
                "message": format!("Instance '{}' not found", instance_id)
            })),
        ).into_response(),
    };
    
    // Check manage permissions
    if !check_instance_manage_access(&state.db, &auth_user, &account.id.to_string()) {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "access_denied",
                "message": "You don't have permission to manage this instance"
            })),
        ).into_response();
    }
    
    match account.update_config(request).await {
        Ok(config) => Json(json!({
            "message": "Configuration updated successfully",
            "config": config,
            "restart_required": true
        })).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "config_update_failed",
                "message": e.to_string()
            })),
        ).into_response(),
    }
}

// =============================================================================
// Access Control Helpers
// =============================================================================

/// Check if user has access to an instance (read or delete)
///
/// Returns true if:
/// - User is authenticated via secret token (full access)
/// - User is an admin (full access)  
/// - User owns the instance
/// - User has been granted access to the instance
/// - require_owner is true: only owner or admin can access
fn check_instance_access(
    db: &Arc<DatabaseService>,
    auth_user: &Option<Extension<AuthenticatedUser>>,
    instance_id: &str,
    require_owner: bool,
) -> bool {
    match auth_user.as_ref().map(|e| &e.0) {
        // Secret token has full access
        Some(AuthenticatedUser::Secret) => true,
        
        // Check local user permissions
        Some(AuthenticatedUser::LocalUser { user_id, is_admin, .. }) => {
            // Admins have full access
            if *is_admin {
                return true;
            }
            
            // Check instance ownership/access
            match db.check_instance_access(instance_id, *user_id) {
                Ok(Some(perms)) => {
                    if require_owner {
                        perms.can_delete // Only owner has can_delete
                    } else {
                        perms.can_read
                    }
                }
                _ => false,
            }
        }
        
        // No auth context - deny
        None => false,
    }
}

/// Check if user has manage access to an instance (start, stop, config)
///
/// Returns true if:
/// - User is authenticated via secret token
/// - User is an admin
/// - User owns the instance
/// - User has been granted manage access (can_manage = true)
fn check_instance_manage_access(
    db: &Arc<DatabaseService>,
    auth_user: &Option<Extension<AuthenticatedUser>>,
    instance_id: &str,
) -> bool {
    match auth_user.as_ref().map(|e| &e.0) {
        // Secret token has full access
        Some(AuthenticatedUser::Secret) => true,
        
        // Check local user permissions
        Some(AuthenticatedUser::LocalUser { user_id, is_admin, .. }) => {
            // Admins have full access
            if *is_admin {
                return true;
            }
            
            // Check instance ownership/access
            match db.check_instance_access(instance_id, *user_id) {
                Ok(Some(perms)) => perms.can_manage,
                _ => false,
            }
        }
        
        // No auth context - deny
        None => false,
    }
}
