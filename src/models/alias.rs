use crate::schema::aliases;
use diesel::types::Integer;
use diesel::{backend, deserialize, serialize, Insertable, Queryable};
use serde::{Deserialize, Serialize};
use std::io::Write;

#[repr(i32)]
#[derive(Debug, Clone, Copy, FromSqlRow, Serialize, Deserialize, AsExpression)]
#[sql_type = "Integer"]
pub enum ObjectType {
    COURSE = 0,
    WORKSHEET = 1,
    TASK = 2,
    SUBTASK = 3,
    DATABASE = 4,
}

impl<DB> deserialize::FromSql<Integer, DB> for ObjectType
where
    DB: backend::Backend,
    i32: deserialize::FromSql<Integer, DB>,
{
    fn from_sql(bytes: Option<&DB::RawValue>) -> deserialize::Result<Self> {
        match i32::from_sql(bytes)? {
            0 => Ok(ObjectType::COURSE),
            1 => Ok(ObjectType::WORKSHEET),
            2 => Ok(ObjectType::TASK),
            3 => Ok(ObjectType::SUBTASK),
            4 => Ok(ObjectType::DATABASE),
            x => Err(format!("Unrecognized variant {}", x).into()),
        }
    }
}

impl<DB> serialize::ToSql<Integer, DB> for ObjectType
where
    DB: backend::Backend,
    i32: serialize::ToSql<Integer, DB>,
{
    fn to_sql<W: Write>(&self, out: &mut serialize::Output<W, DB>) -> serialize::Result {
        (*self as i32).to_sql(out)
    }
}

#[derive(Debug, Deserialize)]
pub struct AliasRequest {
    pub object_id: String,
    pub object_type: ObjectType,
}

#[derive(Debug, Clone, Queryable, Insertable, Serialize)]
#[table_name = "aliases"]
pub struct Alias {
    pub alias: String,
    pub object_id: String,
    pub object_type: ObjectType,
}
