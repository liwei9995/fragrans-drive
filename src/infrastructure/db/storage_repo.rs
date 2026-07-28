use crate::domain::storage::Storage;
use futures_util::stream::StreamExt;
use mongodb::{
    Collection, Database,
    bson::{Document, doc, oid::ObjectId},
};

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

    /// 返回在给定 query 下所有作为 thumbnail 被引用的 _id（用于列表接口排除缩略图单独成条）。
    /// 分页查询：返回 (当前页条目, 总条数)。
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
}
