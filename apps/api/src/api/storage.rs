use crate::api::AppState;
use crate::api::error::AppError;
use crate::api::middleware::{Claims, TokenPurpose, UserContext};
use crate::domain::storage::{
    CreateFolderResponse, StorageListPaginatedResponse, StorageListResponse, StoragePathNode,
    TrashCleanupResponse, TrashRestoreResponse, UpdateStorageResponse,
};
use crate::infrastructure::db::storage_repo::StorageRepository;
use crate::service::storage::StorageService;
use axum::{
    extract::{Json, Multipart, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::{DateTime, Utc};
use jsonwebtoken::{DecodingKey, Validation, decode};
use mongodb::bson::{Document, doc};
use serde::{Deserialize, Serialize};
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
    #[serde(rename = "expireInSeconds", default)]
    pub expire_in_seconds: Option<i64>,
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
    #[serde(rename = "isPublic", default)]
    pub is_public: Option<bool>,
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

fn apply_list_filters(query: &mut Document, payload: &GetFilesDto) -> Result<(), AppError> {
    if let Some(keyword) = payload.keyword.as_deref().map(str::trim)
        && !keyword.is_empty()
    {
        let limit_kw: String = keyword.chars().take(100).collect();
        let escaped = regex::escape(&limit_kw);
        query.insert("name", doc! { "$regex": escaped, "$options": "i" });
    }

    if let Some(is_public) = payload.is_public {
        query.insert("isPublic", is_public);
    }

    let mut allowed = Vec::new();
    if payload.types.is_empty() {
        allowed.push("file".to_string());
        allowed.push("folder".to_string());
    } else {
        for t in &payload.types {
            if t == "file" || t == "folder" {
                allowed.push(t.clone());
            } else {
                return Err(AppError::BadRequest("Unknown type filter".into()));
            }
        }
    }
    query.insert("type", doc! { "$in": allowed });
    Ok(())
}

fn build_sort(payload: &GetFilesDto) -> Option<Document> {
    let field = match payload.sort_by.as_deref() {
        Some("name") => "name",
        Some("createdAt") => "createdAt",
        Some("updatedAt") => "updatedAt",
        Some("size") => "size",
        _ => "updatedAt",
    };
    let order = if payload.sort_order == 1 { 1 } else { -1 };
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
    /// File or folder id; also accepts `fileId` in the body (NestJS compatibility).
    #[serde(alias = "fileId")]
    pub id: String,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct SetPublicStatusDto {
    #[serde(rename = "isPublic")]
    pub is_public: bool,
    #[serde(default)]
    pub refresh: bool,
    #[serde(rename = "expiresIn", default)]
    pub expires_in: Option<i64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PublicStatusResponse {
    pub id: String,
    #[serde(rename = "isPublic")]
    pub is_public: bool,
    #[serde(rename = "publicSlug", skip_serializing_if = "Option::is_none")]
    pub public_slug: Option<String>,
    #[serde(rename = "publicUrl", skip_serializing_if = "Option::is_none")]
    pub public_url: Option<String>,
    #[serde(
        rename = "publicExpiresAt",
        skip_serializing_if = "Option::is_none",
        serialize_with = "crate::utils::serde_json_response::serialize_optional_datetime_as_rfc3339"
    )]
    pub public_expires_at: Option<DateTime<Utc>>,
    #[serde(rename = "publicAccessCount", skip_serializing_if = "Option::is_none")]
    pub public_access_count: Option<u64>,
    #[serde(
        rename = "lastPublicAccessedAt",
        skip_serializing_if = "Option::is_none",
        serialize_with = "crate::utils::serde_json_response::serialize_optional_datetime_as_rfc3339"
    )]
    pub last_public_accessed_at: Option<DateTime<Utc>>,
}

pub fn generate_public_slug() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut rng = rand::thread_rng();
    (0..16)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

pub fn is_safe_inline_mime(mime: &str) -> bool {
    let base = mime.split(';').next().unwrap_or(mime).trim();
    let m = base.to_ascii_lowercase();
    if m == "text/html"
        || m == "application/xhtml+xml"
        || m == "text/javascript"
        || m == "application/javascript"
        || m == "application/x-javascript"
    {
        return false;
    }
    matches!(
        m.as_str(),
        "image/jpeg"
            | "image/png"
            | "image/webp"
            | "image/gif"
            | "image/avif"
            | "image/bmp"
            | "image/x-icon"
            | "image/vnd.microsoft.icon"
            | "audio/mpeg"
            | "audio/ogg"
            | "audio/wav"
            | "audio/webm"
            | "audio/aac"
            | "audio/flac"
            | "audio/mp4"
            | "video/mp4"
            | "video/webm"
            | "video/ogg"
            | "video/quicktime"
            | "application/pdf"
            | "application/json"
            | "application/xml"
            | "text/plain"
            | "text/markdown"
            | "text/csv"
            | "text/css"
            | "text/xml"
    ) || (m.starts_with("image/") && m != "image/svg+xml")
        || m.starts_with("audio/")
        || m.starts_with("video/")
        || (m.starts_with("text/") && m != "text/html" && m != "text/javascript")
}

