/*
 * dbsquared
 *
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 1.0.0
 * Contact: jan.christian@gruenhage.xyz
 * Generated by: https://openapi-generator.tech
 */

/// Database : The root of the Database type's schema.

use serde::{Serialize, Deserialize};
use diesel::{Queryable, Insertable};
use crate::schema::databases;

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable, Clone, AsChangeset)]
pub struct Database {
    #[serde(rename = "id")]
    pub id: String,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "database")]
    pub content: String,
}
