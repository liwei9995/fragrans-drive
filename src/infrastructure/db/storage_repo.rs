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

    pub async fn find_many(&self, query: Document) -> Result<Vec<Storage>, mongodb::error::Error> {
        let mut cursor = self.collection.find(query).await?;
        let mut results = Vec::new();
        while let Some(item) = cursor.next().await {
            results.push(item?);
        }
        Ok(results)
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
