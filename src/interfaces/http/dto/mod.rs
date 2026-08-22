//! DTO layer — versioned, `utoipa::Schema` + `serde`, maps to `domain` via `TryFrom`
//! Extracted from `models/mod.rs:13` + `handlers/api/instances.rs:36..431` (part of #11)
//! Domain changes no longer break OpenAPI — DTO is the stability boundary.

pub mod health;
pub mod identity;
pub mod instance;
pub mod messaging;
