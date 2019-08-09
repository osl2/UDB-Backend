mod middleware;
pub use middleware::{JwtAuthentication, Algorithm};
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
    all: serde_json::Value,
}