use crate::schema::aliases;
use diesel::sql_types::Integer;
use diesel::sqlite::Sqlite;
use diesel::{backend, deserialize, serialize, Insertable, Queryable};
use serde::{Deserialize, Serialize};

#[repr(i32)]
#[derive(Debug, Clone, Copy, FromSqlRow, Serialize, Deserialize, AsExpression)]
#[diesel(sql_type = Integer)]
pub enum ObjectType {
    COURSE = 0,
    WORKSHEET = 1,
    TASK = 2,
    SUBTASK = 3,
    DATABASE = 4,
}

impl deserialize::FromSql<Integer, Sqlite> for ObjectType {
    fn from_sql(bytes: backend::RawValue<'_, Sqlite>) -> deserialize::Result<Self> {
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

impl serialize::ToSql<Integer, Sqlite> for ObjectType {
    fn to_sql<'b>(&'b self, out: &mut serialize::Output<'b, '_, Sqlite>) -> serialize::Result {
        out.set_value(diesel::sqlite::SqliteBindValue::from(*self as i32));
        Ok(serialize::IsNull::No)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AliasRequest {
    pub object_id: String,
    pub object_type: ObjectType,
}

#[derive(Debug, Clone, Queryable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = aliases)]
pub struct Alias {
    pub alias: String,
    pub object_id: String,
    pub object_type: ObjectType,
}
