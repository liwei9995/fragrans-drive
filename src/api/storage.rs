use crate::api::AppState;
use crate::api::middleware::{Claims, UserContext};
use crate::domain::storage::{
    CreateFolderResponse, Storage, StorageListPaginatedResponse, StorageListResponse,
    StoragePathNode, TrashCleanupResponse, TrashRestoreResponse, UpdateStorageResponse,
};
use crate::infrastructure::db::storage_repo::StorageRepository;
use crate::service::storage::StorageService;
use axum::{
    extract::{Json, Multipart, Path, Query, State},
    http::{StatusCode, header},
    response::IntoResponse,
};
use crate::api::error::AppError;
use chrono::Utc;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use mongodb::bson::{Document, doc, oid::ObjectId};
use serde::Deserialize;
use std::collections::HashSet;
use utoipa::ToSchema;



#[derive(Deserialize, ToSchema)]
pub struct CreateFolderDto {
    pub name: String,
    #[serde(rename = "parentId")]
    pub parent_id: String,
    #[allow(dead_code)]
    pub r#type: String,
}

#[derive(Debug, Default, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct StorageQueryDto {
    #[serde(rename = "parentId")]
    pub parent_id: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct UpdateStorageDto {
    pub name: String,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct GetDownloadUrlDto {
    #[serde(rename = "fileId")]
    pub file_id: String,
}

#[derive(Deserialize, ToSchema)]
pub struct GetFilesDto {
    #[serde(default)]
    pub query: StorageQueryDto,
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
    if let Some(keyword) = payload.keyword.as_deref().map(str::trim)
        && !keyword.is_empty() {
            query.insert("name", doc! { "$regex": keyword, "$options": "i" });
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
) -> Result<impl IntoResponse, AppError> {
    let mut uploaded_ids = Vec::new();
    let repo = StorageRepository::new(&state.db);
    let service = StorageService::new(repo);
    let mut parent_id = "root".to_string();

    while let Ok(Some(mut field)) = multipart.next_field().await {
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
        if name.is_empty() || name.eq_ignore_ascii_case("unnamed") {
            continue;
        }
        let content_type = field
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_string();

        let temp_dir = std::env::temp_dir();
        let temp_file_path = temp_dir.join(format!("{}.tmp", uuid::Uuid::new_v4()));
        let mut temp_file = tokio::fs::File::create(&temp_file_path).await?;
        
        let mut hasher = md5::Md5::new();
        use md5::Digest;
        let mut size = 0i64;

        while let Some(chunk) = field.chunk().await.map_err(|e| AppError::BadRequest(e.to_string()))? {
            hasher.update(&chunk);
            tokio::io::AsyncWriteExt::write_all(&mut temp_file, &chunk).await?;
            size += chunk.len() as i64;
        }
        tokio::io::AsyncWriteExt::flush(&mut temp_file).await?;
        let hash = hex::encode(hasher.finalize());

        match service.upload_file_chunk(&user_ctx.user_id, &parent_id, &name, &content_type, &temp_file_path, &hash, size).await {
            Ok(id) => uploaded_ids.push(id),
            Err(e) => tracing::error!("Upload chunk failed: {}", e),
        }
        tokio::fs::remove_file(&temp_file_path).await.ok();
    }

    Ok(Json(uploaded_ids))
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
    State(state): State<AppState>,
    user_ctx: UserContext,
    Json(payload): Json<CreateFolderDto>,
) -> Result<impl IntoResponse, AppError> {
    let repo = StorageRepository::new(&state.db);
    let service = StorageService::new(repo);
    let response = service
        .create_folder(payload.name, payload.parent_id, user_ctx.user_id)
        .await?;
    Ok(Json(response))
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
) -> Result<impl IntoResponse, AppError> {
    let repo = StorageRepository::new(&state.db);
    let service = StorageService::new(repo);
    let mut query = Document::new();
    if let Some(ref pid) = payload.query.parent_id {
        query.insert("parentId", pid);
    }
    apply_list_filters(&mut query, &payload);

    let page = payload.page.max(1);
    let limit = payload.limit.min(1000).max(1);
    let sort = build_sort(&payload);

    let (files, total) = service.get_files(&user_ctx.user_id, query, page, limit, sort).await?;

    let token = encode(
        &Header::default(),
        &Claims {
            user_id: user_ctx.user_id.clone(),
            exp: (Utc::now().timestamp() + 900) as usize,
        },
        &EncodingKey::from_secret(state.config.jwt_secret.as_bytes()),
    )
    .unwrap_or_default();
    
    let base_url = state.config.domain.as_str();
    let docs: Vec<StorageListResponse> = files
        .into_iter()
        .map(|s| StorageListResponse::from_storage_with_urls(s, base_url, &token))
        .collect();
    let pages = if total == 0 { 1 } else { total.div_ceil(limit) };
    
    Ok(Json(StorageListPaginatedResponse {
        docs,
        total,
        limit,
        page,
        pages,
    }))
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
    State(state): State<AppState>,
    user_ctx: UserContext,
    Json(payload): Json<GetFilesDto>,
) -> Result<impl IntoResponse, AppError> {
    let repo = StorageRepository::new(&state.db);
    let service = StorageService::new(repo);
    let mut query = Document::new();
    if let Some(ref pid) = payload.query.parent_id {
        query.insert("parentId", pid);
    }
    apply_list_filters(&mut query, &payload);

    let page = payload.page.max(1);
    let limit = payload.limit.min(1000).max(1);
    let sort = build_sort(&payload);

    let (files, total) = service.get_trashed_files(&user_ctx.user_id, query, page, limit, sort, &payload.view_mode).await?;

    let docs: Vec<StorageListResponse> = files.into_iter().map(StorageListResponse::from).collect();
    let pages = if total == 0 { 1 } else { total.div_ceil(limit) };
    
    Ok(Json(StorageListPaginatedResponse {
        docs,
        total,
        limit,
        page,
        pages,
    }))
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
) -> Result<impl IntoResponse, AppError> {
    let mut token = params.get_str("token").unwrap_or("").to_string();
    if token.is_empty()
        && let Some(auth_header) = headers.get(axum::http::header::AUTHORIZATION)
            && let Ok(auth_str) = auth_header.to_str()
                && let Some(stripped) = auth_str.strip_prefix("Bearer ") {
                    token = stripped.to_string();
                }

    let claims = match decode::<Claims>(
        &token,
        &DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
        &Validation::default(),
    ) {
        Ok(data) => data.claims,
        Err(_) => return Err(AppError::Unauthorized("Invalid token".into())),
    };

    let repo = StorageRepository::new(&state.db);
    let service = StorageService::new(repo);

    let (mime_type, data) = service.get_file_content(&id, &claims.user_id).await?;
    Ok(([(header::CONTENT_TYPE, mime_type)], data))
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
) -> Result<impl IntoResponse, AppError> {
    let repo = StorageRepository::new(&state.db);
    let service = StorageService::new(repo);
    service.move_file(&payload.file_id, &payload.parent_id, &user_ctx.user_id).await?;
    Ok(StatusCode::OK)
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
    Json(payload): Json<GetDownloadUrlDto>,
) -> impl IntoResponse {
    let file_id = &payload.file_id;
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
    Json(payload): Json<UpdateStorageDto>,
) -> Result<impl IntoResponse, AppError> {
    let repo = StorageRepository::new(&state.db);
    let service = StorageService::new(repo);
    let response = service.update_file(&id, &user_ctx.user_id, payload.name).await?;
    Ok(Json(response))
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
) -> Result<impl IntoResponse, AppError> {
    let repo = StorageRepository::new(&state.db);
    let service = StorageService::new(repo);
    service.remove_file(&id, &user_ctx.user_id).await?;
    Ok(StatusCode::OK)
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
) -> Result<impl IntoResponse, AppError> {
    let repo = StorageRepository::new(&state.db);
    let service = StorageService::new(repo);
    let response = service.restore_file(&id, &user_ctx.user_id).await?;
    Ok(Json(response))
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
) -> Result<impl IntoResponse, AppError> {
    let repo = StorageRepository::new(&state.db);
    let service = StorageService::new(repo);
    let response = service.restore_trashed_files(&user_ctx.user_id, payload.file_ids, payload.restore_all).await?;
    Ok(Json(response))
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
) -> Result<impl IntoResponse, AppError> {
    let repo = StorageRepository::new(&state.db);
    let service = StorageService::new(repo);
    let response = service.empty_trash(&user_ctx.user_id).await?;
    Ok(Json(response))
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
) -> Result<impl IntoResponse, AppError> {
    let id = payload.id.trim();
    if id.is_empty() {
        return Err(AppError::BadRequest("Missing id or fileId".into()));
    }
    let repo = StorageRepository::new(&state.db);
    let service = StorageService::new(repo);
    let path = service.get_path(id, &user_ctx.user_id).await?;
    Ok(Json(path))
}
