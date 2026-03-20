use crate::api::AppState;
use crate::api::middleware::{Claims, UserContext};
use crate::domain::storage::{
    CreateFolderResponse, Storage, StorageListPaginatedResponse, StorageListResponse,
    StoragePathNode, TrashCleanupResponse, TrashRestoreResponse, UpdateStorageResponse,
};
use crate::infrastructure::db::storage_repo::StorageRepository;
use crate::infrastructure::image::thumbnail::generate_thumbnail;
use crate::infrastructure::storage::local::LocalStorage;
use crate::utils::encryption::get_iv;
use crate::utils::md5::hash_buffer;
use axum::{
    extract::{Json, Multipart, Path, Query, State},
    http::{StatusCode, header},
    response::IntoResponse,
};
use chrono::Utc;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use mongodb::bson::{DateTime as BsonDateTime, Document, doc, oid::ObjectId};
use serde::Deserialize;
use std::collections::HashSet;
use std::path::Path as StdPath;
use utoipa::ToSchema;

async fn collect_related_item_ids(
    repo: &StorageRepository,
    user_id: &str,
    root: &Storage,
) -> Result<Vec<ObjectId>, mongodb::error::Error> {
    let mut all_ids = Vec::new();
    let mut seen = HashSet::new();
    let mut frontier = match root.id {
        Some(id) => vec![id],
        None => return Ok(all_ids),
    };

    while !frontier.is_empty() {
        let mut next_frontier = Vec::new();
        for id in &frontier {
            if seen.insert(*id) {
                all_ids.push(*id);
            }
        }

        let parent_ids: Vec<String> = frontier.iter().map(|id| id.to_hex()).collect();
        let children = repo.find_many_by_parent_ids(parent_ids, user_id).await?;
        for item in children {
            if let Some(id) = item.id {
                next_frontier.push(id);
            }
            if let Some(thumbnail_id) = item
                .thumbnail
                .as_deref()
                .and_then(|thumb| ObjectId::parse_str(thumb).ok())
            {
                if seen.insert(thumbnail_id) {
                    all_ids.push(thumbnail_id);
                }
            }
        }
        frontier = next_frontier;
    }

    if let Some(thumbnail_id) = root
        .thumbnail
        .as_deref()
        .and_then(|thumb| ObjectId::parse_str(thumb).ok())
    {
        if seen.insert(thumbnail_id) {
            all_ids.push(thumbnail_id);
        }
    }

    Ok(all_ids)
}

async fn ensure_restorable_parent(
    repo: &StorageRepository,
    user_id: &str,
    item: &Storage,
) -> Result<bool, mongodb::error::Error> {
    if item.parent_id == "root" {
        return Ok(true);
    }

    let parent_id = match ObjectId::parse_str(&item.parent_id) {
        Ok(id) => id,
        Err(_) => return Ok(false),
    };

    match repo.find_by_id(parent_id).await? {
        Some(parent) => Ok(parent.user_id == user_id && !parent.trashed),
        None => Ok(false),
    }
}

async fn can_restore_item(
    repo: &StorageRepository,
    user_id: &str,
    item: &Storage,
    selected_ids: &HashSet<ObjectId>,
) -> Result<bool, mongodb::error::Error> {
    if item.parent_id == "root" {
        return Ok(true);
    }

    let parent_id = match ObjectId::parse_str(&item.parent_id) {
        Ok(id) => id,
        Err(_) => return Ok(false),
    };

    if selected_ids.contains(&parent_id) {
        return Ok(true);
    }

    ensure_restorable_parent(repo, user_id, item).await
}

#[derive(Deserialize, ToSchema)]
pub struct CreateFolderDto {
    pub name: String,
    #[serde(rename = "parentId")]
    pub parent_id: String,
    #[allow(dead_code)]
    pub r#type: String,
}

