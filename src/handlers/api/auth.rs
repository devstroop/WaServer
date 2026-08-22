//! Authentication API
//!
//! REST API endpoints for user authentication (login, register).
//! These endpoints are public (no authentication required).

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
use uuid::Uuid;

use crate::{
    middleware::auth::{hash_password, hash_token, verify_password},
    models::user::{LoginRequest, LoginResponse, RegisterUserRequest, UserInfo, UserRole},
    services::Database,
};

/// Generate a session token (for web UI auth)
fn generate_session_token() -> String {
    format!("session_{}", Uuid::new_v4().to_string().replace("-", ""))
}

/// Register a new user
#[utoipa::path(
    post,
    path = "/api/v1/auth/register",
    tag = "Auth",
    request_body = RegisterUserRequest,
    responses(
        (status = 201, description = "User registered successfully", body = UserInfo),
        (status = 400, description = "Invalid request"),
        (status = 409, description = "Username or email already exists"),
    )
)]
pub async fn register(
    State(db): State<Database>,
    Json(request): Json<RegisterUserRequest>,
) -> impl IntoResponse {
    // Validate username
    if request.username.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "invalid_request",
                "message": "Username cannot be empty"
            })),
        )
            .into_response();
    }

    // Validate password
    if request.password.len() < 8 {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "invalid_request",
                "message": "Password must be at least 8 characters"
            })),
        )
            .into_response();
    }

    // Validate email if provided
    if let Some(ref email) = request.email {
        if !email.contains('@') || email.len() < 3 {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "invalid_request",
                    "message": "Invalid email format"
                })),
            )
                .into_response();
        }
    }

    let user_id = Uuid::new_v4().to_string();
    let password_hash = hash_password(&request.password);

    // New users get the User role by default
    match db.create_user(
        &user_id,
        &request.username,
        request.email.as_deref(),
        &password_hash,
        UserRole::User,
    ) {
        Ok(user_record) => {
            tracing::info!(
                user_id = %user_id,
                username = %request.username,
                "New user registered"
            );
            (StatusCode::CREATED, Json(UserInfo::from(user_record))).into_response()
        }
        Err(e) => {
            let error_msg = e.to_string();
            let status = if error_msg.contains("already exists") {
                StatusCode::CONFLICT
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            (
                status,
                Json(json!({
                    "error": "registration_failed",
                    "message": error_msg
                })),
            )
                .into_response()
        }
    }
}

/// Login with username/email and password
#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    tag = "Auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 401, description = "Invalid credentials"),
        (status = 403, description = "User is inactive"),
    )
)]
pub async fn login(
    State(db): State<Database>,
    Json(request): Json<LoginRequest>,
) -> impl IntoResponse {
    // Try to find user by username or email
    let user = if let Ok(Some(u)) = db.get_user_by_username(&request.username) {
        Some(u)
    } else if let Ok(Some(u)) = db.get_user_by_email(&request.username) {
        Some(u)
    } else {
        None
    };

    match user {
        Some(user_record) => {
            // Verify password
            if !verify_password(&request.password, &user_record.password_hash) {
                tracing::warn!(
                    username = %request.username,
                    "Login failed - invalid password"
                );
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({
                        "error": "invalid_credentials",
                        "message": "Invalid username/email or password"
                    })),
                )
                    .into_response();
            }

            // Check if user is active
            if !user_record.is_active {
                tracing::warn!(
                    user_id = %user_record.id,
                    username = %user_record.username,
                    "Login failed - user inactive"
                );
                return (
                    StatusCode::FORBIDDEN,
                    Json(json!({
                        "error": "user_inactive",
                        "message": "User account is inactive"
                    })),
                )
                    .into_response();
            }

            // Generate session token and store it as an access token
            let session_token = generate_session_token();
            let token_id = Uuid::new_v4().to_string();
            let token_hash = hash_token(&session_token);

            // Create a session token (ephemeral access token)
            if let Err(e) = db.create_access_token(
                &token_id,
                &user_record.id,
                "Web Session",
                &token_hash,
                None, // Session tokens don't expire (for now)
            ) {
                tracing::error!(
                    user_id = %user_record.id,
                    error = %e,
                    "Failed to create session token"
                );
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "error": "login_failed",
                        "message": "Failed to create session"
                    })),
                )
                    .into_response();
            }

            tracing::info!(
                user_id = %user_record.id,
                username = %user_record.username,
                "User logged in successfully"
            );

            // Session tokens don't expire for now
            let expires_at = "never".to_string();

            (
                StatusCode::OK,
                Json(LoginResponse {
                    user: UserInfo::from(user_record),
                    token: session_token,
                    expires_at,
                }),
            )
                .into_response()
        }
        None => {
            tracing::warn!(
                username = %request.username,
                "Login failed - user not found"
            );
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "error": "invalid_credentials",
                    "message": "Invalid username/email or password"
                })),
            )
                .into_response()
        }
    }
}

/// Validate current session (check if token is valid)
#[utoipa::path(
    get,
    path = "/api/v1/auth/validate",
    tag = "Auth",
    responses(
        (status = 200, description = "Token is valid", body = UserInfo),
        (status = 401, description = "Invalid or expired token"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn validate(
    State(db): State<Database>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    // Extract the Authorization header
    let auth_header = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok());

    if let Some(auth_value) = auth_header {
        if let Some(token) = auth_value.strip_prefix("Bearer ") {
            let token_hash = hash_token(token);

            if let Ok(Some((user, _))) = db.get_user_by_access_token(&token_hash) {
                if user.is_active {
                    return (StatusCode::OK, Json(UserInfo::from(user))).into_response();
                }
            }
        }
    }

    (
        StatusCode::UNAUTHORIZED,
        Json(json!({
            "error": "invalid_token",
            "message": "Invalid or expired token"
        })),
    )
        .into_response()
}

/// Logout (invalidate session token)
#[utoipa::path(
    post,
    path = "/api/v1/auth/logout",
    tag = "Auth",
    responses(
        (status = 200, description = "Logged out successfully"),
        (status = 401, description = "Invalid token"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn logout(
    State(db): State<Database>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    // Extract the Authorization header
    let auth_header = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok());

    if let Some(auth_value) = auth_header {
        if let Some(token) = auth_value.strip_prefix("Bearer ") {
            let token_hash = hash_token(token);

            // Find and delete the session token
            if let Ok(Some((user, token_record))) = db.get_user_by_access_token(&token_hash) {
                if let Err(e) = db.delete_access_token(&token_record.id, &user.id) {
                    tracing::error!(
                        user_id = %user.id,
                        error = %e,
                        "Failed to delete session token during logout"
                    );
                } else {
                    tracing::info!(
                        user_id = %user.id,
                        username = %user.username,
                        "User logged out successfully"
                    );
                }

                return (
                    StatusCode::OK,
                    Json(json!({
                        "message": "Logged out successfully"
                    })),
                )
                    .into_response();
            }
        }
    }

    (
        StatusCode::UNAUTHORIZED,
        Json(json!({
            "error": "invalid_token",
            "message": "Invalid or expired token"
        })),
    )
        .into_response()
}
