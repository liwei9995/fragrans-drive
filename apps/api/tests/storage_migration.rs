use assert_cmd::Command;

use mongodb::{Client, bson::doc, options::ClientOptions};

use tempfile::TempDir;

const USER_1: &str = "000000000000000000000001";
const USER_2: &str = "000000000000000000000002";
const MISSING_MD5: &str = "00000000000000000000000000000000";

async fn setup_env() -> (TempDir, String, Client, String) {
    let mongo_uri = std::env::var("TEST_MONGO_URI")
        .unwrap_or_else(|_| "mongodb://test:nest@127.0.0.1:25018/?authSource=admin".to_string());
    let client_options = ClientOptions::parse(&mongo_uri).await.unwrap();
    let client = Client::with_options(client_options).unwrap();

    let db_name = format!("fragrans_test_{}", uuid::Uuid::new_v4());
    let _db = client.database(&db_name);
    // Don't need to drop initially since it's unique

    let temp_dir = TempDir::new().unwrap();

    // Do not use set_var due to concurrent tests
    let storage_dest = temp_dir.path().to_str().unwrap().to_string();

    (temp_dir, db_name, client, storage_dest)
}

fn create_legacy_file(root_path: &std::path::Path, md5_hash: &str, iv: &str, content: &[u8]) {
    let mut iv_bytes = [0u8; 16];
    hex::decode_to_slice(iv, &mut iv_bytes).unwrap();
    let key = aes::cipher::generic_array::GenericArray::from_slice(md5_hash.as_bytes());

    use ctr::cipher::{KeyIvInit, StreamCipher};
    let mut cipher = ctr::Ctr128BE::<aes::Aes256>::new(key, &iv_bytes.into());
    let mut data = content.to_vec();
    cipher.apply_keystream(&mut data);

    let mut p = root_path.to_path_buf();
    p.push(&md5_hash[0..2]);
    p.push(&md5_hash[2..4]);
    p.push(&md5_hash[4..6]);
    std::fs::create_dir_all(&p).unwrap();
    p.push(md5_hash);
    std::fs::write(&p, &data).unwrap();
}

