use mongodb::{
    Collection, Database,
    bson::{DateTime, Document, doc},
};
use sha2::{Digest, Sha256};

pub struct RefreshSessionRepository {
    collection: Collection<Document>,
}

impl RefreshSessionRepository {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection("refresh_sessions"),
        }
    }

    fn hash_jti(jti: &str) -> String {
        hex::encode(Sha256::digest(jti.as_bytes()))
    }

    pub async fn create(
        &self,
        user_id: &str,
        jti: &str,
        token_version: i32,
        expires_at: DateTime,
    ) -> Result<(), mongodb::error::Error> {
        self.collection
            .insert_one(doc! {
                "userId": user_id,
                "jtiHash": Self::hash_jti(jti),
                "tokenVersion": token_version,
                "expiresAt": expires_at,
            })
            .await?;
        Ok(())
    }

    /// Atomically consumes a refresh session. Exactly one concurrent refresh can succeed.
    pub async fn consume(
        &self,
        user_id: &str,
        jti: &str,
        token_version: i32,
    ) -> Result<bool, mongodb::error::Error> {
        let result = self
            .collection
            .delete_one(doc! {
                "userId": user_id,
                "jtiHash": Self::hash_jti(jti),
                "tokenVersion": token_version,
                "expiresAt": { "$gt": DateTime::now() },
            })
            .await?;
        Ok(result.deleted_count == 1)
    }
}
