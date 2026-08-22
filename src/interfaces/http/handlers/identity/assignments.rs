//! Instance assignments — extracted from `handlers/api/users.rs:605..796` (part of #9)
//! RBAC via `application::identity::RbacService`, pure permission checks.

use axum::{extract::{Extension, Path, State}, http::StatusCode, response::IntoResponse, Json};
use serde_json::json;

use crate::{
    models::{
        auth::AuthenticatedUser,
        user::{AssignInstanceRequest, UserInstancesResponse},
    },
    services::Database,
};

/// `GET /api/v1/users/{user_id}/instances` — admin or self
#[utoipa::path(get, path = "/api/v1/users/{user_id}/instances", tag = "Users", params(("user_id" = String, Path, description = "User ID")), responses((status=200, body=UserInstancesResponse)), security(("bearer_auth" = [])))]
pub async fn get_user_instances(State(db): State<Database>, Extension(auth_user): Extension<AuthenticatedUser>, Path(user_id): Path<String>) -> impl IntoResponse {
    let is_self = auth_user.user_id() == Some(&user_id);
    if !auth_user.is_admin() && !is_self { return (StatusCode::FORBIDDEN, Json(json!({"error":"forbidden","message":"Admin access or self-access required"}))).into_response(); }
    match db.list_user_instances(&user_id) {
        Ok(instances) => (StatusCode::OK, Json(UserInstancesResponse{ instances })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error":"list_failed","message":e.to_string()}))).into_response(),
    }
}

/// `POST /api/v1/users/assign-instance` — admin only
#[utoipa::path(post, path = "/api/v1/users/assign-instance", tag = "Users", request_body=AssignInstanceRequest, responses((status=200, body=crate::models::user::InstanceOwnerRecord)), security(("bearer_auth" = [])))]
pub async fn assign_instance(State(db): State<Database>, Extension(auth_user): Extension<AuthenticatedUser>, Json(req): Json<AssignInstanceRequest>) -> impl IntoResponse {
    if !auth_user.is_admin() { return (StatusCode::FORBIDDEN, Json(json!({"error":"forbidden","message":"Admin access required"}))).into_response(); }
    match db.get_user(&req.user_id) {
        Ok(None) => return (StatusCode::NOT_FOUND, Json(json!({"error":"not_found","message":"User not found"}))).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error":"get_failed","message":e.to_string()}))).into_response(),
        _ => {}
    }
    match db.get_instance(&req.instance_id) {
        Ok(None) => return (StatusCode::NOT_FOUND, Json(json!({"error":"not_found","message":"Instance not found"}))).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error":"get_failed","message":e.to_string()}))).into_response(),
        _ => {}
    }
    match db.assign_instance_to_user(&req.user_id, &req.instance_id, req.permission) {
        Ok(rec) => (StatusCode::OK, Json(rec)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error":"assign_failed","message":e.to_string()}))).into_response(),
    }
}

/// `DELETE /api/v1/users/{user_id}/instances/{instance_id}` — admin only
#[utoipa::path(delete, path = "/api/v1/users/{user_id}/instances/{instance_id}", tag = "Users", params(("user_id" = String, Path, description = "User ID"), ("instance_id" = String, Path, description = "Instance ID")), responses((status=200, description="Instance removed")), security(("bearer_auth" = [])))]
pub async fn remove_instance(State(db): State<Database>, Extension(auth_user): Extension<AuthenticatedUser>, Path((user_id, instance_id)): Path<(String, String)>) -> impl IntoResponse {
    if !auth_user.is_admin() { return (StatusCode::FORBIDDEN, Json(json!({"error":"forbidden","message":"Admin access required"}))).into_response(); }
    match db.remove_instance_from_user(&user_id, &instance_id) {
        Ok(()) => (StatusCode::OK, Json(json!({"message":"Instance removed from user"}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error":"remove_failed","message":e.to_string()}))).into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_assign_handlers_exist() { let _ = std::any::type_name::<AssignInstanceRequest>(); }
}
