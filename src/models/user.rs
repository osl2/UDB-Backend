use crate::schema::users;
use argon2rs;
use base64;
use diesel::{Insertable, Queryable};
use rand::random;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// User: This struct describes a user as it is stored in the databse
#[derive(Debug, Serialize, Deserialize, Queryable, Insertable, Clone, AsChangeset)]
pub struct User {
    id: String,
    pub name: String,
    pub password_hash: String,
    pub salt: String,
}

impl User {
    pub fn new(name: String, password: String, id: Option<Uuid>) -> User {
        let salt: String = (0..32).map(|_| random::<char>()).collect();
        let id = match id {
            Some(id) => id.to_hyphenated().to_string(),
            None => Uuid::new_v4().to_hyphenated().to_string(),
        };
        let password_hash = base64::encode(&argon2rs::argon2d_simple(&password, &salt));
        User {
            id,
            name,
            password_hash,
            salt,
        }
    }
    pub fn verify_password(&self, password: String) -> bool {
        self.password_hash == base64::encode(&argon2rs::argon2d_simple(&password, &self.salt))
    }
    pub fn returnable_userdata(&self) -> serde_json::Value {
        let mut map = serde_json::map::Map::new();
        map.insert("id".to_string(), serde_json::Value::String(self.id.clone()));
        map.insert(
            "name".to_string(),
            serde_json::Value::String(self.name.clone()),
        );
        serde_json::Value::Object(map)
    }
    pub fn get_raw_id(&self) -> String {
        self.id.clone()
    }
    pub fn get_id(&self) -> Uuid {
        Uuid::parse_str(&self.id).unwrap()
    }
}
