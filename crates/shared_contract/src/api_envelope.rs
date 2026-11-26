//! Module Chuẩn Hóa API Envelope
//!
//! Quy chuẩn hệ thống: Toàn bộ API quản trị trả về đúng một bọc thống nhất (Unified Envelope).

use chrono::Utc;
use serde::{Deserialize, Serialize};

/// Cấu trúc bọc API thống nhất toàn hệ thống
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    /// Trạng thái thành công hay thất bại
    pub success: bool,
    /// Dữ liệu trả về khi thành công (null khi có lỗi)
    pub data: Option<T>,
    /// Chi tiết lỗi khi thất bại (null khi thành công)
    pub error: Option<ApiError>,
    /// Metadata đi kèm mỗi request (UUIDv7 + Timestamp UTC ISO-8601)
    pub meta: ApiMeta,
}

impl<T> ApiResponse<T> {
    /// Tạo response thành công
    pub fn ok(data: T, request_id: impl Into<String>) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            meta: ApiMeta::new(request_id),
        }
    }

    /// Tạo response lỗi
    pub fn err(code: impl Into<String>, message: impl Into<String>, request_id: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(ApiError {
                code: code.into(),
                message: message.into(),
                details: None,
            }),
            meta: ApiMeta::new(request_id),
        }
    }

    /// Tạo response lỗi có chi tiết bổ sung
    pub fn err_with_details(
        code: impl Into<String>,
        message: impl Into<String>,
        details: serde_json::Value,
        request_id: impl Into<String>,
    ) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(ApiError {
                code: code.into(),
                message: message.into(),
                details: Some(details),
            }),
            meta: ApiMeta::new(request_id),
        }
    }
}

/// Chi tiết lỗi chuẩn hóa
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiError {
    /// Mã lỗi tường minh dạng UPPER_SNAKE_CASE (VD: UNAUTHORIZED, VALIDATION_ERROR, PROVIDER_TIMEOUT)
    pub code: String,
    /// Thông điệp lỗi chi tiết cho con người
    pub message: String,
    /// Dữ liệu chi tiết bổ sung (tùy chọn)
    pub details: Option<serde_json::Value>,
}

/// Metadata đi kèm request
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiMeta {
    /// Định danh duy nhất của request (UUIDv7)
    pub request_id: String,
    /// Thời điểm sinh response (UTC ISO-8601 chuẩn)
    pub timestamp: String,
}

impl ApiMeta {
    /// Khởi tạo ApiMeta với request_id và thời gian UTC hiện tại
    pub fn new(request_id: impl Into<String>) -> Self {
        Self {
            request_id: request_id.into(),
            timestamp: Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        }
    }
}
