//! Users Management API
//!
//! REST API endpoints for managing users (create, list, get, update, delete).
//! Only admin users and superadmin can manage users.
//! All endpoints require authentication.

use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use uuid::Uuid;

use crate::{
    middleware::auth::{hash_password, hash_token},
    models::{
        auth::AuthenticatedUser,
        user::{
            AccessTokenInfo, AssignInstanceRequest, CreateAccessTokenRequest,
            CreateAccessTokenResponse, CreateUserRequest, CreateUserResponse,
            ListAccessTokensResponse, ListUsersResponse, UpdateUserRequest, UserInfo,
            UserInstancesResponse,
        },
    },
    services::Database,
};

/// Generate a new access token (random UUID-based)
fn generate_access_token() -> String {
    format!("was_{}", Uuid::new_v4().to_string().replace("-", ""))
}

// === User Management Handlers ===

/// List all users (admin only)
#[utoipa::path(
    get,
    path = "/api/v1/users",
    tag = "Users",
    responses(
        (status = 200, description = "List of users", body = ListUsersResponse),
        (status = 403, description = "Admin access required"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_users(
    State(db): State<Database>,
    Extension(auth_user): Extension<AuthenticatedUser>,
) -> impl IntoResponse {
    // Check admin access
    if !auth_user.is_admin() {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "forbidden",
                "message": "Admin access required"
            })),
        )
            .into_response();
    }

    match db.list_users() {
        Ok(users) => {
            let user_infos: Vec<UserInfo> = users.into_iter().map(UserInfo::from).collect();
            let total = user_infos.len();
            (
                StatusCode::OK,
                Json(ListUsersResponse {
                    users: user_infos,
                    total,
                }),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "list_failed",
                "message": e.to_string()
            })),
        )
            .into_response(),
    }
}

/// Create a new user (admin only)
#[utoipa::path(
    post,
    path = "/api/v1/users",
    tag = "Users",
    request_body = CreateUserRequest,
    responses(
        (status = 201, description = "User created", body = CreateUserResponse),
        (status = 400, description = "Invalid request"),
        (status = 403, description = "Admin access required"),
        (status = 409, description = "Username already exists"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_user(
    State(db): State<Database>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    Json(request): Json<CreateUserRequest>,
) -> impl IntoResponse {
    // Check admin access
    if !auth_user.is_admin() {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "forbidden",
                "message": "Admin access required"
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

    let user_id = Uuid::new_v4().to_string();
    let password_hash = hash_password(&request.password);

    match db.create_user(
        &user_id,
        &request.username,
        request.email.as_deref(),
        &password_hash,
        request.role,
    ) {
        Ok(user_record) => {
            let response = CreateUserResponse {
                user: UserInfo::from(user_record),
            };
            (StatusCode::CREATED, Json(response)).into_response()
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
                    "error": "create_failed",
                    "message": error_msg
                })),
            )
                .into_response()
        }
    }
}

/// Get a user by ID (admin only)
#[utoipa::path(
    get,
    path = "/api/v1/users/{user_id}",
    tag = "Users",
    params(
        ("user_id" = String, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User details", body = UserInfo),
        (status = 403, description = "Admin access required"),
        (status = 404, description = "User not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_user(
    State(db): State<Database>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(user_id): Path<String>,
) -> impl IntoResponse {
    // Check admin access or self-access
    let is_self = auth_user.user_id() == Some(&user_id);
    if !auth_user.is_admin() && !is_self {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "forbidden",
                "message": "Admin access required"
            })),
        )
            .into_response();
    }

    match db.get_user(&user_id) {
        Ok(Some(user)) => (StatusCode::OK, Json(UserInfo::from(user))).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "not_found",
                "message": "User not found"
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "get_failed",
                "message": e.to_string()
            })),
        )
            .into_response(),
    }
}

/// Update a user (admin only)
#[utoipa::path(
    patch,
    path = "/api/v1/users/{user_id}",
    tag = "Users",
    params(
        ("user_id" = String, Path, description = "User ID")
    ),
    request_body = UpdateUserRequest,
    responses(
        (status = 200, description = "User updated", body = UserInfo),
        (status = 403, description = "Admin access required"),
        (status = 404, description = "User not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_user(
    State(db): State<Database>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(user_id): Path<String>,
    Json(request): Json<UpdateUserRequest>,
) -> impl IntoResponse {
    // Check admin access
    if !auth_user.is_admin() {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "forbidden",
                "message": "Admin access required"
            })),
        )
            .into_response();
    }

    // Check user exists
    match db.get_user(&user_id) {
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "not_found",
                    "message": "User not found"
                })),
            )
                .into_response()
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "get_failed",
                    "message": e.to_string()
                })),
            )
                .into_response()
        }
        Ok(Some(_)) => {}
    }

    // Apply update
    if let Err(e) = db.update_user(
        &user_id,
        request.username.as_deref(),
        None, // email not updated in this endpoint
        None, // password_hash not updated in this endpoint
        request.role,
        request.is_active,
    ) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "update_failed",
                "message": e.to_string()
            })),
        )
            .into_response();
    }

    // Return updated user
    match db.get_user(&user_id) {
        Ok(Some(user)) => (StatusCode::OK, Json(UserInfo::from(user))).into_response(),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "get_failed",
                "message": "Failed to retrieve updated user"
            })),
        )
            .into_response(),
    }
}

