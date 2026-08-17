use fragrans::infrastructure::storage::local::{LocalStorage, StorageIoError};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use tempfile::TempDir;
use tokio::fs;
use tokio::io::AsyncWriteExt;

const MASTER_KEY: [u8; 32] = [1u8; 32];
const USER_1: &str = "000000000000000000000001";
const USER_2: &str = "000000000000000000000002";

async fn setup_storage() -> (TempDir, LocalStorage) {
    let temp_dir = TempDir::new().unwrap();
    let storage = LocalStorage::new(temp_dir.path().to_path_buf(), MASTER_KEY).unwrap();
    (temp_dir, storage)
}

async fn create_temp_file(content: &[u8]) -> (TempDir, PathBuf, String) {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("source.tmp");
    let mut file = fs::File::create(&path).await.unwrap();
    file.write_all(content).await.unwrap();

    let mut hasher = Sha256::new();
    hasher.update(content);
    let hash = hex::encode(hasher.finalize());

    (dir, path, hash)
}

#[tokio::test]
async fn encrypted_roundtrip_for_boundary_sizes() {
    let (_sd, storage) = setup_storage().await;
    let chunk_size = 1_048_576;

    let sizes = vec![
        0,
        1,
        chunk_size - 1,
        chunk_size,
        chunk_size + 1,
        2 * chunk_size + 17,
    ];

    for size in sizes {
        let content = vec![42u8; size];
        let (_td, path, hash) = create_temp_file(&content).await;

        storage.store_from_file(USER_1, &hash, &path).await.unwrap();
        assert!(storage.exists(USER_1, &hash).await.unwrap());

        let read_back = storage.read_all(USER_1, &hash).await.unwrap();
        assert_eq!(read_back, content, "Failed for size {}", size);
    }
}

#[tokio::test]
async fn tampered_header_is_rejected() {
    let (_sd, storage) = setup_storage().await;
    let content = vec![1, 2, 3];
    let (_td, path, hash) = create_temp_file(&content).await;

    storage.store_from_file(USER_1, &hash, &path).await.unwrap();

    // Tamper header
    let file_path = _sd
        .path()
        .join(USER_1)
        .join(&hash[0..2])
        .join(&hash[2..4])
        .join(&hash[4..6])
        .join(&hash);
    let mut data = fs::read(&file_path).await.unwrap();
    data[0] = b'X'; // alter magic
    fs::write(&file_path, &data).await.unwrap();

    let err = storage.read_all(USER_1, &hash).await.unwrap_err();
    assert!(matches!(err, StorageIoError::Format(_)));
}

#[tokio::test]
async fn tampered_ciphertext_is_rejected() {
    let (_sd, storage) = setup_storage().await;
    let content = vec![1, 2, 3];
    let (_td, path, hash) = create_temp_file(&content).await;

    storage.store_from_file(USER_1, &hash, &path).await.unwrap();

    // Tamper ciphertext (after 33 bytes header)
    let file_path = _sd
        .path()
        .join(USER_1)
        .join(&hash[0..2])
        .join(&hash[2..4])
        .join(&hash[4..6])
        .join(&hash);
    let mut data = fs::read(&file_path).await.unwrap();
    data[35] ^= 1;
    fs::write(&file_path, &data).await.unwrap();

    let err = storage.read_all(USER_1, &hash).await.unwrap_err();
    assert!(matches!(err, StorageIoError::Crypto(_)));
}

#[tokio::test]
async fn wrong_key_is_rejected() {
    let (_sd, storage) = setup_storage().await;
    let content = vec![1, 2, 3];
    let (_td, path, hash) = create_temp_file(&content).await;

    storage.store_from_file(USER_1, &hash, &path).await.unwrap();

    let wrong_key_storage = LocalStorage::new(_sd.path().to_path_buf(), [2u8; 32]).unwrap();
    let err = wrong_key_storage.read_all(USER_1, &hash).await.unwrap_err();
    assert!(matches!(err, StorageIoError::Crypto(_)));
}

#[tokio::test]
async fn wrong_user_is_rejected() {
    let (_sd, storage) = setup_storage().await;
    let content = vec![1, 2, 3];
    let (_td, path, hash) = create_temp_file(&content).await;

    storage.store_from_file(USER_1, &hash, &path).await.unwrap();

    // We can't just read_all with USER_2 because the path includes user_id and it wouldn't exist.
    // So we copy the file to USER_2's path, simulating someone trying to use USER_1's file as their own.
    let path1 = _sd
        .path()
        .join(USER_1)
        .join(&hash[0..2])
        .join(&hash[2..4])
        .join(&hash[4..6])
        .join(&hash);
    let dir2 = _sd
        .path()
        .join(USER_2)
        .join(&hash[0..2])
        .join(&hash[2..4])
        .join(&hash[4..6]);
    fs::create_dir_all(&dir2).await.unwrap();
    let path2 = dir2.join(&hash);
    fs::copy(&path1, &path2).await.unwrap();

    let err = storage.read_all(USER_2, &hash).await.unwrap_err();
    assert!(matches!(err, StorageIoError::Crypto(_))); // Tag should fail because AAD includes user_id
}

