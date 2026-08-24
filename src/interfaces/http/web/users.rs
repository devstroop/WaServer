//! Users & access tokens administration (#33)
//!
//! Admin-gated server-rendered CRUD mapping 1:1 onto existing DB methods.
//! Token secrets are shown exactly once at creation time.

use askama::Template;
use axum::{
    extract::{Form, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use uuid::Uuid;

use crate::models::user::{InstanceOwnerRecord, UserRecord};

use super::pages::{error_fragment, WebSessionExt, VERSION};

/// Admin gate — non-admins get a swap-friendly 403 fragment
#[allow(clippy::result_large_err)]
fn require_admin(session: &WebSessionExt) -> Result<(), Response> {
    if session.admin {
        Ok(())
    } else {
        tracing::warn!(username = %session.username, "non-admin blocked from admin route");
        Err(forbidden_fragment())
    }
}

fn forbidden_fragment() -> Response {
    (
        StatusCode::FORBIDDEN,
        axum::response::Html(error_fragment("Administrator access required.")),
    )
        .into_response()
}

// ---------------------------------------------------------------------------
// Users list
// ---------------------------------------------------------------------------

#[derive(Template)]
#[template(path = "web/pages/users.html")]
pub struct UsersPageTemplate {
    pub title: String,
    pub version: &'static str,
    pub csrf: String,
    pub username: String,
    pub role: String,
    pub admin: bool,
}

/// `GET /app/users`
pub async fn list_page(session: WebSessionExt) -> Response {
    if require_admin(&session).is_err() {
        return forbidden_fragment();
    }
    html_page(UsersPageTemplate {
        title: "Users — WAS Admin".into(),
        version: VERSION,
        csrf: session.csrf_token.clone(),
        username: session.username.clone(),
        role: session.role.clone(),
        admin: session.admin,
    })
}

#[derive(Template)]
#[template(path = "web/_user_table.html")]
pub struct UserTableTemplate {
    pub users: Vec<UserRow>,
    pub csrf: String,
}

pub struct UserRow {
    pub id: String,
    pub username: String,
    pub email: String,
    pub role: &'static str,
    pub badge_class: &'static str,
    pub is_active: bool,
}

fn user_row(u: &UserRecord) -> UserRow {
    let (role, badge_class) = match u.role {
        crate::domain::identity::UserRole::Admin => ("admin", "dt-badge--primary"),
        crate::domain::identity::UserRole::User => ("user", "dt-badge--neutral"),
    };
    UserRow {
        id: u.id.clone(),
        username: u.username.clone(),
        email: u.email.clone().unwrap_or_else(|| "—".into()),
        role,
        badge_class,
        is_active: u.is_active,
    }
}

/// `GET /app/fragments/users` — table body refreshed after mutations
pub async fn table_fragment(
    State(app): State<super::pages::AppState>,
    session: WebSessionExt,
) -> Response {
    let tpl = UserTableTemplate {
        users: app
            .db
            .list_users()
            .unwrap_or_default()
            .iter()
            .map(user_row)
            .collect(),
        csrf: session.csrf_token.clone(),
    };
    (
        StatusCode::OK,
        axum::response::Html(tpl.render().unwrap_or_default()),
    )
        .into_response()
}

#[derive(Deserialize)]
pub struct CreateUserForm {
    pub username: String,
    #[serde(default)]
    pub email: Option<String>,
    pub password: String,
    #[serde(default)]
    pub role: Option<String>,
}

/// `POST /app/users` — create user from the dialog
pub async fn create_post(
    State(app): State<super::pages::AppState>,
    session: WebSessionExt,
    Form(form): Form<CreateUserForm>,
) -> Response {
    if require_admin(&session).is_err() {
        return forbidden_fragment();
    }
    let username = form.username.trim().to_string();
    if username.is_empty() || form.password.len() < 8 {
        return (
            StatusCode::BAD_REQUEST,
            axum::response::Html(error_fragment(
                "Username required and password must be at least 8 characters.",
            )),
        )
            .into_response();
    }
    let role = match form.role.as_deref() {
        Some("admin") => crate::domain::identity::UserRole::Admin,
        _ => crate::domain::identity::UserRole::User,
    };
    match app.db.create_user(
        &Uuid::new_v4().to_string(),
        &username,
        form.email
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty()),
        &crate::middleware::auth::hash_password(&form.password),
        role,
    ) {
        Ok(_) => toast("success", &format!("User {username} created.")),
        Err(e) => {
            let text = if e.to_string().contains("UNIQUE") {
                "Username or email already taken."
            } else {
                &e.to_string()
            };
            (
                StatusCode::BAD_REQUEST,
                axum::response::Html(error_fragment(text)),
            )
                .into_response()
        }
    }
}

/// `POST /app/users/:id/delete`
pub async fn delete_post(
    State(app): State<super::pages::AppState>,
    session: WebSessionExt,
    Path(id): Path<String>,
) -> Response {
    if require_admin(&session).is_err() {
        return forbidden_fragment();
    }
    // Guard against self-deletion
    if let Ok(Some(user)) = app.db.get_user(&id) {
        if user.username == session.username {
            return (
                StatusCode::BAD_REQUEST,
                axum::response::Html(error_fragment("You cannot delete your own account.")),
            )
                .into_response();
        }
    }
    match app.db.delete_user(&id) {
        Ok(()) => toast("success", "User deleted."),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::response::Html(error_fragment(&format!("Delete failed: {e}"))),
        )
            .into_response(),
    }
}

