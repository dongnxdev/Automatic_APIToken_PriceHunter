//! ModelRadar Shared Contract Crate
//!
//! Dùng chung giữa Rust Backend native và Leptos WASM Frontend.
//! Chốt cứng nguyên tắc Exact Decimal (Zero Rounding) theo Ckey.vn.

pub mod api_envelope;
pub mod currency_math;
pub mod enums;
pub mod model_capabilities;
pub mod ws_events;

pub use api_envelope::{ApiError, ApiMeta, ApiResponse};
pub use currency_math::{
    format_usd_exact, format_vnd_exact, ExactPrice, WorkloadProfile,
};
pub use enums::{
    AdapterType, AlertRuleType, AuthMode, DeliveryStatus, DisplayCurrency, LiveTestStatus,
    MappingStatus, Modality, PricingModelKind, ProviderStatus, RoutingMode,
};
pub use model_capabilities::{
    NormalizedCkeyModel, SupportedPath, VideoContract, VideoRateCard,
};
pub use ws_events::{
    OpsLogLineDto, ShellStatusDto, WsClientCommand, WsServerEvent,
};
