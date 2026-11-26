//! Module Năng Lực Kỹ Thuật Model & Schema Dữ Liệu Khớp 100% Ckey.vn

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Danh sách các endpoint path được hỗ trợ (khớp 100% danh mục thực tế từ Ckey)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SupportedPath {
    #[serde(rename = "chat/completions")]
    ChatCompletions,
    #[serde(rename = "completions")]
    Completions,
    #[serde(rename = "messages")]
    Messages,
    #[serde(rename = "responses")]
    Responses,
    #[serde(rename = "gemini")]
    Gemini,
    #[serde(rename = "ollama")]
    Ollama,
    #[serde(rename = "api/generate")]
    ApiGenerate,
    #[serde(rename = "images/generations")]
    ImagesGenerations,
    #[serde(rename = "images/edits")]
    ImagesEdits,
    #[serde(rename = "audio/speech")]
    AudioSpeech,
    #[serde(rename = "embeddings")]
    Embeddings,
    #[serde(rename = "v1/videos")]
    V1Videos,
    #[serde(rename = "v2/video_generation")]
    V2VideoGeneration,
    /// Trường hợp sàn có endpoint mới chưa khai báo
    #[serde(untagged)]
    Other(String),
}

impl SupportedPath {
    pub fn as_str(&self) -> &str {
        match self {
            Self::ChatCompletions => "chat/completions",
            Self::Completions => "completions",
            Self::Messages => "messages",
            Self::Responses => "responses",
            Self::Gemini => "gemini",
            Self::Ollama => "ollama",
            Self::ApiGenerate => "api/generate",
            Self::ImagesGenerations => "images/generations",
            Self::ImagesEdits => "images/edits",
            Self::AudioSpeech => "audio/speech",
            Self::Embeddings => "embeddings",
            Self::V1Videos => "v1/videos",
            Self::V2VideoGeneration => "v2/video_generation",
            Self::Other(s) => s.as_str(),
        }
    }
}

/// Hợp đồng đặc tả gọi API Video (khớp video_contract trong Ckey)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VideoContract {
    #[serde(default)]
    pub protocol: String,
    #[serde(default)]
    pub duration_field: String,
    #[serde(default)]
    pub durations: Vec<i32>,
    #[serde(default)]
    pub max_seconds: i32,
    #[serde(default)]
    pub default_seconds: i32,
    #[serde(default)]
    pub async_mode: bool,
    #[serde(default)]
    pub tier_field: String,
    #[serde(default)]
    pub tiers: Vec<String>,
    #[serde(default)]
    pub sample_body: Option<serde_json::Value>,
    #[serde(default)]
    pub body_style: String,
}

/// Biểu giá chi tiết của Video theo độ phân giải/thời lượng
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VideoRateCard {
    pub resolution: String,
    pub fps: i32,
    pub has_audio: bool,
    pub price_vnd_per_unit: Decimal,
}

/// Model đã được chuẩn hóa từ nguồn Ckey (chứa đầy đủ 19 trường thực tế)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizedCkeyModel {
    pub public_name: String,
    pub display_name: String,
    pub seller_namespace: String,
    pub is_provider_model: bool,
    pub input_price_per_million_vnd: Decimal,
    pub output_price_per_million_vnd: Decimal,
    pub cache_enabled: bool,
    pub cache_read_price_per_million_vnd: Decimal,
    pub cache_write_price_per_million_vnd: Decimal,
    pub price_per_request_vnd: Decimal,
    pub min_charge_per_request_vnd: Decimal,
    pub price_per_second_vnd: Decimal,
    pub request_rate_limit_per_minute: i64,
    pub max_output_tokens_limit: i64,
    pub context_tokens_limit: i64,
    pub supported_paths: Vec<SupportedPath>,
    pub video_durations_allowed: Vec<i32>,
    pub video_max_seconds: i32,
    pub video_rate_cards: Option<Vec<VideoRateCard>>,
    pub video_contract: Option<VideoContract>,
    pub is_media_video_capable: bool,
}
