/*
 * dbsquared
 *
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 1.0.0
 * Contact: jan.christian@gruenhage.xyz
 * Generated by: https://openapi-generator.tech
 */

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Solution {
    #[serde(rename = "sql")]
    SQL(SQLSolution),
    #[serde(rename = "multiple_choice")]
    MultipleChoice(MCSolution),
    #[serde(rename = "plaintext")]
    Text(PlaintextSolution),
}

#[derive(Debug, Serialize)]
pub enum SolutionResult {
    #[serde(rename = "sql")]
    SQL(SQLSolutionResult),
    #[serde(rename = "multiple_choice")]
    MultipleChoice(MCSolutionResult),
    #[serde(rename = "plaintext")]
    Text(PlaintextSolutionResult),
    #[serde(rename = "error")]
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SQLSolution {
    pub query: String,
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

/// Result of the comparison of two SQLSolutions
#[derive(Debug, Serialize)]
pub struct SQLSolutionResult {
    pub correct: bool,
    pub missed_rows: Vec<Vec<String>>,
    pub wrong_rows: Vec<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCSolution {
    pub correct_positions: Vec<i64>,
}

#[derive(Debug, Serialize)]
pub struct MCSolutionResult {
    pub correct: bool,
    pub wrong_choices: Vec<i64>,
    pub missed_choices: Vec<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaintextSolution {
    pub text: String,
}

#[derive(Debug, Serialize)]
pub struct PlaintextSolutionResult {
    pub correct: bool,
    pub correct_answer: String,
}
