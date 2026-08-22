//! Interfaces layer — HTTP handlers, DTOs, router, middleware
//!
//! Keeps `handlers/api/users.rs:845` decomposed and `bin/was.rs:406` thin per #9 #10 #11.
//! Handlers map DTO ↔ domain via `TryFrom`.

pub mod http;
