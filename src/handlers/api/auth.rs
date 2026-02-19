//! Authentication Handlers - Now uses AccountManager with X-Account-Id
//!
//! WhatsApp auth routes require X-Account-Id header.
//! Local auth routes (JWT) use shared AppConfig.

use crate::{
    config::AppConfig,
    middleware::CurrentAccount,
    models::auth::{
        AuthStatusResponse, LocalAuthStatusResponse, LoginRequest, LoginResponse,
        PhoneAuthResponse, PhoneLoginRequest, QrCodeResponse, RefreshTokenRequest,
        RefreshTokenResponse, SuccessResponse,
    },
    models::chat::ErrorResponse,
};
use axum::{extract::State, http::StatusCode, response::Json, Extension};
use std::sync::Arc;
use tracing::{error, info, warn};
use utoipa;

// =============================================================================
// WhatsApp Authentication Endpoints (require X-Account-Id header)
// =============================================================================

/// Get authentication status
#[utoipa::path(
    get,
    path = "/api/v1/auth/status",
    responses(
        (status = 200, description = "Authentication status retrieved successfully", body = AuthStatusResponse),
        (status = 400, description = "Missing X-Account-Id header", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "WhatsAppAccount"
)]
pub async fn get_auth_status(
    Extension(current): Extension<CurrentAccount>,
) -> Result<Json<AuthStatusResponse>, (StatusCode, Json<ErrorResponse>)> {
    let account = current.0;

    match account.get_auth_status().await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            error!("Error checking auth status: {}", e);

            // If browser is not initialized, return a checking response
            if e.to_string().contains("Browser not initialized")
                || e.to_string().contains("not running")
            {
                Ok(Json(AuthStatusResponse {
                    authenticated: false,
                    status: "browser_not_running".to_string(),
                    phone_number: None,
                }))
            } else {
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: format!("Error checking WhatsApp authorization status: {}", e),
                    }),
                ))
            }
        }
    }
}

/// Get QR code for authentication
#[utoipa::path(
    get,
    path = "/api/v1/auth/qr",
    responses(
        (status = 200, description = "QR code retrieved successfully", body = QrCodeResponse),
        (status = 400, description = "Bad request - already authorized", body = ErrorResponse),
        (status = 503, description = "Browser not running", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "WhatsAppAccount"
)]
pub async fn get_qr_code(
    Extension(current): Extension<CurrentAccount>,
) -> Result<Json<QrCodeResponse>, (StatusCode, Json<ErrorResponse>)> {
    let account = current.0;

    // Check if browser is running
    if !account.browser_service().is_running().await {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse {
                error: "Browser not running. Start account first via POST /api/v1/accounts/{id}/start".to_string(),
            }),
        ));
    }

    match account
        .execute_with_busy_flag(async { account.auth_service().get_auth_qr_code().await })
        .await
    {
        Ok(qr_code) => {
            info!("QR code generated successfully for account {}", account.id);
            Ok(Json(QrCodeResponse { qrcode: qr_code }))
        }
        Err(e) => {
            let error_msg = e.to_string();
            error!("Error getting QR code: {}", error_msg);

            if error_msg.contains("Already authorized") {
                Err((
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse { error: error_msg }),
                ))
            } else if error_msg.contains("busy") {
                Err((
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(ErrorResponse {
                        error: "Account is busy with another operation".to_string(),
                    }),
                ))
            } else {
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: format!("Error getting WhatsApp QR code: {}", error_msg),
                    }),
                ))
            }
        }
    }
}

