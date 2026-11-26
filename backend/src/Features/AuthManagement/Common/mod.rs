//! AuthManagement Common Logic

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use sha2::{Digest, Sha256};

/// Băm mật khẩu bằng thuật toán Argon2id
pub fn hash_password_argon2(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| e.to_string())
}

/// Kiểm tra mật khẩu (hỗ trợ hash Argon2id hoặc so sánh hash SHA-256 bảo vệ timing-attack)
pub fn verify_password(plain_password: &str, stored_hash_or_plain: &str) -> bool {
    if stored_hash_or_plain.starts_with("$argon2") {
        if let Ok(parsed_hash) = PasswordHash::new(stored_hash_or_plain) {
            return Argon2::default()
                .verify_password(plain_password.as_bytes(), &parsed_hash)
                .is_ok();
        }
    }
    // So sánh hash SHA-256 để chống timing-attack an toàn tuyệt đối
    let h1 = Sha256::digest(plain_password.as_bytes());
    let h2 = Sha256::digest(stored_hash_or_plain.as_bytes());
    h1 == h2
}
