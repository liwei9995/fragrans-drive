use md5::{Digest, Md5};

pub fn hash_buffer(buffer: &[u8]) -> String {
    let mut hasher = Md5::new();
    hasher.update(buffer);
    hex::encode(hasher.finalize())
}