/// Delete a user (admin only)
#[utoipa::path(
    delete,
    path = "/api/v1/users/{user_id}",
    tag = "Users",
    params(
        ("user_id" = String, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User deleted"),
        (status = 403, description = "Admin access required"),
        (status = 404, description = "User not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_user(
    State(db): State<Database>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(user_id): Path<String>,
) -> impl IntoResponse {
    // Check admin access
    if !auth_user.is_admin() {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "forbidden",
                "message": "Admin access required"
            })),
        )
            .into_response();
    }

    // Check user exists
    match db.get_user(&user_id) {
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "not_found",
                    "message": "User not found"
                })),
            )
                .into_response()
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "get_failed",
                    "message": e.to_string()
                })),
            )
                .into_response()
        }
        Ok(Some(_)) => {}
    }

    match db.delete_user(&user_id) {
        Ok(()) => (
            StatusCode::OK,
            Json(json!({
                "message": "User deleted successfully"
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "delete_failed",
                "message": e.to_string()
            })),
        )
            .into_response(),
    }
}

/// Create an access token for a user (admin or self)
#[utoipa::path(
    post,
    path = "/api/v1/users/{user_id}/tokens",
    tag = "Users",
    params(
        ("user_id" = String, Path, description = "User ID")
    ),
    request_body = CreateAccessTokenRequest,
    responses(
        (status = 201, description = "Access token created", body = CreateAccessTokenResponse),
        (status = 403, description = "Access denied"),
        (status = 404, description = "User not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_access_token(
    State(db): State<Database>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(user_id): Path<String>,
    Json(request): Json<CreateAccessTokenRequest>,
) -> impl IntoResponse {
    // Check admin access or self-access
    let is_self = auth_user.user_id() == Some(&user_id);
    if !auth_user.is_admin() && !is_self {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "forbidden",
                "message": "Admin access or self-access required"
            })),
        )
            .into_response();
    }

    // Check user exists
    match db.get_user(&user_id) {
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "not_found",
                    "message": "User not found"
                })),
            )
                .into_response()
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "get_failed",
                    "message": e.to_string()
                })),
            )
                .into_response()
        }
        Ok(Some(_)) => {}
    }

    let token_id = Uuid::new_v4().to_string();
    let token = generate_access_token();
    let token_hash = hash_token(&token);

    // Calculate expiration from days
    let expires_at = request.expires_in_days.map(|days| {
        let now = chrono::Utc::now();
        let expiry = now + chrono::Duration::days(days as i64);
        expiry.to_rfc3339()
    });

    match db.create_access_token(
        &token_id,
        &user_id,
        &request.name,
        &token_hash,
        expires_at.as_deref(),
    ) {
        Ok(token_record) => (
            StatusCode::CREATED,
            Json(CreateAccessTokenResponse {
                token_info: AccessTokenInfo::from(token_record),
                access_token: token, // Return plaintext token only on creation
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "create_failed",
                "message": e.to_string()
            })),
        )
            .into_response(),
    }
}

/// List access tokens for a user (admin or self)
#[utoipa::path(
    get,
    path = "/api/v1/users/{user_id}/tokens",
    tag = "Users",
    params(
        ("user_id" = String, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "Access tokens list", body = ListAccessTokensResponse),
        (status = 403, description = "Access denied"),
        (status = 404, description = "User not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_access_tokens(
    State(db): State<Database>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(user_id): Path<String>,
) -> impl IntoResponse {
    // Check admin access or self-access
    let is_self = auth_user.user_id() == Some(&user_id);
    if !auth_user.is_admin() && !is_self {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "forbidden",
                "message": "Admin access or self-access required"
            })),
        )
            .into_response();
    }

    match db.list_user_access_tokens(&user_id) {
        Ok(tokens) => {
            let token_infos: Vec<AccessTokenInfo> =
                tokens.into_iter().map(AccessTokenInfo::from).collect();
            (
                StatusCode::OK,
                Json(ListAccessTokensResponse {
                    tokens: token_infos,
                }),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "list_failed",
                "message": e.to_string()
            })),
        )
            .into_response(),
    }
}

