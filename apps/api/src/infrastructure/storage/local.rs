use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit, Payload},
};
use rand::RngCore;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Debug, thiserror::Error)]
pub enum StorageIoError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Hash mismatch: expected {expected}, got {actual}")]
    HashMismatch { expected: String, actual: String },
    #[error("Crypto error: {0}")]
    Crypto(String),
    #[error("Format error: {0}")]
    Format(String),
    #[error("Too many chunks")]
    TooManyChunks,
}

#[derive(Clone)]
pub struct LocalStorage {
    root_path: Arc<PathBuf>,
    master_key: Arc<[u8; 32]>,
}

const MAGIC: &[u8; 8] = b"FRAGRNS\0";
const VERSION: u8 = 1;
const CHUNK_SIZE: u32 = 1_048_576;
const HEADER_SIZE: u64 = 33;

pub type StorageStream = Pin<
    Box<dyn futures::Stream<Item = Result<axum::body::Bytes, StorageIoError>> + Send + 'static>,
>;

fn expected_encrypted_size(plaintext_size: u64) -> Result<u64, StorageIoError> {
    let chunk_count = if plaintext_size == 0 {
        1
    } else {
        plaintext_size.div_ceil(CHUNK_SIZE as u64)
    };
    HEADER_SIZE
        .checked_add(plaintext_size)
        .and_then(|size| size.checked_add(chunk_count.checked_mul(16)?))
        .ok_or_else(|| StorageIoError::Format("Encrypted object size overflow".into()))
}

