use bcrypt::{hash, verify};

/// 密码仅以 bcrypt 哈希形式存储，禁止明文落库。cost 12 约 200ms/次，兼顾安全与性能。
const PASSWORD_COST: u32 = 12;

pub fn hash_password(password: &str) -> String {
    hash(password, PASSWORD_COST).unwrap()
}

pub fn verify_password(password: &str, hashed: &str) -> bool {
    verify(password, hashed).unwrap_or(false)
}
