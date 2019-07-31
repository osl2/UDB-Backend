/*
 * dbsquared
 *
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 1.0.0
 * Contact: jan.christian@gruenhage.xyz
 * Generated by: https://openapi-generator.tech
 */

use serde::{Serialize, Deserialize};
use diesel::prelude::*;
use crate::schema::tasks;
use crate::schema::subtasks_in_tasks;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    #[serde(rename = "id")]
    pub id: String,
    #[serde(rename = "database")]
    pub database_id: String,
    #[serde(rename = "subtasks", skip_serializing_if = "Option::is_none")]
    pub subtasks: Option<Vec<String>>,
}

#[derive(Debug, Queryable, Insertable, AsChangeset)]
#[table_name = "tasks"]
pub struct QueryableTask {
    pub id: String,
    pub database_id: String,
}

impl QueryableTask {
    pub fn from_task(task: Task) -> Self {
        Self {
            id: task.id,
            database_id: task.database_id,
        }
    }
}

#[derive(Debug, Queryable, Insertable)]
pub struct SubtasksInTask {
    pub subtask_id: String,
    pub task_id: String,
    pub position: i32,
}
