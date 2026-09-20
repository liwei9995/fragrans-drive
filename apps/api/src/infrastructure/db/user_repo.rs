use crate::domain::user::User;
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

    pub async fn find_by_id(&self, id: ObjectId) -> Result<Option<User>, mongodb::error::Error> {
        self.collection.find_one(doc! { "_id": id }).await
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, mongodb::error::Error> {
        let email_trimmed = email.trim();
        let email_norm = email_trimmed.to_lowercase();
        self.collection
            .find_one(doc! {
                "$or": [
                    { "email": &email_norm },
                    { "email": { "$regex": format!("^{}$", regex::escape(email_trimmed)), "$options": "i" } }
                ]
            })
            .await
    }

    pub async fn create(&self, mut user: User) -> Result<ObjectId, mongodb::error::Error> {
        user.email = user.email.trim().to_lowercase();
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
                doc! {
                    "$set": { "password": hashed_password },
                    "$inc": { "tokenVersion": 1 }
                },
            )
            .await?;
        Ok(())
    }

    pub async fn add_passkey(
        &self,
        id: ObjectId,
        passkey: crate::domain::user::StoredPasskey,
    ) -> Result<(), mongodb::error::Error> {
        let passkey_doc = mongodb::bson::to_document(&passkey)?;
        self.collection
            .update_one(
                doc! { "_id": id },
                doc! { "$push": { "passkeys": passkey_doc } },
            )
            .await?;
        Ok(())
    }

    pub async fn delete_passkey(
        &self,
        id: ObjectId,
        passkey_id: &str,
    ) -> Result<bool, mongodb::error::Error> {
        let result = self.collection
            .update_one(
                doc! { "_id": id, "passkeys.id": passkey_id },
                doc! { "$pull": { "passkeys": { "id": passkey_id } }, "$inc": { "tokenVersion": 1 } },
            )
            .await?;
        Ok(result.modified_count == 1)
    }

    pub async fn find_by_passkey_id(
        &self,
        passkey_id: &str,
    ) -> Result<Option<User>, mongodb::error::Error> {
        self.collection
            .find_one(doc! { "passkeys.id": passkey_id })
            .await
    }

    pub async fn update_passkey(
        &self,
        id: ObjectId,
        passkey_id: &str,
        old_passkey_json: &str,
        updated_passkey_json: &str,
    ) -> Result<bool, mongodb::error::Error> {
        let result = self.collection
            .update_one(
                doc! { "_id": id, "passkeys": { "$elemMatch": { "id": passkey_id, "passkeyJson": old_passkey_json } } },
                doc! { "$set": { "passkeys.$.passkeyJson": updated_passkey_json } },
            )
            .await?;
        Ok(result.matched_count == 1)
    }

    #[allow(dead_code)]
    pub async fn delete_all(&self) -> Result<u64, mongodb::error::Error> {
        let result = self.collection.delete_many(doc! {}).await?;
        Ok(result.deleted_count)
    }
}
