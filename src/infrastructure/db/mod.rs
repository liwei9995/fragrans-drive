use crate::config::Config;
use mongodb::{Client, Database, IndexModel, options::{ClientOptions, IndexOptions}};
use mongodb::bson::doc;

pub mod storage_repo;
pub mod user_repo;

pub async fn init_db(config: &Config) -> Database {
    let mut client_options = ClientOptions::parse(&config.mongo_uri)
        .await
        .expect("Failed to parse MONGO_URI");

    client_options.app_name = Some("fragrans".to_string());

    let client = Client::with_options(client_options).expect("Failed to initialize MongoDB client");

    let db_name = client
        .default_database()
        .map(|s| s.name().to_string())
        .unwrap_or_else(|| "fragrans".to_string());
    
    let db = client.database(&db_name);
    
    // Ensure indexes are created
    ensure_indexes(&db).await;
    
    db
}

async fn ensure_indexes(db: &Database) {
    let users = db.collection::<mongodb::bson::Document>("users");
    let storage = db.collection::<mongodb::bson::Document>("storage");

    // users: unique email
    let email_index = IndexModel::builder()
        .keys(doc! { "email": 1 })
        .options(IndexOptions::builder().unique(true).build())
        .build();
    let _ = users.create_index(email_index).await;

    // storage: general queries
    let list_index = IndexModel::builder()
        .keys(doc! { "userId": 1, "trashed": 1, "parentId": 1, "type": 1 })
        .build();
    let _ = storage.create_index(list_index).await;

    // storage: deduplication checks
    let hash_index = IndexModel::builder()
        .keys(doc! { "userId": 1, "MD5Hash": 1, "type": 1 })
        .build();
    let _ = storage.create_index(hash_index).await;
}
