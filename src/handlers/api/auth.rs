//! Local Authentication Handlers (JWT-based)
//!
//! Handles server-level authentication (login, logout, token refresh, initial setup).
//! WhatsApp-specific auth operations are in account.rs under /api/v1/account/*.

use crate::{
    config::AppConfig,
    models::auth::{
        LocalAuthStatusResponse, LoginRequest, LoginResponse,
        RefreshTokenRequest, RefreshTokenResponse, SetupRequest, 
        SetupStatusResponse, SuccessResponse,
    },
    models::chat::ErrorResponse,
};
use axum::{extract::State, http::StatusCode, response::Json};
use std::sync::Arc;
use tracing::{error, info, warn};

// =============================================================================
// Local Authentication Endpoints (JWT-based, no X-Account-Id needed)
// =============================================================================

/// State for local auth routes
#[derive(Clone)]
pub struct LocalAuthState {
    pub config: Arc<AppConfig>,
    pub auth_token_service: Option<Arc<crate::services::AuthTokenService>>,
}

/// Get local authentication status
#[utoipa::path(
    get,
    path = "/api/v1/admin/auth/status",
    responses(
        (status = 200, description = "Local auth status retrieved successfully", body = LocalAuthStatusResponse)
    ),
    tag = "Authentication"
)]
pub async fn get_local_auth_status(
    State(_state): State<LocalAuthState>,
) -> Json<LocalAuthStatusResponse> {
    Json(LocalAuthStatusResponse {
        logged_in: false, // Determined by frontend based on having valid token
        username: None,
    })
}

/// Login with username and password
#[utoipa::path(
    post,
    path = "/api/v1/admin/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 401, description = "Invalid credentials", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "Authentication"
)]
pub async fn local_login(
    State(state): State<LocalAuthState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Get auth token service
    let auth_token_service = state.auth_token_service.as_ref().ok_or((
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
            error: "Auth token service not available".to_string(),
        }),
    ))?;

    // Attempt login
    match auth_token_service.login(&request.username, &request.password) {
        Ok(response) => {
            info!("User '{}' logged in successfully", request.username);
            Ok(Json(response))
        }
        Err(e) => {
            warn!("Login failed for user '{}': {}", request.username, e);
            Err((
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Invalid username or password".to_string(),
                }),
            ))
        }
    }
}

/// Refresh access token
#[utoipa::path(
    post,
    path = "/api/v1/admin/auth/refresh",
    request_body = RefreshTokenRequest,
    responses(
        (status = 200, description = "Token refreshed successfully", body = RefreshTokenResponse),
        (status = 401, description = "Invalid or expired refresh token", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "Authentication"
)]
pub async fn refresh_token(
    State(state): State<LocalAuthState>,
    Json(request): Json<RefreshTokenRequest>,
) -> Result<Json<RefreshTokenResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Get auth token service
    let auth_token_service = state.auth_token_service.as_ref().ok_or((
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
            error: "Auth token service not available".to_string(),
        }),
    ))?;

    // Attempt to refresh token
    match auth_token_service.refresh_token(&request.refresh_token) {
        Ok(response) => {
            info!("Token refreshed successfully");
            Ok(Json(response))
        }
        Err(e) => {
            warn!("Token refresh failed: {}", e);
            Err((
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Invalid or expired refresh token".to_string(),
                }),
            ))
        }
    }
}

/// Logout from local auth (revoke refresh token)
#[utoipa::path(
    post,
    path = "/api/v1/admin/auth/logout",
    request_body = RefreshTokenRequest,
    responses(
        (status = 200, description = "Logged out successfully", body = SuccessResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Authentication"
)]
pub async fn local_logout(
    State(state): State<LocalAuthState>,
    Json(request): Json<RefreshTokenRequest>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Get auth token service
    if let Some(auth_token_service) = state.auth_token_service.as_ref() {
        auth_token_service.logout(&request.refresh_token);
        info!("User logged out (token revoked)");
    }

    Ok(Json(SuccessResponse {
        message: "Logged out successfully".to_string(),
    }))
}

// =============================================================================
// Initial Setup Endpoints (no auth required)
// =============================================================================

/// Check if initial setup is required
#[utoipa::path(
    get,
    path = "/api/v1/admin/auth/setup",
    responses(
        (status = 200, description = "Setup status retrieved", body = SetupStatusResponse)
    ),
    tag = "Authentication"
)]
pub async fn get_setup_status(
    State(state): State<LocalAuthState>,
) -> Json<SetupStatusResponse> {
    let needs_setup = state
        .auth_token_service
        .as_ref()
        .map(|s| s.needs_setup())
        .unwrap_or(true);
    
    let message = if needs_setup {
        "Initial setup required. Create your admin account using the setup token from the server console.".to_string()
    } else {
        "Setup complete. Please login.".to_string()
    };
    
    Json(SetupStatusResponse {
        needs_setup,
        message,
    })
}

/// Complete initial setup - create first admin user
#[utoipa::path(
    post,
    path = "/api/v1/admin/auth/setup",
    request_body = SetupRequest,
    responses(
        (status = 200, description = "Setup completed successfully, user logged in", body = LoginResponse),
        (status = 400, description = "Invalid setup token or validation failed", body = ErrorResponse),
        (status = 409, description = "Setup already completed", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "Authentication"
)]
pub async fn complete_setup(
    State(state): State<LocalAuthState>,
    Json(request): Json<SetupRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Get auth token service
    let auth_token_service = state.auth_token_service.as_ref().ok_or((
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
            error: "Auth token service not available".to_string(),
        }),
    ))?;
    
    // Attempt to complete setup
    match auth_token_service.complete_setup(&request.setup_token, &request.username, &request.password) {
        Ok(()) => {
            info!("Initial setup completed - admin user '{}' created", request.username);
            // Auto-login the new user after successful setup
            match auth_token_service.login(&request.username, &request.password) {
                Ok(login_response) => Ok(Json(login_response)),
                Err(e) => {
                    error!("Login failed after setup: {}", e);
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse {
                            error: "Setup succeeded but auto-login failed".to_string(),
                        }),
                    ))
                }
            }
        }
        Err(crate::models::error::AuthError::SetupAlreadyComplete) => {
            Err((
                StatusCode::CONFLICT,
                Json(ErrorResponse {
                    error: "Initial setup has already been completed. Please login instead.".to_string(),
                }),
            ))
        }
        Err(crate::models::error::AuthError::InvalidToken) => {
            warn!("Setup attempted with invalid token");
            Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "Invalid setup token. Check the server console for the correct token.".to_string(),
                }),
            ))
        }
        Err(crate::models::error::AuthError::ValidationFailed(msg)) => {
            Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: msg,
                }),
            ))
        }
        Err(e) => {
            error!("Setup failed: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Setup failed: {}", e),
                }),
            ))
        }
    }
}
