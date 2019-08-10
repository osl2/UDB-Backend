use serde::{Deserialize, Serialize};
use uuid::Uuid;
use rand::random;
use argon2rs;
use base64;
use crate::schema::users;
use diesel::{Insertable, Queryable};

/// User: This struct describes a user as it is stored in the databse
#[derive(Debug, Serialize, Deserialize, Queryable, Insertable, Clone, AsChangeset)]
pub struct User {
    pub id: String,
    pub name: String,
    pub password_hash: String,
    pub salt: String,
}

impl User {
    pub fn new(name: String, password: String) -> User {
        let salt : String = (0..32).map(|_| random::<char>()).collect();
        let id = Uuid::new_v4().to_hyphenated().to_string();
        let hash = base64::encode(&argon2rs::argon2d_simple(&password, &salt));
        User {
            id: id,
            name: name,
            password_hash: hash,
            salt: salt,
        }
    }
}