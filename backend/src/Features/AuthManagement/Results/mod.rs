//! AuthManagement Response Helpers

use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use shared_contract::ApiResponse;

pub fn json_response<T: serde::Serialize>(status: StatusCode, envelope: ApiResponse<T>) -> Response {
    (status, Json(envelope)).into_response()
}

pub fn login_cookie_header(token: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    let cookie_val = format!(
        "modelradar_session={token}; HttpOnly; SameSite=Strict; Path=/; Max-Age=86400"
    );
    if let Ok(hv) = HeaderValue::from_str(&cookie_val) {
        headers.insert(header::SET_COOKIE, hv);
    }
    headers
}
