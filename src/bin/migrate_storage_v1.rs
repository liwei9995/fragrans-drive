use fragrans::{
    config::Config,
    infrastructure::{
        db::storage_repo::StorageRepository,
        storage::local::{LocalStorage, legacy::LegacyReader},
    },
};
use mongodb::{Client, bson::doc, options::ClientOptions};
use sha2::Sha256;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let apply = args.contains(&"--apply".to_string());

    let config = Config::from_env()?;
    let client_options = ClientOptions::parse(&config.mongo_uri).await?;
    let client = Client::with_options(client_options)?;
    let db_name = env::var("MONGO_DB_NAME").unwrap_or_else(|_| "fragrans".to_string());
    let db = client.database(&db_name);
    let repo = StorageRepository::new(&db);
    let local_storage = LocalStorage::new(
        config.storage_destination.clone(),
        config.storage_master_key,
    )?;
    let legacy_reader = LegacyReader::new(&local_storage);

    let mut scanned = 0;
    let mut already_v1 = 0;
    let mut migratable = 0;
    let mut migrated = 0;
    let mut corrupted = 0;
    let mut missing = 0;
    let mut failed = 0;

    // Find all legacy objects that have md5_hash and no content_hash
    let legacy_items = repo
        .find_many(doc! { "contentHash": { "$exists": false }, "MD5Hash": { "$exists": true } })
        .await?;

    for item in legacy_items {
        scanned += 1;
        if item.content_hash.is_some() {
            already_v1 += 1;
            continue;
        }

        let md5_hash = item.md5_hash.unwrap();
        let fetch_res = legacy_reader
            .fetch(&md5_hash, item.iv.as_deref(), None)
            .await;
        match fetch_res {
            Ok(Some(data)) => {
                use md5::Digest;
                let mut hasher = md5::Md5::new();
                hasher.update(&data);
                let recomputed_md5 = hex::encode(hasher.finalize());
                if recomputed_md5 != md5_hash {
                    corrupted += 1;
                    continue;
                }

                migratable += 1;

                if apply {
                    let mut hasher = Sha256::new();
                    hasher.update(&data);
                    let content_hash = hex::encode(hasher.finalize());

                    let temp_file = tempfile::NamedTempFile::new()?;
                    tokio::fs::write(temp_file.path(), &data).await?;

                    if local_storage
                        .store_from_file(&item.user_id, &content_hash, temp_file.path())
                        .await
                        .is_err()
                    {
                        failed += 1;
                        continue;
                    }

                    let read_back = match local_storage.read_all(&item.user_id, &content_hash).await
                    {
                        Ok(read_back) => read_back,
                        Err(_) => {
                            failed += 1;
                            continue;
                        }
                    };
                    if read_back != data {
                        failed += 1;
                        continue;
                    }

                    let update = doc! {
                        "contentHash": &content_hash,
                        "hashAlgorithm": "sha256",
                        "encryptionFormat": 1,
                    };
                    match repo
                        .update_many_by_md5(&item.user_id, &md5_hash, update)
                        .await
                    {
                        Ok(_) => migrated += 1,
                        Err(_) => failed += 1,
                    }
                }
            }
            Ok(None) => {
                missing += 1;
            }
            Err(_) => {
                failed += 1;
            }
        }
    }

    println!("scanned: {}", scanned);
    println!("already_v1: {}", already_v1);
    println!("migratable: {}", migratable);
    println!("migrated: {}", migrated);
    println!("corrupted: {}", corrupted);
    println!("missing: {}", missing);
    println!("failed: {}", failed);

    if apply && (corrupted > 0 || missing > 0 || failed > 0) {
        std::process::exit(1);
    }

    Ok(())
}
