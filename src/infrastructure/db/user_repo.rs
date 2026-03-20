use crate::domain::user::User;
use futures_util::stream::StreamExt;
use mongodb::{
    Collection, Database,
    bson::{doc, oid::ObjectId},
};

pub struct UserRepository {
    collection: Collection<User>,
}

impl UserRepository {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection("users"),
        }
    }

    pub async fn find_all(&self) -> Result<Vec<User>, mongodb::error::Error> {
        let mut cursor = self.collection.find(doc! {}).await?;
        let mut users = Vec::new();
        while let Some(user) = cursor.next().await {
            users.push(user?);
        }
        Ok(users)
    }

    pub async fn find_by_id(&self, id: ObjectId) -> Result<Option<User>, mongodb::error::Error> {
        self.collection.find_one(doc! { "_id": id }).await
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, mongodb::error::Error> {
        self.collection.find_one(doc! { "email": email }).await
    }

    pub async fn create(&self, user: User) -> Result<ObjectId, mongodb::error::Error> {
        let result = self.collection.insert_one(user).await?;
        Ok(result.inserted_id.as_object_id().unwrap())
    }

    pub async fn update_profile(
        &self,
        id: ObjectId,
        update: mongodb::bson::Document,
    ) -> Result<Option<User>, mongodb::error::Error> {
        let result = self
            .collection
            .update_one(doc! { "_id": id }, doc! { "$set": update })
            .await?;
        if result.matched_count == 0 {
            return Ok(None);
        }
        self.collection.find_one(doc! { "_id": id }).await
    }

    pub async fn update_password(
        &self,
        id: ObjectId,
        hashed_password: &str,
    ) -> Result<(), mongodb::error::Error> {
        self.collection
            .update_one(
                doc! { "_id": id },
                doc! { "$set": { "password": hashed_password } },
            )
            .await?;
        Ok(())
    }

    pub async fn delete_one(&self, id: ObjectId) -> Result<Option<User>, mongodb::error::Error> {
        self.collection
            .find_one_and_delete(doc! { "_id": id })
            .await
    }

    #[allow(dead_code)]
    pub async fn delete_all(&self) -> Result<u64, mongodb::error::Error> {
        let result = self.collection.delete_many(doc! {}).await?;
        Ok(result.deleted_count)
    }
}
