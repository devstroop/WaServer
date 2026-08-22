//! Token handlers — extracted from `handlers/api/users.rs:400..603` (part of #9)
//! Thin: validates via `domain::identity::token`, delegates to `application::identity::TokenService`.

use axum::{extract::{Extension, Path, State}, http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
use uuid::Uuid;

use crate::{
    application::identity::{CreateTokenInput, TokenService},
    models::{
        auth::AuthenticatedUser,
        user::{AccessTokenInfo, CreateAccessTokenRequest, CreateAccessTokenResponse, ListAccessTokensResponse},
    },
    services::Database,
};

/// `POST /api/v1/users/{user_id}/tokens` — admin or self
#[utoipa::path(post, path = "/api/v1/users/{user_id}/tokens", tag = "Users", params(("user_id" = String, Path, description = "User ID")), request_body=CreateAccessTokenRequest, responses((status=201, body=CreateAccessTokenResponse)), security(("bearer_auth" = [])))]
pub async fn create_access_token(State(db): State<Database>, Extension(auth_user): Extension<AuthenticatedUser>, Path(user_id): Path<String>, Json(req): Json<CreateAccessTokenRequest>) -> impl IntoResponse {
    let is_self = auth_user.user_id() == Some(&user_id);
    if !auth_user.is_admin() && !is_self { return (StatusCode::FORBIDDEN, Json(json!({"error":"forbidden","message":"Admin access or self-access required"}))).into_response(); }
    match db.get_user(&user_id) {
        Ok(None) => return (StatusCode::NOT_FOUND, Json(json!({"error":"not_found","message":"User not found"}))).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error":"get_failed","message":e.to_string()}))).into_response(),
        _ => {}
    }
    let input = CreateTokenInput { name: req.name.clone(), expires_in_days: req.expires_in_days };
    if let Err(e) = input.validate() { return (StatusCode::BAD_REQUEST, Json(json!({"error":"invalid_request","message":e.to_string()}))).into_response(); }
    let token_id = Uuid::new_v4().to_string();
    let token = TokenService::generate_token();
    let hash = TokenService::hash(&token);
    let expires_at = req.expires_in_days.map(|d| (chrono::Utc::now() + chrono::Duration::days(d as i64)).to_rfc3339());
    match db.create_access_token(&token_id, &user_id, &req.name, &hash, expires_at.as_deref()) {
        Ok(rec) => (StatusCode::CREATED, Json(CreateAccessTokenResponse{ token_info: AccessTokenInfo::from(rec), access_token: token })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error":"create_failed","message":e.to_string()}))).into_response(),
    }
}

/// `GET /api/v1/users/{user_id}/tokens` — admin or self
#[utoipa::path(get, path = "/api/v1/users/{user_id}/tokens", tag = "Users", params(("user_id" = String, Path, description = "User ID")), responses((status=200, body=ListAccessTokensResponse)), security(("bearer_auth" = [])))]
pub async fn list_access_tokens(State(db): State<Database>, Extension(auth_user): Extension<AuthenticatedUser>, Path(user_id): Path<String>) -> impl IntoResponse {
    let is_self = auth_user.user_id() == Some(&user_id);
    if !auth_user.is_admin() && !is_self { return (StatusCode::FORBIDDEN, Json(json!({"error":"forbidden","message":"Admin access or self-access required"}))).into_response(); }
    match db.list_user_access_tokens(&user_id) {
        Ok(tokens) => {
            let infos: Vec<AccessTokenInfo> = tokens.into_iter().map(AccessTokenInfo::from).collect();
            (StatusCode::OK, Json(ListAccessTokensResponse{ tokens: infos })).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error":"list_failed","message":e.to_string()}))).into_response(),
    }
}

/// `DELETE /api/v1/users/{user_id}/tokens/{token_id}` — admin or self
#[utoipa::path(delete, path = "/api/v1/users/{user_id}/tokens/{token_id}", tag = "Users", params(("user_id" = String, Path, description = "User ID"), ("token_id" = String, Path, description = "Token ID")), responses((status=200, description="Token deleted")), security(("bearer_auth" = [])))]
pub async fn delete_access_token(State(db): State<Database>, Extension(auth_user): Extension<AuthenticatedUser>, Path((user_id, token_id)): Path<(String, String)>) -> impl IntoResponse {
    let is_self = auth_user.user_id() == Some(&user_id);
    if !auth_user.is_admin() && !is_self { return (StatusCode::FORBIDDEN, Json(json!({"error":"forbidden","message":"Admin access or self-access required"}))).into_response(); }
    match db.delete_access_token(&token_id, &user_id) {
        Ok(()) => (StatusCode::OK, Json(json!({"message":"Access token deleted successfully"}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error":"delete_failed","message":e.to_string()}))).into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_token_handlers_exist() { let _ = std::any::type_name::<CreateAccessTokenRequest>(); }
}
