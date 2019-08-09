mod middleware;
pub use middleware::{Algorithm, JwtAuthentication};
use std::path::PathBuf;

#[derive(Clone)]
pub enum JwtKey {
    Inline(String),
    File(PathBuf),
}

pub struct AuthenticationData {
    header: serde_json::Value,
    claims: Claims,
}

pub struct Claims {
    sub: Option<String>,
    exp: Option<i64>,
    all: serde_json::Value,
}
