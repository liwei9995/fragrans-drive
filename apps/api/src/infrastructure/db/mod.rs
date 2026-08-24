use crate::config::Config;
use mongodb::bson::doc;
use mongodb::{
    Client, Database, IndexModel,
    options::{ClientOptions, IndexOptions},
};

pub mod refresh_session_repo;
pub mod storage_repo;
pub mod user_repo;

pub async fn init_db(config: &Config) -> Result<Database, mongodb::error::Error> {
    let mut client_options = ClientOptions::parse(&config.mongo_uri).await?;

    client_options.app_name = Some("fragrans".to_string());

    let client = Client::with_options(client_options)?;

    let db_name = client
        .default_database()
        .map(|s| s.name().to_string())
        .unwrap_or_else(|| "fragrans".to_string());

    let db = client.database(&db_name);

    db.run_command(doc! { "ping": 1 }).await?;

    // Ensure indexes are created
    ensure_indexes(&db).await?;

    Ok(db)
}

pub async fn ensure_indexes(db: &Database) -> Result<(), mongodb::error::Error> {
    let users = db.collection::<mongodb::bson::Document>("users");
    let storage = db.collection::<mongodb::bson::Document>("storage");
    let refresh_sessions = db.collection::<mongodb::bson::Document>("refresh_sessions");

    let email_index = IndexModel::builder()
        .keys(doc! { "email": 1 })
        .options(
            IndexOptions::builder()
                .name("users_email_unique".to_string())
                .unique(true)
                .build(),
        )
        .build();
    users.create_index(email_index).await?;

    let refresh_jti_index = IndexModel::builder()
        .keys(doc! { "jtiHash": 1 })
        .options(
            IndexOptions::builder()
                .name("refresh_jti_unique".to_string())
                .unique(true)
                .build(),
        )
        .build();
    refresh_sessions.create_index(refresh_jti_index).await?;

    let refresh_expiry_index = IndexModel::builder()
        .keys(doc! { "expiresAt": 1 })
        .options(
            IndexOptions::builder()
                .name("refresh_expiry_ttl".to_string())
                .expire_after(std::time::Duration::ZERO)
                .build(),
        )
        .build();
    refresh_sessions.create_index(refresh_expiry_index).await?;

    let list_index = IndexModel::builder()
        .keys(
            doc! { "userId": 1, "trashed": 1, "parentId": 1, "type": 1, "updatedAt": -1, "_id": 1 },
        )
        .options(
            IndexOptions::builder()
                .name("storage_active_list".to_string())
                .build(),
        )
        .build();
    storage.create_index(list_index).await?;

    let walk_index = IndexModel::builder()
        .keys(doc! { "userId": 1, "parentId": 1 })
        .options(
            IndexOptions::builder()
                .name("storage_parent_walk".to_string())
                .build(),
        )
        .build();
    storage.create_index(walk_index).await?;

    let content_index = IndexModel::builder()
        .keys(doc! { "userId": 1, "contentHash": 1 })
        .options(
            IndexOptions::builder()
                .name("storage_content_reference".to_string())
                .build(),
        )
        .build();
    storage.create_index(content_index).await?;

    let folder_name_index = IndexModel::builder()
        .keys(doc! { "userId": 1, "parentId": 1, "name": 1 })
        .options(
            IndexOptions::builder()
                .name("storage_active_folder_name_unique".to_string())
                .unique(true)
                .partial_filter_expression(doc! { "type": "folder", "trashed": false })
                .build(),
        )
        .build();
    storage.create_index(folder_name_index).await?;

    Ok(())
}