#[derive(Deserialize, ToSchema)]
pub struct GetFilesDto {
    #[serde(default)]
    #[schema(value_type = Object)]
    pub query: Document,
    #[serde(default)]
    pub keyword: Option<String>,
    #[serde(default)]
    pub types: Vec<String>,
    #[serde(rename = "sortBy", default)]
    pub sort_by: Option<String>,
    #[serde(rename = "sortOrder", default = "default_sort_order")]
    pub sort_order: i32,
    #[serde(rename = "viewMode", default = "default_view_mode")]
    pub view_mode: String,
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_limit")]
    pub limit: u64,
}

#[derive(Deserialize, ToSchema)]
pub struct RestoreTrashDto {
    #[serde(rename = "fileIds", default)]
    pub file_ids: Vec<String>,
    #[serde(rename = "restoreAll", default)]
    pub restore_all: bool,
}

fn default_page() -> u64 {
    1
}
fn default_limit() -> u64 {
    100
}
fn default_sort_order() -> i32 {
    -1
}
fn default_view_mode() -> String {
    "topLevel".to_string()
}

fn apply_list_filters(query: &mut Document, payload: &GetFilesDto) {
    if let Some(keyword) = payload.keyword.as_deref().map(str::trim) {
        if !keyword.is_empty() {
            query.insert("name", doc! { "$regex": keyword, "$options": "i" });
        }
    }

    if !payload.types.is_empty() {
        let allowed: Vec<String> = payload
            .types
            .iter()
            .filter(|t| t.as_str() == "file" || t.as_str() == "folder")
            .cloned()
            .collect();
        if !allowed.is_empty() {
            query.insert("type", doc! { "$in": allowed });
        }
    }
}

fn build_sort(payload: &GetFilesDto) -> Option<Document> {
    let field = match payload.sort_by.as_deref() {
        Some("name") => "name",
        Some("createdAt") => "createdAt",
        Some("updatedAt") => "updatedAt",
        Some("size") => "size",
        Some(_) | None => "updatedAt",
    };
    let order = if payload.sort_order >= 0 { 1 } else { -1 };
    Some(doc! { field: order, "_id": 1 })
}

#[derive(Deserialize, ToSchema)]
pub struct MoveFileDto {
    #[serde(rename = "fileId")]
    pub file_id: String,
    #[serde(rename = "parentId")]
    pub parent_id: String,
}

#[derive(Deserialize, ToSchema)]
pub struct GetPathDto {
    /// 文件或文件夹 id；也接受 body 中的 fileId（兼容 NestJS）。
    #[serde(alias = "fileId")]
    pub id: String,
}

