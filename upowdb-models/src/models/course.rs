use crate::schema::courses;
use crate::schema::worksheets_in_courses;

/// Course : The root of the Course type's schema.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Course {
    #[serde(rename = "id")]
    pub id: String,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "worksheets")]
    pub worksheets: Vec<String>,
}

#[derive(Debug, Queryable, Insertable, AsChangeset)]
#[table_name = "courses"]
pub struct QueryableCourse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}

impl QueryableCourse {
    pub fn from_course(course: Course) -> Self {
        Self {
            id: course.id,
            name: course.name,
            description: course.description,
        }
    }
}

#[derive(Debug, Queryable, Insertable)]
pub struct WorksheetsInCourse {
    pub worksheet_id: String,
    pub course_id: String,
    pub position: i32,
}
