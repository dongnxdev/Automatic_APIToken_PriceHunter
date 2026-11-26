//! Module Cấu Hình Hệ Thống (Config Loader & Fail-Fast Validator)

use crate::Shared::Common::crypto::derive_master_key;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;

#[derive(Debug)]
pub enum ConfigError {
    FileNotFound(String),
    ParseError(String),
    ValidationError(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FileNotFound(p) => write!(f, "Không tìm thấy file cấu hình: {p}"),
            Self::ParseError(msg) => write!(f, "Lỗi phân tích cú pháp JSON cấu hình: {msg}"),
            Self::ValidationError(msg) => write!(f, "Lỗi kiểm định cấu hình (Fail-fast): {msg}"),
        }
    }
}

impl std::error::Error for ConfigError {}

/// Cấu hình máy chủ HTTP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub public_base_url: String,
}

/// Cấu hình tài khoản quản trị Management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagementConfig {
    pub username: String,
    pub password_hash: String,
    pub session_ttl_seconds: u64,
}

/// Cấu hình mặc định tỷ giá ngoại tệ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FxDefaultsConfig {
    pub usd_to_vnd_rate: Decimal,
    pub default_display_currency: String,
}

/// Cấu hình bộ quét Crawler
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlerConfig {
    pub auto_scan_enabled: bool,
    pub default_interval_seconds: u64,
    pub request_timeout_ms: u64,
    pub max_concurrent_providers: usize,
    pub queue_capacity: usize,
}

/// Cấu hình Cờ tính năng (Feature Flags)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlagsConfig {
    pub enable_auto_scan: bool,
    pub enable_smart_gateway: bool,
    pub enable_live_tester: bool,
    pub enable_alert_dispatcher: bool,
    pub enable_price_circuit_breaker: bool,
}

/// Cấu hình một Provider khởi tạo ban đầu
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfigDto {
    pub name: String,
    pub adapter_type: String,
    pub auth_mode: String,
    pub query_param_name: String,
    pub api_key: String,
    pub endpoint: String,
    pub inference_base_url: String,
    pub fx_currency: String,
}

/// Cấu hình ứng dụng toàn cục
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub management: ManagementConfig,
    pub fx_defaults: FxDefaultsConfig,
    pub crawler: CrawlerConfig,
    pub feature_flags: FeatureFlagsConfig,
    pub providers: Vec<ProviderConfigDto>,
    pub master_key: [u8; 32],
}

impl AppConfig {
    /// Đọc và kiểm định cấu hình từ file appsettings.json
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let path_ref = path.as_ref();
        if !path_ref.exists() {
            return Err(ConfigError::FileNotFound(path_ref.display().to_string()));
        }

        let content = std::fs::read_to_string(path_ref)
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;

        // Dùng serde_json::Value để hỗ trợ cả PascalCase trong Readme.md lẫn snake_case
        let root: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;

        // 1. Server config
        let server_val = root.get("server").or_else(|| root.get("Server"));
        let host = server_val
            .and_then(|s| s.get("host").or_else(|| s.get("Host")))
            .and_then(|v| v.as_str())
            .unwrap_or("0.0.0.0")
            .to_string();

        let port = server_val
            .and_then(|s| s.get("port").or_else(|| s.get("Port")))
            .and_then(|v| v.as_u64())
            .unwrap_or(1098) as u16;

        if port == 0 {
            return Err(ConfigError::ValidationError("Server.Port không được bằng 0".into()));
        }

        let database_url = server_val
            .and_then(|s| s.get("database_url").or_else(|| s.get("DatabaseUrl")))
            .and_then(|v| v.as_str())
            .unwrap_or("sqlite://data/modelradar.db?mode=rwc")
            .to_string();

        let public_base_url = server_val
            .and_then(|s| s.get("public_base_url").or_else(|| s.get("PublicBaseUrl")))
            .and_then(|v| v.as_str())
            .unwrap_or("http://localhost:1098")
            .to_string();

        // 2. Management config
        let mgmt_val = root.get("management").or_else(|| root.get("Management"));
        let username = mgmt_val
            .and_then(|m| m.get("username").or_else(|| m.get("Username")))
            .and_then(|v| v.as_str())
            .unwrap_or("admin")
            .trim()
            .to_string();

        if username.is_empty() {
            return Err(ConfigError::ValidationError("Management.Username không được để trống".into()));
        }