#[derive(Template)]
#[template(path = "web/_toast.html")]
pub struct ToastFragment {
    pub tone: &'static str,
    pub message: String,
}

fn toast(tone: &'static str, message: &str) -> Response {
    let tpl = ToastFragment {
        tone,
        message: message.to_string(),
    };
    (
        StatusCode::OK,
        axum::response::Html(tpl.render().unwrap_or_default()),
    )
        .into_response()
}

fn html_page<T: askama::Template>(tpl: T) -> Response {
    (
        StatusCode::OK,
        axum::response::Html(tpl.render().unwrap_or_default()),
    )
        .into_response()
}

// ---------------------------------------------------------------------------
// User detail — profile, tokens, assignments
// ---------------------------------------------------------------------------

#[derive(Template)]
#[template(path = "web/pages/user_detail.html")]
pub struct UserDetailTemplate {
    pub title: String,
    pub version: &'static str,
    pub csrf: String,
    pub username: String,
    pub role: String,
    pub admin: bool,
    pub user_id: String,
    pub user_name: String,
    pub user_email: String,
    pub user_role: &'static str,
    pub badge_class: &'static str,
    pub is_active: bool,
    pub tokens: Vec<TokenRow>,
    pub assignments: Vec<AssignmentRow>,
    pub instance_options: Vec<(String, String)>,
}

pub struct TokenRow {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub last_used: String,
}

pub struct AssignmentRow {
    pub instance_id: String,
    pub label: String,
}

/// `GET /app/users/:id`
pub async fn detail_page(
    State(app): State<super::pages::AppState>,
    session: WebSessionExt,
    Path(user_id): Path<String>,
) -> Response {
    if require_admin(&session).is_err() {
        return forbidden_fragment();
    }
    let Ok(Some(user)) = app.db.get_user(&user_id) else {
        return (
            StatusCode::NOT_FOUND,
            axum::response::Html(error_fragment("User not found.")),
        )
            .into_response();
    };
    let tokens = app.db.list_user_access_tokens(&user_id).unwrap_or_default();
    let assignments = app.db.list_user_instances(&user_id).unwrap_or_default();
    let options = app.manager.list_status_snapshots().await;

    let (role, badge_class) = match user.role {
        crate::domain::identity::UserRole::Admin => ("admin", "dt-badge--primary"),
        crate::domain::identity::UserRole::User => ("user", "dt-badge--neutral"),
    };
    let tpl = UserDetailTemplate {
        title: format!("{} — WAS Admin", user.username),
        version: VERSION,
        csrf: session.csrf_token.clone(),
        username: session.username.clone(),
        role: session.role.clone(),
        admin: session.admin,
        user_id: user.id.clone(),
        user_name: user.username.clone(),
        user_email: user.email.clone().unwrap_or_else(|| "—".into()),
        user_role: role,
        badge_class,
        is_active: user.is_active,
        tokens: tokens
            .iter()
            .map(|t| TokenRow {
                id: t.id.clone(),
                name: t.name.clone(),
                created_at: t.created_at.clone().unwrap_or_else(|| "—".into()),
                last_used: t.last_used.clone().unwrap_or_else(|| "never".into()),
            })
            .collect(),
        assignments: assignments
            .iter()
            .map(|a| AssignmentRow {
                instance_id: a.instance_id.clone(),
                label: a.instance_id.clone(),
            })
            .collect(),
        // (id, label) pairs for the assign dropdown
        instance_options: options
            .iter()
            .map(|s| {
                (
                    s.id.to_string(),
                    s.name
                        .clone()
                        .unwrap_or_else(|| s.phone.clone().unwrap_or_else(|| s.id.to_string())),
                )
            })
            .collect(),
    };
    html_page(tpl)
}

#[derive(Deserialize)]
pub struct CreateTokenForm {
    pub name: String,
}

#[derive(Template)]
#[template(path = "web/_token_revealed.html")]
pub struct TokenRevealedTemplate {
    pub secret: String,
}

/// `POST /app/users/:id/tokens` — mint a token; secret rendered exactly once
pub async fn create_token_post(
    State(app): State<super::pages::AppState>,
    session: WebSessionExt,
    Path(user_id): Path<String>,
    Form(form): Form<CreateTokenForm>,
) -> Response {
    if require_admin(&session).is_err() {
        return forbidden_fragment();
    }
    let name = form.name.trim().to_string();
    if name.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            axum::response::Html(error_fragment("Token name is required.")),
        )
            .into_response();
    }
    let secret = format!("was_{}", Uuid::new_v4().simple());
    let hash = crate::middleware::auth::hash_token(&secret);
    match app
        .db
        .create_access_token(&Uuid::new_v4().to_string(), &user_id, &name, &hash, None)
    {
        Ok(_) => {
            tracing::info!(user_id = %user_id, "web created access token");
            let tpl = TokenRevealedTemplate { secret };
            (
                StatusCode::OK,
                axum::response::Html(tpl.render().unwrap_or_default()),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::response::Html(error_fragment(&format!("Token creation failed: {e}"))),
        )
            .into_response(),
    }
}

