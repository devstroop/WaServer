//! Embedded static assets for the web admin UI (#28)
//!
//! Release builds embed `assets/web/**` into the binary (single-artifact deploys);
//! debug builds read from disk so asset edits show up on refresh.

use axum::{
    http::{header, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "assets/web"]
struct WebAssets;

async fn serve_asset(axum::extract::Path(path): axum::extract::Path<String>) -> impl IntoResponse {
    // Keys are relative to the embedded folder ("assets/web")
    let rel = path.trim_start_matches('/');
    match WebAssets::get(rel) {
        Some(file) => {
            let mime = mime_guess::from_path(rel)
                .first_or_octet_stream()
                .to_string();
            // Vendor files are content-stable between releases; long cache is safe
            let cache = if rel.starts_with("vendor/") {
                "public, max-age=604800"
            } else {
                "public, max-age=300"
            };
            (
                [
                    (header::CONTENT_TYPE, mime),
                    (header::CACHE_CONTROL, cache.to_string()),
                ],
                file.data,
            )
                .into_response()
        }
        None => (StatusCode::NOT_FOUND, "not found").into_response(),
    }
}

/// `GET /assets/web/*path` — embedded UI assets (htmx, uikit dist, tokens, app js/css)
pub fn router() -> Router {
    Router::new().route("/*path", get(serve_asset))
}
