use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

/// API response shape without password. JSON `_id` is hex; createdAt/updatedAt are millisecond timestamp strings.
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct UserResponse {
    #[serde(
        rename = "id",
        serialize_with = "crate::utils::serde_json_response::serialize_object_id_as_hex"
    )]
    #[schema(value_type = String)]
    pub id: Option<ObjectId>,
    pub email: String,
    #[serde(rename = "firstName")]
    pub first_name: String,
    #[serde(rename = "lastName")]
    pub last_name: String,
    pub gender: Option<i32>,
    pub age: Option<i32>,
    pub avatar: Option<String>,
    pub roles: Vec<String>,
    #[serde(
        rename = "createdAt",
        serialize_with = "crate::utils::serde_json_response::serialize_optional_datetime_as_ms_string"
    )]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(
        rename = "updatedAt",
        serialize_with = "crate::utils::serde_json_response::serialize_optional_datetime_as_ms_string"
    )]
    pub updated_at: Option<DateTime<Utc>>,
}

impl From<User> for UserResponse {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            email: u.email,
            first_name: u.first_name,
            last_name: u.last_name,
            gender: u.gender,
            age: u.age,
            avatar: u.avatar,
            roles: u.roles,
            created_at: u.created_at,
            updated_at: u.updated_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<String>)]
    pub id: Option<ObjectId>,
    pub email: String,
    /// Stored in DB only; omitted from API responses via UserResponse.
    pub password: String,

    #[serde(rename = "firstName")]
    pub first_name: String,

    #[serde(rename = "lastName")]
    pub last_name: String,

    pub gender: Option<i32>,
    pub age: Option<i32>,
    pub avatar: Option<String>,

    pub roles: Vec<String>,

    #[serde(rename = "tokenVersion", default)]
    pub token_version: i32,

    #[serde(
        rename = "createdAt",
        skip_serializing_if = "Option::is_none",
        with = "crate::utils::serde_datetime"
    )]
    pub created_at: Option<DateTime<Utc>>,

    #[serde(
        rename = "updatedAt",
        skip_serializing_if = "Option::is_none",
        with = "crate::utils::serde_datetime"
    )]
    pub updated_at: Option<DateTime<Utc>>,
}
