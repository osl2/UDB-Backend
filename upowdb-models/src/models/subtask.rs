use crate::models::Content;
use crate::schema::subtasks;
use diesel::backend;
use diesel::deserialize;

use diesel::serialize;
use diesel::sql_types::Integer;
use serde::{Deserialize, Serialize};
use std::io::Write;

#[repr(i32)]
#[derive(Debug, Clone, Copy, FromSqlRow, Serialize, Deserialize, AsExpression)]
#[sql_type = "Integer"]
pub enum AllowedSQL {
    ALL = 0,
    QUERY = 1,
}

impl<DB> deserialize::FromSql<Integer, DB> for AllowedSQL
where
    DB: backend::Backend,
    i32: deserialize::FromSql<Integer, DB>,
{
    fn from_sql(bytes: Option<&DB::RawValue>) -> deserialize::Result<Self> {
        match i32::from_sql(bytes)? {
            0 => Ok(AllowedSQL::ALL),
            1 => Ok(AllowedSQL::QUERY),
            x => Err(format!("Unrecognized variant {}", x).into()),
        }
    }
}

impl<DB> serialize::ToSql<Integer, DB> for AllowedSQL
where
    DB: backend::Backend,
    i32: serialize::ToSql<Integer, DB>,
{
    fn to_sql<W: Write>(&self, out: &mut serialize::Output<W, DB>) -> serialize::Result {
        (*self as i32).to_sql(out)
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
