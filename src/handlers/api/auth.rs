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
        ForgotPasswordRequest, ForgotPasswordResponse,
        ResetPasswordRequest, ResetPasswordResponse,
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
    path = "/api/v1/auth/current-user",
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
    path = "/api/v1/auth/login",
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
    path = "/api/v1/auth/refresh",
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
    delete,
    path = "/api/v1/auth/logout",
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
    path = "/api/v1/auth/setup",
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
    path = "/api/v1/auth/setup",
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

// =============================================================================
// Password Reset Endpoints
// =============================================================================

/// Request password reset token
///
/// Generates a password reset token for the specified username.
/// In production, this token would be sent via email. For local/dev use,
/// the token is returned in the response.
#[utoipa::path(
    post,
    path = "/api/v1/auth/forgot-password",
    request_body = ForgotPasswordRequest,
    responses(
        (status = 200, description = "Reset token generated (if user exists)", body = ForgotPasswordResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "Authentication"
)]
pub async fn forgot_password(
    State(state): State<LocalAuthState>,
    Json(request): Json<ForgotPasswordRequest>,
) -> Result<Json<ForgotPasswordResponse>, (StatusCode, Json<ErrorResponse>)> {
    let auth_token_service = state.auth_token_service.as_ref().ok_or((
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
            error: "Auth token service not available".to_string(),
        }),
    ))?;

    match auth_token_service.forgot_password(&request.username) {
        Ok(response) => {
            // Log but don't reveal whether user exists
            info!("Password reset requested for username: {}", request.username);
            Ok(Json(response))
        }
        Err(e) => {
            error!("Forgot password failed: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to process password reset request".to_string(),
                }),
            ))
        }
    }
}

/// Reset password using token
///
/// Resets the user's password using a valid reset token from forgot-password.
#[utoipa::path(
    post,
    path = "/api/v1/auth/reset-password",
    request_body = ResetPasswordRequest,
    responses(
        (status = 200, description = "Password reset successfully", body = ResetPasswordResponse),
        (status = 400, description = "Invalid token or validation failed", body = ErrorResponse),
        (status = 401, description = "Token expired", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "Authentication"
)]
pub async fn reset_password(
    State(state): State<LocalAuthState>,
    Json(request): Json<ResetPasswordRequest>,
) -> Result<Json<ResetPasswordResponse>, (StatusCode, Json<ErrorResponse>)> {
    let auth_token_service = state.auth_token_service.as_ref().ok_or((
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
            error: "Auth token service not available".to_string(),
        }),
    ))?;

    match auth_token_service.reset_password(&request.reset_token, &request.new_password) {
        Ok(response) => {
            info!("Password reset completed successfully");
            Ok(Json(response))
        }
        Err(crate::models::error::AuthError::InvalidToken) => {
            warn!("Password reset attempted with invalid token");
            Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "Invalid or already used reset token".to_string(),
                }),
            ))
        }
        Err(crate::models::error::AuthError::TokenExpired) => {
            warn!("Password reset attempted with expired token");
            Err((
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Reset token has expired. Please request a new one.".to_string(),
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
            error!("Password reset failed: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to reset password".to_string(),
                }),
            ))
        }
    }
}

/// Change password (when logged in)
///
/// Changes the password for the currently authenticated user.
/// Requires the current password for verification.
#[utoipa::path(
    post,
    path = "/api/v1/auth/change-password",
    request_body = ChangePasswordRequest,
    responses(
        (status = 200, description = "Password changed successfully", body = ResetPasswordResponse),
        (status = 400, description = "Validation failed", body = ErrorResponse),
        (status = 401, description = "Invalid current password", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Authentication"
)]
pub async fn change_password(
    State(state): State<LocalAuthState>,
    // In a real implementation, you'd extract the username from the JWT token
    // For now, we require it in the request or extract from auth middleware
    Json(request): Json<ChangePasswordWithUsernameRequest>,
) -> Result<Json<ResetPasswordResponse>, (StatusCode, Json<ErrorResponse>)> {
    let auth_token_service = state.auth_token_service.as_ref().ok_or((
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
            error: "Auth token service not available".to_string(),
        }),
    ))?;

    match auth_token_service.change_password(&request.username, &request.current_password, &request.new_password) {
        Ok(response) => {
            info!("Password changed successfully for user '{}'", request.username);
            Ok(Json(response))
        }
        Err(crate::models::error::AuthError::InvalidCredentials) => {
            warn!("Password change failed: invalid current password for '{}'", request.username);
            Err((
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Current password is incorrect".to_string(),
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
            error!("Password change failed: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to change password".to_string(),
                }),
            ))
        }
    }
}

/// Internal request type that includes username (extracted from JWT in middleware)
#[derive(Debug, serde::Deserialize)]
pub struct ChangePasswordWithUsernameRequest {
    pub username: String,
    pub current_password: String,
    pub new_password: String,
}