#[utoipa::path(
    post,
    path = "/v1/storage/upload",
    request_body(content = Vec<u8>, content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "Files uploaded successfully", body = [String])
    ),
    tag = "storage",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn upload_file(
    State(state): State<AppState>,
    user_ctx: UserContext,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut uploaded_ids = Vec::new();
    let repo = StorageRepository::new(&state.db);
    let storage = LocalStorage::new();
    let mut parent_id = "root".to_string();

    while let Ok(Some(field)) = multipart.next_field().await {
        // 表单项 parentId：当前上传目录，列表接口会按 parentId 查，未传则用 root
        if field.name().is_some_and(|n| n == "parentId") {
            if let Ok(bytes) = field.bytes().await {
                let s = String::from_utf8_lossy(&bytes).trim().to_string();
                if !s.is_empty() {
                    parent_id = s;
                }
            }
            continue;
        }

        let name = match field.file_name() {
            Some(n) => n.trim().to_string(),
            None => continue,
        };
        // 跳过无有效文件名的 part，避免列表出现非用户上传的 unnamed 等
        if name.is_empty() || name.eq_ignore_ascii_case("unnamed") {
            continue;
        }
        let content_type = field
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_string();
        let data = field.bytes().await.unwrap();

        let hash = hash_buffer(&data);
        let iv = get_iv();

        // 仅在同一父目录下按 hash 去重：同文件上传到不同文件夹会各有一条记录，列表能按 parentId 查到
        let existing = repo
            .find_one(doc! {
                "MD5Hash": &hash,
                "userId": &user_ctx.user_id,
                "parentId": &parent_id,
                "type": "file",
                "trashed": false,
            })
            .await
            .unwrap();

        if let Some(doc) = existing {
            uploaded_ids.push(doc.id.unwrap().to_hex());
            continue;
        }

        // 物理存储按 hash 单路径，解密用 (hash, iv)。同文件再传若用新 iv 会覆盖磁盘导致旧链接失效，故复用已有文档的 iv 且不再写盘
        let (iv_to_use, need_store) = match repo
            .find_one(doc! { "MD5Hash": &hash, "userId": &user_ctx.user_id, "type": "file" })
            .await
            .unwrap()
        {
            Some(ref d) if d.iv.is_some() => (d.iv.as_deref().unwrap().to_string(), false),
            _ => (iv.clone(), true),
        };

        let mut storage_item = Storage {
            id: None,
            name: name.clone(),
            base_name: Some(
                StdPath::new(&name)
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string(),
            ),
            ext_name: Some(
                StdPath::new(&name)
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string(),
            ),
            mime_type: Some(content_type.clone()),
            encoding: None,
            size: Some(data.len() as i64),
            md5_hash: Some(hash.clone()),
            iv: Some(iv_to_use.clone()),
            parent_id: parent_id.clone(),
            r#type: "file".to_string(),
            user_id: user_ctx.user_id.clone(),
            thumbnail: None,
            trashed: false,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        // Thumbnail：仅对栅格图生成（image 库不支持 SVG 等矢量格式）
        let is_raster_image = content_type.starts_with("image/") && !content_type.contains("svg");
        if is_raster_image {
            if let Ok(thumb_data) = generate_thumbnail(&data) {
                let thumb_hash = hash_buffer(&thumb_data);
                let (thumb_iv_to_use, thumb_need_store) = match repo
                    .find_one(doc! { "MD5Hash": &thumb_hash, "userId": &user_ctx.user_id, "type": "thumbnail" })
                    .await
                    .unwrap()
                {
                    Some(ref d) if d.iv.is_some() => (d.iv.as_deref().unwrap().to_string(), false),
                    _ => (get_iv(), true),
                };

                let thumb_item = Storage {
                    id: None,
                    name: format!("{}_thumbnail", name),
                    base_name: None,
                    ext_name: None,
                    mime_type: Some("image/jpeg".to_string()),
                    encoding: None,
                    size: Some(thumb_data.len() as i64),
                    md5_hash: Some(thumb_hash.clone()),
                    iv: Some(thumb_iv_to_use.clone()),
                    parent_id: parent_id.clone(),
                    r#type: "thumbnail".to_string(),
                    user_id: user_ctx.user_id.clone(),
                    thumbnail: None,
                    trashed: false,
                    created_at: Some(Utc::now()),
                    updated_at: Some(Utc::now()),
                };

                let thumb_id = repo.create(thumb_item).await.unwrap();
                if thumb_need_store {
                    storage
                        .store(&thumb_hash, thumb_data, Some(&thumb_iv_to_use))
                        .await
                        .unwrap();
                }
                storage_item.thumbnail = Some(thumb_id.to_hex());
            }
        }

        let id = repo.create(storage_item).await.unwrap();
        if need_store {
            storage
                .store(&hash, data.to_vec(), Some(&iv_to_use))
                .await
                .unwrap();
        }
        uploaded_ids.push(id.to_hex());
    }

    Json(uploaded_ids).into_response()
}

#[utoipa::path(
    post,
    path = "/v1/storage/folder",
    request_body = CreateFolderDto,
    responses(
        (status = 200, description = "Folder created or existing: id, name, parentId, type, createdAt, updatedAt, exist", body = CreateFolderResponse)
    ),
    tag = "storage",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_folder(
    State(_state): State<AppState>,
    user_ctx: UserContext,
    Json(payload): Json<CreateFolderDto>,
) -> impl IntoResponse {
    let repo = StorageRepository::new(&_state.db);

    // 只认为「未删除」的同名文件夹为已存在，避免返回之前删除过的同名文件夹（列表接口只查 trashed: false）
    let existing = repo
        .find_one(doc! {
            "name": &payload.name,
            "parentId": &payload.parent_id,
            "userId": &user_ctx.user_id,
            "type": "folder",
            "trashed": false,
        })
        .await
        .unwrap();
    if let Some(doc) = existing {
        return Json(CreateFolderResponse {
            id: doc.id,
            name: doc.name,
            parent_id: doc.parent_id,
            r#type: doc.r#type,
            created_at: doc.created_at,
            updated_at: doc.updated_at,
            exist: true,
        })
        .into_response();
    }

    let now = Utc::now();
    let folder = Storage {
        id: None,
        name: payload.name.clone(),
        base_name: None,
        ext_name: None,
        mime_type: None,
        encoding: None,
        size: None,
        md5_hash: None,
        iv: None,
        parent_id: payload.parent_id.clone(),
        r#type: "folder".to_string(),
        user_id: user_ctx.user_id.clone(),
        thumbnail: None,
        trashed: false,
        created_at: Some(now),
        updated_at: Some(now),
    };

    let id = repo.create(folder).await.unwrap();
    Json(CreateFolderResponse {
        id: Some(id),
        name: payload.name,
        parent_id: payload.parent_id,
        r#type: "folder".to_string(),
        created_at: Some(now),
        updated_at: Some(now),
        exist: false,
    })
    .into_response()
}

#[utoipa::path(
    post,
    path = "/v1/storage/list",
    request_body = GetFilesDto,
    responses(
        (status = 200, description = "Paginated list { docs, total, limit, page, pages }", body = StorageListPaginatedResponse)
    ),
    tag = "storage",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_files(
    State(state): State<AppState>,
    user_ctx: UserContext,
    Json(payload): Json<GetFilesDto>,
) -> impl IntoResponse {
    let repo = StorageRepository::new(&state.db);
    let mut query = payload.query.clone();
    query.insert("userId", &user_ctx.user_id);
    query.insert("trashed", false);
    // 只返回文件/文件夹，不把 type=thumbnail 的文档当独立行返回（缩略图仅作为主文件的 thumbnail 字段）
    query.insert("type", doc! { "$in": ["file", "folder"] });
    apply_list_filters(&mut query, &payload);

    let page = payload.page.max(1);
    let limit = payload.limit.min(1000).max(1);
    let sort = build_sort(&payload);

    // 排除“仅作为缩略图”的文档，列表里只显示主文件/文件夹，图片的 thumbnail 作为字段带在对应文件上
    if let Ok(thumbnail_ids) = repo.thumbnail_object_ids(query.clone()).await {
        if !thumbnail_ids.is_empty() {
            query.insert("_id", doc! { "$nin": thumbnail_ids });
        }
    }

    match repo.find_many_paginated(query, page, limit, sort).await {
        Ok((files, total)) => {
            let token = encode(
                &Header::default(),
                &Claims {
                    user_id: user_ctx.user_id.clone(),
                    exp: (Utc::now().timestamp() + 900) as usize, // 15 min for url/thumbnail links
                },
                &EncodingKey::from_secret(state.config.jwt_secret.as_bytes()),
            )
            .unwrap_or_default();
            let base_url = state.config.domain.as_str();
            let docs: Vec<StorageListResponse> = files
                .into_iter()
                .map(|s| StorageListResponse::from_storage_with_urls(s, base_url, &token))
                .collect();
            let pages = if total == 0 {
                1
            } else {
                (total + limit - 1) / limit
            };
            Json(StorageListPaginatedResponse {
                docs,
                total,
                limit,
                page,
                pages,
            })
            .into_response()
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/v1/storage/trash/list",
    request_body = GetFilesDto,
    responses(
        (status = 200, description = "Paginated trash list { docs, total, limit, page, pages }", body = StorageListPaginatedResponse)
    ),
    tag = "storage",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_trashed_files(
    State(_state): State<AppState>,
    user_ctx: UserContext,
    Json(payload): Json<GetFilesDto>,
) -> impl IntoResponse {
    let repo = StorageRepository::new(&_state.db);
    let mut query = payload.query.clone();
    query.insert("userId", &user_ctx.user_id);
    query.insert("trashed", true);
    query.insert("type", doc! { "$in": ["file", "folder"] });
    apply_list_filters(&mut query, &payload);

    let page = payload.page.max(1);
    let limit = payload.limit.min(1000).max(1);
    let sort = build_sort(&payload);

    if payload.view_mode != "all" {
        match repo.trashed_folder_ids(&user_ctx.user_id).await {
            Ok(parent_ids) if !parent_ids.is_empty() => {
                query.insert("parentId", doc! { "$nin": parent_ids });
            }
            Ok(_) => {}
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }

    if let Ok(thumbnail_ids) = repo.thumbnail_object_ids(query.clone()).await {
        if !thumbnail_ids.is_empty() {
            query.insert("_id", doc! { "$nin": thumbnail_ids });
        }
    }

    match repo.find_many_paginated(query, page, limit, sort).await {
        Ok((files, total)) => {
            let docs: Vec<StorageListResponse> =
                files.into_iter().map(StorageListResponse::from).collect();
            let pages = if total == 0 {
                1
            } else {
                (total + limit - 1) / limit
            };
            Json(StorageListPaginatedResponse {
                docs,
                total,
                limit,
                page,
                pages,
            })
            .into_response()
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/v1/storage/{id}",
    params(
        ("id" = String, Path, description = "File storage id"),
        ("token" = Option<String>, Query, description = "Access token for public download")
    ),
    responses(
        (status = 200, description = "File content stream", body = Vec<u8>),
        (status = 404, description = "File not found")
    ),
    tag = "storage"
)]
pub async fn get_file(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(params): Query<Document>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let mut token = params.get_str("token").unwrap_or("").to_string();
    if token.is_empty() {
        if let Some(auth_header) = headers.get(axum::http::header::AUTHORIZATION) {
            if let Ok(auth_str) = auth_header.to_str() {
                if let Some(stripped) = auth_str.strip_prefix("Bearer ") {
                    token = stripped.to_string();
                }
            }
        }
    }

    let claims = match decode::<Claims>(
        &token,
        &DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
        &Validation::default(),
    ) {
        Ok(data) => data.claims,
        Err(_) => return StatusCode::UNAUTHORIZED.into_response(),
    };

    let id_oid = match ObjectId::parse_str(&id) {
        Ok(o) => o,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };

    let repo = StorageRepository::new(&state.db);
    let doc = repo.find_by_id(id_oid).await.unwrap();

    if let Some(item) = doc {
        if item.user_id != claims.user_id || item.trashed {
            return StatusCode::NOT_FOUND.into_response();
        }
        if let Some(hash) = item.md5_hash {
            let storage = LocalStorage::new();
            if let Ok(Some(data)) = storage.fetch(&hash, item.iv.as_deref()).await {
                return (
                    [(
                        header::CONTENT_TYPE,
                        item.mime_type
                            .unwrap_or("application/octet-stream".to_string()),
                    )],
                    data,
                )
                    .into_response();
            }
        }
    }

    StatusCode::NOT_FOUND.into_response()
}

#[utoipa::path(
    post,
    path = "/v1/storage/move",
    request_body = MoveFileDto,
    responses(
        (status = 200, description = "File moved successfully")
    ),
    tag = "storage",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn move_file(
    State(state): State<AppState>,
    user_ctx: UserContext,
    Json(payload): Json<MoveFileDto>,
) -> impl IntoResponse {
    let id_oid = ObjectId::parse_str(&payload.file_id).unwrap();
    let repo = StorageRepository::new(&state.db);
    repo.update_one(
        id_oid,
        &user_ctx.user_id,
        doc! { "parentId": payload.parent_id },
    )
    .await
    .unwrap();
    StatusCode::OK.into_response()
}

#[utoipa::path(
    post,
    path = "/v1/storage/download/url",
    request_body(content = Object, description = "JSON with fileId"),
    responses(
        (status = 200, description = "Download URL generated", body = String)
    ),
    tag = "storage",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_download_url(
    State(state): State<AppState>,
    user_ctx: UserContext,
    Json(payload): Json<Document>,
) -> impl IntoResponse {
    let file_id = payload.get_str("fileId").unwrap_or("");
    let domain = state.config.domain.trim_end_matches('/');
    let token = encode(
        &Header::default(),
        &Claims {
            user_id: user_ctx.user_id.clone(),
            exp: (Utc::now().timestamp() + 900) as usize, // 15 min for download link
        },
        &EncodingKey::from_secret(state.config.jwt_secret.as_bytes()),
    )
    .unwrap_or_default();
    format!("{}/v1/storage/{}?token={}", domain, file_id, token).into_response()
}

#[utoipa::path(
    put,
    path = "/v1/storage/{id}",
    params(
        ("id" = String, Path, description = "File storage id")
    ),
    request_body(content = Object, description = "Updated file properties"),
    responses(
        (status = 200, description = "Updated document: id, name, parentId, type, userId, trashed, createdAt, updatedAt, baseName, extName", body = UpdateStorageResponse),
        (status = 404, description = "Not found or not owned by user")
    ),
    tag = "storage",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_file(
    State(state): State<AppState>,
    user_ctx: UserContext,
    Path(id): Path<String>,
    Json(payload): Json<Document>,
) -> impl IntoResponse {
    let id_oid = match ObjectId::parse_str(&id) {
        Ok(o) => o,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };
    let repo = StorageRepository::new(&state.db);
    match repo.update_one(id_oid, &user_ctx.user_id, payload).await {
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Ok(Some(_)) => match repo.find_by_id(id_oid).await {
            Ok(Some(doc)) if doc.user_id == user_ctx.user_id => {
                Json(UpdateStorageResponse::from(doc)).into_response()
            }
            _ => StatusCode::NOT_FOUND.into_response(),
        },
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

#[utoipa::path(
    delete,
    path = "/v1/storage/{id}",
    params(
        ("id" = String, Path, description = "File storage id")
    ),
    responses(
        (status = 200, description = "File removed successfully")
    ),
    tag = "storage",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn remove_file(
    State(state): State<AppState>,
    user_ctx: UserContext,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let repo = StorageRepository::new(&state.db);
    let id_oid = match ObjectId::parse_str(&id) {
        Ok(o) => o,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };

    let root = match repo.find_by_id(id_oid).await {
        Ok(Some(item)) if item.user_id == user_ctx.user_id => item,
        Ok(Some(_)) | Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let subtree_ids = match collect_related_item_ids(&repo, &user_ctx.user_id, &root).await {
        Ok(ids) => ids,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    match repo
        .update_many_by_ids(
            subtree_ids,
            &user_ctx.user_id,
            doc! { "trashed": true, "updatedAt": BsonDateTime::from_chrono(Utc::now()) },
        )
        .await
    {
        Ok(_) => StatusCode::OK.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/v1/storage/{id}/restore",
    params(
        ("id" = String, Path, description = "File or folder storage id")
    ),
    responses(
        (status = 200, description = "Restored document", body = UpdateStorageResponse),
        (status = 400, description = "Invalid id or parent folder is unavailable"),
        (status = 404, description = "Not found or not owned by user")
    ),
    tag = "storage",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn restore_file(
    State(state): State<AppState>,
    user_ctx: UserContext,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let repo = StorageRepository::new(&state.db);
    let id_oid = match ObjectId::parse_str(&id) {
        Ok(o) => o,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };

    let root = match repo.find_by_id(id_oid).await {
        Ok(Some(item)) if item.user_id == user_ctx.user_id => item,
        Ok(Some(_)) | Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let parent_is_restorable = match ensure_restorable_parent(&repo, &user_ctx.user_id, &root).await
    {
        Ok(value) => value,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    if !parent_is_restorable {
        return (
            StatusCode::BAD_REQUEST,
            "Parent folder is unavailable or still trashed",
        )
            .into_response();
    }

    let subtree_ids = match collect_related_item_ids(&repo, &user_ctx.user_id, &root).await {
        Ok(ids) => ids,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    if repo
        .update_many_by_ids(
            subtree_ids,
            &user_ctx.user_id,
            doc! { "trashed": false, "updatedAt": BsonDateTime::from_chrono(Utc::now()) },
        )
        .await
        .is_err()
    {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    match repo.find_by_id(id_oid).await {
        Ok(Some(doc)) if doc.user_id == user_ctx.user_id => {
            Json(UpdateStorageResponse::from(doc)).into_response()
        }
        Ok(_) => StatusCode::NOT_FOUND.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/v1/storage/trash/restore",
    request_body = RestoreTrashDto,
    responses(
        (status = 200, description = "Restored trashed items", body = TrashRestoreResponse),
        (status = 400, description = "Invalid request or parent folder is unavailable"),
        (status = 404, description = "Some requested items were not found in trash")
    ),
    tag = "storage",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn restore_trashed_files(
    State(state): State<AppState>,
    user_ctx: UserContext,
    Json(payload): Json<RestoreTrashDto>,
) -> impl IntoResponse {
    let repo = StorageRepository::new(&state.db);

    let roots = if payload.restore_all {
        match repo
            .find_many(doc! {
                "userId": &user_ctx.user_id,
                "trashed": true,
                "type": { "$in": ["file", "folder"] },
            })
            .await
        {
            Ok(items) => items,
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    } else {
        if payload.file_ids.is_empty() {
            return (StatusCode::BAD_REQUEST, "fileIds cannot be empty").into_response();
        }

        let mut ids = Vec::new();
        for id in &payload.file_ids {
            match ObjectId::parse_str(id) {
                Ok(oid) => ids.push(oid),
                Err(_) => return (StatusCode::BAD_REQUEST, "Invalid file id").into_response(),
            }
        }

        let requested_count = ids.len();
        let items = match repo.find_many_by_ids(ids, &user_ctx.user_id).await {
            Ok(items) => items
                .into_iter()
                .filter(|item| item.trashed && (item.r#type == "file" || item.r#type == "folder"))
                .collect::<Vec<_>>(),
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };

        if items.len() != requested_count {
            return (StatusCode::NOT_FOUND, "Some items were not found in trash").into_response();
        }

        items
    };

    if roots.is_empty() {
        return Json(TrashRestoreResponse {
            requested_items: 0,
            restored_docs: 0,
        })
        .into_response();
    }

    let selected_ids: HashSet<ObjectId> = roots.iter().filter_map(|item| item.id).collect();
    for item in &roots {
        match can_restore_item(&repo, &user_ctx.user_id, item, &selected_ids).await {
            Ok(true) => {}
            Ok(false) => {
                return (
                    StatusCode::BAD_REQUEST,
                    "Parent folder is unavailable or still trashed",
                )
                    .into_response();
            }
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }

    let mut ids_to_restore = HashSet::new();
    for item in &roots {
        let related_ids = match collect_related_item_ids(&repo, &user_ctx.user_id, item).await {
            Ok(ids) => ids,
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };
        ids_to_restore.extend(related_ids);
    }

    let restored_docs = match repo
        .update_many_by_ids(
            ids_to_restore.into_iter().collect(),
            &user_ctx.user_id,
            doc! { "trashed": false, "updatedAt": BsonDateTime::from_chrono(Utc::now()) },
        )
        .await
    {
        Ok(count) => count,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    Json(TrashRestoreResponse {
        requested_items: roots.len() as u64,
        restored_docs,
    })
    .into_response()
}

#[utoipa::path(
    delete,
    path = "/v1/storage/trash",
    responses(
        (status = 200, description = "Trash emptied and orphaned files garbage-collected", body = TrashCleanupResponse)
    ),
    tag = "storage",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn empty_trash(
    State(state): State<AppState>,
    user_ctx: UserContext,
) -> impl IntoResponse {
    let repo = StorageRepository::new(&state.db);
    let trashed_items = match repo
        .find_many(doc! { "userId": &user_ctx.user_id, "trashed": true })
        .await
    {
        Ok(items) => items,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    if trashed_items.is_empty() {
        return Json(TrashCleanupResponse {
            deleted_docs: 0,
            deleted_files: 0,
        })
        .into_response();
    }

    let ids: Vec<ObjectId> = trashed_items.iter().filter_map(|item| item.id).collect();
    let hashes: HashSet<String> = trashed_items
        .iter()
        .filter_map(|item| item.md5_hash.clone())
        .collect();

    let deleted_docs = match repo.delete_many_by_ids(ids, &user_ctx.user_id).await {
        Ok(count) => count,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let storage = LocalStorage::new();
    let mut deleted_files = 0;
    for hash in hashes {
        let remaining = match repo.count_by_hash(&hash).await {
            Ok(count) => count,
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };
        if remaining == 0 {
            match storage.remove(&hash).await {
                Ok(()) => deleted_files += 1,
                Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            }
        }
    }

    Json(TrashCleanupResponse {
        deleted_docs,
        deleted_files,
    })
    .into_response()
}

#[utoipa::path(
    post,
    path = "/v1/storage/path",
    request_body = GetPathDto,
    responses(
        (status = 200, description = "Path from root to the given file/folder", body = [StoragePathNode]),
        (status = 400, description = "Invalid id"),
        (status = 404, description = "Not found or not owned by user")
    ),
    tag = "storage",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_path(
    State(state): State<AppState>,
    user_ctx: UserContext,
    Json(payload): Json<GetPathDto>,
) -> impl IntoResponse {
    let id = payload.id.trim();
    if id.is_empty() {
        return (StatusCode::BAD_REQUEST, "Missing id or fileId").into_response();
    }
    let oid = match ObjectId::parse_str(id) {
        Ok(o) => o,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid id").into_response(),
    };

    let repo = StorageRepository::new(&state.db);

    let mut current = match repo.find_by_id(oid).await {
        Ok(Some(s)) => s,
        Ok(None) => return (StatusCode::NOT_FOUND, "Not found").into_response(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response(),
    };

    if current.user_id != user_ctx.user_id || current.trashed {
        return (StatusCode::NOT_FOUND, "Not found").into_response();
    }

    let mut path_reversed: Vec<StoragePathNode> = Vec::new();
    loop {
        path_reversed.push(StoragePathNode {
            id: current
                .id
                .as_ref()
                .map(|o| o.to_string())
                .unwrap_or_else(|| "root".to_string()),
            name: current.name.clone(),
            parent_id: current.parent_id.clone(),
            r#type: current.r#type.clone(),
        });
        if current.parent_id == "root" {
            break;
        }
        let parent_oid = match ObjectId::parse_str(&current.parent_id) {
            Ok(o) => o,
            Err(_) => break,
        };
        let parent = match repo.find_by_id(parent_oid).await {
            Ok(Some(s)) => s,
            _ => break,
        };
        if parent.user_id != user_ctx.user_id || parent.trashed {
            break;
        }
        current = parent;
    }

    path_reversed.reverse();
    let path: Vec<StoragePathNode> = path_reversed
        .into_iter()
        .filter(|n| !n.parent_id.is_empty())
        .collect();
    Json(path).into_response()
}
