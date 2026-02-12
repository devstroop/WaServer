use crate::{
    models::auth::{
        AuthStatusResponse, ErrorResponse, PhoneAuthResponse, QrCodeResponse, SuccessResponse,
    },
    services::whatsapp::WhatsAppService,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use std::sync::Arc;
use tracing::{error, info};
use utoipa;

/// Get authentication status
#[utoipa::path(
    get,
    path = "/api/auth/status",
    responses(
        (status = 200, description = "Authentication status retrieved successfully", body = AuthStatusResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Authentication"
)]
pub async fn get_auth_status(
    State(whatsapp_service): State<Arc<WhatsAppService>>,
) -> Result<Json<AuthStatusResponse>, (StatusCode, Json<ErrorResponse>)> {
    match whatsapp_service.auth_service().is_authorized().await {
        Ok(authorized) => {
            if authorized {
                // Try to get sender ID
                match whatsapp_service.auth_service().get_sender_id().await {
                    Ok(sender_id) => Ok(Json(AuthStatusResponse {
                        authorized,
                        sender_id,
                    })),
                    Err(e) => {
                        error!("Error getting sender ID: {}", e);
                        Ok(Json(AuthStatusResponse {
                            authorized,
                            sender_id: None,
                        }))
                    }
                }
            } else {
                Ok(Json(AuthStatusResponse {
                    authorized,
                    sender_id: None,
                }))
            }
        }
        Err(e) => {
            error!("Error checking auth status: {}", e);

            // If browser is not initialized, return a specific response
            if e.to_string().contains("Browser not initialized") {
                Ok(Json(AuthStatusResponse {
                    authorized: false,
                    sender_id: None,
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
    path = "/api/auth/qrcode",
    responses(
        (status = 200, description = "QR code retrieved successfully", body = QrCodeResponse),
        (status = 400, description = "Bad request - already authorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Authentication"
)]
pub async fn get_qr_code(
    State(whatsapp_service): State<Arc<WhatsAppService>>,
) -> Result<Json<QrCodeResponse>, (StatusCode, Json<ErrorResponse>)> {
    match whatsapp_service
        .execute_with_busy_flag(async { whatsapp_service.auth_service().get_auth_qr_code().await })
        .await
    {
        Ok(qr_code) => {
            info!("QR code generated successfully");
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
            } else if error_msg.contains("Browser not initialized") {
                Err((
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(ErrorResponse {
                        error: "Browser service is not available. Cannot generate QR code."
                            .to_string(),
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
    path = "/api/auth/phone/{phone_number}",
    params(
        ("phone_number" = String, Path, description = "Phone number for authentication")
    ),
    responses(
        (status = 200, description = "Phone authentication initiated", body = PhoneAuthResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Authentication"
)]
pub async fn login_with_phone(
    Path(phone_number): Path<String>,
    State(whatsapp_service): State<Arc<WhatsAppService>>,
) -> Result<Json<PhoneAuthResponse>, (StatusCode, Json<ErrorResponse>)> {
    match whatsapp_service
        .execute_with_busy_flag(async {
            whatsapp_service
                .auth_service()
                .login_with_phone_number(&phone_number)
                .await
        })
        .await
    {
        Ok(code) => {
            let formatted_code = code.map(|c| c.replace(",", ""));
            info!("Phone authentication initiated for: {}", phone_number);
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
    path = "/api/auth/logout",
    responses(
        (status = 200, description = "Logged out successfully", body = SuccessResponse),
        (status = 400, description = "Bad request - not authorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Authentication"
)]
pub async fn logout(
    State(whatsapp_service): State<Arc<WhatsAppService>>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<ErrorResponse>)> {
    match whatsapp_service
        .execute_with_busy_flag(async { whatsapp_service.auth_service().logout().await })
        .await
    {
        Ok(_) => {
            info!("User logged out successfully");
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
