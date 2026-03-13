use aes::Aes256;
use ctr::cipher::{KeyIvInit, StreamCipher};
use rand::RngCore;

type Aes256Ctr64BE = ctr::Ctr128BE<Aes256>;

pub fn get_iv() -> String {
    let mut iv = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut iv);
    hex::encode(iv)
}

pub fn encrypt_buffer(key: &[u8], iv: &[u8], data: &mut [u8]) {
    let mut cipher = Aes256Ctr64BE::new(key.into(), iv.into());
    cipher.apply_keystream(data);
}

// In Rust, the same cipher is used for encryption and decryption in CTR mode
pub fn decrypt_buffer(key: &[u8], iv: &[u8], data: &mut [u8]) {
    encrypt_buffer(key, iv, data);
}
