use crate::models::Content;
use crate::schema::subtasks;
use diesel::backend;
use diesel::deserialize;
use diesel::serialize;
use diesel::sql_types::Integer;
use diesel::sqlite::Sqlite;
use serde::{Deserialize, Serialize};

#[repr(i32)]
#[derive(Debug, Clone, Copy, FromSqlRow, Serialize, Deserialize, AsExpression)]
#[diesel(sql_type = Integer)]
pub enum AllowedSQL {
    ALL = 0,
    QUERY = 1,
}

impl deserialize::FromSql<Integer, Sqlite> for AllowedSQL {
    fn from_sql(bytes: backend::RawValue<'_, Sqlite>) -> deserialize::Result<Self> {
        match i32::from_sql(bytes)? {
            0 => Ok(AllowedSQL::ALL),
            1 => Ok(AllowedSQL::QUERY),
            x => Err(format!("Unrecognized variant {}", x).into()),
        }
    }
}

impl serialize::ToSql<Integer, Sqlite> for AllowedSQL {
    fn to_sql<'b>(&'b self, out: &mut serialize::Output<'b, '_, Sqlite>) -> serialize::Result {
        out.set_value(diesel::sqlite::SqliteBindValue::from(*self as i32));
        Ok(serialize::IsNull::No)
    }
}

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable, AsChangeset)]
pub struct Subtask {
    #[serde(rename = "id")]
    pub id: String,
    #[serde(rename = "instruction")]
    pub instruction: String,
    #[serde(rename = "solution_verifiable")]
    pub is_solution_verifiable: bool,
    #[serde(rename = "solution_visible")]
    pub is_solution_visible: bool,
    #[serde(rename = "content")]
    pub content: Content,
}
