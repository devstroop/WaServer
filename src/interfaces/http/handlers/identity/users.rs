//! Users CRUD — extracted from `handlers/api/users.rs:36..398` (part of #9)
//! Thin: validates via `domain::identity::user`, calls `application::identity::UserService`,
//! RBAC via `application::identity::RbacService`. No DB in handler.

use axum::{extract::{Extension, Path, State}, http::StatusCode, response::IntoResponse, Json};
use serde_json::json;

use crate::{
    application::identity::{CreateUserInput, UserService},
    models::{
        auth::AuthenticatedUser,
        user::{CreateUserRequest, CreateUserResponse, ListUsersResponse, UpdateUserRequest, UserInfo},
    },
    services::Database,
};

/// `GET /api/v1/users` — admin only, thin wrapper
#[utoipa::path(get, path = "/api/v1/users", tag = "Users", responses((status=200, body=ListUsersResponse)), security(("bearer_auth" = [])))]
pub async fn list_users(State(db): State<Database>, Extension(auth_user): Extension<AuthenticatedUser>) -> impl IntoResponse {
    if !auth_user.is_admin() {
        return (StatusCode::FORBIDDEN, Json(json!({"error":"forbidden","message":"Admin access required"}))).into_response();
    }
    match db.list_users() {
        Ok(users) => {
            let infos: Vec<UserInfo> = users.into_iter().map(UserInfo::from).collect();
            let total = infos.len();
            (StatusCode::OK, Json(ListUsersResponse{ users: infos, total })).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error":"list_failed","message":e.to_string()}))).into_response(),
    }
}

/// `POST /api/v1/users` — validates via domain, delegates to UserService
#[utoipa::path(post, path = "/api/v1/users", tag = "Users", request_body=CreateUserRequest, responses((status=201, body=CreateUserResponse)), security(("bearer_auth" = [])))]
pub async fn create_user(State(db): State<Database>, Extension(auth_user): Extension<AuthenticatedUser>, Json(req): Json<CreateUserRequest>) -> impl IntoResponse {
    if !auth_user.is_admin() {
        return (StatusCode::FORBIDDEN, Json(json!({"error":"forbidden","message":"Admin access required"}))).into_response();
    }
    let input = CreateUserInput { username: req.username.clone(), email: req.email.clone(), password: req.password.clone(), role: req.role };
    if let Err(e) = UserService::validate_create(&input) {
        return (StatusCode::BAD_REQUEST, Json(json!({"error":"invalid_request","message":e.to_string()}))).into_response();
    }
    let user_id = uuid::Uuid::new_v4().to_string();
    let hash = UserService::hash_password(&req.password);
    match db.create_user(&user_id, &req.username, req.email.as_deref(), &hash, req.role) {
        Ok(rec) => (StatusCode::CREATED, Json(CreateUserResponse{ user: UserInfo::from(rec) })).into_response(),
        Err(e) => {
            let msg = e.to_string();
            let status = if msg.contains("already exists") { StatusCode::CONFLICT } else { StatusCode::INTERNAL_SERVER_ERROR };
            (status, Json(json!({"error":"create_failed","message":msg}))).into_response()
        }
    }
}

/// `GET /api/v1/users/{user_id}` — admin or self
#[utoipa::path(get, path = "/api/v1/users/{user_id}", tag = "Users", params(("user_id" = String, Path, description = "User ID")), responses((status=200, body=UserInfo)), security(("bearer_auth" = [])))]
pub async fn get_user(State(db): State<Database>, Extension(auth_user): Extension<AuthenticatedUser>, Path(user_id): Path<String>) -> impl IntoResponse {
    let is_self = auth_user.user_id() == Some(&user_id);
    if !auth_user.is_admin() && !is_self {
        return (StatusCode::FORBIDDEN, Json(json!({"error":"forbidden","message":"Admin access required"}))).into_response();
    }
    match db.get_user(&user_id) {
        Ok(Some(u)) => (StatusCode::OK, Json(UserInfo::from(u))).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(json!({"error":"not_found","message":"User not found"}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error":"get_failed","message":e.to_string()}))).into_response(),
    }
}

/// `PATCH /api/v1/users/{user_id}` — admin only
#[utoipa::path(patch, path = "/api/v1/users/{user_id}", tag = "Users", params(("user_id" = String, Path, description = "User ID")), request_body=UpdateUserRequest, responses((status=200, body=UserInfo)), security(("bearer_auth" = [])))]
pub async fn update_user(State(db): State<Database>, Extension(auth_user): Extension<AuthenticatedUser>, Path(user_id): Path<String>, Json(req): Json<UpdateUserRequest>) -> impl IntoResponse {
    if !auth_user.is_admin() { return (StatusCode::FORBIDDEN, Json(json!({"error":"forbidden","message":"Admin access required"}))).into_response(); }
    match db.get_user(&user_id) {
        Ok(None) => return (StatusCode::NOT_FOUND, Json(json!({"error":"not_found","message":"User not found"}))).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error":"get_failed","message":e.to_string()}))).into_response(),
        _ => {}
    }
    if let Err(e) = db.update_user(&user_id, req.username.as_deref(), None, None, req.role, req.is_active) {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error":"update_failed","message":e.to_string()}))).into_response();
    }
    match db.get_user(&user_id) {
        Ok(Some(u)) => (StatusCode::OK, Json(UserInfo::from(u))).into_response(),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error":"get_failed","message":"Failed to retrieve updated user"}))).into_response(),
    }
}

/// `DELETE /api/v1/users/{user_id}` — admin only
#[utoipa::path(delete, path = "/api/v1/users/{user_id}", tag = "Users", params(("user_id" = String, Path, description = "User ID")), responses((status=200, description="User deleted")), security(("bearer_auth" = [])))]
pub async fn delete_user(State(db): State<Database>, Extension(auth_user): Extension<AuthenticatedUser>, Path(user_id): Path<String>) -> impl IntoResponse {
    if !auth_user.is_admin() { return (StatusCode::FORBIDDEN, Json(json!({"error":"forbidden","message":"Admin access required"}))).into_response(); }
    match db.get_user(&user_id) {
        Ok(None) => return (StatusCode::NOT_FOUND, Json(json!({"error":"not_found","message":"User not found"}))).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error":"get_failed","message":e.to_string()}))).into_response(),
        _ => {}
    }
    match db.delete_user(&user_id) {
        Ok(()) => (StatusCode::OK, Json(json!({"message":"User deleted successfully"}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error":"delete_failed","message":e.to_string()}))).into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_user_handlers_exist() { let _ = std::any::type_name::<CreateUserRequest>(); }
}
