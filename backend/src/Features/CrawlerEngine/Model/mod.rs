//! Raw API Payload Models Khớp 100% Ckey.vn

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct CkeyApiResponseRaw {
    pub success: bool,
    pub status: u16,
    pub message: String,
    pub data: CkeyDataRaw,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CkeyDataRaw {
    pub count: Option<usize>,
    pub models: Vec<CkeyModelRaw>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CkeyModelRaw {
    pub public_name: String,
    pub display_name: String,
    pub is_provider_model: bool,
    pub input_price_per_million_vnd: serde_json::Value,
    pub output_price_per_million_vnd: serde_json::Value,
    pub price_per_request_vnd: serde_json::Value,
    pub price_per_second_vnd: serde_json::Value,
    pub min_charge_per_request_vnd: serde_json::Value,
    pub cache_enabled: bool,
    pub cache_read_price_per_million_vnd: serde_json::Value,
    pub cache_write_price_per_million_vnd: serde_json::Value,
    #[serde(default)]
    pub request_rate_limit_per_minute: Option<i64>,
    #[serde(default)]
    pub max_output_tokens_limit: Option<i64>,
    #[serde(default)]
    pub context_tokens_limit: Option<i64>,
    #[serde(default)]
    pub supported_paths: Vec<String>,
    #[serde(default)]
    pub video_durations_allowed: Option<Vec<i32>>,
    #[serde(default)]
    pub video_max_seconds: Option<i32>,
    #[serde(default)]
    pub video_rate_cards: Option<serde_json::Value>,
    #[serde(default)]
    pub video_contract: Option<serde_json::Value>,
}