/// Authenticate with phone number
#[utoipa::path(
    post,
    path = "/api/v1/auth/phone",
    request_body = PhoneLoginRequest,
    responses(
        (status = 200, description = "Phone authentication initiated", body = PhoneAuthResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 503, description = "Browser not running", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "WhatsAppAccount"
)]
pub async fn login_with_phone(
    Extension(current): Extension<CurrentAccount>,
    Json(request): Json<PhoneLoginRequest>,
) -> Result<Json<PhoneAuthResponse>, (StatusCode, Json<ErrorResponse>)> {
    let account = current.0;
    let phone_number = request.phone;

    // Check if browser is running
    if !account.browser_service().is_running().await {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse {
                error: "Browser not running. Start account first.".to_string(),
            }),
        ));
    }

    match account
        .execute_with_busy_flag(async {
            account
                .auth_service()
                .login_with_phone_number(&phone_number)
                .await
        })
        .await
    {
        Ok(code) => {
            let formatted_code = code.map(|c| c.replace(",", ""));
            info!(
                "Phone authentication initiated for account {}: {}",
                account.id, phone_number
            );
            Ok(Json(PhoneAuthResponse {
                code: formatted_code,
            }))
        }
        Err(e) => {
            let error_msg = e.to_string();
            error!("Error with phone authentication: {}", error_msg);

            if error_msg.contains("Already authorized") {
                Err((
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse { error: error_msg }),
                ))
            } else if error_msg.contains("busy") {
                Err((
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(ErrorResponse {
                        error: "Account is busy with another operation".to_string(),
                    }),
                ))
            } else {
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: format!("Error logging in to WhatsApp: {}", error_msg),
                    }),
                ))
            }
        }
    }
}

/// Logout from WhatsApp Web
#[utoipa::path(
    post,
    path = "/api/v1/auth/logout",
    responses(
        (status = 200, description = "Logged out successfully", body = SuccessResponse),
        (status = 400, description = "Bad request - not authorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "WhatsAppAccount"
)]
pub async fn logout(
    Extension(current): Extension<CurrentAccount>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<ErrorResponse>)> {
    let account = current.0;

    match account
        .execute_with_busy_flag(async { account.auth_service().logout().await })
        .await
    {
        Ok(_) => {
            account.invalidate_auth_cache().await;
            info!("Account {} logged out successfully", account.id);
            Ok(Json(SuccessResponse {
                message: "Logged out successfully".to_string(),
            }))
        }
        Err(e) => {
            let error_msg = e.to_string();
            error!("Error during logout: {}", error_msg);

            if error_msg.contains("Not authorized") {
                Err((
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse { error: error_msg }),
                ))
            } else {
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: format!("Error logging out of WhatsApp: {}", error_msg),
                    }),
                ))
            }
        }
    }
}

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
    path = "/api/v1/auth/local-status",
    responses(
        (status = 200, description = "Local auth status retrieved successfully", body = LocalAuthStatusResponse)
    ),
    tag = "Authentication"
)]
pub async fn get_local_auth_status(
    State(state): State<LocalAuthState>,
) -> Json<LocalAuthStatusResponse> {
    Json(LocalAuthStatusResponse {
        local_auth_enabled: state.config.local_auth.enabled,
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
        (status = 400, description = "Local auth not enabled", body = ErrorResponse),
        (status = 401, description = "Invalid credentials", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "Authentication"
)]
pub async fn local_login(
    State(state): State<LocalAuthState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Check if local auth is enabled
    if !state.config.local_auth.enabled {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Local authentication is not enabled".to_string(),
            }),
        ));
    }

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
        (status = 400, description = "Local auth not enabled", body = ErrorResponse),
        (status = 401, description = "Invalid or expired refresh token", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "Authentication"
)]
pub async fn refresh_token(
    State(state): State<LocalAuthState>,
    Json(request): Json<RefreshTokenRequest>,
) -> Result<Json<RefreshTokenResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Check if local auth is enabled
    if !state.config.local_auth.enabled {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Local authentication is not enabled".to_string(),
            }),
        ));
    }

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
    path = "/api/v1/auth/local-logout",
    request_body = RefreshTokenRequest,
    responses(
        (status = 200, description = "Logged out successfully", body = SuccessResponse),
        (status = 400, description = "Local auth not enabled", body = ErrorResponse)
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
    // Check if local auth is enabled
    if !state.config.local_auth.enabled {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Local authentication is not enabled".to_string(),
            }),
        ));
    }

    // Get auth token service
    if let Some(auth_token_service) = state.auth_token_service.as_ref() {
        auth_token_service.logout(&request.refresh_token);
        info!("User logged out (token revoked)");
    }

    Ok(Json(SuccessResponse {
        message: "Logged out successfully".to_string(),
    }))
}
