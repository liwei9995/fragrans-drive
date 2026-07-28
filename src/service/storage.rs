use crate::api::error::AppError;
use crate::domain::storage::{
    CreateFolderResponse, Storage,
    StoragePathNode, StorageType, TrashCleanupResponse, TrashRestoreResponse, UpdateStorageResponse,
};
use crate::infrastructure::db::storage_repo::StorageRepository;
use crate::infrastructure::storage::local::LocalStorage;
use crate::infrastructure::image::thumbnail::generate_thumbnail;
use crate::utils::encryption::get_iv;
use crate::utils::md5::hash_buffer;
use chrono::Utc;
use mongodb::bson::{DateTime as BsonDateTime, Document, doc, oid::ObjectId};
use std::collections::HashSet;
use std::path::{Path as StdPath, PathBuf};

pub struct StorageService {
    repo: StorageRepository,
}

impl StorageService {
    pub fn new(repo: StorageRepository) -> Self {
        Self { repo }
    }

    fn validate_name(&self, name: &str) -> Result<String, AppError> {
        let name = name.trim();
        if name.is_empty() || name.chars().count() > 255 || name == "." || name == ".." || name.contains('\0') {
            return Err(AppError::BadRequest("Invalid name".into()));
        }
        Ok(name.to_string())
    }

    async fn validate_parent(
        &self,
        parent_id: &str,
        user_id: &str,
        item_id: Option<&str>,
    ) -> Result<(), AppError> {
        if parent_id == "root" {
            return Ok(());
        }
        if Some(parent_id) == item_id {
            return Err(AppError::BadRequest("Cannot move item into itself".into()));
        }
        let oid = ObjectId::parse_str(parent_id).map_err(|_| AppError::BadRequest("Invalid parent ID".into()))?;
        let parent = self.repo.find_by_id(oid).await?.ok_or_else(|| AppError::BadRequest("Parent not found".into()))?;
        if parent.user_id != user_id || parent.trashed || parent.r#type != StorageType::Folder {
            return Err(AppError::BadRequest("Invalid parent folder".into()));
        }
        