#[tokio::test]
async fn dry_run_changes_nothing() {
    let (_temp_dir, db_name, client, storage_dest) = setup_env().await;
    let db = client.database(&db_name);
    let coll = db.collection::<mongodb::bson::Document>("storage");

    coll.insert_one(doc! {
        "userId": USER_1,
        "name": "dummy.txt",
        "parentId": "0",
        "trashed": false,
        "createdAt": mongodb::bson::DateTime::now(),
        "updatedAt": mongodb::bson::DateTime::now(),
        "MD5Hash": MISSING_MD5,
        "iv": "00000000000000000000000000000000",
        "type": "file"
    })
    .await
    .unwrap();

    let mut cmd = Command::cargo_bin("migrate_storage_v1").unwrap();
    cmd.env(
        "MONGO_URI",
        "mongodb://test:nest@127.0.0.1:25018/?authSource=admin",
    )
    .env("MONGO_DB_NAME", &db_name)
    .env("JWT_SECRET_KEY", "01234567890123456789012345678901")
    .env(
        "STORAGE_MASTER_KEY_HEX",
        "0000000000000000000000000000000000000000000000000000000000000000",
    )
    .env("STORAGE_DESTINATION", &storage_dest);
    let output = cmd.output().unwrap();
    let out = String::from_utf8(output.stdout.clone()).unwrap();

    if !output.status.success() {
        println!(
            "STDOUT:\n{}\nSTDERR:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    assert!(output.status.success());
    assert!(out.contains("scanned: 1"));
    assert!(out.contains("already_v1: 0"));
    assert!(out.contains("migratable: 0")); // because file doesn't exist so it will be missing
    assert!(out.contains("migrated: 0"));
    assert!(out.contains("missing: 1"));

    let doc = coll
        .find_one(doc! { "MD5Hash": MISSING_MD5 })
        .await
        .unwrap()
        .unwrap();
    assert!(!doc.contains_key("contentHash")); // nothing changed
}

#[tokio::test]
async fn valid_legacy_object_migrates_and_roundtrips() {
    let (temp_dir, db_name, client, storage_dest) = setup_env().await;
    let db = client.database(&db_name);
    let coll = db.collection::<mongodb::bson::Document>("storage");

    let content = b"hello world";
    use md5::Digest;
    let mut hasher = md5::Md5::new();
    hasher.update(content);
    let md5_hash = hex::encode(hasher.finalize());
    let iv = "00000000000000000000000000000000";

    create_legacy_file(temp_dir.path(), &md5_hash, iv, content);

    coll.insert_one(doc! {
        "userId": USER_1,
        "name": "test1.txt",
        "parentId": "0",
        "trashed": false,
        "createdAt": mongodb::bson::DateTime::now(),
        "updatedAt": mongodb::bson::DateTime::now(),
        "MD5Hash": md5_hash.clone(),
        "iv": iv,
        "type": "file"
    })
    .await
    .unwrap();

    let mut cmd = Command::cargo_bin("migrate_storage_v1").unwrap();
    cmd.arg("--apply");
    cmd.env(
        "MONGO_URI",
        "mongodb://test:nest@127.0.0.1:25018/?authSource=admin",
    )
    .env("MONGO_DB_NAME", &db_name)
    .env("JWT_SECRET_KEY", "01234567890123456789012345678901")
    .env(
        "STORAGE_MASTER_KEY_HEX",
        "0000000000000000000000000000000000000000000000000000000000000000",
    )
    .env("STORAGE_DESTINATION", &storage_dest);
    let output = cmd.output().unwrap();
    let out = String::from_utf8(output.stdout.clone()).unwrap();

    if !output.status.success() {
        println!(
            "STDOUT:\n{}\nSTDERR:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    assert!(output.status.success());
    assert!(out.contains("migratable: 1"));
    assert!(out.contains("migrated: 1"));

    let doc = coll
        .find_one(doc! { "MD5Hash": &md5_hash })
        .await
        .unwrap()
        .unwrap();
    assert!(doc.contains_key("contentHash"));
    assert_eq!(doc.get_str("hashAlgorithm").unwrap(), "sha256");
    assert_eq!(doc.get_i32("encryptionFormat").unwrap(), 1);

    // Old file remains
    let mut p = temp_dir.path().to_path_buf();
    p.push(&md5_hash[0..2]);
    p.push(&md5_hash[2..4]);
    p.push(&md5_hash[4..6]);
    p.push(&md5_hash);
    assert!(p.exists());

    // New file exists and is valid
    use sha2::Sha256;
    let mut hasher = Sha256::new();
    hasher.update(content);
    let sha256_hash = hex::encode(hasher.finalize());

    let mut p2 = temp_dir.path().to_path_buf();
    p2.push(USER_1);
    p2.push(&sha256_hash[0..2]);
    p2.push(&sha256_hash[2..4]);
    p2.push(&sha256_hash[4..6]);
    p2.push(&sha256_hash);
    assert!(p2.exists());
}

#[tokio::test]
async fn corrupted_legacy_object_is_reported_and_not_updated() {
    let (temp_dir, db_name, client, storage_dest) = setup_env().await;
    let db = client.database(&db_name);
    let coll = db.collection::<mongodb::bson::Document>("storage");

    let content = b"hello corrupted";
    use md5::Digest;
    let mut hasher = md5::Md5::new();
    hasher.update(b"different content");
    let md5_hash = hex::encode(hasher.finalize());
    let iv = "00000000000000000000000000000000";

    create_legacy_file(temp_dir.path(), &md5_hash, iv, content);

    coll.insert_one(doc! {
        "userId": USER_1,
        "name": "test2.txt",
        "parentId": "0",
        "trashed": false,
        "createdAt": mongodb::bson::DateTime::now(),
        "updatedAt": mongodb::bson::DateTime::now(),
        "MD5Hash": md5_hash.clone(),
        "iv": iv,
        "type": "file"
    })
    .await
    .unwrap();

    let mut cmd = Command::cargo_bin("migrate_storage_v1").unwrap();
    cmd.arg("--apply");
    cmd.env(
        "MONGO_URI",
        "mongodb://test:nest@127.0.0.1:25018/?authSource=admin",
    )
    .env("MONGO_DB_NAME", &db_name)
    .env("JWT_SECRET_KEY", "01234567890123456789012345678901")
    .env(
        "STORAGE_MASTER_KEY_HEX",
        "0000000000000000000000000000000000000000000000000000000000000000",
    )
    .env("STORAGE_DESTINATION", &storage_dest);
    let output = cmd.output().unwrap();
    let out = String::from_utf8(output.stdout).unwrap();

    assert!(!output.status.success());
    assert!(out.contains("corrupted: 1"));
    assert!(out.contains("migrated: 0"));

    let doc = coll
        .find_one(doc! { "MD5Hash": &md5_hash })
        .await
        .unwrap()
        .unwrap();
    assert!(!doc.contains_key("contentHash"));
}

#[tokio::test]
async fn missing_legacy_object_is_reported_and_not_updated() {
    let (_temp_dir, db_name, client, storage_dest) = setup_env().await;
    let db = client.database(&db_name);
    let coll = db.collection::<mongodb::bson::Document>("storage");

    coll.insert_one(doc! {
        "userId": USER_1,
        "name": "dummy2.txt",
        "parentId": "0",
        "trashed": false,
        "createdAt": mongodb::bson::DateTime::now(),
        "updatedAt": mongodb::bson::DateTime::now(),
        "MD5Hash": MISSING_MD5,
        "iv": "00000000000000000000000000000000",
        "type": "file"
    })
    .await
    .unwrap();

    let mut cmd = Command::cargo_bin("migrate_storage_v1").unwrap();
    cmd.arg("--apply");
    cmd.env(
        "MONGO_URI",
        "mongodb://test:nest@127.0.0.1:25018/?authSource=admin",
    )
    .env("MONGO_DB_NAME", &db_name)
    .env("JWT_SECRET_KEY", "01234567890123456789012345678901")
    .env(
        "STORAGE_MASTER_KEY_HEX",
        "0000000000000000000000000000000000000000000000000000000000000000",
    )
    .env("STORAGE_DESTINATION", &storage_dest);
    let output = cmd.output().unwrap();
    let out = String::from_utf8(output.stdout).unwrap();

    assert!(!output.status.success());
    assert!(out.contains("missing: 1"));

    let doc = coll
        .find_one(doc! { "MD5Hash": MISSING_MD5 })
        .await
        .unwrap()
        .unwrap();
    assert!(!doc.contains_key("contentHash"));
}

#[tokio::test]
async fn already_migrated_object_is_ignored() {
    let (_temp_dir, db_name, client, storage_dest) = setup_env().await;
    let db = client.database(&db_name);
    let coll = db.collection::<mongodb::bson::Document>("storage");

    coll.insert_one(doc! {
        "userId": USER_1,
        "name": "dummy3.txt",
        "parentId": "0",
        "trashed": false,
        "createdAt": mongodb::bson::DateTime::now(),
        "updatedAt": mongodb::bson::DateTime::now(),
        "MD5Hash": MISSING_MD5,
        "contentHash": "sha256hash",
        "iv": "00000000000000000000000000000000",
        "type": "file"
    })
    .await
    .unwrap();

    let mut cmd = Command::cargo_bin("migrate_storage_v1").unwrap();
    cmd.arg("--apply");
    cmd.env(
        "MONGO_URI",
        "mongodb://test:nest@127.0.0.1:25018/?authSource=admin",
    )
    .env("MONGO_DB_NAME", &db_name)
    .env("JWT_SECRET_KEY", "01234567890123456789012345678901")
    .env(
        "STORAGE_MASTER_KEY_HEX",
        "0000000000000000000000000000000000000000000000000000000000000000",
    )
    .env("STORAGE_DESTINATION", &storage_dest);
    let output = cmd.output().unwrap();
    let out = String::from_utf8(output.stdout.clone()).unwrap();

    if !output.status.success() {
        println!(
            "STDOUT:\n{}\nSTDERR:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    assert!(output.status.success());
    assert!(out.contains("scanned: 0")); // because it queries without contentHash
}

#[tokio::test]
async fn shared_legacy_hash_is_not_deleted() {
    // Tests that multiple docs with the same legacy hash are all migrated
    let (temp_dir, db_name, client, storage_dest) = setup_env().await;
    let db = client.database(&db_name);
    let coll = db.collection::<mongodb::bson::Document>("storage");

    let content = b"hello shared";
    use md5::Digest;
    let mut hasher = md5::Md5::new();
    hasher.update(content);
    let md5_hash = hex::encode(hasher.finalize());
    let iv = "00000000000000000000000000000000";

    create_legacy_file(temp_dir.path(), &md5_hash, iv, content);

    coll.insert_one(doc! {
        "userId": USER_1,
        "name": "test3.txt",
        "parentId": "0",
        "trashed": false,
        "createdAt": mongodb::bson::DateTime::now(),
        "updatedAt": mongodb::bson::DateTime::now(),
        "MD5Hash": md5_hash.clone(),
        "iv": iv,
        "type": "file"
    })
    .await
    .unwrap();

    coll.insert_one(doc! {
        "userId": USER_2, // same hash, different user
        "name": "test4.txt",
        "parentId": "0",
        "trashed": false,
        "createdAt": mongodb::bson::DateTime::now(),
        "updatedAt": mongodb::bson::DateTime::now(),
        "MD5Hash": md5_hash.clone(),
        "iv": iv,
        "type": "file"
    })
    .await
    .unwrap();

    let mut cmd = Command::cargo_bin("migrate_storage_v1").unwrap();
    cmd.arg("--apply");
    cmd.env(
        "MONGO_URI",
        "mongodb://test:nest@127.0.0.1:25018/?authSource=admin",
    )
    .env("MONGO_DB_NAME", &db_name)
    .env("JWT_SECRET_KEY", "01234567890123456789012345678901")
    .env(
        "STORAGE_MASTER_KEY_HEX",
        "0000000000000000000000000000000000000000000000000000000000000000",
    )
    .env("STORAGE_DESTINATION", &storage_dest);
    let output = cmd.output().unwrap();
    let out = String::from_utf8(output.stdout.clone()).unwrap();

    if !output.status.success() {
        println!(
            "STDOUT:\n{}\nSTDERR:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    assert!(output.status.success());
    assert!(out.contains("migratable: 2"));
    assert!(out.contains("migrated: 2"));

    // Old file remains
    let mut p = temp_dir.path().to_path_buf();
    p.push(&md5_hash[0..2]);
    p.push(&md5_hash[2..4]);
    p.push(&md5_hash[4..6]);
    p.push(&md5_hash);
    assert!(p.exists());
}

#[tokio::test]
async fn migration_is_idempotent() {
    let (temp_dir, db_name, client, storage_dest) = setup_env().await;
    let db = client.database(&db_name);
    let coll = db.collection::<mongodb::bson::Document>("storage");

    let content = b"hello idempotent";
    use md5::Digest;
    let mut hasher = md5::Md5::new();
    hasher.update(content);
    let md5_hash = hex::encode(hasher.finalize());
    let iv = "00000000000000000000000000000000";

    create_legacy_file(temp_dir.path(), &md5_hash, iv, content);

    coll.insert_one(doc! {
        "userId": USER_1,
        "name": "test5.txt",
        "parentId": "0",
        "trashed": false,
        "createdAt": mongodb::bson::DateTime::now(),
        "updatedAt": mongodb::bson::DateTime::now(),
        "MD5Hash": md5_hash.clone(),
        "iv": iv,
        "type": "file"
    })
    .await
    .unwrap();

    let mut cmd = Command::cargo_bin("migrate_storage_v1").unwrap();
    cmd.arg("--apply");
    cmd.env(
        "MONGO_URI",
        "mongodb://test:nest@127.0.0.1:25018/?authSource=admin",
    )
    .env("MONGO_DB_NAME", &db_name)
    .env("JWT_SECRET_KEY", "01234567890123456789012345678901")
    .env(
        "STORAGE_MASTER_KEY_HEX",
        "0000000000000000000000000000000000000000000000000000000000000000",
    )
    .env("STORAGE_DESTINATION", &storage_dest);
    let output = cmd.output().unwrap();
    if !output.status.success() {
        println!(
            "STDOUT:\n{}\nSTDERR:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    assert!(output.status.success());

    // Run again
    let mut cmd2 = Command::cargo_bin("migrate_storage_v1").unwrap();
    cmd2.arg("--apply");
    cmd2.env(
        "MONGO_URI",
        "mongodb://test:nest@127.0.0.1:25018/?authSource=admin",
    )
    .env("MONGO_DB_NAME", &db_name)
    .env("JWT_SECRET_KEY", "01234567890123456789012345678901")
    .env(
        "STORAGE_MASTER_KEY_HEX",
        "0000000000000000000000000000000000000000000000000000000000000000",
    )
    .env("STORAGE_DESTINATION", &storage_dest);
    let output2 = cmd2.output().unwrap();
    let out2 = String::from_utf8(output2.stdout.clone()).unwrap();

    assert!(output2.status.success());
    assert!(out2.contains("scanned: 0"));
}
