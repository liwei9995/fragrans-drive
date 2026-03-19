use crate::domain::storage::Storage;
use futures_util::stream::StreamExt;
use mongodb::{
    Collection, Database,
    bson::{Bson, Document, doc, oid::ObjectId},
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

    /// 返回在给定 query 下所有作为 thumbnail 被引用的 _id（用于列表接口排除缩略图单独成条）。
    pub async fn thumbnail_object_ids(&self, query: Document) -> Result<Vec<ObjectId>, mongodb::error::Error> {
        let values = self.collection.distinct("thumbnail", query).await?;
        let mut ids = Vec::new();
        for v in values {
            let s = match v {
                Bson::String(x) if !x.is_empty() => x,
                _ => continue,
            };
            if let Ok(oid) = ObjectId::parse_str(&s) {
                ids.push(oid);
            }
        }
        Ok(ids)
    }

    /// 分页查询：返回 (当前页条目, 总条数)。
    pub async fn find_many_paginated(
        &self,
        query: Document,
        page: u64,
        limit: u64,
    ) -> Result<(Vec<Storage>, u64), mongodb::error::Error> {
        let total = self.collection.count_documents(query.clone()).await?;
        let skip = (page.saturating_sub(1)) * limit;
        let mut cursor = self
            .collection
            .find(query)
            .skip(skip)
            .limit(limit as i64)
            .await?;
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
