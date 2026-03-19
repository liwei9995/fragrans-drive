use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

/// 列表单条：不含敏感字段，id 为 hex，createdAt/updatedAt 为 RFC3339，url/thumbnail 为完整 URL。
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct StorageListResponse {
    #[serde(rename = "id", serialize_with = "crate::utils::serde_json_response::serialize_object_id_as_hex")]
    #[schema(value_type = String)]
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
    #[serde(rename = "parentId")]
    pub parent_id: String,
    #[serde(rename = "type")]
    pub r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    pub trashed: bool,
    #[serde(rename = "createdAt", serialize_with = "crate::utils::serde_json_response::serialize_optional_datetime_as_rfc3339")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(rename = "updatedAt", serialize_with = "crate::utils::serde_json_response::serialize_optional_datetime_as_rfc3339")]
    pub updated_at: Option<DateTime<Utc>>,
}

/// 路径节点（get_path 返回的从 root 到当前文件/文件夹的每一级）。
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct StoragePathNode {
    pub id: String,
    pub name: String,
    #[serde(rename = "parentId")]
    pub parent_id: String,
    #[serde(rename = "type")]
    pub r#type: String,
}

/// 分页列表响应：`{ docs, total, limit, page, pages }`。
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct StorageListPaginatedResponse {
    pub docs: Vec<StorageListResponse>,
    pub total: u64,
    pub limit: u64,
    pub page: u64,
    pub pages: u64,
}

impl StorageListResponse {
    /// 从 Storage 构建列表项，并填入 base_url + token 生成的 url/thumbnail 完整 URL（仅文件有 url）。
    pub fn from_storage_with_urls(
        s: Storage,
        base_url: &str,
        token: &str,
    ) -> Self {
        let base = base_url.trim_end_matches('/');
        let url = (s.r#type == "file" && s.id.is_some()).then(|| {
            format!("{}/v1/storage/{}?token={}", base, s.id.as_ref().unwrap().to_string(), token)
        });
        let thumbnail = s.thumbnail.as_ref().map(|thumb_id| {
            format!("{}/v1/storage/{}?token={}", base, thumb_id, token)
        });
        Self {
            id: s.id,
            name: s.name,
            base_name: s.base_name,
            ext_name: s.ext_name,
            mime_type: s.mime_type,
            encoding: s.encoding,
            size: s.size,
            parent_id: s.parent_id,
            r#type: s.r#type,
            thumbnail,
            url,
            trashed: s.trashed,
            created_at: s.created_at,
            updated_at: s.updated_at,
        }
    }
}

impl From<Storage> for StorageListResponse {
    fn from(s: Storage) -> Self {
        Self {
            id: s.id,
            name: s.name,
            base_name: s.base_name,
            ext_name: s.ext_name,
            mime_type: s.mime_type,
            encoding: s.encoding,
            size: s.size,
            parent_id: s.parent_id,
            r#type: s.r#type,
            thumbnail: s.thumbnail,
            url: None,
            trashed: s.trashed,
            created_at: s.created_at,
            updated_at: s.updated_at,
        }
    }
}

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
