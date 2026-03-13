use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub email: String,

    #[serde(skip_serializing)]
    pub password: String,

    #[serde(rename = "firstName")]
    pub first_name: String,

    #[serde(rename = "lastName")]
    pub last_name: String,

    pub gender: Option<i32>,
    pub age: Option<i32>,
    pub avatar: Option<String>,

    pub roles: Vec<String>,

    #[serde(rename = "createdAt", skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,

    #[serde(rename = "updatedAt", skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}
