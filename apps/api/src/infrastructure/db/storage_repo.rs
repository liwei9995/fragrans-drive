use crate::domain::storage::Storage;
use futures_util::stream::StreamExt;
use mongodb::{
    Collection, Database,
    bson::{Document, doc, oid::ObjectId},
};
use std::collections::HashSet;

#[derive(Clone)]
pub struct StorageRepository {
    collection: Collection<Storage>,
}

impl StorageRepository {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection("storage"),
        }
    }

    pub async fn find_one(
        &self,
        query: Document,
    ) -> Result<Option<Storage>, mongodb::error::Error> {
        self.collection.find_one(query).await
    }

    pub async fn find_by_id(&self, id: ObjectId) -> Result<Option<Storage>, mongodb::error::Error> {
        self.collection.find_one(doc! { "_id": id }).await
    }

    pub async fn find_by_public_slug(
        &self,
        slug: &str,
    ) -> Result<Option<Storage>, mongodb::error::Error> {
        self.collection
            .find_one(doc! {
                "publicSlug": slug,
                "isPublic": true,
                "trashed": false
            })
            .await
    }

    pub async fn set_public_status(
        &self,
        id: ObjectId,
        user_id: &str,
        is_public: bool,
        public_slug: Option<String>,
        public_expires_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Option<Storage>, mongodb::error::Error> {
        let mut set_fields = doc! {
            "isPublic": is_public,
            "updatedAt": mongodb::bson::DateTime::now(),
        };
        if let Some(slug) = public_slug {
            set_fields.insert("publicSlug", slug);
        } else {
            set_fields.insert("publicSlug", mongodb::bson::Bson::Null);
        }
        if let Some(exp) = public_expires_at {
            set_fields.insert("publicExpiresAt", mongodb::bson::DateTime::from_chrono(exp));
        } else {
            set_fields.insert("publicExpiresAt", mongodb::bson::Bson::Null);
        }

        self.collection
            .find_one_and_update(
                doc! { "_id": id, "userId": user_id, "type": "file", "trashed": false },
                doc! { "$set": set_fields },
            )
            .return_document(mongodb::options::ReturnDocument::After)
            .await
    }

    pub async fn record_public_access(&self, id: ObjectId) -> Result<(), mongodb::error::Error> {
        let _ = self
            .collection
            .update_one(
                doc! { "_id": id },
                doc! {
                    "$inc": { "publicAccessCount": 1 },
                    "$set": { "lastPublicAccessedAt": mongodb::bson::DateTime::now() }
                },
            )
            .await?;
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn find_by_hash(&self, hash: &str) -> Result<Option<Storage>, mongodb::error::Error> {
        self.collection.find_one(doc! { "MD5Hash": hash }).await
    }

    #[allow(dead_code)]
    pub async fn find_many(&self, query: Document) -> Result<Vec<Storage>, mongodb::error::Error> {
        let mut cursor = self.collection.find(query.clone()).await?;
        let mut results = Vec::new();
        while let Some(item) = cursor.next().await {
            results.push(item?);
        }
        Ok(results)
    }

    pub async fn find_many_by_parent_ids(
        &self,
        parent_ids: Vec<String>,
        user_id: &str,
    ) -> Result<Vec<Storage>, mongodb::error::Error> {
        self.find_many(doc! {
            "parentId": { "$in": parent_ids },
            "userId": user_id,
        })
        .await
    }

    pub async fn find_many_by_ids(
        &self,
        ids: Vec<ObjectId>,
        user_id: &str,
    ) -> Result<Vec<Storage>, mongodb::error::Error> {
        self.find_many(doc! {
            "_id": { "$in": ids },
            "userId": user_id,
        })
        .await
    }

    pub async fn trashed_folder_ids(
        &self,
        user_id: &str,
    ) -> Result<Vec<String>, mongodb::error::Error> {
        let folders = self
            .find_many(doc! {
                "userId": user_id,
                "trashed": true,
                "type": "folder",
            })
            .await?;
        Ok(folders
            .into_iter()
            .filter_map(|item| item.id.map(|id| id.to_hex()))
            .collect())
    }

    /// Paginated query: returns (page items, total count).
    pub async fn find_many_paginated(
        &self,
        query: Document,
        page: u64,
        limit: u64,
        sort: Option<Document>,
    ) -> Result<(Vec<Storage>, u64), mongodb::error::Error> {
        let total = self.collection.count_documents(query.clone()).await?;
        let skip = (page.saturating_sub(1)) * limit;
        let mut find = self.collection.find(query).skip(skip).limit(limit as i64);
        if let Some(sort) = sort {
            find = find.sort(sort);
        }
        let mut cursor = find.await?;
        let mut results = Vec::new();
        while let Some(item) = cursor.next().await {
            results.push(item?);
        }
        Ok((results, total))
    }

    pub async fn create(&self, item: Storage) -> Result<ObjectId, mongodb::error::Error> {
        let result = self.collection.insert_one(item).await?;
        Ok(result.inserted_id.as_object_id().unwrap())
    }

    pub async fn update_one(
        &self,
        id: ObjectId,
        user_id: &str,
        update: Document,
    ) -> Result<Option<Storage>, mongodb::error::Error> {
        self.collection
            .find_one_and_update(
                doc! { "_id": id, "userId": user_id },
                doc! { "$set": update },
            )
            .await
    }

    pub async fn increment_share_version(
        &self,
        id: ObjectId,
        user_id: &str,
    ) -> Result<Option<Storage>, mongodb::error::Error> {
        self.collection
            .find_one_and_update(
                doc! { "_id": id, "userId": user_id },
                doc! {
                    "$inc": { "shareVersion": 1 },
                    "$set": { "updatedAt": mongodb::bson::DateTime::now() },
                },
            )
            .return_document(mongodb::options::ReturnDocument::After)
            .await
    }

    pub async fn update_many_by_ids(
        &self,
        ids: Vec<ObjectId>,
        user_id: &str,
        update: Document,
    ) -> Result<u64, mongodb::error::Error> {
        if ids.is_empty() {
            return Ok(0);
        }

        let result = self
            .collection
            .update_many(
                doc! {
                    "_id": { "$in": ids },
                    "userId": user_id,
                },
                doc! { "$set": update },
            )
            .await?;
        Ok(result.modified_count)
    }

    pub async fn update_many_by_md5(
        &self,
        user_id: &str,
        md5_hash: &str,
        update: Document,
    ) -> Result<u64, mongodb::error::Error> {
        let result = self
            .collection
            .update_many(
                doc! {
                    "MD5Hash": md5_hash,
                    "userId": user_id,
                },
                doc! { "$set": update },
            )
            .await?;
        Ok(result.modified_count)
    }

    pub async fn delete_many_by_ids(
        &self,
        ids: Vec<ObjectId>,
        user_id: &str,
    ) -> Result<u64, mongodb::error::Error> {
        if ids.is_empty() {
            return Ok(0);
        }

        let result = self
            .collection
            .delete_many(doc! {
                "_id": { "$in": ids },
                "userId": user_id,
            })
            .await?;
        Ok(result.deleted_count)
    }

    pub async fn count_by_hash(&self, hash: &str) -> Result<u64, mongodb::error::Error> {
        self.collection
            .count_documents(doc! { "MD5Hash": hash })
            .await
    }

    pub async fn count_by_user_content_hash(
        &self,
        user_id: &str,
        hash: &str,
    ) -> Result<u64, mongodb::error::Error> {
        self.collection
            .count_documents(doc! { "userId": user_id, "contentHash": hash })
            .await
    }

    /// Batch checks which content hashes still have active references for the given user.
    /// Aggregates with $match and $group to check all hashes in a single batch query.
    pub async fn find_referenced_content_hashes(
        &self,
        user_id: &str,
        hashes: &[String],
    ) -> Result<HashSet<String>, mongodb::error::Error> {
        if hashes.is_empty() {
            return Ok(HashSet::new());
        }

        let mut referenced = HashSet::new();
        let doc_coll = self.collection.clone_with_type::<Document>();

        for chunk in hashes.chunks(1000) {
            let pipeline = vec![
                doc! {
                    "$match": {
                        "userId": user_id,
                        "contentHash": { "$in": chunk }
                    }
                },
                doc! {
                    "$group": {
                        "_id": "$contentHash"
                    }
                },
            ];

            let mut cursor = doc_coll.aggregate(pipeline).await?;
            while let Some(res) = cursor.next().await {
                let doc = res?;
                if let Ok(hash) = doc.get_str("_id") {
                    referenced.insert(hash.to_string());
                }
            }
        }

        Ok(referenced)
    }

    #[allow(dead_code)]
    pub async fn delete_one(
        &self,
        id: ObjectId,
        user_id: &str,
    ) -> Result<Option<Storage>, mongodb::error::Error> {
        self.collection
            .find_one_and_delete(doc! { "_id": id, "userId": user_id })
            .await
    }

    #[allow(dead_code)]
    pub async fn delete_all(&self) -> Result<u64, mongodb::error::Error> {
        let result = self.collection.delete_many(doc! {}).await?;
        Ok(result.deleted_count)
    }

    pub async fn get_storage_usage(
        &self,
        user_id: &str,
    ) -> Result<(i64, u64), mongodb::error::Error> {
        let pipeline = vec![
            doc! {
                "$match": {
                    "userId": user_id,
                    "trashed": false,
                    "type": "file"
                }
            },
            doc! {
                "$group": {
                    "_id": mongodb::bson::Bson::Null,
                    "totalSize": { "$sum": "$size" },
                    "fileCount": { "$sum": 1 }
                }
            },
        ];
        let doc_coll = self.collection.clone_with_type::<Document>();
        let mut cursor = doc_coll.aggregate(pipeline).await?;
        if let Some(res) = cursor.next().await {
            let doc = res?;
            let total_size = doc
                .get_i64("totalSize")
                .or_else(|_| doc.get_i32("totalSize").map(|v| v as i64))
                .or_else(|_| doc.get_f64("totalSize").map(|v| v as i64))
                .unwrap_or(0);
            let file_count = doc
                .get_i64("fileCount")
                .or_else(|_| doc.get_i32("fileCount").map(|v| v as i64))
                .unwrap_or(0)
                .max(0) as u64;
            return Ok((total_size, file_count));
        }
        Ok((0, 0))
    }
}
