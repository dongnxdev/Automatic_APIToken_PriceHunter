//! Module Tiện Ích Thời Gian & Chuẩn Hóa Văn Bản (Single Canonical Text & Time)

use chrono::Utc;
use unicode_normalization::UnicodeNormalization;

/// Lấy thời điểm hiện tại dạng Unix Epoch Milliseconds (i64, 13 chữ số UTC)
#[inline]
pub fn now_utc_ms() -> i64 {
    Utc::now().timestamp_millis()
}

/// Lấy thời điểm hiện tại dạng chuỗi ISO-8601 UTC (YYYY-MM-DDTHH:mm:ss.sssZ)
#[inline]
pub fn now_utc_iso() -> String {
    Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

/// Chuẩn hóa chuỗi văn bản: NFC + trim khoảng trắng 2 đầu.
///
/// Nếu chuỗi rỗng hoặc chỉ toàn khoảng trắng thì ép về None (tương đương null).
pub fn normalize_nfc_trim(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }
    let nfc: String = trimmed.nfc().collect();
    if nfc.is_empty() {
        None
    } else {
        Some(nfc)
    }
}

/// Chuẩn hóa khóa tra cứu (Lookup Key): NFC + trim + lowercase.
///
/// Dùng cho canonical_id, slug, lookup_key, định danh không phân biệt hoa thường.
pub fn normalize_lookup_key(input: &str) -> String {
    let trimmed = input.trim();
    let nfc: String = trimmed.nfc().collect();
    nfc.to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_nfc_trim() {
        assert_eq!(normalize_nfc_trim("   "), None);
        assert_eq!(normalize_nfc_trim(""), None);
        assert_eq!(
            normalize_nfc_trim("  Claude Opus 4.6  "),
            Some("Claude Opus 4.6".to_string())
        );
    }

    #[test]
    fn test_normalize_lookup_key() {
        assert_eq!(
            normalize_lookup_key("  Claude-Opus-4.6  "),
            "claude-opus-4.6"
        );
        assert_eq!(
            normalize_lookup_key("DeepSeek-R1-0528"),
            "deepseek-r1-0528"
        );
    }

    #[test]
    fn test_now_utc() {
        assert!(now_utc_ms() > 0);
        assert!(now_utc_iso().ends_with('Z'));
    }
}
