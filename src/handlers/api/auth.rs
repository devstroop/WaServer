//! Authentication API
//!
//! REST API endpoints for user authentication (login, register).
//! These endpoints are public (no authentication required).

use axum::{
    extract::{ConnectInfo, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use std::net::SocketAddr;
use uuid::Uuid;

use crate::{
    middleware::auth::{hash_password, hash_token, is_bcrypt_hash, verify_password, AuthState},
    models::user::{LoginRequest, LoginResponse, RegisterUserRequest, UserInfo, UserRole},
};

/// Generate a session token (API bearer token)
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
    State(auth): State<AuthState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    Json(request): Json<RegisterUserRequest>,
) -> impl IntoResponse {
    // Anti account-spam: every registration attempt counts against the IP (#44)
    let reg_key = format!("reg|{}", peer.ip());
    if let Err(remaining) = auth.throttle.hit(&reg_key) {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            [("retry-after", remaining.as_secs().to_string())],
            Json(json!({
                "error": "rate_limited",
                "message": format!("Too many attempts — retry in {}s", remaining.as_secs())
            })),
        )
            .into_response();
    }

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

    // Bootstrap: the FIRST registered user becomes admin so a fresh install
    // has an administrator even with the static secret key disabled (#41)
    let first_user = auth.db.list_users().map(|u| u.is_empty()).unwrap_or(false);
    let role = if first_user {
        UserRole::Admin
    } else {
        UserRole::User
    };

    match auth.db.create_user(
        &user_id,
        &request.username,
        request.email.as_deref(),
        &password_hash,
        role,
    ) {
        Ok(user_record) => {
            tracing::info!(
                user_id = %user_id,
                username = %request.username,
                is_admin = first_user,
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
    State(auth): State<AuthState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    Json(request): Json<LoginRequest>,
) -> impl IntoResponse {
    // Brute-force protection: failures per (IP|username) within window (#44)
    let throttle_key = format!("login|{}|{}", peer.ip(), request.username.to_lowercase());
    if let Some(remaining) = auth.throttle.is_blocked(&throttle_key) {
        tracing::warn!(%peer, "login throttled");
        return (
            StatusCode::TOO_MANY_REQUESTS,
            [("retry-after", remaining.as_secs().to_string())],
            Json(json!({
                "error": "rate_limited",
                "message": format!("Too many failed attempts — retry in {}s", remaining.as_secs())
            })),
        )
            .into_response();
    }

    // Try to find user by username or email
    let user = if let Ok(Some(u)) = auth.db.get_user_by_username(&request.username) {
        Some(u)
    } else if let Ok(Some(u)) = auth.db.get_user_by_email(&request.username) {
        Some(u)
    } else {
        None
    };

    match user {
        Some(user_record) => {
            // Verify password
            if !verify_password(&request.password, &user_record.password_hash) {
                auth.throttle.hit(&throttle_key).ok();
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
                auth.throttle.hit(&throttle_key).ok();
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

            // Migration: legacy SHA256 hash → bcrypt (transparent on successful login)
            if !is_bcrypt_hash(&user_record.password_hash) {
                let new_hash = hash_password(&request.password);
                if let Err(e) = auth
                    .db
                    .update_user(&user_record.id, None, None, Some(&new_hash), None, None)
                {
                    tracing::warn!(user_id=%user_record.id, error=%e, "Failed to migrate legacy password hash to bcrypt");
                } else {
                    tracing::info!(user_id=%user_record.id, "Migrated legacy SHA256 password hash to bcrypt");
                }
            }

            // Generate session token and store it as an access token
            let session_token = generate_session_token();
            let token_id = Uuid::new_v4().to_string();
            let token_hash = hash_token(&session_token);

            // Create a session token (ephemeral access token)
            let expires_at = auth.session_expiry();
            if let Err(e) = auth.db.create_access_token(
                &token_id,
                &user_record.id,
                "Web Session",
                &token_hash,
                Some(&expires_at),
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

            let expires_at = format!("{} (UTC)", expires_at);

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
            auth.throttle.hit(&throttle_key).ok();
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
    State(auth): State<AuthState>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    // Extract the Authorization header
    let auth_header = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok());

    if let Some(auth_value) = auth_header {
        if let Some(token) = auth_value.strip_prefix("Bearer ") {
            let token_hash = hash_token(token);

            if let Ok(Some((user, _))) = auth.db.get_user_by_access_token(&token_hash) {
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
    State(auth): State<AuthState>,
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
            if let Ok(Some((user, token_record))) = auth.db.get_user_by_access_token(&token_hash) {
                if let Err(e) = auth.db.delete_access_token(&token_record.id, &user.id) {
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

/// Revoke all sessions for the authenticated user (logout-all, #42)
#[utoipa::path(
    post,
    path = "/api/v1/auth/logout-all",
    tag = "Auth",
    responses(
        (status = 200, description = "All sessions revoked"),
        (status = 401, description = "Invalid token"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn logout_all(
    State(auth): State<AuthState>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let auth_header = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok());

    if let Some(auth_value) = auth_header {
        if let Some(token) = auth_value.strip_prefix("Bearer ") {
            let token_hash = hash_token(token);
            if let Ok(Some((user, _))) = auth.db.get_user_by_access_token(&token_hash) {
                let revoked = auth.db.delete_user_web_sessions(&user.id).unwrap_or(0);
                tracing::info!(user_id = %user.id, revoked, "revoked all sessions");
                return (
                    StatusCode::OK,
                    Json(json!({
                        "message": "All sessions revoked",
                        "revoked": revoked
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
