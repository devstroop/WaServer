//! Admin API - Users, Roles, and Permissions
//!
//! REST API endpoints for managing users, roles, and permissions.
//! These are stub implementations that return "not implemented" responses.

use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;

use crate::models::admin::{
    CreateUserRequest, UpdateUserRequest,
    CreateRoleRequest, UpdateRoleRequest,
};

// =============================================================================
// Users
// =============================================================================

/// List all users
#[utoipa::path(
    get,
    path = "/api/v1/users",
    tag = "Users",
    responses(
        (status = 200, description = "List of users", body = Vec<User>),
        (status = 501, description = "Not implemented"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_users() -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({
            "error": "not_implemented",
            "message": "User management is not yet implemented"
        })),
    )
}

/// Create a new user
#[utoipa::path(
    post,
    path = "/api/v1/users",
    tag = "Users",
    request_body = CreateUserRequest,
    responses(
        (status = 201, description = "User created", body = User),
        (status = 400, description = "Invalid request"),
        (status = 409, description = "User already exists"),
        (status = 501, description = "Not implemented"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_user(
    Json(_request): Json<CreateUserRequest>,
) -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({
            "error": "not_implemented",
            "message": "User management is not yet implemented"
        })),
    )
}

/// Get user by ID
#[utoipa::path(
    get,
    path = "/api/v1/users/{user_id}",
    tag = "Users",
    params(
        ("user_id" = String, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User info", body = User),
        (status = 404, description = "User not found"),
        (status = 501, description = "Not implemented"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_user(
    Path(_user_id): Path<String>,
) -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({
            "error": "not_implemented",
            "message": "User management is not yet implemented"
        })),
    )
}

/// Update a user
#[utoipa::path(
    put,
    path = "/api/v1/users/{user_id}",
    tag = "Users",
    params(
        ("user_id" = String, Path, description = "User ID")
    ),
    request_body = UpdateUserRequest,
    responses(
        (status = 200, description = "User updated", body = User),
        (status = 400, description = "Invalid request"),
        (status = 404, description = "User not found"),
        (status = 501, description = "Not implemented"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_user(
    Path(_user_id): Path<String>,
    Json(_request): Json<UpdateUserRequest>,
) -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({
            "error": "not_implemented",
            "message": "User management is not yet implemented"
        })),
    )
}

/// Delete a user
#[utoipa::path(
    delete,
    path = "/api/v1/users/{user_id}",
    tag = "Users",
    params(
        ("user_id" = String, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User deleted"),
        (status = 404, description = "User not found"),
        (status = 501, description = "Not implemented"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_user(
    Path(_user_id): Path<String>,
) -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({
            "error": "not_implemented",
            "message": "User management is not yet implemented"
        })),
    )
}

// =============================================================================
// Roles
// =============================================================================

/// List all roles
#[utoipa::path(
    get,
    path = "/api/v1/roles",
    tag = "Roles",
    responses(
        (status = 200, description = "List of roles", body = Vec<Role>),
        (status = 501, description = "Not implemented"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_roles() -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({
            "error": "not_implemented",
            "message": "Role management is not yet implemented"
        })),
    )
}

/// Create a new role
#[utoipa::path(
    post,
    path = "/api/v1/roles",
    tag = "Roles",
    request_body = CreateRoleRequest,
    responses(
        (status = 201, description = "Role created", body = Role),
        (status = 400, description = "Invalid request"),
        (status = 409, description = "Role already exists"),
        (status = 501, description = "Not implemented"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_role(
    Json(_request): Json<CreateRoleRequest>,
) -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({
            "error": "not_implemented",
            "message": "Role management is not yet implemented"
        })),
    )
}

/// Get role by ID
#[utoipa::path(
    get,
    path = "/api/v1/roles/{role_id}",
    tag = "Roles",
    params(
        ("role_id" = String, Path, description = "Role ID")
    ),
    responses(
        (status = 200, description = "Role info", body = Role),
        (status = 404, description = "Role not found"),
        (status = 501, description = "Not implemented"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_role(
    Path(_role_id): Path<String>,
) -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({
            "error": "not_implemented",
            "message": "Role management is not yet implemented"
        })),
    )
}

/// Update a role
#[utoipa::path(
    put,
    path = "/api/v1/roles/{role_id}",
    tag = "Roles",
    params(
        ("role_id" = String, Path, description = "Role ID")
    ),
    request_body = UpdateRoleRequest,
    responses(
        (status = 200, description = "Role updated", body = Role),
        (status = 400, description = "Invalid request"),
        (status = 404, description = "Role not found"),
        (status = 501, description = "Not implemented"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_role(
    Path(_role_id): Path<String>,
    Json(_request): Json<UpdateRoleRequest>,
) -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({
            "error": "not_implemented",
            "message": "Role management is not yet implemented"
        })),
    )
}

/// Delete a role
#[utoipa::path(
    delete,
    path = "/api/v1/roles/{role_id}",
    tag = "Roles",
    params(
        ("role_id" = String, Path, description = "Role ID")
    ),
    responses(
        (status = 200, description = "Role deleted"),
        (status = 404, description = "Role not found"),
        (status = 501, description = "Not implemented"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_role(
    Path(_role_id): Path<String>,
) -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({
            "error": "not_implemented",
            "message": "Role management is not yet implemented"
        })),
    )
}

// =============================================================================
// Permissions
// =============================================================================

/// List all permissions
#[utoipa::path(
    get,
    path = "/api/v1/permissions",
    tag = "Permissions",
    responses(
        (status = 200, description = "List of permissions", body = Vec<Permission>),
        (status = 501, description = "Not implemented"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_permissions() -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({
            "error": "not_implemented",
            "message": "Permission management is not yet implemented"
        })),
    )
}

/// Get permission by ID
#[utoipa::path(
    get,
    path = "/api/v1/permissions/{permission_id}",
    tag = "Permissions",
    params(
        ("permission_id" = String, Path, description = "Permission ID")
    ),
    responses(
        (status = 200, description = "Permission info", body = Permission),
        (status = 404, description = "Permission not found"),
        (status = 501, description = "Not implemented"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_permission(
    Path(_permission_id): Path<String>,
) -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({
            "error": "not_implemented",
            "message": "Permission management is not yet implemented"
        })),
    )
}
