/*
 * dbsquared
 *
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 1.0.0
 * Contact: jan.christian@gruenhage.xyz
 * Generated by: https://openapi-generator.tech
 */

/// AccountCreation : This struct is passed to the registration endpoint for creating an account.

#[allow(unused_imports)]
use serde_json::Value;


#[derive(Debug, Serialize, Deserialize)]
pub struct AccountCreation {
    #[serde(rename = "username")]
    pub username: String,
    #[serde(rename = "password")]
    pub password: String,
}

impl AccountCreation {
    /// This struct is passed to the registration endpoint for creating an account.
    pub fn new(username: String, password: String) -> AccountCreation {
        AccountCreation {
            username: username,
            password: password,
        }
    }
}


