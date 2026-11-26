//! ProviderVault Common Utilities

use crate::Shared::Utils::normalize_lookup_key;

/// Tự động tách query parameter khỏi endpoint nếu người dùng lỡ dán URL kèm ?key=...
pub fn clean_endpoint_and_extract_key(endpoint: &str, param_name: &str) -> (String, Option<String>) {
    let pattern = format!("?{param_name}=");
    let pattern_amp = format!("&{param_name}=");

    if let Some(pos) = endpoint.find(&pattern) {
        let clean = endpoint[..pos].to_string();
        let rest = &endpoint[pos + pattern.len()..];
        let key = rest.split('&').next().unwrap_or("").to_string();
        (clean, if key.is_empty() { None } else { Some(key) })
    } else if let Some(pos) = endpoint.find(&pattern_amp) {
        let clean = endpoint[..pos].to_string();
        let rest = &endpoint[pos + pattern_amp.len()..];
        let key = rest.split('&').next().unwrap_or("").to_string();
        (clean, if key.is_empty() { None } else { Some(key) })
    } else {
        (endpoint.trim().to_string(), None)
    }
}

/// Tạo slug an toàn cho provider từ tên
pub fn slugify(name: &str) -> String {
    let lower = normalize_lookup_key(name);
    let mut slug = String::new();
    let mut prev_dash = false;
    for c in lower.chars() {
        if c.is_alphanumeric() {
            slug.push(c);
            prev_dash = false;
        } else if !prev_dash {
            slug.push('-');
            prev_dash = true;
        }
    }
    slug.trim_matches('-').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_endpoint_and_extract_key() {
        let url = "https://ckey.vn/api/llm/models?key=47e84a329ee0915f27fab83d9215d45b";
        let (clean, key) = clean_endpoint_and_extract_key(url, "key");
        assert_eq!(clean, "https://ckey.vn/api/llm/models");
        assert_eq!(key, Some("47e84a329ee0915f27fab83d9215d45b".to_string()));
    }

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Ckey.vn Pro"), "ckey-vn-pro");
        assert_eq!(slugify("  OpenRouter AI  "), "openrouter-ai");
    }
}