#[tokio::test]
async fn wrong_hash_is_rejected() {
    let (_sd, storage) = setup_storage().await;
    let content = vec![1, 2, 3];
    let (_td, path, hash) = create_temp_file(&content).await;

    storage.store_from_file(USER_1, &hash, &path).await.unwrap();

    let mut wrong_hash = hash.clone();
    wrong_hash.replace_range(0..2, "ff");

    let path1 = _sd
        .path()
        .join(USER_1)
        .join(&hash[0..2])
        .join(&hash[2..4])
        .join(&hash[4..6])
        .join(&hash);
    let dir2 = _sd
        .path()
        .join(USER_1)
        .join(&wrong_hash[0..2])
        .join(&wrong_hash[2..4])
        .join(&wrong_hash[4..6]);
    fs::create_dir_all(&dir2).await.unwrap();
    let path2 = dir2.join(&wrong_hash);
    fs::copy(&path1, &path2).await.unwrap();

    let err = storage.read_all(USER_1, &wrong_hash).await.unwrap_err();
    assert!(matches!(err, StorageIoError::Crypto(_))); // AAD includes content_hash
}

#[tokio::test]
async fn same_content_for_different_users_uses_different_paths() {
    let (_sd, storage) = setup_storage().await;
    let content = vec![1, 2, 3];
    let (_td, path, hash) = create_temp_file(&content).await;

    storage.store_from_file(USER_1, &hash, &path).await.unwrap();
    storage.store_from_file(USER_2, &hash, &path).await.unwrap();

    let path1 = _sd
        .path()
        .join(USER_1)
        .join(&hash[0..2])
        .join(&hash[2..4])
        .join(&hash[4..6])
        .join(&hash);
    let path2 = _sd
        .path()
        .join(USER_2)
        .join(&hash[0..2])
        .join(&hash[2..4])
        .join(&hash[4..6])
        .join(&hash);

    assert!(path1.exists());
    assert!(path2.exists());
    assert_ne!(path1, path2);
}

#[tokio::test]
async fn failed_write_leaves_no_temp_or_partial_file() {
    let (_sd, storage) = setup_storage().await;
    let content = vec![1, 2, 3];
    let (_td, path, hash) = create_temp_file(&content).await;

    // Store with a wrong hash to trigger failure
    let wrong_hash = hash.replace('a', "b").replace('1', "2"); // just to be different
    let res = storage.store_from_file(USER_1, &wrong_hash, &path).await;
    assert!(res.is_err());

    let dir = _sd
        .path()
        .join(USER_1)
        .join(&wrong_hash[0..2])
        .join(&wrong_hash[2..4])
        .join(&wrong_hash[4..6]);

    // Check neither a temporary nor final file exists.
    assert!(!dir.join(&wrong_hash).exists());
    let entries = std::fs::read_dir(&dir)
        .map(|entries| entries.count())
        .unwrap_or_default();
    assert_eq!(entries, 0);
}

#[tokio::test]
async fn concurrent_identical_writes_produce_one_readable_object() {
    let (_sd, storage) = setup_storage().await;
    let content = vec![7; 3 * 1_048_576 + 17];
    let (_td, path, hash) = create_temp_file(&content).await;

    let s1 = storage.clone();
    let s2 = storage.clone();
    let p1 = path.clone();
    let p2 = path.clone();
    let h1 = hash.clone();
    let h2 = hash.clone();

    let j1 = tokio::spawn(async move { s1.store_from_file(USER_1, &h1, &p1).await });

    let j2 = tokio::spawn(async move { s2.store_from_file(USER_1, &h2, &p2).await });

    let (r1, r2) = tokio::join!(j1, j2);
    // At least one should succeed (atomic rename might overwrite or fail if it's identical)
    // Actually, both might succeed since they write to their own .tmp files, then rename to the same target.
    // They just overwrite each other with identical valid content.
    assert!(r1.unwrap().is_ok() || r2.unwrap().is_ok());

    let read_back = storage.read_all(USER_1, &hash).await.unwrap();
    assert_eq!(read_back, content);

    let object_dir = _sd
        .path()
        .join(USER_1)
        .join(&hash[0..2])
        .join(&hash[2..4])
        .join(&hash[4..6]);
    assert_eq!(std::fs::read_dir(object_dir).unwrap().count(), 1);
}

#[tokio::test]
async fn invalid_storage_identifiers_are_rejected() {
    let (_sd, storage) = setup_storage().await;
    let content = b"content";
    let (_td, path, hash) = create_temp_file(content).await;

    let escaped_name = format!("escape-{}", uuid::Uuid::new_v4());
    let invalid_user_id = format!("../{escaped_name}");
    let invalid_user = storage
        .store_from_file(&invalid_user_id, &hash, &path)
        .await;
    assert!(matches!(invalid_user, Err(StorageIoError::Format(_))));

    let invalid_hash = storage
        .store_from_file(USER_1, &hash.to_uppercase(), &path)
        .await;
    assert!(matches!(invalid_hash, Err(StorageIoError::Format(_))));
    assert!(!_sd.path().parent().unwrap().join(escaped_name).exists());
}

#[tokio::test]
async fn two_users_upload_same_content_and_both_download_original() {
    let (_sd, storage) = setup_storage().await;
    let content = vec![1, 2, 3, 4, 5];
    let (_td, path, hash) = create_temp_file(&content).await;

    storage.store_from_file(USER_1, &hash, &path).await.unwrap();
    storage.store_from_file(USER_2, &hash, &path).await.unwrap();

    let read1 = storage.read_all(USER_1, &hash).await.unwrap();
    let read2 = storage.read_all(USER_2, &hash).await.unwrap();

    assert_eq!(read1, content);
    assert_eq!(read2, content);
}
