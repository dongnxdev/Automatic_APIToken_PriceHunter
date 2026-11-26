//! Module Che Giấu Thông Tin Nhạy Cảm (Secret Masking & URL Redaction)

use regex::Regex;
use std::sync::OnceLock;

static URL_KEY_REGEX: OnceLock<Regex> = OnceLock::new();

fn get_url_key_regex() -> &'static Regex {
    URL_KEY_REGEX.get_or_init(|| {
        Regex::new(r"(?i)([?&](?:key|api_key|token|secret)=)[^&\s]+").expect("Valid regex")
    })
}

/// Che giấu chuỗi bí mật (API Key, Token) chỉ để lộ vài ký tự đầu và cuối
///
/// Ví dụ:
/// - `47e84a329ee0915f27fab83d9215d45b69c067eb2c5cf3a3` -> `"47e8••••cf3a3"`
/// - `sk-ckey-928374829349f2a` -> `"sk-c••••9f2a"`
pub fn mask_secret(secret: &str) -> String {
    let len = secret.len();
    if len <= 8 {
        return "••••••••".to_string();
    }
    let head = &secret[..4];
    let tail = &secret[len - 4..];
    format!("{head}••••{tail}")
}

/// Che giấu query parameter bí mật trong URL (?key=... hoặc &key=...)
///
/// Ví dụ:
/// - `https://ckey.vn/api/llm/models?key=47e84a329ee0915f` -> `https://ckey.vn/api/llm/models?key=***REDACTED***`
pub fn mask_url_secrets(url: &str) -> String {
    let re = get_url_key_regex();
    re.replace_all(url, "${1}***REDACTED***").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_secret() {
        let key = "47e84a329ee0915f27fab83d9215d45b69c067eb2c5cf3a3";
        assert_eq!(mask_secret(key), "47e8••••f3a3");
    }

    #[test]
    fn test_mask_url_secrets() {
        let url = "https://ckey.vn/api/llm/models?key=47e84a329ee0915f27fab83d9215d45b69c067eb2c5cf3a3&other=1";
        let masked = mask_url_secrets(url);
        assert_eq!(
            masked,
            "https://ckey.vn/api/llm/models?key=***REDACTED***&other=1"
        );
    }
}
