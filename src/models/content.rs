/*
 * dbsquared
 *
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 1.0.0
 * Contact: jan.christian@gruenhage.xyz
 * Generated by: https://openapi-generator.tech
 */


#[allow(unused_imports)]
use serde_json::Value;


#[derive(Debug, Serialize, Deserialize)]
pub enum Content {
    #[serde(rename = "sql")]
    SQL(::models::SqlContent),
    #[serde(rename = "multiple_choice")]
    MC(::models::McContent),
    Plaintext,
    Instruction,
}

impl Content {
    pub fn new() -> Content {
        Content::Instruction
    }
}


