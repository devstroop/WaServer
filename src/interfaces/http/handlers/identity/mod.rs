//! Identity handlers — split from `handlers/api/users.rs:845` per #9
//! Each handler validates via `domain::identity`, calls `application::identity::*`, no `rusqlite`.

pub mod assignments;
pub mod me;
pub mod tokens;
pub mod users;