        if let Some(item_id) = item_id {
            // Check if parent_id is a descendant of item_id
            let item_oid = ObjectId::parse_str(item_id).map_err(|_| AppError::BadRequest("Invalid item ID".into()))?;
            let mut current_parent_oid = oid;
            let mut seen = HashSet::new();
            loop {
                if current_parent_oid == item_oid {
                    return Err(AppError::BadRequest("Cannot move folder into its descendant".into()));
                }
                if !seen.insert(current_parent_oid) {
                    return Err(AppError::BadRequest("Cycle detected in parent chain".into()));
                }
                let current_parent = self.repo.find_by_id(current_parent_oid).await?.ok_or_else(|| AppError::BadRequest("Dangling parent".into()))?;
                if current_parent.parent_id == "root" {
                    break;
                }
                current_parent_oid = ObjectId::parse_str(&current_parent.parent_id).map_err(|_| AppError::BadRequest("Invalid parent ID in DB".into()))?;
            }
        }
        Ok(())
    }

    async fn collect_related_item_ids(
        &self,
        user_id: &str,
        root: &Storage,
    ) -> Result<Vec<ObjectId>, AppError> {
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
            let children = self.repo.find_many_by_parent_ids(parent_ids, user_id).await?;
            for item in children {
                if let Some(id) = item.id {
                    next_frontier.push(id);
                }
                if let Some(thumbnail_id) = item
                    .thumbnail
                    .as_deref()
                    .and_then(|thumb| ObjectId::parse_str(thumb).ok())
                    && seen.insert(thumbnail_id) {
                        all_ids.push(thumbnail_id);
                    }
            }
            frontier = next_frontier;
        }

        if let Some(thumbnail_id) = root
            .thumbnail
            .as_deref()
            .and_then(|thumb| ObjectId::parse_str(thumb).ok())
            && seen.insert(thumbnail_id) {
                all_ids.push(thumbnail_id);
            }

        Ok(all_ids)
    }

    async fn ensure_restorable_parent(
        &self,
        user_id: &str,
        item: &Storage,
    ) -> Result<bool, AppError> {
        if item.parent_id == "root" {
            return Ok(true);
        }

        let parent_id = match ObjectId::parse_str(&item.parent_id) {
            Ok(id) => id,
            Err(_) => return Ok(false),
        };

        match self.repo.find_by_id(parent_id).await? {
            Some(parent) => Ok(parent.user_id == user_id && !parent.trashed),
            None => Ok(false),
        }
    }

    async fn can_restore_item(
        &self,
        user_id: &str,
        item: &Storage,
        selected_ids: &HashSet<ObjectId>,
    ) -> Result<bool, AppError> {
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

        self.ensure_restorable_parent(user_id, item).await
    }

    pub async fn create_folder(
        &self,
        name: String,
        parent_id: String,
        user_id: String,
    ) -> Result<CreateFolderResponse, AppError> {
        let name = self.validate_name(&name)?;
        self.validate_parent(&parent_id, &user_id, None).await?;

        let existing = self
            .repo
            .find_one(doc! {
                "name": &name,
                "parentId": &parent_id,
                "userId": &user_id,
                "type": "folder",
                "trashed": false,
            })
            .await?;
        
        if let Some(doc) = existing {
            return Ok(CreateFolderResponse {
                id: doc.id,
                name: doc.name,
                parent_id: doc.parent_id,
                r#type: StorageType::Folder,
                created_at: doc.created_at,
                updated_at: doc.updated_at,
                exist: true,
            });
        }

        let now = Utc::now();
        let folder = Storage {
            id: None,
            name: name.clone(),
            base_name: None,
            ext_name: None,
            mime_type: None,
            encoding: None,
            size: None,
            md5_hash: None,
            iv: None,
            parent_id: parent_id.clone(),
            r#type: StorageType::Folder,
            user_id,
            thumbnail: None,
            trashed: false,
            created_at: Some(now),
            updated_at: Some(now),
        };

        let id = self.repo.create(folder).await?;
        Ok(CreateFolderResponse {
            id: Some(id),
            name,
            parent_id,
            r#type: StorageType::Folder,
            created_at: Some(now),
            updated_at: Some(now),
            exist: false,
        })
    }

    pub async fn upload_file_chunk(
        &self,
        user_id: &str,
        parent_id: &str,
        name: &str,
        content_type: &str,
        temp_file_path: &PathBuf,
        hash: &str,
        size: i64,
    ) -> Result<String, AppError> {
        let storage = LocalStorage::new();
        let iv = get_iv();

        let existing = self.repo
            .find_one(doc! {
                "MD5Hash": hash,
                "userId": user_id,
                "parentId": parent_id,
                "type": "file",
                "trashed": false,
            })
            .await?;

        if let Some(doc) = existing {
            return doc.id.map(|id| id.to_hex()).ok_or_else(|| AppError::DatabaseError(mongodb::error::Error::custom("missing id")));
        }

        let (iv_to_use, need_store) = match self.repo
            .find_one(doc! { "MD5Hash": hash, "userId": user_id, "type": "file" })
            .await?
        {
            Some(ref d) if d.iv.is_some() => (d.iv.as_deref().unwrap_or_default().to_string(), false),
            _ => (iv.clone(), true),
        };

        let mut storage_item = Storage {
            id: None,
            name: name.to_string(),
            base_name: Some(
                StdPath::new(name)
                    .file_stem()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or_default()
                    .to_string(),
            ),
            ext_name: Some(
                StdPath::new(name)
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string(),
            ),
            mime_type: Some(content_type.to_string()),
            encoding: None,
            size: Some(size),
            md5_hash: Some(hash.to_string()),
            iv: Some(iv_to_use.clone()),
            parent_id: parent_id.to_string(),
            r#type: StorageType::File,
            user_id: user_id.to_string(),
            thumbnail: None,
            trashed: false,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        let is_raster_image = content_type.starts_with("image/") && !content_type.contains("svg");
        if is_raster_image {
            let data = tokio::fs::read(temp_file_path).await?;
            let thumb_data = tokio::task::spawn_blocking(move || generate_thumbnail(&data))
                .await
                .map_err(|_| AppError::BadRequest("Thumbnail task failed".into()))??;
            
            let thumb_hash = hash_buffer(&thumb_data);
            let (thumb_iv_to_use, thumb_need_store) = match self.repo
                .find_one(doc! { "MD5Hash": &thumb_hash, "userId": user_id, "type": "thumbnail" })
                .await?
            {
                Some(ref d) if d.iv.is_some() => (d.iv.as_deref().unwrap_or_default().to_string(), false),
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
                parent_id: parent_id.to_string(),
                r#type: StorageType::Thumbnail,
                user_id: user_id.to_string(),
                thumbnail: None,
                trashed: false,
                created_at: Some(Utc::now()),
                updated_at: Some(Utc::now()),
            };

            let thumb_id = self.repo.create(thumb_item).await?;
            if thumb_need_store {
                storage.store(&thumb_hash, thumb_data, Some(&thumb_iv_to_use)).await?;
            }
            storage_item.thumbnail = Some(thumb_id.to_hex());
        }

        let id = self.repo.create(storage_item).await?;
        if need_store {
            storage.store_from_file(hash, temp_file_path, Some(&iv_to_use)).await?;
        }
        Ok(id.to_hex())
    }

    pub async fn get_files(
        &self,
        user_id: &str,
        mut query: Document,
        page: u64,
        limit: u64,
        sort: Option<Document>,
    ) -> Result<(Vec<Storage>, u64), AppError> {
        query.insert("userId", user_id);
        query.insert("trashed", false);
        query.insert("type", doc! { "$in": ["file", "folder"] });

        if let Ok(thumbnail_ids) = self.repo.thumbnail_object_ids(query.clone()).await
            && !thumbnail_ids.is_empty() {
                query.insert("_id", doc! { "$nin": thumbnail_ids });
            }

        let (files, total) = self.repo.find_many_paginated(query, page, limit, sort).await?;
        Ok((files, total))
    }

    pub async fn get_trashed_files(
        &self,
        user_id: &str,
        mut query: Document,
        page: u64,
        limit: u64,
        sort: Option<Document>,
        view_mode: &str,
    ) -> Result<(Vec<Storage>, u64), AppError> {
        query.insert("userId", user_id);
        query.insert("trashed", true);
        query.insert("type", doc! { "$in": ["file", "folder"] });

        if view_mode != "all"
            && let Ok(parent_ids) = self.repo.trashed_folder_ids(user_id).await
                && !parent_ids.is_empty() {
                    query.insert("parentId", doc! { "$nin": parent_ids });
                }

        if let Ok(thumbnail_ids) = self.repo.thumbnail_object_ids(query.clone()).await
            && !thumbnail_ids.is_empty() {
                query.insert("_id", doc! { "$nin": thumbnail_ids });
            }

        let (files, total) = self.repo.find_many_paginated(query, page, limit, sort).await?;
        Ok((files, total))
    }

    pub async fn get_file_content(
        &self,
        file_id: &str,
        user_id: &str,
    ) -> Result<(String, Vec<u8>), AppError> {
        let id_oid = ObjectId::parse_str(file_id)
            .map_err(|_| AppError::BadRequest("Invalid id".into()))?;

        let doc = self.repo.find_by_id(id_oid).await?;
        if let Some(item) = doc {
            if item.user_id != user_id || item.trashed {
                return Err(AppError::NotFound("File not found".into()));
            }
            if let Some(hash) = item.md5_hash {
                let storage = LocalStorage::new();
                if let Ok(Some(data)) = storage.fetch(&hash, item.iv.as_deref()).await {
                    let mime = item.mime_type.unwrap_or("application/octet-stream".to_string());
                    return Ok((mime, data));
                }
            }
        }
        Err(AppError::NotFound("File not found".into()))
    }

    pub async fn move_file(
        &self,
        file_id: &str,
        parent_id: &str,
        user_id: &str,
    ) -> Result<(), AppError> {
        self.validate_parent(parent_id, user_id, Some(file_id)).await?;
        let id_oid = ObjectId::parse_str(file_id)
            .map_err(|_| AppError::BadRequest("Invalid id".into()))?;
        self.repo.update_one(id_oid, user_id, doc! { "parentId": parent_id }).await?;
        Ok(())
    }

    pub async fn update_file(
        &self,
        file_id: &str,
        user_id: &str,
        name: String,
    ) -> Result<UpdateStorageResponse, AppError> {
        let name = self.validate_name(&name)?;
        let id_oid = ObjectId::parse_str(file_id)
            .map_err(|_| AppError::BadRequest("Invalid id".into()))?;
        
        let ext_name = StdPath::new(&name)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        
        let base_name = StdPath::new(&name)
            .file_stem()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default()
            .to_string();

        let payload = doc! { "name": name, "baseName": base_name, "extName": ext_name, "updatedAt": BsonDateTime::from_chrono(Utc::now()) };

        match self.repo.update_one(id_oid, user_id, payload).await? {
            None => Err(AppError::NotFound("Not found".into())),
            Some(_) => {
                let doc = self.repo.find_by_id(id_oid).await?
                    .ok_or_else(|| AppError::NotFound("Not found".into()))?;
                if doc.user_id == user_id {
                    Ok(UpdateStorageResponse::from(doc))
                } else {
                    Err(AppError::NotFound("Not found".into()))
                }
            }
        }
    }

    pub async fn remove_file(
        &self,
        file_id: &str,
        user_id: &str,
    ) -> Result<(), AppError> {
        let id_oid = ObjectId::parse_str(file_id)
            .map_err(|_| AppError::BadRequest("Invalid id".into()))?;

        let root = self.repo.find_by_id(id_oid).await?
            .ok_or_else(|| AppError::NotFound("Not found".into()))?;
        
        if root.user_id != user_id {
            return Err(AppError::NotFound("Not found".into()));
        }

        let subtree_ids = self.collect_related_item_ids(user_id, &root).await?;
        
        self.repo.update_many_by_ids(
            subtree_ids,
            user_id,
            doc! { "trashed": true, "updatedAt": BsonDateTime::from_chrono(Utc::now()) },
        ).await?;

        Ok(())
    }

    pub async fn restore_file(
        &self,
        file_id: &str,
        user_id: &str,
    ) -> Result<UpdateStorageResponse, AppError> {
        let id_oid = ObjectId::parse_str(file_id)
            .map_err(|_| AppError::BadRequest("Invalid id".into()))?;

        let root = self.repo.find_by_id(id_oid).await?
            .ok_or_else(|| AppError::NotFound("Not found".into()))?;
        
        if root.user_id != user_id {
            return Err(AppError::NotFound("Not found".into()));
        }

        if !self.ensure_restorable_parent(user_id, &root).await? {
            return Err(AppError::BadRequest("Parent folder is unavailable or still trashed".into()));
        }

        let subtree_ids = self.collect_related_item_ids(user_id, &root).await?;
        
        self.repo.update_many_by_ids(
            subtree_ids,
            user_id,
            doc! { "trashed": false, "updatedAt": BsonDateTime::from_chrono(Utc::now()) },
        ).await?;

        let doc = self.repo.find_by_id(id_oid).await?.unwrap();
        Ok(UpdateStorageResponse::from(doc))
    }

    pub async fn restore_trashed_files(
        &self,
        user_id: &str,
        file_ids: Vec<String>,
        restore_all: bool,
    ) -> Result<TrashRestoreResponse, AppError> {
        let roots = if restore_all {
            self.repo.find_many(doc! {
                "userId": user_id,
                "trashed": true,
                "type": { "$in": ["file", "folder"] },
            }).await?
        } else {
            if file_ids.is_empty() {
                return Err(AppError::BadRequest("fileIds cannot be empty".into()));
            }
            let mut ids = Vec::new();
            for id in &file_ids {
                ids.push(ObjectId::parse_str(id).map_err(|_| AppError::BadRequest("Invalid id".into()))?);
            }
            let requested_count = ids.len();
            let items = self.repo.find_many_by_ids(ids, user_id).await?
                .into_iter()
                .filter(|item| item.trashed && (item.r#type == StorageType::File || item.r#type == StorageType::Folder))
                .collect::<Vec<_>>();
            if items.len() != requested_count {
                return Err(AppError::NotFound("Some items were not found in trash".into()));
            }
            items
        };

        if roots.is_empty() {
            return Ok(TrashRestoreResponse { requested_items: 0, restored_docs: 0 });
        }

        let selected_ids: HashSet<ObjectId> = roots.iter().filter_map(|item| item.id).collect();
        for item in &roots {
            if !self.can_restore_item(user_id, item, &selected_ids).await? {
                return Err(AppError::BadRequest("Parent folder is unavailable or still trashed".into()));
            }
        }

        let mut ids_to_restore = HashSet::new();
        for item in &roots {
            ids_to_restore.extend(self.collect_related_item_ids(user_id, item).await?);
        }

        let restored_docs = self.repo.update_many_by_ids(
            ids_to_restore.into_iter().collect(),
            user_id,
            doc! { "trashed": false, "updatedAt": BsonDateTime::from_chrono(Utc::now()) },
        ).await?;

        Ok(TrashRestoreResponse {
            requested_items: roots.len() as u64,
            restored_docs,
        })
    }

    pub async fn empty_trash(
        &self,
        user_id: &str,
    ) -> Result<TrashCleanupResponse, AppError> {
        let trashed_items = self.repo.find_many(doc! { "userId": user_id, "trashed": true }).await?;
        if trashed_items.is_empty() {
            return Ok(TrashCleanupResponse { deleted_docs: 0, deleted_files: 0 });
        }

        let ids: Vec<ObjectId> = trashed_items.iter().filter_map(|item| item.id).collect();
        let hashes: HashSet<String> = trashed_items.iter().filter_map(|item| item.md5_hash.clone()).collect();

        let deleted_docs = self.repo.delete_many_by_ids(ids, user_id).await?;
        let storage = LocalStorage::new();
        let mut deleted_files = 0;

        for hash in hashes {
            let remaining = self.repo.count_by_hash(&hash).await?;
            if remaining == 0
                && storage.remove(&hash).await.is_ok() {
                    deleted_files += 1;
                }
        }

        Ok(TrashCleanupResponse { deleted_docs, deleted_files })
    }

    pub async fn get_path(
        &self,
        file_id: &str,
        user_id: &str,
    ) -> Result<Vec<StoragePathNode>, AppError> {
        let oid = ObjectId::parse_str(file_id).map_err(|_| AppError::BadRequest("Invalid id".into()))?;
        let mut current = self.repo.find_by_id(oid).await?
            .ok_or_else(|| AppError::NotFound("Not found".into()))?;

        if current.user_id != user_id || current.trashed {
            return Err(AppError::NotFound("Not found".into()));
        }

        let mut path_reversed: Vec<StoragePathNode> = Vec::new();
        let mut seen = HashSet::new();

        loop {
            let oid = current.id.unwrap_or_else(ObjectId::new);
            if !seen.insert(oid) {
                tracing::error!("Cycle detected in storage hierarchy for item: {}", oid);
                return Err(AppError::BadRequest("Cycle detected in storage hierarchy".into()));
            }

            path_reversed.push(StoragePathNode {
                id: current.id.as_ref().map(|o| o.to_string()).unwrap_or_else(|| "root".to_string()),
                name: current.name.clone(),
                parent_id: current.parent_id.clone(),
                r#type: current.r#type.clone(),
            });
            if current.parent_id == "root" { break; }
            let parent_oid = match ObjectId::parse_str(&current.parent_id) {
                Ok(o) => o,
                Err(e) => return Err(AppError::BadRequest(format!("Invalid parent ID in DB: {}", e))),
            };
            let parent = match self.repo.find_by_id(parent_oid).await {
                Ok(Some(s)) => s,
                _ => return Err(AppError::BadRequest("Dangling parent".into())),
            };
            if parent.user_id != user_id || parent.trashed { break; }
            current = parent;
        }

        path_reversed.reverse();
        let path: Vec<StoragePathNode> = path_reversed.into_iter().filter(|n| !n.parent_id.is_empty()).collect();
        Ok(path)
    }
}
