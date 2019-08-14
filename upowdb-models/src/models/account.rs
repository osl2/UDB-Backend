/// Account: This struct is passed to the registration endpoint for creating an account,
/// and to the account endpoint for modifying an existing one.
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    #[serde(rename = "username")]
    pub username: String,
    #[serde(rename = "password")]
    pub password: String,
}