        let raw_pass = mgmt_val
            .and_then(|m| {
                m.get("password_hash")
                    .or_else(|| m.get("PasswordHash"))
                    .or_else(|| m.get("password"))
                    .or_else(|| m.get("Password"))
            })
            .and_then(|v| v.as_str())
            .unwrap_or("admin123456")
            .to_string();

        // 3. FX defaults
        let fx_val = root.get("fx_defaults").or_else(|| root.get("FxDefaults"));
        let usd_to_vnd_rate = fx_val
            .and_then(|f| f.get("usd_to_vnd_rate"))
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<Decimal>().ok())
            .unwrap_or_else(|| Decimal::new(25450, 0));

        // 4. Crawler config
        let crawl_val = root.get("crawler").or_else(|| root.get("Crawler"));
        let auto_scan_enabled = crawl_val
            .and_then(|c| c.get("auto_scan_enabled"))
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let default_interval_seconds = crawl_val
            .and_then(|c| c.get("default_interval_seconds"))
            .and_then(|v| v.as_u64())
            .unwrap_or(180);

        let request_timeout_ms = crawl_val
            .and_then(|c| c.get("request_timeout_ms"))
            .and_then(|v| v.as_u64())
            .unwrap_or(15000);

        // 5. Feature Flags
        let flags_val = root.get("feature_flags").or_else(|| root.get("FeatureFlags"));
        let feature_flags = FeatureFlagsConfig {
            enable_auto_scan: flags_val
                .and_then(|f| f.get("enable_auto_scan"))
                .and_then(|v| v.as_bool())
                .unwrap_or(true),
            enable_smart_gateway: flags_val
                .and_then(|f| f.get("enable_smart_gateway"))
                .and_then(|v| v.as_bool())
                .unwrap_or(true),
            enable_live_tester: flags_val
                .and_then(|f| f.get("enable_live_tester"))
                .and_then(|v| v.as_bool())
                .unwrap_or(true),
            enable_alert_dispatcher: flags_val
                .and_then(|f| f.get("enable_alert_dispatcher"))
                .and_then(|v| v.as_bool())
                .unwrap_or(true),
            enable_price_circuit_breaker: flags_val
                .and_then(|f| f.get("enable_price_circuit_breaker"))
                .and_then(|v| v.as_bool())
                .unwrap_or(true),
        };

        // 6. Providers parser (Tương thích Readme.md và bóc tách ?key=)
        let mut providers = Vec::new();
        if let Some(arr) = root.get("provider").or_else(|| root.get("Provider")).and_then(|v| v.as_array()) {
            for item in arr {
                let name = item
                    .get("name")
                    .or_else(|| item.get("Name"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("Ckey")
                    .to_string();

                let mut raw_endpoint = item
                    .get("endpoint")
                    .or_else(|| item.get("Endpoint"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("https://ckey.vn/api/llm/models")
                    .to_string();

                let mut api_key = item
                    .get("api_key")
                    .or_else(|| item.get("API_Key"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                // Tự động bóc tách ?key= nếu người dùng dán URL nguyên khối
                if let Some(pos) = raw_endpoint.find("?key=") {
                    let extracted_key = &raw_endpoint[pos + 5..];
                    if !extracted_key.is_empty() && api_key.is_empty() {
                        api_key = extracted_key.to_string();
                    }
                    raw_endpoint = raw_endpoint[..pos].to_string();
                }

                providers.push(ProviderConfigDto {
                    name,
                    adapter_type: "CKEY_LLM".into(),
                    auth_mode: "QUERY_KEY".into(),
                    query_param_name: "key".into(),
                    api_key,
                    endpoint: raw_endpoint,
                    inference_base_url: "https://ckey.vn/v1".into(),
                    fx_currency: "VND".into(),
                });
            }
        }

        // Master key từ env hoặc fallback an toàn
        let master_key_raw = std::env::var("MODELRADAR_MASTER_KEY")
            .unwrap_or_else(|_| "modelradar-local-dev-master-key-2026-safe".to_string());
        let master_key = derive_master_key(&master_key_raw);

        Ok(Self {
            server: ServerConfig {
                host,
                port,
                database_url,
                public_base_url,
            },
            management: ManagementConfig {
                username,
                password_hash: raw_pass,
                session_ttl_seconds: 86400,
            },
            fx_defaults: FxDefaultsConfig {
                usd_to_vnd_rate,
                default_display_currency: "VND".into(),
            },
            crawler: CrawlerConfig {
                auto_scan_enabled,
                default_interval_seconds,
                request_timeout_ms,
                max_concurrent_providers: 4,
                queue_capacity: 256,
            },
            feature_flags,
            providers,
            master_key,
        })
    }
}