/// Delete an access token (admin or self)
#[utoipa::path(
    delete,
    path = "/api/v1/users/{user_id}/tokens/{token_id}",
    tag = "Users",
    params(
        ("user_id" = String, Path, description = "User ID"),
        ("token_id" = String, Path, description = "Token ID")
    ),
    responses(
        (status = 200, description = "Token deleted"),
        (status = 403, description = "Access denied"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_access_token(
    State(db): State<Database>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path((user_id, token_id)): Path<(String, String)>,
) -> impl IntoResponse {
    // Check admin access or self-access
    let is_self = auth_user.user_id() == Some(&user_id);
    if !auth_user.is_admin() && !is_self {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "forbidden",
                "message": "Admin access or self-access required"
            })),
        )
            .into_response();
    }

    match db.delete_access_token(&token_id, &user_id) {
        Ok(()) => (
            StatusCode::OK,
            Json(json!({
                "message": "Access token deleted successfully"
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "delete_failed",
                "message": e.to_string()
            })),
        )
            .into_response(),
    }
}

// === Instance Assignment Handlers ===

/// Get user's instance permissions
#[utoipa::path(
    get,
    path = "/api/v1/users/{user_id}/instances",
    tag = "Users",
    params(
        ("user_id" = String, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User's instance permissions", body = UserInstancesResponse),
        (status = 403, description = "Access denied"),
        (status = 404, description = "User not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_user_instances(
    State(db): State<Database>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(user_id): Path<String>,
) -> impl IntoResponse {
    // Check admin access or self-access
    let is_self = auth_user.user_id() == Some(&user_id);
    if !auth_user.is_admin() && !is_self {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "forbidden",
                "message": "Admin access or self-access required"
            })),
        )
            .into_response();
    }

    match db.list_user_instances(&user_id) {
        Ok(instances) => {
            (StatusCode::OK, Json(UserInstancesResponse { instances })).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "list_failed",
                "message": e.to_string()
            })),
        )
            .into_response(),
    }
}

/// Assign instance to a user (admin only)
#[utoipa::path(
    post,
    path = "/api/v1/users/assign-instance",
    tag = "Users",
    request_body = AssignInstanceRequest,
    responses(
        (status = 200, description = "Instance assigned", body = InstanceOwnerRecord),
        (status = 403, description = "Admin access required"),
        (status = 404, description = "User or instance not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn assign_instance(
    State(db): State<Database>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    Json(request): Json<AssignInstanceRequest>,
) -> impl IntoResponse {
    // Check admin access
    if !auth_user.is_admin() {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "forbidden",
                "message": "Admin access required"
            })),
        )
            .into_response();
    }

    // Verify user exists
    match db.get_user(&request.user_id) {
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "not_found",
                    "message": "User not found"
                })),
            )
                .into_response()
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "get_failed",
                    "message": e.to_string()
                })),
            )
                .into_response()
        }
        Ok(Some(_)) => {}
    }

    // Verify instance exists
    match db.get_instance(&request.instance_id) {
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "not_found",
                    "message": "Instance not found"
                })),
            )
                .into_response()
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "get_failed",
                    "message": e.to_string()
                })),
            )
                .into_response()
        }
        Ok(Some(_)) => {}
    }

    match db.assign_instance_to_user(&request.user_id, &request.instance_id, request.permission) {
        Ok(record) => (StatusCode::OK, Json(record)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "assign_failed",
                "message": e.to_string()
            })),
        )
            .into_response(),
    }
}

/// Remove instance from a user (admin only)
#[utoipa::path(
    delete,
    path = "/api/v1/users/{user_id}/instances/{instance_id}",
    tag = "Users",
    params(
        ("user_id" = String, Path, description = "User ID"),
        ("instance_id" = String, Path, description = "Instance ID")
    ),
    responses(
        (status = 200, description = "Instance removed from user"),
        (status = 403, description = "Admin access required"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn remove_instance(
    State(db): State<Database>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path((user_id, instance_id)): Path<(String, String)>,
) -> impl IntoResponse {
    // Check admin access
    if !auth_user.is_admin() {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "forbidden",
                "message": "Admin access required"
            })),
        )
            .into_response();
    }

    match db.remove_instance_from_user(&user_id, &instance_id) {
        Ok(()) => (
            StatusCode::OK,
            Json(json!({
                "message": "Instance removed from user"
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "remove_failed",
                "message": e.to_string()
            })),
        )
            .into_response(),
    }
}

/// Get current user info (self)
#[utoipa::path(
    get,
    path = "/api/v1/users/me",
    tag = "Users",
    responses(
        (status = 200, description = "Current user info", body = UserInfo),
        (status = 401, description = "Not authenticated as user"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_me(
    State(db): State<Database>,
    Extension(auth_user): Extension<AuthenticatedUser>,
) -> impl IntoResponse {
    match auth_user {
        AuthenticatedUser::Secret => (
            StatusCode::OK,
            Json(json!({
                "id": "superadmin",
                "username": "superadmin",
                "role": "admin",
                "is_active": true,
                "note": "Authenticated via secret key"
            })),
        )
            .into_response(),
        AuthenticatedUser::User { id, .. } => match db.get_user(&id) {
            Ok(Some(user)) => (StatusCode::OK, Json(UserInfo::from(user))).into_response(),
            Ok(None) => (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "not_found",
                    "message": "User not found"
                })),
            )
                .into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "get_failed",
                    "message": e.to_string()
                })),
            )
                .into_response(),
        },
    }
}