impl LocalStorage {
    pub fn new(root_path: PathBuf, master_key: [u8; 32]) -> Result<Self, std::io::Error> {
        if !root_path.exists() {
            std::fs::create_dir_all(&root_path)?;
        } else if !root_path.is_dir() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Storage root is not a directory",
            ));
        }
        Ok(Self {
            root_path: Arc::new(root_path),
            master_key: Arc::new(master_key),
        })
    }

    fn get_path(&self, user_id: &str, sha256_hash: &str) -> Result<PathBuf, StorageIoError> {
        if mongodb::bson::oid::ObjectId::parse_str(user_id).is_err() {
            return Err(StorageIoError::Format("Invalid user id".into()));
        }
        if sha256_hash.len() != 64
            || !sha256_hash
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(StorageIoError::Format("Invalid SHA-256 hash".into()));
        }

        let mut path = (*self.root_path).clone();
        path.push(user_id);
        path.push(&sha256_hash[0..2]);
        path.push(&sha256_hash[2..4]);
        path.push(&sha256_hash[4..6]);
        path.push(sha256_hash);
        Ok(path)
    }

    pub async fn store_from_async_read<R: tokio::io::AsyncRead + Unpin>(
        &self,
        user_id: &str,
        content_hash: &str,
        mut in_reader: R,
        plaintext_size: u64,
    ) -> Result<(), StorageIoError> {
        let path = self.get_path(user_id, content_hash)?;
        if path.exists() {
            return Ok(());
        }

        let temp_path = path.with_extension("tmp");
        if let Some(parent) = temp_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let result: Result<(), StorageIoError> = async {
            let mut out_file = fs::File::create(&temp_path).await?;
            let mut base_nonce = [0u8; 12];
            rand::thread_rng().fill_bytes(&mut base_nonce);

            out_file.write_all(MAGIC).await?;
            out_file.write_u8(VERSION).await?;
            out_file.write_u32(CHUNK_SIZE).await?;
            out_file.write_u64(plaintext_size).await?;
            out_file.write_all(&base_nonce).await?;

            let master_key = self.master_key.clone();
            let cipher = Aes256Gcm::new(master_key.as_ref().into());

            let mut hasher = Sha256::new();
            let mut buffer = vec![0u8; CHUNK_SIZE as usize];
            let mut chunk_index: u32 = 0;
            let mut total_read = 0;

            loop {
                let mut chunk_bytes = 0;
                while chunk_bytes < CHUNK_SIZE as usize {
                    let n = in_reader.read(&mut buffer[chunk_bytes..]).await?;
                    if n == 0 {
                        break;
                    }
                    chunk_bytes += n;
                }

                if chunk_bytes == 0 && total_read > 0 {
                    break;
                }

                hasher.update(&buffer[..chunk_bytes]);
                total_read += chunk_bytes as u64;

                let mut nonce_bytes = base_nonce;
                let index_bytes = chunk_index.to_be_bytes();
                for i in 0..4 {
                    nonce_bytes[8 + i] ^= index_bytes[i];
                }
                let nonce = Nonce::from(nonce_bytes);

                let mut aad = Vec::with_capacity(33 + user_id.len() + 1 + content_hash.len() + 8);
                aad.extend_from_slice(MAGIC);
                aad.push(VERSION);
                aad.extend_from_slice(&CHUNK_SIZE.to_be_bytes());
                aad.extend_from_slice(&plaintext_size.to_be_bytes());
                aad.extend_from_slice(&base_nonce);
                aad.extend_from_slice(user_id.as_bytes());
                aad.push(0x00);
                aad.extend_from_slice(content_hash.as_bytes());
                aad.extend_from_slice(&chunk_index.to_be_bytes());
                aad.extend_from_slice(&(chunk_bytes as u32).to_be_bytes());

                let cipher = cipher.clone();
                let msg = buffer[..chunk_bytes].to_vec();
                let ciphertext = tokio::task::spawn_blocking(move || {
                    cipher.encrypt(
                        &nonce,
                        Payload {
                            msg: &msg,
                            aad: &aad,
                        },
                    )
                })
                .await
                .map_err(|e| StorageIoError::Crypto(format!("Task error: {}", e)))?
                .map_err(|e| StorageIoError::Crypto(e.to_string()))?;

                out_file.write_all(&ciphertext).await?;

                if chunk_index == u32::MAX {
                    return Err(StorageIoError::TooManyChunks);
                }
                chunk_index += 1;

                if chunk_bytes < CHUNK_SIZE as usize {
                    break;
                }
            }

            let hash_result = hex::encode(hasher.finalize());
            if hash_result != content_hash {
                return Err(StorageIoError::HashMismatch {
                    expected: content_hash.to_string(),
                    actual: hash_result,
                });
            }

            out_file.sync_all().await?;
            drop(out_file);
            fs::rename(&temp_path, &path).await?;
            Ok(())
        }
        .await;

        if result.is_err()
            && let Err(cleanup_error) = fs::remove_file(&temp_path).await
            && cleanup_error.kind() != std::io::ErrorKind::NotFound
        {
            tracing::error!(
                path = %temp_path.display(),
                error = %cleanup_error,
                "failed to remove incomplete storage object"
            );
        }

        result
    }

    pub async fn store_from_file(
        &self,
        user_id: &str,
        content_hash: &str,
        source: &Path,
    ) -> Result<(), StorageIoError> {
        let plaintext_size = fs::metadata(source).await?.len();
        let in_file = fs::File::open(source).await?;
        self.store_from_async_read(user_id, content_hash, in_file, plaintext_size).await
    }

    pub async fn read_all(
        &self,
        user_id: &str,
        content_hash: &str,
    ) -> Result<Vec<u8>, StorageIoError> {
        let path = self.get_path(user_id, content_hash)?;
        if !path.exists() {
            return Err(StorageIoError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "File not found",
            )));
        }

        let mut in_file = fs::File::open(path).await?;

        let mut magic = [0u8; 8];
        in_file.read_exact(&mut magic).await?;
        if &magic != MAGIC {
            return Err(StorageIoError::Format("Bad magic".into()));
        }

        let version = in_file.read_u8().await?;
        if version != VERSION {
            return Err(StorageIoError::Format(format!(
                "Unsupported version: {}",
                version
            )));
        }

        let chunk_size = in_file.read_u32().await?;
        if chunk_size != CHUNK_SIZE {
            return Err(StorageIoError::Format(format!(
                "Invalid chunk size: {}",
                chunk_size
            )));
        }

        let plaintext_size = in_file.read_u64().await?;
        let expected_file_size = expected_encrypted_size(plaintext_size)?;
        if in_file.metadata().await?.len() != expected_file_size {
            return Err(StorageIoError::Format(
                "Encrypted object length does not match header".into(),
            ));
        }

        let mut base_nonce = [0u8; 12];
        in_file.read_exact(&mut base_nonce).await?;

        let cipher = Aes256Gcm::new(self.master_key.as_ref().into());

        let capacity = usize::try_from(plaintext_size)
            .map_err(|_| StorageIoError::Format("Plaintext is too large".into()))?;
        let mut result = Vec::with_capacity(capacity);
        let mut chunk_index: u32 = 0;
        let mut read_plaintext = 0;

        loop {
            let remaining = plaintext_size - read_plaintext;
            if remaining == 0 && chunk_index > 0 {
                let mut extra = [0u8; 1];
                if in_file.read(&mut extra).await? > 0 {
                    return Err(StorageIoError::Format("Extra ciphertext".into()));
                }
                break;
            }

            let expected_plaintext_len = std::cmp::min(CHUNK_SIZE as u64, remaining) as u32;
            let expected_ciphertext_len = expected_plaintext_len as usize + 16;

            let mut ciphertext = vec![0u8; expected_ciphertext_len];
            in_file.read_exact(&mut ciphertext).await?;

            let mut nonce_bytes = base_nonce;
            let index_bytes = chunk_index.to_be_bytes();
            for i in 0..4 {
                nonce_bytes[8 + i] ^= index_bytes[i];
            }
            let nonce = Nonce::from(nonce_bytes);

            let mut aad = Vec::with_capacity(33 + user_id.len() + 1 + content_hash.len() + 8);
            aad.extend_from_slice(MAGIC);
            aad.push(VERSION);
            aad.extend_from_slice(&CHUNK_SIZE.to_be_bytes());
            aad.extend_from_slice(&plaintext_size.to_be_bytes());
            aad.extend_from_slice(&base_nonce);
            aad.extend_from_slice(user_id.as_bytes());
            aad.push(0x00);
            aad.extend_from_slice(content_hash.as_bytes());
            aad.extend_from_slice(&chunk_index.to_be_bytes());
            aad.extend_from_slice(&expected_plaintext_len.to_be_bytes());

            let cipher = cipher.clone();
            let plaintext = tokio::task::spawn_blocking(move || {
                cipher.decrypt(
                    &nonce,
                    Payload {
                        msg: &ciphertext,
                        aad: &aad,
                    },
                )
            })
            .await
            .map_err(|e| StorageIoError::Crypto(format!("Task error: {}", e)))?
            .map_err(|e| StorageIoError::Crypto(e.to_string()))?;

            result.extend_from_slice(&plaintext);
            read_plaintext += plaintext.len() as u64;

            if chunk_index == u32::MAX {
                return Err(StorageIoError::TooManyChunks);
            }
            chunk_index += 1;

            if expected_plaintext_len < CHUNK_SIZE {
                let mut extra = [0u8; 1];
                if in_file.read(&mut extra).await? > 0 {
                    return Err(StorageIoError::Format("Extra ciphertext".into()));
                }
                break;
            }
        }

        Ok(result)
    }

    pub async fn remove(&self, user_id: &str, content_hash: &str) -> Result<(), StorageIoError> {
        let path = self.get_path(user_id, content_hash)?;
        if path.exists() {
            fs::remove_file(path).await?;
        }
        Ok(())
    }

    pub async fn exists(&self, user_id: &str, content_hash: &str) -> Result<bool, StorageIoError> {
        Ok(self.get_path(user_id, content_hash)?.exists())
    }

    pub async fn stream_content(
        &self,
        user_id: String,
        content_hash: String,
        range_start: u64,
        range_end: Option<u64>,
    ) -> Result<(u64, u64, StorageStream), StorageIoError> {
        let path = self.get_path(&user_id, &content_hash)?;
        if !path.exists() {
            return Err(StorageIoError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "File not found",
            )));
        }
        let mut in_file = fs::File::open(path).await?;

        let mut magic = [0u8; 8];
        in_file.read_exact(&mut magic).await?;
        if &magic != MAGIC {
            return Err(StorageIoError::Format("Bad magic".into()));
        }

        let version = in_file.read_u8().await?;
        if version != VERSION {
            return Err(StorageIoError::Format(format!(
                "Unsupported version: {}",
                version
            )));
        }

        let chunk_size = in_file.read_u32().await?;
        if chunk_size != CHUNK_SIZE {
            return Err(StorageIoError::Format(format!(
                "Invalid chunk size: {}",
                chunk_size
            )));
        }

        let plaintext_size = in_file.read_u64().await?;
        let expected_file_size = expected_encrypted_size(plaintext_size)?;
        if in_file.metadata().await?.len() != expected_file_size {
            return Err(StorageIoError::Format(
                "Encrypted object length does not match header".into(),
            ));
        }

        let mut base_nonce = [0u8; 12];
        in_file.read_exact(&mut base_nonce).await?;

        let master_key = self.master_key.clone();

        let actual_end = std::cmp::min(range_end.unwrap_or(plaintext_size.saturating_sub(1)), plaintext_size.saturating_sub(1));
        let range_start = std::cmp::min(range_start, plaintext_size);
        let range_len = if range_start <= actual_end { actual_end - range_start + 1 } else { 0 };
        
        let start_chunk = if range_len == 0 { 0 } else { (range_start / (CHUNK_SIZE as u64)) as u32 };
        let end_chunk = if range_len == 0 { 0 } else { (actual_end / (CHUNK_SIZE as u64)) as u32 };
        
        let file_offset = HEADER_SIZE + (start_chunk as u64 * (CHUNK_SIZE as u64 + 16));
        use std::io::SeekFrom;
        use tokio::io::AsyncSeekExt;
        in_file.seek(SeekFrom::Start(file_offset)).await?;

        struct State {
            in_file: fs::File,
            master_key: Arc<[u8; 32]>,
            user_id: String,
            content_hash: String,
            plaintext_size: u64,
            base_nonce: [u8; 12],
            chunk_index: u32,
            end_chunk: u32,
            range_start: u64,
            range_len: u64,
            done: bool,
        }

        let state = State {
            in_file,
            master_key,
            user_id,
            content_hash,
            plaintext_size,
            base_nonce,
            chunk_index: start_chunk,
            end_chunk,
            range_start,
            range_len,
            done: range_len == 0,
        };

        use futures::StreamExt;
        let stream = futures::stream::unfold(state, |mut s| async move {
            if s.done {
                return None;
            }

            let chunk_offset = s.chunk_index as u64 * CHUNK_SIZE as u64;
            let remaining = s.plaintext_size.saturating_sub(chunk_offset);
            
            if remaining == 0 && s.chunk_index > 0 {
                let mut extra = [0u8; 1];
                match s.in_file.read(&mut extra).await {
                    Ok(n) if n > 0 => {
                        s.done = true;
                        let fut = async move { Err(StorageIoError::Format("Extra ciphertext".into())) };
                        return Some((Box::pin(fut) as Pin<Box<dyn std::future::Future<Output = Result<axum::body::Bytes, StorageIoError>> + Send>>, s));
                    }
                    Ok(_) => return None,
                    Err(e) => {
                        s.done = true;
                        let fut = async move { Err(StorageIoError::Io(e)) };
                        return Some((Box::pin(fut) as Pin<Box<dyn std::future::Future<Output = Result<axum::body::Bytes, StorageIoError>> + Send>>, s));
                    }
                }
            }

            let expected_plaintext_len = std::cmp::min(CHUNK_SIZE as u64, remaining) as u32;
            let expected_ciphertext_len = expected_plaintext_len as usize + 16;

            let mut ciphertext = vec![0u8; expected_ciphertext_len];
            if let Err(e) = s.in_file.read_exact(&mut ciphertext).await {
                s.done = true;
                let fut = async move { Err(StorageIoError::Io(e)) };
                return Some((Box::pin(fut) as Pin<Box<dyn std::future::Future<Output = Result<axum::body::Bytes, StorageIoError>> + Send>>, s));
            }

            let mut nonce_bytes = s.base_nonce;
            let index_bytes = s.chunk_index.to_be_bytes();
            for i in 0..4 {
                nonce_bytes[8 + i] ^= index_bytes[i];
            }
            let nonce = Nonce::from(nonce_bytes);

            let mut aad = Vec::with_capacity(33 + s.user_id.len() + 1 + s.content_hash.len() + 8);
            aad.extend_from_slice(MAGIC);
            aad.push(VERSION);
            aad.extend_from_slice(&CHUNK_SIZE.to_be_bytes());
            aad.extend_from_slice(&s.plaintext_size.to_be_bytes());
            aad.extend_from_slice(&s.base_nonce);
            aad.extend_from_slice(s.user_id.as_bytes());
            aad.push(0x00);
            aad.extend_from_slice(s.content_hash.as_bytes());
            aad.extend_from_slice(&s.chunk_index.to_be_bytes());
            aad.extend_from_slice(&expected_plaintext_len.to_be_bytes());

            let cipher = Aes256Gcm::new(s.master_key.as_ref().into());
            
            let is_last_chunk = s.chunk_index == s.end_chunk;
            let chunk_index = s.chunk_index;
            let range_start = s.range_start;
            let range_len = s.range_len;

            if s.chunk_index == u32::MAX || is_last_chunk {
                s.done = true;
            }
            s.chunk_index += 1;

            let fut = async move {
                let plaintext_result = tokio::task::spawn_blocking(move || {
                    cipher.decrypt(
                        &nonce,
                        Payload {
                            msg: &ciphertext,
                            aad: &aad,
                        },
                    )
                }).await;

                let plaintext = match plaintext_result {
                    Ok(Ok(p)) => p,
                    Ok(Err(e)) => return Err(StorageIoError::Crypto(e.to_string())),
                    Err(e) => return Err(StorageIoError::Crypto(format!("Task error: {}", e))),
                };

                let current_chunk_offset = chunk_index as u64 * CHUNK_SIZE as u64;
                let chunk_end_offset = current_chunk_offset + plaintext.len() as u64;
                let range_end = range_start + range_len;

                let intersect_start = std::cmp::max(current_chunk_offset, range_start);
                let intersect_end = std::cmp::min(chunk_end_offset, range_end);

                if intersect_start < intersect_end {
                    let start_idx = (intersect_start - current_chunk_offset) as usize;
                    let end_idx = (intersect_end - current_chunk_offset) as usize;
                    Ok(axum::body::Bytes::from(plaintext[start_idx..end_idx].to_vec()))
                } else {
                    Ok(axum::body::Bytes::new())
                }
            };
            
            Some((Box::pin(fut) as Pin<Box<dyn std::future::Future<Output = Result<axum::body::Bytes, StorageIoError>> + Send>>, s))
        }).buffered(4);

        Ok((plaintext_size, range_len, Box::pin(stream)))
    }
}
pub mod legacy {
    use super::{LocalStorage, StorageIoError};
    use ctr::cipher::{KeyIvInit, StreamCipher};

