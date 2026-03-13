use crate::config::Config;
use mongodb::{Client, Database, options::ClientOptions};

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
    client.database(&db_name)
}
