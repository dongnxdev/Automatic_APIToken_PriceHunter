//! CrawlerEngine Common Helpers

use rust_decimal::Decimal;
use shared_contract::SupportedPath;
use std::str::FromStr;

/// Parse một serde_json::Value (Number hoặc String) sang rust_decimal::Decimal chính xác
///
/// Dùng v.to_string() để đảm bảo chuyển qua string rồi parse vào Decimal, tránh sai số float!
pub fn parse_decimal_value(v: &serde_json::Value) -> Decimal {
    match v {
        serde_json::Value::Number(n) => {
            Decimal::from_str(&n.to_string()).unwrap_or(Decimal::ZERO)
        }
        serde_json::Value::String(s) => {
            Decimal::from_str(s.trim()).unwrap_or(Decimal::ZERO)
        }
        _ => Decimal::ZERO,
    }
}

/// Trích xuất seller_namespace từ public_name của Ckey
pub fn extract_seller_namespace(public_name: &str, is_provider_model: bool) -> String {
    if !is_provider_model {
        return "direct".to_string();
    }
    if let Some(pos) = public_name.find('/') {
        let prefix = &public_name[..pos].trim();
        if !prefix.is_empty() {
            return prefix.to_string();
        }
    }
    "community".to_string()
}

/// Kiểm tra xem model có thực sự là model Video hay không (loại bỏ nhiễu video_contract mặc định của Ckey)
pub fn is_media_video_model(
    paths: &[SupportedPath],
    price_per_second: Decimal,
    video_max_seconds: i32,
) -> bool {
    let has_video_path = paths.iter().any(|p| {
        matches!(
            p,
            SupportedPath::V1Videos
                | SupportedPath::V2VideoGeneration
        )
    });

    has_video_path || price_per_second > Decimal::ZERO || video_max_seconds > 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_decimal_value() {
        let val_num = serde_json::json!(474.5);
        assert_eq!(parse_decimal_value(&val_num), Decimal::from_str("474.5").unwrap());

        let val_str = serde_json::json!("323.4816");
        assert_eq!(parse_decimal_value(&val_str), Decimal::from_str("323.4816").unwrap());
    }

    #[test]
    fn test_extract_seller_namespace() {
        assert_eq!(extract_seller_namespace("rouyea98/claude-opus-4-6", true), "rouyea98");
        assert_eq!(extract_seller_namespace("claude-opus-4.6", false), "direct");
    }
}
