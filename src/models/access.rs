use diesel::{Queryable, Insertable};
use crate::schema::access;

#[derive(Debug, Queryable, Insertable, Clone, AsChangeset)]
#[table_name = "access"]
pub struct Access {
    pub user_id: String,
    pub object_id: String,
}