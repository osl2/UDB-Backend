use crate::schema::tasks_in_worksheets;
use crate::schema::worksheets;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Worksheet {
    #[serde(rename = "id")]
    pub id: String,
    #[serde(rename = "name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "is_online")]
    pub is_online: bool,
    #[serde(rename = "is_solution_online")]
    pub is_solution_online: bool,
    #[serde(rename = "tasks")]
    pub tasks: Vec<String>,
}

#[derive(Queryable, Insertable, AsChangeset)]
#[table_name = "worksheets"]
pub struct QueryableWorksheet {
    pub id: String,
    pub name: Option<String>,
    pub is_online: bool,
    pub is_solution_online: bool,
}

impl QueryableWorksheet {
    pub fn from_worksheet(worksheet: Worksheet) -> Self {
        Self {
            id: worksheet.id,
            name: worksheet.name,
            is_online: worksheet.is_online,
            is_solution_online: worksheet.is_solution_online,
        }
    }
}

#[derive(Debug, Queryable, Insertable)]
pub struct TasksInWorksheet {
    pub task_id: String,
    pub worksheet_id: String,
    pub position: i32,
}
