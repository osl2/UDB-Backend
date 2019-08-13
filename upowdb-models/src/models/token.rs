use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Token {
    #[serde(rename = "token", skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
}
