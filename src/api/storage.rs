use crate::api::AppState;
use crate::api::middleware::UserContext;
use crate::domain::storage::Storage;
use crate::infrastructure::db::storage_repo::StorageRepository;
use crate::infrastructure::image::thumbnail::generate_thumbnail;
use crate::infrastructure::storage::local::LocalStorage;
use crate::utils::encryption::get_iv;
use crate::utils::md5::hash_buffer;
use axum::{
    extract::{Json, Multipart, Path, Query, State},
    http::{header, StatusCode},
    response::IntoResponse,
};
use chrono::Utc;
use mongodb::bson::{Document, doc, oid::ObjectId};
use serde::Deserialize;
use utoipa::ToSchema;
use std::path::Path as StdPath;

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
    // pagination ignored for now, simple Vec
}

#[derive(Deserialize, ToSchema)]
pub struct MoveFileDto {
    #[serde(rename = "fileId")]
    pub file_id: String,
    #[serde(rename = "parentId")]
    pub parent_id: String,
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

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.file_name().unwrap_or("unnamed").to_string();
        let content_type = field
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_string();
        let data = field.bytes().await.unwrap();

        let hash = hash_buffer(&data);
        let iv = get_iv();

        // De-duplication check
        let existing = repo
            .find_one(doc! { "MD5Hash": &hash, "userId": &user_ctx.user_id })
            .await
            .unwrap();

        if let Some(doc) = existing {
            uploaded_ids.push(doc.id.unwrap().to_hex());
            continue;
        }

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
            iv: Some(iv.clone()),
            parent_id: "root".to_string(), // Simplified for now, should get from body
            r#type: "file".to_string(),
            user_id: user_ctx.user_id.clone(),
            thumbnail: None,
            trashed: false,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        // Thumbnail?
        if content_type.starts_with("image/") {
            if let Ok(thumb_data) = generate_thumbnail(&data) {
                let thumb_hash = hash_buffer(&thumb_data);
                let thumb_iv = get_iv();

                let thumb_item = Storage {
                    id: None,
                    name: format!("{}_thumbnail", name),
                    base_name: None,
                    ext_name: None,
                    mime_type: Some("image/jpeg".to_string()),
                    encoding: None,
                    size: Some(thumb_data.len() as i64),
                    md5_hash: Some(thumb_hash.clone()),
                    iv: Some(thumb_iv.clone()),
                    parent_id: "root".to_string(),
                    r#type: "thumbnail".to_string(),
                    user_id: user_ctx.user_id.clone(),
                    thumbnail: None,
                    trashed: false,
                    created_at: Some(Utc::now()),
                    updated_at: Some(Utc::now()),
                };

                let thumb_id = repo.create(thumb_item).await.unwrap();
                storage
                    .store(&thumb_hash, thumb_data, Some(&thumb_iv))
                    .await
                    .unwrap();
                storage_item.thumbnail = Some(thumb_id.to_hex());
            }
        }

        let id = repo.create(storage_item).await.unwrap();
        storage
            .store(&hash, data.to_vec(), Some(&iv))
            .await
            .unwrap();
        uploaded_ids.push(id.to_hex());
    }

    Json(uploaded_ids).into_response()
}

#[utoipa::path(
    post,
    path = "/v1/storage/folder",
    request_body = CreateFolderDto,
    responses(
        (status = 200, description = "Folder created or existing returned", body = Storage)
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
) -> impl IntoResponse {
    let repo = StorageRepository::new(&state.db);

    // Check if exists
    let existing = repo.find_one(doc! { "name": &payload.name, "parentId": &payload.parent_id, "userId": &user_ctx.user_id, "type": "folder" }).await.unwrap();
    if let Some(doc) = existing {
        return Json(doc).into_response();
    }

    let folder = Storage {
        id: None,
        name: payload.name,
        base_name: None,
        ext_name: None,
        mime_type: None,
        encoding: None,
        size: None,
        md5_hash: None,
        iv: None,
        parent_id: payload.parent_id,
        r#type: "folder".to_string(),
        user_id: user_ctx.user_id,
        thumbnail: None,
        trashed: false,
        created_at: Some(Utc::now()),
        updated_at: Some(Utc::now()),
    };

    let id = repo.create(folder.clone()).await.unwrap();
    let mut res = folder;
    res.id = Some(id);
    Json(res).into_response()
}

#[utoipa::path(
    post,
    path = "/v1/storage/list",
    request_body = GetFilesDto,
    responses(
        (status = 200, description = "List of files", body = [Storage])
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
    let mut query = payload.query;
    query.insert("userId", &user_ctx.user_id);
    query.insert("trashed", false);

    match repo.find_many(query).await {
        Ok(files) => Json(files).into_response(),
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
) -> impl IntoResponse {
    // Public download with token
    let _token = params.get_str("token").unwrap_or("");
    // Validate token (simplified: in main we decodes it)
    // For now we assume if id is valid and doc exists, we serve.
    // Real implementation should check JWT here if it's not handled by middleware.

    let id_oid = match ObjectId::parse_str(&id) {
        Ok(o) => o,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };

    let repo = StorageRepository::new(&state.db);
    let doc = repo.find_by_id(id_oid).await.unwrap();

    if let Some(item) = doc {
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
    _user_ctx: UserContext,
    Json(payload): Json<Document>,
) -> impl IntoResponse {
    let file_id = payload.get_str("fileId").unwrap_or("");
    let domain = &state.config.domain;
    let token = "TODO";
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
        (status = 200, description = "File updated successfully")
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
    let id_oid = ObjectId::parse_str(&id).unwrap();
    let repo = StorageRepository::new(&state.db);
    repo.update_one(id_oid, &user_ctx.user_id, payload)
        .await
        .unwrap();
    StatusCode::OK.into_response()
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
    let id_oid = ObjectId::parse_str(&id).unwrap();
    let repo = StorageRepository::new(&state.db);
    // Legacy: trash first
    repo.update_one(id_oid, &user_ctx.user_id, doc! { "trashed": true })
        .await
        .unwrap();
    StatusCode::OK.into_response()
}

#[utoipa::path(
    post,
    path = "/v1/storage/path",
    request_body(content = Object),
    responses(
        (status = 501, description = "Not implemented")
    ),
    tag = "storage",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_path(
    State(_state): State<AppState>,
    _user_ctx: UserContext,
    Json(_payload): Json<Document>,
) -> impl IntoResponse {
    // Recursive path logic
    StatusCode::NOT_IMPLEMENTED.into_response()
}
