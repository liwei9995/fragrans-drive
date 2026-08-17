use bcrypt::{hash, verify};

/// Passwords are stored only as bcrypt hashes (never plaintext). Cost 12 is ~200ms/hash, balancing security and latency.
const PASSWORD_COST: u32 = 12;

pub fn hash_password(password: &str) -> String {
    hash(password, PASSWORD_COST).unwrap()
}

pub fn verify_password(password: &str, hashed: &str) -> bool {
    verify(password, hashed).unwrap_or(false)
}
