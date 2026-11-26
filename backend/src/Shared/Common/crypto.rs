//! Module Mã Hóa Đối Xứng AES-256-GCM (Secrets Encryption at Rest)

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use sha2::{Digest, Sha256};
use std::fmt;

#[derive(Debug)]
pub enum CryptoError {
    InvalidKeyLength,
    EncryptionFailed,
    DecryptionFailed,
    InvalidBase64,
    InvalidPayload,
}

impl fmt::Display for CryptoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidKeyLength => write!(f, "Khóa AES-256 phải có độ dài đúng 32 bytes"),
            Self::EncryptionFailed => write!(f, "Mã hóa dữ liệu AES-256-GCM thất bại"),
            Self::DecryptionFailed => write!(f, "Giải mã dữ liệu AES-256-GCM thất bại"),
            Self::InvalidBase64 => write!(f, "Chuỗi base64 ciphertext không hợp lệ"),
            Self::InvalidPayload => write!(f, "Dữ liệu payload ciphertext quá ngắn hoặc bị hỏng"),
        }
    }
}

impl std::error::Error for CryptoError {}

/// Chuyển đổi một passphrase hoặc chuỗi master key bất kỳ thành khóa 32 bytes chuẩn
pub fn derive_master_key(raw_key: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(raw_key.as_bytes());
    let result = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&result);
    key
}

/// Mã hóa chuỗi plaintext bằng AES-256-GCM
///
/// Output: chuỗi Base64 chứa `12 bytes Nonce || Ciphertext + Tag`
pub fn encrypt_aes_gcm(plaintext: &str, key: &[u8; 32]) -> Result<String, CryptoError> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| CryptoError::InvalidKeyLength)?;

    // Sinh Nonce ngẫu nhiên 96-bit (12 bytes)
    let nonce_bytes: [u8; 12] = {
        let mut n = [0u8; 12];
        for (i, b) in n.iter_mut().enumerate() {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos();
            *b = ((now >> (i * 8)) ^ (i as u128 * 37)) as u8;
        }
        n
    };
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|_| CryptoError::EncryptionFailed)?;

    // Ghép nonce (12 bytes) + ciphertext
    let mut combined = Vec::with_capacity(nonce_bytes.len() + ciphertext.len());
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);

    // Encode Base64 thủ công gọn nhẹ hoặc dùng macro chuẩn
    Ok(base64_encode(&combined))
}

/// Giải mã chuỗi Base64 bằng AES-256-GCM
pub fn decrypt_aes_gcm(ciphertext_b64: &str, key: &[u8; 32]) -> Result<String, CryptoError> {
    let combined = base64_decode(ciphertext_b64).map_err(|_| CryptoError::InvalidBase64)?;
    if combined.len() < 12 {
        return Err(CryptoError::InvalidPayload);
    }

    let (nonce_bytes, encrypted) = combined.split_at(12);
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| CryptoError::InvalidKeyLength)?;
    let nonce = Nonce::from_slice(nonce_bytes);

    let decrypted = cipher
        .decrypt(nonce, encrypted)
        .map_err(|_| CryptoError::DecryptionFailed)?;

    String::from_utf8(decrypted).map_err(|_| CryptoError::DecryptionFailed)
}

const BASE64_ALPHABET: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn base64_encode(data: &[u8]) -> String {
    let mut result = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as usize;
        let b1 = if chunk.len() > 1 { chunk[1] as usize } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as usize } else { 0 };

        let n = (b0 << 16) | (b1 << 8) | b2;

        result.push(BASE64_ALPHABET[(n >> 18) & 63] as char);
        result.push(BASE64_ALPHABET[(n >> 12) & 63] as char);

        if chunk.len() > 1 {
            result.push(BASE64_ALPHABET[(n >> 6) & 63] as char);
        } else {
            result.push('=');
        }

        if chunk.len() > 2 {
            result.push(BASE64_ALPHABET[n & 63] as char);
        } else {
            result.push('=');
        }
    }
    result
}

fn base64_decode(s: &str) -> Result<Vec<u8>, ()> {
    let clean = s.trim().replace(['\r', '\n', ' '], "");
    let mut table = [0xFFu8; 256];
    for (i, &c) in BASE64_ALPHABET.iter().enumerate() {
        table[c as usize] = i as u8;
    }

    let bytes = clean.as_bytes();
    let mut out = Vec::with_capacity((bytes.len() / 4) * 3);

    for chunk in bytes.chunks(4) {
        if chunk.is_empty() {
            break;
        }
        let c0 = table[chunk[0] as usize];
        let c1 = if chunk.len() > 1 { table[chunk[1] as usize] } else { 0 };
        let c2 = if chunk.len() > 2 && chunk[2] != b'=' { table[chunk[2] as usize] } else { 0 };
        let c3 = if chunk.len() > 3 && chunk[3] != b'=' { table[chunk[3] as usize] } else { 0 };

        if c0 == 0xFF || c1 == 0xFF {
            return Err(());
        }

        let n = ((c0 as u32) << 18) | ((c1 as u32) << 12) | ((c2 as u32) << 6) | (c3 as u32);
        out.push(((n >> 16) & 0xFF) as u8);

        if chunk.len() > 2 && chunk[2] != b'=' {
            out.push(((n >> 8) & 0xFF) as u8);
        }
        if chunk.len() > 3 && chunk[3] != b'=' {
            out.push((n & 0xFF) as u8);
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes_encrypt_decrypt_roundtrip() {
        let master = derive_master_key("my-super-secret-master-key-2026");
        let plaintext = "47e84a329ee0915f27fab83d9215d45b69c067eb2c5cf3a3";

        let cipher = encrypt_aes_gcm(plaintext, &master).expect("Encryption ok");
        assert_ne!(cipher, plaintext);

        let decrypted = decrypt_aes_gcm(&cipher, &master).expect("Decryption ok");
        assert_eq!(decrypted, plaintext);
    }
}
