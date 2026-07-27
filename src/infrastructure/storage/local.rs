use crate::utils::encryption::{decrypt_buffer, encrypt_buffer};
use std::env;
use std::path::PathBuf;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub struct LocalStorage {
    root_path: PathBuf,
}

impl Default for LocalStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl LocalStorage {
    pub fn new() -> Self {
        let root = env::var("STORAGE_DESTINATION").unwrap_or_else(|_| "bucket/storage".to_string());

        let root_path = PathBuf::from(root);
        if !root_path.exists() {
            std::fs::create_dir_all(&root_path).unwrap();
        }

        Self { root_path }
    }

    fn get_path(&self, id: &str) -> PathBuf {
        // Sharding like legacy: first 2 chars, next 2 chars, next 2 chars
        let mut path = self.root_path.clone();
        if id.len() >= 6 {
            path.push(&id[0..2]);
            path.push(&id[2..4]);
            path.push(&id[4..6]);
        }
        path.push(id);
        path
    }

    #[allow(dead_code)]
    pub async fn exists(&self, id: &str) -> bool {
        self.get_path(id).exists()
    }

    pub async fn store(
        &self,
        id: &str,
        mut data: Vec<u8>,
        iv: Option<&str>,
    ) -> Result<(), std::io::Error> {
        let path = self.get_path(id);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }

        if let Some(iv_hex) = iv {
            let iv_bytes = hex::decode(iv_hex)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;
            // Use ID hash as key for consistency (though legacy used ID as string, scrypt was used in some parts but encryptBuffer used key directly)
            // Legacy encryption.ts: encryptBuffer = (key: string, iv: string, content)
            // It seems it used the hash (id) as the key.
            encrypt_buffer(id.as_bytes(), &iv_bytes, &mut data);
        }

        let mut file = fs::File::create(path).await?;
        file.write_all(&data).await?;
        Ok(())
    }

    
    pub async fn store_from_file(
        &self,
        id: &str,
        temp_file_path: &PathBuf,
        iv: Option<&str>,
    ) -> Result<(), std::io::Error> {
        let path = self.get_path(id);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let mut in_file = fs::File::open(temp_file_path).await?;
        let mut out_file = fs::File::create(path).await?;
        
        if let Some(iv_hex) = iv {
            let iv_bytes = hex::decode(iv_hex)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;
            
            use ctr::cipher::{KeyIvInit, StreamCipher};
            type Aes256Ctr64BE = ctr::Ctr128BE<aes::Aes256>;
            
            let mut cipher = Aes256Ctr64BE::new(id.as_bytes().into(), iv_bytes.as_slice().into());
            
            let mut buffer = [0u8; 8192];
            loop {
                let n = in_file.read(&mut buffer).await?;
                if n == 0 { break; }
                cipher.apply_keystream(&mut buffer[..n]);
                out_file.write_all(&buffer[..n]).await?;
            }
        } else {
            tokio::io::copy(&mut in_file, &mut out_file).await?;
        }
        
        Ok(())
    }

    pub async fn fetch(
        &self,
        id: &str,
        iv: Option<&str>,
    ) -> Result<Option<Vec<u8>>, std::io::Error> {
        let path = self.get_path(id);
        if !path.exists() {
            return Ok(None);
        }

        let mut file = fs::File::open(path).await?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await?;

        if let Some(iv_hex) = iv {
            let iv_bytes = hex::decode(iv_hex)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;
            decrypt_buffer(id.as_bytes(), &iv_bytes, &mut buffer);
        }

        Ok(Some(buffer))
    }

    #[allow(dead_code)]
    pub async fn remove(&self, id: &str) -> Result<(), std::io::Error> {
        let path = self.get_path(id);
        if path.exists() {
            fs::remove_file(path).await?;
            // Optional: clear empty parent dirs (like clearDir in legacy)
            // For now, simple remove is enough for the prototype
        }
        Ok(())
    }
}