    pub struct LegacyReader<'a> {
        storage: &'a LocalStorage,
    }

    impl<'a> LegacyReader<'a> {
        pub fn new(storage: &'a LocalStorage) -> Self {
            Self { storage }
        }

        fn get_legacy_path(&self, md5_hash: &str) -> Result<std::path::PathBuf, StorageIoError> {
            if md5_hash.len() != 32
                || !md5_hash
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            {
                return Err(StorageIoError::Format("Invalid legacy MD5 hash".into()));
            }

            let path = self.storage.root_path.clone();
            let mut p = (*path).clone();
            p.push(&md5_hash[0..2]);
            p.push(&md5_hash[2..4]);
            p.push(&md5_hash[4..6]);
            p.push(md5_hash);
            Ok(p)
        }

        pub async fn fetch(
            &self,
            md5_hash: &str,
            iv: Option<&str>,
            max_bytes: Option<u64>,
        ) -> Result<Option<Vec<u8>>, StorageIoError> {
            let path = self.get_legacy_path(md5_hash)?;
            if !path.exists() {
                return Ok(None);
            }
            if let Some(max_bytes) = max_bytes
                && tokio::fs::metadata(&path).await?.len() > max_bytes
            {
                return Err(StorageIoError::Format(
                    "Legacy object exceeds the read limit".into(),
                ));
            }

            let mut data = tokio::fs::read(&path).await?;

            if let Some(iv_str) = iv {
                let mut iv_bytes = [0u8; 16];
                if hex::decode_to_slice(iv_str, &mut iv_bytes).is_err() {
                    return Err(StorageIoError::Format("Invalid IV hex".into()));
                }

                let key = aes::cipher::generic_array::GenericArray::from_slice(md5_hash.as_bytes());
                let mut cipher = ctr::Ctr128BE::<aes::Aes256>::new(key, &iv_bytes.into());
                cipher.apply_keystream(&mut data);
            }

            Ok(Some(data))
        }
    }
}
