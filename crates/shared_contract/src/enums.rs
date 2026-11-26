//! Module Định Nghĩa Các Enum Chuẩn Toàn Hệ Thống (Single Canonical Enums)

use serde::{Deserialize, Serialize};

/// Loại Adapter kết nối tới nguồn cung cấp
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AdapterType {
    /// Adapter chuyên biệt cho nền tảng Ckey.vn
    CkeyLlm,
    /// Adapter tương thích chuẩn OpenAI (/v1/models)
    OpenAiCompat,
    /// Adapter tùy biến dựa trên JSON Pointer
    CustomJson,
}

impl AdapterType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CkeyLlm => "CKEY_LLM",
            Self::OpenAiCompat => "OPENAI_COMPAT",
            Self::CustomJson => "CUSTOM_JSON",
        }
    }
}

/// Chế độ truyền xác thực API Key
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuthMode {
    /// Truyền key qua query parameter trên URL (?key=...)
    QueryKey,
    /// Truyền key qua header Authorization: Bearer <key>
    BearerHeader,
    /// Truyền key qua header x-api-key: <key>
    XApiKeyHeader,
}

/// Loại mô hình tính cước giá
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PricingModelKind {
    /// Tính theo số lượng token Input/Output/Cache
    PerToken,
    /// Thu phí khoán cố định theo mỗi request (in=0, out=0, req>0)
    FlatPerRequest,
    /// Tính theo số giây tạo video
    PerSecondVideo,
}

/// Trạng thái hoạt động của một Provider nguồn
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProviderStatus {
    Online,
    Unauthorized401,
    RateLimited429,
    Timeout,
    CircuitOpen,
    Disabled,
}

impl ProviderStatus {
    pub fn is_online(&self) -> bool {
        matches!(self, Self::Online)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Online => "ONLINE",
            Self::Unauthorized401 => "UNAUTHORIZED_401",
            Self::RateLimited429 => "RATE_LIMITED_429",
            Self::Timeout => "TIMEOUT",
            Self::CircuitOpen => "CIRCUIT_OPEN",
            Self::Disabled => "DISABLED",
        }
    }
}

/// Chế độ định tuyến của Smart Gateway
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RoutingMode {
    /// Luôn chọn kênh bán có giá Blended rẻ nhất
    Cheapest,
    /// Chọn kênh rẻ nhất trong số các kênh có ping < max_latency_ms
    CheapestWithLatencyCap,
    /// Ghim cố định vào một kênh bán/provider cụ thể
    PinnedProvider,
}

/// Loại luật báo động
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AlertRuleType {
    /// Giá model giảm dưới ngưỡng cài đặt
    PriceBelow,
    /// Provider/Kênh bán bị lỗi liên tiếp N lần
    ProviderFailureStreak,
    /// Provider bị lỗi xác thực 401
    ProviderAuth401,
    /// Phát hiện model mới tinh chưa được map
    ModelUnmapped,
    /// Giá bất ngờ tăng vọt
    PriceSpike,
}

/// Trạng thái gửi tin cảnh báo
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DeliveryStatus {
    Sent,
    Retrying,
    DeadLetter,
}

/// Trạng thái kiểm chứng Test Sống (Live Ping & Token Tester)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LiveTestStatus {
    /// Model trả về khớp hoàn toàn với tên model yêu cầu
    Matched,
    /// Model trả về bị lệch (treo đầu dê bán thịt chó)
    Mismatched,
    /// Request gọi thất bại (lỗi mạng / timeout / HTTP lỗi)
    Failed,
}

/// Trạng thái ánh xạ tên model
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MappingStatus {
    AutoMapped,
    ManualMapped,
    Unmapped,
    Blacklisted,
}

/// Tiền tệ hiển thị trên giao diện người dùng
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum DisplayCurrency {
    #[default]
    Vnd,
    Usd,
}

/// Phương thức đa phương tiện của model
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Modality {
    Text,
    Image,
    Audio,
    Video,
    Embedding,
}
