mod middleware;
pub use middleware::{Algorithm, JwtAuthentication};
use std::path::PathBuf;

#[derive(Clone)]
pub enum JwtKey {
    Inline(String),
    File(PathBuf),
}

#[derive(Clone)]
pub struct AuthenticationData {
    pub header: serde_json::Value,
    pub claims: Claims,
}

#[derive(Clone)]
pub struct Claims {
    pub sub: Option<String>,
    pub exp: Option<i64>,
    pub all: serde_json::Value,
}