pub fn rfc5987_encode(input: &str) -> String {
    let mut encoded = String::with_capacity(input.len() * 3);
    for b in input.bytes() {
        if b.is_ascii_alphanumeric()
            || matches!(
                b,
                b'!' | b'#' | b'$' | b'&' | b'+' | b'-' | b'.' | b'^' | b'_' | b'`' | b'|' | b'~'
            )
        {
            encoded.push(b as char);
        } else {
            encoded.push_str(&format!("%{:02X}", b));
        }
    }
    encoded
}

pub fn sanitize_ascii_filename(input: &str) -> String {
    let sanitized: String = input
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric()
                || matches!(character, ' ' | '.' | '_' | '-' | '(' | ')' | '[' | ']')
            {
                character
            } else {
                '_'
            }
        })
        .collect();
    if sanitized.is_empty() {
        "file".to_string()
    } else {
        sanitized
    }
}

pub fn build_content_disposition(disposition_type: &str, filename: &str) -> String {
    let ascii_fallback = sanitize_ascii_filename(filename);
    let utf8_encoded = rfc5987_encode(filename);
    format!(
        "{}; filename=\"{}\"; filename*=UTF-8''{}",
        disposition_type, ascii_fallback, utf8_encoded
    )
}

fn is_length_limit(e: &(dyn std::error::Error + 'static)) -> bool {
    let mut source = e.source();
    while let Some(err) = source {
        let msg = err.to_string();
        if msg.contains("length limit exceeded")
            || msg.contains("payload too large")
            || msg.contains("Content-Length")
        {
            return true;
        }
        source = err.source();
    }
    false
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
#[axum::debug_handler(state = AppState)]
pub async fn upload_file(
    State(state): State<AppState>,
    user_ctx: UserContext,
    multipart_res: Result<Multipart, axum::extract::multipart::MultipartRejection>,
) -> Result<impl IntoResponse, AppError> {
    let repo = StorageRepository::new(&state.db);
    let service = StorageService::new(repo.clone(), state.local_storage.clone());
    let mut parent_id = "root".to_string();
    let max_file_size = state.config.max_upload_bytes as i64;

    let mut multipart = match multipart_res {
        Ok(m) => m,
        Err(e) => {
            if e.status() == StatusCode::PAYLOAD_TOO_LARGE
                || e.to_string().contains("payload too large")
                || e.to_string().contains("length limit exceeded")
            {
                return Err(AppError::PayloadTooLarge(e.to_string()));
            }
            return Err(AppError::BadRequest(e.to_string()));
        }
    };

    let mut uploaded_ids = Vec::new();
    let mut expected_hash: Option<String> = None;
    let mut expected_size: Option<i64> = None;

    while let Some(mut field) = multipart.next_field().await.map_err(|e| {
        if is_length_limit(&e)
            || e.to_string().contains("payload too large")
            || e.to_string().contains("length limit exceeded")
        {
            AppError::PayloadTooLarge(e.to_string())
        } else {
            AppError::BadRequest(e.to_string())
        }
    })? {
        if field.name().is_some_and(|n| n == "parentId") {
            let bytes = field.bytes().await.map_err(|error| {
                if is_length_limit(&error) {
                    AppError::PayloadTooLarge(error.to_string())
                } else {
                    AppError::BadRequest(error.to_string())
                }
            })?;
            let value = String::from_utf8(bytes.to_vec())
                .map_err(|_| AppError::BadRequest("parentId must be UTF-8".into()))?;
            let value = value.trim();
            if !value.is_empty() {
                parent_id = value.to_string();
                for uploaded_id in &uploaded_ids {
                    if let Ok(oid) = mongodb::bson::oid::ObjectId::parse_str(uploaded_id) {
                        let _ = repo
                            .update_one(oid, &user_ctx.user_id, doc! { "parentId": &parent_id })
                            .await;
                    }
                }
            }
            continue;
        }

        if field.name().is_some_and(|n| n == "hash") {
            let bytes = field
                .bytes()
                .await
                .map_err(|e| AppError::BadRequest(e.to_string()))?;
            if let Ok(value) = String::from_utf8(bytes.to_vec()) {
                expected_hash = Some(value.trim().to_string());
            }
            continue;
        }

        if field.name().is_some_and(|n| n == "size") {
            let bytes = field
                .bytes()
                .await
                .map_err(|e| AppError::BadRequest(e.to_string()))?;
            if let Ok(value) = String::from_utf8(bytes.to_vec())
                && let Ok(size) = value.trim().parse::<i64>()
            {
                expected_size = Some(size);
            }
            continue;
        }

        let name = match field.file_name() {
            Some(n) => n.trim().to_string(),
            None => continue,
        };
        if name.is_empty() || name.eq_ignore_ascii_case("unnamed") {
            return Err(AppError::BadRequest("Filename cannot be empty".to_string()));
        }
        let content_type = field
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_string();

        let hash = expected_hash.take();
        let size = expected_size.take();

        if let (Some(hash_val), Some(size_val)) = (hash, size) {
            if size_val > max_file_size {
                return Err(AppError::PayloadTooLarge(format!(
                    "File exceeds limit of {} bytes",
                    max_file_size
                )));
            }
            use futures::StreamExt;
            let stream = field.map(|res| res.map_err(|e| std::io::Error::other(e.to_string())));
            let reader = tokio_util::io::StreamReader::new(stream);

            let id = service
                .upload_stream(
                    &user_ctx.user_id,
                    &parent_id,
                    &name,
                    &content_type,
                    reader,
                    &hash_val,
                    size_val,
                )
                .await?;
            uploaded_ids.push(id);
        } else {
            // Fallback for older clients without hash/size
            let temp_file = tempfile::NamedTempFile::new()
                .map_err(|e| AppError::InternalError(e.to_string()))?;
            let std_file = temp_file
                .as_file()
                .try_clone()
                .map_err(|e| AppError::InternalError(e.to_string()))?;
            let mut async_file = tokio::fs::File::from_std(std_file);

            let mut hasher = sha2::Sha256::new();
            use sha2::Digest;
            let mut file_size = 0i64;

            while let Some(chunk) = field.chunk().await.map_err(|e| {
                if is_length_limit(&e)
                    || e.to_string().contains("payload too large")
                    || e.to_string().contains("length limit exceeded")
                {
                    AppError::PayloadTooLarge(e.to_string())
                } else {
                    AppError::BadRequest(e.to_string())
                }
            })? {
                hasher.update(&chunk);
                tokio::io::AsyncWriteExt::write_all(&mut async_file, &chunk)
                    .await
                    .map_err(|e| AppError::InternalError(e.to_string()))?;
                file_size += chunk.len() as i64;
                if file_size > max_file_size {
                    return Err(AppError::PayloadTooLarge(format!(
                        "File exceeds limit of {} bytes",
                        max_file_size
                    )));
                }
            }
            tokio::io::AsyncWriteExt::flush(&mut async_file)
                .await
                .map_err(|e| AppError::InternalError(e.to_string()))?;
            let hash_val = hex::encode(hasher.finalize());

            let id = service
                .upload_file_chunk(
                    &user_ctx.user_id,
                    &parent_id,
                    &name,
                    &content_type,
                    &temp_file.path().to_path_buf(),
                    &hash_val,
                    file_size,
                )
                .await?;
            uploaded_ids.push(id);
        }
    }

    if uploaded_ids.is_empty() {
        return Err(AppError::BadRequest("No files uploaded".to_string()));
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
    let service = StorageService::new(repo, state.local_storage.clone());
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
    let service = StorageService::new(repo, state.local_storage.clone());
    let mut query = Document::new();
    if let Some(ref pid) = payload.query.parent_id {
        query.insert("parentId", pid);
    }
    apply_list_filters(&mut query, &payload)?;

    let page = payload.page.max(1);
    let limit = payload.limit.clamp(1, 1000);
    let sort = build_sort(&payload);

    let (files, total) = service
        .get_files(&user_ctx.user_id, query, page, limit, sort)
        .await?;

    let thumb_ids: Vec<mongodb::bson::oid::ObjectId> = files
        .iter()
        .filter_map(|s| s.thumbnail.as_deref())
        .filter_map(|id| mongodb::bson::oid::ObjectId::parse_str(id).ok())
        .collect();
    let thumb_share_versions: std::collections::HashMap<String, i32> = if thumb_ids.is_empty() {
        std::collections::HashMap::new()
    } else {
        StorageRepository::new(&state.db)
            .find_many_by_ids(thumb_ids, &user_ctx.user_id)
            .await?
            .into_iter()
            .filter_map(|t| t.id.map(|id| (id.to_hex(), t.share_version)))
            .collect()
    };

    let base_url = state.config.domain.as_str();
    let docs: Vec<StorageListResponse> = files
        .into_iter()
        .map(|s| {
            StorageListResponse::from_storage_with_urls(
                s,
                base_url,
                &user_ctx.user_id,
                &state.config.jwt_secret,
                &thumb_share_versions,
            )
        })
        .collect::<Result<_, _>>()?;
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
    let service = StorageService::new(repo, state.local_storage.clone());
    let mut query = Document::new();
    if let Some(ref pid) = payload.query.parent_id {
        query.insert("parentId", pid);
    }
    apply_list_filters(&mut query, &payload)?;

    let page = payload.page.max(1);
    let limit = payload.limit.clamp(1, 1000);
    let sort = build_sort(&payload);

    let (files, total) = service
        .get_trashed_files(
            &user_ctx.user_id,
            query,
            page,
            limit,
            sort,
            &payload.view_mode,
        )
        .await?;

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
        && let Some(stripped) = auth_str.strip_prefix("Bearer ")
    {
        token = stripped.to_string();
    }

    let repo = StorageRepository::new(&state.db);
    let obj_id = mongodb::bson::oid::ObjectId::parse_str(&id)
        .map_err(|_| AppError::BadRequest("Invalid id".into()))?;
    let existing = repo
        .find_by_id(obj_id)
        .await?
        .ok_or_else(|| AppError::NotFound("File not found".into()))?;

    let (owner_user_id, token_exp) = if token.is_empty() {
        return Err(AppError::Unauthorized("Invalid token".into()));
    } else {
        let claims = match decode::<Claims>(
            &token,
            &DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
            &Validation::default(),
        ) {
            Ok(data) => data.claims,
            Err(_) => return Err(AppError::Unauthorized("Invalid token".into())),
        };

        if claims.purpose != TokenPurpose::Download {
            return Err(AppError::Unauthorized("Invalid token purpose".into()));
        }
        if claims.file_id.as_deref() != Some(id.as_str()) {
            return Err(AppError::Unauthorized(
                "Token not scoped to this file".into(),
            ));
        }
        if claims.share_version.unwrap_or(0) != existing.share_version {
            return Err(AppError::Unauthorized("Share link has been revoked".into()));
        }
        (claims.user_id, claims.exp)
    };

    let mut has_range = false;
    let mut range_start = 0;
    let mut range_end = None;
    if let Some(range_header) = headers.get(axum::http::header::RANGE)
        && let Ok(range_str) = range_header.to_str()
        && let Some(stripped) = range_str.strip_prefix("bytes=")
    {
        has_range = true;
        let parts: Vec<&str> = stripped.split('-').collect();
        if parts.len() == 2 {
            if let Ok(start) = parts[0].parse::<u64>() {
                range_start = start;
            }
            if let Ok(end) = parts[1].parse::<u64>() {
                range_end = Some(end);
            }
        }
    }

    let service = StorageService::new(repo, state.local_storage.clone());

    let is_preview = params
        .get_str("preview")
        .map(|p| p == "1" || p.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    if is_preview
        && let Ok((preview_data, preview_mime)) =
            service.get_or_generate_preview(&id, &owner_user_id).await
    {
        use axum::http::header::{
            ACCEPT_RANGES, CACHE_CONTROL, CONTENT_DISPOSITION, CONTENT_LENGTH, CONTENT_TYPE,
            REFERRER_POLICY, X_CONTENT_TYPE_OPTIONS,
        };
        let mut res_headers = axum::http::HeaderMap::new();
        res_headers.insert(CONTENT_TYPE, preview_mime.parse().unwrap());
        res_headers.insert(
            CONTENT_LENGTH,
            preview_data.len().to_string().parse().unwrap(),
        );
        let disposition = build_content_disposition("inline", &existing.name);
        res_headers.insert(CONTENT_DISPOSITION, disposition.parse().unwrap());
        let now = chrono::Utc::now().timestamp() as usize;
        let max_age = token_exp.saturating_sub(now).clamp(60, 86400);
        res_headers.insert(
            CACHE_CONTROL,
            format!("private, max-age={}", max_age).parse().unwrap(),
        );
        res_headers.insert(
            axum::http::HeaderName::from_static("content-security-policy"),
            "default-src 'none'; sandbox allow-downloads"
                .parse()
                .unwrap(),
        );
        res_headers.insert(REFERRER_POLICY, "no-referrer".parse().unwrap());
        res_headers.insert(X_CONTENT_TYPE_OPTIONS, "nosniff".parse().unwrap());
        res_headers.insert(ACCEPT_RANGES, "bytes".parse().unwrap());

        return Ok((
            axum::http::StatusCode::OK,
            res_headers,
            axum::body::Body::from(preview_data),
        )
            .into_response());
    }

    let (filename, mime_type, total_size, range_len, stream) = service
        .stream_file_content(id, owner_user_id, range_start, range_end)
        .await?;
    let force_download = params
        .get_str("download")
        .map(|d| d == "1" || d.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    let disposition_type = if !force_download && is_safe_inline_mime(&mime_type) {
        "inline"
    } else {
        "attachment"
    };
    let disposition = build_content_disposition(disposition_type, &filename);

    use axum::http::header::{
        ACCEPT_RANGES, CACHE_CONTROL, CONTENT_DISPOSITION, CONTENT_LENGTH, CONTENT_RANGE,
        CONTENT_TYPE, REFERRER_POLICY, X_CONTENT_TYPE_OPTIONS,
    };
    let mut res_headers = axum::http::HeaderMap::new();
    res_headers.insert(CONTENT_TYPE, mime_type.parse().unwrap());
    res_headers.insert(CONTENT_LENGTH, range_len.to_string().parse().unwrap());
    res_headers.insert(CONTENT_DISPOSITION, disposition.parse().unwrap());
    let now = chrono::Utc::now().timestamp() as usize;
    let max_age = token_exp.saturating_sub(now).clamp(60, 86400);
    res_headers.insert(
        CACHE_CONTROL,
        format!("private, max-age={}", max_age).parse().unwrap(),
    );
    res_headers.insert(
        axum::http::HeaderName::from_static("content-security-policy"),
        "default-src 'none'; sandbox allow-downloads"
            .parse()
            .unwrap(),
    );
    res_headers.insert(REFERRER_POLICY, "no-referrer".parse().unwrap());
    res_headers.insert(X_CONTENT_TYPE_OPTIONS, "nosniff".parse().unwrap());
    res_headers.insert(ACCEPT_RANGES, "bytes".parse().unwrap());

    let status = if has_range || range_len < total_size {
        let actual_end = range_start + range_len.saturating_sub(1);
        res_headers.insert(
            CONTENT_RANGE,
            format!("bytes {}-{}/{}", range_start, actual_end, total_size)
                .parse()
                .unwrap(),
        );
        axum::http::StatusCode::PARTIAL_CONTENT
    } else {
        axum::http::StatusCode::OK
    };

    Ok((status, res_headers, axum::body::Body::from_stream(stream)).into_response())
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
    let service = StorageService::new(repo, state.local_storage.clone());
    service
        .move_file(&payload.file_id, &payload.parent_id, &user_ctx.user_id)
        .await?;
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
) -> Result<impl IntoResponse, AppError> {
    let file_id = &payload.file_id;
    let repo = StorageRepository::new(&state.db);
    let obj_id = mongodb::bson::oid::ObjectId::parse_str(file_id)
        .map_err(|_| AppError::BadRequest("Invalid fileId".into()))?;

    let existing = repo
        .find_by_id(obj_id)
        .await?
        .ok_or_else(|| AppError::NotFound("File not found".into()))?;

    if existing.user_id != user_ctx.user_id {
        return Err(AppError::NotFound("File not found".into())); // Don't leak existence
    }
    if existing.trashed {
        return Err(AppError::BadRequest("File is trashed".into()));
    }
    if existing.r#type != crate::domain::storage::StorageType::File
        && existing.r#type != crate::domain::storage::StorageType::Thumbnail
    {
        return Err(AppError::BadRequest("Not a file".into()));
    }

    let expire_secs = match payload.expire_in_seconds {
        None => 900,
        Some(s) if (1..=86_400).contains(&s) => s,
        Some(_) => {
            return Err(AppError::BadRequest(
                "expireInSeconds must be between 1 and 86400".into(),
            ));
        }
    };

    let domain = state.config.domain.trim_end_matches('/');
    let token = crate::api::middleware::create_token(
        &state.config.jwt_secret,
        &user_ctx.user_id,
        TokenPurpose::Download,
        Some(file_id.to_string()),
        (chrono::Utc::now().timestamp() + expire_secs) as usize,
        Some(existing.share_version),
        None,
    )?;
    Ok(format!(
        "{}/v1/storage/{}?download=1&token={}",
        domain, file_id, token
    )
    .into_response())
}

#[utoipa::path(
    post,
    path = "/v1/storage/{id}/revoke_share",
    params(
        ("id" = String, Path, description = "File storage id")
    ),
    responses(
        (status = 200, description = "Share revoked; returns new shareVersion", body = serde_json::Value)
    ),
    tag = "storage",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn revoke_share(
    State(state): State<AppState>,
    user_ctx: UserContext,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let repo = StorageRepository::new(&state.db);
    let obj_id = mongodb::bson::oid::ObjectId::parse_str(&id)
        .map_err(|_| AppError::BadRequest("Invalid fileId".into()))?;

    let updated = repo
        .increment_share_version(obj_id, &user_ctx.user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("File not found".into()))?;

    Ok(Json(serde_json::json!({
        "message": "Share revoked successfully",
        "shareVersion": updated.share_version
    }))
    .into_response())
}

#[utoipa::path(
    put,
    path = "/v1/storage/{id}/public",
    params(
        ("id" = String, Path, description = "File storage id")
    ),
    request_body = SetPublicStatusDto,
    responses(
        (status = 200, description = "Public status updated", body = PublicStatusResponse),
        (status = 400, description = "Invalid request"),
        (status = 404, description = "File not found")
    ),
    tag = "storage",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn set_public_status(
    State(state): State<AppState>,
    user_ctx: UserContext,
    Path(id): Path<String>,
    Json(payload): Json<SetPublicStatusDto>,
) -> Result<impl IntoResponse, AppError> {
    let repo = StorageRepository::new(&state.db);
    let obj_id = mongodb::bson::oid::ObjectId::parse_str(&id)
        .map_err(|_| AppError::BadRequest("Invalid fileId".into()))?;

    let existing = repo
        .find_by_id(obj_id)
        .await?
        .ok_or_else(|| AppError::NotFound("File not found".into()))?;

    if existing.user_id != user_ctx.user_id {
        return Err(AppError::NotFound("File not found".into()));
    }
    if existing.trashed {
        return Err(AppError::BadRequest(
            "Cannot set public status on trashed file".into(),
        ));
    }
    if existing.r#type != crate::domain::storage::StorageType::File {
        return Err(AppError::BadRequest("Only files can be made public".into()));
    }

    let expires_at = if payload.is_public {
        payload.expires_in.and_then(|secs| {
            if secs > 0 {
                Some(Utc::now() + chrono::Duration::seconds(secs))
            } else {
                None
            }
        })
    } else {
        None
    };

    let updated = if payload.is_public {
        let mut attempts = 0;
        loop {
            attempts += 1;
            let slug = if !payload.refresh && existing.is_public && existing.public_slug.is_some() {
                existing.public_slug.clone()
            } else {
                Some(generate_public_slug())
            };

            match repo
                .set_public_status(obj_id, &user_ctx.user_id, true, slug, expires_at)
                .await
            {
                Ok(Some(doc)) => break doc,
                Ok(None) => return Err(AppError::NotFound("File not found".into())),
                Err(e) if attempts < 3 && e.to_string().contains("11000") => {
                    tracing::warn!(
                        "Slug collision detected on attempt {}, retrying...",
                        attempts
                    );
                    continue;
                }
                Err(e) => return Err(AppError::DatabaseError(e)),
            }
        }
    } else {
        repo.set_public_status(obj_id, &user_ctx.user_id, false, None, None)
            .await?
            .ok_or_else(|| AppError::NotFound("File not found".into()))?
    };

    let domain = state.config.domain.trim_end_matches('/');
    let public_url = updated
        .public_slug
        .as_ref()
        .map(|s| format!("{}/v1/p/{}", domain, s));

    Ok(Json(PublicStatusResponse {
        id,
        is_public: updated.is_public,
        public_slug: updated.public_slug,
        public_url,
        public_expires_at: updated.public_expires_at,
        public_access_count: updated.public_access_count,
        last_public_accessed_at: updated.last_public_accessed_at,
    }))
}

#[utoipa::path(
    get,
    path = "/v1/p/{slug}",
    params(
        ("slug" = String, Path, description = "Public file slug"),
        ("download" = Option<String>, Query, description = "Force download if set to '1' or 'true'")
    ),
    responses(
        (status = 200, description = "File content stream", body = Vec<u8>),
        (status = 206, description = "Partial content"),
        (status = 304, description = "Not modified"),
        (status = 404, description = "File not found")
    ),
    tag = "storage"
)]
pub async fn get_public_file(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Query(params): Query<Document>,
    method: axum::http::Method,
    headers: axum::http::HeaderMap,
) -> Result<axum::response::Response, AppError> {
    get_public_file_impl(state, slug, params, method, headers).await
}

pub async fn get_public_file_with_name(
    State(state): State<AppState>,
    Path((slug, _filename)): Path<(String, String)>,
    Query(params): Query<Document>,
    method: axum::http::Method,
    headers: axum::http::HeaderMap,
) -> Result<axum::response::Response, AppError> {
    get_public_file_impl(state, slug, params, method, headers).await
}

async fn get_public_file_impl(
    state: AppState,
    slug: String,
    params: Document,
    method: axum::http::Method,
    headers: axum::http::HeaderMap,
) -> Result<axum::response::Response, AppError> {
    let repo = StorageRepository::new(&state.db);
    let existing = repo
        .find_by_public_slug(&slug)
        .await?
        .ok_or_else(|| AppError::NotFound("File not found".into()))?;

    if !existing.is_public
        || existing.trashed
        || existing.r#type != crate::domain::storage::StorageType::File
    {
        return Err(AppError::NotFound("File not found".into()));
    }

    if let Some(expires_at) = existing.public_expires_at
        && expires_at < Utc::now()
    {
        return Err(AppError::NotFound("Public direct link has expired".into()));
    }

    let etag = existing
        .content_hash
        .clone()
        .or_else(|| existing.md5_hash.clone())
        .unwrap_or_else(|| existing.id.map(|id| id.to_hex()).unwrap_or_default());
    let etag_header_val = format!("\"{}\"", etag);

    if let Some(if_none_match) = headers.get(axum::http::header::IF_NONE_MATCH)
        && let Ok(inm) = if_none_match.to_str()
    {
        let inm_trimmed = inm.trim();
        if inm_trimmed == etag_header_val || inm_trimmed == "*" || inm_trimmed.contains(&etag) {
            let mut res = axum::http::StatusCode::NOT_MODIFIED.into_response();
            res.headers_mut()
                .insert(axum::http::header::ETAG, etag_header_val.parse().unwrap());
            res.headers_mut().insert(
                axum::http::header::CACHE_CONTROL,
                "public, max-age=86400, stale-while-revalidate=3600"
                    .parse()
                    .unwrap(),
            );
            return Ok(res);
        }
    }

    let force_download = params
        .get_str("download")
        .map(|d| d == "1" || d.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    if let Some(file_id) = existing.id {
        let repo_bg = repo.clone();
        tokio::spawn(async move {
            let _ = repo_bg.record_public_access(file_id).await;
        });
    }

    let filename = existing.name.clone();
    let mime_type = existing
        .mime_type
        .clone()
        .unwrap_or_else(|| "application/octet-stream".to_string());
    let total_size = existing.size.unwrap_or(0) as u64;

    let is_preview = params
        .get_str("preview")
        .map(|p| p == "1" || p.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    if is_preview {
        let service = StorageService::new(repo.clone(), state.local_storage.clone());
        if let Some(file_id) = existing.id
            && let Ok((preview_data, preview_mime)) = service
                .get_or_generate_preview(&file_id.to_hex(), &existing.user_id)
                .await
        {
            use axum::http::header::{
                ACCEPT_RANGES, CACHE_CONTROL, CONTENT_DISPOSITION, CONTENT_LENGTH, CONTENT_TYPE,
                ETAG, REFERRER_POLICY, X_CONTENT_TYPE_OPTIONS,
            };
            let mut res_headers = axum::http::HeaderMap::new();
            res_headers.insert(CONTENT_TYPE, preview_mime.parse().unwrap());
            res_headers.insert(
                CONTENT_LENGTH,
                preview_data.len().to_string().parse().unwrap(),
            );
            let disposition = build_content_disposition("inline", &filename);
            res_headers.insert(CONTENT_DISPOSITION, disposition.parse().unwrap());
            res_headers.insert(
                CACHE_CONTROL,
                "public, max-age=86400, stale-while-revalidate=3600"
                    .parse()
                    .unwrap(),
            );
            res_headers.insert(ETAG, etag_header_val.parse().unwrap());
            res_headers.insert(X_CONTENT_TYPE_OPTIONS, "nosniff".parse().unwrap());
            res_headers.insert(
                axum::http::HeaderName::from_static("content-security-policy"),
                "default-src 'none'; sandbox allow-downloads"
                    .parse()
                    .unwrap(),
            );
            res_headers.insert(
                REFERRER_POLICY,
                "strict-origin-when-cross-origin".parse().unwrap(),
            );
            res_headers.insert(ACCEPT_RANGES, "bytes".parse().unwrap());

            return Ok((
                axum::http::StatusCode::OK,
                res_headers,
                axum::body::Body::from(preview_data),
            )
                .into_response());
        }
    }

    let disposition_type = if !force_download && is_safe_inline_mime(&mime_type) {
        "inline"
    } else {
        "attachment"
    };
    let disposition = build_content_disposition(disposition_type, &filename);

    use axum::http::header::{
        ACCEPT_RANGES, CACHE_CONTROL, CONTENT_DISPOSITION, CONTENT_LENGTH, CONTENT_RANGE,
        CONTENT_TYPE, ETAG, REFERRER_POLICY, X_CONTENT_TYPE_OPTIONS,
    };
    let mut res_headers = axum::http::HeaderMap::new();
    res_headers.insert(CONTENT_TYPE, mime_type.parse().unwrap());
    res_headers.insert(CONTENT_DISPOSITION, disposition.parse().unwrap());
    res_headers.insert(
        CACHE_CONTROL,
        "public, max-age=86400, stale-while-revalidate=3600"
            .parse()
            .unwrap(),
    );
    res_headers.insert(ETAG, etag_header_val.parse().unwrap());
    res_headers.insert(X_CONTENT_TYPE_OPTIONS, "nosniff".parse().unwrap());
    res_headers.insert(
        axum::http::HeaderName::from_static("content-security-policy"),
        "default-src 'none'; sandbox allow-downloads"
            .parse()
            .unwrap(),
    );
    res_headers.insert(
        REFERRER_POLICY,
        "strict-origin-when-cross-origin".parse().unwrap(),
    );
    res_headers.insert(ACCEPT_RANGES, "bytes".parse().unwrap());

    // Fast path for HEAD requests: zero disk I/O, zero AES decryption
    if method == axum::http::Method::HEAD {
        res_headers.insert(CONTENT_LENGTH, total_size.to_string().parse().unwrap());
        return Ok((
            axum::http::StatusCode::OK,
            res_headers,
            axum::body::Body::empty(),
        )
            .into_response());
    }

    let mut has_range = false;
    let mut range_start = 0;
    let mut range_end = None;
    if let Some(range_header) = headers.get(axum::http::header::RANGE)
        && let Ok(range_str) = range_header.to_str()
        && let Some(stripped) = range_str.strip_prefix("bytes=")
    {
        has_range = true;
        let parts: Vec<&str> = stripped.split('-').collect();
        if parts.len() == 2 {
            if let Ok(start) = parts[0].parse::<u64>() {
                range_start = start;
            }
            if let Ok(end) = parts[1].parse::<u64>() {
                range_end = Some(end);
            }
        }
    }

    let service = StorageService::new(repo, state.local_storage.clone());
    let file_id = existing.id.map(|id| id.to_hex()).unwrap_or_default();
    let (_fname, _mtype, _tsize, range_len, stream) = service
        .stream_file_content(file_id, existing.user_id.clone(), range_start, range_end)
        .await?;

    res_headers.insert(CONTENT_LENGTH, range_len.to_string().parse().unwrap());

    let status = if has_range || range_len < total_size {
        let actual_end = range_start + range_len.saturating_sub(1);
        res_headers.insert(
            CONTENT_RANGE,
            format!("bytes {}-{}/{}", range_start, actual_end, total_size)
                .parse()
                .unwrap(),
        );
        axum::http::StatusCode::PARTIAL_CONTENT
    } else {
        axum::http::StatusCode::OK
    };

    Ok((status, res_headers, axum::body::Body::from_stream(stream)).into_response())
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
    let service = StorageService::new(repo, state.local_storage.clone());
    let response = service
        .update_file(&id, &user_ctx.user_id, payload.name)
        .await?;
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
    let service = StorageService::new(repo, state.local_storage.clone());
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
    let service = StorageService::new(repo, state.local_storage.clone());
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
    let service = StorageService::new(repo, state.local_storage.clone());
    let response = service
        .restore_trashed_files(&user_ctx.user_id, payload.file_ids, payload.restore_all)
        .await?;
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
    let service = StorageService::new(repo, state.local_storage.clone());
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
    let service = StorageService::new(repo, state.local_storage.clone());
    let path = service.get_path(id, &user_ctx.user_id).await?;
    Ok(Json(path))
}

#[derive(Debug, Serialize, ToSchema)]
pub struct StorageUsageResponse {
    #[serde(rename = "usedBytes")]
    pub used_bytes: i64,
    #[serde(rename = "fileCount")]
    pub file_count: u64,
    #[serde(rename = "quotaBytes")]
    pub quota_bytes: i64,
}

#[utoipa::path(
    get,
    path = "/v1/storage/usage",
    responses(
        (status = 200, description = "Storage usage and quota for the current user", body = StorageUsageResponse)
    ),
    tag = "storage",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_storage_usage(
    State(state): State<AppState>,
    user_ctx: UserContext,
) -> Result<impl IntoResponse, AppError> {
    let repo = StorageRepository::new(&state.db);
    let service = StorageService::new(repo, state.local_storage.clone());
    let (used_bytes, file_count) = service.get_storage_usage(&user_ctx.user_id).await?;
    const DEFAULT_QUOTA_BYTES: i64 = 50 * 1024 * 1024 * 1024; // 50 GB
    Ok(Json(StorageUsageResponse {
        used_bytes,
        file_count,
        quota_bytes: DEFAULT_QUOTA_BYTES,
    }))
}
