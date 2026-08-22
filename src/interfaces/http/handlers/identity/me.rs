//! `GET /api/v1/users/me` — self info, extracted from `handlers/api/users.rs:799` (part of #9)

use axum::{extract::{Extension, State}, http::StatusCode, response::IntoResponse, Json};
use serde_json::json;

use crate::{models::{auth::AuthenticatedUser, user::UserInfo}, services::Database};

#[utoipa::path(get, path = "/api/v1/users/me", tag = "Users", responses((status=200, body=UserInfo)), security(("bearer_auth" = [])))]
pub async fn get_me(State(db): State<Database>, Extension(auth_user): Extension<AuthenticatedUser>) -> impl IntoResponse {
    match auth_user {
        AuthenticatedUser::Secret => (StatusCode::OK, Json(json!({"id":"superadmin","username":"superadmin","role":"admin","is_active":true,"note":"Authenticated via secret key"}))).into_response(),
        AuthenticatedUser::User{ id, .. } => match db.get_user(&id) {
            Ok(Some(u)) => (StatusCode::OK, Json(UserInfo::from(u))).into_response(),
            Ok(None) => (StatusCode::NOT_FOUND, Json(json!({"error":"not_found","message":"User not found"}))).into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error":"get_failed","message":e.to_string()}))).into_response(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_me_exists() { let _ = std::any::type_name::<UserInfo>(); }
}
