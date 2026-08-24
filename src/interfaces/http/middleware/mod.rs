//! Middleware stack — extracted from `bin/was.rs:361` ServiceBuilder (part of #10)
//! Re-exports `middleware::{correlation_id, request_metrics, security_headers}` and provides
//! a single `http_middleware_stack` builder for `router::build_router`.

pub mod stack;
pub use stack::http_middleware_stack;
