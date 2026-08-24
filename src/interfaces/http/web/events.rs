//! SSE live updates — server-pushed overview + status fragments (#58)
//!
//! `GET /app/events` is a guarded SSE stream. The server pushes `overview`
//! events every 2s (heartbeat) using the same render path as the polled
//! fragment, so status flips appear <1s after the next tick. Polling on the
//! client is replaced by `hx-sse:swap="overview"`.
//!
//! The broadcast channel is shared via `AppState::events`.

use axum::{
    body::Body,
    extract::State,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use tokio_stream::{wrappers::BroadcastStream, StreamExt};

use super::pages::AppState;

/// Spawn the periodic broadcaster that renders the overview fragment
/// and publishes it. Runs for the lifetime of the process.
/// No-op when called outside a Tokio runtime (e.g. unit tests).
pub fn spawn_overview_broadcaster(app: AppState) {
    if tokio::runtime::Handle::try_current().is_err() {
        return;
    }
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(2));
        loop {
            interval.tick().await;
            let html = super::pages::render_overview_html(&app).await;
            // one-line data payload: newlines collapsed so the SSE wire is a single `data:` line
            let one_line = html.replace('\n', " ").replace('\r', "");
            let _ = app.events.send(one_line);
        }
    });
}

/// `GET /app/events` — SSE stream (guarded; CSRF not required for GET).
/// Emits `event: overview` with the rendered fragment as `data:`.
pub async fn events(State(app): State<AppState>) -> Response {
    let rx = app.events.subscribe();
    let stream = BroadcastStream::new(rx)
        .filter_map(|res| res.ok())
        .map(|html| {
            // htmx sse extension expects `event: <name>` + `data: <html>`
            let wire = format!("event: overview\ndata: {}\n\n", html);
            Ok::<_, std::convert::Infallible>(wire)
        });

    // heartbeat keepalive: `: ping` comment every 15s keeps intermediaries warm
    let heartbeat = tokio_stream::wrappers::IntervalStream::new(tokio::time::interval(
        std::time::Duration::from_secs(15),
    ))
    .map(|_| Ok::<String, std::convert::Infallible>(": ping\n\n".to_string()));

    let body = Body::from_stream(tokio_stream::StreamExt::merge(stream, heartbeat));

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/event-stream")
        .header(header::CACHE_CONTROL, "no-cache")
        .header(header::CONNECTION, "keep-alive")
        .header("X-Accel-Buffering", "no")
        .body(body)
        .unwrap()
        .into_response()
}
