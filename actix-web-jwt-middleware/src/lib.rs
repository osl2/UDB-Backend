mod middleware;
pub use middleware::{JwtAuthentication, Algorithm};
use std::path::PathBuf;

#[derive(Clone)]
pub enum JwtKey {
    Inline(String),
    File(PathBuf),
}

pub struct AuthenticationData(pub serde_json::Value);