/// `POST /app/users/:id/tokens/:token_id/revoke`
pub async fn revoke_token_post(
    State(app): State<super::pages::AppState>,
    session: WebSessionExt,
    Path((user_id, token_id)): Path<(String, String)>,
) -> Response {
    if require_admin(&session).is_err() {
        return forbidden_fragment();
    }
    match app.db.delete_access_token(&token_id, &user_id) {
        Ok(()) => toast("success", "Token revoked."),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::response::Html(error_fragment(&format!("Revoke failed: {e}"))),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
pub struct AssignForm {
    pub instance_id: String,
}

/// `POST /app/users/:id/assign`
pub async fn assign_post(
    State(app): State<super::pages::AppState>,
    session: WebSessionExt,
    Path(user_id): Path<String>,
    Form(form): Form<AssignForm>,
) -> Response {
    if require_admin(&session).is_err() {
        return forbidden_fragment();
    }
    match app.db.assign_instance_to_user(
        &user_id,
        &form.instance_id,
        crate::domain::identity::InstancePermission::Operator,
    ) {
        Ok(_) => toast("success", "Instance assigned."),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            axum::response::Html(error_fragment(&format!("Assign failed: {e}"))),
        )
            .into_response(),
    }
}

/// `POST /app/users/:id/unassign/:instance_id`
pub async fn unassign_post(
    State(app): State<super::pages::AppState>,
    session: WebSessionExt,
    Path((user_id, instance_id)): Path<(String, String)>,
) -> Response {
    if require_admin(&session).is_err() {
        return forbidden_fragment();
    }
    match app.db.remove_instance_from_user(&user_id, &instance_id) {
        Ok(()) => toast("success", "Instance unassigned."),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::response::Html(error_fragment(&format!("Unassign failed: {e}"))),
        )
            .into_response(),
    }
}

/// `GET /app/fragments/users/:id/assignments` — refreshed after assign/unassign
pub async fn assignments_fragment(
    State(app): State<super::pages::AppState>,
    session: WebSessionExt,
    Path(user_id): Path<String>,
) -> Response {
    if require_admin(&session).is_err() {
        return forbidden_fragment();
    }
    let assignments = app.db.list_user_instances(&user_id).unwrap_or_default();
    let snapshots = app.manager.list_status_snapshots().await;
    let label_for = |id: &str| {
        snapshots
            .iter()
            .find(|s| s.id.to_string() == id)
            .map(|s| {
                s.name
                    .clone()
                    .unwrap_or_else(|| s.phone.clone().unwrap_or_else(|| s.id.to_string()))
            })
            .unwrap_or_else(|| id.to_string())
    };
    let tpl = AssignmentsTemplate {
        csrf: session.csrf_token.clone(),
        user_id,
        assignments: assignments
            .iter()
            .map(|a| AssignmentRow {
                instance_id: a.instance_id.clone(),
                label: label_for(&a.instance_id),
            })
            .collect(),
    };
    (
        StatusCode::OK,
        axum::response::Html(tpl.render().unwrap_or_default()),
    )
        .into_response()
}

#[derive(Template)]
#[template(path = "web/_assignment_list.html")]
pub struct AssignmentsTemplate {
    pub csrf: String,
    pub user_id: String,
    pub assignments: Vec<AssignmentRow>,
}

// ---------------------------------------------------------------------------
// Profile (/app/me)
// ---------------------------------------------------------------------------

#[derive(Template)]
#[template(path = "web/pages/me.html")]
pub struct MeTemplate {
    pub title: String,
    pub version: &'static str,
    pub csrf: String,
    pub username: String,
    pub role: String,
    pub admin: bool,
    pub email: String,
    pub instances: Vec<AssignmentRow>,
}

/// `GET /app/me` — own profile; resolves identity from the session cookie
pub async fn me_page(
    State(app): State<super::pages::AppState>,
    session: WebSessionExt,
) -> Response {
    let record = app
        .db
        .get_user_by_username(&session.username)
        .ok()
        .flatten();
    let email = record
        .as_ref()
        .and_then(|u| u.email.clone())
        .unwrap_or_else(|| "—".into());
    let instances = match &record {
        Some(u) => app.db.list_user_instances(&u.id).unwrap_or_default(),
        None => Vec::<InstanceOwnerRecord>::new(),
    };
    let tpl = MeTemplate {
        title: "Profile — WAS Admin".into(),
        version: VERSION,
        csrf: session.csrf_token.clone(),
        username: session.username.clone(),
        role: session.role.clone(),
        admin: session.admin,
        email,
        instances: instances
            .iter()
            .map(|a| AssignmentRow {
                instance_id: a.instance_id.clone(),
                label: a.instance_id.clone(),
            })
            .collect(),
    };
    html_page(tpl)
}
