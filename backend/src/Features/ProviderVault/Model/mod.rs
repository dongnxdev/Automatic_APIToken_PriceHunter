//! ProviderVault Data Transfer Objects

use serde::{Deserialize, Serialize};
use shared_contract::NormalizedCkeyModel;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderDto {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub adapter_type: String,
    pub auth_mode: String,
    pub auth_param_name: String,
    pub models_endpoint_url: String,
    pub inference_base_url: String,
    pub api_key_masked: String,
    pub has_proxy: bool,
    pub fx_currency: String,
    pub fx_rate_to_vnd_decimal: String,
    pub price_multiplier_decimal: String,
    pub scan_interval_seconds: i64,
    pub is_enabled: bool,
    pub status: String,
    pub last_ping_ms: Option<i64>,
    pub last_model_count: i64,
    pub last_error_message: Option<String>,
    pub last_scanned_at_ms: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateProviderRequest {
    pub name: String,
    pub adapter_type: Option<String>,
    pub auth_mode: Option<String>,
    pub auth_param_name: Option<String>,
    pub models_endpoint_url: String,
    pub inference_base_url: Option<String>,
    pub api_key: String,
    pub proxy_url: Option<String>,
    pub fx_currency: Option<String>,
    pub fx_rate_to_vnd_decimal: Option<String>,
    pub price_multiplier_decimal: Option<String>,
    pub scan_interval_seconds: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProviderRequest {
    pub name: Option<String>,
    pub adapter_type: Option<String>,
    pub auth_mode: Option<String>,
    pub auth_param_name: Option<String>,
    pub models_endpoint_url: Option<String>,
    pub inference_base_url: Option<String>,
    pub api_key: Option<String>,
    pub proxy_url: Option<String>,
    pub fx_currency: Option<String>,
    pub fx_rate_to_vnd_decimal: Option<String>,
    pub price_multiplier_decimal: Option<String>,
    pub scan_interval_seconds: Option<i64>,
    pub is_enabled: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct RevealSecretResponse {
    pub provider_id: String,
    pub api_key_plaintext: String,
    pub proxy_url_plaintext: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SandboxParseRequest {
    pub adapter_type: String,
    pub auth_mode: String,
    pub auth_param_name: Option<String>,
    pub endpoint_url: String,
    pub api_key: String,
    pub proxy_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SandboxParseResponse {
    pub http_status: u16,
    pub ping_ms: i64,
    pub total_models_found: usize,
    pub raw_sample_json: serde_json::Value,
    pub normalized_preview: Vec<NormalizedCkeyModel>,
    pub parser_warnings: Vec<String>,
}
