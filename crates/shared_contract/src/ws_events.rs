//! Module Định Nghĩa Các Sự Kiện WebSocket (Real-time Stream Contracts)

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Sự kiện từ Backend đẩy xuống Frontend qua kết nối WebSocket /ws/ops
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum WsServerEvent {
    /// Phản hồi ping để đo RTT độ trễ mạng (LIVE 18ms)
    HeartbeatPong {
        client_time_ms: i64,
        server_time_ms: i64,
    },
    /// Cập nhật đồng bộ các chỉ số App Shell (Badges, FX, Countdown)
    ShellStatusUpdate(ShellStatusDto),
    /// Dòng log hệ thống thời gian thực (đã che sạch secret)
    OpsLogLine(OpsLogLineDto),
    /// Thông báo biến động giá của một model chuẩn
    PriceRadarUpdated {
        canonical_id: String,
        top_seller: String,
        new_blended_vnd: Decimal,
        diff_percent: Option<Decimal>,
    },
    /// Thông báo cảnh báo được kích hoạt
    AlertTriggered {
        rule_name: String,
        message: String,
        severity: String,
    },
}

/// Lệnh từ Frontend gửi lên Backend qua WebSocket
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "action", content = "payload")]
pub enum WsClientCommand {
    /// Ping đo độ trễ
    HeartbeatPing {
        client_time_ms: i64,
    },
    /// Đăng ký bộ lọc log thời gian thực
    FilterOpsLogs {
        min_level: Option<String>,
        seller_namespace: Option<String>,
    },
}

/// Trạng thái tổng quan của App Shell
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShellStatusDto {
    pub is_auto_scan_enabled: bool,
    pub next_scan_epoch_ms: i64,
    pub fx_rate_vnd_per_usd: Decimal,
    pub online_sources_count: u32,
    pub total_sources_count: u32,
    pub active_sellers_count: u32,
    pub provider_errors_count: u32,
    pub unmapped_models_count: u32,
    pub unread_alerts_count: u32,
}

/// Dòng log hiển thị tại Live Terminal (Màn 6)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpsLogLineDto {
    pub timestamp_iso: String,
    pub level: String,
    pub target: String,
    pub message: String,
}
