use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, utoipa::ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum StorageType {
    File,
    Folder,
    Thumbnail,
}

/// 列表单条：不含敏感字段，id 为 hex，createdAt/updatedAt 为 RFC3339，url/thumbnail 为完整 URL。
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct StorageListResponse {
    #[serde(
        rename = "id",
        serialize_with = "crate::utils::serde_json_response::serialize_object_id_as_hex"
    )]
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
    pub r#type: StorageType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    pub trashed: bool,
    #[serde(
        rename = "createdAt",
        serialize_with = "crate::utils::serde_json_response::serialize_optional_datetime_as_rfc3339"
    )]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(
        rename = "updatedAt",
        serialize_with = "crate::utils::serde_json_response::serialize_optional_datetime_as_rfc3339"
    )]
    pub updated_at: Option<DateTime<Utc>>,
}

/// 创建文件夹接口返回：id, name, parentId, type, createdAt, updatedAt, exist（已存在为 true，新建为 false）。
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct CreateFolderResponse {
    #[serde(
        rename = "id",
        serialize_with = "crate::utils::serde_json_response::serialize_object_id_as_hex"
    )]
    #[schema(value_type = String)]
    pub id: Option<ObjectId>,
    pub name: String,
    #[serde(rename = "parentId")]
    pub parent_id: String,
    #[serde(rename = "type")]
    pub r#type: StorageType,
    #[serde(
        rename = "createdAt",
        serialize_with = "crate::utils::serde_json_response::serialize_optional_datetime_as_rfc3339"
    )]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(
        rename = "updatedAt",
        serialize_with = "crate::utils::serde_json_response::serialize_optional_datetime_as_rfc3339"
    )]
    pub updated_at: Option<DateTime<Utc>>,
    pub exist: bool,
}

/// PUT /storage/:id 返回：id, name, parentId, type, userId, trashed, createdAt, updatedAt, baseName, extName。
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct UpdateStorageResponse {
    #[serde(
        rename = "id",
        serialize_with = "crate::utils::serde_json_response::serialize_object_id_as_hex"
    )]
    #[schema(value_type = String)]
    pub id: Option<ObjectId>,
    pub name: String,
    #[serde(rename = "parentId")]
    pub parent_id: String,
    #[serde(rename = "type")]
    pub r#type: StorageType,
    #[serde(rename = "userId")]
    pub user_id: String,
    pub trashed: bool,
    #[serde(
        rename = "createdAt",
        serialize_with = "crate::utils::serde_json_response::serialize_optional_datetime_as_rfc3339"
    )]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(
        rename = "updatedAt",
        serialize_with = "crate::utils::serde_json_response::serialize_optional_datetime_as_rfc3339"
    )]
    pub updated_at: Option<DateTime<Utc>>,
    #[serde(rename = "baseName")]
    pub base_name: Option<String>,
    #[serde(rename = "extName")]
    pub ext_name: Option<String>,
}

/// 路径节点（get_path 返回的从 root 到当前文件/文件夹的每一级）。
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct StoragePathNode {
    pub id: String,
    pub name: String,
    #[serde(rename = "parentId")]
    pub parent_id: String,
    #[serde(rename = "type")]
    pub r#type: StorageType,
}

/// 分页列表响应：`{ docs, total, limit, page, pages }`。
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct StorageListPaginatedResponse {
    pub docs: Vec<StorageListResponse>,
    pub total: u64,
    pub limit: u64,
    pub page: u64,
    pub pages: u64,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct TrashCleanupResponse {
    #[serde(rename = "deletedDocs")]
    pub deleted_docs: u64,
    #[serde(rename = "deletedFiles")]
    pub deleted_files: u64,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct TrashRestoreResponse {
    #[serde(rename = "requestedItems")]
    pub requested_items: u64,
    #[serde(rename = "restoredDocs")]
    pub restored_docs: u64,
}

impl StorageListResponse {
    /// 从 Storage 构建列表项，并填入 base_url + token 生成的 url/thumbnail 完整 URL（仅文件有 url）。
    /// `thumb_share_versions` maps thumbnail id hex → that doc's shareVersion.
    pub fn from_storage_with_urls(
        s: Storage,
        base_url: &str,
        user_id: &str,
        secret: &str,
        thumb_share_versions: &std::collections::HashMap<String, i32>,
    ) -> Result<Self, crate::api::error::AppError> {
        let base = base_url.trim_end_matches('/');
        let expires_at = (Utc::now().timestamp() + 900) as usize;
        let url = if s.r#type == StorageType::File {
            let Some(file_id) = s.id.as_ref().map(|id| id.to_hex()) else {
                return Err(crate::api::error::AppError::InternalError(
                    "Stored file is missing an id".into(),
                ));
            };
            let token = crate::api::middleware::create_token(
                secret,
                user_id,
                crate::api::middleware::TokenPurpose::Download,
                Some(file_id.clone()),
                expires_at,
                Some(s.share_version),
            )?;
            Some(format!("{base}/v1/storage/{file_id}?token={token}"))
        } else {
            None
        };
        let thumbnail = match s.thumbnail.as_ref() {
            Some(thumb_id) => {
                let thumb_version = thumb_share_versions.get(thumb_id).copied().unwrap_or(0);
                let token = crate::api::middleware::create_token(
                    secret,
                    user_id,
                    crate::api::middleware::TokenPurpose::Download,
                    Some(thumb_id.clone()),
                    expires_at,
                    Some(thumb_version),
                )?;
                Some(format!("{base}/v1/storage/{thumb_id}?token={token}"))
            }
            None => None,
        };
        Ok(Self {
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
        })
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

impl From<Storage> for UpdateStorageResponse {
    fn from(s: Storage) -> Self {
        let base_name = if s.r#type == StorageType::Folder {
            s.base_name.or(Some(s.name.clone()))
        } else {
            s.base_name
        };
        Self {
            id: s.id,
            name: s.name,
            parent_id: s.parent_id,
            r#type: s.r#type,
            user_id: s.user_id,
            trashed: s.trashed,
            created_at: s.created_at,
            updated_at: s.updated_at,
            base_name,
            ext_name: s.ext_name,
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

    #[serde(
        rename = "contentHash",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub content_hash: Option<String>,

    #[serde(
        rename = "hashAlgorithm",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub hash_algorithm: Option<String>,

    #[serde(
        rename = "encryptionFormat",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub encryption_format: Option<u8>,

    #[serde(rename = "shareVersion", default)]
    pub share_version: i32,

    #[serde(rename = "parentId")]
    pub parent_id: String,

    #[serde(rename = "type")]
    pub r#type: StorageType,

    #[serde(rename = "userId")]
    pub user_id: String,

    pub thumbnail: Option<String>,

    pub trashed: bool,

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
