use crate::schema::access;
use diesel::{Insertable, Queryable};

#[derive(Debug, Queryable, Insertable, Clone, AsChangeset)]
#[table_name = "access"]
pub struct Access {
    pub user_id: String,
    pub object_id: String,
}
