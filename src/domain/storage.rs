use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct Storage {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<String>)]
    pub id: Option<ObjectId>,
    pub name: String,

    #[serde(rename = "baseName")]
    pub base_name: Option<String>,

    #[serde(rename = "extName")]
    pub ext_name: Option<String>,

    #[serde(rename = "mimeType")]
    pub mime_type: Option<String>,

    pub encoding: Option<String>,
    pub size: Option<i64>,

    #[serde(rename = "MD5Hash")]
    pub md5_hash: Option<String>,

    pub iv: Option<String>,

    #[serde(rename = "parentId")]
    pub parent_id: String,

    pub r#type: String, // "file" or "folder"

    #[serde(rename = "userId")]
    pub user_id: String,

    pub thumbnail: Option<String>,

    pub trashed: bool,

    #[serde(rename = "createdAt", skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,

    #[serde(rename = "updatedAt", skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}
