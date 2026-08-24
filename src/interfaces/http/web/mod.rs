//! Web admin UI — server-rendered htmx 4 + uikit (#27 #28)
//!
//! Routes live under `/app` (pages) and `/assets/web` (embedded static).
//! Auth: cookie sessions reusing the API session-token store; CSRF via
//! per-session derived tokens on unsafe methods.

pub mod assets;
pub mod csrf;
pub mod guard;
pub mod pages;
pub mod session;

use axum::Router;

use crate::middleware::auth::AuthState;

/// Mount points: `nest("/app", web::router(...))` + `nest("/assets/web", web::assets::router())`
pub fn router(auth_state: AuthState) -> Router {
    pages::router(auth_state)
}
