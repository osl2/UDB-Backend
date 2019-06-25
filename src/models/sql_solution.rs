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
pub struct SqlSolution {
    #[serde(rename = "rows")]
    pub rows: Vec<String>,
    #[serde(rename = "query")]
    pub query: String,
}

impl SqlSolution {
    pub fn new(rows: Vec<String>, query: String) -> SqlSolution {
        SqlSolution {
            rows: rows,
            query: query,
        }
    }
}


