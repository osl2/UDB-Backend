use crate::schema::subtasks_in_tasks;
use crate::schema::tasks;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    #[serde(rename = "id")]
    pub id: String,
    #[serde(rename = "database")]
    pub database_id: String,
    #[serde(rename = "subtasks")]
    pub subtasks: Vec<String>,
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
